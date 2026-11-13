import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { FLOWS, assertStageInFlow, configFor, flowOf, seatsFor } from '../../src/pipeline/flows.js';
import {
  CODE_SPECIALTIES,
  DEFAULT_CONFIG,
  DESIGN_SPECIALTIES,
  type PipelineConfig,
  STAGE_NAMES,
} from '../../src/pipeline/stages.js';

// ---------------------------------------------------------------------------
// ADR-010 flow registry: the stage list, the review seats, the budgets, the
// document-size stages and the plan line cap become registry data, resolved
// once through `configFor`. `full` is the untouched status quo (its config
// partial is empty and `configFor('full', base)` is deep-equal to a complete
// `base`), and the seat panel is caller data, so the resolver validates it:
// known specialty names, `planMaxLines` absent-or-positive, and a non-empty
// panel for every review phase the flow's own stages run.
// ---------------------------------------------------------------------------

const LIGHT_SEATS = ['correctness', 'testing'];
const FULL_STAGES = [
  'code-review',
  'design-review',
  'doc-sync',
  'editorial-review',
  'fix',
  'implement',
  'proposal',
  'proposal-final',
  'proposal-revise',
  'research',
  'test-authoring',
  'test-update',
];

/** The CLI's own config shape (`cli.ts`: `{ ...DEFAULT_CONFIG, maxParallelReviews }`). */
function cliConfig(maxParallelReviews = 1): PipelineConfig {
  return { ...DEFAULT_CONFIG, maxParallelReviews };
}

describe('flow registry shape', () => {
  // proves: Test strategy #1 — the light flow is exactly the four stages ADR-010
  // names, in declaration order.
  it('declares the light stages as plan, implement, fix, code-review', () => {
    expect(FLOWS.light.stages).toEqual(['plan', 'implement', 'fix', 'code-review']);
  });

  // proves: Test strategy #1 — the size meter measures `plan` for light and the
  // three proposal stages for full (replaces `DESIGN_DOC_STAGES`).
  it('declares which stages the document-size meter measures', () => {
    expect(FLOWS.light.docStages).toEqual(['plan']);
    expect(FLOWS.full.docStages).toEqual(['proposal', 'proposal-revise', 'proposal-final']);
  });

  // proves: Test strategy #1 — the regression guard on `full`: today's twelve
  // stages, listed exactly, and every stage the runner already knows.
  it('lists the full flow twelve stages exactly', () => {
    expect([...FLOWS.full.stages].sort()).toEqual(FULL_STAGES);
    expect([...FLOWS.full.stages].sort()).toEqual([...STAGE_NAMES].sort());
  });

  // proves: Test strategy #1 — the light panel is the two seats, and light has
  // no design phase at all.
  it('declares the light seats as correctness and testing with no design seats', () => {
    expect(FLOWS.light.config.reviewSeats?.code).toEqual(LIGHT_SEATS);
    expect(FLOWS.light.config.reviewSeats?.design).toEqual([]);
  });

  // proves: Test strategy #1/#3 — `full` carries an empty config partial, so
  // `DEFAULT_CONFIG` stays the sole default authority for it.
  it('keeps the full flow config partial empty', () => {
    expect(FLOWS.full.config).toEqual({});
    expect(FLOWS.full.name).toBe('full');
    expect(FLOWS.light.name).toBe('light');
  });
});

describe('configFor: the light budgets', () => {
  // proves: Test strategy #2 — the documented light numbers, applied over
  // DEFAULT_CONFIG, with the operator's `--max-parallel` intact by spread
  // order alone (no key is written after the flow partial).
  it('applies the light partial and leaves the lane knob last', () => {
    const cfg = configFor('light', cliConfig(1));
    expect(cfg.maxRounds).toBe(2);
    expect(cfg.maxReviews).toEqual({ design: 0, code: 6 });
    expect(cfg.maxPassesPerRound).toBe(3);
    expect(cfg.stageTimeoutMs).toBe(12 * 60_000);
    expect(cfg.planMaxLines).toBe(80);
    expect(cfg.reviewSeats?.code).toEqual(LIGHT_SEATS);
    expect(cfg.maxParallelReviews).toBe(1);
  });

  // proves: Test strategy #2 — a wider operator lane survives the flow partial.
  it('applies the operator lane width it was handed', () => {
    expect(configFor('light', cliConfig(2)).maxParallelReviews).toBe(2);
  });

  // proves: Test strategy #2 — the knobs the light partial does not name are
  // shared by construction, not copied: they equal DEFAULT_CONFIG's.
  it('shares every knob the light partial does not name', () => {
    const cfg = configFor('light', cliConfig());
    expect(cfg.blockingSeverities).toEqual(DEFAULT_CONFIG.blockingSeverities);
    expect(cfg.pollIntervalMs).toBe(DEFAULT_CONFIG.pollIntervalMs);
    expect(cfg.requestTimeoutMs).toBe(DEFAULT_CONFIG.requestTimeoutMs);
    expect(cfg.minNudgeIntervalMs).toBe(DEFAULT_CONFIG.minNudgeIntervalMs);
  });
});

