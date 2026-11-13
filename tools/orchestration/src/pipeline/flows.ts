import { CODE_SPECIALTIES, DEFAULT_CONFIG, DESIGN_SPECIALTIES } from './stages.js';
import type { PipelineConfig } from './stages.js';
import type { ReviewPhase } from './states.js';
import type { PipelineState } from './types.js';

// ---------------------------------------------------------------------------
// Flow registry (ADR-010 §3): a flow is a stage sequence, the document stages
// the size meter measures, and a config partial — the budgets, the review-seat
// panel and the plan line ceiling — resolved once through `configFor`.
// `state.flow` is recorded once, at run entry, and every consumer reads the
// flow from there: nothing infers the mode downstream, and the full flow's
// partial is empty, so `full` is the status quo by construction.
// ---------------------------------------------------------------------------

/** Which flow a run follows: the registry's two keys, and nothing else. */
export type Flow = 'full' | 'light';

/**
 * A flow value read from outside this process — a source document's `flow:`
 * frontmatter, a checkpoint's `flow` field. The open `string` arm carries
 * untrusted text to `flowDef`, the registry lookup, which is the only gate: an
 * unreadable value reaches a usage error there instead of a silent `full` run.
 * Every value that leaves the registry is a `Flow`, so the compiler keeps
 * guarding the flow at the seams that consume it.
 */
export type FlowId = Flow | (string & {});

/**
 * The two review panels a flow can run. The editorial round is a single
 * language pass over the design panel (`states.ts` maps it to `design`), not a
 * seat list of its own, so a panel has no `editorial` key: every panel reader —
 * `seatsFor`, `PANELS`, the validators — is keyed on these two.
 */
export type SeatPhase = Exclude<ReviewPhase, 'editorial'>;

export interface FlowDef {
  /** Registry key, carried for operator-facing text and registry self-reference. */
  name: Flow;
  /** Every stage in the flow, in run order. */
  stages: readonly string[];
  /** Stages whose product is the design document, i.e. the size meter's list. */
  docStages: readonly string[];
  /** Operator-facing stage path, as `pipeline status` and ESCALATION.md print it. */
  path: string;
  /**
   * Budget, seat-panel and plan-cap overrides, applied last in `configFor`.
   * `full`'s carried nothing until ADR-016 Decision 4 added the single
   * `docMaxLines` key: `DEFAULT_CONFIG` stays the default authority for every
   * other knob there.
   */
  config: Partial<PipelineConfig>;
}

export const FLOWS: Record<Flow, FlowDef> = {
  full: {
    name: 'full',
    stages: [
      'research',
      'proposal',
      'proposal-revise',
      'proposal-final',
      'design-review',
      'editorial-review',
      'test-authoring',
      'implement',
      'fix',
      'test-update',
      'code-review',
      'doc-sync',
    ],
    docStages: ['research', 'proposal', 'proposal-revise', 'proposal-final'],
    path: 'research → proposal → design review → proposal-final → editorial review → test authoring → implement → code review → doc-sync',
    // ADR-016 Decision 4: the one key the full flow's registry writes — the
    // document line ceiling, with headroom above the largest full-flow
    // document this repo has shipped (733 lines) and far below the 2,696-line
    // runaway the size meter exists for. Everything else stays DEFAULT_CONFIG's.
    config: { docMaxLines: 900 },
  },
  light: {
    name: 'light',
    // ADR-010's four stages, in run order: one plan document, the coder, the
    // fix loop, and a two-seat code panel. No research, design panel,
    // editorial round, test engineer or doc-sync commit.
    stages: ['plan', 'implement', 'fix', 'code-review'],
    // The plan is the flow's only document, written at the proposal path.
    docStages: ['plan'],
    path: 'plan → implement → code review',
    config: {
      maxRounds: 2,
      maxReviews: { design: 0, code: 6 },
      maxPassesPerRound: 3,
      stageTimeoutMs: 12 * 60_000,
      planMaxLines: 80,
      reviewSeats: { design: [], code: ['correctness', 'testing'] },
    },
  },
};

