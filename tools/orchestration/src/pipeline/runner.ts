import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, unlinkSync, writeFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, relative, resolve } from 'node:path';

import type { OpenCodeClient, SessionMessageAssistant, SessionMessageInfo } from '@opencode/client';
import { z } from 'zod';

import { createSemaphore } from './concurrency.js';
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
  type IndependenceAuditOptions,
  auditSessionIndependence,
  brief,
  printable,
} from './independence.js';
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
import {
  type CheckpointWriter,
  createStateWriter,
  designDirOf,
  loadState,
  saveState,
} from './state-store.js';
import {
  type ReviewPhase,
  decideNext,
  gateName,
  reviewBudgetUsed,
  specialtiesFor,
} from './states.js';
import type { Finding, LockInfo, PipelineState, StageResult } from './types.js';

// ---------------------------------------------------------------------------
// The inference lane. Mutating stages keep exactly one session at a time, each
// a fresh top-level session bound to its stage agent. A `kind: 'review'` pass
// fans its seats out through a semaphore of `cfg.maxParallelReviews` (default
// 1 — the same seats, strictly one at a time) and finalizes once per pass.
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

// Hygiene is NOT done here, and the runner never executes a formatter.
//
// A previous revision of this code ran `node_modules/.bin/prettier` of every
// package the stage touched, before every commit. That is a runner-executed
// binary whose path is a naming convention inside the worktree: `node_modules`
// is git-ignored, so a planted `package.json` + `node_modules/.bin/<tool>`
// under any probed package dir is invisible to `dirtyFiles` and
// `enforceOwnership`, never appears in a human-reviewed commit, and still gets
// exec'd by the process that owns the checkpoint, the git authorship, and the
// lock. Nothing in a model-writable worktree can be made trustworthy from a
// path convention, so no stage-execution path runs anything out of one.
//
// The `format:check` failure that step papered over has an ownership answer
// instead: the gate names the files, the red `npm run check` summons `fix`
// (which runs `npm run format` itself — an allow-listed command from the
// package's committed `package.json`), and when the dirty files are test files
// the fix stage cannot own, its `testsNeedUpdate: true` reply routes the work
// to `test-update`, which owns them. `docs/workflows.md` carries the same
// answer for operators.

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

/**
 * Checkpoint the run: record the moment this snapshot was TAKEN, then write it.
 *
 * The stamp lives at the write site, not inside `saveState`. ADR-003's review
 * passes hand their state to a queued writer, and a snapshot stamped when the
 * queue reaches it would mean "when the writer woke up" rather than "when the
 * run looked like this" — exactly the masquerade that made `saveState` stop
 * re-stamping. Stamping at every snapshot point keeps the persisted
 * `updatedAt` meaning "when this checkpoint was taken" on both write paths:
 * the plain sequential one here and the drained writer inside a pass.
 */