describe('configFor: the full flow is untouched', () => {
  // proves: Test strategy #3 — `configFor('full', base)` is deep-equal to a
  // complete `base`: the full flow's behavior cannot drift through the
  // resolution seam.
  it('returns a complete base unchanged', () => {
    const base: PipelineConfig = {
      maxRounds: 5,
      maxReviews: { design: 40, code: 20 },
      maxPassesPerRound: 6,
      blockingSeverities: ['blocker', 'major'],
      pollIntervalMs: 1,
      stageTimeoutMs: 5_000,
      requestTimeoutMs: 1_000,
      minNudgeIntervalMs: 0,
      maxParallelReviews: 2,
      reviewSeats: { design: [...DESIGN_SPECIALTIES], code: [...CODE_SPECIALTIES] },
    };
    expect(configFor('full', base)).toEqual(base);
    expect(configFor('full', cliConfig())).toEqual(cliConfig());
  });

  // proves: Test strategy #3 (Non-goals; Risks, budget-drift row) — the full
  // flow's cost ceilings are pinned on `DEFAULT_CONFIG` itself. Asserting
  // `configFor('full', base)` against a literal `base` is tautological against
  // the constant, so the knob values are the assertion: a silent edit of the
  // pass backstop or the stage timeout is a test failure.
  it('pins the full flow cost ceilings on DEFAULT_CONFIG', () => {
    expect(DEFAULT_CONFIG.maxPassesPerRound).toBe(6);
    expect(DEFAULT_CONFIG.stageTimeoutMs).toBe(30 * 60_000);
    expect(DEFAULT_CONFIG.maxRounds).toBe(5);
    expect(DEFAULT_CONFIG.maxReviews).toEqual({ design: 40, code: 20 });
    expect(DEFAULT_CONFIG.planMaxLines).toBeUndefined();

    const cfg = configFor('full', DEFAULT_CONFIG);
    expect(cfg.maxPassesPerRound).toBe(6);
    expect(cfg.stageTimeoutMs).toBe(30 * 60_000);
  });

  // proves: Test strategy #3 — the additive-shape guarantee: a complete config
  // literal written without the two new optional fields keeps compiling and
  // resolves to the five code seats.
  it('resolves a legacy-shaped literal to the five code seats', () => {
    const legacy: PipelineConfig = {
      maxRounds: 5,
      maxReviews: { design: 40, code: 20 },
      maxPassesPerRound: 6,
      blockingSeverities: ['blocker', 'major'],
      pollIntervalMs: 1,
      stageTimeoutMs: 5_000,
      requestTimeoutMs: 1_000,
    };
    const cfg = configFor('full', legacy);
    expect(cfg.planMaxLines).toBeUndefined();
    expect(seatsFor('code', cfg)).toEqual(CODE_SPECIALTIES);
    expect(seatsFor('design', cfg)).toEqual(DESIGN_SPECIALTIES);
  });

  // proves: Test strategy #4 — the single read seam for `state.flow`: an
  // absent key is `full`, so every pre-flow checkpoint behaves as today.
  it('reads an absent flow key as full', () => {
    expect(flowOf({})).toBe('full');
    expect(flowOf({ flow: 'light' })).toBe('light');
    expect(flowOf({ flow: 'full' })).toBe('full');
  });
});

