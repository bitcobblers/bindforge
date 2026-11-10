import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, unlinkSync, writeFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, relative, resolve } from 'node:path';

import type { OpenCodeClient, SessionMessageAssistant, SessionMessageInfo } from '@opencode/client';
import { z } from 'zod';

import {
  TEST_PATH_RE,
  allReviewsApproved,
  flipAdrStatus,
  isSemanticCommit,
  lastSeatReply,
  testsGreen as runChecks,
  sourceAdmissible,
  stagePathAllowed,
} from './gates.js';
import { type IdentityPin, type RunIdentity, deriveIdentity, identityClash } from './identity.js';
import {
  type PassPlan,
  hasSeats,
  planPass,
  recordSeatVerdict,
  reportPathFor,
  reviewSessionsUsed,
  seatDecision,
  seatSummary,
} from './panel.js';
import {
  type ModelInfo,
  type TemplateVars,
  extractJsonObject,
  loadTemplate,
  normalizeReplyShape,
  renderTemplate,
  templateHash,
  writeTranscript,
} from './prompts.js';
import { escalationMarkdown, escalationPhase, proposalGrowthLine } from './report.js';
import {
  DEFAULT_CONFIG,
  DEFAULT_MIN_NUDGE_INTERVAL_MS,
  DEFAULT_REQUEST_TIMEOUT_MS,
  assertForceableStage,
  stageDef,
  stageReplySchema,
} from './stages.js';
import type { PipelineConfig } from './stages.js';
import { designDirOf, loadState, saveState } from './state-store.js';
import {
  type ReviewPhase,
  decideNext,
  gateName,
  reviewBudgetUsed,
  specialtiesFor,
} from './states.js';
import type { LockInfo, PipelineState, StageResult } from './types.js';

// ---------------------------------------------------------------------------
// The sequential inference lane. Exactly one stage session runs at a time;
// every stage gets a fresh top-level session bound to its stage agent.
// ---------------------------------------------------------------------------

export interface RunOptions {
  repoDir: string;
  sourcePath: string;
  promptsDir: string;
  client: OpenCodeClient;
  cfg?: PipelineConfig;
  forceStage?: string;
  /** Grant this many review rounds beyond the cap, recorded as operator consent. */
  extendBudget?: number;
  /** Pin the run identity instead of deriving it from the source file name. */
  pin?: IdentityPin;
  /** Start even when the identity's branch or design directory is taken. */
  force?: boolean;
}

export interface ResumeOptions {
  repoDir: string;
  slug: string;
  promptsDir: string;
  client: OpenCodeClient;
  cfg?: PipelineConfig;
  forceStage?: string;
  /** Grant this many review rounds beyond the cap, recorded as operator consent. */
  extendBudget?: number;
}

const LOCK_FILE = '.pipeline.lock';

// --- git helpers (the runner is the only committer; models never touch git) --

