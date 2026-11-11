import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import {
  type ResumeOptions,
  acquireLock,
  latestTextReply,
  listSessionMessages,
  resumePipeline,
  statusLines,
} from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// --------------------------------------------------------------------------
// The round-4 correctness incident: the server returns messages newest-first
// on a default 50-message page, but the drive loop assumed oldest-first and
// unpaged. A reviewer produced nine valid verdict replies; the runner read a
// stale narration turn 60 messages back and exhausted the nudge budget.
// --------------------------------------------------------------------------

function msg(created: number, type: string, extra: object = {}): SessionMessageInfo {
  return { id: `m_${String(created)}`, time: { created }, type, ...extra } as SessionMessageInfo;
}

function assistantWithText(created: number, text: string): SessionMessageInfo {
  return msg(created, 'assistant', { content: [{ type: 'text', text }] });
}

/** Stub server with the real endpoint's semantics: newest-first pages,
 * default page size 50, opaque cursor carrying the offset. */
function stubServer(stored: SessionMessageInfo[]) {
  const calls: Array<{ limit?: number; order?: string; cursor?: string }> = [];
  const client = {
    message: {
      list: async (input: { limit?: number; order?: string; cursor?: string }) => {
        calls.push(input);
        // The real cursor encodes the page order; following it keeps the
        // paging direction without an explicit `order` parameter.
        const order =
          input.cursor !== undefined ? input.cursor.split(':')[0] : (input.order ?? 'desc');
        const sorted =
          order === 'asc'
            ? [...stored].sort((a, b) => a.time.created - b.time.created)
            : [...stored].sort((a, b) => b.time.created - a.time.created);
        const offset = input.cursor === undefined ? 0 : Number(input.cursor.split(':')[1]);
        const size = input.limit ?? 50;
        const end = Math.min(offset + size, sorted.length);
        return {
          data: sorted.slice(offset, end),
          cursor: {
            previous: offset > 0 ? `${order}:${String(Math.max(0, offset - size))}` : null,
            next: end < sorted.length ? `${order}:${String(end)}` : null,
          },
        };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, calls };
}

describe('listSessionMessages', () => {
  it('pages newest-first servers fully, oldest-first', async () => {
    const stored = Array.from({ length: 450 }, (_, k) =>
      msg(k + 1, 'user', { text: `t${String(k)}` }),
    );
    const { client, calls } = stubServer(stored);

    const got = await listSessionMessages(client, 'ses_x');

    expect(got).toHaveLength(450);
    for (let k = 1; k < got.length; k++) {
      expect(got[k].time.created).toBeGreaterThan(got[k - 1].time.created);
    }
    expect(calls[0].order).toBe('asc');
  });

  it('reads past the default 50-message window', async () => {
    const stored = Array.from({ length: 63 }, (_, k) =>
      msg(k + 1, 'user', { text: `t${String(k)}` }),
    );
    const { client } = stubServer(stored);

    const got = await listSessionMessages(client, 'ses_x');

    // The raw default page the old code used would have lost 13 messages.
    expect(got).toHaveLength(63);
  });
});

describe('latestTextReply', () => {
  it('picks the newest text-carrying assistant, not the oldest', () => {
    const stored = [
      assistantWithText(1000, 'Let me verify the file tail and revision log.'),
      msg(1100, 'assistant', { content: [{ type: 'tool' }] }),
      assistantWithText(1200, '{"verdict":"request-changes","findings":[]}'),
      msg(1300, 'assistant', { content: [{ type: 'tool' }] }),
    ];
    const assistants = stored.filter((m) => m.type === 'assistant');

    const reply = latestTextReply(assistants);

    expect((reply?.content ?? []).map((p) => (p as { text?: string }).text).join('')).toBe(
      '{"verdict":"request-changes","findings":[]}',
    );
  });

  it('recovers the round-4 failure: compliant reply inside a long session', async () => {
    // 60 messages: early prose reply, much later the real JSON verdict —
    // exactly the shape the drive loop misread as "narration instead of reply".
    const stored: SessionMessageInfo[] = [
      assistantWithText(1000, 'All r3 finding carriers are present in the body.'),
      msg(1100, 'idle', { outcome: 'succeeded' }),
    ];
    for (let k = 0; k < 56; k++) {
      stored.push(msg(1200 + k * 10, 'assistant', { content: [{ type: 'tool' }] }));
    }
    stored.push(assistantWithText(2000, '{"verdict":"approve","findings":[]}'));
    stored.sort((a, b) => b.time.created - a.time.created); // as the server sends them

    const { client } = stubServer(stored);
    const all = await listSessionMessages(client, 'ses_x');
    const assistants = all.filter((m) => m.type === 'assistant');

    const reply = latestTextReply(assistants);
    const text = (reply?.content ?? [])
      .filter((p): p is { type: 'text'; text: string } => p.type === 'text')
      .map((p) => p.text)
      .join('');
    expect(text).toBe('{"verdict":"approve","findings":[]}');
  });
});

// --------------------------------------------------------------------------
// ADR-006: checkpoint trust and terminal-record guarantees. A failed
// terminal save leaves a split record — the checkpoint says running while
// ESCALATION.md says escalated — and status, resume, and the lock must all
// answer for it.
// --------------------------------------------------------------------------

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

const DESIGN_DIR = join('docs', 'designs', '002-x');

function baseState(partial: Partial<PipelineState> = {}): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'running',
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

/** Repo with a committed checkpoint, the design doc, and a green check gate. */
function tmpRepo(state: PipelineState): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-runner-adr006-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
  writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), '# proposal\n\nfirst body\n');
  // Green test gate without a real package: `npm run check` must exit 0.
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), `${JSON.stringify(state, null, 2)}\n`);
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** Stub client: every stage answers with a valid reply, so a run reaches done.
 * Turns grow the way a real session grows them — the read taken right after
 * the stage prompt is the baseline with no assistant turn, and the turn shows
 * up on a later poll — so the runner's reply contract sees a reply instead of
 * treating the stage as an empty one and ending the attempt at the deadline
 * (`stage-timeout`). A fixture that answered every read from the first would
 * exercise the retry/escalate path instead of the resume path pinned here.
 */