describe('seatsFor: one resolver for every seat reader', () => {
  // proves: Test strategy #1/#2 (Budget resolution) — the light flow's code
  // panel is the registry pair and the full flow's is the standing five.
  it('resolves the panel from the flow config, not from a *_SPECIALTIES import', () => {
    expect(seatsFor('code', configFor('light', cliConfig()))).toEqual(LIGHT_SEATS);
    expect(seatsFor('code', configFor('full', cliConfig()))).toEqual(CODE_SPECIALTIES);
  });

  // proves: Test strategy #1/#2 (the registry contract, `seatsFor(): readonly
  // string[]`) — the resolver hands out a copy: a reader cannot append to the
  // panel the registry declares, so the seats a run launches are the seats the
  // flow names, for the whole run.
  it("hands out a copy, never the registry's own seat array", () => {
    const light = seatsFor('code', configFor('light', cliConfig())) as string[];
    light.push('security');
    expect(seatsFor('code', configFor('light', cliConfig()))).toEqual(LIGHT_SEATS);
    expect(FLOWS.light.config.reviewSeats?.code).toEqual(LIGHT_SEATS);

    const full = seatsFor('code', configFor('full', cliConfig())) as string[];
    full.push('complexity');
    expect(seatsFor('code', configFor('full', cliConfig()))).toEqual(CODE_SPECIALTIES);
    expect(CODE_SPECIALTIES).toEqual([
      'correctness',
      'compatibility',
      'security',
      'performance',
      'testing',
    ]);
  });
});

describe('configFor: the base is a partial', () => {
  // proves: Test strategy #2/#3 (A4's seams; the registry contract,
  // `configFor(flow, base?: Partial<PipelineConfig>)`) — a caller hands a
  // partial and gets a complete config: `DEFAULT_CONFIG` remains the sole
  // default authority, and the flow partial applies over it.
  it('fills a partial base from DEFAULT_CONFIG', () => {
    expect(configFor('full', {})).toEqual(DEFAULT_CONFIG);

    const light = configFor('light', { maxParallelReviews: 3 });
    expect(light.maxParallelReviews).toBe(3);
    expect(light.maxRounds).toBe(2);
    expect(light.maxReviews).toEqual({ design: 0, code: 6 });
    expect(light.maxPassesPerRound).toBe(3);
    expect(light.planMaxLines).toBe(80);
    expect(light.pollIntervalMs).toBe(DEFAULT_CONFIG.pollIntervalMs);
  });
});

describe('assertStageInFlow', () => {
  // proves: Test strategy #6 (Selecting the flow) — a `--stage` the flow does
  // not declare is a usage error before the lock, in both directions.
  it('rejects a stage the flow does not declare', () => {
    expect(() => assertStageInFlow('light', 'plan')).not.toThrow();
    expect(() => assertStageInFlow('light', 'code-review')).not.toThrow();
    expect(() => assertStageInFlow('full', 'research')).not.toThrow();
    expect(() => assertStageInFlow('light', 'research')).toThrow(/light/);
    expect(() => assertStageInFlow('light', 'research')).toThrow(/research/);
    expect(() => assertStageInFlow('full', 'plan')).toThrow(/full/);
  });
});

