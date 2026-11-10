import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../src/pipeline/runner.js';
import { type PipelineConfig } from '../src/pipeline/stages.js';
import type { PipelineState } from '../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-003 escalation inside a fanned-out pass. A seat that fails twice does
// not call finalizeEscalate mid-pass: it sets a pass-level `escalationPending`
// flag, and a seat acquiring a permit after the flag is set releases it and
// returns without prompting — no seat that today's StopDriving would skip ever
// gets run. Escalation lands after the sibling barrier: drain, ONE pass
// enforceOwnership sweep, then ONE finalizeEscalate. The retry stays inside
// its own permit and its own session (one retry, same session, for an
// invalid reply): at N = 2 the waiting third seat must NOT get the freed
// permit for the retry, so if the retry released and re-acquired, the queued
// seat would launch — which these tests forbid. The fixtures clear the nudge
// floor (`minNudgeIntervalMs: 0`) so the seat really does fail the reply
// contract: on the default floor the attempt would end at the deadline as
// `stage-timeout`, a non-resumable zombie session that proves a different path.
// ---------------------------------------------------------------------------

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 5,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
  // The seat under test must fail the REPLY CONTRACT, which is the failure the
  // design's retry path is written for: an unparseable reply ends the attempt as
  // `invalid-reply`, and `retryInSameSession` resumes that same session for
  // attempt 2. With the default nudge floor the unparseable reply would instead
  // end the attempt at the deadline as `stage-timeout`, and a timed-out session
  // is a cancelled zombie that cannot be resumed: attempt 2 would open a second
  // session and the pass would stop proving Test strategy #5's same-permit,
  // same-seat retry. Same pairing as `tests/pipeline-reply-contract.test.ts`.
  minNudgeIntervalMs: 0,
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
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-escalate-lane-'));
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
  // Untracked out-of-policy probe: the pass ownership sweep that must run
  // BEFORE finalizeEscalate reverts it; a missing sweep lets `git add -A`
  // in the escalation commit sweep it in instead.
  mkdirSync(join(dir, 'tools'), { recursive: true });
  writeFileSync(join(dir, 'tools', 'stray.ts'), 'export const stray = 1;\n');
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A', '--', 'docs', 'package.json']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

/** Assert the pre-escalation ownership sweep ran: the stray never reaches the
 * escalation commit and the tree ends clean (r1 security revision). */
function assertSweepRanBeforeEscalate(dir: string): void {
  const sha = gitAt(dir, ['log', '--format=%H', '--grep=^docs: 002-x pipeline escalated$']).trim();
  expect(sha).not.toBe('');
  const files = gitAt(dir, ['show', '--name-only', '--format=', sha]).trim().split('\n');
  expect(files).not.toContain('tools/stray.ts');
  expect(gitAt(dir, ['status', '--porcelain'])).toBe('');
}

/**
 * Stub server: `invalid` keys answer every prompt with an unparseable reply —
 * with `minNudgeIntervalMs: 0` that is the invalid-reply failure: the runner
 * burns its nudge budget, the attempt ends as `invalid-reply`, and attempt 2
 * resumes the SAME session while still holding the same permit. `slowMs` sets
 * each seat's prompt window (default 60 ms): the failing seat keeps a short
 * window so both of its attempts settle before the slow sibling gives up its
 * permit, so the escalation flag lands while that sibling is still in flight
 * and every queued seat is stopped at `acquire`.
 */
