import { randomBytes } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { mkdir, readFile, rename, stat, unlink, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';

import { z } from 'zod';

import type { PipelineState } from './types.js';
import { FindingSchema, NNN_RE, SLUG_RE, STATE_FILE } from './types.js';

export { STATE_FILE };

export function statePath(designDir: string): string {
  return join(designDir, STATE_FILE);
}

/** Repo-relative design directory for a run, e.g. docs/designs/002-x. */
export function designDirOf(state: PipelineState): string {
  return join('docs', 'designs', `${state.nnn}-${state.slug}`);
}

/** A seat's persisted verdict record; the StateSchema header below owns the
 * compatibility rules the whole shape follows. */
const SeatSchema = z.object({
  verdict: z.enum(['approve', 'request-changes']),
  stamp: z.string(),
  dissents: z.number().int().min(0),
  // Review independence (own-findings injection); added after run 1, so it
  // is optional and checkpoints written before it still parse. Every string
  // in it is model-authored, and the runner caps each recorded finding's
  // fields (`boundedFindings`) before storing them: the seat board is cloned
  // into every later checkpoint and re-injected into that seat's next prompt,
  // so one oversized reply would otherwise inflate the checkpoint forever.
  findings: z.array(FindingSchema).optional(),
});

/**
 * The cross-version resume contract (ADR-006 §2), stated as rule set:
 *
 * - GUARANTEED: any runner loads and resumes any `version: 1` checkpoint
 *   written by an earlier runner. Evolution rules: new fields are added
 *   optional with behavior-safe defaults; no field is ever removed, retyped,
 *   renamed, or re-interpreted — `updatedAt` keeps meaning "when this
 *   snapshot was taken" and `stageCommits` keeps its bounded-window meaning,
 *   as named contract, not comment. Backfill at resume happens only where
 *   absence changes behavior (the `baseRef` precedent in the runner).
 * - MAY REFUSE: an unknown `version` value, or a shape this schema rejects.
 *   Refusal never writes or deletes anything.
 * - A version bump is a separate, deliberate step: introducing `2` requires
 *   per-version readers at the version seam below plus a golden fixture per
 *   era (`tests/pipeline/fixtures/checkpoints/`) before the bump may ship.
 *
 * Run 1's own compatibility reviewer raised the risk this grammar guards
 * against from the inside: a validator that rejects what the generator
 * produces makes a run unresumable by its own checkpoint. So the grammar
 * here is the *same* NNN_RE / SLUG_RE that deriveIdentity validates, and
 * every field added after run 1 is optional.
 */
const StateSchema = z.object({
  version: z.literal(1),
  nnn: z.string().regex(NNN_RE),
  slug: z.string().regex(SLUG_RE),
  branch: z.string().min(1),
  baseRef: z.string().optional(),
  source: z.object({ kind: z.enum(['adr', 'brief']), path: z.string().min(1) }),
  // ADR-010: the flow the run follows, stamped once at entry. Additive-optional
  // — its absence is a pre-ADR-010 checkpoint and reads as `full` — and
  // validated, not coerced: a checkpoint is model-visible data at the resume
  // seam, so a flow the registry does not define refuses to load.
  flow: z.enum(['full', 'light']).optional(),
  // ADR-010 §4: the sticky plan stop, the record the stop escalates on
  // (`PipelineState.planStop`). Optional like every field added after run 1,
  // and typed, not coerced: the writer refuses a shape the loader would not
  // read back, so the stop can only ever carry the string it names.
  planStop: z.string().optional(),
  status: z.enum(['running', 'done', 'escalated']),
  lastStage: z.string(),
  designRound: z.number().int().min(0),
  codeRound: z.number().int().min(0),
  designBudgetUsed: z.number().int().min(0),
  codeBudgetUsed: z.number().int().min(0),
  editorialDone: z.boolean(),
  testsGreen: z.boolean(),
  testsNeedUpdate: z.boolean(),
  stageFailures: z.record(z.string(), z.number()),
  stageCommits: z.array(z.string()).optional(),
  forceNext: z.string().nullable(),
  extensionRounds: z.number().int().min(0).optional(),
  extensionSessions: z.number().int().min(0).optional(),
  seats: z
    .object({
      design: z.record(z.string(), SeatSchema).optional(),
      code: z.record(z.string(), SeatSchema).optional(),
    })
    .optional(),
  designCycle: z.number().int().min(0).optional(),
  codeCycle: z.number().int().min(0).optional(),
  designStamp: z.string().optional(),
  codeStamp: z.string().optional(),
  reviseNoChange: z.boolean().optional(),
  lastReviewPass: z
    .object({
      phase: z.enum(['design', 'editorial', 'code']),
      round: z.number(),
      cycle: z.number().optional(),
      seats: z.array(z.string()),
    })
    .optional(),
  results: z.array(
    z.object({
      stage: z.string(),
      round: z.number(),
      sessionID: z.string(),
      agent: z.string(),
      at: z.number(),
      // Optional: failed attempts persist without a reply (JSON.stringify
      // drops the undefined key), and a checkpoint must re-read what saveState
      // writes — required here made run 003 unresumable.
      reply: z.unknown().optional(),
      transcript: z.string(),
      durationMs: z.number().optional(),
      nudges: z.number().optional(),
      failure: z.string().optional(),
      docLines: z.number().optional(),
      docLinesAdded: z.number().optional(),
      docLinesRemoved: z.number().optional(),
    }),
  ),
  gateLog: z.array(
    z.object({
      gate: z.string(),
      round: z.number(),
      proceed: z.boolean(),
      reason: z.string(),
      at: z.number(),
    }),
  ),
  startedAt: z.number(),
  updatedAt: z.number(),
});

/**
 * Ceiling on the raw checkpoint read (security r1, finding 5). The file is
 * untrusted input — the committed copy can carry pre-fence model-authored
 * bytes (Risks row 2) — so the loader sizes it before parsing rather than
 * after: an over-cap file is refused with the same operator postures the
 * version refusal offers, spending bounded memory. Real checkpoints sit far
 * under this; the cap only ever fires on hostile or corrupt input.
 */
const MAX_CHECKPOINT_BYTES = 8 * 1024 * 1024;

/** Load a run's checkpoint, or null if none exists yet. */
export async function loadState(designDir: string): Promise<PipelineState | null> {
  const file = statePath(designDir);
  let raw: string;
  try {
    const info = await stat(file);
    if (info.size > MAX_CHECKPOINT_BYTES) {
      throw new Error(
        `pipeline checkpoint is too large: ${file} is ${String(info.size)} bytes, over the ` +
          `ceiling of ${String(MAX_CHECKPOINT_BYTES)}. Restore the committed copy ` +
          `(git checkout HEAD -- ${file}) or start a fresh run.`,
      );
    }
    raw = await readFile(file, 'utf8');
  } catch (error) {
    const code = (error as NodeJS.ErrnoException).code;
    if (code === 'ENOENT') {
      return null;
    }
    if (code === 'EISDIR' || code === 'ENOTDIR' || code === 'ENOTEMPTY') {
      // A directory (or a non-directory parent) occupies the checkpoint path.
      // A rename cannot repair that in place (A5); name the repair instead of
      // throwing a bare errno at `resume` and `status`.
      throw new Error(
        `pipeline checkpoint path is occupied: ${file} is not a readable checkpoint file. ` +
          `Repair it with: rm -r ${file} && git checkout HEAD -- ${file}`,
        { cause: error },
      );
    }
    throw error;
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw) as unknown;
  } catch (error) {
    // Run 1 proved the plausible cause: an interrupted write. Say what to do
    // instead of throwing a bare SyntaxError at `resume` and `status`.
    throw new Error(
      `pipeline checkpoint is not readable: ${file} (${String(
        (error as Error).message,
      )}). It was interrupted mid-write; recover it with: git checkout HEAD -- ${file}`,
      { cause: error },
    );
  }
  if (parsed === null || typeof parsed !== 'object') {
    throw new Error(`pipeline checkpoint is not an object: ${file}`);
  }
  const version = (parsed as { version?: unknown }).version;
  if (version !== 1) {
    // The refusal seam a future per-version reader ladder would mount at
    // (ADR-006 §2). The message keeps today's stem, names the supported
    // version, and states the ADR's own two interim postures. Refusal has
    // touched nothing on disk: `loadState` is read-only.
    throw new Error(
      `unsupported pipeline state version: ${String(version)}. This runner reads ` +
        `version 1 checkpoints. Until a compatible reader exists, restore the committed ` +
        `copy (git checkout HEAD -- ${file}) and resume, or start a fresh run.`,
    );
  }
  const checked = StateSchema.safeParse(parsed);
  if (!checked.success) {
    const issues = checked.error.issues
      .map((issue) => `${issue.path.join('.') || '(root)'}: ${issue.message}`)
      .join('; ');
    throw new Error(`pipeline checkpoint is invalid: ${file} (${issues})`);
  }
  // ADR-006 §1, correctness r1 finding 1: seed the in-process authorship
  // record with the bytes this loader just read. Without it, trust starts
  // empty in a fresh process and the resumed run's own newest checkpoint copy
  // — dirty at the sweep whenever an interruption landed between the last
  // loop-tail save and a single-stage resume — is reported as tampering and
  // reverted to `headBefore`, contradicting §1's "dirty ⇒ runner". Seeding
  // trusts exactly the bytes the run inherited from the last runner; a stage
  // write or delete afterwards still mismatches the record and is still
  // reverted, so the fence keeps its teeth.
  recordRunnerWrite(file, raw);
  return parsed as PipelineState;
}

