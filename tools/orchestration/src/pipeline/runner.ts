import { spawnSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { basename, join, relative, resolve } from 'node:path';

import type { OpenCodeClient, SessionMessageAssistant } from '@opencode/client';
import { z } from 'zod';

import {
  TEST_PATH_RE,
  allReviewsApproved,
  flipAdrStatus,
  isSemanticCommit,
  nextNnn,
  testsGreen as runChecks,
  sourceAdmissible,
  stagePathAllowed,
} from './gates.js';
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
import { DEFAULT_CONFIG, stageDef, stageReplySchema } from './stages.js';
import type { PipelineConfig } from './stages.js';
import { STATE_FILE, designDirOf, loadState, saveState } from './state-store.js';
import { type ReviewPhase, decideNext, gateName, specialtiesFor } from './states.js';
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
}

export interface ResumeOptions {
  repoDir: string;
  slug: string;
  promptsDir: string;
  client: OpenCodeClient;
  cfg?: PipelineConfig;
  forceStage?: string;
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

function gitOk(repoDir: string, args: string[]): boolean {
  return spawnSync('git', args, { cwd: repoDir }).status === 0;
}

function dirtyFiles(repoDir: string): string[] {
  return git(repoDir, ['status', '--porcelain'])
    .split('\n')
    .filter((line) => line.length > 0)
    .map((line) => line.slice(3).trim());
}

/**
 * Deterministic ownership enforcement: revert every path the stage changed
 * (committed mid-stage or left dirty) that the stage may not touch. Models
 * can route around agent-config permissions (e.g. via subagents), so this
 * runner-side check is the real boundary.
 */
function enforceOwnership(repoDir: string, headBefore: string, stage: string): string[] {
  const committed = gitOk(repoDir, ['diff', '--quiet', headBefore, 'HEAD'])
    ? []
    : git(repoDir, ['diff', '--name-only', headBefore, 'HEAD'])
        .split('\n')
        .filter((l) => l.length > 0);
  const touched = new Set([...committed, ...dirtyFiles(repoDir)]);
  const violations = [...touched].filter((p) => !stagePathAllowed(stage, p));
  if (violations.length === 0) {
    return [];
  }
  for (const p of violations) {
    if (gitOk(repoDir, ['cat-file', '-e', `${headBefore}:${p}`])) {
      git(repoDir, ['restore', `--source=${headBefore}`, '--staged', '--worktree', '--', p]);
    } else {
      gitOk(repoDir, ['rm', '-f', '-r', '--', p]);
      rmSync(resolve(repoDir, p), { force: true });
    }
  }
  if (git(repoDir, ['rev-parse', 'HEAD']).trim() !== headBefore) {
    // The model committed violations mid-stage: revert commit so the committed
    // tree honors ownership too.
    git(repoDir, ['add', '-A']);
    if (!gitOk(repoDir, ['diff', '--cached', '--quiet'])) {
      git(repoDir, ['commit', '-m', `fix: revert ${stage} stage changes outside its ownership`]);
    }
  }
  return violations;
}

/** Commit exactly the current stage's changes; model message or fallback. */
function commitStage(
  repoDir: string,
  slug: string,
  stageKey: string,
  fallbackType: string,
  reply: unknown,
): void {
  if (dirtyFiles(repoDir).length === 0) {
    return;
  }
  git(repoDir, ['add', '-A']);
  if (gitOk(repoDir, ['diff', '--cached', '--quiet'])) {
    return;
  }
  const proposed = (reply as { commitMessage?: string } | undefined)?.commitMessage;
  const message =
    proposed && isSemanticCommit(proposed)
      ? proposed
      : `${fallbackType}: ${slug} ${stageKey} stage`;
  git(repoDir, ['commit', '-m', message]);
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

interface RunIdentity {
  slug: string;
  nnn: string;
  kind: 'adr' | 'brief';
  branch: string;
}

function slugFromSource(sourcePath: string): string {
  return basename(sourcePath, '.md').replace(/^\d{3}-/, '');
}

function newIdentity(repoDir: string, sourcePath: string): RunIdentity {
  const rel = relative(repoDir, sourcePath);
  const kind: 'adr' | 'brief' = rel.startsWith(join('docs', 'adrs')) ? 'adr' : 'brief';
  const m = /^docs\/(adrs|prompts)\/(\d{3})-/.exec(rel);
  const slug = slugFromSource(sourcePath);
  let nnn: string;
  if (kind === 'adr' && m?.[2]) {
    nnn = m[2];
  } else {
    const existing = [
      ...listDir(join(repoDir, 'docs', 'adrs')),
      ...listDir(join(repoDir, 'docs', 'designs')),
    ];
    nnn = nextNnn(existing);
  }
  return { slug, nnn, kind, branch: `pipeline/${nnn}-${slug}` };
}

function listDir(dir: string): string[] {
  try {
    return readdirSync(dir);
  } catch {
    return [];
  }
}

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
    'you are going to do; just do it. Your final message must contain exactly one',
    'JSON object and nothing else: no markdown fences, no prose, no preamble.',
    `JSON Schema: ${jsonSchema}`,
  ].join('\n');
}