function git(repoDir: string, args: string[]): string {
  const res = spawnSync('git', args, { cwd: repoDir, encoding: 'utf8' });
  if (res.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed: ${res.stderr.trim()}`);
  }
  return res.stdout;
}

/** Stages whose product is `02-proposal.md`; only they get the size meter. */
const DESIGN_DOC_STAGES = new Set(['proposal', 'proposal-revise', 'proposal-final']);

/**
 * Size meter for the design document: its line count and the worktree-vs-HEAD
 * delta that produced it, measured after the stage's inference phase and
 * before its commit. Measured here rather than asked of the stage, because the
 * run that grew 6x was written by stages that each reported closing everything.
 */
function proposalSize(
  repoDir: string,
  designDir: string,
): { docLines: number; added: number; removed: number } | undefined {
  const doc = join(designDir, '02-proposal.md');
  if (!existsSync(doc)) {
    return undefined;
  }
  const text = readFileSync(doc, 'utf8');
  // Same count as `wc -l`: a trailing newline does not start a new line.
  const docLines = text.split('\n').length - (text.endsWith('\n') ? 1 : 0);
  const rel = relative(repoDir, doc).split(/[\\/]/).join('/');
  if (!gitOk(repoDir, ['cat-file', '-e', `HEAD:${rel}`])) {
    return { docLines, added: docLines, removed: 0 };
  }
  const [, added = '0', removed = '0'] =
    /^(\d+)\t(\d+)\t/.exec(git(repoDir, ['diff', '--numstat', 'HEAD', '--', rel])) ?? [];
  return { docLines, added: Number(added), removed: Number(removed) };
}

function gitOk(repoDir: string, args: string[]): boolean {
  return spawnSync('git', args, { cwd: repoDir }).status === 0;
}

/**
 * The revision stamp a funnel verdict is pinned to: the blob id of the design
 * document itself (ADR-002). Empty when the document is not in HEAD yet —
 * equality still decides staleness, only the diff scope degrades.
 */
function blobStamp(repoDir: string, relPath: string): string {
  if (!gitOk(repoDir, ['cat-file', '-e', `HEAD:${relPath}`])) {
    return '';
  }
  return git(repoDir, ['rev-parse', `HEAD:${relPath}`]).trim();
}

/**
 * Paths out of a `--porcelain=v1 -z` listing. NUL separation means paths are
 * never quoted or escaped, `--no-renames` means a rename is never collapsed
 * into one pseudo-path (`old -> new`), and `--untracked-files=all` means an
 * untracked directory is listed file by file instead of as a single entry the
 * delete path then fails on with ERR_FS_EISDIR.
 */
export function porcelainPaths(porcelain: string): string[] {
  return porcelain
    .split('\0')
    .map((entry) => (entry.length > 3 ? entry.slice(3) : ''))
    .filter((p) => p.length > 0);
}

/** NUL-separated path lists that carry no status prefix (`diff-tree`). */
function nulPaths(raw: string): string[] {
  return raw.split('\0').filter((p) => p.length > 0);
}

function dirtyFiles(repoDir: string): string[] {
  return porcelainPaths(
    git(repoDir, ['status', '--porcelain=v1', '-z', '--untracked-files=all', '--no-renames']),
  );
}

export interface OwnershipReport {
  /** Paths reverted because the stage touched them outside its ownership. */
  reverted: string[];
  /**
   * Commits inside the stage window that the runner did not author. They are
   * named and left alone: the runner cannot prove who wrote them, and silently
   * reverting operator commits destroyed harness fixes during run 1.
   */
  foreign: Array<{ sha: string; subject: string }>;
}

/**
 * Deterministic ownership enforcement: revert every path the stage left
 * dirty, plus every path changed by a commit the runner itself authored inside
 * the stage window. Models never commit (no agent has git access), so anything
 * else in the range is foreign work: reported, never destroyed. Run 1's
 * version diffed `headBefore..HEAD` and blamed the stage for everything in the
 * range, which reverted an operator commit landed mid-stage and deleted files
 * it never named in the log.
 */
export function enforceOwnership(
  repoDir: string,
  headBefore: string,
  stage: string,
  state: PipelineState,
): OwnershipReport {
  const own = state.stageCommits ?? [];
  const foreign: Array<{ sha: string; subject: string }> = [];
  const committed: string[] = [];
  if (git(repoDir, ['rev-parse', 'HEAD']).trim() !== headBefore) {
    for (const sha of git(repoDir, ['rev-list', `${headBefore}..HEAD`])
      .split('\n')
      .filter((l) => l.length > 0)) {
      if (own.includes(sha)) {
        committed.push(
          ...nulPaths(
            git(repoDir, ['diff-tree', '-r', '--no-commit-id', '-z', '--name-only', sha]),
          ),
        );
      } else {
        foreign.push({
          sha,
          subject: git(repoDir, ['log', '-1', '--format=%s', sha]).trim(),
        });
      }
    }
  }
  const touched = new Set([...committed, ...dirtyFiles(repoDir)]);
  const violations = [...touched].filter((p) => !stagePathAllowed(stage, p));
  const reverted: string[] = [];
  for (const p of violations) {
    if (gitOk(repoDir, ['cat-file', '-e', `${headBefore}:${p}`])) {
      git(repoDir, ['restore', `--source=${headBefore}`, '--staged', '--worktree', '--', p]);
    } else {
      gitOk(repoDir, ['rm', '-f', '--', p]);
      try {
        rmSync(resolve(repoDir, p), { force: true });
      } catch (error) {
        // Deleting an out-of-ownership path must never take the run down.
        console.warn(`ownership: could not remove ${p}: ${(error as Error).message}`);
      }
    }
    reverted.push(p);
  }
  if (reverted.length > 0 && git(repoDir, ['rev-parse', 'HEAD']).trim() !== headBefore) {
    // Committed violations: revert commit so the committed tree honors
    // ownership too. The paths go in the commit body: a count alone made the
    // 10:53 event in run 1 unreconstructable.
    git(repoDir, ['add', '-A']);
    if (!gitOk(repoDir, ['diff', '--cached', '--quiet'])) {
      git(repoDir, [
        'commit',
        '-m',
        `fix: revert ${stage} stage changes outside its ownership`,
        '-m',
        `reverted paths: ${reverted.join(', ')}`,
      ]);
      recordStageCommit(state, git(repoDir, ['rev-parse', 'HEAD']).trim());
    }
  }
  return { reverted, foreign };
}

/** Commit exactly the current stage's changes; model message or fallback. */
function commitStage(
  repoDir: string,
  slug: string,
  stageKey: string,
  fallbackType: string,
  reply: unknown,
): string | null {
  if (dirtyFiles(repoDir).length === 0) {
    return null;
  }
  git(repoDir, ['add', '-A']);
  if (gitOk(repoDir, ['diff', '--cached', '--quiet'])) {
    return null;
  }
  const proposed = (reply as { commitMessage?: string } | undefined)?.commitMessage;
  const message =
    proposed && isSemanticCommit(proposed)
      ? proposed
      : `${fallbackType}: ${slug} ${stageKey} stage`;
  git(repoDir, ['commit', '-m', message]);
  return git(repoDir, ['rev-parse', 'HEAD']).trim();
}

/**
 * Remember the commits the runner authored, so ownership enforcement can tell
 * its own work from everyone else's. Bounded: only recent commits can appear
 * inside a stage window.
 */
function recordStageCommit(state: PipelineState, sha: string | null | undefined): void {
  if (!sha) {
    return;
  }
  const own = (state.stageCommits ??= []);
  own.push(sha);
  const excess = state.stageCommits.length - 200;
  if (excess > 0) {
    own.splice(0, excess);
  }
}

const STAGE_COMMIT_TYPE: Record<string, string> = {
  research: 'docs',
  proposal: 'docs',
  'proposal-revise': 'docs',
  'proposal-final': 'docs',
  'design-review': 'docs',
  'editorial-review': 'docs',
  'test-authoring': 'test',
  'test-update': 'test',
  implement: 'feat',
  fix: 'fix',
  'code-review': 'docs',
  'doc-sync': 'docs',
};

// --- run lock ---------------------------------------------------------------

function pidAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function acquireLock(repoDir: string, slug: string, stage: string): void {
  const path = join(repoDir, LOCK_FILE);
  if (existsSync(path)) {
    let holder: LockInfo | null;
    try {
      holder = JSON.parse(readFileSync(path, 'utf8')) as LockInfo;
    } catch {
      holder = null;
    }
    if (holder && pidAlive(holder.pid)) {
      throw new Error(
        `another pipeline run holds the lock: slug=${holder.slug} stage=${holder.stage} pid=${holder.pid}`,
      );
    }
    // Stale lock: cleared with a warning, per ADR-001.
    console.warn(
      `clearing stale pipeline lock held by ${holder?.slug ?? 'unknown'} (pid ${holder?.pid ?? '?'})`,
    );
  }
  const info: LockInfo = { slug, pid: process.pid, stage, at: Date.now() };
  writeFileSync(path, JSON.stringify(info));
}

function releaseLock(repoDir: string): void {
  try {
    unlinkSync(join(repoDir, LOCK_FILE));
  } catch {
    /* already gone */
  }
}

// --- source resolution -------------------------------------------------------
// Identity derivation lives in identity.ts; a run's branch and design
// directory are both derived from `nnn-slug`, so a taken identity must be
// detected before anything is written (see identityClash).

function freshState(id: RunIdentity, sourceRel: string): PipelineState {
  const now = Date.now();
  return {
    version: 1,
    slug: id.slug,
    nnn: id.nnn,
    branch: id.branch,
    source: { kind: id.kind, path: sourceRel },
    status: 'running',
    lastStage: '',
    designRound: 0,
    codeRound: 0,
    designBudgetUsed: 0,
    codeBudgetUsed: 0,
    editorialDone: false,
    testsGreen: false,
    testsNeedUpdate: false,
    stageFailures: {},
    stageCommits: [],
    forceNext: null,
    results: [],
    gateLog: [],
    startedAt: now,
    updatedAt: now,
  };
}

function ensureBranch(repoDir: string, branch: string): void {
  const current = git(repoDir, ['rev-parse', '--abbrev-ref', 'HEAD']).trim();
  if (current === branch) {
    return;
  }
  if (gitOk(repoDir, ['show-ref', '--verify', '--quiet', `refs/heads/${branch}`])) {
    git(repoDir, ['checkout', branch]);
  } else {
    git(repoDir, ['checkout', '-b', branch]);
  }
}

// --- reply format (mirrors opencode-runner's private helper) -----------------

function replyFormat(schema: z.ZodType): string {
  const jsonSchema = JSON.stringify(z.toJSONSchema(schema));
  return [
    'Finish ALL your work with tool calls before replying. Do not describe what',
    'you are going to do; just do it. Never end a turn except by replying with',
    'the JSON object below: if work remains, end the turn only with further',
    'tool calls, never with prose. Your final message must contain exactly one',
    'JSON object and nothing else: no markdown fences, no prose, no preamble.',
    `JSON Schema: ${jsonSchema}`,
  ].join('\n');
}

// Small models routinely end a turn mid-plan ("Now let me verify...") and go
// idle without ever emitting the required reply. The runner nudges the SAME
// session (idle sessions accept more prompts) instead of failing the stage.
/**
 * Fetch every message of a session in creation order.
 *
 * The server defaults to newest-first order with a 50-message page. Reading
 * with neither `order` nor `limit` silently inverts long sessions and
 * truncates them: the drive loop re-read the OLDEST text turn as the stage
 * reply while the model's actual replies sat beyond the window — a compliant
 * agent exhausted its nudge budget because the runner was blind to it.
 * Request `asc` pages (capped below the server's limit ceiling), follow
 * `cursor.next` until exhausted, and re-sort defensively.
 */
export async function listSessionMessages(
  client: OpenCodeClient,
  sessionID: string,
  requestTimeoutMs: number = DEFAULT_REQUEST_TIMEOUT_MS,
): Promise<SessionMessageInfo[]> {
  const all: SessionMessageInfo[] = [];
  let cursor: string | undefined;
  for (;;) {
    const page = await bounded(() =>
      client.message.list(
        {
          sessionID,
          limit: 200,
          ...(cursor === undefined ? { order: 'asc' as const } : { cursor }),
        },
        { signal: AbortSignal.timeout(requestTimeoutMs) },
      ),
    );
    all.push(...page.data);
    if (!page.cursor.next) {
      break;
    }
    cursor = page.cursor.next;
  }
  all.sort((a, b) => a.time.created - b.time.created);
  return all;
}

/**
 * One bounded retry per request: a single dropped round-trip must not fail a
 * stage the model already completed, and no request may hang the poll loop.
 * The caller still owns the stage deadline.
 */
async function bounded<T>(fn: () => Promise<T>): Promise<T> {
  const attempt = async (): Promise<T> =>
    fn().then(
      (value) => value,
      (error: unknown) => {
        throw new Error(
          `pipeline request failed: ${String(error instanceof Error ? error.message : error)}`,
        );
      },
    );
  try {
    return await attempt();
  } catch (error) {
    // Each attempt carries its own request window, so the retry cannot double
    // the stall of a timeout.
    console.warn(`${String((error as Error).message)}; retrying once`);
    return await attempt();
  }
}

/**
 * A timed-out session is still live and its agent keeps editing the very
 * files the retry produces (run 1's attempt-2 races). Remove it before the
 * runner starts anything else.
 */
async function cancelSession(
  client: OpenCodeClient,
  sessionID: string,
  stageKey: string,
  requestTimeoutMs: number,
): Promise<void> {
  try {
    await client.session.remove({ sessionID }, { signal: AbortSignal.timeout(requestTimeoutMs) });
  } catch (error) {
    console.warn(
      `stage ${stageKey}: could not remove the timed-out session ${sessionID}: ` +
        String(error instanceof Error ? error.message : error),
    );
  }
}

/**
 * The reply turn: the LAST assistant message carrying text. Small models
 * routinely end a stage with a tool-call turn and no trailing text, so the
 * newest message is often not the reply — but it must be the NEWEST
 * text-carrying one, never the first. Requires creation-ordered input
 * (see listSessionMessages).
 */
export function latestTextReply(
  assistants: SessionMessageAssistant[],
): SessionMessageAssistant | undefined {
  return [...assistants]
    .reverse()
    .find((m: SessionMessageAssistant) =>
      (m.content ?? []).some((c: { type: string }) => c.type === 'text'),
    );
}

// Eight, not two: every stage of a thorough session ends several turns with
// narration, and a two-nudge budget made structurally careful sessions fail
// stages they had otherwise completed.
const MAX_NUDGES = 8;
const NUDGE_CONTINUE = [
  'You stopped before finishing the stage. Do not narrate plans.',
  'Continue the work now with tool calls until it is complete, then reply',
  'with ONLY the JSON object required by the schema above.',
].join(' ');

/**
 * A literal example of the required object, derived from the schema. The last
 * nudge used to re-send the JSON Schema, and a model that echoes a schema
 * produces a schema: five of run 1's 43 stage attempts died on exactly that,
 * including both round-4 correctness replies.
 */
export function replyShapeExample(schema: z.ZodType): string {
  const shape = exampleOf(z.toJSONSchema(schema) as unknown);
  return [
    'Reply with ONLY a JSON object shaped exactly like this (same keys and',
    `types, your own values):
${shape}`,
  ].join(' ');
}

function exampleOf(node: unknown, depth = 0): string {
  if (depth > 3 || typeof node !== 'object' || node === null) {
    return '"..."';
  }
  const n = node as Record<string, unknown>;
  if (Array.isArray(n['enum'])) {
    return n['enum'].map((v) => JSON.stringify(v)).join(' | ');
  }
  switch (n['type']) {
    case 'object': {
      const props = (n['properties'] ?? {}) as Record<string, unknown>;
      const required = Array.isArray(n['required']) ? (n['required'] as string[]) : [];
      const keys = required.length > 0 ? required : Object.keys(props);
      const body = keys
        .map((key) => `${JSON.stringify(key)}: ${exampleOf(props[key], depth + 1)}`)
        .join(', ');
      return `{${body}}`;
    }
    case 'array':
      return `[${exampleOf(n['items'], depth + 1)}]`;
    case 'boolean':
      return 'false';
    case 'integer':
    case 'number':
      return '0';
    default:
      return '"..."';
  }
}

// --- stage execution ----------------------------------------------------------

interface StageOutcome {
  ok: boolean;
  reply?: unknown;
  failure?: string;
  /** The session this attempt ran in; lets a reply retry reuse its work. */
  sessionID?: string;
}

interface StageHandle {
  /** Registry stage name, e.g. `design-review`. */
  stage: string;
  stageKey: string;
  /** 1 for the first attempt, 2 for the retry (transcript disambiguation). */
  attempt: number;
  round: number;
  /** Funnel pass index within the round (ADR-002); absent for single-pass stages. */
  pass?: number;
  specialty?: string;
  vars: TemplateVars;
  /**
   * Set on the reply retry: continue the same session instead of starting a
   * fresh one. The failed attempt's edits, reports and reading are real work
   * — run 1 re-ran whole stages from scratch and re-did every tool call. A
   * timed-out session is never resumed: the runner removed it as a zombie.
   */
  resume?: { sessionID: string; text: string };
}

export async function executeStage(
  opts: RunOptions,
  state: PipelineState,
  handle: StageHandle,
  cfg: PipelineConfig,
): Promise<StageOutcome> {
  const def = stageDef(handle.stage, handle.specialty);
  const designDirRel = designDirOf(state);
  const designDir = join(opts.repoDir, designDirRel);

  let prompt: string;
  let templateFile: string | null = null;
  let sha: string | undefined;
  if (def.template) {
    const text = await loadTemplate(opts.promptsDir, def.template);
    templateFile = `tools/orchestration/prompts/${def.template}`;
    sha = templateHash(text);
    prompt = renderTemplate(text, handle.vars);
  } else {
    prompt = [
      `Complete the ${def.name} stage for design ${state.nnn}-${state.slug}`,
      `at ${designDirRel} as your role defines it.`,
      `For any git diff, use ${state.baseRef ?? 'main'} as the base:`,
      `\`git diff ${state.baseRef ?? 'main'}...HEAD\`.`,
    ].join(' ');
  }
  const schema = stageReplySchema(def);
  const fullPrompt = handle.resume ? handle.resume.text : `${prompt}\n\n${replyFormat(schema)}`;

  const requestTimeoutMs = cfg.requestTimeoutMs ?? DEFAULT_REQUEST_TIMEOUT_MS;
  const minNudgeIntervalMs = cfg.minNudgeIntervalMs ?? DEFAULT_MIN_NUDGE_INTERVAL_MS;
  const headBefore = git(opts.repoDir, ['rev-parse', 'HEAD']).trim();
  const sessionID =
    handle.resume?.sessionID ??
    (
      await opts.client.session.create(
        {
          title: `pipeline ${state.slug}: ${handle.stageKey}`,
          agent: def.agent,
          // Bind the session to the repo worktree so file edits, skills, and
          // agent discovery all happen in the right location.
          location: { directory: opts.repoDir },
        },
        { signal: AbortSignal.timeout(requestTimeoutMs) },
      )
    ).id;
  if (!sessionID) {
    return { ok: false, failure: 'server returned no session id' };
  }
  await opts.client.session.prompt(
    { sessionID, text: fullPrompt },
    { signal: AbortSignal.timeout(requestTimeoutMs) },
  );

  // Assistant-message count observed immediately after the most recent prompt
  // (stage prompt or nudge). An idle marker closes a prompt only once the
  // session has actually produced a new turn: without growth past this
  // baseline the poll below would re-read the previous reply and burn the
  // whole nudge budget in seconds on a slow backend.
  let assistantBaseline = (
    await listSessionMessages(opts.client, sessionID, requestTimeoutMs)
  ).filter((m) => m.type === 'assistant').length;

  const startedAt = Date.now();
  // Wall clock of the last prompt (stage prompt or nudge): the floor between
  // prompts keeps the runner from burst-nudging a session that is still busy.
  let lastPromptAt = startedAt;
  const deadline = startedAt + cfg.stageTimeoutMs;
  let rawText = '';
  let model: ModelInfo | undefined;
  let nudges = 0;
  let outcome: StageOutcome;

  // One assistant message == one model turn (verified against the server).
  // Each idle marker closes one prompt (the stage prompt plus any nudges).
  for (;;) {
    let status: 'done' | 'timeout' | 'no-turns';
    for (;;) {
      let data: SessionMessageInfo[];
      try {
        data = await listSessionMessages(opts.client, sessionID, requestTimeoutMs);
      } catch (error) {
        // The server is unreachable or hung: keep polling inside the stage
        // deadline instead of ending the run over one failed round-trip.
        console.warn(
          `stage ${handle.stageKey}: ${String((error as Error).message)}; still waiting`,
        );
        if (Date.now() >= deadline) {
          status = 'timeout';
          break;
        }
        await new Promise((r) => setTimeout(r, cfg.pollIntervalMs));
        continue;
      }
      const assistants = data.filter((m) => m.type === 'assistant');
      const idles = data.filter((m) => m.type === 'idle').length;
      if (idles >= nudges + 1 && assistants.length === 0) {
        // A bound-but-nonexistent agent produces idle markers and zero model
        // turns; nudging would be pointless. Fail fast and say why.
        status = 'no-turns';
        break;
      }
      if (idles >= nudges + 1 && assistants.length > assistantBaseline) {
        const lastTextual = latestTextReply(assistants);
        rawText = (lastTextual?.content ?? [])
          .filter((p): p is { type: 'text'; text: string } => p.type === 'text')
          .map((p) => p.text)
          .join('');
        model = lastTextual?.model ?? assistants.at(-1)?.model;
        status = 'done';
        break;
      }
      if (idles >= nudges + 1) {
        // The idle marker closed the prompt but the session produced no new
        // turn: treat it as an empty reply so the nudge path bounds the
        // failure instead of the poll loop spinning to the stage timeout.
        rawText = '';
        model = assistants.at(-1)?.model;
        status = 'done';
        break;
      }
      if (Date.now() >= deadline) {
        status = 'timeout';
        break;
      }
      await new Promise((r) => setTimeout(r, cfg.pollIntervalMs));
    }

    if (status === 'timeout') {
      // Never retry against a live zombie: the abandoned session keeps
      // writing this stage's output files.
      await cancelSession(opts.client, sessionID, handle.stageKey, requestTimeoutMs);
      outcome = { ok: false, failure: 'stage-timeout' };
      break;
    }
    if (status === 'no-turns') {
      outcome = {
        ok: false,
        failure: `no model turns — agent "${def.agent}" may not exist`,
      };
      break;
    }

    // Floor between prompts. Without it the runner fired eight nudges in 69
    // seconds at sessions that were still working, then read their replies as
    // failures. Waiting also gives a slow reply time to land.
    const nudgeGate = async (): Promise<'go' | 'wait' | 'over'> => {
      const wait = lastPromptAt + minNudgeIntervalMs - Date.now();
      if (wait <= 0) {
        return 'go';
      }
      // The floor would push the next prompt past the stage deadline: this is
      // a slow stage, not an unresponsive one, so it ends as a timeout (the
      // session is cancelled) rather than as a reply failure.
      if (Date.now() + wait >= deadline) {
        return 'over';
      }
      console.warn(
        `stage ${handle.stageKey}: no reply yet; holding nudge ${String(nudges + 1)}/` +
          `${String(MAX_NUDGES)} for ${String(wait)}ms`,
      );
      await new Promise((r) => setTimeout(r, wait));
      return 'wait'; // re-poll first: the reply may have landed while waiting
    };

    const parsed = normalizeReplyShape(extractJsonObject(rawText));
    if (parsed === undefined) {
      const gate = nudges < MAX_NUDGES ? await nudgeGate() : 'go';
      if (gate === 'over') {
        await cancelSession(opts.client, sessionID, handle.stageKey, requestTimeoutMs);
        outcome = { ok: false, failure: 'stage-timeout' };
        break;
      }
      if (gate === 'wait') continue;
      if (nudges < MAX_NUDGES) {
        nudges += 1;
        console.warn(
          `stage ${handle.stageKey}: narration instead of reply; nudge ${String(nudges)}/${String(MAX_NUDGES)}`,
        );
        // The final nudge changes kind: "continue the work" invites edit-heavy
        // agents to keep editing and summarize in prose — exactly the pattern
        // that exhausts the budget. The last chance asks ONLY for the reply.
        const lastChance =
          nudges === MAX_NUDGES
            ? 'FINAL CHANCE: stop all work now. Do not edit any file. Do not summarize. ' +
              'Your very next message must be ONLY the JSON object shown below — ' +
              'nothing else. The stage fails immediately if it is not pure JSON.'
            : NUDGE_CONTINUE;
        await opts.client.session.prompt(
          {
            sessionID,
            text: `${lastChance}\n\n${replyShapeExample(schema)}`,
          },
          { signal: AbortSignal.timeout(requestTimeoutMs) },
        );
        lastPromptAt = Date.now();
        assistantBaseline = (
          await listSessionMessages(opts.client, sessionID, requestTimeoutMs)
        ).filter((m) => m.type === 'assistant').length;
        continue;
      }
      outcome = { ok: false, failure: 'invalid-reply: not JSON' };
      break;
    }
    const checked = schema.safeParse(parsed);
    if (checked.success) {
      outcome = { ok: true, reply: checked.data };
      break;
    }
    const gate = nudges < MAX_NUDGES ? await nudgeGate() : 'go';
    if (gate === 'over') {
      await cancelSession(opts.client, sessionID, handle.stageKey, requestTimeoutMs);
      outcome = { ok: false, failure: 'stage-timeout' };
      break;
    }
    if (gate === 'wait') continue;
    if (nudges < MAX_NUDGES) {
      nudges += 1;
      console.warn(
        `stage ${handle.stageKey}: reply failed schema; nudge ${String(nudges)}/${String(MAX_NUDGES)}`,
      );
      await opts.client.session.prompt(
        {
          sessionID: sessionID,
          text: [
            `Your reply did not validate: ${checked.error.issues[0]?.message ?? 'schema mismatch'}.`,
            'Reply again with ONLY the JSON object. No other text.',
            '',
            replyShapeExample(schema),
          ].join(' '),
        },
        { signal: AbortSignal.timeout(requestTimeoutMs) },
      );
      lastPromptAt = Date.now();
      assistantBaseline = (
        await listSessionMessages(opts.client, sessionID, requestTimeoutMs)
      ).filter((m) => m.type === 'assistant').length;
      continue;
    }
    outcome = {
      ok: false,
      failure: `invalid-reply: ${checked.error.issues[0]?.message ?? 'schema'}`,
    };
    break;
  }

  const endedAt = Date.now();
  const transcript = await writeTranscript(designDir, {
    stage: handle.stageKey + (handle.attempt > 1 ? `:attempt${String(handle.attempt)}` : ''),
    round: handle.round,
    pass: handle.pass,
    agent: def.agent,
    model,
    templateFile,
    templateSha256: sha,
    startedAt,
    endedAt,
    prompt: fullPrompt,
    rawReply: rawText,
  });

  const size =
    outcome.ok && DESIGN_DOC_STAGES.has(def.name)
      ? proposalSize(opts.repoDir, designDir)
      : undefined;
  const result: StageResult = {
    stage: handle.stageKey,
    round: handle.round,
    sessionID: sessionID,
    agent: def.agent,
    at: endedAt,
    reply: outcome.reply,
    transcript: relative(opts.repoDir, transcript),
    durationMs: endedAt - startedAt,
    nudges,
    ...(outcome.ok ? {} : { failure: outcome.failure }),
    ...(size
      ? {
          docLines: size.docLines,
          docLinesAdded: size.added,
          docLinesRemoved: size.removed,
        }
      : {}),
  };
  state.results.push(result);
  console.log(
    `stage ${handle.stageKey}: ${outcome.ok ? 'ok' : `FAIL (${outcome.failure})`} ` +
      `${String(Math.round((endedAt - startedAt) / 1000))}s, ${String(nudges)} nudge(s)` +
      `${handle.attempt > 1 ? ', retry' : ''}` +
      `${size ? `, proposal ${String(size.docLines)} lines (+${String(size.added)}/-${String(size.removed)})` : ''}`,
  );

  if (outcome.ok) {
    const report = enforceOwnership(opts.repoDir, headBefore, def.name, state);
    if (report.reverted.length > 0) {
      console.warn(
        `ownership: reverted ${String(report.reverted.length)} file(s) changed by ${def.name}: ` +
          report.reverted.join(', '),
      );
    }
    for (const c of report.foreign) {
      console.warn(
        `ownership: commit ${c.sha.slice(0, 8)} ("${c.subject}") landed during ${def.name}; ` +
          'left untouched — it is not runner-authored',
      );
    }
    if (
      (def.name === 'implement' || def.name === 'fix') &&
      report.reverted.some((p) => TEST_PATH_RE.test(p))
    ) {
      state.testsNeedUpdate = true;
    }
    if (typeof (outcome.reply as { testsNeedUpdate?: boolean })?.testsNeedUpdate === 'boolean') {
      state.testsNeedUpdate = (outcome.reply as { testsNeedUpdate: boolean }).testsNeedUpdate;
    }
    recordStageCommit(
      state,
      commitStage(
        opts.repoDir,
        `${state.nnn}-${state.slug}`,
        handle.stageKey,
        STAGE_COMMIT_TYPE[def.name] ?? 'chore',
        outcome.reply,
      ),
    );
  }
  return { ...outcome, sessionID };
}