/**
 * Atomically checkpoint the state: write to a unique temp file in the same
 * directory, then rename. A crash mid-write can never truncate the last
 * good checkpoint.
 *
 * The written snapshot is exactly the state that was handed in — `updatedAt`
 * is NOT re-stamped here. With ADR-003's fanned-out review passes the write
 * that lands for a snapshot is the one that was enqueued for it, possibly
 * later; re-stamping at rename time would make the checkpoint's `updatedAt`
 * mean "when the queue got around to it" instead of "when this snapshot was
 * taken", and would let a queued snapshot masquerade as a newer one.
 * The stamp is therefore the CALLER's job, applied where the snapshot is
 * taken: `checkpoint()` in the runner for the sequential path, and before
 * every `createStateWriter.enqueue` inside a pass. Both write paths keep the
 * persisted field meaning "when this snapshot was taken".
 */
/**
 * Every checkpoint write-path failure (`saveState`'s mkdir/writeFile/rename
 * trio) surfaces as this class (ADR-006 §3). A structurally unwritable
 * checkpoint is an escalation-worthy condition, not a crash: the drive loop
 * tells the two apart by this type. A refused invalid snapshot keeps plain
 * Error semantics — a refused write rejects its caller only.
 */
export class CheckpointWriteFailure extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = 'CheckpointWriteFailure';
  }
}

