import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs';
import { readFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, relative, resolve } from 'node:path';

import type { OpenCodeClient, SessionMessageAssistant, SessionMessageInfo } from '@opencode/client';
import { z } from 'zod';

import { basicAuthHeader } from '../opencode-runner.js';
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
import { applyMoves, validateMoves } from './moves.js';
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
  CheckpointWriteFailure,
  type CheckpointWriter,
  createStateWriter,
  designDirOf,
  holdsRunnerBytes,
  loadState,
  recordRunnerDelete,
  recordRunnerWrite,
  runnerBytesFor,
  saveState,
} from './state-store.js';
import {
  type ReviewPhase,
  decideNext,
  gateName,
  reviewBudgetUsed,
  specialtiesFor,
} from './states.js';
import {
  ESCALATION_FILE,
  type Finding,
  LOCK_FILE,
  type LockInfo,
  type MovePair,
  type PipelineState,
  STATE_FILE,
  type StageResult,
  namesReservedSegment,
} from './types.js';

// ---------------------------------------------------------------------------
// The inference lane. Mutating stages keep exactly one session at a time, each
// a fresh top-level session bound to its stage agent. A `kind: 'review'` pass
// fans its seats out through a semaphore of `cfg.maxParallelReviews` (default
// 1 — the same seats, strictly one at a time) and finalizes once per pass.
// ---------------------------------------------------------------------------

/**
 * The server connection the client was created with. The child-session walk's
 * third enumeration route — a direct `GET /session/:id/children` (compat
 * matrix: "OpenCode server/SDK without a children method") — hits this
 * baseUrl with the same basic-auth header `createClient` sends. In-memory
 * run options only; nothing here enters the checkpoint.
 */
