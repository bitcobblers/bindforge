import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it, vi } from 'vitest';

import {
  type RunOptions,
  escalatesWithoutRetry,
  executeStage,
  replyShapeExample,
  retryInSameSession,
} from '../../src/pipeline/runner.js';
import { type PipelineConfig, stageDef, stageReplySchema } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// Run 1's reply contract failed 5 times in 43 stage attempts, and each failure
// threw away a session that had already done the work: round 4 died on two
// correctness replies that were valid JSON the runner had mis-read, and the
// final nudge re-sent the JSON Schema, which the model answered by echoing a
// schema. Nudges also arrived eight deep in 69 seconds at sessions that were
// still working.
// ---------------------------------------------------------------------------

// Fault injection for the other failure shape the proposal's Wiring paragraph
// names: "a failure — a returned issue or a throw from a rule". The validator's
// rule-4 tracked-file lookup is its one git call, so armed, that spawn throws
// rather than reporting an exit status. `validateMoves` then either propagates
// the throw (the runner's catch turns it into the same failure string) or reads
// it as "not a tracked regular file"; both shapes must arrive as
// `invalid-reply: moves` and neither may crash the stage. Nothing else in this
// file shells out with those arguments, so the passthrough stays exact.
const gitFault = vi.hoisted(() => ({ armed: false }));

vi.mock('node:child_process', async (importOriginal) => {
  const actual = await importOriginal<typeof import('node:child_process')>();
  const isArmedCall = (callArgs: unknown[]): boolean => {
    if (!gitFault.armed) {
      return false;
    }
    const argv = callArgs.flatMap((a) => (Array.isArray(a) ? a.map(String) : [String(a)]));
    return argv.some((a) => a.includes('ls-tree')) && argv.some((a) => a.includes('boom.md'));
  };
  return {
    ...actual,
    spawnSync: (...callArgs: Parameters<typeof actual.spawnSync>) => {
      if (isArmedCall(callArgs)) {
        throw new Error('injected validator fault: git ls-tree');
      }
      return actual.spawnSync(...callArgs);
    },
    execFileSync: (...callArgs: Parameters<typeof actual.execFileSync>) => {
      if (isArmedCall(callArgs)) {
        throw new Error('injected validator fault: git ls-tree');
      }
      return actual.execFileSync(...callArgs);
    },
  };
});

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
  const dir = mkTmpDir('bindforge-reply-');
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
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
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
      // Test strategy #9: the descendant walk's enumeration endpoint. This
      // fixture creates no child sessions, so an empty page is the truth.
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
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

  // proves: Test strategy #6 — failure-classification of the moves outcomes:
  // `invalid-reply: moves …` resumes the same session (validator rejections are
  // invalid replies); `moves-apply-failed: …` does not (attempt 2 starts a fresh
  // session, the failure being environmental), and neither does
  // `moves-rollback-failed: …` — which is terminal, pinned by the next suite.
  it('classifies moves failures by their prefix', () => {
    expect(
      retryInSameSession('invalid-reply: moves — from is not tracked: src/a.ts', 'ses_1'),
    ).toBe(true);
    expect(retryInSameSession('moves-apply-failed: ENOTDIR rename src/a.ts', 'ses_1')).toBe(false);
    expect(retryInSameSession('moves-rollback-failed: EACCES undo src/a.ts', 'ses_1')).toBe(false);
    expect(retryInSameSession('moves-rollback-failed: EACCES undo src/a.ts', undefined)).toBe(
      false,
    );
  });
});

