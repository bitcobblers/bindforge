import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import {
  DEFAULT_CONFIG,
  type PipelineConfig,
  assertForceableStage,
} from '../../src/pipeline/stages.js';
import { decideNext } from '../../src/pipeline/states.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// Run 1 spent nine design review rounds against a cap of five. The cap was
// only checked by the round-end gate, so the escalation hatch (`--stage
// proposal-revise`) re-entered review with no cap check at entry: the gate log
// shows same-second `gate deny` + `escalate` pairs for r5 through r9, and the
// extra rounds are indistinguishable from model-driven ones afterwards.
// ---------------------------------------------------------------------------

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 4_000,
  requestTimeoutMs: 1_000,
};

/** A run that has already spent the whole design budget on round 5. */
function exhaustedState(): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'escalated',
    lastStage: 'proposal-revise',
    designRound: 5,
    codeRound: 0,
    designBudgetUsed: 5,
    codeBudgetUsed: 0,
    editorialDone: false,
    testsGreen: false,
    testsNeedUpdate: false,
    stageFailures: {},
    stageCommits: [],
    forceNext: null,
    results: [],
    gateLog: [],
    startedAt: 0,
    updatedAt: 0,
  };
}

function tmpRepo(state: PipelineState): { dir: string; options: (extra: object) => ResumeOptions } {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-budget-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  const designDir = join(dir, 'docs', 'designs', '002-x');
  mkdirSync(designDir, { recursive: true });
  writeFileSync(join(designDir, '02-proposal.md'), '# proposal\n\nbody\n');
  writeFileSync(join(designDir, '.pipeline.json'), `${JSON.stringify(state, null, 2)}\n`);
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return {
    dir,
    options: (extra: object) => ({
      repoDir: dir,
      slug: '002-x',
      promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
      client: requestCountingClient(),
      cfg: FAST,
      ...extra,
    }),
  };
}

/**
 * Stub server where every reviewer answers with a request-changes verdict.
 * Replies become visible one poll after the prompt that asked for them, the
 * way a real turn lands: a stub that answers instantly makes the runner's
 * baseline check look like an empty reply.
 */