describe('configFor validation: the seat panel is untrusted caller data', () => {
  // proves: Test strategy #23 — an empty code panel is a usage error for every
  // flow: unvalidated, `seatsConverged` is vacuously true and a code review
  // "converges" with zero sessions.
  it('rejects an empty code panel for every flow', () => {
    const base: PipelineConfig = {
      ...cliConfig(),
      reviewSeats: { design: [...DESIGN_SPECIALTIES], code: [] },
    };
    expect(() => configFor('light', base)).toThrow(/reviewSeats/);
    expect(() => configFor('full', base)).toThrow(/reviewSeats/);
  });

  // proves: Test strategy #23 — the emptiness rule is scoped to the phases the
  // flow's `stages` runs: `design: []` throws under `full` (its editorial round
  // reads the design panel) and resolves under `light`, which runs no design
  // stage. The registry's own light `design: []` must resolve, so #1 holds.
  it('scopes the design-panel rule to the flow that runs a design phase', () => {
    const noDesign: PipelineConfig = {
      ...cliConfig(),
      reviewSeats: { design: [], code: [...LIGHT_SEATS] },
    };
    expect(() => configFor('full', noDesign)).toThrow(/reviewSeats/);
    expect(configFor('light', noDesign).reviewSeats?.design).toEqual([]);
    const registryLight = configFor('light', { ...cliConfig(), ...FLOWS.light.config });
    expect(registryLight.reviewSeats?.design).toEqual([]);
  });

  // proves: Test strategy #23 — a seat name no agent backs is a usage error.
  it('rejects an unknown seat name', () => {
    const typo: PipelineConfig = {
      ...cliConfig(),
      reviewSeats: { design: [...DESIGN_SPECIALTIES], code: ['corectness'] },
    };
    expect(() => configFor('full', typo)).toThrow(/corectness/);
    const lightTypo: PipelineConfig = {
      ...cliConfig(),
      reviewSeats: { design: [], code: ['corectness'] },
    };
    expect(() => configFor('light', lightTypo)).toThrow(/corectness/);
  });

  // proves: Test strategy #23 (security r1 finding 3) — each panel names only
  // its own seats: `complexity` is a design seat, so a code panel naming it
  // resolves no `codereview/code-*` agent, and `testing` is a code seat, so a
  // design panel naming it resolves no `review/design-*` agent. Each is the
  // pre-lock usage error, not a failed stage attempt after the lock and an
  // escalation record.
  it('rejects a seat named in the panel it does not belong to', () => {
    const designInCode: PipelineConfig = {
      ...cliConfig(),
      reviewSeats: { design: [...DESIGN_SPECIALTIES], code: ['complexity'] },
    };
    expect(() => configFor('full', designInCode)).toThrow(/complexity/);
    expect(() => configFor('light', designInCode)).toThrow(/complexity/);

    const codeInDesign: PipelineConfig = {
      ...cliConfig(),
      reviewSeats: { design: ['testing'], code: [...CODE_SPECIALTIES] },
    };
    expect(() => configFor('full', codeInDesign)).toThrow(/testing/);
  });

  // proves: Test strategy #23 (security r1 finding 3) — a `reviewSeats` that
  // omits a phase key is the same clean usage error, not a `TypeError` out of
  // the resolver: the caller's panel is validated before it is read.
  it('rejects a panel with a missing phase key', () => {
    const noCode = { design: [...DESIGN_SPECIALTIES] } as PipelineConfig['reviewSeats'];
    const noDesign = { code: [...CODE_SPECIALTIES] } as PipelineConfig['reviewSeats'];
    const omitCode: PipelineConfig = { ...cliConfig(), reviewSeats: noCode };
    const omitDesign: PipelineConfig = { ...cliConfig(), reviewSeats: noDesign };
    expect(() => configFor('full', omitCode)).toThrow(/reviewSeats/);
    expect(() => configFor('full', omitDesign)).toThrow(/reviewSeats/);
    // light runs no design stage and its partial is the last writer of the
    // panel, so a base naming one key still resolves the registry's pair.
    const light = configFor('light', omitCode);
    expect(light.reviewSeats?.code).toEqual(LIGHT_SEATS);
  });

  // proves: Test strategy #23 — `planMaxLines` is absent-or-positive: the full
  // flow's absence passes, a 0 is a usage error.
  it('rejects a zero plan cap and accepts its absence', () => {
    expect(configFor('full', cliConfig()).planMaxLines).toBeUndefined();
    expect(() => configFor('full', { ...cliConfig(), planMaxLines: 0 })).toThrow(/planMaxLines/);
    expect(() => configFor('light', { ...cliConfig(), planMaxLines: 0 })).toThrow(/planMaxLines/);
  });

  // proves: Test strategy #23 — the flow partial is the last writer of its own
  // keys, so a caller-supplied five-seat panel cannot widen a light run.
  it('keeps the registry seats when the base carries a wider panel', () => {
    const wide: PipelineConfig = {
      ...cliConfig(),
      reviewSeats: { design: [...DESIGN_SPECIALTIES], code: [...CODE_SPECIALTIES] },
    };
    expect(configFor('light', wide).reviewSeats?.code).toEqual(LIGHT_SEATS);
  });
});

describe('platform surface', () => {
  // proves: Test strategy #21 — no new platform surface: the Node floor is the
  // package's existing `engines` pin and the registry adds no OS branch.
  it('pins the engines floor and adds no platform-specific path', () => {
    const pkg = JSON.parse(
      readFileSync(fileURLToPath(new URL('../../package.json', import.meta.url)), 'utf8'),
    ) as { engines?: { node?: string } };
    expect(pkg.engines?.node).toBe('>=22.23.4');

    const registry = readFileSync(
      fileURLToPath(new URL('../../src/pipeline/flows.ts', import.meta.url)),
      'utf8',
    );
    expect(registry).not.toMatch(/process\.platform/);
    expect(registry).not.toMatch(/win32|darwin/);
  });
});
