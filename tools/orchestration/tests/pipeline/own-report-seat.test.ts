import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import { type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// ADR-016 Decision 1 end to end, on the review-pass lane. A design seat writes
// its report and then RE-READS exactly that path — the `read`-tool route and
// the `git diff` route §3's per-seat allow opens. The identity machinery takes
// the path from the rendered prompt (the runner hands each seat its
// `reportPath`), so the staged deeds name the attempt's own report by the
// runner's own spelling, not a fixture guess. One sibling path added to the
// same session must fail the attempt, naming the sibling. Seat retry/permit
// semantics stay as design 007 left them: one fresh retry inside the seat's
// turn, the panel proceeds after it.
//
// Before the identity exemption lands, the first case fails: today's audit
// fails ANY completed `read` of a `reviews/` path on a review stage, own
// report included.
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
  const dir = mkTmpDir('bindforge-own-report-');
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

function optionsFor(dir: string, client: OpenCodeClient): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: FAST,
    client,
  };
}

function savedState(dir: string): PipelineState {
  return JSON.parse(
    readFileSync(join(dir, 'docs', 'designs', '002-x', '.pipeline.json'), 'utf8'),
  ) as PipelineState;
}

/** Message stamps ride the real clock: the audit's `since` filter is a real
 * timestamp, so a virtual clock at 0 would hide every deed from it. */
function realClock(): () => number {
  let clock = 0;
  return () => {
    clock = Math.max(Date.now(), clock + 1);
    return clock;
  };
}

/**
 * Stub whose `design-review:security` seat's FIRST session stages the seat
 * deeds: it writes its own report, re-reads that exact path with `read`, then
 * reaches it again through a `git diff` shell part — the path taken verbatim
 * from the `reportPath` the prompt rendered for that session. `sibling: true`
 * additionally completes a `read` of the correctness seat's report. Every
 * other session (and the security seat's own retry) just approves.
 */
function seatStub({ sibling }: { sibling: boolean }) {
  const prompts: Array<{ sessionID: string; text: string }> = [];
  const creates: Array<{ id: string; key: string; ordinal: number }> = [];
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  const perKey: Record<string, number> = {};
  const stamp = realClock();
  let created = 0;
  let deedsStaged = 0;

  const approve = JSON.stringify({ verdict: 'approve', findings: [] });

  const replyFor = (key: string): string => {
    if (key.endsWith('-review')) return 'impossible'; // stage keys here always carry :specialty
    if (key.includes('-review:')) return approve;
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 0, commitMessage: 'test: pin the contract' });
    if (key === 'implement')
      return JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok' });
  };

  /** The report path the runner rendered into this session's prompt — the
   * `ownReportPath` its audit options must carry, read from the source of
   * truth instead of a fixture-side guess at the pass numbering. */
  const ownReportOf = (sessionID: string): string | undefined => {
    const text = prompts.filter((p) => p.sessionID === sessionID).at(-1)?.text ?? '';
    return /docs\/designs\/[0-9a-z-]+\/reviews\/design\/security-r\d+(?:c\d+)?\.md/.exec(text)?.[0];
  };

  const toolPart = (name: string, input: Record<string, unknown>): unknown => ({
    type: 'tool',
    id: `p_${name}_${String(stamp())}`,
    name,
    state: { status: 'completed', input, content: [] },
  });

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
          if (s.key === 'design-review:security' && s.ordinal === 0) {
            const own = ownReportOf(input.sessionID);
            if (own !== undefined) {
              deedsStaged += 1;
              messages.push({
                id: `m_write${String(i)}`,
                time: { created: stamp() },
                type: 'assistant',
                content: [toolPart('write', { filePath: own, content: 'my findings' })],
              });
              messages.push({
                id: `m_read${String(i)}`,
                time: { created: stamp() },
                type: 'assistant',
                content: [toolPart('read', { filePath: own })],
              });
              messages.push({
                id: `m_diff${String(i)}`,
                time: { created: stamp() },
                type: 'assistant',
                content: [toolPart('shell', { command: `git diff ${own}` })],
              });
              if (sibling) {
                const other = own.replace('security', 'correctness');
                messages.push({
                  id: `m_sib${String(i)}`,
                  time: { created: stamp() },
                  type: 'assistant',
                  content: [toolPart('read', { filePath: other })],
                });
              }
            }
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

  return {
    client,
    creates,
    prompts,
    deedsStaged: () => deedsStaged,
    ownReport: () => {
      const first = creates.find((c) => c.key === 'design-review:security');
      return first === undefined ? undefined : ownReportOf(first.id);
    },
  };
}

describe('the own-report seat round trip (ADR-016 Decision 1)', () => {
  // proves: Test strategy #11 — a design seat that writes its report and then
  // re-reads exactly that path (via `read` and via `git diff`) passes the audit
  // and the attempt: no retry session is launched, its one result records no
  // failure and the panel proceeds.
  it('lets the seat re-read its own report without burning the attempt', async () => {
    const dir = tmpRepo(baseState());
    const stub = seatStub({ sibling: false });

    const { status } = await resumePipeline(optionsFor(dir, stub.client));

    // The fixture guard: the prompt must actually have handed the seat a
    // report path, and the deeds must have been staged against it.
    const own = stub.ownReport();
    expect(own, 'the seat prompt names its reportPath').toBeDefined();
    expect(stub.deedsStaged(), 'the seat deeds were staged').toBeGreaterThan(0);

    const seatCreates = stub.creates.filter((c) => c.key === 'design-review:security');
    expect(seatCreates, 'the own-report seat is never retried').toHaveLength(1);

    const saved = savedState(dir);
    const seatResults = saved.results.filter((r) => r.stage === 'design-review:security');
    expect(seatResults).toHaveLength(1);
    expect(seatResults[0]?.failure, 'the attempt stayed clean').toBeUndefined();
    expect((seatResults[0]?.reply as { verdict?: string } | undefined)?.verdict).toBe('approve');
    expect(status).toBe('done');
  });

  // proves: Test strategy #11 — the same seat session with one sibling path
  // added fails the attempt, and the failure names the sibling; the exemption
  // is report-identity-scoped, and the seat's retry/permit semantics stay
  // design 007's: one fresh retry, every other seat launched once, the
  // panel proceeds.
  it('fails the seat that also opens a sibling report, naming the sibling', async () => {
    const dir = tmpRepo(baseState());
    const stub = seatStub({ sibling: true });

    const { status } = await resumePipeline(optionsFor(dir, stub.client));

    const own = stub.ownReport();
    expect(own, 'the seat prompt names its reportPath').toBeDefined();
    expect(stub.deedsStaged(), 'the seat deeds were staged').toBeGreaterThan(0);

    const seatCreates = stub.creates.filter((c) => c.key === 'design-review:security');
    expect(seatCreates, 'one retry, not more').toHaveLength(2);

    const saved = savedState(dir);
    const seatResults = saved.results.filter((r) => r.stage === 'design-review:security');
    expect(seatResults).toHaveLength(2);
    const failed = seatResults[0];
    expect(failed?.failure, 'attempt 1 fails on the sibling deed').toBeDefined();
    expect(failed?.failure).toContain('independence violation');
    expect(failed?.failure).toContain((own as string).replace('security', 'correctness'));
    expect((seatResults[1]?.reply as { verdict?: string } | undefined)?.verdict).toBe('approve');
    expect(status).toBe('done');
  });
});