function requestCountingClient() {
  const calls: string[] = [];
  const sessions = new Map<string, { prompts: number; lists: number; revise: boolean }>();
  let clock = 0;
  const verdictText = JSON.stringify({
    verdict: 'request-changes',
    findings: [{ severity: 'major', location: 'Rollout', problem: 'no rollback path' }],
  });
  const reviseText = JSON.stringify({ changed: true, summary: 'addressed the findings' });
  const client = {
    session: {
      create: async (input: { title?: string }) => {
        calls.push(`create:${input.title ?? ''}`);
        const id = `ses_${String(calls.length)}`;
        sessions.set(id, {
          prompts: 0,
          lists: 0,
          revise: (input.title ?? '').includes('proposal-revise'),
        });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (s) s.prompts += 1;
      },
      remove: async () => undefined,
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { prompts: 0, lists: 0, revise: false };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          clock += 1;
          messages.push({
            id: `m_a${String(i)}`,
            time: { created: clock },
            type: 'assistant',
            content: [{ type: 'text', text: s.revise ? reviseText : verdictText }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(i)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, calls };
}

describe('assertForceableStage', () => {
  it('accepts single stages and whole review rounds', () => {
    for (const name of ['proposal-revise', 'implement', 'design-review', 'code-review']) {
      expect(() => assertForceableStage(name)).not.toThrow();
    }
  });

  it('rejects a typo with the list of valid targets', () => {
    expect(() => assertForceableStage('desing-review')).toThrow(/unknown --stage/);
    expect(() => assertForceableStage('desing-review')).toThrow(/proposal-revise/);
  });
});

describe('decideNext with forceNext', () => {
  it('forces a whole review round, not just a single stage', () => {
    const s = exhaustedState();
    s.forceNext = 'design-review';
    s.designRound = 2;
    expect(decideNext(s, DEFAULT_CONFIG)).toEqual({ kind: 'review', phase: 'design', round: 3 });
    s.forceNext = 'code-review';
    s.codeRound = 1;
    expect(decideNext(s, DEFAULT_CONFIG)).toEqual({ kind: 'review', phase: 'code', round: 2 });
  });
});

describe('review budget at round entry', () => {
  it('escalates a natural round-end denial when no extension was granted', async () => {
    const state = exhaustedState();
    // A real escalation: round 5 ran and the gate denied, so the graph's own
    // budget check at the design-review transition is what stops the run.
    state.lastStage = 'design-review';
    const { options } = tmpRepo(state);
    const { client, calls } = requestCountingClient();

    const status = await resumePipeline({ ...options({}), client });

    expect(status).toBe('escalated');
    expect(calls).toEqual([]);
  });

  it('re-opens the revise transition under a grant; only the round spends it', async () => {
    const state = exhaustedState();
    state.lastStage = 'design-review';
    const { dir, options } = tmpRepo(state);
    const { client, calls } = requestCountingClient();

    const status = await resumePipeline({ ...options({ extendBudget: 1 }), client });

    // One revise stage plus the five reviewers of the granted round 6.
    expect(calls.filter((c) => c.startsWith('create:'))).toHaveLength(6);
    expect(status).toBe('escalated');

    const saved = JSON.parse(
      readFileSync(join(dir, 'docs', 'designs', '002-x', '.pipeline.json'), 'utf8'),
    ) as PipelineState;
    expect(saved.designRound).toBe(6);
    expect(saved.designBudgetUsed).toBe(6);
    expect(saved.extensionRounds).toBe(0);
    const extensions = saved.gateLog.filter((g) => g.gate === 'operatorExtension');
    const reasons = extensions.map((g) => g.reason).join('\n');
    expect(reasons).toContain(
      'extended the review budget by 1 review session(s) over the caps of 40/20',
    );
    expect(reasons).toContain('over-cap design round entered');
  });

  it('refuses an over-cap round that the operator did not grant', async () => {
    const { dir, options } = tmpRepo(exhaustedState());
    const { client, calls } = requestCountingClient();

    const status = await resumePipeline({ ...options({ forceStage: 'design-review' }), client });

    expect(status).toBe('escalated');
    // No review sessions were spent: the cap is checked before the round.
    expect(calls).toEqual([]);
    const report = readFileSync(join(dir, 'docs', 'designs', '002-x', 'ESCALATION.md'), 'utf8');
    expect(report).toContain('revise-round backstop reached (5/5 rounds)');
    expect(report).toContain('--extend-budget');
  });

  it('runs an over-cap round the operator granted, and logs the consent', async () => {
    const { dir, options } = tmpRepo(exhaustedState());
    const { client, calls } = requestCountingClient();

    const status = await resumePipeline({
      ...options({ forceStage: 'design-review', extendBudget: 1 }),
      client,
    });

    // The five reviewers of the granted round 6, and nothing else: the round
    // denied, and the graph will not re-enter review over the cap.
    expect(calls.filter((c) => c.startsWith('create:'))).toHaveLength(5);
    expect(status).toBe('escalated');

    const saved = JSON.parse(
      readFileSync(join(dir, 'docs', 'designs', '002-x', '.pipeline.json'), 'utf8'),
    ) as PipelineState;
    expect(saved.designRound).toBe(6);
    expect(saved.designBudgetUsed).toBe(6);
    expect(saved.extensionRounds).toBe(0);
    const forces = saved.gateLog.filter((g) => g.gate === 'operatorForce');
    expect(forces).toHaveLength(1);
    expect(forces[0].reason).toContain('design-review');
    const extensions = saved.gateLog.filter((g) => g.gate === 'operatorExtension');
    // One entry recording the grant, one recording the round it paid for.
    expect(extensions.map((g) => g.reason).join('\n')).toContain('over the caps of 40/20');
    expect(extensions.map((g) => g.reason).join('\n')).toContain('over-cap design round entered');
    expect(existsSync(join(dir, '.pipeline.lock'))).toBe(false);
  });
});