/** The flow a state belongs to; a pre-ADR-010 checkpoint is a full run. */
export function flowOf(state: Pick<PipelineState, 'flow'>): Flow {
  return state.flow ?? 'full';
}

/**
 * The registry entry for `flow`, throwing with the canonical message for a
 * value the registry does not define. Every flow-parameterised gate calls this
 * first, so an unknown flow is a validation failure everywhere — never a
 * silent fallback to `full`, whose rules are the wider ones.
 */
export function flowDef(flow: FlowId): FlowDef {
  const def = (FLOWS as Record<string, FlowDef>)[flow];
  if (def === undefined) {
    throw new Error(`unknown flow "${flow}" (expected ${Object.keys(FLOWS).join(' or ')})`);
  }
  return def;
}

/**
 * The config for a flow — the single read point for every consumer of a budget
 * or seat value. The caller's base is validated first (the seat panel is
 * untrusted caller data: an empty code panel would let `seatsConverged` return
 * vacuously true and a code review "converge" with zero sessions), then the
 * flow's partial lands last, so a caller-supplied five-seat panel cannot widen
 * a light run while the operator's `--max-parallel` survives.
 */
export function configFor(
  flow: Flow,
  base: Partial<PipelineConfig> = DEFAULT_CONFIG,
): PipelineConfig {
  const def = flowDef(flow);
  validateConfig(base, def, flow);
  const merged: PipelineConfig = { ...DEFAULT_CONFIG, ...base, ...def.config };
  validateConfig(merged, def, flow);
  // The resolved panel is what every seat reader reads, so it must name a panel
  // for each phase the flow runs. A caller's base may name fewer keys than that
  // — the flow's partial is the last writer of `reviewSeats` — so presence is a
  // rule about the resolved config, never about the data handed in.
  validatePanelPresence(merged, def, flow);
  return merged;
}

/**
 * `seatsFor(phase, configFor(flow))` is the panel that flow runs. The panel is
 * caller data, so `configFor` is what proves the names are seats, not this
 * function: it reads the resolved config, falling back to the standing five
 * for a config that names no panel (`DEFAULT_CONFIG`, pre-ADR-010 shaped).
 */
export function seatsFor(phase: SeatPhase, cfg: PipelineConfig): readonly string[] {
  const seats = cfg.reviewSeats?.[phase];
  if (seats === undefined) {
    return [...(phase === 'code' ? CODE_SPECIALTIES : DESIGN_SPECIALTIES)];
  }
  return [...seats];
}

/** `flow` is a flow that has `stage` in its stage sequence. */
export function isStageInFlow(stage: string, flow: FlowId): boolean {
  return flowDef(flow).stages.includes(stage);
}

/** Throws the usage error when `flow` does not run `stage`. */
export function assertStageInFlow(flow: Flow, stage: string): void {
  if (!isStageInFlow(stage, flow)) {
    throw new Error(
      `stage '${stage}' is not part of flow '${flow}' ` +
        `(the ${flow} flow runs: ${flowDef(flow).stages.join(', ')})`,
    );
  }
}

// --- validation --------------------------------------------------------------

const PANELS: Record<SeatPhase, readonly string[]> = {
  design: DESIGN_SPECIALTIES,
  code: CODE_SPECIALTIES,
};

function validateConfig(cfg: Partial<PipelineConfig>, def: FlowDef, flow: Flow): void {
  validateSeatNames(cfg);
  validatePlanMaxLines(cfg);
  validateDocMaxLines(cfg);
  validatePanels(cfg, def, flow);
}

/**
 * Every review phase the flow runs must resolve to a panel. A `reviewSeats`
 * that names fewer phases than the flow runs is a usage error with the same
 * shape as the other panel refusals, not a `TypeError` out of `seatsFor` at the
 * first pass entry. Only the resolved config is judged: the flow's own partial
 * is the last writer of the panel, so a base naming one phase key still
 * resolves to the flow's pair.
 */
