import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { auditSessionIndependence } from '../../src/pipeline/independence.js';
import * as runner from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// Design 007 §2: the deeds of children. The runner walks the attempt
// session's descendant sessions (BFS over the parentID tree, visited-set
// guarded) and runs each descendant's message list through the same pure
// matcher — that is what makes exempting delegation TEXT safe. These unit
// tests drive the walk against fake clients only: no network, no server.
// ---------------------------------------------------------------------------

const NOW = 1_000_000;

function completedToolPart(name: string, input: Record<string, unknown>): SessionMessageInfo {
  return {
    id: 'm_t',
    time: { created: NOW },
    type: 'assistant',
    agent: 'a',
    model: {},
    content: [{ type: 'tool', id: 'p1', name, state: { status: 'completed', input, content: [] } }],
  } as unknown as SessionMessageInfo;
}

interface FakeSession {
  id: string;
  parentID?: string;
  messages: SessionMessageInfo[];
}

/**
 * Fake server answering BOTH enumeration routes the walk may take (A4):
 * `session.children` for one parent, and one paged `session.list` the walker
 * can index by `parentID`.
 */
function treeClient(sessions: FakeSession[]): OpenCodeClient {
  const byID = new Map(sessions.map((s) => [s.id, s]));
  return {
    session: {
      create: async () => ({ id: 'ses_unused' }),
      prompt: async () => undefined,
      remove: async () => undefined,
      children: async (input: { sessionID: string }) =>
        sessions.filter((s) => s.parentID === input.sessionID).map((s) => ({ id: s.id })),
      list: async () => ({
        data: sessions.map((s) => ({ id: s.id, parentID: s.parentID })),
        cursor: { previous: null, next: null },
      }),
    },
    message: {
      list: async (input: { sessionID: string }) => ({
        data: byID.get(input.sessionID)?.messages ?? [],
        cursor: { previous: null, next: null },
      }),
    },
  } as unknown as OpenCodeClient;
}

/**
 * Fake server answering ONLY the fallback enumeration route (A4): it carries
 * no `session.children` anywhere, so the walk has to index the `session.list`
 * pages by `parentID`. Pages come back `pageSize` entries at a time behind an
 * opaque `cursor.next` offset — the endpoint's real paging semantics.
 */
function listOnlyClient(sessions: FakeSession[], pageSize = 2) {
  const byID = new Map(sessions.map((s) => [s.id, s]));
  let listCalls = 0;
  let delivered = 0;
  const client = {
    session: {
      create: async () => ({ id: 'ses_unused' }),
      prompt: async () => undefined,
      remove: async () => undefined,
      list: async (input?: { limit?: number; cursor?: unknown }) => {
        listCalls += 1;
        const named = typeof input?.cursor === 'string' ? Number(input.cursor) : Number.NaN;
        const offset = Number.isNaN(named) ? delivered : named;
        const end = Math.min(offset + pageSize, sessions.length);
        delivered = end;
        return {
          data: sessions.slice(offset, end).map((s) => ({ id: s.id, parentID: s.parentID })),
          cursor: { previous: null, next: end < sessions.length ? String(end) : null },
        };
      },
    },
    message: {
      list: async (input: { sessionID: string }) => ({
        data: byID.get(input.sessionID)?.messages ?? [],
        cursor: { previous: null, next: null },
      }),
    },
  } as unknown as OpenCodeClient;
  return { client, listCalls: () => listCalls };
}

type ChildWalk = (
  client: OpenCodeClient,
  sessionID: string,
  requestTimeoutMs: number,
) => AsyncIterable<{ sessionID: string; messages: SessionMessageInfo[] }>;

/**
 * The walk is design 007 §2's new export; until it exists every test below
 * fails with this named reason instead of a bare import error.
 */
function requireWalk(): ChildWalk {
  const candidate = (runner as unknown as Record<string, unknown>)['listChildSessionMessages'];
  if (typeof candidate !== 'function') {
    throw new Error('design 007 §2: runner.ts does not export listChildSessionMessages');
  }
  return candidate as ChildWalk;
}

