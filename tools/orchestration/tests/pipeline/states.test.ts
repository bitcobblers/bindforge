import { describe, expect, it } from 'vitest';

import { configFor } from '../../src/pipeline/flows.js';
import { CODE_SPECIALTIES, DEFAULT_CONFIG, DESIGN_SPECIALTIES } from '../../src/pipeline/stages.js';
import { type NextStep, type ReviewPhase, decideNext } from '../../src/pipeline/states.js';
import type { PipelineState, VerdictReply } from '../../src/pipeline/types.js';

function freshState(): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '001',
    branch: 'pipeline/001-x',
    source: { kind: 'brief', path: 'docs/prompts/x.md' },
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
    forceNext: null,
    results: [],
    gateLog: [],
    startedAt: 0,
    updatedAt: 0,
  };
}

/**
 * Apply what the runner's drive loop applies at a funnel pass: rounds spend
 * budget only on entry to a NEW round, a pass refreshes the revision stamp,
 * and finishing a pass advances the cycle counter.
 */
function enterReview(state: PipelineState, step: NextStep): void {
  if (step.kind !== 'review') return;
  if (step.phase === 'design') {
    if (step.round > state.designRound) {
      state.designCycle = 0;
      state.designBudgetUsed += 1;
    }
    state.designRound = step.round;
    state.designStamp = `s${String(step.round)}`;
    state.designCycle = (state.designCycle ?? 0) + 1;
  } else if (step.phase === 'code') {
    if (step.round > state.codeRound) {
      state.codeCycle = 0;
      state.codeBudgetUsed += 1;
    }
    state.codeRound = step.round;
    state.codeStamp = `h${String(step.round)}`;
    state.codeCycle = (state.codeCycle ?? 0) + 1;
  } else {
    state.designRound = step.round; // editorial: numbering only
  }
  state.lastStage = `${step.phase}-review`;
}

/** Record the pass's verdicts both places the runner keeps them. */
function withVerdicts(
  state: PipelineState,
  phase: ReviewPhase,
  reply: VerdictReply,
  seats?: Record<string, 'approve' | 'request-changes'>,
): void {
  const specialties = phase === 'code' ? CODE_SPECIALTIES : DESIGN_SPECIALTIES;
  const round = phase === 'code' ? state.codeRound : state.designRound;
  for (const specialty of specialties) {
    state.results.push({
      stage: `${phase}-review:${specialty}`,
      round,
      sessionID: 's',
      agent: 'a',
      at: 0,
      reply,
      transcript: 't',
    });
    if (phase === 'design' || phase === 'code') {
      const board = phase === 'design' ? 'design' : 'code';
      const seatStamp = phase === 'design' ? (state.designStamp ?? '') : (state.codeStamp ?? '');
      const verdict =
        seats?.[specialty] ?? (reply.verdict === 'approve' ? 'approve' : 'request-changes');
      state.seats = {
        ...state.seats,
        [board]: {
          ...state.seats?.[board],
          [specialty]: {
            verdict,
            stamp: seatStamp,
            dissents: verdict === 'request-changes' ? 1 : 0,
          },
        },
      };
    }
  }
}

const approve: VerdictReply = { verdict: 'approve', findings: [] };
const reject: VerdictReply = {
  verdict: 'request-changes',
  findings: [{ severity: 'major', location: '§1', problem: 'nope' }],
};

function stageOf(step: NextStep): string {
  if (step.kind !== 'stage') throw new Error(`expected stage, got ${step.kind}`);
  return step.stage;
}