/**
 * ADR-006 §1 — what the runner itself wrote, kept in memory. The ownership
 * sweep is author-agnostic (it filters every touched path, runner saves
 * included), so it pairs the reserved-name rule with this authorship test:
 * a match is skipped while it still holds the bytes `saveState` last wrote
 * for it, or is absent because the runner deleted it, and reverted otherwise.
 * Per-process state; `loadState` seeds it from the checkpoint it just read,
 * so a new process starts trusting exactly the bytes the last runner left on
 * disk and "dirty ⇒ runner" survives the process boundary (correctness r1,
 * finding 1). Anything a stage then writes or deletes mismatches the record
 * and is still reverted.
 */
const runnerBytes = new Map<string, string>();
const runnerDeleted = new Set<string>();

/** Record the bytes the runner is about to have at `path`. */
export function recordRunnerWrite(path: string, bytes: string): void {
  runnerBytes.set(resolve(path), bytes);
}

/**
 * The bytes the runner last recorded for `path`, if any. The ownership sweep
 * uses these to restore a tampered untracked checkpoint (correctness r1,
 * finding 2): with nothing committed at the path yet, the recorded bytes are
 * the only good copy the run has.
 */
export function runnerBytesFor(path: string): string | undefined {
  return runnerBytes.get(resolve(path));
}

/** Record that the runner itself deleted `path`. */
export function recordRunnerDelete(path: string): void {
  runnerDeleted.add(resolve(path));
}