/**
 * Only a missing reply is retried in the same session: a timed-out session was
 * cancelled as a zombie, and a stage whose agent does not exist has no work to
 * resume.
 */
export function retryInSameSession(
  failure: string | undefined,
  sessionID: string | undefined,
): boolean {
  return sessionID !== undefined && (failure ?? '').startsWith('invalid-reply');
}

/**
 * Prompt for the retry of a stage whose reply never validated. The session
 * still holds everything it read and wrote, so the retry asks only for the
 * missing part instead of starting the stage over.
 */
function repairPrompt(def: { name: string }, schema: z.ZodType, failure: string): string {
  return [
    `Your previous attempt at the ${def.name} stage ended without a usable reply (${failure}).`,
    'Do not redo work that is already done: finish only what is missing, then reply.',
    '',
    replyShapeExample(schema),
  ].join('\n');
}

// --- escalation ---------------------------------------------------------------

function gateLog(
  state: PipelineState,
  gate: string,
  round: number,
  result: { proceed: boolean; reason: string },
): void {
  state.gateLog.push({
    gate,
    round,
    proceed: result.proceed,
    reason: result.reason,
    at: Date.now(),
  });
}

// --- the drive loop -------------------------------------------------------------

class StopDriving extends Error {}

export async function runPipeline(opts: RunOptions): Promise<PipelineState['status']> {
  const cfg = opts.cfg ?? DEFAULT_CONFIG;
  const sourceAbs = resolve(opts.repoDir, opts.sourcePath);
  const id = deriveIdentity(opts.repoDir, sourceAbs, opts.pin);

  const clash = identityClash(opts.repoDir, id);
  if (clash.length > 0 && !opts.force) {
    // ensureBranch checks out an existing branch, so a taken identity would
    // silently continue a previous run on its history while `run` writes a
    // fresh checkpoint and a fresh baseRef under it.
    throw new Error(
      `refusing to start run ${id.nnn}-${id.slug}:\n- ${clash.join('\n- ')}\n` +
        'pass --force to continue on the existing identity anyway',
    );
  }

  if (opts.forceStage) {
    // Before the lock, before ESCALATION.md, before any state: a typo used to
    // crash deep inside stageDef with the marker already deleted.
    assertForceableStage(opts.forceStage);
  }

  acquireLock(opts.repoDir, id.slug, 'start');
  try {
    const state = freshState(id, relative(opts.repoDir, sourceAbs));
    if (opts.forceStage) {
      state.forceNext = opts.forceStage;
    }
    if (opts.extendBudget) {
      state.extensionSessions = (state.extensionSessions ?? 0) + opts.extendBudget;
      state.extensionRounds = (state.extensionRounds ?? 0) + opts.extendBudget;
    }
    state.baseRef = git(opts.repoDir, ['rev-parse', 'HEAD']).trim();
    ensureBranch(opts.repoDir, state.branch);
    await saveState(join(opts.repoDir, designDirOf(state)), state);
    const verdict = await sourceAdmissible(sourceAbs, id.kind);
    gateLog(state, 'sourceAdmissible', 0, verdict);
    if (!verdict.proceed) {
      await finalizeEscalate(opts.repoDir, state, `source not admissible: ${verdict.reason}`, cfg);
      return state.status;
    }
    await drive(opts, state, cfg);
    return state.status;
  } finally {
    releaseLock(opts.repoDir);
  }
}

