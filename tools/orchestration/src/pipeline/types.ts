import { z } from 'zod';

// ---------------------------------------------------------------------------
// Verdicts: the control signal of the pipeline. Markdown review files are
// the human record; these JSON replies are what the state machine reads.
// ---------------------------------------------------------------------------

export const Severity = z.enum(['blocker', 'major', 'minor', 'nit']);
export type Severity = z.infer<typeof Severity>;

export const FindingSchema = z.object({
  severity: Severity,
  location: z.string().min(1),
  problem: z.string().min(1),
  fix: z.string().optional(),
});
export type Finding = z.infer<typeof FindingSchema>;

export const VerdictSchema = z.object({
  verdict: z.enum(['approve', 'request-changes']),
  findings: z.array(FindingSchema),
});
export type VerdictReply = z.infer<typeof VerdictSchema>;

/** Optional model-authored commit message every mutating stage may include. */
const commitMessage = z.string().min(3).optional();

/**
 * ADR-005: one file move, declared as data. The stage never touches the paths;
 * the runner validates the list and performs the renames itself.
 */
export const MovePairSchema = z.object({
  from: z.string().min(1).max(200),
  to: z.string().min(1).max(200),
});
export type MovePair = z.infer<typeof MovePairSchema>;

/**
 * The move list a reply may carry. The bounds cap the validation and apply work
 * one reply can make the runner spend; an endpoint over the length cap or a
 * list over the pair cap fails `safeParse` like any invalid reply. Optional
 * with default `[]` keeps the checkpoint additive (A13): `version` stays 1.
 */
const moves = z.array(MovePairSchema).max(64).default([]);

export const StageReplySchemas = {
  research: z.object({ summary: z.string().min(1), commitMessage }),
  proposal: z.object({ commitMessage }),
  revise: z.object({
    changed: z.boolean().default(true),
    summary: z.string().min(1).optional(),
    commitMessage: z.string().min(1).optional(),
  }),
  'proposal-final': z.object({
    ambiguities: z.array(z.string()).default([]),
    commitMessage,
  }),
  'test-authoring': z.object({
    testsAdded: z.number().int().min(0).default(0),
    commitMessage,
  }),
  'test-update': z.object({ commitMessage, moves }),
  implement: z.object({ testsNeedUpdate: z.boolean(), commitMessage, moves }),
  fix: z.object({ testsNeedUpdate: z.boolean(), commitMessage, moves }),
  'doc-sync': z.object({ drifted: z.boolean(), commitMessage }),
  review: VerdictSchema,
} as const;
export type StageKind = keyof typeof StageReplySchemas;

// ---------------------------------------------------------------------------
// Persistent pipeline state, stored in docs/designs/NNN-slug/.pipeline.json
// ---------------------------------------------------------------------------

export interface SourceRef {
  kind: 'adr' | 'brief';
  path: string;
}

export interface StageResult {
  /** Stage name; review stages are `<phase>-review:<specialty>`. */
  stage: string;
  round: number;
  sessionID: string;
  agent: string;
  at: number;
  /**
   * The stage's validated reply. Absent on failed attempts: `outcome.reply`
   * is undefined there and `JSON.stringify` drops the key, so a checkpoint
   * that had ever stored a failed result could not re-parse while the schema
   * required the key — run 003 became unresumable exactly that way.
   */
  reply?: unknown;
  transcript: string;
  /** Wall-clock duration of the attempt (operator observability). */
  durationMs?: number;
  /** Nudges the runner sent to this session before the reply closed. */
  nudges?: number;
  /** Failure text when the attempt failed; absent when it succeeded. */
  failure?: string;
  /**
   * Design-scope size meter, recorded for the stages that write
   * `02-proposal.md`: lines in the document afterwards, and the worktree delta
   * that produced them. Run 2 grew 401 -> 2,696 lines over 8 revise rounds and
   * nothing in the checkpoint or `pipeline status` showed it, so the design
   * grew past its ADR while the operator watched findings instead. The runner
   * measures this itself; a stage cannot report its own growth.
   */
  docLines?: number;
  docLinesAdded?: number;
  docLinesRemoved?: number;
}

/**
 * One seat's latest verdict for a funnel phase (ADR-002). `stamp` is the
 * revision the verdict describes: the blob id of `02-proposal.md` for design,
 * HEAD for code. A seat is stale when its stamp no longer matches the current
 * one — panel selection is a function of these records and the stamp alone.
 */