function approveClient(dir: string): { client: OpenCodeClient; created: string[] } {
  const created: string[] = [];
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  let clock = 0;
  let revisions = 0;

  const replyFor = (key: string): string => {
    if (key.endsWith('-review') || key.includes('-review:')) return approve;
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'proposal-revise')
      return JSON.stringify({ changed: true, summary: 'ok', commitMessage: 'docs: revise' });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 0, commitMessage: 'test: pin the contract' });
    if (key === 'implement')
      return JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok', commitMessage: 'docs: ok' });
  };

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created.push(key);
        const id = `ses_${String(created.length)}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        // A revise stage that answers `changed: true` leaves a new revision
        // stamp behind, as the funnel fixtures' stub does.
        if (s.key === 'proposal-revise') {
          revisions += 1;
          writeFileSync(
            join(dir, DESIGN_DIR, '02-proposal.md'),
            `# proposal\n\nrevision ${String(revisions)}\n`,
          );
        }
      },
      remove: async () => undefined,
      // The independence walk's enumeration endpoint: this fixture creates no
      // child sessions, so an empty page is the truth.
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { key: '', prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const text = replyFor(s.key);
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
  return { client, created };
}

function optionsFor(dir: string, extra: object = {}): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: FAST,
    ...extra,
  };
}

