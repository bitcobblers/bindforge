import { allReviewsApproved } from './gates.js';
import { dissentingSeats, planPass } from './panel.js';
import { CODE_SPECIALTIES, DESIGN_SPECIALTIES } from './stages.js';
import type { PipelineConfig } from './stages.js';
import type { PipelineState } from './types.js';

// ---------------------------------------------------------------------------
// The state graph, declaratively. Transitions reference gates by name and
// read only the persisted state — the runner supplies `testsGreen` by
// executing `npm run check` after each mutating stage.
// ---------------------------------------------------------------------------

export type ReviewPhase = 'design' | 'editorial' | 'code';

export type NextStep =
  | { kind: 'stage'; stage: string }
  | { kind: 'review'; phase: ReviewPhase; round: number }
  | { kind: 'done' }
  | { kind: 'escalate'; reason: string };

const stage = (name: string): NextStep => ({ kind: 'stage', stage: name });
const review = (phase: ReviewPhase, round: number): NextStep => ({ kind: 'review', phase, round });
const escalate = (reason: string): NextStep => ({ kind: 'escalate', reason });

export function specialtiesFor(phase: ReviewPhase): readonly string[] {
  return phase === 'code' ? CODE_SPECIALTIES : DESIGN_SPECIALTIES;
}

/**
 * The pass gate in the gate log is named after the stage whose seats it gates,
 * so a `design-review` entry reads next to that pass's commit and transcripts.
 * A label only: which gate runs, and what it decides, is unchanged.
 */
const gateNameFor: Record<ReviewPhase, string> = {
  design: 'design-review',
  editorial: 'editorial-review',
  code: 'code-review',
};

export function gateName(phase: ReviewPhase): string {
  return gateNameFor[phase];
}

/** `design-review` -> 'design'; anything else -> undefined. */
export function reviewPhaseOf(name: string): ReviewPhase | undefined {
  if (name === 'design-review') return 'design';
  if (name === 'editorial-review') return 'editorial';
  if (name === 'code-review') return 'code';
  return undefined;
}

/**
 * Rounds used in the phase a review round belongs to. Editorial rounds
 * continue the design round sequence but do not consume the design budget on
 * entry (ADR-001 charges the budget when a failed editorial round re-enters
 * revise), so the cap check skips them.
 */
/**
 * Is another round in this phase legal? `maxRounds` is the cap; each
 * recorded operator extension buys one more revise -> review cycle. The
 * extension is *consumed* at review-round entry in the drive loop — here it
 * only makes the transition legal, so an extension that is never spent on a
 * review round stays granted.
 */
function budgetLeft(state: PipelineState, used: number, cfg: PipelineConfig): boolean {
  return used < cfg.maxRounds || (state.extensionRounds ?? 0) > 0;
}

export function reviewBudgetUsed(state: PipelineState, phase: ReviewPhase): number {
  return phase === 'code'
    ? state.codeBudgetUsed
    : phase === 'editorial'
      ? 0
      : state.designBudgetUsed;
}

/**
 * Green tests after implement/fix: dissenter seats re-review inside the
 * current code round; with no seat ledger yet a new round starts.
 */
function codeReviewEntry(state: PipelineState): NextStep {
  const open = dissentingSeats(state, 'code');
  return review('code', open.length > 0 ? state.codeRound : state.codeRound + 1);
}

