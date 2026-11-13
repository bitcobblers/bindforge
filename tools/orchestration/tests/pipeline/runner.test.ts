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
import { loadState } from '../../src/pipeline/state-store.js';
import type { PipelineState, StageResult } from '../../src/pipeline/types.js';

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

// ---------------------------------------------------------------------------
// Run 010, Test strategy #10: status prints one flow line, rendered from the
// same `configFor(flowOf(state))` the runner drives with — so the numbers an
// operator reads for a light run are the effective light numbers, not the
// full-flow defaults the checkpoint never carried. A full run gains that one
// line and nothing else: every pre-existing line stays byte-equal, and an
// explicit `flow: full` renders identically to an absent key.
// ---------------------------------------------------------------------------

describe('statusLines: the flow line', () => {
  const linesOf = (state: PipelineState): string[] => statusLines(state, false);
  const flowLines = (state: PipelineState): string[] =>
    linesOf(state).filter((line) => line.trimStart().startsWith('flow:'));
  const SEAT_CYCLE = ['correctness', 'testing'];

  /** `n` code-seat sessions on the ledger — what `reviewSessionsUsed` counts. */
  const reviewResults = (n: number): StageResult[] =>
    Array.from({ length: n }, (_, i) => {
      const seat = SEAT_CYCLE[i % SEAT_CYCLE.length] ?? 'correctness';
      return {
        stage: `code-review:${seat}`,
        round: 1,
        sessionID: `ses_review_${String(i)}`,
        agent: `codereview/code-${seat}`,
        at: i,
        reply: { verdict: 'approve', findings: [] },
        transcript: 't',
      };
    });

  // proves: Test strategy #10 — the light line names the flow, its stage path,
  // its two seats, and the effective rounds and code-review budget.
  it('prints the light flow line with the effective numbers', () => {
    const output = linesOf(baseState({ flow: 'light' })).join('\n');
    expect(output).toContain(
      'light (plan → implement → code review, seats: correctness, testing; rounds 2, code reviews 6)',
    );
    expect(flowLines(baseState({ flow: 'light' }))).toHaveLength(1);
  });

  // proves: Test strategy #10 — a full run gains the one line, naming the flow
  // and the standing five-seat panel, and the rest of the block is untouched.
  it('adds one flow line to a full run', () => {
    const full = linesOf(baseState({}));
    expect(flowLines(baseState({}))).toHaveLength(1);
    expect(flowLines(baseState({}))[0]).toMatch(/flow:\s+full/);
    expect(full.join('\n')).toContain('compatibility');
    const rest = full.filter((line) => !flowLines(baseState({})).includes(line));
    expect(rest.some((line) => line.includes('slug:'))).toBe(true);
    expect(rest.some((line) => line.includes('pipeline/002-x'))).toBe(true);
    expect(rest.join('\n')).not.toMatch(/flow:/);
  });

  // proves: Test strategy #10 (the registry's `path` field) — the full flow's
  // line names the full flow's own stages: `plan` is a stage of the light flow
  // in this release, so it never reads as a step of `full` in `pipeline status`
  // or ESCALATION.md.
  it('names no light-only stage in the full flow path', () => {
    const line = flowLines(baseState({}))[0] ?? '';
    expect(line).toContain('research');
    expect(line).not.toMatch(/\bplan\b/);
  });

  // proves: Test strategy #10 (A3's additive shape) — an explicit full renders
  // byte-equal to an absent key, and the light key is the only difference.
  it('renders an explicit full exactly as an absent key', () => {
    expect(linesOf(baseState({ flow: 'full' }))).toEqual(linesOf(baseState({})));
    expect(linesOf(baseState({ flow: 'light' })).join('\n')).not.toEqual(
      linesOf(baseState({})).join('\n'),
    );
  });

  // proves: Test strategy #10 (the light `maxRounds` consumer, correctness r1
  // finding 1) — the printed unit follows the record. `codeBudgetUsed` counts
  // mutating rounds in BOTH flows, so the light budget line says `rounds` exactly
  // as the full-flow line always did, and the session count the line carries is
  // `reviewSessionsUsed(state, 'code')` — the number `maxReviews.code` caps.
  // Printing the round counter under `sessions` put the same number on the line
  // twice under two labels, which is the record an operator misread as cost.
  it('prints the budget line with the unit the number counts', () => {
    const budgetLine = (state: PipelineState): string =>
      linesOf(state)
        .filter((line) => line.includes('budgets'))
        .join('\n');

    const lightTwo = budgetLine(baseState({ flow: 'light', codeRound: 1, codeBudgetUsed: 2 }));
    expect(lightTwo).toContain('code 2 rounds');
    expect(lightTwo).not.toContain('code 2 sessions');
    expect(budgetLine(baseState({ codeRound: 1, codeBudgetUsed: 2 }))).toContain('code 2 rounds');

    // Four review sessions spent of the light 6: the session count rides on the
    // line's review count, where the phase cap sits.
    const reviewed = budgetLine(
      baseState({ flow: 'light', codeRound: 1, codeBudgetUsed: 2, results: reviewResults(4) }),
    );
    expect(reviewed).toContain('code 2 rounds');
    expect(reviewed).toContain('code 4 sessions');
  });
});

