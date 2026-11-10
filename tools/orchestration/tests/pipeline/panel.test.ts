import { describe, expect, it } from 'vitest';

import {
  cycleOf,
  dissentingSeats,
  planPass,
  recordSeatVerdict,
  reportPathFor,
  reviewSessionsUsed,
  seatDecision,
  seatsConverged,
  staleSeats,
} from '../../src/pipeline/panel.js';
import { DEFAULT_CONFIG, type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-002 funnel planner. The unit of review is the pass: a revision returns
// only to dissenting seats, approvals are caught up once at ratification, and
// staleness is read purely from verdict records and revision stamps — the
// runner never parses reviewer reports to pick a panel (late catch only).
// ---------------------------------------------------------------------------

const CFG: PipelineConfig = { ...DEFAULT_CONFIG, maxPassesPerRound: 6 };
const SEATS = ['correctness', 'compatibility', 'security', 'performance', 'complexity'];

function designState(partial: Partial<PipelineState> = {}): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'running',
    lastStage: 'design-review',
    designRound: 1,
    codeRound: 0,
    designBudgetUsed: 1,
    codeBudgetUsed: 0,
    editorialDone: false,
    testsGreen: false,
    testsNeedUpdate: false,
    stageFailures: {},
    forceNext: null,
    designStamp: 's1',
    results: [],
    gateLog: [],
    startedAt: 0,
    updatedAt: 0,
    ...partial,
  };
}

/** Seats as they stand after `c0` on stamp s1 with these dissenters. */
function seatsAfter(dissents: string[], stamp = 's1'): PipelineState {
  const s = designState();
  s.seats = {
    design: Object.fromEntries(
      SEATS.map((sp) => [
        sp,
        {
          verdict: dissents.includes(sp) ? 'request-changes' : 'approve',
          stamp,
          dissents: dissents.includes(sp) ? 1 : 0,
        },
      ]),
    ),
  };
  return s;
}

describe('planPass panels', () => {
  it('opens a fresh phase with the full five-seat panel at c0', () => {
    expect(planPass(designState(), 'design', CFG)).toEqual({
      kind: 'pass',
      pass: 'full',
      seats: SEATS,
      cycle: 0,
    });
  });

  it('sends a revision back to the dissenters only', () => {
    const s = seatsAfter(['security']);
    s.designCycle = 1; // c0 ran
    expect(planPass(s, 'design', CFG)).toEqual({
      kind: 'pass',
      pass: 'dissent',
      seats: ['security'],
      cycle: 1,
    });
  });

  it('catches approvals up in one ratification pass once dissent is cleared', () => {
    const s = seatsAfter(['security']);
    s.designCycle = 1;
    // security re-approved on the new revision s2; the rest still describe s1.
    recordSeatVerdict(s, 'design', 'security', 'approve', 's2');
    s.designStamp = 's2';
    expect(planPass(s, 'design', CFG)).toEqual({
      kind: 'pass',
      pass: 'ratify',
      seats: ['correctness', 'compatibility', 'performance', 'complexity'],
      cycle: 1,
    });
  });

  it('a ratifier that dissents rejoins the dissenter set', () => {
    const s = seatsAfter(['security']);
    recordSeatVerdict(s, 'design', 'security', 'approve', 's2');
    s.designStamp = 's2';
    // The ratification ran and complexity rejected the revised text.
    recordSeatVerdict(s, 'design', 'complexity', 'request-changes', 's2');
    s.designCycle = 2;
    expect(planPass(s, 'design', CFG)).toEqual({
      kind: 'pass',
      pass: 'dissent',
      seats: ['complexity'],
      cycle: 2,
    });
    // The other approvers wait: staleness alone never re-runs a seat while a
    // dissent is open (late catch only).
    expect(staleSeats(s, 'design')).toEqual(['correctness', 'compatibility', 'performance']);
  });

  it('converges only when every seat approves the current stamp', () => {
    const s = seatsAfter([]);
    expect(seatsConverged(s, 'design')).toBe(true);
    expect(planPass(s, 'design', CFG)).toEqual({ kind: 'converged' });
    s.designStamp = 's2';
    expect(seatsConverged(s, 'design')).toBe(false);
  });

  it('runs the backstop before a round can eat the phase budget', () => {
    const s = seatsAfter(['security', 'correctness']);
    s.designCycle = CFG.maxPassesPerRound;
    const plan = planPass(s, 'design', CFG);
    expect(plan.kind).toBe('blocked');
    expect(plan.kind === 'blocked' && plan.reason).toContain('pass backstop');
  });

  it('deadlock-stops when the designer replies changed:false and a seat still dissents', () => {
    const s = seatsAfter(['performance']);
    s.designCycle = 1;
    s.reviseNoChange = true;
    const plan = planPass(s, 'design', CFG);
    expect(plan.kind === 'blocked' && plan.reason).toContain('deadlock');
    expect(plan.kind === 'blocked' && plan.reason).toContain('performance');
  });
});

describe('seat ledger bookkeeping', () => {
  it('records verdicts pinned to the revision shown, counting dissents', () => {
    const s = designState();
    recordSeatVerdict(s, 'design', 'security', 'request-changes', 's1');
    recordSeatVerdict(s, 'design', 'security', 'request-changes', 's2');
    expect(s.seats?.design?.security).toEqual({
      verdict: 'request-changes',
      stamp: 's2',
      dissents: 2,
    });
  });

  it('counts review sessions, retries included', () => {
    const s = designState();
    const r = (stage: string) => ({
      stage,
      round: 1,
      sessionID: 'x',
      agent: 'a',
      at: 0,
      reply: {},
      transcript: 't',
    });
    s.results = [
      r('design-review:security'),
      r('design-review:security'), // retry
      r('proposal-revise'),
      r('code-review:testing'),
    ];
    expect(reviewSessionsUsed(s, 'design')).toBe(2);
    expect(reviewSessionsUsed(s, 'code')).toBe(1);
  });

  it('treats an approve carrying a blocking finding as a dissent', () => {
    expect(
      seatDecision(
        { verdict: 'approve', findings: [{ severity: 'major', location: 'x', problem: 'p' }] },
        CFG,
      ),
    ).toBe('request-changes');
    expect(
      seatDecision(
        { verdict: 'approve', findings: [{ severity: 'minor', location: 'x', problem: 'p' }] },
        CFG,
      ),
    ).toBe('approve');
  });

  it('names funnel reports by round and pass, editorial by round only', () => {
    expect(reportPathFor('docs/designs/002-x', 'design', 'security', 1, 2)).toBe(
      'docs/designs/002-x/reviews/design/security-r1c2.md',
    );
    expect(reportPathFor('docs/designs/002-x', 'editorial', 'security', 2, undefined)).toBe(
      'docs/designs/002-x/reviews/design/security-r2.md',
    );
  });

  it('reads cycles from the phase counter', () => {
    expect(cycleOf(designState(), 'design')).toBe(0);
    expect(cycleOf(designState({ designCycle: 3 }), 'design')).toBe(3);
  });

  it('keeps the code phase ledger separate from design', () => {
    const s = designState();
    recordSeatVerdict(s, 'design', 'security', 'approve', 's1');
    recordSeatVerdict(s, 'code', 'testing', 'request-changes', 'h1');
    expect(dissentingSeats(s, 'design')).toEqual([]);
    expect(dissentingSeats(s, 'code')).toEqual(['testing']);
  });
});
