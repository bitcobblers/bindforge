import { describe, expect, it } from 'vitest';

import { configFor } from '../../src/pipeline/flows.js';
import { documentOutgrewFlow } from '../../src/pipeline/gates.js';
import { DEFAULT_CONFIG } from '../../src/pipeline/stages.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';

// ---------------------------------------------------------------------------
// ADR-016 Decision 4, the document ceiling. `documentOutgrewFlow` is the full
// flow's sibling of `planOutgrewFlow`: over length turns the stage attempt into
// a normal failure naming the count and the cap (so the retry-then-escalate
// budget repairs it), under-cap and an absent cap leave the measure-and-warn
// posture untouched. The cap itself is registry config — `docMaxLines` in
// `FLOWS.full.config`, set to 900 (headroom above the largest shipped full-flow
// document) — validated like `planMaxLines`, and left absent for the light flow
// whose plan stays under `planMaxLines`.
// ---------------------------------------------------------------------------

function denialOf(gate: unknown): string | null {
  if (typeof gate === 'string') return gate === '' ? null : gate;
  if (gate && typeof gate === 'object') {
    const g = gate as { proceed?: boolean; reason?: string };
    if (typeof g.proceed === 'boolean') return g.proceed ? null : (g.reason ?? 'denied');
    if (typeof g.reason === 'string' && g.reason.length > 0) return g.reason;
  }
  return gate ? JSON.stringify(gate) : null;
}

const FULL: PipelineConfig = { ...DEFAULT_CONFIG, docMaxLines: 900 };

describe('documentOutgrewFlow (ADR-016 Decision 4)', () => {
  // proves: Test strategy #6 — over the cap denies, and the reason carries both
  // the line count and the cap so the repair names what to cut to.
  it('denies a document over the cap, naming the count and the cap', () => {
    const denial = denialOf(documentOutgrewFlow(901, FULL));
    expect(denial, 'a 901-line document outgrew the cap').not.toBeNull();
    expect(denial as string).toContain('document outgrew');
    expect(denial as string).toContain('901');
    expect(denial as string).toContain('900');
  });

  // proves: Test strategy #6 — under the cap (and exactly at it) allows.
  it('allows a document at or under the cap', () => {
    expect(denialOf(documentOutgrewFlow(899, FULL))).toBeNull();
    expect(denialOf(documentOutgrewFlow(900, FULL))).toBeNull();
  });

  // proves: Test strategy #6 — an absent cap is the unchanged measure-and-warn
  // posture: the gate never fires.
  it('gates nothing when the cap is absent', () => {
    expect(denialOf(documentOutgrewFlow(5_000, DEFAULT_CONFIG))).toBeNull();
    expect(denialOf(documentOutgrewFlow(5_000, configFor('light', DEFAULT_CONFIG)))).toBeNull();
  });
});

describe('the docMaxLines flow config (ADR-016 Decision 4)', () => {
  // proves: Test strategy #6 — the full flow ships `docMaxLines: 900`; the light
  // flow leaves it absent (its plan stays under `planMaxLines`).
  it('sets 900 for the full flow and leaves it absent for light', () => {
    expect(configFor('full', DEFAULT_CONFIG).docMaxLines).toBe(900);
    expect(configFor('light', DEFAULT_CONFIG).docMaxLines).toBeUndefined();
  });

  // proves: Test strategy #6 — a non-positive `docMaxLines` is a validation
  // failure, not a silent no-cap run: the same shape `validatePlanMaxLines`
  // already enforces.
  it('refuses a non-positive docMaxLines', () => {
    for (const bad of [0, -1]) {
      expect(
        () => configFor('full', { ...DEFAULT_CONFIG, docMaxLines: bad }),
        `docMaxLines: ${String(bad)}`,
      ).toThrow(/docMaxLines/);
    }
  });
});