export interface SeatRecord {
  verdict: 'approve' | 'request-changes';
  /** Revision the verdict was taken on (git blob id or commit sha). */
  stamp: string;
  /** Passes in which this seat dissented (operator-facing, not gating). */
  dissents: number;
  /**
   * The findings this seat held on the revision its verdict describes. The
   * runner re-injects these into that seat's next-pass prompt so a reviewer
   * never reads a report file - not even its own, and never another seat's.
   * Cross-seat report reading anchors verdicts and poisons the panel's
   * independence; transcripts are out of band entirely.
   */
  findings?: Finding[];
}

/** Per-seat ledger for the funnel phases. Editorial has no seats. */
export interface SeatBoard {
  design?: Record<string, SeatRecord>;
  code?: Record<string, SeatRecord>;
}

/** The review pass the next revise/fix stage must consume (ADR-002 scope). */
export interface LastReviewPass {
  phase: 'design' | 'editorial' | 'code';
  round: number;
  /** Pass index within the round; undefined for the single-pass editorial. */
  cycle?: number;
  seats: string[];
}

export interface GateLogEntry {
  gate: string;
  round: number;
  proceed: boolean;
  reason: string;
  at: number;
}

export type RunStatus = 'running' | 'done' | 'escalated';

/** Checkpoint file name inside the design directory. */
export const STATE_FILE = '.pipeline.json';

/**
 * Run lock file at the repository root. Named here rather than in runner.ts so
 * the moves validator can reject it as a path segment without importing the
 * runner (which imports the validator).
 */
export const LOCK_FILE = '.pipeline.lock';

/**
 * Run-identity grammar. `nnn-slug` is used as a single path segment (the
 * design directory) and as a git branch name, so both parts are constrained;
 * the generator in identity.ts must obey this grammar too, or a validator that
 * rejects it makes runs unresumable by their own checkpoint.
 */
export const NNN_RE = /^\d{3}$/;
export const SLUG_RE = /^[a-z0-9][a-z0-9-]*$/;

export interface PipelineState {
  version: 1;
  slug: string;
  nnn: string;
  branch: string;
  /**
   * Commit the pipeline branch forked from: the diff base for code review
   * and doc sync (never the repo default branch, which may be far behind).
   */
  baseRef?: string;
  source: SourceRef;
  status: RunStatus;
  /** Last completed stage ('' before the first stage). */
  lastStage: string;
  /** Number of the last executed design/editorial and code review rounds. */
  designRound: number;
  codeRound: number;
  /** Round-budget consumption; incremented on entry to any design/code
   * review round AND on entry to any fix stage (termination guarantee). */
  designBudgetUsed: number;
  codeBudgetUsed: number;
  editorialDone: boolean;
  /** Outcome of the most recent `npm run check` pre-check. */
  testsGreen: boolean;
  /** Set by an implement/fix reply or by a test-path ownership revert. */
  testsNeedUpdate: boolean;
  /** Stage failures consumed so far (step exhaustion, invalid reply). */
  stageFailures: Record<string, number>;
  /**
   * SHAs of commits the runner authored (oldest first, bounded window).
   * Ownership enforcement reverts paths from these commits only: models never
   * commit, so any other commit inside a stage window is operator work and is
   * reported instead of reverted. Optional so checkpoints written before it
   * existed still load (they own no commits).
   */
  stageCommits?: string[];
  /** One-shot forced stage for `pipeline run --stage <name>`. */
  forceNext: string | null;
  /**
   * Review rounds granted beyond `maxRounds` by an explicit `--extend-budget`.
   * Consumed one per round entry and logged, so an over-cap round always has
   * an operator consent line in the gate log. Run 1 reached 9/5 because a
   * forced `proposal-revise` re-entered review with no cap check at entry.
   */
  extensionRounds?: number;
  /**
   * Review sessions granted beyond `maxReviews` by `--extend-budget` (ADR-002
   * counts cost in sessions, not rounds). Consumed one per over-cap pass entry,
   * each consumption logged, so an over-cap pass always has an operator
   * consent line in the gate log.
   */
  extensionSessions?: number;
  /** Per-seat verdict ledger for the funnel phases; absent pre-ADR-002. */
  seats?: SeatBoard;
  /** Review passes already run inside the current design/code round. */
  designCycle?: number;
  codeCycle?: number;
  /**
   * Revision the current round's verdicts must describe: blob id of
   * `02-proposal.md` (design) and HEAD (code), refreshed at every pass entry.
   */
  designStamp?: string;
  codeStamp?: string;
  /** The last proposal-revise replied `changed: false` (deadlock stop input). */
  reviseNoChange?: boolean;
  /** Which reports the next revise/fix stage was handed. */
  lastReviewPass?: LastReviewPass;
  results: StageResult[];
  gateLog: GateLogEntry[];
  startedAt: number;
  updatedAt: number;
}

export interface LockInfo {
  slug: string;
  pid: number;
  stage: string;
  at: number;
}
