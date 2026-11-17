import { z } from 'zod';

import type { Flow } from './flows.js';

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
  kind: 'adr' | 'brief' | 'task';
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
  /**
   * Which document that meter measured: the design document for the proposal
   * stages and the light plan, `01-research.md` for `research` (ADR-016
   * Decision 4 widens the metered set to the research document). Additive-
   * optional like every field added after run 1 (A6), and its absence reads as
   * the design document — the only document the meter measured before the
   * widening — so the growth aggregate never folds a research document into
   * the line it labels `proposal:`.
   */
  docName?: string;
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

/**
 * ADR-016 Decision 2: one leg of the check gate, as the runner ran it. The
 * harness used to spawn every leg with `stdio: 'ignore'` and keep one boolean,
 * so a seat grading a behavior claim and a coder facing a red gate were both
 * told the outcome without the evidence. `command` is the leg's command line
 * as spawned, `cwd` the package dir it ran in, `exit` the child's exit code —
 * null when the spawn failed or a signal killed it, with the failure spelled
 * out in `tail` — and `tail` the merged stdout+stderr decoded as UTF-8 with
 * replacement characters, bounded to its last `MAX_GATE_TAIL_CHARS` characters
 * at capture. It is output of the code under review, never an instruction to
 * whoever reads the record.
 */
export interface GateLeg {
  command: string;
  cwd: string;
  exit: number | null;
  tail: string;
}

/**
 * One check-gate run, persisted beside `testsGreen` so the boolean and the
 * evidence behind it land in the same checkpoint (a crash after a red gate
 * loses both or neither). Additive-optional like every field added after run 1
 * (A6): a pre-016 checkpoint carries none, and its absence renders the
 * stand-in line rather than an empty section.
 */
export interface GateEvidence {
  at: number;
  green: boolean;
  legs: GateLeg[];
}

/**
 * ADR-020 (design 020 §1/§2) — the restart-boundary record: which stage's
 * commit window net-changed the harness's executable surface, the surface paths
 * it changed (writer-side posture: `printable` copies, bounded like
 * `boundedGateEvidence`), and when the runner classified it. The pause is this
 * record plus its gate-log entry, boundary commit and printed summary — never
 * a status value and never a marker file beside the checkpoint.
 */
export interface RestartBoundary {
  /** Registry name of the stage whose window opened the boundary. */
  stage: string;
  /** The surface paths the window's net diff changed, bounded and printable. */
  paths: string[];
  /** Wall clock of the classification. */
  at: number;
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
 * Escalation marker at the design directory root — the run's terminal record
 * when a gate stops it. Like the checkpoint, only the runner writes it
 * (ADR-006 §1).
 */
export const ESCALATION_FILE = 'ESCALATION.md';

/**
 * The run's two metered documents (ADR-016 Decision 4 widens the size meter to
 * the research document). The meter names which one a count came from, so the
 * operator-facing growth line stays the design document's instead of summing
 * two different documents under one `proposal:` label.
 */
export const PROPOSAL_DOC = '02-proposal.md';
export const RESEARCH_DOC = '01-research.md';

/**
 * The `saveState` tmp-file shape (`state-store.ts`): the checkpoint name plus
 * a random 8-hex suffix and `.tmp`. Part of the runner-reserved name set.
 */
export const STATE_TMP_RE = /^\.pipeline\.json\.[0-9a-f]{8}\.tmp$/i;

/**
 * True when any `/`-separated segment of `path` names a runner-reserved file:
 * the checkpoint (`STATE_FILE`), a `saveState` tmp, the escalation marker, or
 * the run lock (`LOCK_FILE`). Compared segment-by-segment and case-
 * insensitively — the way the moves validator already compares these names —
 * so the ADR-006 §1 fence cannot be routed around by case or by burying the
 * name in a deeper directory. The lock is included because §1's trust posture
 * leans on single-flight (§4): the moves validator already names it reserved
 * (moves.ts rule 2), and leaving the *write* side open let a coder-class
 * stage delete or rewrite the lock and break that premise (security r1,
 * finding 4); the boundary re-assert in the runner closes the loop.
 */
export function namesReservedSegment(path: string): boolean {
  for (const segment of path.split('/')) {
    const lower = segment.toLowerCase();
    if (
      lower === STATE_FILE.toLowerCase() ||
      lower === ESCALATION_FILE.toLowerCase() ||
      lower === LOCK_FILE.toLowerCase() ||
      STATE_TMP_RE.test(segment)
    ) {
      return true;
    }
  }
  return false;
}

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
  /**
   * ADR-010: which flow this run follows — recorded once, at run entry, from
   * the source document's `flow:` key, and read from there by every consumer.
   * Absent is a pre-ADR-010 checkpoint and means `full`, so nothing infers the
   * mode downstream.
   */
  flow?: Flow;
  /**
   * ADR-010 §4: the light flow's plan stop, sticky. Recorded when the plan
   * gate refuses, so the stop outlives the escalation the refusal minted:
   * `decideNext`'s `case 'plan'` escalates on this record while it stands,
   * which is what stops a plain `pipeline resume` from spending the flow's
   * coder session on the plan the flow refused. Cleared by the next `plan`
   * pass whose gate proceeds — the operator's in-flow repair is
   * `pipeline resume <slug> --stage plan`, the promotion repair is a new
   * full-flow entry. Additive-optional (ADR-006 §2): no pre-ADR-010
   * checkpoint carries it, and its absence is "no stop standing".
   */
  planStop?: string;
  /**
   * ADR-020 (design 020 §2): the restart boundary, sticky. A runner commit
   * window whose NET diff changed the executable surface — a module the process
   * imported at start, or a template it renders at stage time — is judged by
   * code that is already stale for it, so the run stops as a clean pause and
   * the *next process* carries it on. Recorded beside `planStop` (the same
   * sticky companion-field shape, A10) rather than as a `RunStatus` member:
   * widening the `status` enum is the retype ADR-006 §2 forbids, and a pre-020
   * reader loads this checkpoint with the unknown key inert. `resume` is the
   * restart: it gate-logs the boundary it crosses and clears this field, so
   * every resume starts from "no boundary standing". Additive-optional: no
   * earlier checkpoint carries it, and its absence is the same absence rule as
   * `flow`, `planStop` and `gateEvidence`.
   */
  restartRequired?: RestartBoundary;
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
  /**
   * ADR-016 Decision 2: the one gate record — per-leg command, exit and
   * bounded tail — captured at the same point `testsGreen` is set. Field
   * kept, never retyped (A6); this is the record the `code-review`, `fix`,
   * `fix-tests` and `implement` prompts render.
   */
  gateEvidence?: GateEvidence;
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
