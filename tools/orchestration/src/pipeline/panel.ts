import { CODE_SPECIALTIES, DESIGN_SPECIALTIES } from './stages.js';
import type { PipelineConfig } from './stages.js';
import type { PipelineState, SeatRecord, VerdictReply } from './types.js';

// ---------------------------------------------------------------------------
// The review funnel (ADR-002). A revision goes back only to the seats whose
// blocking findings it was written to answer; seats that already approved see
// it once, at a ratification pass over the cumulative diff since they approved.
//
// Run 002-run2 measured why: 40 design reviewer sessions bought 33 blocking/
// major findings, the three lowest-yield seats consumed ~45% of review time for
// 2 of them, and 0 themes were re-raised across rounds 1-6 while the gate
// denied 8 times out of 8. Re-running the whole panel every round did not
// converge; it grew the document.
//
// Everything here is pure over the checkpoint: the runner owns the seat ledger
// and the revision stamps, so the state graph can plan a pass without git and
// nothing parses reviewer reports for anchors (late catch only).
// ---------------------------------------------------------------------------

/** Phases the funnel applies to. Editorial is a single language pass, not a vote. */
export type FunnelPhase = 'design' | 'code';

export type PassKind = 'full' | 'dissent' | 'ratify';

export type PassPlan =
  | { kind: 'pass'; pass: PassKind; seats: string[]; cycle: number }
  | { kind: 'converged' }
  | { kind: 'blocked'; reason: string };

export function funnelSpecialties(phase: FunnelPhase): readonly string[] {
  return phase === 'code' ? CODE_SPECIALTIES : DESIGN_SPECIALTIES;
}

function seatsOf(state: PipelineState, phase: FunnelPhase): Record<string, SeatRecord> {
  return (phase === 'code' ? state.seats?.code : state.seats?.design) ?? {};
}

/** Passes already run in the current round of this phase. */
export function cycleOf(state: PipelineState, phase: FunnelPhase): number {
  return (phase === 'code' ? state.codeCycle : state.designCycle) ?? 0;
}

/** The revision the current round's verdicts must describe. */
export function stampOf(state: PipelineState, phase: FunnelPhase): string {
  return (phase === 'code' ? state.codeStamp : state.designStamp) ?? '';
}

/** Review sessions this phase has already spent, retries included. */
export function reviewSessionsUsed(state: PipelineState, phase: FunnelPhase): number {
  return state.results.filter((r) => r.stage.startsWith(`${phase}-review:`)).length;
}

/** Seats whose latest verdict requests changes on the revision they were shown. */
export function dissentingSeats(state: PipelineState, phase: FunnelPhase): string[] {
  const seats = seatsOf(state, phase);
  return funnelSpecialties(phase).filter((s) => seats[s]?.verdict === 'request-changes');
}

/**
 * Seats whose verdict does not describe the current revision: approved text
 * that has since been revised. Late catch only — staleness alone never re-runs
 * a seat while a dissent is open.
 */
export function staleSeats(state: PipelineState, phase: FunnelPhase): string[] {
  const seats = seatsOf(state, phase);
  const stamp = stampOf(state, phase);
  return funnelSpecialties(phase).filter(
    (s) => seats[s] === undefined || seats[s]?.stamp !== stamp,
  );
}

/** The phase is settled: every seat has approved, on the current revision. */
export function seatsConverged(state: PipelineState, phase: FunnelPhase): boolean {
  const seats = seatsOf(state, phase);
  const stamp = stampOf(state, phase);
  return funnelSpecialties(phase).every(
    (s) => seats[s]?.verdict === 'approve' && seats[s]?.stamp === stamp,
  );
}

/**
 * The seat-level verdict: a reviewer's `approve` carrying a blocking-severity
 * finding is a dissent, same as the old round gate's
 * "no blocking findings may be open" rule, re-expressed per seat.
 */
export function seatDecision(
  reply: VerdictReply,
  cfg: PipelineConfig,
): 'approve' | 'request-changes' {
  if (reply.verdict !== 'approve') {
    return 'request-changes';
  }
  return reply.findings.some((f) => cfg.blockingSeverities.includes(f.severity))
    ? 'request-changes'
    : 'approve';
}