describe('decideNext: happy path', () => {
  it('walks research -> ... -> done with approvals and green tests', () => {
    const s = freshState();
    const cfg = DEFAULT_CONFIG;

    expect(stageOf(decideNext(s, cfg))).toBe('research');
    s.lastStage = 'research';
    expect(stageOf(decideNext(s, cfg))).toBe('proposal');
    s.lastStage = 'proposal';

    let step = decideNext(s, cfg);
    expect(step).toEqual({ kind: 'review', phase: 'design', round: 1 });
    enterReview(s, step);
    withVerdicts(s, 'design', approve);
    expect(stageOf(decideNext(s, cfg))).toBe('proposal-final');
    s.lastStage = 'proposal-final';

    step = decideNext(s, cfg);
    expect(step).toEqual({ kind: 'review', phase: 'editorial', round: 2 });
    enterReview(s, step);
    withVerdicts(s, 'editorial', approve);
    expect(stageOf(decideNext(s, cfg))).toBe('test-authoring');
    s.lastStage = 'test-authoring';
    expect(stageOf(decideNext(s, cfg))).toBe('implement');
    s.lastStage = 'implement';

    // implement green
    s.testsGreen = true;
    step = decideNext(s, cfg);
    expect(step).toEqual({ kind: 'review', phase: 'code', round: 1 });
    enterReview(s, step);
    withVerdicts(s, 'code', approve);
    expect(stageOf(decideNext(s, cfg))).toBe('doc-sync');
    s.lastStage = 'doc-sync';
    expect(decideNext(s, cfg)).toEqual({ kind: 'done' });
  });

  it('authoring precedes implementation (test-first)', () => {
    const s = freshState();
    s.lastStage = 'editorial-review';
    return void s; // handled in full walk above; kept for intent
  });
});