export interface ServerConnection {
  baseUrl: string;
  username?: string;
  password?: string;
}

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
  /** Connection for the child walk's direct-HTTP enumeration fallback. */
  server?: ServerConnection;
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
  /** Connection for the child walk's direct-HTTP enumeration fallback. */
  server?: ServerConnection;
}

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
    // ADR-006 §1: the sweep is author-agnostic, so it pairs the reserved-name
    // rule with an authorship test. A match still holding the bytes the
    // runner last wrote (or absent by the runner's own hand — the resume
    // marker delete) is the runner's own work: keep it, so boundary commits
    // advance the committed checkpoint instead of restoring `headBefore`.
    if (holdsRunnerBytes(resolve(repoDir, p))) {
      continue;
    }
    // A directory planted at a reserved path (the checkpoint replacement):
    // clear it first so the restore can land the committed file again and no
    // planted byte reaches a `git add -A`.
    const abs = resolve(repoDir, p);
    if (namesReservedSegment(p) && existsSync(abs) && statSync(abs).isDirectory()) {
      rmSync(abs, { recursive: true, force: true });
    }
    if (gitOk(repoDir, ['cat-file', '-e', `${headBefore}:${p}`])) {
      git(repoDir, ['restore', `--source=${headBefore}`, '--staged', '--worktree', '--', p]);
    } else {
      // Correctness r1, finding 2: a tampered reserved path with nothing
      // committed at it yet — a new run's untracked checkpoint — is the run's
      // only copy: §1 promises the sweep "removes it and restores the file",
      // so write back the bytes the runner last recorded instead of deleting
      // what the run must resume from.
      const recorded = namesReservedSegment(p) ? runnerBytesFor(abs) : undefined;
      if (recorded !== undefined) {
        writeFileSync(abs, recorded, 'utf8');
      } else {
        gitOk(repoDir, ['rm', '-f', '--', p]);
        try {
          rmSync(resolve(repoDir, p), { force: true });
        } catch (error) {
          // Deleting an out-of-ownership path must never take the run down.
          console.warn(`ownership: could not remove ${p}: ${(error as Error).message}`);
        }
      }
    }
    reverted.push(p);
  }
  if (reverted.length > 0 && git(repoDir, ['rev-parse', 'HEAD']).trim() !== headBefore) {
    // Committed violations: revert commit so the committed tree honors
    // ownership too. The paths go in the commit body: a count alone made the
    // 10:53 event in run 1 unreconstructable.
    stageAll(repoDir);
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

/**
 * Stage everything the run owns, and only that.
 *
 * `LOCK_FILE` is a process artifact, never a tree artifact: the repo normally
 * gitignores it, but the §1 fence and the §4 lock cannot depend on the
 * operator's ignore file. In a repo that does not ignore it, a plain
 * `git add -A` would commit the run's own lock and then leave its deletion
 * dirty at the end — an unowned change in every boundary gate. So the lock is
 * unstaged straight after the staging pass: nothing between the two calls is
 * committed, so the path never reaches a commit or the run's terminal record.
 * (A pathspec exclusion is not the tool here: git exits non-zero when a
 * literal pathspec names an ignored, untracked file, which is exactly the
 * normal state of the lock.)
 */
function stageAll(repoDir: string): void {
  git(repoDir, ['add', '-A']);
  gitOk(repoDir, ['reset', '-q', '--', LOCK_FILE]);
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
  stageAll(repoDir);
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

/**
 * Atomic exclusive create (ADR-006 §4, A6): the old `existsSync`+`writeFileSync`
 * pair was check-then-write and admitted two winners in a race. `'wx'` is one
 * atomic create, so on local filesystems exactly one pipeline process takes
 * the lock. On `EEXIST` apply today's pid-liveness logic unchanged: a live
 * holder throws with today's fatal message, a dead holder is cleared with
 * today's warning and one `'wx'` retry; a racing loser re-reads and throws.
 * The stale-clear keeps a narrow race — the lock-ID nonce that would close it
 * is Deferred.
 */
export function acquireLock(repoDir: string, slug = 'unknown', stage = 'start'): void {
  const path = join(repoDir, LOCK_FILE);
  const info: LockInfo = { slug, pid: process.pid, stage, at: Date.now() };
  const bytes = JSON.stringify(info);
  // The lock is a reserved name (§1 fence), so the ownership sweep sees it
  // whenever the repo does not gitignore it. Recording the bytes this process
  // wrote is what tells the sweep "runner-authored": it leaves the lock alone
  // mid-run, and restores these exact bytes if a stage deletes or rewrites it.
  try {
    writeFileSync(path, bytes, { flag: 'wx' });
    recordRunnerWrite(path, bytes);
    return;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'EEXIST') {
      throw error;
    }
  }
  const holderOf = (): LockInfo | null => {
    try {
      return JSON.parse(readFileSync(path, 'utf8')) as LockInfo;
    } catch {
      return null;
    }
  };
  const holder = holderOf();
  if (holder && pidAlive(holder.pid)) {
    throw new Error(
      `another pipeline run holds the lock: slug=${holder.slug} stage=${holder.stage} pid=${holder.pid}`,
    );
  }
  // Stale lock: cleared with a warning, per ADR-001.
  console.warn(
    `clearing stale pipeline lock held by ${holder?.slug ?? 'unknown'} (pid ${holder?.pid ?? '?'})`,
  );
  try {
    unlinkSync(path);
  } catch {
    /* already gone */
  }
  try {
    writeFileSync(path, bytes, { flag: 'wx' });
    recordRunnerWrite(path, bytes);
  } catch {
    // A racing winner re-took the path between the clear and the retry:
    // re-read the holder and refuse like a live one.
    const rival = holderOf();
    throw new Error(
      `another pipeline run holds the lock: slug=${rival?.slug ?? 'unknown'} ` +
        `stage=${rival?.stage ?? 'unknown'} pid=${String(rival?.pid ?? '?')}`,
    );
  }
}

function releaseLock(repoDir: string): void {
  const path = join(repoDir, LOCK_FILE);
  try {
    unlinkSync(path);
  } catch {
    /* already gone */
  }
  // The run's own last act on the path: recorded so the §1 sweep reads its
  // absence as runner-authored. A stage that re-creates the lock is a content
  // mismatch against the recorded bytes and is still reverted.
  recordRunnerDelete(path);
}

/**
 * ADR-006 §4, security r1 finding 4: re-assert single-flight at every stage
 * boundary. `LOCK_FILE` is in the reserved-segment set (types.ts), so a stage
 * write to it is an ownership violation the sweep reverts — the bytes recorded
 * at `acquireLock`/`ensureLockHeld` are what marks a match runner-authored —
 * and `stageAll` never stages the path, so committing the lock is not a route
 * to trusting it either. The boundary re-assertion stands on its own: during a
 * stage window a coder-class stage can still delete or rewrite it before any
 * sweep runs, after which the next `acquireLock` would admit a second writer
 * or a planted live-pid holder could block `resume`. A lock that is missing,
 * unreadable, or names a dead pid is re-taken with the same `'wx'` exclusive
 * create as `acquireLock` (dead-holder posture unchanged); a lock held live by
 * another pid means this run has already lost single-flight, and it refuses to
 * continue rather than run unlocked beside a rival.
 */
function ensureLockHeld(repoDir: string, slug: string): void {
  const path = join(repoDir, LOCK_FILE);
  if (existsSync(path)) {
    let holder: LockInfo | null;
    try {
      holder = JSON.parse(readFileSync(path, 'utf8')) as LockInfo;
    } catch {
      holder = null;
    }
    if (holder !== null && holder.pid === process.pid) {
      return;
    }
    if (holder !== null && pidAlive(holder.pid)) {
      throw new Error(
        `lost the pipeline lock at a stage boundary: another run holds it ` +
          `(slug=${holder.slug} stage=${holder.stage} pid=${String(holder.pid)})`,
      );
    }
    // Missing bytes to trust: clear and re-take, the same stale-holder
    // posture as `acquireLock`.
    try {
      unlinkSync(path);
    } catch {
      /* already gone */
    }
  }
  const info: LockInfo = { slug, pid: process.pid, stage: 'boundary', at: Date.now() };
  const bytes = JSON.stringify(info);
  try {
    writeFileSync(path, bytes, { flag: 'wx' });
    recordRunnerWrite(path, bytes);
  } catch {
    // Lost the race to a rival that took the path between the clear and the
    // re-take: refuse to continue — two writers is exactly what §4 forbids.
    throw new Error(
      'lost the pipeline lock at a stage boundary: another run re-took it; refusing to continue',
    );
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
// `session.list` indexed by `parentID` (capped crawl, see below), else a
// direct `GET /session/:id/children` against the configured server
// connection with the client's auth headers — all three exist server-side
// (A3), so the walk always has a route (the compat matrix's "not whether").
// Only when no connection was configured either does the walk throw; the
// audit call site catches that and proceeds with the gap logged — a stage
// failure on an infrastructure fault is not what this audit is for (the
// permission fence stays the primary control).
//
// Cost rules (perf r1c0 #1–#4): the call site only starts a walk when the
// attempt's own list holds a `subagent` part, and at most once per growth of
// that count, so attempts that delegated nothing pay nothing and a nudge
// loop cannot pay the walk nine times; the paged crawl carries a page
// budget and a repeated-cursor stop; and every request inside the walk is
// clamped to the stage deadline — an expired walk throws into the call
// site's caught partial-audit path, naming the truncation in the gate line.

interface SessionChildApi {
  children?: (input: { sessionID: string }, options?: { signal: AbortSignal }) => Promise<unknown>;
  list?: (input?: Record<string, unknown>, options?: { signal: AbortSignal }) => Promise<unknown>;
}

/**
 * Cap on the paged `session.list` crawl. The server's session store grows
 * with every run this location has ever had; the walk's budget does not.
 * Hitting the cap throws into the call site's caught partial-audit path
 * instead of spinning (perf r1c0 #3).
 */
const MAX_SESSION_LIST_PAGES = 50;

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

/**
 * The compat matrix's third enumeration route: a direct
 * `GET /session/:id/children` against the configured `baseUrl` with the
 * same basic-auth header `createClient` sends. The path carries the v2
 * client's `/api` prefix (see the `DEFAULT_BASE_URL` note in
 * `opencode-runner.ts`), so it hits the same server surface the SDK does.
 * The response is unvalidated until `sessionEntries` shapes it.
 */
async function directChildSessions(
  connection: ServerConnection,
  sessionID: string,
  signal: AbortSignal,
): Promise<unknown> {
  const url = new URL(`/api/session/${encodeURIComponent(sessionID)}/children`, connection.baseUrl);
  const headers = basicAuthHeader({
    username: connection.username,
    password: connection.password,
  });
  const response = await fetch(url, { ...(headers === undefined ? {} : { headers }), signal });
  if (!response.ok) {
    throw new Error(`GET ${url.pathname} responded ${String(response.status)}`);
  }
  return await response.json();
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

export interface ChildWalkOptions {
  /** Server connection enabling the direct-HTTP enumeration route. */
  connection?: ServerConnection;
  /** Stage deadline (wall clock): the walk clamps every request to what it leaves. */
  deadline?: number;
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
  options: ChildWalkOptions = {},
): AsyncGenerator<{ sessionID: string; messages: SessionMessageInfo[] }> {
  const api = client.session as unknown as SessionChildApi;
  const visited = new Set<string>([sessionID]);
  const queue: string[] = [];
  // `bounded()` caps one round-trip; the deadline caps the whole walk
  // (perf r1c0 #4 — "the caller still owns the stage deadline", and here it
  // finally exercises that ownership). An expired budget throws into the
  // call site's caught partial-audit path, naming the truncation.
  const budgetMs = (): number => {
    if (options.deadline === undefined) {
      return requestTimeoutMs;
    }
    const remaining = options.deadline - Date.now();
    if (remaining <= 0) {
      throw new Error(`the child-session walk of ${sessionID} was truncated at the stage deadline`);
    }
    return Math.min(requestTimeoutMs, remaining);
  };
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
      const timeoutMs = budgetMs();
      let listed: unknown;
      try {
        listed = await bounded(() =>
          children({ sessionID: parent }, { signal: AbortSignal.timeout(timeoutMs) }),
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
          messages: await descendantMessages(client, child.id, budgetMs()),
        };
        queue.push(child.id);
      }
    }
    return;
  }
  const list = typeof api.list === 'function' ? api.list.bind(api) : undefined;
  if (list !== undefined) {
    // One paged enumeration for the whole walk (not one per node), indexed by
    // `parentID` — the same `cursor.next` crawl as `listSessionMessages`,
    // plus the bounds the pre-existing crawl lacks (sec r1c0 #4, perf r1c0
    // #3): a repeated cursor stops the crawl, and a page budget keeps its
    // request count and map size off the server's total session count.
    const byParent = new Map<string, string[]>();
    let cursor: string | undefined;
    let pages = 0;
    for (;;) {
      const timeoutMs = budgetMs();
      let page: unknown;
      try {
        page = await bounded(() =>
          list(
            { limit: 200, ...(cursor === undefined ? {} : { cursor }) },
            { signal: AbortSignal.timeout(timeoutMs) },
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
      if (next === cursor) {
        throw new Error(
          `listing sessions to find the children of ${sessionID}: ` +
            'the server repeated the same cursor.next — the crawl stopped',
        );
      }
      pages += 1;
      if (pages >= MAX_SESSION_LIST_PAGES) {
        throw new Error(
          `listing sessions to find the children of ${sessionID}: the session store ` +
            `outgrew the ${String(MAX_SESSION_LIST_PAGES)}-page crawl budget`,
        );
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
          messages: await descendantMessages(client, childID, budgetMs()),
        };
        queue.push(childID);
      }
    }
    return;
  }
  if (options.connection !== undefined) {
    // Third route (compat matrix): a client with neither SDK method still
    // gets the walk through the server endpoint (A3), so the enumeration
    // route is never "whether", only which.
    const connection = options.connection;
    queue.push(sessionID);
    for (let head = 0; head < queue.length; head += 1) {
      const parent = queue[head];
      if (parent === undefined) {
        continue;
      }
      const timeoutMs = budgetMs();
      let listed: unknown;
      try {
        listed = await bounded(() =>
          directChildSessions(connection, parent, AbortSignal.timeout(timeoutMs)),
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
          messages: await descendantMessages(client, child.id, budgetMs()),
        };
        queue.push(child.id);
      }
    }
    return;
  }
  throw new Error(
    `enumerating the children of ${sessionID}: the client exposes neither session.children ` +
      'nor session.list, and no server connection was configured for the direct route',
  );
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
 * Every `subagent` tool part in the list — any status, any age. The walk's
 * guard (perf r1c0 #1): a list holding none has no delegation whose deeds
 * could go unseen, so no network walk is worth paying. Restricting the guard
 * to completed in-window parts would open a detection gap — a resumed
 * attempt's part sits pre-`since` while its child's deeds sit in-window, and
 * a still-running child is exactly §2's Risks row.
 */
function countDelegationParts(messages: readonly SessionMessageInfo[]): number {
  let count = 0;
  for (const m of messages) {
    if (m.type !== 'assistant') {
      continue;
    }
    for (const part of m.content ?? []) {
      if (part.type === 'tool' && part.name === 'subagent') {
        count += 1;
      }
    }
  }
  return count;
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
 * ADR-005: a schema-valid reply that carries a non-empty `moves` list is only
 * accepted once `validateMoves` clears the whole list against `headBefore` and
 * the disk. The issue text (including one from a rule that threw internally)
 * comes back here so the caller can put it on the existing invalid-reply path;
 * a stage with no declared moves is unaffected. A moves-bearing reply whose
 * window has no base revision fails closed: without `headBefore` the list
 * cannot be validated, and accepting it would apply it in finalize unvalidated.
 */
function movesValidationIssue(
  opts: RunOptions,
  stageName: string,
  headBefore: string | undefined,
  reply: unknown,
): string | null {
  const moves = (reply as { moves?: MovePair[] } | undefined)?.moves;
  if (!Array.isArray(moves) || moves.length === 0) {
    return null;
  }
  if (headBefore === undefined) {
    return `${stageName} declared moves without a base revision to validate them against`;
  }
  try {
    return validateMoves(opts.repoDir, headBefore, stageName, moves);
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
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
 *
 * `headBefore` is handed only by the single-stage path (`executeStage`): a
 * reply that declares `moves` is validated against it before it is accepted,
 * which is why the inference phase needs the window's base revision. Review
 * schemas carry no moves, so the pass fan-out passes nothing; a moves-bearing
 * reply that arrives without a base revision is rejected rather than applied
 * unvalidated in finalize.
 */
async function executeInference(
  opts: RunOptions,
  state: PipelineState,
  handle: StageHandle,
  cfg: PipelineConfig,
  templateText?: string,
  headBefore?: string,
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
  // Assistant-message count observed right after the stage (or resume) prompt
  // goes in. An idle marker closes a prompt only once the session has actually
  // produced a new turn: without growth past this baseline the poll below would
  // re-read the previous reply and burn the whole nudge budget in seconds on a
  // slow backend.
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
  // The moves issue this attempt was last nudged for. A session that answers the
  // moves nudge with narration, or with nothing at all, still died on that list:
  // the exhausted-nudge branch below would label it `invalid-reply: not JSON` and
  // the gate log and repair prompt would never name the pair. The proposal names
  // the final failure of a rejected list `invalid-reply: moves — <issue>`.
  let lastMovesIssue: string | undefined;
  // The most complete message list seen while polling this attempt; the
  // independence audit reads the tool calls and shells out of it.
  let lastSeen: SessionMessageInfo[] = [];
  // Per-attempt count of the `subagent` parts the child walk has already
  // covered (perf r1c0 #2): a later pass — a nudge, or a `wait` re-entry
  // with no new prompt — re-audits the parent list (pure, cheap) but re-walks
  // only if that count has grown, bounding the walk to at most one network
  // pass per new delegation instead of one per completed turn.
  let walkedDelegations = 0;

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
    // Cost rules (perf r1c0 #1, #2): only attempts that actually delegated
    // are worth a walk, and at most one walk per growth of the delegation
    // count — a nudge loop must not pay the walk nine times per attempt.
    const delegations = countDelegationParts(lastSeen);
    if (violations.length === 0 && delegations > walkedDelegations) {
      walkedDelegations = delegations;
      let descendants = 0;
      try {
        for await (const child of listChildSessionMessages(
          opts.client,
          sessionID,
          requestTimeoutMs,
          { connection: opts.server, deadline },
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
        // the partial audit is named in the run record. But a deed the walk
        // ALREADY saw is not thrown away with the rest of it (sec r1c0 #1):
        // the design's failure mode for a broken walk is a detection gap for
        // deeds UNSEEN, never erasure of evidence in hand, so the violations
        // collected before the failure survive into the check below — a
        // violation seen is a violation failed for.
        console.warn(
          `stage ${handle.stageKey}: independence audit could not complete the child-session ` +
            `walk for ${sessionID}: ` +
            `${brief(printable(String(error instanceof Error ? error.message : error)))} — ` +
            (violations.length === 0
              ? 'auditing this attempt on its own session alone'
              : `keeping the ${String(violations.length)} violation(s) the walk had already found`),
        );
        gateLog(state, 'independenceAudit', handle.round, {
          proceed: true,
          reason:
            `child-session walk incomplete for session ${sessionID}: ` +
            `${String(error instanceof Error ? error.message : error)}; ` +
            (violations.length === 0
              ? 'the attempt was audited on its own message list alone'
              : `${String(violations.length)} violation(s) already found by the walk were kept`),
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
      outcome = {
        ok: false,
        failure:
          lastMovesIssue === undefined
            ? 'invalid-reply: not JSON'
            : `invalid-reply: moves — ${lastMovesIssue}`,
      };
      break;
    }
    const checked = schema.safeParse(parsed);
    const movesIssue = checked.success
      ? movesValidationIssue(opts, def.name, headBefore, checked.data)
      : null;
    if (checked.success && movesIssue === null) {
      outcome = { ok: true, reply: checked.data };
      break;
    }
    // A rejected moves list rides the invalid-reply path exactly like a schema
    // mismatch — same pacing floor, same nudge with the issue, same same-session
    // attempt 2, same escalate on the second failure (proposal, Wiring into
    // retry: "behave exactly as for any invalid reply"). `moves — ` is what
    // makes it nameable in the gate log and the repair prompt.
    const problem = checked.success
      ? `moves — ${movesIssue ?? 'rejected'}`
      : (checked.error.issues[0]?.message ?? 'schema mismatch');
    if (checked.success) {
      lastMovesIssue = movesIssue ?? 'rejected';
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
            `Your reply did not validate: ${problem}.`,
            'Reply again with ONLY the JSON object. No other text.',
            '',
            replyShapeExample(schema),
          ].join(' '),
        },
        { signal: AbortSignal.timeout(requestTimeoutMs) },
      );
      lastPromptAt = Date.now();
      // Re-baseline, exactly as the narration nudges above do: without it the
      // idle-count check cannot tell "the session produced the repair" from
      // "idle with no new turn", and the poll re-reads the reply it already
      // rejected as if it were the answer to this nudge.
      assistantBaseline = (
        await listSessionMessages(opts.client, sessionID, requestTimeoutMs)
      ).filter((m) => m.type === 'assistant').length;
      continue;
    }
    outcome = { ok: false, failure: `invalid-reply: ${problem}` };
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

/**
 * One stage's window: the declared moves, then the sweep and commit. Skipped
 * when the stage failed.
 *
 * The moves land first, ahead of `enforceOwnership`: a validated pair's
 * endpoints already satisfy `stagePathAllowed` for this stage, so the sweep
 * leaves the delete and the add alone and `git add -A` in `commitStage` stages
 * them as one rename (rename detection is diff-time, A5). A failed apply is
 * environmental, not a reply defect: the list has already rolled back, so
 * finalize returns without sweeping, staging, or committing and leaves exactly
 * the preserved-worktree state a failed reply leaves today — attempt 2 starts a
 * fresh session because the failure carries no `invalid-reply` prefix, and its
 * sweep is the first thing that touches those paths (A4).
 */
function finalizeStage(
  repoDir: string,
  state: PipelineState,
  headBefore: string,
  stageName: string,
  stageKey: string,
  outcome: StageOutcome,
): StageOutcome {
  if (!outcome.ok) {
    return outcome;
  }
  const moves = (outcome.reply as { moves?: MovePair[] } | undefined)?.moves;
  if (Array.isArray(moves) && moves.length > 0) {
    const failure = applyMoves(repoDir, headBefore, moves);
    if (failure !== null) {
      const text =
        `${failure.rollbackFailed ? 'moves-rollback-failed' : 'moves-apply-failed'}: ` +
        failure.error;
      // The ledger entry for this attempt was pushed when its reply last
      // validated — before this window ran. A move failure lands after that
      // write, so amend the entry: the retry decision, the gate log, and the
      // operator-facing results all key on the failed attempt's `failure`, and
      // an `ok` ledger next to a known-damaged worktree is a lie.
      for (let i = state.results.length - 1; i >= 0; i -= 1) {
        const entry = state.results[i];
        if (entry !== undefined && entry.sessionID === outcome.sessionID) {
          entry.failure = text;
          break;
        }
      }
      // No sweep, no staging, no commit — the same preserved state a failed
      // reply leaves. `dirtyFiles` lists untracked paths file by file
      // (`--untracked-files=all`), so attempt 2's sweep sees every path
      // whether or not the index holds them.
      return { ...outcome, ok: false, failure: text };
    }
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
  return outcome;
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
  const outcome = await executeInference(opts, state, handle, cfg, undefined, headBefore);
  return finalizeStage(opts.repoDir, state, headBefore, stageName, handle.stageKey, outcome);
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
 * resume. ADR-005 keeps the two retryable moves outcomes on opposite sides of
 * this test: a validator rejection arrives as `invalid-reply: moves — <issue>`
 * and repairs in the same session, while `moves-apply-failed:` is environmental
 * and starts a fresh one. The third class never reaches this choice at all —
 * `escalatesWithoutRetry` stops it before attempt 2 is built.
 */
export function retryInSameSession(
  failure: string | undefined,
  sessionID: string | undefined,
): boolean {
  return sessionID !== undefined && (failure ?? '').startsWith('invalid-reply');
}

/**
 * The one failure class that must not start attempt 2: a rollback rename that
 * itself failed left the worktree half-moved, the one state nothing in-process
 * can undo (proposal, Apply; Risks). Retrying would point a new session at the
 * damaged tree and commit the half-applied list, so the driver escalates to the
 * operator instead.
 */
export function escalatesWithoutRetry(failure: string | undefined): boolean {
  return (failure ?? '').startsWith('moves-rollback-failed');
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
    try {
      await checkpoint(join(opts.repoDir, designDirOf(state)), state);
    } catch (error) {
      // ADR-006 §3: the entry save gets the same guard as the drive loop —
      // an unwritable checkpoint is an escalation with a record, not a crash.
      if (error instanceof CheckpointWriteFailure) {
        await finalizeEscalate(opts.repoDir, state, printable(error.message), cfg);
        return state.status;
      }
      throw error;
    }
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
  // The status as persisted, before this resume flips it to `running`: the
  // split-record test below compares the marker against what the checkpoint
  // itself claimed.
  const persistedStatus = state.status;
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
    // record lives on in the gate log and git history). ADR-006 §3: a marker
    // next to a checkpoint that still says `running` is a SPLIT record — the
    // terminal save failed mid-escalation. Gate-log what the marker recorded
    // (its failure line) before deleting it, so the new run's gate log
    // explains the gap it repaired; the checkpoint keeps its last good save.
    const markerAbs = join(opts.repoDir, designDirRel, ESCALATION_FILE);
    if (existsSync(markerAbs)) {
      if (persistedStatus === 'running') {
        let failureLine = '';
        try {
          failureLine =
            readFileSync(markerAbs, 'utf8')
              .split('\n')
              .find((line) => line.includes('checkpoint save failed')) ?? '';
        } catch {
          /* unreadable marker: name the split without quoting it */
        }
        gateLog(state, 'terminalSaveFailed', 0, {
          proceed: true,
          reason:
            'resumed a split record: ESCALATION.md was present while the checkpoint still said running; ' +
            (failureLine !== '' ? failureLine : 'the terminal checkpoint save failed'),
        });
      }
      try {
        unlinkSync(markerAbs);
      } catch {
        /* no marker */
      }
      // The sweep is author-agnostic; record the delete so the marker's
      // removal is not restored from `headBefore` at the next boundary.
      recordRunnerDelete(markerAbs);
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
      await finalizeDone(opts.repoDir, state, cfg);
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
        // A rollback rename that itself failed leaves a half-moved worktree,
        // the one state nothing in-process can undo. Attempt 2 would point a
        // fresh session at that tree, and its sweep cannot see the damage — the
        // half-applied pair's endpoints satisfy `stagePathAllowed` by
        // construction — so a valid attempt-2 reply would commit it. Stop at the
        // operator instead (proposal, Apply; Risks row 3).
        if (escalatesWithoutRetry(out.failure)) {
          await finalizeEscalate(
            opts.repoDir,
            state,
            `stage ${handle.stageKey} failed with a half-applied moves list ` +
              `(${out.failure}) — no retry: the worktree is known-damaged`,
            cfg,
          );
          throw new StopDriving();
        }
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
      // §4 (security r1, finding 4): the lock may only be gone because a
      // stage destroyed it — §1 fences the name, and the sweep can put it back
      // only from the bytes this process recorded. Re-assert it at this stage
      // boundary before the next window opens; a rival holding it stops the
      // run rather than admitting a second writer.
      ensureLockHeld(opts.repoDir, state.slug);
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
      if (error instanceof CheckpointWriteFailure) {
        // ADR-006 §3: the checkpoint is the run's spine — a structurally
        // unwritable checkpoint escalates with a record instead of crashing
        // without one (the run-2 blocker). This also converts the seat's
        // checkpoint rejection inside a pass (the `Promise.allSettled`
        // rethrow) into an escalation. Then stop driving.
        await finalizeEscalate(opts.repoDir, state, printable(error.message), cfg);
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

async function finalizeDone(
  repoDir: string,
  state: PipelineState,
  cfg: PipelineConfig,
): Promise<void> {
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
  // Security r1, finding 3: the `done` save is a terminal save like every
  // other §3 guards — the directory-at-the-checkpoint-path class converts to
  // an escalation with a record here too, instead of crashing the completion
  // path with no `done`/`escalated` record and no commit.
  try {
    await checkpoint(designDir, state);
  } catch (error) {
    if (!(error instanceof CheckpointWriteFailure)) {
      throw error;
    }
    restoreCommittedCheckpoint(repoDir, designDirOf(state));
    await finalizeEscalate(repoDir, state, printable(error.message), cfg);
    return;
  }
  recordStageCommit(
    state,
    commitStage(repoDir, `${state.nnn}-${state.slug}`, 'done', 'docs', {
      commitMessage: `docs: ${state.nnn}-${state.slug} pipeline complete`,
    }),
  );
}

/**
 * In-process equivalent of the printed repair, run after a failed terminal
 * save: when HEAD holds a file at the checkpoint path, clear the replacement
 * and land the committed copy back. Without it the escalation commit's
 * `git add -A` could stage planted bytes at a reserved path into the run's
 * terminal record. Silent best-effort: when nothing is committed at that
 * path (a run that never landed a checkpoint) the planted path stays exactly
 * where it is for the operator, and the printed repair stands.
 */
function restoreCommittedCheckpoint(repoDir: string, designDirRel: string): void {
  const cpRel = join(designDirRel, STATE_FILE);
  if (!gitOk(repoDir, ['cat-file', '-e', `HEAD:${cpRel}`])) {
    return;
  }
  try {
    const abs = resolve(repoDir, cpRel);
    if (existsSync(abs) && statSync(abs).isDirectory()) {
      rmSync(abs, { recursive: true, force: true });
    }
    git(repoDir, ['restore', '--source=HEAD', '--staged', '--worktree', '--', cpRel]);
  } catch (error) {
    console.warn(
      `could not restore the committed checkpoint: ${printable((error as Error).message)}`,
    );
  }
}

export async function finalizeEscalate(
  repoDir: string,
  state: PipelineState,
  reason: string,
  cfg: PipelineConfig,
): Promise<void> {
  // The reason usually quotes failure text the model or the server chose (a
  // seat failure, an independence violation naming a path). It lands in the
  // gate log, on the operator's terminal, and in ESCALATION.md: keep it on
  // one line and free of control/escape bytes.
  const safeReason = printable(reason);
  state.status = 'escalated';
  gateLog(state, 'escalate', 0, { proceed: false, reason: safeReason });
  const designDirRel = designDirOf(state);
  const designDir = join(repoDir, designDirRel);
  mkdirSync(designDir, { recursive: true });
  const phase = escalationPhase(state);
  // Atomic tmp+rename, like every other checkpoint write: this is the one
  // record the operator must be able to read afterwards, and interrupting a
  // run is most likely exactly when it is being written. ADR-006 §3: the
  // save is guarded, because an escalation that dies on its own failing
  // write leaves no terminal record at all (the run-2 blocker). The reason
  // survives in the in-memory gate log the report renders from (A4).
  let saveFailure: CheckpointWriteFailure | undefined;
  try {
    await checkpoint(designDir, state);
  } catch (error) {
    if (error instanceof CheckpointWriteFailure) {
      saveFailure = error;
    } else {
      throw error;
    }
  }
  const reportRel = join(designDirRel, ESCALATION_FILE);
  const markerAbs = join(repoDir, reportRel);
  const markdown =
    escalationMarkdown(state, phase, cfg.blockingSeverities) +
    // `saveFailure.message` already begins `checkpoint save failed:` (the
    // `saveState` stem); appending the prefix again made ESCALATION.md — and
    // the `terminalSaveFailed` reason that quotes this line — read the stem
    // twice (correctness r1, finding 5).
    (saveFailure === undefined ? '' : `\n${printable(saveFailure.message)}\n`);
  // Security r1, findings 1 and 2: a stage can plant a directory at the
  // marker path — the boundary sweep reverts the files under it, but an
  // empty untracked directory is invisible to `dirtyFiles` and survives
  // invisibly, exactly the residue class §3 closes for the checkpoint. Clear
  // it first, then land the marker through a tmp+rename pair like every
  // other terminal write, and if even that fails, print the whole report to
  // stderr: the run's one terminal record must survive either way.
  if (existsSync(markerAbs) && statSync(markerAbs).isDirectory()) {
    rmSync(markerAbs, { recursive: true, force: true });
  }
  const markerTmp = `${markerAbs}.${randomBytes(4).toString('hex')}.tmp`;
  try {
    writeFileSync(markerTmp, markdown, 'utf8');
    renameSync(markerTmp, markerAbs);
    recordRunnerWrite(markerAbs, markdown);
  } catch (error) {
    try {
      unlinkSync(markerTmp);
    } catch {
      /* no stray tmp */
    }
    console.error(
      `\npipeline ESCALATED — the terminal marker could not be written ` +
        `(${printable((error as Error).message)}). Full report:\n${markdown}`,
    );
  }
  // Nothing printed on escalation in run 1: a 47-minute resume that ended
  // escalated looked like a quiet success on the operator's terminal.
  console.error(
    `\npipeline ESCALATED (${phase} phase, after stage ${state.lastStage}): ${safeReason}\n` +
      `  report: ${reportRel}\n` +
      `  status: pipeline status ${state.nnn}-${state.slug}\n` +
      `  retry after a fix: pipeline resume ${state.nnn}-${state.slug}`,
  );
  if (saveFailure !== undefined) {
    // The split record is accepted, bounded, and visible: the checkpoint
    // keeps its last good save, ESCALATION.md is the terminal marker, and
    // `resume` repairs the gap (gate-logging it). Print the exact repair;
    // the `rm -r` variant covers a directory at the checkpoint path (A5).
    const cpRel = join(designDirRel, STATE_FILE);
    console.error(
      `  checkpoint save failed: the checkpoint keeps its last good save.\n` +
        `    repair a replaced path with: rm -r ${cpRel} && git checkout HEAD -- ${cpRel}, ` +
        `then pipeline resume ${state.nnn}-${state.slug}`,
    );
    restoreCommittedCheckpoint(repoDir, designDirRel);
  }
  recordStageCommit(
    state,
    commitStage(repoDir, `${state.nnn}-${state.slug}`, 'escalate', 'docs', {
      commitMessage: `docs: ${state.nnn}-${state.slug} pipeline escalated`,
    }),
  );
}

/**
 * Human-facing explanation for `pipeline status`, from the gate log only.
 * `escalationMarkerPresent` (ADR-006 §3) makes the SPLIT RECORD visible: an
 * `ESCALATION.md` marker next to a checkpoint that still says `running` means
 * the last terminal save failed — the SQLite journal-existence posture, the
 * marker is the terminal truth, the checkpoint keeps its last good save.
 */
export function statusLines(state: PipelineState, escalationMarkerPresent = false): string[] {
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
  if (escalationMarkerPresent && state.status === 'running') {
    lines.push(
      `note:    split record: ESCALATION.md marks this run escalated while the checkpoint still ` +
        `says running — the last terminal checkpoint save failed. The committed checkpoint is the ` +
        `recovery anchor: git checkout HEAD -- <design dir>/${STATE_FILE}, then resume (the resume ` +
        `gate-logs and clears the marker).`,
    );
  }
  return lines;
}

export function renderStatus(state: PipelineState, escalationMarkerPresent = false): string {
  return statusLines(state, escalationMarkerPresent).join('\n');
}
