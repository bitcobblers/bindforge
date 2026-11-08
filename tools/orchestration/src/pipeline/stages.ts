import { StageReplySchemas } from './types.js';
import type { StageKind } from './types.js';

// ---------------------------------------------------------------------------
// Stage registry: stage -> agent id -> prompt template -> reply schema, and
// the pipeline config defaults. This module is the single source of record
// for per-stage `steps` caps (agent files must NOT set `steps`).
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
  maxRounds: number;
  stepsDefault: number;
  blockingSeverities: string[];
  pollIntervalMs: number;
  stageTimeoutMs: number;
}

export const DEFAULT_CONFIG: PipelineConfig = {
  maxRounds: 5,
  stepsDefault: 40,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 2_000,
  stageTimeoutMs: 15 * 60_000,
};

export interface StageDef {
  name: string;
  kind: StageKind;
  agent: string;
  /** Template file under prompts/, or null for a fixed built-in instruction. */
  template: string | null;
  steps: number;
}

interface StageBase {
  kind: StageKind;
  /** Agent id, or a function of the review specialty. */
  agent: string | ((specialty: string) => string);
  template: string | null;
  steps?: number;
}

const STAGE_BASES: Record<string, StageBase> = {
  research: { kind: 'research', agent: 'research/researcher', template: 'research.md' },
  proposal: { kind: 'proposal', agent: 'design/designer', template: 'proposal.md' },
  'proposal-revise': {
    kind: 'proposal',
    agent: 'design/designer',
    template: 'proposal-revise.md',
  },
  'proposal-final': {
    kind: 'proposal-final',
    agent: 'design/editor',
    template: 'proposal-final.md',
    steps: 20,
  },
  'design-review': {
    kind: 'review',
    agent: (s) => `review/design-${s}`,
    template: 'design-review.md',
    steps: 30,
  },
  'editorial-review': {
    kind: 'review',
    agent: (s) => `review/design-${s}`,
    template: 'design-review.md',
    steps: 25,
  },
  'test-authoring': {
    kind: 'test-authoring',
    agent: 'code/test-engineer',
    template: 'test-authoring.md',
    steps: 45,
  },
  implement: { kind: 'implement', agent: 'code/coder', template: 'implement.md', steps: 60 },
  fix: { kind: 'fix', agent: 'code/coder', template: 'fix.md', steps: 50 },
  'test-update': { kind: 'test-update', agent: 'code/test-engineer', template: 'test-update.md' },
  'code-review': {
    kind: 'review',
    agent: (s) => `codereview/code-${s}`,
    template: 'code-review.md',
    steps: 35,
  },
  'doc-sync': { kind: 'doc-sync', agent: 'ops/doc-sync', template: 'doc-sync.md', steps: 40 },
};

export const STAGE_NAMES = Object.keys(STAGE_BASES);

/**
 * Resolve a stage definition. Review stages take their specialty (which also
 * selects the agent); non-review stages ignore it.
 */
export function stageDef(
  name: string,
  specialty: string | undefined,
  cfg: PipelineConfig,
): StageDef {
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
    steps: base.steps ?? cfg.stepsDefault,
  };
}

/** Reply schema a stage's JSON reply is validated against. */
export function stageReplySchema(def: StageDef) {
  return StageReplySchemas[def.kind];
}