// ---------------------------------------------------------------------------
// Run 010, correctness r1c3 finding 1: `pipeline status` prints the plan stop
// from the stop the gate recorded (`state.planStop`), never from the stage a
// plan stop happens to leave. A light run in the window after a plan that
// cleared the cap, and a light `implement` whose failure left `lastStage` on
// `plan`, are both runs that are not stopped — and a flow whose whole point is
// that the operator reads the run's effective standing cannot be told a plan
// was refused when none was.
// ---------------------------------------------------------------------------

describe('statusLines: the plan stop', () => {
  const PLAN_STOP =
    'plan outgrew the light flow (120 lines, cap 80): set flow: full and re-run with --force';

  const planStopLines = (state: PipelineState): string[] =>
    statusLines(state, false).filter((line) => /plan stop/i.test(line));

  /** The line in either rendering: the stop is checkpoint data, so no
   * verbosity (marker on disk) is needed to print it, and it must not appear
   * in one and hide in the other. */
  const claimsAPlanStop = (state: PipelineState): boolean =>
    [false, true].some((verbose) => statusLines(state, verbose).join('\n').includes('plan stop'));

  // proves: Test strategy #10/#19 (correctness r1c3 finding 1) — a running
  // light run at the plan stage with no stop recorded prints nothing about a
  // plan stop: it is mid-flight into `implement`, not stopped.
  it('says nothing about a plan stop for a running light run', () => {
    expect(claimsAPlanStop(baseState({ flow: 'light', lastStage: 'plan' }))).toBe(false);
  });

  // proves: Test strategy #19 (correctness r1c3 finding 1) — the escalated
  // shape a failed light `implement` leaves (the plan stage's own value in
  // `lastStage`, no stop on the state) is not reported as a plan stop, and a
  // full run never is: the plan gate is light-only, its cap absent.
  it('says nothing about a plan stop that was never recorded', () => {
    expect(
      claimsAPlanStop(baseState({ flow: 'light', lastStage: 'plan', status: 'escalated' })),
    ).toBe(false);
    expect(claimsAPlanStop(baseState({ lastStage: 'plan', status: 'escalated' }))).toBe(false);
    expect(
      claimsAPlanStop(baseState({ flow: 'light', lastStage: 'implement', status: 'escalated' })),
    ).toBe(false);
  });

  // proves: Test strategy #19/#10 (correctness r1c3 finding 1) — the stop that
  // is recorded is printed, once, naming the refusal the gate recorded and the
  // two moves that respect it: the promotion and the in-flow re-plan. The plain
  // resume the run answers with no stop is the move a plan stop must not offer.
  it('prints the recorded plan stop with the moves that respect it', () => {
    const lines = planStopLines(
      baseState({ flow: 'light', lastStage: 'plan', status: 'escalated', planStop: PLAN_STOP }),
    );
    expect(lines).toHaveLength(1);
    const line = lines[0] ?? '';
    expect(line).toContain('plan outgrew the light flow (120 lines, cap 80)');
    expect(line).toContain('--force');
    expect(line).toContain('pipeline resume --stage plan 002-x');
    expect(line).not.toContain('pipeline resume 002-x');
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

describe('a runner error inside the drive loop', () => {
  // proves: ADR-006 §3's posture for the failure the section does not name.
  // Run 010's design review died on a template render over model-authored
  // findings; the throw escaped `drive`, the lock released, the checkpoint kept
  // saying `running`, and no ESCALATION.md was minted — the split record §3
  // exists to prevent. An unhandled error has to terminate the run the way
  // every other §3 failure does: gate-log entry, marker, terminal status.
  it('escalates with a record instead of crashing with the checkpoint running', async () => {
    const dir = tmpRepo(baseState({ lastStage: 'proposal', designRound: 0 }));
    const promptsDir = mkdtempSync(join(tmpdir(), 'bindforge-unrenderable-template-'));
    writeFileSync(join(promptsDir, 'design-review.md'), 'review {{designDir}} for {{noSuchVar}}\n');
    const { client } = approveClient(dir);

    const status = await resumePipeline(optionsFor(dir, { client, promptsDir }));
    expect(status).toBe('escalated');

    const saved = savedState(dir);
    expect(saved.status).toBe('escalated');
    expect(saved.gateLog.at(-1)?.gate).toBe('escalate');
    expect(saved.gateLog.at(-1)?.reason).toMatch(/unhandled pipeline error/);
    // The run is resumable from here: the checkpoint is the terminal record,
    // and the marker names the stop.
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(true);
    expect(await loadState(join(dir, DESIGN_DIR))).not.toBeNull();
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
