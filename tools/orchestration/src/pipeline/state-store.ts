import { randomBytes } from 'node:crypto';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

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

/**
 * The persisted shape, as data. Run 1's own compatibility reviewer raised the
 * risk this guards against from the inside: a validator that rejects what the
 * generator produces makes a run unresumable by its own checkpoint. So the
 * grammar here is the *same* NNN_RE / SLUG_RE that deriveIdentity validates,
 * and every field added after run 1 is optional.
 */
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

const StateSchema = z.object({
  version: z.literal(1),
  nnn: z.string().regex(NNN_RE),
  slug: z.string().regex(SLUG_RE),
  branch: z.string().min(1),
  baseRef: z.string().optional(),
  source: z.object({ kind: z.enum(['adr', 'brief']), path: z.string().min(1) }),
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

/** Load a run's checkpoint, or null if none exists yet. */
export async function loadState(designDir: string): Promise<PipelineState | null> {
  const file = statePath(designDir);
  let raw: string;
  try {
    raw = await readFile(file, 'utf8');
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return null;
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
      )}). It was interrupted mid-write; recover it with: git checkout HEAD -- ${join(
        designDir,
        STATE_FILE,
      )}`,
      { cause: error },
    );
  }
  if (parsed === null || typeof parsed !== 'object') {
    throw new Error(`pipeline checkpoint is not an object: ${file}`);
  }
  const version = (parsed as { version?: unknown }).version;
  if (version !== 1) {
    throw new Error(`unsupported pipeline state version: ${String(version)}`);
  }
  const checked = StateSchema.safeParse(parsed);
  if (!checked.success) {
    const issues = checked.error.issues
      .map((issue) => `${issue.path.join('.') || '(root)'}: ${issue.message}`)
      .join('; ');
    throw new Error(`pipeline checkpoint is invalid: ${file} (${issues})`);
  }
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
  await mkdir(designDir, { recursive: true });
  await writeFile(tmp, `${JSON.stringify(state, null, 2)}\n`, 'utf8');
  await rename(tmp, target);
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