describe('decideNext: review loops and budgets', () => {
  it('routes a rejected design round back through proposal-revise', () => {
    const s = freshState();
    s.lastStage = 'proposal';
    const step = decideNext(s, DEFAULT_CONFIG);
    enterReview(s, step);
    withVerdicts(s, 'design', reject);
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('proposal-revise');
    s.lastStage = 'proposal-revise';
    // ADR-002: the revision goes back to the dissenters inside round 1, at a
    // higher pass number — a new round would spend a revise-round grant.
    expect(decideNext(s, DEFAULT_CONFIG)).toEqual({ kind: 'review', phase: 'design', round: 1 });
  });

  // proves: Budget resolution — every seat-list reader takes the `cfg` its
  // caller already holds (security r1 finding 5): the dissenter list that routes
  // the `proposal-revise` transition is the caller's resolved panel, not the
  // standing five. A dissent from a seat the panel does not name is dropped from
  // the revision input, exactly as the panel says, and the design review opens
  // as a new round.
  it('drives the dissenter list from the caller-supplied design panel', () => {
    const seatsDissenting = (dissenter: string): Record<string, 'approve' | 'request-changes'> => {
      const seats: Record<string, 'approve' | 'request-changes'> = {};
      for (const sp of DESIGN_SPECIALTIES) {
        seats[sp] = sp === dissenter ? 'request-changes' : 'approve';
      }
      return seats;
    };

    const standing = configFor('full', DEFAULT_CONFIG);
    const narrow = configFor('full', {
      ...DEFAULT_CONFIG,
      reviewSeats: { design: ['compatibility'], code: [...CODE_SPECIALTIES] },
    });

    // Design round 1, one dissenter: `complexity`, a design seat.
    const atRevise = (): PipelineState => {
      const s = freshState();
      s.lastStage = 'proposal';
      enterReview(s, decideNext(s, standing));
      withVerdicts(s, 'design', reject, seatsDissenting('complexity'));
      s.lastStage = 'proposal-revise';
      return s;
    };

    // The standing five name the dissenter: the revision goes back to it inside
    // round 1, at a higher pass number.
    const inPanel = decideNext(atRevise(), standing);
    expect(inPanel).toEqual({ kind: 'review', phase: 'design', round: 1 });

    // The caller's panel names only the seat that approved: the same state
    // resolves to no dissenter, so the design review is a NEW round.
    const outOfPanel = decideNext(atRevise(), narrow);
    expect(outOfPanel.kind).toBe('review');
    expect(outOfPanel.kind === 'review' && outOfPanel.round).toBeGreaterThan(1);
  });

  it('revisiting stale approvals runs a ratification pass in the same round', () => {
    const s = freshState();
    s.lastStage = 'proposal';
    enterReview(s, decideNext(s, DEFAULT_CONFIG)); // design r1 c0, full panel
    withVerdicts(s, 'design', approve);
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('proposal-final');
    s.lastStage = 'proposal-final';
    enterReview(s, decideNext(s, DEFAULT_CONFIG)); // editorial r2
    withVerdicts(s, 'editorial', reject);
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('proposal-revise');
    s.lastStage = 'proposal-revise';
    // No design dissenter: the next design review is a NEW round whose seats
    // are all stale approvals — a ratification pass, not a fresh full panel.
    const step = decideNext(s, DEFAULT_CONFIG);
    expect(step).toEqual({ kind: 'review', phase: 'design', round: 3 });
    enterReview(s, step); // refreshes the stamp; approvals are now stale
    const g = decideNext(s, DEFAULT_CONFIG);
    expect(g).toEqual({ kind: 'review', phase: 'design', round: 3 }); // same round: ratify
  });

  it('escalates when the design budget is exhausted', () => {
    const s = freshState();
    s.lastStage = 'design-review';
    s.designRound = 5;
    s.designBudgetUsed = 5;
    withVerdicts(s, 'design', reject);
    const step = decideNext(s, DEFAULT_CONFIG);
    expect(step.kind).toBe('escalate');
  });

  it('editorial failure consumes the design budget on re-entry', () => {
    const s = freshState();
    s.lastStage = 'editorial-review';
    s.designRound = 3;
    s.designBudgetUsed = 3;
    s.designStamp = 's3';
    withVerdicts(s, 'design', approve);
    withVerdicts(s, 'editorial', reject);
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('proposal-revise');
    s.designBudgetUsed = 5;
    s.lastStage = 'editorial-review';
    expect(decideNext(s, DEFAULT_CONFIG).kind).toBe('escalate');
  });

  it('red implement goes straight to fix and every fix entry consumes budget', () => {
    const s = freshState();
    s.lastStage = 'implement';
    s.testsGreen = false;
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('fix');
    s.lastStage = 'fix';
    s.testsGreen = false;
    s.codeBudgetUsed = 5;
    expect(decideNext(s, DEFAULT_CONFIG).kind).toBe('escalate');
  });

  it('fix with testsNeedUpdate routes through test-update before review', () => {
    const s = freshState();
    s.lastStage = 'fix';
    s.testsNeedUpdate = true;
    s.testsGreen = true;
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('test-update');
    s.lastStage = 'test-update';
    expect(decideNext(s, DEFAULT_CONFIG)).toEqual({ kind: 'review', phase: 'code', round: 1 });
  });

  it('implement with testsNeedUpdate routes through test-update before review', () => {
    const s = freshState();
    s.lastStage = 'implement';
    s.testsGreen = true;
    s.testsNeedUpdate = true;
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('test-update');
    // The runner clears the flag once test-update has run.
    s.lastStage = 'test-update';
    s.testsNeedUpdate = false;
    expect(decideNext(s, DEFAULT_CONFIG)).toEqual({ kind: 'review', phase: 'code', round: 1 });
  });

  it('rejected code round goes to fix until budget runs out', () => {
    const s = freshState();
    s.lastStage = 'code-review';
    s.codeRound = 1;
    s.codeBudgetUsed = 1;
    withVerdicts(s, 'code', reject);
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('fix');
    s.lastStage = 'code-review';
    s.codeRound = 5;
    s.codeBudgetUsed = 5;
    expect(decideNext(s, DEFAULT_CONFIG).kind).toBe('escalate');
  });

  it('forceNext overrides the graph once', () => {
    const s = freshState();
    s.lastStage = 'research';
    s.forceNext = 'doc-sync';
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('doc-sync');
  });

  it('unknown stage has no silent transition', () => {
    const s = freshState();
    s.lastStage = 'bogus';
    expect(() => decideNext(s, DEFAULT_CONFIG)).toThrow(/no transition/);
  });
});

// ---------------------------------------------------------------------------
// Run 010, Test strategy #8: the light flow's transitions, from a state whose
// `flow` key says `light`. Three touch points in the graph — the entry, the
// new `plan` case, and the converged exit — plus the `testsNeedUpdate` guard:
// light has no test engineer, so the flag never routes to `test-update`. The
// backstop string stays the one `decideNext` writes for both flows; the light
// promotion suffix belongs to `finalizeEscalate` and is asserted on the record
// (Test strategy #14), never here.
// ---------------------------------------------------------------------------

