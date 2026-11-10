import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../src/pipeline/runner.js';
import { type PipelineConfig } from '../src/pipeline/stages.js';
import type { PipelineState } from '../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-003 pass-level fan-out. The seats a `kind: 'review'` pass selects under
// ADR-002 launch in the fixed handed order through an in-process semaphore of
// size cfg.maxParallelReviews — `planPass` output is scheduled verbatim, the
// semaphore's FIFO queue keeps session-creation order even when N < |panel|,
// and N = 1 is the same code path (a size-1 semaphore, not a branch).
// ---------------------------------------------------------------------------

const DESIGN_SEATS = ['correctness', 'compatibility', 'security', 'performance', 'complexity'];

const BASE: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

/** Chosen design / Configuration: `maxParallelReviews` on PipelineConfig. */
function cfgFor(maxParallelReviews: number, extra: Partial<PipelineConfig> = {}): PipelineConfig {
  return { ...BASE, ...extra, maxParallelReviews };
}

const DESIGN_DOC = join('docs', 'designs', '002-x', '02-proposal.md');

function baseState(partial: Partial<PipelineState> = {}): PipelineState {
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
    ...partial,
  };
}

function tmpRepo(state: PipelineState): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-fanout-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  writeFileSync(join(dir, DESIGN_DOC), '# proposal\n\nfirst body\n');
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

interface FanoutStubOptions {
  /** stageKey -> session ordinals that answer request-changes (0-based). */
  rejects: Record<string, number[]>;
  /** revise sessions write a fresh document (new revision stamp). */
  reviseWrites: boolean;
  /**
   * Prompt handlers stay open for a seat-specific window so concurrency is
   * observable: seat k's window is 15 + 20k ms, so all seats launched at the
   * same instant overlap while the shortest window is still open.
   */
  staggeredPrompts: boolean;
}

function fanoutStub(dir: string, opts: FanoutStubOptions) {
  const created: Record<string, number> = {};
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  const creates: string[] = [];
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const reject = JSON.stringify({
    verdict: 'request-changes',
    findings: [{ severity: 'major', location: 'Decision', problem: 'still broken' }],
  });
  let clock = 0;
  let revisions = 0;
  let inFlight = 0;
  let maxInFlight = 0;

  const promptDelayMs = (key: string): number => {
    if (!opts.staggeredPrompts) return 0;
    const seat = key.includes('-review:') ? key.split(':').pop() : undefined;
    const ordinal = seat === undefined ? -1 : DESIGN_SEATS.indexOf(seat);
    return ordinal < 0 ? 15 : 15 + ordinal * 20;
  };

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created[key] = (created[key] ?? 0) + 1;
        const id = `ses_${String(Object.values(created).reduce((a, b) => a + b, 0))}`;
        sessions.set(id, { key, ordinal: created[key] - 1, prompts: 0, lists: 0 });
        creates.push(key);
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        inFlight += 1;
        maxInFlight = Math.max(maxInFlight, inFlight);
        if (s.key === 'proposal-revise' && opts.reviseWrites) {
          revisions += 1;
          writeFileSync(join(dir, DESIGN_DOC), `# proposal\n\nrevision ${String(revisions)}\n`);
        }
        const delay = promptDelayMs(s.key);
        if (delay > 0) await new Promise((resolve) => setTimeout(resolve, delay));
        inFlight -= 1;
      },
      remove: async () => undefined,
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { key: '', ordinal: 0, prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          clock += 1;
          messages.push({
            id: `m_a${String(clock)}`,
            time: { created: clock },
            type: 'assistant',
            content: [{ type: 'text', text: replyFor(s.key, s.ordinal) }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(clock)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;

  const rejected = (key: string, ordinal: number) => (opts.rejects[key] ?? []).includes(ordinal);
  function replyFor(key: string, ordinal: number): string {
    if (key.endsWith('-review')) return 'impossible'; // stage keys here always carry :specialty
    if (key.includes('-review:')) return rejected(key, ordinal) ? reject : approve;
    if (key === 'proposal-revise') return JSON.stringify({ changed: true, summary: 'closed it' });
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 0, commitMessage: 'test: pin the contract' });
    if (key === 'implement')
      return JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok' });
  }

  return { client, creates, maxInFlight: () => maxInFlight };
}

function optionsFor(dir: string, extra: object): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../prompts/', import.meta.url)),
    ...extra,
  };
}