function laneStub(opts: { invalid: string[]; slowMs: Record<string, number> }) {
  const creates: string[] = [];
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  let clock = 0;
  let inFlight = 0;
  let maxInFlight = 0;

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        creates.push(key);
        const id = `ses_${String(creates.length)}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        inFlight += 1;
        maxInFlight = Math.max(maxInFlight, inFlight);
        const delay = opts.slowMs[s.key] ?? 60;
        if (delay > 0) await new Promise((resolve) => setTimeout(resolve, delay));
        inFlight -= 1;
      },
      remove: async () => undefined,
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { key: '', prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const text = opts.invalid.includes(s.key) ? 'impossible' : approve;
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

  return { client, creates, maxInFlight: () => maxInFlight };
}

function subjects(dir: string): string[] {
  return execFileSync('git', ['log', '--reverse', '--format=%s'], { cwd: dir, encoding: 'utf8' })
    .trim()
    .split('\n');
}

function savedState(dir: string): PipelineState {
  return JSON.parse(
    readFileSync(join(dir, 'docs', 'designs', '002-x', '.pipeline.json'), 'utf8'),
  ) as PipelineState;
}

function optionsFor(dir: string, client: OpenCodeClient, maxParallelReviews: number) {
  const options: ResumeOptions = {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../prompts/', import.meta.url)),
    cfg: { ...FAST, maxParallelReviews },
    client,
  };
  return options;
}

describe('seat failing twice inside a fanned-out pass', () => {
  // proves: Test strategy #5 — at N = 1 (today's exact control flow): the
  // seat's retry runs (inside the seat's own permit), and once the seat fails
  // twice no unstarted seat runs; escalation is one ownership sweep then one
  // finalizeEscalate.
  it('skips every unstarted seat after a seat fails twice at N = 1', async () => {
    const dir = tmpRepo(baseState());
    const stub = laneStub({
      invalid: ['design-review:correctness'],
      // Short prompt windows: both attempts (stage prompt + nudge budget, twice)
      // settle in well under the 5 s deadline, so the attempt really ends as
      // `invalid-reply` and attempt 2 resumes the seat's own session.
      slowMs: { 'design-review:correctness': 5 },
    });

    const status = await resumePipeline(optionsFor(dir, stub.client, 1));

    expect(status).toBe('escalated');
    // The invalid-reply retry reuses the seat's own session…
    expect(stub.creates.filter((k) => k.includes('-review:'))).toEqual([
      'design-review:correctness',
    ]);
    // …but both attempts are recorded: the retry really ran.
    const saved = savedState(dir);
    expect(saved.results.filter((r) => r.stage === 'design-review:correctness')).toHaveLength(2);
    expect(stub.maxInFlight()).toBe(1); // the retry never doubled up the single permit
    // No other seat was ever prompted: the escalationPending flag stopped them.
    expect(stub.creates.filter((k) => k.includes('-review:'))).toHaveLength(1);

    // Exactly one finalizeEscalate: the only commit after base is the terminal
    // record; no per-seat or per-pass review commit was minted mid-pass.
    expect(subjects(dir)).toEqual(['docs: base', 'docs: 002-x pipeline escalated']);
    expect(existsSync(join(dir, 'docs', 'designs', '002-x', 'ESCALATION.md'))).toBe(true);
    assertSweepRanBeforeEscalate(dir);

    expect(saved.status).toBe('escalated');
    // The failure is recorded as today (failure record + gate entry).
    expect(saved.stageFailures['design-review:correctness']).toBeDefined();
    expect(saved.gateLog.some((g) => g.gate === 'stageFailure')).toBe(true);
  });

  // proves: Test strategy #5 — at N = 2: the retry happens inside the failing
  // seat's own permit while the sibling still holds the other one (a
  // release-and-re-acquire would hand the free permit to the queued security
  // seat, which must never start), the running sibling settles before the
  // escalation lands, and escalation is one ownership sweep then one
  // finalizeEscalate.
  it('holds the retry inside its permit and escalates once at N = 2', async () => {
    const dir = tmpRepo(baseState());
    const stub = laneStub({
      invalid: ['design-review:correctness'],
      // The failing seat answers fast, so both of its attempts settle in a few
      // hundred ms; the sibling keeps its prompt open until ~600 ms and so holds
      // the other permit, which is what keeps the three queued seats parked in
      // the semaphore's waiter queue until the flag lands.
      slowMs: { 'design-review:correctness': 5, 'design-review:compatibility': 600 },
    });

    const status = await resumePipeline(optionsFor(dir, stub.client, 2));

    expect(status).toBe('escalated');
    // correctness: one session, retried in place inside its own permit while the
    // other permit stays busy until ~600 ms — security, performance and
    // complexity never got a session (the escalationPending flag stopped them at
    // acquire, and the retry never handed its permit to a sibling).
    expect(stub.creates.filter((k) => k.includes('-review:'))).toEqual([
      'design-review:correctness',
      'design-review:compatibility',
    ]);
    expect(stub.maxInFlight()).toBe(2); // in-flight never exceeded N, retry included

    // The sibling that was already running completed and was recorded normally
    // before the escalation.
    const saved = savedState(dir);
    const settled = saved.results.find((r) => r.stage === 'design-review:compatibility');
    expect(settled).toBeDefined();
    expect((settled?.reply as { verdict?: string } | undefined)?.verdict).toBe('approve');
    expect(saved.results.filter((r) => r.stage === 'design-review:correctness')).toHaveLength(2);
    expect(saved.stageFailures['design-review:correctness']).toBeDefined();
    expect(saved.gateLog.some((g) => g.gate === 'stageFailure')).toBe(true);

    // One finalizeEscalate after the barrier — no pass commit, no per-seat commits.
    expect(subjects(dir)).toEqual(['docs: base', 'docs: 002-x pipeline escalated']);
    expect(existsSync(join(dir, 'docs', 'designs', '002-x', 'ESCALATION.md'))).toBe(true);
    assertSweepRanBeforeEscalate(dir);
  });
});