export async function resumePipeline(opts: ResumeOptions): Promise<PipelineState['status']> {
  const cfg = opts.cfg ?? DEFAULT_CONFIG;
  const designDirRel = join('docs', 'designs', opts.slug);
  const state = await loadState(join(opts.repoDir, designDirRel));
  if (!state) {
    throw new Error(`no pipeline checkpoint at ${designDirRel}`);
  }
  if (opts.forceStage) {
    assertForceableStage(opts.forceStage);
  }
  acquireLock(opts.repoDir, state.slug, 'resume');
  try {
    state.status = 'running';
    if (opts.forceStage) {
      state.forceNext = opts.forceStage;
      gateLog(state, 'operatorForce', 0, {
        proceed: true,
        reason: `operator forced the next step: ${opts.forceStage}`,
      });
    }
    if (opts.extendBudget) {
      state.extensionSessions = (state.extensionSessions ?? 0) + opts.extendBudget;
      state.extensionRounds = (state.extensionRounds ?? 0) + opts.extendBudget;
      gateLog(state, 'operatorExtension', 0, {
        proceed: true,
        reason:
          `operator extended the review budget by ${String(opts.extendBudget)} review session(s) ` +
          `over the caps of ${String(cfg.maxReviews.design)}/${String(cfg.maxReviews.code)} ` +
          `(and ${String(opts.extendBudget)} revise round(s) over the backstop of ${String(cfg.maxRounds)})`,
      });
    }
    // A resume means the cause was addressed; drop the stale marker (the
    // record lives on in the gate log and git history).
    try {
      unlinkSync(join(opts.repoDir, designDirRel, 'ESCALATION.md'));
    } catch {
      /* no marker */
    }
    if (!state.baseRef) {
      // Backfill for checkpoints written before baseRef existed: fork point
      // of the pipeline branch, falling back to the main repo default.
      const mb = spawnSync('git', ['merge-base', 'main', state.branch], {
        cwd: opts.repoDir,
        encoding: 'utf8',
      });
      state.baseRef = mb.status === 0 ? mb.stdout.trim() : 'main';
    }
    ensureBranch(opts.repoDir, state.branch);
    await drive(opts, state, cfg);
    return state.status;
  } finally {
    releaseLock(opts.repoDir);
  }
}