describe('escalatesWithoutRetry', () => {
  // proves: Test strategy #6 — the rollback failure is the terminal class.
  // `retryInSameSession(...) === false` cannot tell "fresh attempt 2" from "no
  // attempt 2": it is the same value a `moves-apply-failed` outcome has. The
  // driver branches on `escalatesWithoutRetry` instead — a rollback failure
  // escalates to the operator instead of pointing a new session at a
  // known-damaged worktree, and every other class keeps today's retry.
  it('escalates without a second attempt only for the rollback-failure class', () => {
    expect(escalatesWithoutRetry('moves-rollback-failed: EACCES undo src/a.ts')).toBe(true);
    expect(escalatesWithoutRetry('moves-apply-failed: ENOTDIR rename src/a.ts')).toBe(false);
    expect(escalatesWithoutRetry('invalid-reply: moves — from is not tracked: src/a.ts')).toBe(
      false,
    );
    expect(escalatesWithoutRetry('stage-timeout')).toBe(false);
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

// ---------------------------------------------------------------------------
// ADR-005: the implement, fix and test-update replies gain
// `moves: { from, to }[]`. Schema-level violations fail `safeParse` like any
// invalid reply; a schema-valid list that `validateMoves` rejects takes the
// exact same invalid-reply retry/escalation path (nudge with the issue,
// same-session attempt 2, second failure escalates).
// ---------------------------------------------------------------------------

describe('moves in reply schemas', () => {
  // proves: Test strategy #1 — `moves` is optional with default [] on all three
  // move-capable entries, keeping the checkpoint additive (A13).
  it('parses a missing moves field as the empty list for implement, fix and test-update', () => {
    const cases: Array<[string, object]> = [
      ['implement', { testsNeedUpdate: false, commitMessage: 'feat: x' }],
      ['fix', { testsNeedUpdate: false, commitMessage: 'fix: x' }],
      ['test-update', { commitMessage: 'test: x' }],
    ];
    for (const [stage, reply] of cases) {
      const result = stageReplySchema(stageDef(stage, undefined)).safeParse(reply);
      expect(result.success, stage).toBe(true);
      const data = (result as { data?: { moves?: unknown } }).data;
      expect(data?.moves, stage).toEqual([]);
    }
  });

  // proves: Test strategy #1 — malformed pairs (non-string, empty endpoint), an
  // endpoint over the 200-char cap, and a list over the 64-pair cap all fail
  // safeParse like any invalid reply.
  it('rejects malformed pairs, over-long endpoints and over-cap lists', () => {
    const schema = stageReplySchema(stageDef('implement', undefined));
    const base = { testsNeedUpdate: false, commitMessage: 'feat: x' };
    expect(schema.safeParse({ ...base, moves: [{ from: 5, to: 'src/a.ts' }] }).success).toBe(false);
    expect(schema.safeParse({ ...base, moves: [{ from: '', to: 'src/a.ts' }] }).success).toBe(
      false,
    );
    expect(schema.safeParse({ ...base, moves: [{ from: 'src/a.ts', to: '' }] }).success).toBe(
      false,
    );
    expect(
      schema.safeParse({ ...base, moves: [{ from: 'a'.repeat(201), to: 'src/a.ts' }] }).success,
    ).toBe(false);
    const overPairCap = Array.from({ length: 65 }, (_, i) => ({
      from: `src/f${String(i)}.ts`,
      to: `src/g${String(i)}.ts`,
    }));
    expect(schema.safeParse({ ...base, moves: overPairCap }).success).toBe(false);
  });
});

describe('a rejected moves list routes through the invalid-reply retry path', () => {
  const promptsDir = fileURLToPath(new URL('../../prompts/', import.meta.url));

  const IMPL_HANDLE = {
    stage: 'implement',
    stageKey: 'implement',
    attempt: 1,
    round: 1,
    specialty: undefined,
    vars: { designDir: 'docs/designs/002-x' },
  };

  function movesRepo(): string {
    const dir = mkTmpDir('bindforge-moves-reply-');
    const git = (args: string[]) =>
      execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
    mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
    mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
    writeFileSync(join(dir, 'docs', 'adrs', '002-x.md'), 'x\n');
    writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), 'v1\n');
    writeFileSync(join(dir, 'docs', 'designs', '002-x', 'note.md'), 'note\n');
    // Tracked, ordinary file the throwing-validator case moves: otherwise legal,
    // so the only thing that can reject it is the rule that throws.
    writeFileSync(join(dir, 'docs', 'designs', '002-x', 'boom.md'), 'boom\n');
    git(['init', '-q', '-b', 'main']);
    git(['add', '-A']);
    git(['commit', '-q', '-m', 'docs: base']);
    return dir;
  }

  /**
   * Stub server that answers a session's n-th prompt with `texts[n-1]`
   * (clamped), each turn visible one poll after the prompt — same turn
   * visibility as `stubServer`, but the reply can change per prompt so a
   * repair prompt can flip the answer.
   */
  function sequenceStubServer(texts: string[]) {
    const prompts: Array<{ sessionID: string; text: string }> = [];
    const turns = new Map<string, Array<{ text: string; visibleFrom: number }>>();
    const lists = new Map<string, number>();
    const promptCounts = new Map<string, number>();
    let clock = 0;
    let created = 0;
    const client = {
      session: {
        create: async () => {
          created += 1;
          const id = `ses_impl_${String(created)}`;
          turns.set(id, []);
          lists.set(id, 0);
          promptCounts.set(id, 0);
          return { id };
        },
        prompt: async (input: { sessionID: string; text: string }) => {
          prompts.push(input);
          const n = (promptCounts.get(input.sessionID) ?? 0) + 1;
          promptCounts.set(input.sessionID, n);
          const mine = turns.get(input.sessionID) ?? [];
          mine.push({
            text: texts[Math.min(n - 1, texts.length - 1)],
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

  // proves: Test strategy #9 — an attempt whose moves list the validator rejects
  // fails with the `invalid-reply: moves` reason naming the offending pair;
  // attempt 2 resumes the same session with a repair prompt quoting the failure,
  // and its valid list lands.
  it('fails attempt 1 with invalid-reply: moves and repairs it in the same session', async () => {
    const invalid = JSON.stringify({
      testsNeedUpdate: false,
      commitMessage: 'feat: rename the note',
      moves: [{ from: 'docs/designs/002-x/ghost.md', to: 'docs/designs/002-x/landed.md' }],
    });
    const valid = JSON.stringify({
      testsNeedUpdate: false,
      commitMessage: 'feat: rename the note',
      moves: [{ from: 'docs/designs/002-x/note.md', to: 'docs/designs/002-x/renamed.md' }],
    });
    // Nine invalid answers cover the whole attempt-1 prompt budget
    // (stage prompt + 8 nudges); the 10th answer — attempt 2's — is valid.
    const { client, prompts, created } = sequenceStubServer([
      ...Array<string>(9).fill(invalid),
      valid,
    ]);
    const opts: RunOptions = {
      repoDir: movesRepo(),
      sourcePath: 'docs/adrs/002-x.md',
      promptsDir,
      client,
    };
    const cfg = reviewConfig();

    const first = await executeStage(opts, state(), IMPL_HANDLE, cfg);
    expect(first.failure).toMatch(/^invalid-reply: moves/);
    expect(first.failure).toContain('docs/designs/002-x/ghost.md');
    // The in-attempt nudge loop hands back the issue message naming the pair.
    expect(prompts.some((p) => p.text.includes('docs/designs/002-x/ghost.md'))).toBe(true);

    const repairText = `Your reply failed — ${first.failure}. Reply with a valid moves list.`;
    const second = await executeStage(
      opts,
      state(),
      {
        ...IMPL_HANDLE,
        attempt: 2,
        resume: { sessionID: first.sessionID as string, text: repairText },
      },
      cfg,
    );

    expect(second.ok).toBe(true);
    // The retry ran in the same session: retryInSameSession accepts the
    // `invalid-reply: moves` prefix, and the repair prompt quoted the failure.
    expect(created()).toBe(1);
    expect(prompts.filter((p) => p.text === repairText)).toHaveLength(1);
  });

  // proves: Test strategy #9 — a second rejection escalates with the validator
  // message naming the pair in the reason. `CON` is a Windows-reserved segment
  // (rule 1, pinned on Linux for every platform), and it arrives here as the
  // validator's returned issue; the runner also catches a rule that throws, so
  // both shapes the proposal's Wiring paragraph names land on the same
  // `invalid-reply: moves` path.
  it('escalates a second rejection carrying the validator message naming the pair', async () => {
    const reserved = JSON.stringify({
      testsNeedUpdate: false,
      commitMessage: 'feat: rename the note',
      moves: [{ from: 'CON', to: 'docs/designs/002-x/renamed.md' }],
    });
    const { client, created } = sequenceStubServer(Array<string>(20).fill(reserved));
    const opts: RunOptions = {
      repoDir: movesRepo(),
      sourcePath: 'docs/adrs/002-x.md',
      promptsDir,
      client,
    };
    const cfg = reviewConfig();

    const first = await executeStage(opts, state(), IMPL_HANDLE, cfg);
    expect(first.failure).toMatch(/^invalid-reply: moves/);
    expect(first.failure).toContain('CON');

    const second = await executeStage(
      opts,
      state(),
      {
        ...IMPL_HANDLE,
        attempt: 2,
        resume: { sessionID: first.sessionID as string, text: 'repair: fix the moves list' },
      },
      cfg,
    );
    expect(second.ok).toBe(false);
    expect(second.failure).toMatch(/^invalid-reply: moves/);
    expect(second.failure).toContain('CON');
    expect(created()).toBe(1);
  });

  // proves: Test strategy #2 — the Windows-reserved-name clause's second half,
  // which the proposal carries on the rejection path: "that last asserting a
  // rule that throws still lands as `invalid-reply: moves`, not a crash". The
  // fault is a throwing git lookup inside `validateMoves` (testing review
  // r1c0, finding 8: `CON` alone never throws, so the claim needs real
  // injection). Both shapes the proposal's Wiring paragraph names — a returned
  // issue, or a throw a rule lets escape — end on the same invalid-reply
  // failure, and the awaited `executeStage` resolving at all is the assertion
  // that the stage did not crash.
  it('turns a validator rule that throws into an invalid-reply: moves failure', async () => {
    const throwing = JSON.stringify({
      testsNeedUpdate: false,
      commitMessage: 'feat: rename the boom note',
      moves: [{ from: 'docs/designs/002-x/boom.md', to: 'docs/designs/002-x/renamed.md' }],
    });
    const { client, created } = sequenceStubServer(Array<string>(20).fill(throwing));
    const opts: RunOptions = {
      repoDir: movesRepo(),
      sourcePath: 'docs/adrs/002-x.md',
      promptsDir,
      client,
    };

    gitFault.armed = true;
    try {
      const first = await executeStage(opts, state(), IMPL_HANDLE, reviewConfig());
      expect(first.ok).toBe(false);
      expect(first.failure).toMatch(/^invalid-reply: moves/);
      // The proposal names both shapes — a returned issue, or a throw from a
      // rule — so either the error message or the pair reaches the reason; what
      // is fixed is the class and that the failure carries an issue beyond it.
      expect((first.failure ?? '').length).toBeGreaterThan('invalid-reply: moves'.length);
      expect(created()).toBe(1);
    } finally {
      gitFault.armed = false;
    }
  });

  // proves: Test strategy #9 — the rejected list keeps every way the invalid-reply
  // path can end, the pacing floor included: with a nudge floor longer than the
  // stage deadline (the floor a moves rejection skipped in round 1, correctness
  // review finding 5), the attempt ends at the deadline as `stage-timeout` after
  // the single stage prompt instead of burning the nudge budget instantly — the
  // exact shape of the `nudge pacing` suite above for a schema mismatch.
  it('holds a rejected moves list to the same nudge pacing floor and deadline', async () => {
    const invalid = JSON.stringify({
      testsNeedUpdate: false,
      commitMessage: 'feat: rename the note',
      moves: [{ from: 'docs/designs/002-x/ghost.md', to: 'docs/designs/002-x/landed.md' }],
    });
    const { client, prompts } = sequenceStubServer(Array<string>(20).fill(invalid));
    const opts: RunOptions = {
      repoDir: movesRepo(),
      sourcePath: 'docs/adrs/002-x.md',
      promptsDir,
      client,
    };

    const outcome = await executeStage(
      opts,
      state(),
      IMPL_HANDLE,
      reviewConfig({ minNudgeIntervalMs: 5_000, stageTimeoutMs: 300 }),
    );
    expect(prompts).toHaveLength(1);
    expect(outcome.failure).toBe('stage-timeout');
  });
});