/**
 * True while `path` still holds the bytes the runner last wrote there, or is
 * absent because the runner deleted it — the one match the ownership sweep
 * skips (a stage write to a reserved path is a content mismatch against it).
 */
export function holdsRunnerBytes(path: string): boolean {
  const abs = resolve(path);
  // A recorded delete outranks a recorded write. One process often does both
  // to the same reserved path — `finalizeEscalate` writes `ESCALATION.md` and
  // records its bytes, the next `resumePipeline` deletes the marker (§3) — and
  // §1's rule is that a match is skipped while it holds the runner's bytes OR
  // "is absent because the runner deleted it". Testing the write record first
  // made the read fail on the runner's own delete and the sweep resurrected
  // the marker from `headBefore`, undoing the §3 repair. A stage that
  // re-creates the path is a content mismatch again and is still reverted.
  if (runnerDeleted.has(abs)) {
    return !existsSync(abs);
  }
  const bytes = runnerBytes.get(abs);
  if (bytes !== undefined) {
    try {
      return readFileSync(abs, 'utf8') === bytes;
    } catch {
      return false; // deleted or replaced: not what the runner left
    }
  }
  return false;
}

export async function saveState(designDir: string, state: PipelineState): Promise<void> {
  // Write nothing the loader would refuse to read back: a checkpoint the run
  // cannot resume is worse than a crash.
  const checked = StateSchema.safeParse(state);
  if (!checked.success) {
    const issues = checked.error.issues
      .map((issue) => `${issue.path.join('.') || '(root)'}: ${issue.message}`)
      .join('; ');
    throw new Error(`refusing to write an invalid pipeline checkpoint (${issues})`);
  }
  const target = statePath(designDir);
  const tmp = `${target}.${randomBytes(4).toString('hex')}.tmp`;
  const text = `${JSON.stringify(state, null, 2)}\n`;
  try {
    await mkdir(designDir, { recursive: true });
    await writeFile(tmp, text, 'utf8');
    await rename(tmp, target);
  } catch (error) {
    // A failed rename must not leave a stray `.pipeline.json.<hex>.tmp`
    // lying where the next boundary's `git add -A` would sweep it.
    await unlink(tmp).catch(() => undefined);
    const code = (error as NodeJS.ErrnoException).code;
    const occupied = code === 'EISDIR' || code === 'ENOTDIR' || code === 'ENOTEMPTY';
    throw new CheckpointWriteFailure(
      `checkpoint save failed: ${String((error as Error).message)}` +
        (occupied
          ? ` — the checkpoint path is occupied; repair it with: ` +
            `rm -r ${target} && git checkout HEAD -- ${target}`
          : ''),
      { cause: error },
    );
  }
  recordRunnerWrite(target, text);
}

/**
 * ADR-003 §Self-update: the one in-process checkpoint write path used from
 * inside a concurrent review pass. Concurrent `saveState` calls race their
 * tmp+rename pairs: an older snapshot can land last (lost reviewer record),
 * and a stray `.pipeline.json.*.tmp` can sit where the pass's `git add -A`
 * sweeps it. This writer snapshots at enqueue time and drains through
 * `saveState` one write at a time in enqueue order, so the last landed
 * rename is the newest snapshot; `drain()` is the barrier's guarantee that
 * no checkpoint write is in flight when pass finalize mutates git.
 */
export interface CheckpointWriter {
  /** Snapshot `state` now; the write runs when all earlier enqueues landed. */
  enqueue(state: PipelineState): Promise<void>;
  /** Resolve once every enqueued write has landed (or failed). */
  drain(): Promise<void>;
}

export function createStateWriter(designDir: string): CheckpointWriter {
  let queue: Promise<void> = Promise.resolve();
  return {
    enqueue(state: PipelineState): Promise<void> {
      // Snapshot synchronously: the caller's state keeps mutating while
      // this write waits its turn; the queued bytes are the state as enqueued.
      const snapshot = structuredClone(state);
      const write = queue.then(() => saveState(designDir, snapshot));
      // A refused snapshot (invalid state) rejects the caller's promise but
      // never stalls the chain: later writes still land.
      queue = write.then(
        () => undefined,
        () => undefined,
      );
      return write;
    },
    drain(): Promise<void> {
      return queue;
    },
  };
}
