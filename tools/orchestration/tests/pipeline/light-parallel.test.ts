import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// The lane knob with a 2-seat list (Compatibility matrix, `--max-parallel`
// rows): the ADR-003 semaphore is seat-count-agnostic, so a light pass launches
// its two seats in the handed order at N = 2, lands one commit per pass, and at
// N = 3 the schedule is unchanged — 2 seats never fill 3 lanes.
// ---------------------------------------------------------------------------

const DESIGN_DIR = 'docs/designs/002-x';
const BRIEF = 'docs/prompts/002-x.md';
const LIGHT_SEATS = ['correctness', 'testing'];

const BASE: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

function tmpRepo(): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-light-parallel-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
  writeFileSync(join(dir, BRIEF), '---\nflow: light\n---\n\nBuild the thing.\n', 'utf8');
  const state: PipelineState = {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'brief', path: BRIEF },
    status: 'escalated',
    lastStage: '',
    flow: 'light',
    designRound: 0,
    codeRound: 0,
    designBudgetUsed: 0,
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
  mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
  writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), `${JSON.stringify(state, null, 2)}\n`);
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** Both seats approve; each writes the report its prompt names. */
function approvingStub(dir: string) {
  const sessionsCreated: string[] = [];
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  let clock = 0;
  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        sessionsCreated.push(key);
        const id = `ses_${String(sessionsCreated.length)}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string; text?: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        if (s.key === 'plan') {
          const plan = '# Plan\n\n## Problem\n\nx\n\n## Test plan\n\nthe gate\n';
          writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), plan, 'utf8');
        }
        if (s.key === 'implement') {
          mkdirSync(join(dir, 'src'), { recursive: true });
          writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n', 'utf8');
        }
        if (s.key.includes('-review:')) {
          const text = input.text ?? '';
          const found = /docs\/designs\/[^\s`]+\/reviews\/code\/[\w.-]+\.md/.exec(text);
          if (found !== null) {
            const path = join(dir, found[0]);
            mkdirSync(join(path, '..'), { recursive: true });
            writeFileSync(path, `report from ${s.key}\n`, 'utf8');
          }
        }
      },
      remove: async () => undefined,
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { key: '', prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const text = s.key.includes('-review:')
          ? '{"verdict": "approve", "findings": []}'
          : s.key === 'implement'
            ? '{"testsNeedUpdate": false, "commitMessage": "feat: implement it"}'
            : '{"summary": "ok", "commitMessage": "docs: plan"}';
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          clock += 1;
          messages.push({
            id: `m_a${String(clock)}`,
            time: { created: clock },
            type: 'assistant',
            content: [{ type: 'text', text }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(clock)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, sessionsCreated };
}

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

async function runAt(maxParallelReviews: number): Promise<{ dir: string; saved: PipelineState }> {
  const dir = tmpRepo();
  const { client } = approvingStub(dir);
  const options: ResumeOptions = {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: { ...BASE, maxParallelReviews },
    client,
  };

  const status = await resumePipeline(options);
  expect(status).toBe('done');
  return {
    dir,
    saved: JSON.parse(readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8')),
  };
}

describe('the lane knob with 2 seats', () => {
  // proves: Test strategy #20 — at `--max-parallel 2` the two seats launch in
  // the handed order and the pass lands exactly one commit.
  it('launches both seats in handed order and commits the pass once', async () => {
    const { dir, saved } = await runAt(2);

    const stages = saved.results.map((r) => r.stage);
    const reviewStages = stages.filter((s) => s.includes('code-review'));
    expect(reviewStages).toEqual(LIGHT_SEATS.map((seat) => `code-review:${seat}`));

    const subjects = gitAt(dir, ['log', '--format=%s']).trim().split('\n');
    expect(subjects.filter((s) => s.includes('code-review'))).toHaveLength(1);

    for (const seat of LIGHT_SEATS) {
      expect(existsSync(join(dir, DESIGN_DIR, 'reviews', 'code', `${seat}-r1c0.md`))).toBe(true);
      expect(saved.seats?.code?.[seat]?.verdict).toBe('approve');
    }
    expect(Object.keys(saved.seats?.code ?? {})).toEqual(LIGHT_SEATS);
  });

  // proves: Test strategy #20 — the schedule is unchanged at `--max-parallel 3`:
  // two seats never fill three lanes, so the same fixture produces the same
  // result order, the same seat ledger and the same commits.
  it('matches the N = 3 run result for result', async () => {
    const two = await runAt(2);
    const three = await runAt(3);

    const stages = (saved: PipelineState) => saved.results.map((r) => r.stage);
    const seats = (saved: PipelineState) => Object.keys(saved.seats?.code ?? {});
    expect(stages(three.saved)).toEqual(stages(two.saved));
    expect(seats(three.saved)).toEqual(seats(two.saved));
    expect(gitAt(three.dir, ['log', '--format=%s'])).toBe(gitAt(two.dir, ['log', '--format=%s']));
  });

  // proves: Test strategy #20 (the pass's ledger tail is re-sorted into handed
  // order) — the record a run leaves cannot depend on the lane width: serial
  // launches finish in a different order than a 2-lane fan-out, and the
  // checkpoint's result order, the seat ledger and the commits come out the same.
  it('matches the N = 1 run result for result', async () => {
    const one = await runAt(1);
    const two = await runAt(2);

    const stages = (saved: PipelineState) => saved.results.map((r) => r.stage);
    const seats = (saved: PipelineState) => Object.keys(saved.seats?.code ?? {});
    expect(seats(one.saved)).toEqual(LIGHT_SEATS);
    expect(stages(one.saved)).toEqual(stages(two.saved));
    expect(seats(one.saved)).toEqual(seats(two.saved));
    expect(gitAt(one.dir, ['log', '--format=%s'])).toBe(gitAt(two.dir, ['log', '--format=%s']));
  });
});