async function collect(walk: ChildWalk, client: OpenCodeClient, root: string) {
  const seen: Array<{ sessionID: string; messages: SessionMessageInfo[] }> = [];
  for await (const entry of walk(client, root, 1_000)) seen.push(entry);
  return seen;
}

describe('listChildSessionMessages', () => {
  // proves: Test strategy #5 — the walk is a BFS over the attempt session's
  // descendant sessions: a grandchild (depth 2) is reached, and the attempt
  // session itself is never among the yielded lists.
  it('reaches grandchild sessions two levels down', async () => {
    const client = treeClient([
      { id: 'ses_root', messages: [] },
      {
        id: 'ses_child',
        parentID: 'ses_root',
        messages: [completedToolPart('read', { filePath: 'docs/designs/007-x/transcripts/r0.md' })],
      },
      {
        id: 'ses_grandchild',
        parentID: 'ses_child',
        messages: [completedToolPart('glob', { pattern: 'docs/designs/007-x/reviews/**' })],
      },
    ]);
    const seen = await collect(requireWalk(), client, 'ses_root');
    expect(seen.map((e) => e.sessionID).sort()).toEqual(['ses_child', 'ses_grandchild']);
  });

  // proves: Test strategy #5 — a malformed parentID graph cannot loop: the
  // visited set terminates the walk and yields each descendant exactly once.
  it('terminates on a parentID cycle', async () => {
    const client = treeClient([
      { id: 'ses_a', parentID: 'ses_b', messages: [] },
      {
        id: 'ses_b',
        parentID: 'ses_a',
        messages: [completedToolPart('read', { filePath: 'docs/designs/007-x/transcripts/r0.md' })],
      },
    ]);
    const seen = await collect(requireWalk(), client, 'ses_a');
    const ids = seen.map((e) => e.sessionID);
    expect(ids).toContain('ses_b');
    expect(ids).not.toContain('ses_a');
    expect(new Set(ids).size).toBe(ids.length);
  });

  // proves: Test strategy #5 — `since` bounds what the matcher judges, not
  // what the walk fetches: a child list from before the attempt is walked,
  // and running it through the matcher with the attempt's `since` yields no
  // violation (the "don't pay twice" rule, A7).
  it('walks pre-since child messages without judging them', async () => {
    const client = treeClient([
      { id: 'ses_root', messages: [] },
      {
        id: 'ses_child',
        parentID: 'ses_root',
        messages: [completedToolPart('read', { filePath: 'docs/designs/007-x/transcripts/r0.md' })],
      },
    ]);
    const seen = await collect(requireWalk(), client, 'ses_root');
    expect(seen).toHaveLength(1);
    expect(seen[0]?.sessionID).toBe('ses_child');
    expect(seen[0]?.messages).toHaveLength(1);
    expect(
      auditSessionIndependence(seen[0]?.messages ?? [], { since: NOW + 1, reviewStage: false }),
    ).toEqual([]);
  });

  // proves: Test strategy #5 — the enumeration route real runs actually take.
  // The pinned SDK exposes no `session.children` (code review testing r1c0
  // finding 1), so the parentID index over `session.list`, not the children
  // method, is what carries the design's promise: the grandchild two levels
  // down is reached through it.
  it('reaches descendants through the parentID index on a list-only client', async () => {
    const sessions: FakeSession[] = [
      { id: 'ses_root', messages: [] },
      {
        id: 'ses_child',
        parentID: 'ses_root',
        messages: [completedToolPart('read', { filePath: 'docs/designs/007-x/transcripts/r0.md' })],
      },
      {
        id: 'ses_grandchild',
        parentID: 'ses_child',
        messages: [completedToolPart('glob', { pattern: 'docs/designs/007-x/reviews/**' })],
      },
    ];
    const fake = listOnlyClient(sessions, 10);
    const seen = await collect(requireWalk(), fake.client, 'ses_root');
    expect(seen.map((e) => e.sessionID).sort()).toEqual(['ses_child', 'ses_grandchild']);
  });

  // proves: Test strategy #5 — the walk's cost rule (design 007 §2): ONE
  // enumeration per walk, not one per node (code review performance r1c0
  // finding 3). A two-level tree on the list route costs a single
  // `session.list` call.
  it('enumerates the session list once per walk, not once per node', async () => {
    const sessions: FakeSession[] = [
      { id: 'ses_root', messages: [] },
      { id: 'ses_child', parentID: 'ses_root', messages: [] },
      { id: 'ses_grandchild', parentID: 'ses_child', messages: [] },
    ];
    const fake = listOnlyClient(sessions, 10);
    const seen = await collect(requireWalk(), fake.client, 'ses_root');
    expect(seen.map((e) => e.sessionID).sort()).toEqual(['ses_child', 'ses_grandchild']);
    expect(fake.listCalls()).toBe(1);
  });

  // proves: Test strategy #5 — paging: descendants recorded on a later page
  // are still found, because the enumeration follows `cursor.next` until the
  // server stops offering one (code review testing r1c0 finding 1).
  it('follows cursor.next across session-list pages', async () => {
    const sessions: FakeSession[] = [
      { id: 'ses_root', messages: [] },
      { id: 'ses_child', parentID: 'ses_root', messages: [] },
      { id: 'ses_sibling', parentID: 'ses_root', messages: [] },
      { id: 'ses_unrelated', parentID: 'ses_other_run', messages: [] },
      {
        id: 'ses_grandchild',
        parentID: 'ses_child',
        messages: [completedToolPart('read', { filePath: 'docs/designs/007-x/transcripts/r0.md' })],
      },
    ];
    const fake = listOnlyClient(sessions, 2);
    const seen = await collect(requireWalk(), fake.client, 'ses_root');
    const grandchild = seen.find((e) => e.sessionID === 'ses_grandchild');
    expect(grandchild?.messages).toHaveLength(1);
    expect(seen.map((e) => e.sessionID).sort()).toEqual([
      'ses_child',
      'ses_grandchild',
      'ses_sibling',
    ]);
    expect(fake.listCalls()).toBe(3);
  });
});