async function drive(
  opts: RunOptions | ResumeOptions,
  state: PipelineState,
  cfg: PipelineConfig,
): Promise<void> {
  const designDirRel = designDirOf(state);
  const designDir = join(opts.repoDir, designDirRel);
  let proposalSnapshot: string | null = null;

  for (;;) {
    const next = decideNext(state, cfg);
    state.forceNext = null;

    if (next.kind === 'done') {
      await finalizeDone(opts.repoDir, state);
      return;
    }
    if (next.kind === 'escalate') {
      await finalizeEscalate(opts.repoDir, state, next.reason, cfg);
      return;
    }

    const runWithRetry = async (handle: StageHandle): Promise<StageOutcome> => {
      let out = await executeStage(opts as RunOptions, state, { ...handle, attempt: 1 }, cfg);
      if (!out.ok) {
        state.stageFailures[handle.stageKey] = (state.stageFailures[handle.stageKey] ?? 0) + 1;
        gateLog(state, 'stageFailure', handle.round, {
          proceed: false,
          reason: `${handle.stageKey}: ${out.failure}`,
        });
        await saveState(designDir, state);
        // A reply failure means the work is usually done and only the closing
        // object was missing: retry in the same session, where the report and
        // the reading are still there. A timeout is different — that session
        // was cancelled as a zombie, so attempt 2 starts fresh.
        const def = stageDef(handle.stage, handle.specialty);
        const resumable = retryInSameSession(out.failure, out.sessionID);
        out = await executeStage(
          opts as RunOptions,
          state,
          {
            ...handle,
            attempt: 2,
            ...(resumable
              ? {
                  resume: {
                    sessionID: out.sessionID as string,
                    text: repairPrompt(def, stageReplySchema(def), out.failure ?? 'no reply'),
                  },
                }
              : {}),
          },
          cfg,
        );
        if (!out.ok) {
          state.stageFailures[handle.stageKey] = (state.stageFailures[handle.stageKey] ?? 0) + 1;
          await finalizeEscalate(
            opts.repoDir,
            state,
            `stage ${handle.stageKey} failed twice (${out.failure}) — nudge/reply rule: one retry, then escalate`,
            cfg,
          );
          throw new StopDriving();
        }
      }
      return out;
    };

    try {
      if (next.kind === 'review') {
        const phase = next.phase;
        if (phase === 'editorial') {
          // ADR-002 leaves the editorial phase as written: it is a language
          // pass, not a vote, so its one five-seat confirmation review stays.
          state.designRound = next.round; // editorial continues the rN sequence
          if (proposalSnapshot !== null) {
            writeEditorialDiff(designDir, state.slug, next.round, proposalSnapshot, opts.repoDir);
          }
          await saveState(designDir, state);
          for (const specialty of specialtiesFor('editorial')) {
            await runWithRetry({
              stage: 'editorial-review',
              stageKey: `editorial-review:${specialty}`,
              attempt: 1,
              round: next.round,
              specialty,
              vars: reviewVars(designDirRel, state, 'editorial', next.round, specialty),
            });
            await saveState(designDir, state);
          }
          state.lastStage = 'editorial-review';
          const editorialRejects = specialtiesFor('editorial').filter(
            (sp) => lastSeatReply(state, 'editorial', sp)?.verdict !== 'approve',
          );
          state.lastReviewPass = {
            phase: 'editorial',
            round: next.round,
            seats:
              editorialRejects.length > 0 ? editorialRejects : [...specialtiesFor('editorial')],
          };
          const g = allReviewsApproved(
            state,
            'editorial',
            next.round,
            specialtiesFor('editorial'),
            cfg,
          );
          gateLog(state, gateName('editorial'), next.round, g);
          await saveState(designDir, state);
          continue;
        }

        const isDesign = phase === 'design';
        // ADR-002: review cost is measured in sessions. The cap is checked at
        // pass entry, not only at the round gate — run 1's forced-round lesson
        // survives: no entry sneaks past a cap, session or round.
        const cap = cfg.maxReviews[phase];
        const spent = reviewSessionsUsed(state, phase);
        if (spent >= cap) {
          const left = state.extensionSessions ?? 0;
          if (left <= 0) {
            await finalizeEscalate(
              opts.repoDir,
              state,
              `${phase} review session budget exhausted (${String(spent)}/${String(cap)} sessions); ` +
                `extend it explicitly: pipeline resume ${state.nnn}-${state.slug} --extend-budget <n>`,
              cfg,
            );
            return;
          }
          state.extensionSessions = left - 1;
          gateLog(state, 'operatorExtension', next.round, {
            proceed: true,
            reason:
              `over-cap ${phase} review pass entered under operator extension ` +
              `(${String(left - 1)} session grant(s) left)`,
          });
        }
        // The round counter is the backstop on the mutating revise/fix loop,
        // checked only when a NEW round starts: passes inside one round spend
        // sessions and bounded by `maxPassesPerRound`, never a round.
        if (next.round > (isDesign ? state.designRound : state.codeRound)) {
          const usedRounds = reviewBudgetUsed(state, phase);
          if (usedRounds >= cfg.maxRounds) {
            const left = state.extensionRounds ?? 0;
            if (left <= 0) {
              await finalizeEscalate(
                opts.repoDir,
                state,
                `${phase} revise-round backstop reached (${String(usedRounds)}/${String(
                  cfg.maxRounds,
                )} rounds); ` +
                  `this bounds the mutating loop, not review cost — sessions extend with --extend-budget`,
                cfg,
              );
              return;
            }
            state.extensionRounds = left - 1;
            gateLog(state, 'operatorExtension', next.round, {
              proceed: true,
              reason:
                `over-cap ${phase} round entered under operator extension ` +
                `(${String(left - 1)} granted round(s) left)`,
            });
          }
          if (isDesign) {
            state.designCycle = 0;
            state.designBudgetUsed += 1;
          } else {
            state.codeCycle = 0;
            state.codeBudgetUsed += 1;
          }
        }
        if (isDesign) {
          state.designRound = next.round;
          state.designStamp = blobStamp(opts.repoDir, `${designDirRel}/02-proposal.md`);
        } else {
          state.codeRound = next.round;
          state.codeStamp = git(opts.repoDir, ['rev-parse', 'HEAD']).trim();
        }

        const plan = planPass(state, phase, cfg);
        if (plan.kind !== 'pass') {
          gateLog(
            state,
            gateName(phase),
            next.round,
            plan.kind === 'converged'
              ? { proceed: true, reason: 'all seats already approve the current revision' }
              : { proceed: false, reason: plan.reason },
          );
          await saveState(designDir, state);
          continue;
        }
        console.log(
          `review ${phase} r${String(next.round)}c${String(plan.cycle)} ` +
            `(${plan.pass}: ${plan.seats.join(', ')})`,
        );
        await saveState(designDir, state);

        for (const specialty of plan.seats) {
          await runWithRetry({
            stage: `${phase}-review`,
            stageKey: `${phase}-review:${specialty}`,
            attempt: 1,
            round: next.round,
            pass: plan.cycle,
            specialty,
            vars: reviewVars(designDirRel, state, phase, next.round, specialty, plan),
          });
          await saveState(designDir, state);
        }
        state.lastStage = `${phase}-review`;
        // Pin every seat's latest valid verdict to the revision it was shown;
        // a retry's verdict supersedes the failed attempt it retried.
        const passStamp = isDesign ? (state.designStamp ?? '') : (state.codeStamp ?? '');
        for (const specialty of plan.seats) {
          const reply = lastSeatReply(state, phase, specialty);
          if (reply) {
            recordSeatVerdict(state, phase, specialty, seatDecision(reply, cfg), passStamp);
          }
        }
        if (isDesign) {
          state.designCycle = plan.cycle + 1;
        } else {
          state.codeCycle = plan.cycle + 1;
        }
        // The next revise answers the seats that dissented on this pass — not
        // the whole panel's reports, which is how run 2 grew the document 6x.
        const board = isDesign ? state.seats?.design : state.seats?.code;
        const rejecters = plan.seats.filter((sp) => board?.[sp]?.verdict === 'request-changes');
        state.lastReviewPass = {
          phase,
          round: next.round,
          cycle: plan.cycle,
          seats: rejecters.length > 0 ? rejecters : [...plan.seats],
        };
        const after = planPass(state, phase, cfg);
        gateLog(
          state,
          gateName(phase),
          next.round,
          after.kind === 'converged'
            ? { proceed: true, reason: 'all five seats approve the current revision stamp' }
            : after.kind === 'blocked'
              ? { proceed: false, reason: after.reason }
              : {
                  proceed: false,
                  reason: `${after.pass} pass c${String(after.cycle)} pending: ${after.seats.join(', ')}`,
                },
        );
        await saveState(designDir, state);
        continue;
      }

      // single stage
      const name = next.stage;
      if (name === 'fix') {
        state.codeBudgetUsed += 1;
      }
      if (name === 'proposal-final') {
        proposalSnapshot = await readFile(join(designDir, '02-proposal.md'), 'utf8');
      }
      const out = await runWithRetry({
        stage: name,
        stageKey: name,
        attempt: 1,
        round: name === 'proposal-revise' ? state.designRound : state.codeRound,
        vars: stageVars(designDirRel, state, name),
      });
      state.lastStage = name;
      if (name === 'proposal-revise' && out.ok) {
        // Deadlock-stop input: objections that revisions do not move are the
        // run-2 failure mode, and `changed: false` names it in one pass.
        state.reviseNoChange = (out.reply as { changed?: boolean } | undefined)?.changed === false;
      }

      if (name === 'test-update') {
        // The request has been served; a later fix or implement stage sets it
        // again if it touches owned test paths.
        state.testsNeedUpdate = false;
      }
      if (name === 'implement' || name === 'fix' || name === 'test-update') {
        state.testsGreen = await runChecks(opts.repoDir);
        gateLog(state, 'testsGreen', 0, {
          proceed: state.testsGreen,
          reason: state.testsGreen ? 'npm run check green' : 'npm run check red',
        });
      }
      await saveState(designDir, state);
    } catch (error) {
      if (error instanceof StopDriving) {
        return;
      }
      throw error;
    }
  }
}

