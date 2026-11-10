import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import {
  type RunOptions,
  executeStage,
  replyShapeExample,
  retryInSameSession,
} from '../src/pipeline/runner.js';
import { type PipelineConfig, stageDef, stageReplySchema } from '../src/pipeline/stages.js';
import type { PipelineState } from '../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// Run 1's reply contract failed 5 times in 43 stage attempts, and each failure
// threw away a session that had already done the work: round 4 died on two
// correctness replies that were valid JSON the runner had mis-read, and the
// final nudge re-sent the JSON Schema, which the model answered by echoing a
// schema. Nudges also arrived eight deep in 69 seconds at sessions that were
// still working.
// ---------------------------------------------------------------------------

function reviewConfig(overrides: Partial<PipelineConfig> = {}): PipelineConfig {
  return {
    maxRounds: 5,
    blockingSeverities: ['blocker', 'major'],
    pollIntervalMs: 1,
    stageTimeoutMs: 3_000,
    requestTimeoutMs: 500,
    minNudgeIntervalMs: 0,
    ...overrides,
  };
}

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

function tmpRepo(): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-reply-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
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
    promptsDir: fileURLToPath(new URL('../prompts/', import.meta.url)),
    client,
  };
}

const HANDLE = {
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
};

const VERDICT = JSON.stringify({ verdict: 'approve', findings: [] });

/**
 * Stub server whose sessions answer with `text` once the runner has asked at
 * least `answerAfter` times. Each prompt produces one turn that becomes
 * visible only on a list call *after* the prompt — the way a real reply
 * lands — so a resumed attempt's answer grows the assistant count past the
 * baseline the runner took right after re-prompting.
 */
function stubServer(text: string, answerAfter: number) {
  const prompts: Array<{ sessionID: string; text: string }> = [];
  const turns = new Map<string, Array<{ text: string; visibleFrom: number }>>();
  const lists = new Map<string, number>();
  let clock = 0;
  let created = 0;
  const client = {
    session: {
      create: async (input: { agent?: string }) => {
        created += 1;
        const id = `ses_${String(created)}_${input.agent ?? ''}`;
        turns.set(id, []);
        lists.set(id, 0);
        return { id };
      },
      prompt: async (input: { sessionID: string; text: string }) => {
        prompts.push(input);
        const mine = turns.get(input.sessionID) ?? [];
        const answered = mine.length + 1 >= answerAfter;
        mine.push({
          text: answered ? text : 'Let me check one more file.',
          visibleFrom: (lists.get(input.sessionID) ?? 0) + 2,
        });
      },
      remove: async () => undefined,
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const n = (lists.get(input.sessionID) ?? 0) + 1;
        lists.set(input.sessionID, n);
        const messages: SessionMessageInfo[] = [];
        for (const t of turns.get(input.sessionID) ?? []) {
          if (n < t.visibleFrom) {
            continue;
          }
          clock += 1;
          messages.push({
            id: `m_a${String(messages.length)}`,
            time: { created: clock },
            type: 'assistant',
            content: [{ type: 'text', text: t.text }],
          });
          clock += 1;
          messages.push({
            id: `m_i${String(messages.length)}`,
            time: { created: clock },
            type: 'idle',
          });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, prompts, created: () => created };
}

describe('retryInSameSession', () => {
  it('resumes only a missing reply', () => {
    expect(retryInSameSession('invalid-reply: not JSON', 'ses_1')).toBe(true);
    expect(retryInSameSession('invalid-reply: missing verdict', 'ses_1')).toBe(true);
    // Cancelled zombie: nothing to resume, and resuming would resurrect it.
    expect(retryInSameSession('stage-timeout', 'ses_1')).toBe(false);
    expect(retryInSameSession('no model turns — agent may not exist', 'ses_1')).toBe(false);
    // A stage that never got a session cannot be resumed either.
    expect(retryInSameSession('invalid-reply: not JSON', undefined)).toBe(false);
  });
});

describe('replyShapeExample', () => {
  it('shows the required keys as an object, not a schema to echo', () => {
    const def = stageDef('design-review', 'correctness');
    const text = replyShapeExample(stageReplySchema(def));
    expect(text).toContain('"verdict": "approve" | "request-changes"');
    expect(text).toContain('"findings"');
    expect(text).toContain('"severity"');
    expect(text).not.toContain('JSON Schema');
  });
});

describe('nudge pacing', () => {
  it('holds the nudge until the floor has passed, then times out instead of bursting', async () => {
    const { client, prompts } = stubServer(VERDICT, 99);
    const outcome = await executeStage(
      options(client),
      state(),
      HANDLE,
      reviewConfig({ minNudgeIntervalMs: 5_000, stageTimeoutMs: 300 }),
    );
    // One prompt (the stage prompt), no nudge storm: the stage ends on the
    // deadline instead.
    expect(prompts).toHaveLength(1);
    expect(outcome.failure).toBe('stage-timeout');
  });

  it('nudges up to the budget when the floor is clear', async () => {
    const { client, prompts } = stubServer(VERDICT, 99);
    const outcome = await executeStage(options(client), state(), HANDLE, reviewConfig());
    // stage prompt + MAX_NUDGES (8).
    expect(prompts).toHaveLength(9);
    expect(outcome.failure).toBe('invalid-reply: not JSON');
    // The ultimatum asks for the object, not the schema.
    expect(prompts.at(-1)?.text).toContain('FINAL CHANCE');
    expect(prompts.at(-1)?.text).toContain('"verdict": "approve" | "request-changes"');
    expect(prompts.at(-1)?.text).not.toContain('JSON Schema');
  });
});

describe('attempt 2 resumes the session that failed to reply', () => {
  it('reuses the session and asks only for the reply', async () => {
    const { client, prompts, created } = stubServer(VERDICT, 10);
    const opts = options(client);
    const cfg = reviewConfig();

    const first = await executeStage(opts, state(), HANDLE, cfg);
    expect(first.failure).toBe('invalid-reply: not JSON');
    expect(created()).toBe(1);

    const second = await executeStage(
      opts,
      state(),
      {
        ...HANDLE,
        attempt: 2,
        resume: { sessionID: first.sessionID as string, text: 'finish and reply' },
      },
      cfg,
    );

    expect(second.ok).toBe(true);
    // The retry ran in the same session: no second create, no re-sent prompt.
    expect(created()).toBe(1);
    expect(prompts.filter((p) => p.text === 'finish and reply')).toHaveLength(1);
    // The stage prompt of attempt 1 is not repeated in attempt 2.
    expect(prompts.filter((p) => p.text.includes('JSON Schema'))).toHaveLength(1);
  });
});