export function decideNext(state: PipelineState, cfg: PipelineConfig): NextStep {
  if (state.forceNext) {
    // A whole review round is the obvious thing to force after a round dies to
    // a crashed reviewer, so it is a first-class target, not a stage name.
    const phase = reviewPhaseOf(state.forceNext);
    if (phase) {
      return review(phase, (phase === 'code' ? state.codeRound : state.designRound) + 1);
    }
    return stage(state.forceNext);
  }

  switch (state.lastStage) {
    case '':
      return stage('research');
    case 'research':
      return stage('proposal');
    case 'proposal':
      return review('design', state.designRound + 1);

    case 'design-review': {
      // ADR-002: a round is a sequence of passes over revision stamps, not one
      // five-seat sweep. The phase advances when every seat approves the
      // current stamp; a denial names the seats that must see the next revision.
      const plan = planPass(state, 'design', cfg);
      if (plan.kind === 'converged') return stage('proposal-final');
      if (plan.kind === 'blocked') return escalate(`design review: ${plan.reason}`);
      if (plan.pass === 'ratify') {
        // Approvals that predate the current revision: no new mutation, no
        // revise round spent, so the round backstop does not gate re-entry.
        return review('design', state.designRound);
      }
      // Dissent (or a degenerate round that ended with no verdict recorded at
      // all — a forced or crashed pass): only a revision can move it, and the
      // revise loop is what `maxRounds` backstops.
      return budgetLeft(state, state.designBudgetUsed, cfg)
        ? stage('proposal-revise')
        : escalate(
            `design review: ${
              plan.pass === 'dissent'
                ? `${plan.seats.join(', ')} ${plan.seats.length === 1 ? 'dissents' : 'dissent'}`
                : 'no verdict recorded for any seat'
            } and the revise-round backstop is exhausted ` +
              `(${String(state.designBudgetUsed)}/${String(cfg.maxRounds)})`,
          );
    }
    case 'proposal-revise': {
      const open = dissentingSeats(state, 'design');
      if (open.length > 0) {
        // Deadlock stop (run 2's failure mode in one pass instead of eight
        // rounds): objections that revisions do not move.
        if (state.reviseNoChange === true) {
          return escalate(
            `design review: deadlock — the revision changed nothing while ` +
              `${open.join(', ')} ${open.length === 1 ? 'still dissents' : 'still dissent'}`,
          );
        }
        // The revision goes back to the dissenters only, inside this round.
        return review('design', state.designRound);
      }
      return review('design', state.designRound + 1);
    }
    case 'proposal-final':
      // The editor may have shifted meaning while fixing language: the same
      // five reviewers run one confirmation round over the edit diff.
      return review('editorial', state.designRound + 1);
    case 'editorial-review': {
      const g = allReviewsApproved(state, 'editorial', state.designRound, DESIGN_SPECIALTIES, cfg);
      if (g.proceed) return stage('test-authoring');
      return budgetLeft(state, state.designBudgetUsed, cfg)
        ? stage('proposal-revise')
        : escalate(`editorial review: ${g.reason}`);
    }

    case 'test-authoring':
      return stage('implement');
    case 'implement':
    case 'test-update':
      // ADR-001's test-ownership rule applies to implementation diffs too: if
      // the coder's test-path edits were reverted (or its reply asked for it),
      // the test engineer catches up before the code review spends five
      // sessions on a diff whose tests are known stale. The runner clears the
      // flag once test-update has run, and never re-enters it from here.
      if (state.lastStage === 'implement' && state.testsNeedUpdate) return stage('test-update');
      if (state.testsGreen) return codeReviewEntry(state);
      return budgetLeft(state, state.codeBudgetUsed, cfg)
        ? stage('fix')
        : escalate('tests not green at the code-review budget limit');
    case 'fix':
      if (state.testsNeedUpdate) return stage('test-update');
      if (state.testsGreen) return codeReviewEntry(state);
      return budgetLeft(state, state.codeBudgetUsed, cfg)
        ? stage('fix')
        : escalate('tests not green at the code-review budget limit');

    case 'code-review': {
      const plan = planPass(state, 'code', cfg);
      if (plan.kind === 'converged') return stage('doc-sync');
      if (plan.kind === 'blocked') return escalate(`code review: ${plan.reason}`);
      if (plan.pass === 'ratify') return review('code', state.codeRound);
      return budgetLeft(state, state.codeBudgetUsed, cfg)
        ? stage('fix')
        : escalate(
            `code review: ${
              plan.pass === 'dissent'
                ? `${plan.seats.join(', ')} ${plan.seats.length === 1 ? 'dissents' : 'dissent'}`
                : 'no verdict recorded for any seat'
            } and the fix-round backstop is exhausted ` +
              `(${String(state.codeBudgetUsed)}/${String(cfg.maxRounds)})`,
          );
    }
    case 'doc-sync':
      return { kind: 'done' };

    default:
      throw new Error(`no transition defined from stage: ${state.lastStage}`);
  }
}
