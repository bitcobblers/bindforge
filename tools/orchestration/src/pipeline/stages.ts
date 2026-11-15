import { StageReplySchemas } from './types.js';
import type { StageKind } from './types.js';

// ---------------------------------------------------------------------------
// Stage registry: stage -> agent id -> prompt template -> reply schema, and
// the pipeline config defaults. Turn budgets are bounded ONLY by the per-
// stage timeout; there is no step cap.
// ---------------------------------------------------------------------------

export const DESIGN_SPECIALTIES = [
  'correctness',
  'compatibility',
  'security',
  'performance',
  'complexity',
] as const;

export const CODE_SPECIALTIES = [
  'correctness',
  'compatibility',
  'security',
  'performance',
  'testing',
] as const;

export interface PipelineConfig {
  /**
   * Backstop on the mutating revise/fix loop, NOT the review cost cap: one
   * entry per design round or fix stage, the termination guarantee for red
   * tests and unmovable objections. Review cost is bounded by `maxReviews`.
   */
  maxRounds: number;
  /**
   * Review sessions per phase (ADR-002). A round costs 1-5 sessions per pass
   * and an unbounded number of passes, so sessions are the unit an operator
   * actually buys with `--extend-budget`.
   */
  maxReviews: { design: number; code: number };
  /** Dissent/ratify passes allowed inside one round before the run escalates. */
  maxPassesPerRound: number;
  blockingSeverities: string[];
  pollIntervalMs: number;
  stageTimeoutMs: number;
  /**
   * Ceiling for a single HTTP round-trip to the server. Optional so existing
   * configs keep compiling; see DEFAULT_REQUEST_TIMEOUT_MS.
   */
  requestTimeoutMs?: number;
  /**
   * Floor between prompts to one session (stage prompt, nudge, retry). Run 1
   * fired eight nudges in 69 seconds at sessions that were still working.
   */
  minNudgeIntervalMs?: number;
  /**
   * Maximum review-seat sessions in flight at once inside one pass
   * (ADR-003). Optional so existing configs keep compiling; the sole default
   * authority is `DEFAULT_CONFIG` (1 — today's serial lane, the degenerate
   * fan-out). Seats launch in the fixed handed order through a semaphore of
   * this size; retries hold their seat's permit. Operator guidance: N must
   * not exceed the inference server's parallel slot count, which the runner
   * cannot verify.
   */
  maxParallelReviews?: number;
  /**
   * Per-phase review-seat panel (ADR-010 §3). Optional, and `DEFAULT_CONFIG`
   * deliberately leaves it absent: absent means the standing five-seat panels,
   * so every pre-ADR-010 config keeps its exact behaviour. A flow's own partial
   * is the only thing that narrows a panel — `configFor` applies it last, so a
   * caller-supplied panel cannot widen a light run.
   */
  reviewSeats?: { design: string[]; code: string[] };
  /**
   * Line ceiling for the light flow's plan document (ADR-010 §4). Absent means
   * no plan-size gate at all, which is the full flow's behaviour: the cap is
   * flow config, not a universal document budget.
   */
  planMaxLines?: number;
  /**
   * ADR-016 Decision 4: line ceiling for the full flow's design documents
   * (`01-research.md` and `02-proposal.md`). Registry config, following
   * `planMaxLines`: absent means the unchanged measure-and-warn posture, and
   * over length is a normal stage failure the retry budget repairs. Set in
   * `FLOWS.full.config`, and deliberately absent for `light`, whose plan stays
   * under `planMaxLines`.
   */
  docMaxLines?: number;
}

/**
 * No pipeline call had a request timeout, so one hung or half-open request
 * stalled the poll loop forever and an unhandled rejection ended the run.
 */
export const DEFAULT_REQUEST_TIMEOUT_MS = 30_000;

/** See PipelineConfig.minNudgeIntervalMs. */
export const DEFAULT_MIN_NUDGE_INTERVAL_MS = 20_000;

export const DEFAULT_CONFIG: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  /**
   * Dissent/ratify passes inside one round — unchanged by ADR-010, whose
   * Non-goals rule out any change to the full flow's budgets. Run 010 hit 6 and
   * stopped with every seat reading `approve`, four of them stale: the funnel
   * was one ratify pass from converging. That is a stamp rule, not a price, and
   * this change fixes it where the cause is — a ratify pass no longer re-stamps
   * to `HEAD` and ages the approvals it is ratifying (`runner.ts`, drive loop).
   *
   * Note the gap the run exposed: `--extend-budget` buys sessions and rounds,
   * and `planPass` blocks at `cycle >= maxPassesPerRound` with no extension
   * channel, so this number is the one budget an operator cannot buy at the
   * stop it causes — the only remedy is editing this default. Extending the
   * budget grammar to passes is a funnel change (ADR-002's accounting), not a
   * constant, and deliberately not taken here.
   */
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 2_000,
  /**
   * Wall-clock cap per stage attempt. The sole bound on a stage's turns (no
   * step cap), so this is the cost ceiling of a wedged session: it is the
   * full flow's 30 minutes, and the light flow's 12 (ADR-010's budget table),
   * both as registry data — the light flow buys its shorter stage through
   * `FLOWS.light.config`, never by moving this default.
   */
  stageTimeoutMs: 30 * 60_000,
  // ADR-003: opt-in fan-out; 1 keeps the serial lane on the same code path.
  // Operator decision 2026-11-15 raises the shipped default to 2; ADR-003's
  // "default N = 1" clause is superseded, the flag/env stay the knob.
  maxParallelReviews: 2,
};

