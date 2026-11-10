import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import { type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// Design 007 §2 + §3, end to end over a temp git repo with a mock client:
// a completed child-session deed fails the parent attempt (violation text
// appended with `(via child session …)`), and the non-resumable attempt 2 —
// on both the single-stage lane and the review-pass lane — carries attempt
// 1's failure reason in its prompt. The retry budget is untouched: one
// retry, then escalation.
// ---------------------------------------------------------------------------

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

function baseState(): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'escalated',
    lastStage: 'proposal',
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
}

function tmpRepo(state: PipelineState): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-deeds-retry-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), '# proposal\n\nbody\n');
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  writeFileSync(
    join(dir, 'docs', 'designs', '002-x', '.pipeline.json'),
    `${JSON.stringify(state, null, 2)}\n`,
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

function optionsFor(dir: string, client: OpenCodeClient, extra: object): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: FAST,
    client,
    ...extra,
  };
}

function savedState(dir: string): PipelineState {
  return JSON.parse(
    readFileSync(join(dir, 'docs', 'designs', '002-x', '.pipeline.json'), 'utf8'),
  ) as PipelineState;
}

/**
 * Message stamps ride the real clock: the audit's `since` filter is a real
 * timestamp, so a virtual clock at 0 would hide every deed from it.
 */
function realClock(): () => number {
  let clock = 0;
  return () => {
    clock = Math.max(Date.now(), clock + 1);
    return clock;
  };
}

const CHILD_READ = 'docs/designs/002-x/transcripts/r0.md';

/**
 * Stub server for the single-stage lane: every `implement` session delegates
 * through a COMPLETED `subagent` part whose text names nothing forbidden;
 * each such session has one child session — enumerable through BOTH SDK
 * routes the walk may take (A4) — and that child's own message list holds a
 * completed `read` of a transcripts file. `listOnly: true` removes
 * `session.children`, the shape the pinned SDK actually exposes, so the walk
 * has to find the deed through the `session.list` + `parentID` route;
 * `listCalls()` counts those enumerations.
 */