function validatePanelPresence(cfg: PipelineConfig, def: FlowDef, flow: Flow): void {
  const seats = cfg.reviewSeats;
  if (seats === undefined) {
    return;
  }
  for (const phase of phasesRun(def)) {
    if (seats[phase] === undefined) {
      throw new Error(
        `reviewSeats.${phase} is not a list of seat names — flow '${flow}' runs the ` +
          `${phase} review phase and the resolved panel names none (the ${phase} panel's seats: ` +
          `${PANELS[phase].join(', ')})`,
      );
    }
  }
}

/**
 * A seat name is valid for the panel that names it: each panel is the set of
 * seat agents that exists (`review/design-*` and `codereview/code-*`), so
 * `complexity` — a design seat — is a usage error in `reviewSeats.code`, where
 * it would resolve to a `codereview/code-complexity` agent that no run can
 * prompt. A phase key that is not a list is the same clean refusal.
 */
function validateSeatNames(cfg: Partial<PipelineConfig>): void {
  const seats = cfg.reviewSeats;
  if (seats === undefined) {
    return;
  }
  for (const phase of ['design', 'code'] as const) {
    const panel = seats[phase];
    if (panel === undefined) {
      // A phase key the caller did not write is not a name to check; presence is
      // the resolved config's rule (`validatePanelPresence`).
      continue;
    }
    if (!Array.isArray(panel)) {
      throw new Error(
        `reviewSeats.${phase} is not a list of seat names — a review panel is an array ` +
          `(the ${phase} panel's seats: ${PANELS[phase].join(', ')})`,
      );
    }
    for (const seat of panel) {
      if (!PANELS[phase].includes(seat)) {
        throw new Error(
          `unknown review seat "${seat}" in reviewSeats.${phase} ` +
            `(the ${phase} panel's seats: ${PANELS[phase].join(', ')})`,
        );
      }
    }
  }
}

function validatePlanMaxLines(cfg: Partial<PipelineConfig>): void {
  const max = cfg.planMaxLines;
  if (max === undefined) {
    return;
  }
  if (!Number.isInteger(max) || max <= 0) {
    throw new Error(`planMaxLines must be a positive integer (got ${String(max)})`);
  }
}

/**
 * ADR-016 Decision 4: the same shape as `validatePlanMaxLines`, so a
 * non-positive `docMaxLines` is a usage error rather than a run that silently
 * carries no document ceiling while its meter still prints line counts.
 */
function validateDocMaxLines(cfg: Partial<PipelineConfig>): void {
  const max = cfg.docMaxLines;
  if (max === undefined) {
    return;
  }
  if (!Number.isInteger(max) || max <= 0) {
    throw new Error(`docMaxLines must be a positive integer (got ${String(max)})`);
  }
}

/**
 * The empty-panel rule is scoped to the review phases the flow's own stages
 * run: `design: []` is a usage error under `full`, whose editorial round reads
 * that panel, and resolves under `light`, which runs no design stage at all.
 * A phase key that is absent is not an empty panel — it is the case the flow's
 * partial fills (`validatePanelPresence` judges the resolved config).
 */
function validatePanels(cfg: Partial<PipelineConfig>, def: FlowDef, flow: Flow): void {
  const seats = cfg.reviewSeats;
  if (seats === undefined) {
    return;
  }
  for (const phase of phasesRun(def)) {
    const panel = seats[phase];
    if (panel === undefined) {
      continue;
    }
    if (panel.length === 0) {
      throw new Error(
        `reviewSeats.${phase} is empty under flow '${flow}' — a review phase cannot run an ` +
          `empty panel (the ${flow} flow runs: ${def.stages.join(', ')})`,
      );
    }
  }
}

/** Which review panels a flow's stage list runs. */
function phasesRun(def: FlowDef): SeatPhase[] {
  const runs: SeatPhase[] = [];
  if (def.stages.some((s) => s === 'design-review' || s === 'editorial-review')) {
    runs.push('design');
  }
  if (def.stages.includes('code-review')) {
    runs.push('code');
  }
  return runs;
}
