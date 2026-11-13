import { describe, expect, it } from 'vitest';

import { configFor } from '../../src/pipeline/flows.js';
import {
  cycleOf,
  dissentingSeats,
  planPass,
  recordSeatVerdict,
  reportPathFor,
  reviewSessionsUsed,
  seatDecision,
  seatSummary,
  seatsConverged,
  staleSeats,
} from '../../src/pipeline/panel.js';
import {
  CODE_SPECIALTIES,
  DEFAULT_CONFIG,
  type PipelineConfig,
} from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-002 funnel planner. The unit of review is the pass: a revision returns
// only to dissenting seats, approvals are caught up once at ratification, and
// staleness is read purely from verdict records and revision stamps — the
// runner never parses reviewer reports to pick a panel (late catch only).
// ---------------------------------------------------------------------------

const CFG: PipelineConfig = { ...DEFAULT_CONFIG, maxPassesPerRound: 6 };
/** The five design seats. The code panel is the separate `CODE_SPECIALTIES`:
 * the five `codereview/code-*` agents, where `complexity` is design-only. */
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

// ---------------------------------------------------------------------------
// Run 010, Test strategy #9 and #10: the funnel on the light flow's 2-seat
// code panel. Every rule — full pass, revision to the dissenters only,
// ratification of stale approvals, convergence, the deadlock stop — runs
// unmodified on the shorter seat list, which is the config's
// `reviewSeats.code`, not a constant: `planPass` and the readers take the
// resolved config. `seatSummary` prints the flow's seats, two rows in a light
// board and the standing five in a full one.
// ---------------------------------------------------------------------------

describe('the funnel on the light 2-seat panel', () => {
  const LIGHT = configFor('light', DEFAULT_CONFIG);
  const LIGHT_SEATS = ['correctness', 'testing'];

  const codeState = (partial: Partial<PipelineState> = {}): PipelineState =>
    designState({ lastStage: 'code-review', codeRound: 1, codeStamp: 'h1', ...partial });

  /** The light code ledger as it stands after a pass on `stamp` with these dissenters. */
  const codeSeatsAfter = (dissents: string[], stamp = 'h1'): PipelineState => {
    const s = codeState();
    s.seats = {
      code: Object.fromEntries(
        LIGHT_SEATS.map((sp) => [
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
  };

  const summaryText = (summary: unknown): string =>
    Array.isArray(summary) ? (summary as unknown[]).join('\n') : String(summary);

  // proves: Test strategy #9 — a full pass on the light panel names both seats
  // and only the light seats: the list is the flow's, so a light run never
  // launches the five-seat board.
  it('opens the code phase with exactly the light seats', () => {
    expect(planPass(codeState(), 'code', LIGHT)).toEqual({
      kind: 'pass',
      pass: 'full',
      seats: LIGHT_SEATS,
      cycle: 0,
    });
    expect(planPass(codeState(), 'code', DEFAULT_CONFIG).kind).toBe('pass');
    const fullPlan = planPass(codeState(), 'code', DEFAULT_CONFIG);
    expect(fullPlan.kind === 'pass' && fullPlan.seats).toEqual(CODE_SPECIALTIES);
  });

  // proves: Test strategy #9 — a one-seat dissent returns only that seat, and
  // the reader sees exactly that seat on a 2-seat ledger.
  it('sends the revision to the single dissenter', () => {
    const s = codeSeatsAfter(['testing']);
    s.codeCycle = 1;
    expect(planPass(s, 'code', LIGHT)).toEqual({
      kind: 'pass',
      pass: 'dissent',
      seats: ['testing'],
      cycle: 1,
    });
    expect(dissentingSeats(s, 'code', LIGHT)).toEqual(['testing']);
  });

  // proves: Test strategy #9 — once the dissenter re-approves the new stamp,
  // the stale approver runs one ratification pass on the current code stamp.
  it('runs one ratification pass for the stale approver', () => {
    const s = codeSeatsAfter(['testing']);
    s.codeCycle = 1;
    recordSeatVerdict(s, 'code', 'testing', 'approve', 'h2');
    s.codeStamp = 'h2';
    expect(planPass(s, 'code', LIGHT)).toEqual({
      kind: 'pass',
      pass: 'ratify',
      seats: ['correctness'],
      cycle: 1,
    });
    expect(staleSeats(s, 'code', LIGHT)).toEqual(['correctness']);
  });

  // proves: Test strategy #9 — convergence is "both seats approve the current
  // stamp", on a 2-seat ledger as on a 5-seat one.
  it('converges when both seats approve the current stamp', () => {
    const s = codeSeatsAfter([]);
    expect(seatsConverged(s, 'code', LIGHT)).toBe(true);
    expect(planPass(s, 'code', LIGHT)).toEqual({ kind: 'converged' });
    s.codeStamp = 'h2';
    expect(seatsConverged(s, 'code', LIGHT)).toBe(false);
  });

  // proves: Test strategy #9 — the deadlock stop fires with a 2-seat ledger and
  // `reviseNoChange`, naming the seat still dissenting: the rule is
  // seat-count-agnostic, and `maxRounds: 2` makes it the likelier terminal.
  it('deadlock-stops on a 2-seat ledger naming the dissenter', () => {
    const s = codeSeatsAfter(['testing']);
    s.codeCycle = 1;
    s.reviseNoChange = true;
    const plan = planPass(s, 'code', LIGHT);
    expect(plan.kind).toBe('blocked');
    expect(plan.kind === 'blocked' && plan.reason).toContain('deadlock');
    expect(plan.kind === 'blocked' && plan.reason).toContain('testing');
  });

  // proves: Test strategy #9 (the funnel runs unmodified on the light seat list;
  // correctness r1 finding 2) — an operator grant on the state does not lift the
  // pass cap here: `planPass` blocks at `cfg.maxPassesPerRound` in both flows
  // and names the cap it applied. The grant is spent at the runner's pass-entry
  // cap check, never inside the panel.
  it('blocks at the pass cap with a grant in hand, naming that cap', () => {
    const s = codeSeatsAfter(['testing']);
    s.codeCycle = 3;
    s.extensionSessions = 2;
    const plan = planPass(s, 'code', LIGHT);
    expect(plan.kind).toBe('blocked');
    expect(plan.kind === 'blocked' && plan.reason).toContain('pass backstop');
    expect(plan.kind === 'blocked' && plan.reason).toContain('cap 3 passes per round');
  });

  // proves: Test strategy #10 — `seatSummary` prints the flow's seats: exactly
  // two rows in a light board, and the standing five under `full`.
  it('prints exactly the two light seats and the five full ones', () => {
    const s = codeSeatsAfter(['testing']);
    const light = summaryText(seatSummary(s, 'code', LIGHT));
    expect(light).toContain('correctness');
    expect(light).toContain('testing');
    for (const absent of ['security', 'performance', 'complexity', 'compatibility']) {
      expect(light, `the light board prints no ${absent} seat`).not.toContain(absent);
    }

    const full = designState();
    full.seats = {
      code: Object.fromEntries(
        CODE_SPECIALTIES.map((sp) => [sp, { verdict: 'approve', stamp: 'h1', dissents: 0 }]),
      ),
    };
    const fullText = summaryText(seatSummary(full, 'code', DEFAULT_CONFIG));
    for (const seat of CODE_SPECIALTIES) {
      expect(fullText).toContain(seat);
    }
  });
});