/** Which panel runs next, or why the phase is over / stuck. */
export function planPass(state: PipelineState, phase: FunnelPhase, cfg: PipelineConfig): PassPlan {
  const seats = seatsOf(state, phase);
  const cycle = cycleOf(state, phase);
  if (funnelSpecialties(phase).every((s) => seats[s] === undefined)) {
    return { kind: 'pass', pass: 'full', seats: [...funnelSpecialties(phase)], cycle: 0 };
  }

  const open = dissentingSeats(state, phase);
  const stale = open.length === 0 ? staleSeats(state, phase) : [];

  if (open.length === 0 && stale.length === 0) {
    return { kind: 'converged' };
  }
  if (cycle >= cfg.maxPassesPerRound) {
    return {
      kind: 'blocked',
      reason:
        `pass backstop: ${String(cycle)} review passes already run in this ${phase} round ` +
        `(cap ${String(cfg.maxPassesPerRound)} passes per round) — the funnel is not converging`,
    };
  }
  if (open.length > 0) {
    if (state.reviseNoChange === true) {
      return {
        kind: 'blocked',
        reason:
          `deadlock: the last revision changed nothing while ${open.join(', ')} ` +
          `${open.length === 1 ? 'dissents' : 'dissent'} — the designer and these seats are at an impasse`,
      };
    }
    return { kind: 'pass', pass: 'dissent', seats: open, cycle };
  }
  return { kind: 'pass', pass: 'ratify', seats: stale, cycle };
}

/**
 * Record what a seat decided, on the revision it was shown. The stamp is the
 * pass's stamp, not "now": a ratifier that approved text three revisions old
 * must not look satisfied with the current one.
 */
export function recordSeatVerdict(
  state: PipelineState,
  phase: FunnelPhase,
  specialty: string,
  verdict: 'approve' | 'request-changes',
  stamp: string,
): void {
  const bySeat = { ...seatsOf(state, phase) };
  const prior = bySeat[specialty];
  bySeat[specialty] = {
    verdict,
    stamp,
    dissents: (prior?.dissents ?? 0) + (verdict === 'request-changes' ? 1 : 0),
  };
  state.seats =
    phase === 'design' ? { ...state.seats, design: bySeat } : { ...state.seats, code: bySeat };
}

/**
 * Report path for one seat-pass. Design/code passes carry the pass number
 * (`-rNcM`); editorial keeps single-pass `-rN` naming (ADR-002 leaves it alone).
 */
export function reportPathFor(
  designDirRel: string,
  phase: FunnelPhase | 'editorial',
  specialty: string,
  round: number,
  cycle: number | undefined,
): string {
  const stamp = cycle === undefined ? `r${String(round)}` : `r${String(round)}c${String(cycle)}`;
  // Editorial reports have always lived under reviews/design/ (the phase is a
  // language pass over the same document, not a separate review surface).
  const dir = phase === 'editorial' ? 'design' : phase;
  return `${designDirRel}/reviews/${dir}/${specialty}-${stamp}.md`;
}

/** One-line seat table for `pipeline status` and the escalation report. */
export function seatSummary(state: PipelineState, phase: FunnelPhase): string {
  const seats = seatsOf(state, phase);
  const stamp = stampOf(state, phase);
  const parts = funnelSpecialties(phase).map((s) => {
    const seat = seats[s];
    const mark =
      seat === undefined ? 'pending' : seat.verdict === 'approve' ? 'approve' : 'DISSENT';
    // `*` marks a verdict that no longer describes the current revision.
    const aged = seat !== undefined && seat.stamp !== stamp ? '*' : '';
    return `${s}:${mark}${aged}`;
  });
  return `${phase} seats: ${parts.join(' ')} (round r${String(
    phase === 'code' ? state.codeRound : state.designRound,
  )} pass c${String(cycleOf(state, phase))})`;
}

/** Does this phase have any seat records at all? (for status/report lines) */
export function hasSeats(state: PipelineState, phase: FunnelPhase): boolean {
  return Object.keys(seatsOf(state, phase)).length > 0;
}