describe('decideNext: the light flow', () => {
  const lightConfig = configFor('light', DEFAULT_CONFIG);

  const lightState = (flow: 'light' | 'full' = 'light'): PipelineState => {
    const s = freshState();
    s.flow = flow;
    return s;
  };

  // proves: Test strategy #8 — `''` → plan and plan → implement: the light
  // entry replaces research, and the new `plan` case hands off to the coder.
  it('enters at plan and hands the plan to implement', () => {
    const s = lightState();
    expect(stageOf(decideNext(s, lightConfig))).toBe('plan');
    s.lastStage = 'plan';
    expect(stageOf(decideNext(s, lightConfig))).toBe('implement');
  });

  // proves: Test strategy #8 — the new `plan` case is unreachable in the full
  // flow: the entry there is research, and `plan` has no full-flow transition.
  it('keeps the full flow entering at research and rejecting plan', () => {
    const s = lightState('full');
    expect(stageOf(decideNext(s, DEFAULT_CONFIG))).toBe('research');
    s.lastStage = 'plan';
    expect(() => decideNext(s, DEFAULT_CONFIG)).toThrow(/no transition/);
  });

  // proves: Test strategy #8 — green implement opens code review round 1, and
  // a converged 2-seat panel ends the run: light has no doc-sync stage.
  it('reviews green work and ends at convergence', () => {
    const s = lightState();
    s.lastStage = 'implement';
    s.testsGreen = true;
    const step = decideNext(s, lightConfig);
    expect(step).toEqual({ kind: 'review', phase: 'code', round: 1 });
    enterReview(s, step);
    withVerdicts(s, 'code', approve);
    expect(decideNext(s, lightConfig)).toEqual({ kind: 'done' });
  });

  // proves: Test strategy #8 — a dissent with budget left goes to `fix`, shared
  // with the full flow.
  it('routes a dissent with budget left to fix', () => {
    const s = lightState();
    s.lastStage = 'code-review';
    s.codeRound = 1;
    s.codeBudgetUsed = 1;
    enterReview(s, { kind: 'review', phase: 'code', round: 1 });
    withVerdicts(s, 'code', reject);
    expect(stageOf(decideNext(s, lightConfig))).toBe('fix');
  });

  // proves: Test strategy #8 — the exhausted backstop escalates with the
  // fix-round string exactly as `decideNext` writes it: byte-equal across the
  // flows, and carrying no light suffix (that append is `finalizeEscalate's`).
  it('escalates with the un-suffixed fix-round string', () => {
    const reasonOf = (flow: 'light' | 'full'): string => {
      const s = lightState(flow);
      s.lastStage = 'code-review';
      s.codeRound = 6;
      s.codeBudgetUsed = 3;
      enterReview(s, { kind: 'review', phase: 'code', round: 6 });
      withVerdicts(s, 'code', reject);
      const step = decideNext(s, configFor(flow, DEFAULT_CONFIG));
      expect(step.kind).toBe('escalate');
      return (step as { reason?: string }).reason ?? '';
    };
    const light = reasonOf('light');
    expect(light).toMatch(/backstop/);
    expect(light).not.toContain('light flow');
    expect(light).toBe(reasonOf('full'));
  });

  // proves: Test strategy #8 — `testsNeedUpdate: true` never routes to
  // `test-update` in light: green work goes to review and red work to `fix`,
  // while the full flow's routing to `test-update` is unchanged.
  it('never routes light through test-update', () => {
    const green = lightState();
    green.lastStage = 'implement';
    green.testsGreen = true;
    green.testsNeedUpdate = true;
    expect(decideNext(green, lightConfig)).toEqual({ kind: 'review', phase: 'code', round: 1 });

    const red = lightState();
    red.lastStage = 'fix';
    red.testsGreen = false;
    red.testsNeedUpdate = true;
    expect(stageOf(decideNext(red, lightConfig))).toBe('fix');

    const fromPlan = lightState();
    fromPlan.lastStage = 'plan';
    fromPlan.testsNeedUpdate = true;
    expect(stageOf(decideNext(fromPlan, lightConfig))).toBe('implement');

    for (const lastStage of ['implement', 'fix']) {
      const full = lightState('full');
      full.lastStage = lastStage;
      full.testsGreen = true;
      full.testsNeedUpdate = true;
      expect(stageOf(decideNext(full, DEFAULT_CONFIG))).toBe('test-update');
    }
  });
});