function deedsStub({ listOnly = false }: { listOnly?: boolean } = {}) {
  const prompts: Array<{ sessionID: string; text: string }> = [];
  const creates: Array<{ id: string; key: string }> = [];
  const info = new Map<string, { id: string; parentID?: string }>();
  const childrenByParent = new Map<string, string[]>();
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  const stamp = realClock();
  let created = 0;
  let listCalls = 0;

  const replyFor = (key: string): string => {
    if (key === 'implement') {
      return JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' });
    }
    return JSON.stringify({ verdict: 'approve', findings: [] });
  };

  const childrenOf = async (input: { sessionID: string }) =>
    (childrenByParent.get(input.sessionID) ?? []).map((id) => ({ id }));

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created += 1;
        const id = `ses_${String(created)}`;
        creates.push({ id, key });
        sessions.set(id, { key, prompts: 0, lists: 0 });
        info.set(id, { id });
        if (key === 'implement') {
          const child = `ses_child_of_${id}`;
          info.set(child, { id: child, parentID: id });
          childrenByParent.set(id, [child]);
        }
        return { id };
      },
      prompt: async (input: { sessionID: string; text: string }) => {
        prompts.push(input);
        const s = sessions.get(input.sessionID);
        if (s) s.prompts += 1;
      },
      remove: async () => undefined,
      ...(listOnly ? {} : { children: childrenOf }),
      list: async () => {
        listCalls += 1;
        return { data: [...info.values()], cursor: { previous: null, next: null } };
      },
    },
    message: {
      list: async (input: { sessionID: string }) => {
        if (info.get(input.sessionID)?.parentID !== undefined) {
          // A child session's own deeds: a completed read of run records.
          return {
            data: [
              {
                id: `m_child_${input.sessionID}`,
                time: { created: stamp() },
                type: 'assistant',
                content: [
                  {
                    type: 'tool',
                    id: 'p_child',
                    name: 'read',
                    state: {
                      status: 'completed',
                      input: { filePath: CHILD_READ },
                      content: [],
                    },
                  },
                ],
              },
            ],
            cursor: { previous: null, next: null },
          };
        }
        const s = sessions.get(input.sessionID);
        if (!s) return { data: [], cursor: { previous: null, next: null } };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          messages.push({
            id: `m_del${String(i)}`,
            time: { created: stamp() },
            type: 'assistant',
            content: [
              {
                type: 'tool',
                id: `p_del${String(i)}`,
                name: 'subagent',
                state: {
                  status: 'completed',
                  input: { task: 'run the package tests and summarize', agent: 'explore' },
                  content: [],
                },
              },
            ],
          });
          messages.push({
            id: `m_a${String(i)}`,
            time: { created: stamp() },
            type: 'assistant',
            content: [{ type: 'text', text: replyFor(s.key) }],
          });
          messages.push({ id: `m_i${String(i)}`, time: { created: stamp() }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;

  return { client, prompts, creates, listCalls: () => listCalls };
}

describe('child deeds fail the parent attempt end to end', () => {
  // proves: Test strategy #6 — a stage session whose child session reads a
  // transcripts/ file fails the attempt with an independence violation
  // naming the path and the child session; the fresh attempt-2 prompt
  // recorded by the mock client carries attempt 1's failure reason; the
  // budget is unchanged: one retry, then escalation.
  it('fails an attempt on a child deed and feeds attempt 2 the failure reason', async () => {
    const dir = tmpRepo(baseState());
    const stub = deedsStub();

    const status = await resumePipeline(optionsFor(dir, stub.client, { forceStage: 'implement' }));

    expect(status).toBe('escalated');

    // Budget unchanged: attempt 1 + exactly one retry, both fresh sessions
    // (an independence violation is non-resumable).
    const attempts = stub.creates.filter((c) => c.key === 'implement');
    expect(attempts).toHaveLength(2);

    const firstAttempt = stub.prompts.filter((p) => p.sessionID === attempts[0]?.id);
    const secondAttempt = stub.prompts.filter((p) => p.sessionID === attempts[1]?.id);
    expect(firstAttempt[0]?.text).not.toContain('Previous attempt');
    expect(secondAttempt[0]?.text).toContain("Previous attempt's gate failure");
    expect(secondAttempt[0]?.text).toContain('independence violation');
    expect(secondAttempt[0]?.text).toContain('transcripts');
    expect(secondAttempt[0]?.text).toContain('Do not repeat the behavior that caused that failure');

    const saved = savedState(dir);
    const failed = saved.results.filter((r) => r.stage === 'implement');
    expect(failed).toHaveLength(2);
    for (const result of failed) {
      expect(result.failure).toContain('independence violation');
      expect(result.failure).toContain('transcripts');
      expect(result.failure).toContain('(via child session');
    }
    expect(saved.stageFailures['implement']).toBeDefined();
    // One gate entry per failure: the failed attempt lands a `stageFailure`,
    // the exhausted retry lands the run's `escalate` naming that stage. The
    // pair — not two `stageFailure` lines — is the budget-unchanged shape:
    // §3 lists gate entries and the escalation flow among what stays as-is,
    // so a second failure entry would mean the retry became a new attempt.
    const stageFailures = saved.gateLog.filter((g) => g.gate === 'stageFailure');
    const escalations = saved.gateLog.filter((g) => g.gate === 'escalate');
    expect(stageFailures).toHaveLength(1);
    expect(escalations).toHaveLength(1);
    expect(escalations[0]?.reason).toContain('implement');
  });

  // proves: Test strategy #6 — the enumeration route every real run takes:
  // the pinned SDK exposes no `session.children`, so the child deed has to be
  // found through the paged `session.list` + `parentID` index. Same outcome
  // as the children route: the attempt fails on the child's transcripts read
  // and attempt 2 is told why (code review testing r1c0 finding 1).
  it('fails the attempt on a child deed found through the session-list route', async () => {
    const dir = tmpRepo(baseState());
    const stub = deedsStub({ listOnly: true });

    const status = await resumePipeline(optionsFor(dir, stub.client, { forceStage: 'implement' }));

    expect(status).toBe('escalated');
    const attempts = stub.creates.filter((c) => c.key === 'implement');
    expect(attempts).toHaveLength(2);

    const saved = savedState(dir);
    const failed = saved.results.filter((r) => r.stage === 'implement');
    expect(failed).toHaveLength(2);
    for (const result of failed) {
      expect(result.failure).toContain('independence violation');
      expect(result.failure).toContain('transcripts');
      expect(result.failure).toContain('(via child session');
    }

    const secondAttempt = stub.prompts.filter((p) => p.sessionID === attempts[1]?.id);
    expect(secondAttempt[0]?.text).toContain("Previous attempt's gate failure");
    expect(secondAttempt[0]?.text).toContain('independence violation');
  });

  // proves: Test strategy #6 — the walk's cost rule (design 007 §2): one
  // descendant enumeration per attempt, off the poll loop. The stage loop
  // re-audits the attempt's own message list on every re-entry, so an
  // unbounded walk would pay the tree once per turn instead of once per
  // attempt (code review performance r1c0 finding 2).
  it('pays at most one descendant enumeration per attempt', async () => {
    const dir = tmpRepo(baseState());
    const stub = deedsStub({ listOnly: true });

    await resumePipeline(optionsFor(dir, stub.client, { forceStage: 'implement' }));

    const attempts = stub.creates.filter((c) => c.key === 'implement');
    expect(attempts).toHaveLength(2);
    expect(stub.listCalls()).toBeLessThanOrEqual(attempts.length);
  });
});

// ---------------------------------------------------------------------------
// Test strategy #7: the review-pass lane. Run 005's motivating seat — one
// globbing `reviews/design/*` and failing twice on a byte-identical prompt —
// lives here, not in the single-stage lane. Seat retry/permit semantics stay
// as-is (one fresh retry inside the seat's turn, the pass proceeds after it).
// ---------------------------------------------------------------------------

/**
 * Stub whose design seats all approve — except the security seat's FIRST
 * session, which additionally completes a `glob` over the seat report
 * directory. Its verdict is fine; the deed is not.
 */
function seatStub() {
  const prompts: Array<{ sessionID: string; text: string }> = [];
  const creates: Array<{ id: string; key: string; ordinal: number }> = [];
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  const perKey: Record<string, number> = {};
  const stamp = realClock();
  let created = 0;

  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const violating = (key: string, ordinal: number) =>
    key === 'design-review:security' && ordinal === 0;

  const replyFor = (key: string): string => {
    if (key.endsWith('-review')) return 'impossible'; // stage keys here always carry :specialty
    if (key.includes('-review:')) return approve;
    if (key === 'proposal-revise') return JSON.stringify({ changed: true, summary: 'closed it' });
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 0, commitMessage: 'test: pin the contract' });
    if (key === 'implement')
      return JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok' });
  };

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        perKey[key] = (perKey[key] ?? 0) + 1;
        created += 1;
        const id = `ses_${String(created)}`;
        sessions.set(id, { key, ordinal: perKey[key] - 1, prompts: 0, lists: 0 });
        creates.push({ id, key, ordinal: perKey[key] - 1 });
        return { id };
      },
      prompt: async (input: { sessionID: string; text: string }) => {
        prompts.push(input);
        const s = sessions.get(input.sessionID);
        if (s) s.prompts += 1;
      },
      remove: async () => undefined,
      children: async () => [],
      list: async () => ({
        data: creates.map((c) => ({ id: c.id })),
        cursor: { previous: null, next: null },
      }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return { data: [], cursor: { previous: null, next: null } };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          if (violating(s.key, s.ordinal)) {
            messages.push({
              id: `m_glob${String(i)}`,
              time: { created: stamp() },
              type: 'assistant',
              content: [
                {
                  type: 'tool',
                  id: `p_glob${String(i)}`,
                  name: 'glob',
                  state: {
                    status: 'completed',
                    input: { pattern: 'docs/designs/002-x/reviews/design/*' },
                    content: [],
                  },
                },
              ],
            });
          }
          messages.push({
            id: `m_a${String(i)}`,
            time: { created: stamp() },
            type: 'assistant',
            content: [{ type: 'text', text: replyFor(s.key) }],
          });
          messages.push({ id: `m_i${String(i)}`, time: { created: stamp() }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;

  return { client, prompts, creates };
}

describe('review-pass seat retry carries the failure reason', () => {
  // proves: Test strategy #7 — a seat whose attempt 1 violates independence
  // gets a fresh attempt-2 session whose prompt contains the failure reason;
  // seat retry/permit semantics are otherwise unchanged: one retry for the
  // seat, every other seat launched once, the run completes.
  it('feeds the seat attempt-2 prompt the independence failure and finishes', async () => {
    const dir = tmpRepo(baseState());
    const stub = seatStub();

    const status = await resumePipeline(
      optionsFor(dir, stub.client, { cfg: { ...FAST, maxParallelReviews: 1 } }),
    );

    expect(status).toBe('done');

    // c0 panel in handed order, and the security seat alone gets a second
    // session: the retry stays one attempt, no c1 dissent pass is triggered.
    // Seat retry semantics are unchanged — the retry launches immediately
    // after the seat that failed, before the remaining seats.
    const seatCreates = stub.creates.filter((c) => c.key.includes('design-review:'));
    expect(seatCreates.map((c) => c.key)).toEqual([
      'design-review:correctness',
      'design-review:compatibility',
      'design-review:security',
      'design-review:security',
      'design-review:performance',
      'design-review:complexity',
    ]);

    const securityRetry = seatCreates[3];
    const retryPrompts = stub.prompts.filter((p) => p.sessionID === securityRetry?.id);
    expect(retryPrompts[0]?.text).toContain("Previous attempt's gate failure");
    expect(retryPrompts[0]?.text).toContain('independence violation');
    expect(retryPrompts[0]?.text).toContain('reviews');
    expect(retryPrompts[0]?.text).toContain('Do not repeat the behavior that caused that failure');

    const saved = savedState(dir);
    const seatResults = saved.results.filter((r) => r.stage === 'design-review:security');
    expect(seatResults).toHaveLength(2);
    expect(seatResults[0]?.failure).toContain('independence violation');
    expect((seatResults[1]?.reply as { verdict?: string } | undefined)?.verdict).toBe('approve');
  });
});