/** All five design seats dissent; the c0 pass runs, then the run escalates. */
function dissentFixture(maxParallelReviews: number) {
  const dir = tmpRepo(baseState());
  const stub = fanoutStub(dir, {
    rejects: Object.fromEntries(DESIGN_SEATS.map((s) => [`design-review:${s}`, [0]])),
    reviseWrites: true,
    staggeredPrompts: true,
  });
  return { dir, stub, cfg: cfgFor(maxParallelReviews, { maxReviews: { design: 5, code: 20 } }) };
}

describe('review-pass fan-out at cfg.maxParallelReviews', () => {
  // proves: Test strategy #2 — full-panel pass launches all five design seats
  // concurrently at N >= |panel| (max in-flight observed == |panel|), and
  // in-flight never exceeds N at any N. One test per N so each gets its own
  // timeout budget; the staggered stub pins the exact peak.
  for (const n of [1, 2, 3, 5]) {
    it(`fans out the design pass at N = ${n}`, async () => {
      const { dir, stub, cfg } = dissentFixture(n);
      const status = await resumePipeline(optionsFor(dir, { client: stub.client, cfg }));
      expect(status).toBe('escalated');
      // At N = 5 every seat of the panel is in flight at once; at every N the
      // in-flight count never climbs past N (maxInFlight is the peak over the
      // whole run, retries and later stages included).
      expect(stub.maxInFlight()).toBe(Math.min(n, DESIGN_SEATS.length));
    });
  }

  // proves: Test strategy #3 — the dissent pass launches only the dissenting
  // seats, the ratification pass only the stale approvers (planPass output
  // scheduled verbatim, at N >= |panel|)
  it('launches only the seats planPass hands: dissenters, then stale approvers', async () => {
    const dir = tmpRepo(baseState());
    const stub = fanoutStub(dir, {
      rejects: { 'design-review:security': [0] }, // only the first security session dissents
      reviseWrites: true,
      staggeredPrompts: false,
    });

    const status = await resumePipeline(optionsFor(dir, { client: stub.client, cfg: cfgFor(5) }));
    expect(status).toBe('done');

    // c0: the full panel in handed order. c1: the security seat alone.
    // c2 (ratification): the four stale approvers in handed order — security,
    // already re-approved on the new stamp, is not re-launched.
    const designCreates = stub.creates.filter((k) => k.startsWith('design-review:'));
    expect(designCreates).toEqual([
      ...DESIGN_SEATS.map((s) => `design-review:${s}`),
      'design-review:security',
      'design-review:correctness',
      'design-review:compatibility',
      'design-review:performance',
      'design-review:complexity',
    ]);
  });

  // proves: Test strategy #4 — launch order at N < |panel|: session.create
  // order equals the handed seat order (FIFO waiter queue)
  it('creates design seat sessions in handed order at N = 2', async () => {
    const { dir, stub, cfg } = dissentFixture(2);

    const status = await resumePipeline(optionsFor(dir, { client: stub.client, cfg }));
    expect(status).toBe('escalated');

    // The fan-out really ran two-at-a-time (not accidentally serial)…
    expect(stub.maxInFlight()).toBe(2);
    // …and the FIFO queue kept creation order exactly as planPass handed it.
    expect(stub.creates.filter((k) => k.startsWith('design-review:'))).toEqual(
      DESIGN_SEATS.map((s) => `design-review:${s}`),
    );
  });
});