// Small models routinely end a turn mid-plan ("Now let me verify...") and go
// idle without ever emitting the required reply. The runner nudges the SAME
// session (idle sessions accept more prompts) instead of failing the stage.
const MAX_NUDGES = 2;
const NUDGE_CONTINUE = [
  'You stopped before finishing the stage. Do not narrate plans.',
  'Continue the work now with tool calls until it is complete, then reply',
  'with ONLY the JSON object required by the schema above.',
].join(' ');

// --- stage execution ----------------------------------------------------------

interface StageOutcome {
  ok: boolean;
  reply?: unknown;
  failure?: string;
}

interface StageHandle {
  /** Registry stage name, e.g. `design-review`. */
  stage: string;
  stageKey: string;
  /** 1 for the first attempt, 2 for the retry (transcript disambiguation). */
  attempt: number;
  round: number;
  specialty?: string;
  vars: TemplateVars;
}

async function executeStage(
  opts: RunOptions,
  state: PipelineState,
  handle: StageHandle,
  cfg: PipelineConfig,
): Promise<StageOutcome> {
  const def = stageDef(handle.stage, handle.specialty, cfg);
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
  const fullPrompt = `${prompt}\n\n${replyFormat(schema)}`;

  const headBefore = git(opts.repoDir, ['rev-parse', 'HEAD']).trim();
  const session = await opts.client.session.create({
    title: `pipeline ${state.slug}: ${handle.stageKey}`,
    agent: def.agent,
    // Bind the session to the repo worktree so file edits, skills, and
    // agent discovery all happen in the right location.
    location: { directory: opts.repoDir },
  });
  if (!session.id) {
    return { ok: false, failure: 'server returned no session id' };
  }
  await opts.client.session.prompt({ sessionID: session.id, text: fullPrompt });

  const startedAt = Date.now();
  const deadline = startedAt + cfg.stageTimeoutMs;
  let rawText = '';
  let model: ModelInfo | undefined;
  let nudges = 0;
  let outcome: StageOutcome;

  // One assistant message == one model turn (verified against the server).
  // Each idle marker closes one prompt (the stage prompt plus any nudges).
  for (;;) {
    let status: 'done' | 'exhausted' | 'timeout' | 'no-turns';
    for (;;) {
      const { data } = await opts.client.message.list({ sessionID: session.id });
      const assistants = data.filter((m) => m.type === 'assistant');
      if (assistants.length >= def.steps) {
        try {
          await opts.client.session.interrupt({ sessionID: session.id });
        } catch {
          /* best-effort abort */
        }
        status = 'exhausted';
        break;
      }
      const idles = data.filter((m) => m.type === 'idle').length;
      if (idles >= nudges + 1 && assistants.length === 0) {
        // A bound-but-nonexistent agent produces idle markers and zero model
        // turns; nudging would be pointless. Fail fast and say why.
        status = 'no-turns';
        break;
      }
      if (idles >= nudges + 1) {
        // Small models routinely end a stage with a tool-call turn and no
        // trailing text; the reply is the LAST assistant message that
        // carries text parts, not strictly the final one.
        const lastTextual = [...assistants]
          .reverse()
          .find((m: SessionMessageAssistant) =>
            (m.content ?? []).some((c: { type: string }) => c.type === 'text'),
          );
        rawText = (lastTextual?.content ?? [])
          .filter((p): p is { type: 'text'; text: string } => p.type === 'text')
          .map((p) => p.text)
          .join('');
        model = lastTextual?.model ?? assistants.at(-1)?.model;
        status = 'done';
        break;
      }
      if (Date.now() >= deadline) {
        status = 'timeout';
        break;
      }
      await new Promise((r) => setTimeout(r, cfg.pollIntervalMs));
    }

    if (status === 'exhausted') {
      outcome = { ok: false, failure: `steps-exhausted (cap ${String(def.steps)})` };
      break;
    }
    if (status === 'timeout') {
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

    const parsed = normalizeReplyShape(extractJsonObject(rawText));
    if (parsed === undefined) {
      if (nudges < MAX_NUDGES) {
        nudges += 1;
        console.warn(
          `stage ${handle.stageKey}: narration instead of reply; nudge ${String(nudges)}/${String(MAX_NUDGES)}`,
        );
        await opts.client.session.prompt({
          sessionID: session.id,
          text: `${NUDGE_CONTINUE}\n\n${replyFormat(schema)}`,
        });
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
    if (nudges < MAX_NUDGES) {
      nudges += 1;
      console.warn(
        `stage ${handle.stageKey}: reply failed schema; nudge ${String(nudges)}/${String(MAX_NUDGES)}`,
      );
      await opts.client.session.prompt({
        sessionID: session.id,
        text: [
          `Your reply did not validate: ${checked.error.issues[0]?.message ?? 'schema mismatch'}.`,
          'Reply again with ONLY the JSON object matching the schema. No other text.',
          '',
          replyFormat(schema),
        ].join(' '),
      });
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
    agent: def.agent,
    model,
    templateFile,
    templateSha256: sha,
    startedAt,
    endedAt,
    prompt: fullPrompt,
    rawReply: rawText,
  });

  const result: StageResult = {
    stage: handle.stageKey,
    round: handle.round,
    sessionID: session.id,
    agent: def.agent,
    at: endedAt,
    reply: outcome.reply,
    transcript: relative(opts.repoDir, transcript),
  };
  state.results.push(result);

  if (outcome.ok) {
    const violations = enforceOwnership(opts.repoDir, headBefore, def.name);
    if (violations.length > 0) {
      console.warn(
        `ownership: reverted ${String(violations.length)} file(s) changed by ${def.name}`,
      );
    }
    if (
      (def.name === 'implement' || def.name === 'fix') &&
      violations.some((p) => TEST_PATH_RE.test(p))
    ) {
      state.testsNeedUpdate = true;
    }
    if (typeof (outcome.reply as { testsNeedUpdate?: boolean })?.testsNeedUpdate === 'boolean') {
      state.testsNeedUpdate = (outcome.reply as { testsNeedUpdate: boolean }).testsNeedUpdate;
    }
    commitStage(
      opts.repoDir,
      `${state.nnn}-${state.slug}`,
      handle.stageKey,
      STAGE_COMMIT_TYPE[def.name] ?? 'chore',
      outcome.reply,
    );
  }
  return outcome;
}

// --- escalation ---------------------------------------------------------------

function openFindingsOfRound(state: PipelineState, phase: ReviewPhase, round: number) {
  const out: {
    specialty: string;
    severity: string;
    location: string;
    problem: string;
    fix?: string;
  }[] = [];
  for (const r of state.results) {
    if (r.stage.startsWith(`${phase}-review:`) && r.round === round && r.reply) {
      const verdict = r.reply as {
        findings?: { severity: string; location: string; problem: string; fix?: string }[];
      };
      const specialty = r.stage.split(':')[1] ?? '?';
      for (const f of verdict.findings ?? []) {
        out.push({ specialty, ...f });
      }
    }
  }
  return out;
}

function escalationMarkdown(state: PipelineState): string {
  const denied = state.gateLog.filter((g) => !g.proceed).slice(-8);
  const findings = openFindingsOfRound(
    state,
    state.codeRound > 0 && state.lastStage.includes('code') ? 'code' : 'design',
    state.lastStage.includes('code') ? state.codeRound : state.designRound,
  );
  return [
    `# ESCALATED: ${state.nnn}-${state.slug}`,
    '',
    `Run stopped at stage \`${state.lastStage}\` (status: ${state.status}).`,
    '',
    '## Recent gate denials',
    ...denied.map((g) => `- ${g.gate} (r${g.round}): ${g.reason}`),
    '',
    '## Open findings from the last review round',
    ...(findings.length > 0
      ? findings.map((f) => `- [${f.severity}] (${f.specialty}) ${f.location} — ${f.problem}`)
      : ['- (none recorded)']),
    '',
    'Fix the underlying cause (usually the brief or the proposal), then run:',
    `\`pipeline resume ${state.nnn}-${state.slug}\``,
    '',
  ].join('\n');
}

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
  const id = newIdentity(opts.repoDir, sourceAbs);

  acquireLock(opts.repoDir, id.slug, 'start');
  try {
    const state = freshState(id, relative(opts.repoDir, sourceAbs));
    if (opts.forceStage) {
      state.forceNext = opts.forceStage;
    }
    state.baseRef = git(opts.repoDir, ['rev-parse', 'HEAD']).trim();
    ensureBranch(opts.repoDir, state.branch);
    await saveState(join(opts.repoDir, designDirOf(state)), state);
    const verdict = await sourceAdmissible(sourceAbs, id.kind);
    gateLog(state, 'sourceAdmissible', 0, verdict);
    if (!verdict.proceed) {
      finalizeEscalate(opts.repoDir, state, `source not admissible: ${verdict.reason}`);
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
  acquireLock(opts.repoDir, state.slug, 'resume');
  try {
    state.status = 'running';
    if (opts.forceStage) {
      state.forceNext = opts.forceStage;
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
      finalizeEscalate(opts.repoDir, state, next.reason);
      return;
    }

    const runWithRetry = async (handle: StageHandle): Promise<void> => {
      let out = await executeStage(opts as RunOptions, state, { ...handle, attempt: 1 }, cfg);
      if (!out.ok) {
        gateLog(state, 'stageFailure', handle.round, {
          proceed: false,
          reason: `${handle.stageKey}: ${out.failure}`,
        });
        await saveState(designDir, state);
        out = await executeStage(opts as RunOptions, state, { ...handle, attempt: 2 }, cfg);
        if (!out.ok) {
          finalizeEscalate(
            opts.repoDir,
            state,
            `stage ${handle.stageKey} failed twice (${out.failure}) — steps/reply cap rule: one retry, then escalate`,
          );
          throw new StopDriving();
        }
      }
    };

    try {
      if (next.kind === 'review') {
        if (next.phase === 'design') {
          state.designRound = next.round;
          state.designBudgetUsed += 1;
        } else if (next.phase === 'code') {
          state.codeRound = next.round;
          state.codeBudgetUsed += 1;
        } else {
          state.designRound = next.round; // editorial continues the rN sequence
          if (proposalSnapshot !== null) {
            writeEditorialDiff(designDir, state.slug, next.round, proposalSnapshot, opts.repoDir);
          }
        }
        await saveState(designDir, state);

        for (const specialty of specialtiesFor(next.phase)) {
          await runWithRetry({
            stage: `${next.phase}-review`,
            stageKey: `${next.phase}-review:${specialty}`,
            attempt: 1,
            round: next.round,
            specialty,
            vars: reviewVars(designDirRel, state, next.phase, next.round, specialty),
          });
          await saveState(designDir, state);
        }
        state.lastStage = `${next.phase}-review`;
        const g = allReviewsApproved(
          state,
          next.phase,
          next.round,
          specialtiesFor(next.phase),
          cfg,
        );
        gateLog(state, gateName(next.phase), next.round, g);
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
      await runWithRetry({
        stage: name,
        stageKey: name,
        attempt: 1,
        round: name === 'proposal-revise' ? state.designRound : state.codeRound,
        vars: stageVars(designDirRel, state, name),
      });
      state.lastStage = name;

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

function stageVars(designDirRel: string, state: PipelineState, name: string): TemplateVars {
  const round = name === 'proposal-revise' ? state.designRound : state.codeRound;
  const reviewFiles =
    name === 'proposal-revise'
      ? reviewReportList(designDirRel, 'design', state.designRound, specialtiesFor('design'))
      : name === 'fix' || name === 'test-update'
        ? reviewReportList(designDirRel, 'code', state.codeRound, specialtiesFor('code'))
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
): TemplateVars {
  const scope =
    phase === 'design'
      ? 'the whole document'
      : phase === 'editorial'
        ? `the diff file ${designDirRel}/reviews/design/editorial-r${round}.diff`
        : `git diff ${state.baseRef ?? 'main'}...HEAD`;
  return {
    designDir: designDirRel,
    sourcePath: state.source.path,
    round,
    specialty,
    scope,
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
  writeFileSync(join(designDir, 'reviews', 'design', `editorial-r${round}.diff`), out, 'utf8');
}

// --- finalization -----------------------------------------------------------------

function saveStateSync(designDir: string, state: PipelineState): void {
  state.updatedAt = Date.now();
  writeFileSync(join(designDir, STATE_FILE), `${JSON.stringify(state, null, 2)}\n`, 'utf8');
}

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
  commitStage(repoDir, `${state.nnn}-${state.slug}`, 'done', 'docs', {
    commitMessage: `docs: ${state.nnn}-${state.slug} pipeline complete`,
  });
}

function finalizeEscalate(repoDir: string, state: PipelineState, reason: string): void {
  state.status = 'escalated';
  gateLog(state, 'escalate', 0, { proceed: false, reason });
  const designDir = join(repoDir, designDirOf(state));
  mkdirSync(designDir, { recursive: true });
  // Persist the terminal state *before* anything else can fail: `pipeline
  // status` must never read a stale 'running' checkpoint after escalation.
  saveStateSync(designDir, state);
  writeFileSync(join(designDir, 'ESCALATION.md'), escalationMarkdown(state), 'utf8');
  commitStage(repoDir, `${state.nnn}-${state.slug}`, 'escalate', 'docs', {
    commitMessage: `docs: ${state.nnn}-${state.slug} pipeline escalated`,
  });
}

/** Human-facing explanation for `pipeline status`, from the gate log only. */
export function renderStatus(state: PipelineState): string {
  const lines = [
    `run:     ${state.nnn}-${state.slug} (${state.status})`,
    `source:  ${state.source.kind}: ${state.source.path}`,
    `branch:  ${state.branch}`,
    `stages:  last=${state.lastStage || '(none)'} designRound=${state.designRound} codeRound=${state.codeRound}`,
    `budgets: design ${state.designBudgetUsed} used, code ${state.codeBudgetUsed} used`,
    'recent gates:',
    ...state.gateLog
      .slice(-6)
      .map((g) => `  ${g.proceed ? 'ok  ' : 'DENY'} ${g.gate} r${g.round}: ${g.reason}`),
  ];
  return lines.join('\n');
}