// --- stage vars and editorial diff ----------------------------------------------

function reviewReportList(
  rel: string,
  phase: ReviewPhase,
  round: number,
  specialties: readonly string[],
): string {
  return specialties.map((s) => `${rel}/reviews/${phase}/${s}-r${round}.md`).join(' ');
}

/**
 * The reports handed to the next mutating stage: ADR-002 sends the designer
 * only what the last pass's dissenting seats wrote, not every report of every
 * round — run 2's designer received five full reports per round and grew the
 * document sixfold trying to answer all of them. Falls back to the full panel
 * for checkpoints written before seat passes existed.
 */
function reviewFilesFor(
  designDirRel: string,
  state: PipelineState,
  phases: readonly ('design' | 'editorial' | 'code')[],
  fallbackPhase: 'design' | 'code',
  fallbackRound: number,
): string {
  const pass = state.lastReviewPass;
  if (pass && phases.includes(pass.phase)) {
    return pass.seats
      .map((sp) => reportPathFor(designDirRel, pass.phase, sp, pass.round, pass.cycle))
      .join(' ');
  }
  return reviewReportList(
    designDirRel,
    fallbackPhase,
    fallbackRound,
    specialtiesFor(fallbackPhase),
  );
}

function stageVars(designDirRel: string, state: PipelineState, name: string): TemplateVars {
  const round = name === 'proposal-revise' ? state.designRound : state.codeRound;
  const reviewFiles =
    name === 'proposal-revise'
      ? reviewFilesFor(designDirRel, state, ['design', 'editorial'], 'design', state.designRound)
      : name === 'fix' || name === 'test-update'
        ? reviewFilesFor(designDirRel, state, ['code'], 'code', state.codeRound)
        : '';
  return {
    designDir: designDirRel,
    sourcePath: state.source.path,
    round: Math.max(round, 1),
    specialty: '',
    scope: `git diff ${state.baseRef ?? 'main'}...HEAD`,
    reviewFiles,
  };
}