async function checkpoint(designDir: string, state: PipelineState): Promise<void> {
  state.updatedAt = Date.now();
  await saveState(designDir, state); // the validating tmp+rename write
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

// --- deeds of children (design 007 §2) --------------------------------------
//
// A completed `subagent` part carries no access, so the evidence for a
// delegation is the child sessions' OWN completed tool parts. The walk is a
// BFS over the attempt session's descendant sessions — visited-set guarded,
// so a malformed parentID graph cannot loop — yielding one message list at
// a time so peak memory stays one list. The enumeration route is probed at
// call time: `session.children` when the client exposes it, else one paged
// `session.list` indexed by `parentID`. If neither is available the walk
// throws; the audit call site catches that and proceeds parent-only with the
// gap logged — a stage failure on an infrastructure fault is not what this
// audit is for (the permission fence stays the primary control).

interface SessionChildApi {
  children?: (input: { sessionID: string }, options?: { signal: AbortSignal }) => Promise<unknown>;
  list?: (input?: Record<string, unknown>, options?: { signal: AbortSignal }) => Promise<unknown>;
}

/** Session records out of a `children`/`list` response (array or `{ data }`). */
function sessionEntries(page: unknown): Array<{ id: string; parentID?: string }> {
  let data: unknown[] = [];
  if (Array.isArray(page)) {
    data = page;
  } else if (page !== null && typeof page === 'object') {
    const record = page as { data?: unknown };
    if (Array.isArray(record.data)) {
      data = record.data;
    }
  }
  const out: Array<{ id: string; parentID?: string }> = [];
  for (const entry of data) {
    if (entry === null || typeof entry !== 'object') {
      continue;
    }
    const id = (entry as { id?: unknown }).id;
    if (typeof id !== 'string' || id.length === 0) {
      continue;
    }
    const parentID = (entry as { parentID?: unknown }).parentID;
    if (typeof parentID === 'string' && parentID.length > 0) {
      out.push({ id, parentID });
    } else {
      out.push({ id });
    }
  }
  return out;
}

/** The `cursor.next` of a paged response, if it carries one. */
function pageCursorNext(page: unknown): string | undefined {
  if (page === null || typeof page !== 'object') {
    return undefined;
  }
  const next = (page as { cursor?: { next?: unknown } }).cursor?.next;
  return typeof next === 'string' && next.length > 0 ? next : undefined;
}

async function descendantMessages(
  client: OpenCodeClient,
  sessionID: string,
  requestTimeoutMs: number,
): Promise<SessionMessageInfo[]> {
  try {
    return await listSessionMessages(client, sessionID, requestTimeoutMs);
  } catch (error) {
    throw new Error(
      `fetching the message list of ${sessionID}: ` +
        String(error instanceof Error ? error.message : error),
      { cause: error },
    );
  }
}

/**
 * Every descendant session of `sessionID` (children, grandchildren, …) with
 * its full creation-ordered message list. A point-in-time snapshot taken
 * after the attempt's reply: a descendant still running at that moment
 * contributes only its completed parts.
 */
export async function* listChildSessionMessages(
  client: OpenCodeClient,
  sessionID: string,
  requestTimeoutMs: number,
): AsyncGenerator<{ sessionID: string; messages: SessionMessageInfo[] }> {
  const api = client.session as unknown as SessionChildApi;
  const visited = new Set<string>([sessionID]);
  const queue: string[] = [];
  // `bind` keeps the method call on the client object: SDK methods may close
  // over or use `this`.
  const children = typeof api.children === 'function' ? api.children.bind(api) : undefined;
  if (children !== undefined) {
    queue.push(sessionID);
    for (let head = 0; head < queue.length; head += 1) {
      const parent = queue[head];
      if (parent === undefined) {
        continue;
      }
      let listed: unknown;
      try {
        listed = await bounded(() =>
          children({ sessionID: parent }, { signal: AbortSignal.timeout(requestTimeoutMs) }),
        );
      } catch (error) {
        throw new Error(
          `enumerating the children of ${parent}: ` +
            String(error instanceof Error ? error.message : error),
          { cause: error },
        );
      }
      for (const child of sessionEntries(listed)) {
        if (visited.has(child.id)) {
          continue;
        }
        visited.add(child.id);
        yield {
          sessionID: child.id,
          messages: await descendantMessages(client, child.id, requestTimeoutMs),
        };
        queue.push(child.id);
      }
    }
    return;
  }
  const list = typeof api.list === 'function' ? api.list.bind(api) : undefined;
  if (list === undefined) {
    throw new Error(
      `enumerating the children of ${sessionID}: the client exposes neither session.children nor session.list`,
    );
  }
  // One paged enumeration for the whole walk (not one per node), indexed by
  // `parentID` — the same `cursor.next` crawl as `listSessionMessages`.
  const byParent = new Map<string, string[]>();
  let cursor: string | undefined;
  for (;;) {
    let page: unknown;
    try {
      page = await bounded(() =>
        list(
          { limit: 200, ...(cursor === undefined ? {} : { cursor }) },
          { signal: AbortSignal.timeout(requestTimeoutMs) },
        ),
      );
    } catch (error) {
      throw new Error(
        `listing sessions to find the children of ${sessionID}: ` +
          String(error instanceof Error ? error.message : error),
        { cause: error },
      );
    }
    for (const entry of sessionEntries(page)) {
      if (entry.parentID === undefined || entry.parentID === entry.id) {
        continue;
      }
      const siblings = byParent.get(entry.parentID) ?? [];
      siblings.push(entry.id);
      byParent.set(entry.parentID, siblings);
    }
    const next = pageCursorNext(page);
    if (next === undefined) {
      break;
    }
    cursor = next;
  }
  queue.push(sessionID);
  for (let head = 0; head < queue.length; head += 1) {
    const parent = queue[head];
    if (parent === undefined) {
      continue;
    }
    for (const childID of byParent.get(parent) ?? []) {
      if (visited.has(childID)) {
        continue;
      }
      visited.add(childID);
      yield {
        sessionID: childID,
        messages: await descendantMessages(client, childID, requestTimeoutMs),
      };
      queue.push(childID);
    }
  }
}

/** A completed `subagent` part in this attempt's audited window: a delegation happened. */
function hasCompletedDelegation(messages: readonly SessionMessageInfo[], since: number): boolean {
  for (const m of messages) {
    if ((m.time?.created ?? 0) < since) {
      continue;
    }
    if (m.type !== 'assistant') {
      continue;
    }
    for (const part of m.content ?? []) {
      if (part.type === 'tool' && part.name === 'subagent' && part.state.status === 'completed') {
        return true;
      }
    }
  }
  return false;
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
  /** Set on a fresh-session attempt 2: why attempt 1 failed (log excerpt). */
  failureNote?: string;
}

/**
 * Which template a stage runs. `fix` is the one trigger-sensitive stage: the
 * planner enters it either from a red testsGreen gate (states.ts: implement/
 * test-update/fix) or from a code-review dissent (tests green by then). A
 * tests-red fix must not be handed review reports — when review has not run
 * they do not exist, and run 003's fix was sent five phantom paths and told
 * to guess the real task.
 */
export function stageTemplate(
  def: { name: string; template: string | null },
  stage: string,
  state: PipelineState,
): string | null {
  if (stage === 'fix' && !state.testsGreen) {
    return 'fix-tests.md';
  }
  return def.template;
}

/**
 * The git-free inference phase of one stage attempt: render the prompt,
 * create (or resume) the session, poll/nudge to a validated reply, write the
 * transcript, and append the `StageResult`. No git mutation lives here — the
 * caller owns the stage/pass window (`headBefore`, captured before the
 * prompt) and the finalize (ownership + commit). See `executeStage` for the
 * single-stage combined path and `runReviewPass` for the pass-level one.
 *
 * `templateText` preloads the prompt template: the pass fan-out loads it
 * once and hands it to every seat, so the only await before `session.create`
 * is the semaphore's — keeping session-creation order the handed seat order
 * at any N (ADR-003 §Execution).
 */
async function executeInference(
  opts: RunOptions,
  state: PipelineState,
  handle: StageHandle,
  cfg: PipelineConfig,
  templateText?: string,
): Promise<StageOutcome> {
  const def = stageDef(handle.stage, handle.specialty);
  const template = stageTemplate(def, handle.stage, state);
  const designDirRel = designDirOf(state);
  const designDir = join(opts.repoDir, designDirRel);

  let prompt: string;
  let templateFile: string | null = null;
  let sha: string | undefined;
  if (template) {
    const text = templateText ?? (await loadTemplate(opts.promptsDir, template));
    templateFile = `tools/orchestration/prompts/${template}`;
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
  // A fresh-session attempt 2 otherwise sees a byte-identical prompt with
  // nothing about why attempt 1 failed. The note enters as a labeled log
  // excerpt, bounded by the same `brief(printable(...))` posture as the gate
  // log; quoting forbidden paths in it is safe because prompt text is not
  // audit input. Mirrors the `repairPrompt` precedent for the resumable lane.
  const failureFeedback =
    handle.resume === undefined && handle.failureNote !== undefined
      ? `\n\nPrevious attempt's gate failure (log excerpt): ${handle.failureNote}.\n` +
        'Do not repeat the behavior that caused that failure.'
      : '';
  const fullPrompt = handle.resume
    ? handle.resume.text
    : `${prompt}${failureFeedback}\n\n${replyFormat(schema)}`;

  const requestTimeoutMs = cfg.requestTimeoutMs ?? DEFAULT_REQUEST_TIMEOUT_MS;
  const minNudgeIntervalMs = cfg.minNudgeIntervalMs ?? DEFAULT_MIN_NUDGE_INTERVAL_MS;
  // No git here at all: the caller captured the window's `headBefore` before
  // this prompt and runs the finalize after it.
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
  const auditStart = Date.now();
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
  // The most complete message list seen while polling this attempt; the
  // independence audit reads the tool calls and shells out of it.
  let lastSeen: SessionMessageInfo[] = [];

  // One assistant message == one model turn (verified against the server).
  // Each idle marker closes one prompt (the stage prompt plus any nudges).
  for (;;) {
    let status: 'done' | 'timeout' | 'no-turns';
    for (;;) {
      let data: SessionMessageInfo[];
      try {
        data = await listSessionMessages(opts.client, sessionID, requestTimeoutMs);
        lastSeen = data;
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

    // Review independence: inspect what this attempt's session actually read
    // or ran, not just what its reply claims. Transcripts are out of band
    // for every stage; report files are out of band for review seats —
    // including writes to a seat other than the one the prompt handed it. The
    // permissions say so; this checks up on it. A violation fails the attempt,
    // attempt 2 starts a fresh session (not resumable), then escalate —
    // the same budget as any stage failure.
    // Design 007 §2: a delegation part is exempt on its own text, so the
    // child sessions' deeds are the evidence. The parent list is audited
    // first — when it already fails, the walk is skipped and the attempt
    // fails on what is in hand. Otherwise every descendant list goes through
    // the same matcher, and its violations name the session they came from.
    const auditOptions: IndependenceAuditOptions = {
      since: auditStart,
      reviewStage: handle.stage.endsWith('-review'),
      ownReportPath:
        typeof handle.vars['reportPath'] === 'string' ? handle.vars['reportPath'] : undefined,
    };
    const violations = auditSessionIndependence(lastSeen, auditOptions);
    if (violations.length === 0) {
      let descendants = 0;
      try {
        for await (const child of listChildSessionMessages(
          opts.client,
          sessionID,
          requestTimeoutMs,
        )) {
          descendants += 1;
          for (const violation of auditSessionIndependence(child.messages, auditOptions)) {
            violations.push(`${violation} (via child session ${child.sessionID})`);
          }
        }
        // Tripwire: an empty-but-successful enumeration next to a completed
        // delegation part must not masquerade as a clean audit.
        if (descendants === 0 && hasCompletedDelegation(lastSeen, auditStart)) {
          console.warn(
            `stage ${handle.stageKey}: independence audit enumerated no child sessions of ` +
              `${sessionID} despite a completed delegation part — the deeds of any child ` +
              `this server did not enumerate went unseen`,
          );
        }
      } catch (error) {
        // An infrastructure fault on the walk is not a stage failure (A10):
        // the attempt is audited on its own session alone, and the partial
        // audit is named in the run record.
        violations.length = 0;
        console.warn(
          `stage ${handle.stageKey}: independence audit could not complete the child-session ` +
            `walk for ${sessionID}: ${String(error instanceof Error ? error.message : error)} — ` +
            'auditing this attempt on its own session alone',
        );
        gateLog(state, 'independenceAudit', handle.round, {
          proceed: true,
          reason:
            `child-session walk incomplete for session ${sessionID}: ` +
            `${String(error instanceof Error ? error.message : error)}; ` +
            'the attempt was audited on its own message list alone',
        });
      }
    }
    if (violations.length > 0) {
      console.warn(`stage ${handle.stageKey}: independence violation — ${violations.join('; ')}`);
      outcome = { ok: false, failure: `independence violation: ${violations.join('; ')}` };
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
    `stage ${handle.stageKey}: ${outcome.ok ? 'ok' : `FAIL (${printable(outcome.failure ?? '')})`} ` +
      `${String(Math.round((endedAt - startedAt) / 1000))}s, ${String(nudges)} nudge(s)` +
      `${handle.attempt > 1 ? ', retry' : ''}` +
      `${size ? `, proposal ${String(size.docLines)} lines (+${String(size.added)}/-${String(size.removed)})` : ''}`,
  );

  return { ...outcome, sessionID };
}

// --- the finalize half of the two-phase contract ----------------------------
// `executeInference` runs no git. Everything that mutates the worktree lives
// here, against the window (`headBefore`) the caller captured before the
// prompt: one window per stage on the mutating chain, one per PASS on the
// review fan-out — a per-seat sweep would revert a sibling's in-flight report
// and `git add -A` would sweep a sibling's half-written file into the wrong
// commit.

/** The warnings and the test-bookkeeping an ownership sweep produces. */
function applyOwnershipReport(
  report: OwnershipReport,
  state: PipelineState,
  stageName: string,
): void {
  if (report.reverted.length > 0) {
    console.warn(
      `ownership: reverted ${String(report.reverted.length)} file(s) changed by ${stageName}: ` +
        report.reverted.join(', '),
    );
  }
  for (const c of report.foreign) {
    console.warn(
      `ownership: commit ${c.sha.slice(0, 8)} ("${c.subject}") landed during ${stageName}; ` +
        'left untouched — it is not runner-authored',
    );
  }
  if (
    (stageName === 'implement' || stageName === 'fix') &&
    report.reverted.some((p) => TEST_PATH_RE.test(p))
  ) {
    state.testsNeedUpdate = true;
  }
}

/** One stage's window: today's sweep + commit, skipped when the stage failed. */
function finalizeStage(
  repoDir: string,
  state: PipelineState,
  headBefore: string,
  stageName: string,
  stageKey: string,
  outcome: StageOutcome,
): void {
  if (!outcome.ok) {
    return;
  }
  applyOwnershipReport(enforceOwnership(repoDir, headBefore, stageName, state), state, stageName);
  if (typeof (outcome.reply as { testsNeedUpdate?: boolean })?.testsNeedUpdate === 'boolean') {
    state.testsNeedUpdate = (outcome.reply as { testsNeedUpdate: boolean }).testsNeedUpdate;
  }
  recordStageCommit(
    state,
    commitStage(
      repoDir,
      `${state.nnn}-${state.slug}`,
      stageKey,
      STAGE_COMMIT_TYPE[stageName] ?? 'chore',
      outcome.reply,
    ),
  );
}

/**
 * The combined path a single (non-review) stage has always used: the inference
 * phase, then this stage's own finalize window. Review passes do not use it —
 * they fan out `executeInference` and finalize once, at pass level.
 */
export async function executeStage(
  opts: RunOptions,
  state: PipelineState,
  handle: StageHandle,
  cfg: PipelineConfig,
): Promise<StageOutcome> {
  const stageName = stageDef(handle.stage, handle.specialty).name;
  const headBefore = git(opts.repoDir, ['rev-parse', 'HEAD']).trim();
  const outcome = await executeInference(opts, state, handle, cfg);
  finalizeStage(opts.repoDir, state, headBefore, stageName, handle.stageKey, outcome);
  return outcome;
}

/**
 * One review pass's window, after every seat settled: ONE ownership sweep for
 * the whole pass, and one commit for it — review replies carry no
 * `commitMessage` (VerdictReply), so the message is the fallback shape with
 * the pass's stage name: `docs: <nnn-slug> design-review stage`.
 *
 * `commit` is false on the escalation path, where `finalizeEscalate` mints the
 * terminal commit instead: the sweep still runs, so a sibling's out-of-policy
 * file can never ride into that commit.
 */
function finalizeReviewPass(
  repoDir: string,
  state: PipelineState,
  headBefore: string,
  stageName: string,
  commit: boolean,
): void {
  applyOwnershipReport(enforceOwnership(repoDir, headBefore, stageName, state), state, stageName);
  if (!commit) {
    return;
  }
  recordStageCommit(
    state,
    commitStage(
      repoDir,
      `${state.nnn}-${state.slug}`,
      stageName,
      STAGE_COMMIT_TYPE[stageName] ?? 'chore',
      undefined,
    ),
  );
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
    // A gate reason can carry text the model or the server chose (a stage
    // failure, an independence violation naming a path the session ran). The
    // gate log is printed by `pipeline status` and copied into ESCALATION.md,
    // so no control or escape byte from that text reaches either.
    reason: printable(result.reason),
    at: Date.now(),
  });
}

// --- the review-pass fan-out (ADR-003) -------------------------------------------

interface ReviewPassArgs {
  opts: RunOptions;
  state: PipelineState;
  cfg: PipelineConfig;
  designDir: string;
  /** Registry stage name: `design-review` | `editorial-review` | `code-review`. */
  stage: string;
  round: number;
  /** Funnel pass inside the round; undefined for the single editorial pass. */
  pass?: number;
  /** The seats exactly as `planPass` / `specialtiesFor` handed them, in order. */
  seats: readonly string[];
  varsFor: (specialty: string) => TemplateVars;
  /** HEAD captured at pass entry, before any seat prompted. */
  headBefore: string;
}

/**
 * One review pass on the bounded-concurrency lane: the handed seats launch
 * through a semaphore of `cfg.maxParallelReviews` (1 = the serial lane, the
 * same code path, not a branch), each seat holding its permit across its own
 * retry so total in-flight inference never exceeds N, and the pass finalizes
 * once — one ownership sweep and one commit — after every seat settled and
 * every checkpoint write drained.
 *
 * A seat that fails twice does not call `finalizeEscalate` mid-pass: it records
 * the failure (as today) and raises the pass-level flag. A seat that acquires
 * after that releases its permit and returns without prompting, so no seat
 * today's `StopDriving` would skip ever runs. The reason comes back for
 * `drive` to escalate once; `null` means the pass completed.
 */
async function runReviewPass(args: ReviewPassArgs): Promise<string | null> {
  const { opts, state, cfg } = args;
  if (args.seats.length === 0) {
    return null;
  }
  // DEFAULT_CONFIG is the sole default authority: a config object that omits
  // the field lands on the serial lane, never on `createSemaphore(undefined)`.
  const limit = cfg.maxParallelReviews ?? DEFAULT_CONFIG.maxParallelReviews ?? 1;
  const sem = createSemaphore(limit);
  const writer: CheckpointWriter = createStateWriter(args.designDir);
  let escalation: string | null = null;

  // Every in-pass snapshot is stamped at the moment it is TAKEN, right before
  // it is queued: the writer never re-stamps at rename time (a queued snapshot
  // would masquerade as a newer one), so stamping here is what keeps
  // `.pipeline.json`'s `updatedAt` meaning "when this was written" — and keeps
  // enqueue order and timestamp order the same ordering.
  const enqueueSnapshot = async (): Promise<void> => {
    state.updatedAt = Date.now();
    await writer.enqueue(state);
  };

  // The template is read once for the whole pass, before any seat starts, so
  // the semaphore is the only await ahead of `session.create`: the waiter
  // queue's FIFO order is then also the session-creation order.
  const template = stageDef(args.stage, args.seats[0]).template;
  const templateText = template ? await loadTemplate(opts.promptsDir, template) : undefined;

  const runSeat = async (specialty: string): Promise<void> => {
    const release = await sem.acquire();
    try {
      if (escalation !== null) {
        return;
      }
      const handle: StageHandle = {
        stage: args.stage,
        stageKey: `${args.stage}:${specialty}`,
        attempt: 1,
        round: args.round,
        pass: args.pass,
        specialty,
        vars: args.varsFor(specialty),
      };
      let out = await executeInference(opts, state, handle, cfg, templateText);
      if (!out.ok) {
        state.stageFailures[handle.stageKey] = (state.stageFailures[handle.stageKey] ?? 0) + 1;
        gateLog(state, 'stageFailure', args.round, {
          proceed: false,
          reason: `${handle.stageKey}: ${out.failure}`,
        });
        // Every in-pass checkpoint write goes through the drained writer; a
        // seat calling `saveState` here would race its siblings' renames.
        await enqueueSnapshot();
        if (escalation !== null) {
          return; // a sibling gave up while this seat was answering: spare it
        }
        // A reply failure means the work is usually done and only the closing
        // object was missing: retry in the same session, where the report and
        // the reading are still there. A timeout is different — that session
        // was cancelled as a zombie, so attempt 2 starts fresh. Both attempts
        // run inside this seat's permit.
        const def = stageDef(handle.stage, specialty);
        const resumable = retryInSameSession(out.failure, out.sessionID);
        out = await executeInference(
          opts,
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
              : { failureNote: brief(printable(out.failure ?? 'no reply')) }),
          },
          cfg,
          templateText,
        );
        if (!out.ok) {
          state.stageFailures[handle.stageKey] = (state.stageFailures[handle.stageKey] ?? 0) + 1;
          escalation =
            `stage ${handle.stageKey} failed twice (${out.failure}) — ` +
            'nudge/reply rule: one retry, then escalate';
          await enqueueSnapshot();
          return;
        }
      }
      await enqueueSnapshot();
    } finally {
      release();
    }
  };

  const settled = await Promise.allSettled(args.seats.map((specialty) => runSeat(specialty)));
  // Barrier: settled seats only, then every queued checkpoint write has landed,
  // so the pass's `git add -A` cannot sweep a half-written `.pipeline.json.tmp`.
  await writer.drain();
  for (const s of settled) {
    if (s.status === 'rejected') {
      throw s.reason; // a runner error (template, transcript, checkpoint) ends the run, as today
    }
  }
  // Proposal: "The sweep reverts out-of-policy sibling files and escalation
  // still lands." `enforceOwnership`/`commitStage` shell out to git and throw
  // on any non-zero exit; a throw from the escalation-path sweep would escape
  // `drive` before `finalizeEscalate`, leaving status `running`, no
  // ESCALATION.md, and the run's only terminal record never minted. On the
  // escalation path the sweep is therefore best-effort: a failure is logged
  // and escalation lands anyway. On the success path a runner error still
  // ends the run, as today.
  if (escalation === null) {
    finalizeReviewPass(opts.repoDir, state, args.headBefore, args.stage, true);
  } else {
    try {
      finalizeReviewPass(opts.repoDir, state, args.headBefore, args.stage, false);
    } catch (error) {
      console.warn(
        `review pass ${args.stage}: ownership sweep failed before escalate: ` +
          String(error instanceof Error ? error.message : error),
      );
    }
  }
  return escalation;
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
    await checkpoint(join(opts.repoDir, designDirOf(state)), state);
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
        await checkpoint(designDir, state);
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
              : { failureNote: brief(printable(out.failure ?? 'no reply')) }),
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
          await checkpoint(designDir, state);
          const editorialHead = git(opts.repoDir, ['rev-parse', 'HEAD']).trim();
          const editorialEscalation = await runReviewPass({
            opts: opts as RunOptions,
            state,
            cfg,
            designDir,
            stage: 'editorial-review',
            round: next.round,
            // ADR-002 leaves editorial naming alone: no pass number.
            pass: undefined,
            seats: specialtiesFor('editorial'),
            varsFor: (specialty) =>
              reviewVars(designDirRel, state, 'editorial', next.round, specialty),
            headBefore: editorialHead,
          });
          if (editorialEscalation !== null) {
            await finalizeEscalate(opts.repoDir, state, editorialEscalation, cfg);
            return;
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
          await checkpoint(designDir, state);
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
          await checkpoint(designDir, state);
          continue;
        }
        console.log(
          `review ${phase} r${String(next.round)}c${String(plan.cycle)} ` +
            `(${plan.pass}: ${plan.seats.join(', ')})`,
        );
        await checkpoint(designDir, state);

        // ADR-003: the seats `planPass` handed launch in that order through a
        // semaphore of `cfg.maxParallelReviews`; the whole pass owns ONE git
        // window, captured here, before any seat prompted.
        const passHead = git(opts.repoDir, ['rev-parse', 'HEAD']).trim();
        const passEscalation = await runReviewPass({
          opts: opts as RunOptions,
          state,
          cfg,
          designDir,
          stage: `${phase}-review`,
          round: next.round,
          pass: plan.cycle,
          seats: plan.seats,
          varsFor: (specialty) =>
            reviewVars(designDirRel, state, phase, next.round, specialty, plan),
          headBefore: passHead,
        });
        if (passEscalation !== null) {
          await finalizeEscalate(opts.repoDir, state, passEscalation, cfg);
          return;
        }
        state.lastStage = `${phase}-review`;
        // Pin every seat's latest valid verdict to the revision it was shown;
        // a retry's verdict supersedes the failed attempt it retried.
        const passStamp = isDesign ? (state.designStamp ?? '') : (state.codeStamp ?? '');
        for (const specialty of plan.seats) {
          const reply = lastSeatReply(state, phase, specialty);
          if (reply) {
            recordSeatVerdict(
              state,
              phase,
              specialty,
              seatDecision(reply, cfg),
              passStamp,
              boundedFindings(reply.findings),
            );
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
        await checkpoint(designDir, state);
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
      await checkpoint(designDir, state);
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

/**
 * Caps on what one seat's findings contribute. They are model-authored text,
 * and they are persisted into the seat board — which is cloned into every
 * later checkpoint and re-injected into that seat's own next-pass prompt. One
 * runaway reply would otherwise inflate the checkpoint and every subsequent
 * prompt of that seat forever (security r1 nit: shape-validated is not
 * size-limited). The cap is applied where findings are RECORDED, so the
 * ledger on disk stays bounded; `reviewVars` bounds again on the way out, for
 * ledgers written before the cap existed.
 */
const MAX_RECORDED_FINDINGS = 40;
const MAX_FINDING_FIELD_CHARS = 1_000;

function boundedField(value: string): string {
  const safe = printable(value);
  return safe.length > MAX_FINDING_FIELD_CHARS
    ? `${safe.slice(0, MAX_FINDING_FIELD_CHARS)}… [elided by the checkpoint cap]`
    : safe;
}

function boundedFindings(findings: readonly Finding[] | undefined): Finding[] | undefined {
  if (findings === undefined) {
    return undefined;
  }
  const kept = findings.slice(0, MAX_RECORDED_FINDINGS).map((f) => ({
    severity: f.severity,
    location: boundedField(f.location),
    problem: boundedField(f.problem),
    ...(f.fix === undefined ? {} : { fix: boundedField(f.fix) }),
  }));
  const elided = findings.length - kept.length;
  if (elided > 0) {
    kept.push({
      // A nit never gates: elision is a record, not a dissent.
      severity: 'nit',
      location: '(findings elided)',
      problem: `${String(elided)} further finding(s) elided by the checkpoint cap`,
    });
  }
  return kept;
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
      prevFindings: '(none — the editorial phase is a single language pass per round)',
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
        : `git diff ${state.baseRef ?? 'main'}...HEAD -- ':(exclude)docs/designs'`
      : priorStamp === '' || currentStamp === ''
        ? 'the whole current document (no revision stamp to diff against)'
        : phase === 'design'
          ? `the sections changed since you last reviewed: run ` +
            `\`git diff ${priorStamp} ${currentStamp}\` (both are blob ids of 02-proposal.md), ` +
            'then read the full current document'
          : `git diff ${priorStamp}..HEAD -- ':(exclude)docs/designs', read against the ` +
            'full current document';
  const passNote =
    passKind === 'full'
      ? `full-panel pass c${String(cycle)}`
      : passKind === 'dissent'
        ? `dissent pass c${String(cycle)}: the last revision was written to answer YOUR findings`
        : `ratification pass c${String(cycle)}: revisions landed after you approved; ` +
          'you are checking only that they did not break what you approved';
  // Review independence: the seat's own previous findings come to it through
  // the checkpoint, so it never opens a report — its own or another seat's —
  // and the panel's five reads stay independent. Bounded on the way out too:
  // a ledger written before the record cap existed can still hold a runaway
  // finding, and this text goes straight into the seat's prompt.
  const own = boundedFindings(state.seats?.[phase]?.[specialty]?.findings);
  const prevFindings =
    own !== undefined && own.length > 0
      ? JSON.stringify(own, null, 2)
      : '(none — this is your first pass, or your last verdict held no findings)';
  return {
    designDir: designDirRel,
    sourcePath: state.source.path,
    round,
    specialty,
    scope,
    passNote,
    reportPath: reportPathFor(designDirRel, phase, specialty, round, cycle),
    reviewFiles: '',
    prevFindings,
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
  await checkpoint(designDir, state);
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
  // The reason usually quotes failure text the model or the server chose (a
  // seat failure, an independence violation naming a path). It lands in the
  // gate log, on the operator's terminal, and in ESCALATION.md: keep it on one
  // line and free of control/escape bytes.
  const safeReason = printable(reason);
  state.status = 'escalated';
  gateLog(state, 'escalate', 0, { proceed: false, reason: safeReason });
  const designDirRel = designDirOf(state);
  const designDir = join(repoDir, designDirRel);
  mkdirSync(designDir, { recursive: true });
  const phase = escalationPhase(state);
  // Atomic tmp+rename, like every other checkpoint write: this is the one
  // record the operator must be able to read afterwards, and interrupting a
  // run is most likely exactly when it is being written.
  await checkpoint(designDir, state);
  const reportRel = join(designDirRel, 'ESCALATION.md');
  writeFileSync(
    join(repoDir, reportRel),
    escalationMarkdown(state, phase, cfg.blockingSeverities),
    'utf8',
  );
  // Nothing printed on escalation in run 1: a 47-minute resume that ended
  // escalated looked like a quiet success on the operator's terminal.
  console.error(
    `\npipeline ESCALATED (${phase} phase, after stage ${state.lastStage}): ${safeReason}\n` +
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
      .map((g) => `  ${g.proceed ? 'ok  ' : 'DENY'} ${g.gate} r${g.round}: ${printable(g.reason)}`),
  ];
  return lines.join('\n');
}