export interface StageDef {
  name: string;
  kind: StageKind;
  agent: string;
  /** Template file under prompts/, or null for a fixed built-in instruction. */
  template: string | null;
}

interface StageBase {
  kind: StageKind;
  /** Agent id, or a function of the review specialty. */
  agent: string | ((specialty: string) => string);
  template: string | null;
}

const STAGE_BASES: Record<string, StageBase> = {
  research: { kind: 'research', agent: 'research/researcher', template: 'research.md' },
  // ADR-010 §2: the light flow's single design stage. Same reply contract as
  // `proposal` (kind `proposal`), a different agent and template, and a plan
  // that lands at the proposal path so every downstream consumer that reads
  // `02-proposal.md` — implement, code review, the size meter — needs no fork.
  plan: { kind: 'proposal', agent: 'design/planner', template: 'plan.md' },
  proposal: { kind: 'proposal', agent: 'design/designer', template: 'proposal.md' },
  'proposal-revise': {
    kind: 'revise',
    agent: 'design/designer',
    template: 'proposal-revise.md',
  },
  'proposal-final': {
    kind: 'proposal-final',
    agent: 'design/editor',
    template: 'proposal-final.md',
  },
  'design-review': {
    kind: 'review',
    agent: (s) => `review/design-${s}`,
    template: 'design-review.md',
  },
  'editorial-review': {
    kind: 'review',
    agent: (s) => `review/design-${s}`,
    template: 'design-review.md',
  },
  'test-authoring': {
    kind: 'test-authoring',
    agent: 'code/test-engineer',
    template: 'test-authoring.md',
  },
  implement: { kind: 'implement', agent: 'code/coder', template: 'implement.md' },
  fix: { kind: 'fix', agent: 'code/coder', template: 'fix.md' },
  'test-update': { kind: 'test-update', agent: 'code/test-engineer', template: 'test-update.md' },
  'code-review': {
    kind: 'review',
    agent: (s) => `codereview/code-${s}`,
    template: 'code-review.md',
  },
  'doc-sync': { kind: 'doc-sync', agent: 'ops/doc-sync', template: 'doc-sync.md' },
};

/**
 * Registry entries that exist only in the light flow (ADR-010 §2). Excluded
 * from `STAGE_NAMES` because that export is the full flow's stage list, which
 * the flow registry asserts equals `FLOWS.full.stages` — the regression guard
 * that `full` is today's twelve stages.
 */
const LIGHT_ONLY_STAGES = ['plan'];

/** The full flow's stages, in registry order: the twelve stages ADR-010 found. */
export const STAGE_NAMES = Object.keys(STAGE_BASES).filter(
  (name) => !LIGHT_ONLY_STAGES.includes(name),
);

/**
 * Everything `--stage` accepts: the single stages, and a whole review round
 * by phase. Forcing `design-review` used to fall into stageDef, which threw
 * "stage design-review requires a review specialty" after the lock was taken
 * and ESCALATION.md deleted, exiting 1 with no gate entry.
 */
export const REVIEW_STAGE_NAMES = ['design-review', 'editorial-review', 'code-review'];

/**
 * Validates a `--stage` value against the full flow's stage list; throws a
 * usage error listing the valid ones. The production gate is
 * `assertStageInFlow` (flows.ts), which reads the flow's own registry and so
 * also admits the light flow's `plan`; every name accepted here is in
 * `FLOWS.full.stages`, so the registry check subsumes this one. It stays as
 * the `STAGE_NAMES` regression guard — a stage that leaves the registry with
 * no flow declaring it fails here, which the per-flow check cannot see.
 */
export function assertForceableStage(name: string): void {
  if (STAGE_NAMES.includes(name) || REVIEW_STAGE_NAMES.includes(name)) {
    return;
  }
  throw new Error(
    `unknown --stage "${name}". Valid: ${[...STAGE_NAMES, ...REVIEW_STAGE_NAMES].join(', ')}`,
  );
}

/**
 * Resolve a stage definition. Review stages take their specialty (which also
 * selects the agent); non-review stages ignore it.
 */
export function stageDef(name: string, specialty: string | undefined): StageDef {
  const base = STAGE_BASES[name];
  if (!base) {
    throw new Error(`unknown pipeline stage: ${name}`);
  }
  const agent = typeof base.agent === 'function' ? base.agent(specialty ?? '') : base.agent;
  if (typeof base.agent === 'function' && !specialty) {
    throw new Error(`stage ${name} requires a review specialty`);
  }
  return {
    name,
    kind: base.kind,
    agent,
    template: base.template,
  };
}

/** Reply schema a stage's JSON reply is validated against. */
export function stageReplySchema(def: StageDef) {
  return StageReplySchemas[def.kind];
}