function reviewVars(
  designDirRel: string,
  state: PipelineState,
  phase: ReviewPhase,
  round: number,
  specialty: string,
  plan?: Extract<PassPlan, { kind: 'pass' }>,
): TemplateVars {
  if (phase === 'editorial') {
    return {
      designDir: designDirRel,
      sourcePath: state.source.path,
      round,
      specialty,
      scope: `the diff file ${designDirRel}/reviews/design/editorial-r${round}.diff`,
      passNote: 'the single editorial confirmation pass',
      reportPath: reportPathFor(designDirRel, 'editorial', specialty, round, undefined),
      reviewFiles: '',
    };
  }
  const cycle = plan?.cycle ?? 0;
  const passKind = plan?.pass ?? 'full';
  const priorStamp =
    phase === 'design'
      ? (state.seats?.design?.[specialty]?.stamp ?? '')
      : (state.seats?.code?.[specialty]?.stamp ?? '');
  const currentStamp = phase === 'design' ? (state.designStamp ?? '') : (state.codeStamp ?? '');
  const scope =
    passKind === 'full'
      ? phase === 'design'
        ? 'the whole document'
        : `git diff ${state.baseRef ?? 'main'}...HEAD`
      : priorStamp === '' || currentStamp === ''
        ? 'the whole current document (no revision stamp to diff against)'
        : phase === 'design'
          ? `the sections changed since you last reviewed: run ` +
            `\`git diff ${priorStamp} ${currentStamp}\` (both are blob ids of 02-proposal.md), ` +
            'then read the full current document'
          : `git diff ${priorStamp}..HEAD, read against the full current document`;
  const passNote =
    passKind === 'full'
      ? `full-panel pass c${String(cycle)}`
      : passKind === 'dissent'
        ? `dissent pass c${String(cycle)}: the last revision was written to answer YOUR findings`
        : `ratification pass c${String(cycle)}: revisions landed after you approved; ` +
          'you are checking only that they did not break what you approved';
  return {
    designDir: designDirRel,
    sourcePath: state.source.path,
    round,
    specialty,
    scope,
    passNote,
    reportPath: reportPathFor(designDirRel, phase, specialty, round, cycle),
    reviewFiles: '',
  };
}

