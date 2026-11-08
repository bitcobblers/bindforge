import { describe, expect, it } from 'vitest';

import { CODE_SPECIALTIES, DEFAULT_CONFIG, DESIGN_SPECIALTIES } from '../src/pipeline/stages.js';
import { type NextStep, type ReviewPhase, decideNext } from '../src/pipeline/states.js';
import type { PipelineState, VerdictReply } from '../src/pipeline/types.js';

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

/** Apply the same round/budget increments the runner's drive loop applies. */
function enterReview(state: PipelineState, step: NextStep): void {
  if (step.kind !== 'review') return;
  if (step.phase === 'design') {
    state.designRound = step.round;
    state.designBudgetUsed += 1;
  } else if (step.phase === 'code') {
    state.codeRound = step.round;
    state.codeBudgetUsed += 1;
  } else {
    state.designRound = step.round; // editorial: numbering only
  }
  state.lastStage = `${step.phase}-review`;
}

function withVerdicts(state: PipelineState, phase: ReviewPhase, reply: VerdictReply): void {
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
    expect(decideNext(s, DEFAULT_CONFIG)).toEqual({ kind: 'review', phase: 'design', round: 2 });
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