function savedState(dir: string): PipelineState {
  return JSON.parse(readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8')) as PipelineState;
}

describe('statusLines', () => {
  // proves: Test strategy #8 — the split record is visible: with the
  // checkpoint saying running and an ESCALATION.md marker on disk, status
  // prints a split-record line naming the marker (§3, SQLite's
  // journal-existence posture).
  it('shows the split-record line for a running checkpoint with an escalation marker', () => {
    const output = statusLines(baseState({ status: 'running' }), true).join('\n');
    expect(output).toMatch(/split record/i);
    expect(output).toContain('ESCALATION.md');
  });

  // proves: Test strategy #8 — the line is specific to the split: a running
  // checkpoint without the marker, and a terminal checkpoint with it, are
  // both rendered without one.
  it('says nothing about a split record when marker and checkpoint agree', () => {
    const running = statusLines(baseState({ status: 'running' }), false).join('\n');
    expect(running).not.toMatch(/split record/i);
    const terminal = statusLines(baseState({ status: 'escalated' }), true).join('\n');
    expect(terminal).not.toMatch(/split record/i);
  });
});

describe('resume with a split record', () => {
  // proves: Test strategy #9 — resumePipeline with an ESCALATION.md marker
  // and a running checkpoint appends a terminalSaveFailed gate-log entry
  // naming the marker's failure line before deleting the marker, so the new
  // run's gate log explains the gap it repaired.
  it('gate-logs terminalSaveFailed before deleting the marker', async () => {
    const dir = tmpRepo(baseState({ status: 'running' }));
    const marker = join(dir, DESIGN_DIR, 'ESCALATION.md');
    writeFileSync(
      marker,
      '# pipeline escalated\n\ncheckpoint save failed: EISDIR: illegal operation on a directory, rename\n',
      'utf8',
    );
    const { client } = approveClient(dir);

    const status = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('done');

    const saved = savedState(dir);
    const entries = saved.gateLog.filter((g) => g.gate === 'terminalSaveFailed');
    expect(entries).toHaveLength(1);
    // testing r1 #6: the reason quotes the marker's failure line exactly —
    // one `checkpoint save failed:` stem, not a doubled one (deviation 13).
    expect(entries[0].reason).toContain(
      'checkpoint save failed: EISDIR: illegal operation on a directory, rename',
    );
    // The gate-log entry survives even though the marker is gone: the
    // marker was read before it was deleted.
    expect(existsSync(marker)).toBe(false);
  });
});

describe('acquireLock', () => {
  // proves: Test strategy #10 — the run lock admits one pipeline process per
  // repo: a live holder blocks with today's fatal message, a dead holder is
  // cleared and the lock re-acquired, and the create itself is atomic and
  // exclusive — a refused acquisition never touches the holder's bytes
  // (ADR-006 §4, A6). Testing r1 #1: the previous `Promise.all` fixture could
  // not fail under the check-then-write implementation (a synchronous
  // acquire interleaves nothing in-process), so the exclusivity is pinned on
  // the primitive instead.
  async function tryAcquire(repoDir: string): Promise<string | null> {
    try {
      await acquireLock(repoDir);
      return null;
    } catch (error) {
      return error instanceof Error ? error.message : String(error);
    }
  }

  it('blocks a second runner while the lock holder is alive', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'bindforge-lock-live-'));
    writeFileSync(
      join(dir, '.pipeline.lock'),
      JSON.stringify({ slug: 'other', pid: process.pid, stage: 'start', at: 0 }),
      'utf8',
    );

    const message = await tryAcquire(dir);

    expect(message).not.toBeNull();
    // proves: Test strategy #10 / testing r1 #2 — the refusal is a
    // recognizable contract, not incidental prose: today's live-holder
    // message is fatal and names the holder's slug, stage, and pid
    // (ADR-006 §4 keeps it; the race-loser re-read throws the same shape).
    expect(message).toMatch(/^another pipeline run holds the lock: slug=\S+ stage=\S+ pid=\d+$/);
  });

  it('clears a dead holder and acquires', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'bindforge-lock-dead-'));
    // A pid no process can hold: liveness fails, the stale lock is cleared.
    writeFileSync(
      join(dir, '.pipeline.lock'),
      JSON.stringify({ slug: 'other', pid: 2_147_483_647, stage: 'start', at: 0 }),
      'utf8',
    );

    expect(await tryAcquire(dir)).toBeNull();
    expect(readFileSync(join(dir, '.pipeline.lock'), 'utf8')).toContain(String(process.pid));
  });

  it('creates the lock exclusively: a refused acquisition never clobbers the holder', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'bindforge-lock-race-'));
    const holder = JSON.stringify({ slug: 'other', pid: process.pid, stage: 'start', at: 0 });
    writeFileSync(join(dir, '.pipeline.lock'), holder, 'utf8');

    const message = await tryAcquire(dir);

    expect(message).not.toBeNull();
    expect(message).toMatch(/^another pipeline run holds the lock: slug=\S+ stage=\S+ pid=\d+$/);
    // A plain writeFileSync over the path would have replaced these bytes;
    // the `'wx'` exclusive create cannot. Two winners from the old
    // existsSync+writeFileSync race are what this makes impossible.
    expect(readFileSync(join(dir, '.pipeline.lock'), 'utf8')).toBe(holder);
  });
});
