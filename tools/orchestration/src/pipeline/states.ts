import { allReviewsApproved } from './gates.js';
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

const gateNameFor: Record<ReviewPhase, string> = {
  design: 'allDesignReviewsApproved',
  editorial: 'allEditorialReviewsApproved',
  code: 'allCodeReviewsApproved',
};

export function gateName(phase: ReviewPhase): string {
  return gateNameFor[phase];
}

export function decideNext(state: PipelineState, cfg: PipelineConfig): NextStep {
  if (state.forceNext) {
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
      const g = allReviewsApproved(state, 'design', state.designRound, DESIGN_SPECIALTIES, cfg);
      if (g.proceed) return stage('proposal-final');
      return state.designBudgetUsed < cfg.maxRounds
        ? stage('proposal-revise')
        : escalate(`design review: ${g.reason}`);
    }
    case 'proposal-revise':
      return review('design', state.designRound + 1);
    case 'proposal-final':
      // The editor may have shifted meaning while fixing language: the same
      // five reviewers run one confirmation round over the edit diff.
      return review('editorial', state.designRound + 1);
    case 'editorial-review': {
      const g = allReviewsApproved(state, 'editorial', state.designRound, DESIGN_SPECIALTIES, cfg);
      if (g.proceed) return stage('test-authoring');
      return state.designBudgetUsed < cfg.maxRounds
        ? stage('proposal-revise')
        : escalate(`editorial review: ${g.reason}`);
    }

    case 'test-authoring':
      return stage('implement');
    case 'implement':
    case 'test-update':
      if (state.testsGreen) return review('code', state.codeRound + 1);
      return state.codeBudgetUsed < cfg.maxRounds
        ? stage('fix')
        : escalate('tests not green at the code-review budget limit');
    case 'fix':
      if (state.testsNeedUpdate) return stage('test-update');
      if (state.testsGreen) return review('code', state.codeRound + 1);
      return state.codeBudgetUsed < cfg.maxRounds
        ? stage('fix')
        : escalate('tests not green at the code-review budget limit');

    case 'code-review': {
      const g = allReviewsApproved(state, 'code', state.codeRound, CODE_SPECIALTIES, cfg);
      if (g.proceed) return stage('doc-sync');
      return state.codeBudgetUsed < cfg.maxRounds
        ? stage('fix')
        : escalate(`code review: ${g.reason}`);
    }
    case 'doc-sync':
      return { kind: 'done' };

    default:
      throw new Error(`no transition defined from stage: ${state.lastStage}`);
  }
}
