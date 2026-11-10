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
});

// ---------------------------------------------------------------------------
// The zero-descendant tripwire (design 007 §2): an empty-but-successful
// enumeration next to a completed delegation part must not masquerade as a
// clean audit — it warns. Observed through console.warn, the channel the
// design names for both walk failure modes.
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
 * `subagent` part, while the session tree holds no descendants at all:
 * enumeration succeeds and comes back empty. Message timestamps ride the
 * real clock so the audit's `since` filter keeps them visible.
 */
function delegatingClient() {
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
        const id = created === 1 ? 'ses_tripwire' : `ses_${String(created)}`;
        sessions.set(id, { prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (s) s.prompts += 1;
      },
      remove: async () => undefined,
      list: async () => ({
        data: [...sessions.keys()].map((id) => ({ id })),
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

describe('zero-descendant tripwire at the audit call site', () => {
  afterEach(() => vi.restoreAllMocks());

  // proves: Test strategy #5 — a completed `subagent` part with zero
  // enumerated descendants warns, naming the session that could not be
  // enumerated; the attempt itself is not failed — a blind audit is not a
  // violation, and the fence still guards the access (A10).
  it('warns when a delegated attempt enumerates no child sessions', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const outcome = await runner.executeStage(
      {
        repoDir: tmpRepo(),
        sourcePath: 'docs/adrs/002-x.md',
        promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
        client: delegatingClient(),
      },
      state(),
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

    expect(outcome.ok).toBe(true);
    const warns = warn.mock.calls.map((call) => call.join(' ')).join('\n');
    expect(warns).toContain('ses_tripwire');
  });
});
