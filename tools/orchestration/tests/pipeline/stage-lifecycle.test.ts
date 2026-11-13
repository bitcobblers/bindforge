import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type RunOptions, executeStage } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// A timed-out stage session is never cancelled: the runner recorded the
// failure and immediately started attempt 2 against the same files while the
// abandoned agent was still editing them. ADR-003 fans a review pass out, so
// the race multiplies with N.
// ---------------------------------------------------------------------------

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 5,
  stageTimeoutMs: 40,
  requestTimeoutMs: 1_000,
};

function state(): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'running',
    lastStage: 'proposal',
    designRound: 1,
    codeRound: 0,
    designBudgetUsed: 1,
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

/** Session that keeps working (tool turns, no idle marker) until it is cut off. */
function busyClient() {
  const calls: string[] = [];
  let turn = 0;
  const client = {
    session: {
      create: async () => {
        calls.push('create');
        return { id: 'ses_busy' };
      },
      prompt: async () => {
        calls.push('prompt');
      },
      remove: async (input: { sessionID: string }) => {
        calls.push(`remove:${input.sessionID}`);
      },
      // Test strategy #9: the descendant walk's enumeration endpoint. This
      // fixture creates no child sessions, so an empty page is the truth.
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async () => {
        turn += 1;
        const message = {
          id: `m_${String(turn)}`,
          time: { created: turn },
          type: 'assistant',
          content: [{ type: 'tool' }],
        } as unknown as SessionMessageInfo;
        return { data: [message], cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, calls };
}

/** The runner shells out to git for every stage, so the fixture is a repo. */
function tmpRepo(): string {
  const dir = mkTmpDir('bindforge-stage-lifecycle-');
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'adrs', '002-x.md'), 'x\n');
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

function options(client: OpenCodeClient): RunOptions {
  return {
    repoDir: tmpRepo(),
    sourcePath: 'docs/adrs/002-x.md',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    client,
  };
}

describe('executeStage timeout', () => {
  it('removes the abandoned session before the runner moves on', async () => {
    const { client, calls } = busyClient();
    const s = state();

    const outcome = await executeStage(
      options(client),
      s,
      {
        stage: 'design-review',
        stageKey: 'design-review:correctness',
        attempt: 1,
        round: 1,
        specialty: 'correctness',
        vars: {
          designDir: 'docs/designs/002-x',
          sourcePath: 'docs/adrs/002-x.md',
          round: 1,
          specialty: 'correctness',
          scope: 'the whole document',
          passNote: 'full-panel pass c0',
          prevFindings: '(none — test fixture)',
          reportPath: 'docs/designs/002-x/reviews/design/correctness-r1c0.md',
          reviewFiles: '',
        },
      },
      FAST,
    );

    expect(outcome).toEqual({ ok: false, failure: 'stage-timeout', sessionID: 'ses_busy' });
    expect(calls).toContain('remove:ses_busy');
    // The stage record still exists so `pipeline status` can see the attempt.
    expect(s.results).toHaveLength(1);
    expect(s.results[0].failure).toBe('stage-timeout');
    expect(typeof s.results[0].durationMs).toBe('number');
  });

  it('reports a removal failure without losing the timeout verdict', async () => {
    const { client, calls } = busyClient();
    client.session.remove = async () => {
      calls.push('remove-failed');
      throw new Error('server gone');
    };

    const outcome = await executeStage(
      options(client),
      state(),
      {
        stage: 'design-review',
        stageKey: 'design-review:security',
        attempt: 1,
        round: 1,
        specialty: 'security',
        vars: {
          designDir: 'docs/designs/002-x',
          sourcePath: 'docs/adrs/002-x.md',
          round: 1,
          specialty: 'security',
          scope: 'the whole document',
          passNote: 'full-panel pass c0',
          prevFindings: '(none — test fixture)',
          reportPath: 'docs/designs/002-x/reviews/design/security-r1c0.md',
          reviewFiles: '',
        },
      },
      FAST,
    );

    expect(outcome).toEqual({ ok: false, failure: 'stage-timeout', sessionID: 'ses_busy' });
  });
});