function writeEditorialDiff(
  designDir: string,
  slug: string,
  round: number,
  preEdit: string,
  repoDir: string,
): void {
  const snapFile = join(tmpdir(), `bindforge-editorial-${slug}-r${round}.md`);
  writeFileSync(snapFile, preEdit, 'utf8');
  const res = spawnSync(
    'git',
    ['diff', '--no-index', '--', snapFile, join(designDir, '02-proposal.md')],
    { cwd: repoDir, encoding: 'utf8' },
  );
  if (res.status !== null && res.status > 1) {
    throw new Error(`editorial diff failed: ${res.stderr}`);
  }
  const out = res.stdout.trim().length > 0 ? res.stdout : '(the editor changed nothing)\n';
  mkdirSync(join(designDir, 'reviews', 'design'), { recursive: true });
  writeFileSync(join(designDir, 'reviews', 'design', `editorial-r${round}.diff`), out, 'utf8');
}

// --- finalization -----------------------------------------------------------------

async function finalizeDone(repoDir: string, state: PipelineState): Promise<void> {
  const designDir = join(repoDir, designDirOf(state));
  if (state.source.kind === 'adr') {
    const adrPath = join(repoDir, state.source.path);
    const content = readFileSync(adrPath, 'utf8');
    const patched = flipAdrStatus(content);
    if (patched === null) {
      console.warn(`ADR status not flipped (unexpected format): ${state.source.path}`);
    } else {
      writeFileSync(adrPath, patched, 'utf8');
    }
  }
  state.status = 'done';
  gateLog(state, 'done', 0, { proceed: true, reason: 'pipeline complete' });
  // Save BEFORE committing so the completion commit contains the terminal
  // status; a save after the commit would leave the checkpoint dirty.
  await saveState(designDir, state);
  recordStageCommit(
    state,
    commitStage(repoDir, `${state.nnn}-${state.slug}`, 'done', 'docs', {
      commitMessage: `docs: ${state.nnn}-${state.slug} pipeline complete`,
    }),
  );
}

export async function finalizeEscalate(
  repoDir: string,
  state: PipelineState,
  reason: string,
  cfg: PipelineConfig,
): Promise<void> {
  state.status = 'escalated';
  gateLog(state, 'escalate', 0, { proceed: false, reason });
  const designDirRel = designDirOf(state);
  const designDir = join(repoDir, designDirRel);
  mkdirSync(designDir, { recursive: true });
  const phase = escalationPhase(state);
  // Atomic tmp+rename, like every other checkpoint write: this is the one
  // record the operator must be able to read afterwards, and interrupting a
  // run is most likely exactly when it is being written.
  await saveState(designDir, state);
  const reportRel = join(designDirRel, 'ESCALATION.md');
  writeFileSync(
    join(repoDir, reportRel),
    escalationMarkdown(state, phase, cfg.blockingSeverities),
    'utf8',
  );
  // Nothing printed on escalation in run 1: a 47-minute resume that ended
  // escalated looked like a quiet success on the operator's terminal.
  console.error(
    `\npipeline ESCALATED (${phase} phase, after stage ${state.lastStage}): ${reason}\n` +
      `  report: ${reportRel}\n` +
      `  status: pipeline status ${state.nnn}-${state.slug}\n` +
      `  retry after a fix: pipeline resume ${state.nnn}-${state.slug}`,
  );
  recordStageCommit(
    state,
    commitStage(repoDir, `${state.nnn}-${state.slug}`, 'escalate', 'docs', {
      commitMessage: `docs: ${state.nnn}-${state.slug} pipeline escalated`,
    }),
  );
}

/** Human-facing explanation for `pipeline status`, from the gate log only. */
export function renderStatus(state: PipelineState): string {
  const growth = proposalGrowthLine(state);
  const lines = [
    `run:     ${state.nnn}-${state.slug} (${state.status})`,
    `source:  ${state.source.kind}: ${state.source.path}`,
    `branch:  ${state.branch}`,
    `stages:  last=${state.lastStage || '(none)'} designRound=${state.designRound} codeRound=${state.codeRound}`,
    `budgets: design ${state.designBudgetUsed} rounds, code ${state.codeBudgetUsed} rounds; ` +
      `reviews: design ${reviewSessionsUsed(state, 'design')} sessions, code ${reviewSessionsUsed(
        state,
        'code',
      )} sessions`,
    ...(hasSeats(state, 'design') ? [seatSummary(state, 'design')] : []),
    ...(hasSeats(state, 'code') ? [seatSummary(state, 'code')] : []),
    ...(growth === null ? [] : [growth]),
    'recent gates:',
    ...state.gateLog
      .slice(-6)
      .map((g) => `  ${g.proceed ? 'ok  ' : 'DENY'} ${g.gate} r${g.round}: ${g.reason}`),
  ];
  return lines.join('\n');
}