// ---------------------------------------------------------------------------
// The audit call site (design 007 §2): the walk runs only for an attempt that
// delegated; an empty-but-successful enumeration next to a completed
// delegation part must not masquerade as a clean audit — it warns; and a
// descendant fetch that fails degrades only the deeds the walk never saw.
// Both failure modes surface through console.warn, the channel the design
// names.
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
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-child-tripwire-'));
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

/**
 * Stub whose session replies with a valid verdict after a COMPLETED
 * `subagent` part (`delegate: false` replies without ever delegating), while
 * the session tree holds no descendants at all: enumeration succeeds and
 * comes back empty. Message timestamps ride the real clock so the audit's
 * `since` filter keeps them visible. `listCalls()` counts the `session.list`
 * calls, i.e. how many times the audit enumerated the session tree.
 * `loop: true` makes every page repeat its `cursor.next` — the shape of a
 * server whose pagination never advances.
 */
function delegatingClient(opts: { delegate?: boolean; loop?: boolean } = {}) {
  const { delegate = true, loop = false } = opts;
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const sessions = new Map<string, { prompts: number; lists: number }>();
  let clock = 0;
  let created = 0;
  let listCalls = 0;
  const stamp = () => {
    clock = Math.max(Date.now(), clock + 1);
    return clock;
  };
  const client = {
    session: {
      create: async () => {
        created += 1;
        const id = created === 1 ? 'ses_tripwire' : `ses_${String(created)}`;
        sessions.set(id, { prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (s) s.prompts += 1;
      },
      remove: async () => undefined,
      list: async () => {
        listCalls += 1;
        return {
          data: [...sessions.keys()].map((id) => ({ id })),
          cursor: { previous: null, next: loop ? '0' : null },
        };
      },
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return { data: [], cursor: { previous: null, next: null } };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          if (delegate) {
            messages.push({
              id: `m_del${String(i)}`,
              time: { created: stamp() },
              type: 'assistant',
              content: [
                {
                  type: 'tool',
                  id: 'p_del',
                  name: 'subagent',
                  state: {
                    status: 'completed',
                    input: { task: 'run the package tests and summarize', agent: 'explore' },
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
            content: [{ type: 'text', text: approve }],
          });
          messages.push({ id: `m_i${String(i)}`, time: { created: stamp() }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, listCalls: () => listCalls };
}

/**
 * Stub whose attempt session delegates once, and whose tree holds two child
 * sessions: the first one's own message list carries a completed `read` of a
 * run-records file, the second one's message list fails to fetch. So the walk
 * sees a deed and then loses the rest of the tree — the failure mode code
 * review security r1c0 finding 1 named.
 */
function partialWalkClient() {
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const sessions = new Map<string, { prompts: number; lists: number }>();
  let clock = 0;
  let created = 0;
  const stamp = () => {
    clock = Math.max(Date.now(), clock + 1);
    return clock;
  };
  const client = {
    session: {
      create: async () => {
        created += 1;
        const id = created === 1 ? 'ses_partial' : `ses_${String(created)}`;
        sessions.set(id, { prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (s) s.prompts += 1;
      },
      remove: async () => undefined,
      children: async (input: { sessionID: string }) =>
        input.sessionID === 'ses_partial'
          ? [{ id: 'ses_deed_child' }, { id: 'ses_bad_child' }]
          : [],
      list: async () => ({
        data: [...sessions.keys()].map((id) => ({ id })),
        cursor: { previous: null, next: null },
      }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        if (input.sessionID === 'ses_deed_child') {
          return {
            data: [
              {
                id: 'm_deed',
                time: { created: stamp() },
                type: 'assistant',
                content: [
                  {
                    type: 'tool',
                    id: 'p_deed',
                    name: 'read',
                    state: {
                      status: 'completed',
                      input: { filePath: 'docs/designs/002-x/transcripts/proposal-r0.md' },
                      content: [],
                    },
                  },
                ],
              },
            ],
            cursor: { previous: null, next: null },
          };
        }
        if (input.sessionID === 'ses_bad_child') {
          throw new Error('simulated failure mid-walk');
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
                id: 'p_del',
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
            content: [{ type: 'text', text: approve }],
          });
          messages.push({ id: `m_i${String(i)}`, time: { created: stamp() }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return client;
}

/** The seat handle the two call-site suites below drive. */
const SEAT_HANDLE = {
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

function seatOptions(client: OpenCodeClient) {
  return {
    repoDir: tmpRepo(),
    sourcePath: 'docs/adrs/002-x.md',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    client,
  };
}

async function runSeat(client: OpenCodeClient) {
  return runner.executeStage(seatOptions(client), state(), SEAT_HANDLE, FAST);
}

describe('the walk is keyed to delegation at the audit call site', () => {
  afterEach(() => vi.restoreAllMocks());

  // proves: Test strategy #5 — a completed `subagent` part with zero
  // enumerated descendants warns, naming the session that could not be
  // enumerated; the attempt itself is not failed — a blind audit is not a
  // violation, and the fence still guards the access (A10).
  it('warns when a delegated attempt enumerates no child sessions', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const fake = delegatingClient();
    const outcome = await runSeat(fake.client);

    expect(outcome.ok).toBe(true);
    const warns = warn.mock.calls.map((call) => call.join(' ')).join('\n');
    expect(warns).toContain('ses_tripwire');
  });

  // proves: Test strategy #5 — the walk costs an attempt nothing it did not
  // delegate: an attempt whose message list holds no `subagent` part never
  // enumerates the session tree at all (code review performance r1c0
  // finding 1), which is what keeps every clean non-delegating attempt free
  // of the walk's requests.
  it('enumerates no session tree for an attempt that delegated nothing', async () => {
    const fake = delegatingClient({ delegate: false });
    const outcome = await runSeat(fake.client);

    expect(outcome.ok).toBe(true);
    expect(fake.listCalls()).toBe(0);
  });

  // proves: Test strategy #5 — a session-list crawl bounds itself: a server
  // that answers every page with the same `cursor.next` cannot hang the
  // attempt in the post-reply audit. The walk gives up, the partial audit is
  // logged, and the attempt is judged on its own message list alone (code
  // review performance r1c0 finding 3, correctness r1c0 finding 2).
  it('degrades to the partial audit when the session-list cursor never advances', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const fake = delegatingClient({ loop: true });
    const outcome = await runSeat(fake.client);

    expect(outcome.ok).toBe(true);
    // The crawl stops asking instead of paging forever: against this in-memory
    // stub an unbounded crawl would reach thousands of calls inside the 5 s
    // stage window, a bounded one a handful.
    expect(fake.listCalls()).toBeLessThan(100);
    const warns = warn.mock.calls.map((call) => call.join(' ')).join('\n');
    expect(warns).toMatch(/ses_\w+/);
  });
});

// ---------------------------------------------------------------------------
// The guard that keys the walk to delegation must key on the DELEGATION, not
// on a completed in-window part (code review performance r1c0 finding 1): a
// resumed attempt keeps its earlier `subagent` part outside the audit window
// while that child's own deeds sit inside it. Skipping the walk because the
// part is out of window would hide the deed the text exemption traded away.
// ---------------------------------------------------------------------------

/**
 * Stub in that shape: the attempt session's `subagent` part is stamped long
 * before the attempt began, and the one child session it delegated to holds an
 * in-window `read` of a run-records file.
 */
function staleDelegationClient(): OpenCodeClient {
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const sessions = new Map<string, { prompts: number; lists: number }>();
  let clock = 0;
  let created = 0;
  const stale = Date.now() - 600_000;
  const stamp = () => {
    clock = Math.max(Date.now(), clock + 1);
    return clock;
  };
  return {
    session: {
      create: async () => {
        created += 1;
        const id = created === 1 ? 'ses_stale' : `ses_${String(created)}`;
        sessions.set(id, { prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (s) s.prompts += 1;
      },
      remove: async () => undefined,
      children: async (input: { sessionID: string }) =>
        input.sessionID === 'ses_stale' ? [{ id: 'ses_stale_child' }] : [],
    },
    message: {
      list: async (input: { sessionID: string }) => {
        if (input.sessionID === 'ses_stale_child') {
          return {
            data: [
              {
                id: 'm_stale_deed',
                time: { created: stamp() },
                type: 'assistant',
                content: [
                  {
                    type: 'tool',
                    id: 'p_stale_deed',
                    name: 'read',
                    state: {
                      status: 'completed',
                      input: { filePath: 'docs/designs/002-x/transcripts/proposal-r0.md' },
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
            time: { created: stale },
            type: 'assistant',
            content: [
              {
                type: 'tool',
                id: 'p_del',
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
            content: [{ type: 'text', text: approve }],
          });
          messages.push({ id: `m_i${String(i)}`, time: { created: stamp() }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
}

describe('a resumed attempt still walks the tree it delegated', () => {
  // proves: Test strategy #6 — the child deed still fails the attempt when the
  // parent's delegation part predates the attempt window: the walk is keyed to
  // the delegation, not to an in-window completed part, so the exemption cannot
  // hide the deed behind an older part.
  it('fails the attempt on a child deed behind an out-of-window delegation', async () => {
    const outcome = await runSeat(staleDelegationClient());

    expect(outcome.ok).toBe(false);
    const failure = outcome.failure ?? '';
    expect(failure).toContain('independence violation');
    expect(failure).toContain('transcripts');
    expect(failure).toContain('(via child session');
    expect(failure).toContain('ses_stale_child');
  });
});

describe('a failed descendant fetch erases only the deeds it never saw', () => {
  afterEach(() => vi.restoreAllMocks());

  // proves: Test strategy #6 — the partial-audit path degrades the unseen
  // remainder of the walk, never evidence in hand: the first child's
  // transcripts read is still a violation that fails the attempt even though
  // a later descendant's message list throws, and the incomplete walk is
  // logged naming the session that could not be fetched (code review
  // security r1c0 finding 1).
  it('fails the attempt on a deed seen before a later child fetch throws', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const outcome = await runSeat(partialWalkClient());

    expect(outcome.ok).toBe(false);
    const failure = outcome.failure ?? '';
    expect(failure).toContain('independence violation');
    expect(failure).toContain('transcripts');
    expect(failure).toContain('(via child session');

    const warns = warn.mock.calls.map((call) => call.join(' ')).join('\n');
    expect(warns).toContain('ses_bad_child');
  });
});
