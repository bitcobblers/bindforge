import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it, vi } from 'vitest';

import { templateHash } from '../../src/pipeline/prompts.js';
import {
  type ResumeOptions,
  type RunOptions,
  acquireLock,
  executeStage,
  latestTextReply,
  listSessionMessages,
  resumePipeline,
  statusLines,
} from '../../src/pipeline/runner.js';
import { DEFAULT_CONFIG, type PipelineConfig } from '../../src/pipeline/stages.js';
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

    const { status } = await resumePipeline(optionsFor(dir, { client }));
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

    const { status } = await resumePipeline(optionsFor(dir, { client, promptsDir }));
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

// ===========================================================================
// ADR-020 (design 020): the restart boundary. A runner commit window whose
// NET diff touches the executable surface (`tools/orchestration/src/**` or
// `tools/orchestration/prompts/**`) sets the `restartRequired` record, and the
// drive loop stops the run as a clean pause — gate entry, checkpoint record,
// `chore: <nnn>-<slug> restart boundary` commit — charging nothing. `resume`
// is the restart: it crosses the boundary and drives on with the committed
// tree already imported.
// ===========================================================================

interface RestartRecord {
  stage: string;
  paths: string[];
  at: number;
}
type StateWithBoundary = PipelineState & { restartRequired?: RestartRecord };
/** The widened run/resume result the CLI maps exit codes from (§3). */
interface DriveResult {
  status: 'running' | 'done' | 'escalated';
  restartRequired?: RestartRecord;
}
const asResult = (raw: unknown): DriveResult => raw as DriveResult;

const SRC_PROBE = join('tools', 'orchestration', 'src', 'pipeline', 'probe.ts');
const PROMPTS_PROBE = join('tools', 'orchestration', 'prompts', 'probe.md');
/** The boundary record's path cap: the same `MAX_GATE_LEGS` array bound the
 * ADR-016 gate evidence keeps (`gate-evidence-capture.test.ts` pins it at 32). */
const MAX_RECORD_PATHS = 32;
/** True when the string still carries a control byte `printable` must replace. */
const hasControlChar = (value: string): boolean => /\p{Cc}/u.test(value);

/** Dynamic: `finalizeRestartBoundary` is what the design adds beside
 * `finalizeDone`; a named static import would fail this file at load while
 * the feature is unimplemented. */
const runnerModule = (await import('../../src/pipeline/runner.js')) as {
  finalizeRestartBoundary?: (repoDir: string, state: PipelineState, cfg: PipelineConfig) => unknown;
};

function gitOut(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

/** Light-flow repo: brief with `flow: light`, committed checkpoint, and a
 * `check` gate the fixture controls (green by default). The light route's
 * `implement` is the smallest coder window. `checkScript` is the gate command
 * verbatim — the escalate-after-boundary fixture uses it to replace the
 * checkpoint path with a directory: the gate runs after the window finalized
 * and before the drive iteration's save, which is exactly the window the
 * ADR-006 §3 terminal-save guard has to cover. Keep the script free of double
 * quotes and backslashes: it is spliced into the `package.json` string. */
function lightRepo(state: PipelineState, checkScript = 'process.exit(0)'): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-boundary-'));
  mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'prompts', '002-x.md'), '---\nflow: light\n---\n\nBuild it.\n');
  mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
  writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), `${JSON.stringify(state, null, 2)}\n`);
  writeFileSync(
    join(dir, 'package.json'),
    `{"name":"t","scripts":{"check":"node -e \\"${checkScript}\\""}}\n`,
  );
  gitOut(dir, ['init', '-q', '-b', 'main']);
  gitOut(dir, ['add', '-A']);
  gitOut(dir, ['commit', '-q', '-m', 'docs: base']);
  return dir;
}

function lightBoundaryState(): PipelineState {
  return baseState({
    source: { kind: 'brief', path: 'docs/prompts/002-x.md' },
    status: 'escalated',
    lastStage: '',
    flow: 'light',
  });
}

interface BoundaryStubSpec {
  /** extra on-disk writes the implement session makes during its prompt */
  implementWrites?: (dir: string) => void;
  /** extra on-disk writes the plan session makes (out-of-ownership probes) */
  planWrites?: (dir: string) => void;
}

/** Light-flow stub: plan writes the plan note, implement the module plus the
 * fixture's probe writes, the two light seats approve. */
function boundaryStub(dir: string, spec: BoundaryStubSpec = {}) {
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  const created: Record<string, number> = {};
  const sessionsCreated: string[] = [];
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  let clock = 0;

  const replyFor = (key: string): string => {
    if (key.includes('-review:')) return approve;
    if (key === 'plan') return JSON.stringify({ summary: 'planned', commitMessage: 'docs: plan' });
    if (key === 'implement' || key === 'fix')
      return JSON.stringify({
        testsNeedUpdate: false,
        commitMessage: key === 'fix' ? 'fix: address the findings' : 'feat: implement it',
      });
    return JSON.stringify({ summary: 'ok', commitMessage: 'docs: ok' });
  };

  const planDoc = [
    '# Plan: 002-x',
    '',
    '## Problem',
    'missing',
    '',
    '## Chosen approach',
    'one module',
    '',
    '## Files to touch',
    '- src/thing.ts',
    '',
    '## Test plan',
    '- gate green',
    '',
    '## Done criteria',
    '- gate green',
    '',
  ].join('\n');

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created[key] = (created[key] ?? 0) + 1;
        sessionsCreated.push(key);
        const id = `ses_${String(Object.values(created).reduce((a, b) => a + b, 0))}`;
        sessions.set(id, { key, ordinal: created[key] - 1, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string; text?: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        if (s.key === 'plan') {
          writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), `${planDoc}\n`);
          spec.planWrites?.(dir);
        }
        if (s.key === 'implement') {
          mkdirSync(join(dir, 'src'), { recursive: true });
          writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n');
          writeFileSync(join(dir, DESIGN_DIR, '03-build-notes.md'), '# Build notes\n\nshipped\n');
          spec.implementWrites?.(dir);
        }
        if (s.key.includes('-review:')) {
          const found = /docs\/designs\/[^\s`]+\/reviews\/code\/[\w.-]+\.md/.exec(input.text ?? '');
          if (found !== null) {
            const path = join(dir, found[0]);
            mkdirSync(join(path, '..'), { recursive: true });
            writeFileSync(path, `report from ${s.key}\n`);
          }
        }
      },
      remove: async () => undefined,
      // The descendant walk's enumeration endpoint: no child sessions here.
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
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
            content: [{ type: 'text', text: replyFor(s.key) }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(clock)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, sessionsCreated };
}

describe('restart boundary: window classification', () => {
  // proves: Test strategy #3 — a finalizeStage window whose net diff touches a
  // stage template sets the record, naming the stage and the hit path.
  it('opens the boundary when an implement window adds a template', async () => {
    const dir = lightRepo(lightBoundaryState());
    const { client, sessionsCreated } = boundaryStub(dir, {
      implementWrites: (d) => {
        mkdirSync(join(d, 'tools', 'orchestration', 'prompts'), { recursive: true });
        writeFileSync(join(d, PROMPTS_PROBE), '# probe template\n');
      },
    });

    const result = asResult(await resumePipeline(optionsFor(dir, { client })));

    expect(result.status).toBe('running');
    expect(result.restartRequired).toBeDefined();
    expect(result.restartRequired?.stage).toBe('implement');
    expect(result.restartRequired?.paths).toContain(PROMPTS_PROBE);
    const saved = savedState(dir) as StateWithBoundary;
    expect(saved.status).toBe('running');
    expect(saved.restartRequired?.stage).toBe('implement');
    // The pause is the loop-top check refusing another window: no stage past
    // `implement` ran in this process.
    expect(sessionsCreated).toEqual(['plan', 'implement']);
  });

  // proves: Test strategy #3 — the same through a src file: the surface rule
  // is directory-scoped, so a module path under `tools/orchestration/src/`
  // hits it the moment it exists.
  it('opens the boundary when an implement window adds a src module', async () => {
    const dir = lightRepo(lightBoundaryState());
    const { client } = boundaryStub(dir, {
      implementWrites: (d) => {
        mkdirSync(join(d, 'tools', 'orchestration', 'src', 'pipeline'), { recursive: true });
        writeFileSync(join(d, SRC_PROBE), 'export const probe = 1;\n');
      },
    });

    const result = asResult(await resumePipeline(optionsFor(dir, { client })));

    expect(result.restartRequired).toBeDefined();
    expect(result.restartRequired?.stage).toBe('implement');
    expect(result.restartRequired?.paths).toContain(SRC_PROBE);
  });

  // proves: Test strategy #3 — the check runs at finalizeReviewPass too, on
  // the same net-diff rule: a commit landing inside the review window (here
  // an operator commit mid-stage, which the ownership sweep reports and
  // leaves alone) is inside the window's net diff, so the pass window opens
  // the boundary naming the pass stage and the hit path.
  it('opens the boundary when a review-pass window nets a surface path', async () => {
    const state = baseState({ lastStage: 'proposal' });
    const dir = tmpRepo(state);
    const { client } = approveClient(dir);
    const prompt = client.session.prompt;
    let probed = false;
    client.session.prompt = async (input: { sessionID: string }) => {
      if (!probed) {
        probed = true;
        mkdirSync(join(dir, 'tools', 'orchestration', 'src', 'pipeline'), { recursive: true });
        writeFileSync(join(dir, SRC_PROBE), 'export const probe = 1;\n');
        gitOut(dir, ['add', '-A']);
        gitOut(dir, ['commit', '-q', '-m', 'chore: operator commit lands mid-pass']);
      }
      return prompt(input);
    };

    const result = asResult(await resumePipeline(optionsFor(dir, { client })));

    expect(result.restartRequired).toBeDefined();
    expect(result.restartRequired?.stage).toBe('design-review');
    expect(result.restartRequired?.paths).toContain(SRC_PROBE);
    const saved = savedState(dir) as StateWithBoundary;
    expect(saved.gateLog[saved.gateLog.length - 1]?.gate).toBe('restart-boundary');
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
  });

  // proves: Test strategy #3 — an ordinary product run sees no boundary: a
  // window whose diff is `docs/**`-only (plus product src) never hits the
  // rule, and the run drives to done.
  it('leaves a docs-only window unclassified', async () => {
    const dir = lightRepo(lightBoundaryState());
    const { client } = boundaryStub(dir);

    const result = asResult(await resumePipeline(optionsFor(dir, { client })));

    expect(result.status).toBe('done');
    expect(result.restartRequired).toBeUndefined();
    const saved = savedState(dir) as StateWithBoundary;
    expect(saved.restartRequired).toBeUndefined();
    expect(saved.gateLog.filter((g) => /restart/i.test(g.gate))).toEqual([]);
  });

  // proves: Test strategy #3 (the net rule, A3/A4) — a surface-path write by a
  // stage that does not own it is fully reverted by the ownership sweep inside
  // the same window, so the window's NET diff has no surface path and opens
  // no boundary: a failed attempt to change the harness leaves nothing.
  it('does not open a boundary for a surface write the sweep fully reverted', async () => {
    const dir = lightRepo(lightBoundaryState());
    const { client } = boundaryStub(dir, {
      // `plan` is confined to docs/** (stagePathAllowed): this probe is swept.
      planWrites: (d) => {
        mkdirSync(join(d, 'tools', 'orchestration', 'prompts'), { recursive: true });
        writeFileSync(join(d, PROMPTS_PROBE), '# swept attempt at the harness\n');
      },
    });

    const result = asResult(await resumePipeline(optionsFor(dir, { client })));

    expect(result.status).toBe('done');
    expect(result.restartRequired).toBeUndefined();
    expect(existsSync(join(dir, PROMPTS_PROBE))).toBe(false); // the sweep removed it
    const saved = savedState(dir) as StateWithBoundary;
    expect(saved.restartRequired).toBeUndefined();
    expect(saved.gateLog.filter((g) => /restart/i.test(g.gate))).toEqual([]);
  });

  // proves: Test strategy #3 (§2's writer-side posture: `printable` copies,
  // bounded like `boundedGateEvidence`; security r1 #2/#3, testing r1 #3) — a
  // window whose surface hit is larger than the record cap keeps the bounded
  // shape: `MAX_RECORD_PATHS` real paths plus one elision line, every stored
  // copy of every path `printable`-sanitized (git's `-z` net-diff output hands
  // a control byte in a file name through raw), and every "N harness file(s)"
  // render — the gate entry, the status line, and the pause's printed driver
  // summary — counts the window's true total, not the marker-padded array.
  it('bounds a record-cap-sized window and keeps every path copy printable', async () => {
    // The control byte sits early in the name so the probe survives the cap:
    // git lists the net diff bytewise, and 0x07 precedes every printable byte,
    // so this is the first path the bounded record keeps.
    const probes: string[] = ['00-ctl\u0007.md'];
    for (let i = 0; i < MAX_RECORD_PATHS + 1; i += 1) {
      probes.push(`probe-${String(i).padStart(2, '0')}.md`);
    }
    const trueCount = new RegExp(`${String(probes.length)} harness file\\(s\\)`);
    const paddedCount = new RegExp(`${String(MAX_RECORD_PATHS + 1)} harness file\\(s\\)`);

    const dir = lightRepo(lightBoundaryState());
    const { client } = boundaryStub(dir, {
      implementWrites: (d) => {
        mkdirSync(join(d, 'tools', 'orchestration', 'prompts'), { recursive: true });
        for (const name of probes) {
          writeFileSync(join(d, 'tools', 'orchestration', 'prompts', name), '# probe template\n');
        }
      },
    });

    // The driver-facing summary line is captured too: it is the third copy of
    // the record's values (§2's summary, the pause's own §3 line).
    const printed: string[] = [];
    const logSpy = vi.spyOn(console, 'log').mockImplementation((...args: unknown[]) => {
      printed.push(args.join(' '));
      return undefined;
    });
    let result: DriveResult;
    try {
      result = asResult(await resumePipeline(optionsFor(dir, { client })));
    } finally {
      logSpy.mockRestore();
    }

    expect(result.status).toBe('running');
    const paths = result.restartRequired?.paths ?? [];
    const real = paths.filter((p) => !/elided/i.test(p));
    const notes = paths.filter((p) => /elided/i.test(p));
    expect(real).toHaveLength(MAX_RECORD_PATHS);
    expect(notes).toHaveLength(1);
    expect(paths.length).toBeLessThanOrEqual(MAX_RECORD_PATHS + 1);
    // Sanitized, not dropped: the control byte is gone and the name survives.
    for (const path of paths) {
      expect(hasControlChar(path), JSON.stringify(path)).toBe(false);
    }
    expect(real.some((p) => p.includes('00-ctl'))).toBe(true);

    // The count every copy renders is the window's total (34), never the
    // padded array's length (33).
    const saved = savedState(dir) as StateWithBoundary;
    const entry = saved.gateLog[saved.gateLog.length - 1];
    expect(entry?.gate).toBe('restart-boundary');
    // Every copy of a path stays printable: the gate entry interpolates the
    // record's paths, so the control byte must not survive into the run's
    // history (§2's writer-side posture, security r1 #2).
    expect(entry?.reason, JSON.stringify(entry)).toBeDefined();
    expect(hasControlChar(entry?.reason ?? ''), JSON.stringify(entry)).toBe(false);
    expect(entry?.reason).toMatch(trueCount);
    expect(entry?.reason).not.toMatch(paddedCount);
    const lines = statusLines(saved, false).filter((line) => /restart/i.test(line));
    expect(lines).toHaveLength(1);
    expect(hasControlChar(lines[0] ?? ''), JSON.stringify(lines)).toBe(false);
    expect(lines[0]).toMatch(trueCount);
    expect(lines[0]).not.toMatch(paddedCount);
    // The pause's own printed summary — §3's driver line — is the third copy:
    // it names the boundary, and nothing printed for the run carries the
    // marker-padded count or a control byte (compatibility r1 #3, security
    // r1 #2/#3).
    expect(printed.some((chunk) => /restart/i.test(chunk))).toBe(true);
    // Where the summary does render the count, it is the window's total.
    expect(printed.some((chunk) => trueCount.test(chunk))).toBe(true);
    for (const chunk of printed) {
      expect(chunk).not.toMatch(paddedCount);
      // Line by line: a multi-line log call's own `\n` separators are control
      // bytes, not the record's.
      for (const line of chunk.split('\n')) {
        expect(hasControlChar(line), JSON.stringify(line)).toBe(false);
      }
    }
  });
});

describe('finalizeRestartBoundary: the terminal pause', () => {
  const record: RestartRecord = { stage: 'implement', paths: [PROMPTS_PROBE], at: 3 };

  function boundaryStateFor(): { dir: string; state: PipelineState } {
    const state = baseState({ status: 'running', lastStage: 'implement' });
    return { dir: tmpRepo(state), state };
  }

  const callFinalize = async (dir: string, state: PipelineState): Promise<void> => {
    const finalize = runnerModule.finalizeRestartBoundary;
    if (typeof finalize !== 'function') {
      throw new Error('runner.js exports no finalizeRestartBoundary yet');
    }
    await finalize(dir, state, DEFAULT_CONFIG);
  };

  // proves: Test strategy #4 — the pause records itself: a `restart-boundary`
  // gate entry naming the stage and paths, a checkpoint save carrying the
  // record, and a terminal `chore: <nnn>-<slug> restart boundary` commit whose
  // body names the changed paths and which lands the record in the COMMITTED
  // checkpoint (save-before-commit), leaving the worktree clean.
  it('gate-logs, saves the record, and mints the boundary commit', async () => {
    const { dir, state } = boundaryStateFor();
    (state as StateWithBoundary).restartRequired = record;

    await callFinalize(dir, state);

    const saved = savedState(dir) as StateWithBoundary;
    expect(saved.gateLog[saved.gateLog.length - 1]?.gate).toBe('restart-boundary');
    expect(saved.gateLog[saved.gateLog.length - 1]?.reason).toContain('implement');
    expect(saved.gateLog[saved.gateLog.length - 1]?.reason).toContain(PROMPTS_PROBE);
    expect(saved.restartRequired).toEqual(record);
    expect(statusOfHead(dir)).toBe('chore: 002-x restart boundary');
    expect(gitOut(dir, ['show', '-s', '--format=%B'])).toContain(PROMPTS_PROBE);

    // The record is in the committed checkpoint: the pause is run history.
    const committed = JSON.parse(
      gitOut(dir, ['show', 'HEAD:docs/designs/002-x/.pipeline.json']),
    ) as StateWithBoundary;
    expect(committed.restartRequired).toEqual(record);

    expect(gitOut(dir, ['status', '--porcelain']).trim()).toBe('');
  });

  // proves: Test strategy #4 — nothing else distinguishes it from a normal
  // stage end: no `ESCALATION.md`, no `stageFailures` increment, no round,
  // budget or session charge moves (the next stage's charge lands once, in
  // the resumed process).
  it('charges no failure accounting and writes no escalation marker', async () => {
    const { dir, state } = boundaryStateFor();
    state.designBudgetUsed = 2;
    state.codeBudgetUsed = 1;
    (state as StateWithBoundary).restartRequired = record;
    const before = structuredClone(state);

    await callFinalize(dir, state);

    expect(state.status).toBe('running');
    expect(state.stageFailures).toEqual(before.stageFailures);
    expect(state.designRound).toBe(before.designRound);
    expect(state.codeRound).toBe(before.codeRound);
    expect(state.designBudgetUsed).toBe(before.designBudgetUsed);
    expect(state.codeBudgetUsed).toBe(before.codeBudgetUsed);
    expect(state.results).toHaveLength(before.results.length);
    expect(state.gateLog.filter((g) => g.gate === 'escalate')).toEqual([]);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
  });

  // proves: Test strategy #4 / §3 — the pause prints its one-line summary: the
  // stage that opened it, the changed paths, and the repair command.
  it('prints the driver summary naming the stage, paths, and resume command', async () => {
    const { dir, state } = boundaryStateFor();
    (state as StateWithBoundary).restartRequired = record;
    const printed: string[] = [];
    const logSpy = vi.spyOn(console, 'log').mockImplementation((...args: unknown[]) => {
      printed.push(args.join(' '));
      return undefined;
    });
    const errSpy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
      printed.push(args.join(' '));
      return undefined;
    });
    try {
      await callFinalize(dir, state);
    } finally {
      logSpy.mockRestore();
      errSpy.mockRestore();
    }
    const out = printed.join('\n');
    expect(out).toContain('implement');
    expect(out).toContain(PROMPTS_PROBE);
    expect(out).toContain('pipeline resume 002-x');
  });

  // proves: Test strategy #4 — the ADR-006 §3 terminal-save guard: a pause
  // that cannot be recorded must not masquerade as a clean stop. An injected
  // CheckpointWriteFailure (a directory at the checkpoint path) restores the
  // committed copy and escalates instead: no boundary commit, no record in
  // the committed checkpoint, the escalation record written.
  it('escalates instead of pausing when its checkpoint save fails', async () => {
    const { dir, state } = boundaryStateFor();
    (state as StateWithBoundary).restartRequired = record;
    // Replace the checkpoint file with a directory: every `saveState` rename
    // into the path fails as EISDIR — the CheckpointWriteFailure shape the
    // terminal-save guard is written for.
    rmSync(join(dir, DESIGN_DIR, '.pipeline.json'));
    mkdirSync(join(dir, DESIGN_DIR, '.pipeline.json'));

    await callFinalize(dir, state);

    expect(state.status).toBe('escalated');
    expect(state.gateLog[state.gateLog.length - 1]?.gate).toBe('escalate');
    expect(statusOfHead(dir)).not.toBe('chore: 002-x restart boundary');
    const committed = JSON.parse(
      gitOut(dir, ['show', 'HEAD:docs/designs/002-x/.pipeline.json']),
    ) as StateWithBoundary;
    expect(committed.restartRequired).toBeUndefined();
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// security r1 #1 / compatibility r1 #2: an escalation always outranks a pause.
// A boundary record set inside the same drive iteration must never ride into
// the escalated run's terminal record — the widened result is what the CLI
// maps its integer from, and `2` keeps its exact "read ESCALATION.md" meaning,
// so the documented `while [ $? -eq 3 ]` driver loop can never auto-resume an
// escalation past its own report.
// ---------------------------------------------------------------------------

describe('escalation after a surface-touching window', () => {
  // proves: Test strategy #4 (security r1 #1, compatibility r1 #2) — the save
  // after the finalized window fails: the gate command plants the
  // directory-at-checkpoint sabotage (the `checkpoint-integration.test.ts`
  // trick) because the gate runs after the window set the record and before
  // the iteration's save. The run ends escalated, not paused: the widened
  // result carries no record, so it maps to exit 2, the escalated checkpoint
  // carries no record, `ESCALATION.md` stands, and no boundary commit lands.
  // The same stub with a green gate and an untouched checkpoint pauses clean
  // (pinned above), so what this fixture changes is only the save.
  it('escalates with no restart record when the save after the window fails', async () => {
    // Linux-only fixture, like the rest of the suite (the matrix's Windows
    // row): the sabotage path is spelled with `/` because it runs as a shell
    // string inside the repo the gate executes in.
    const checkpointRel = 'docs/designs/002-x/.pipeline.json';
    const sabotage =
      `const fs=require('node:fs');` +
      `fs.rmSync('${checkpointRel}',{recursive:true,force:true});` +
      `fs.mkdirSync('${checkpointRel}',{recursive:true});`;
    const dir = lightRepo(lightBoundaryState(), sabotage);
    const { client } = boundaryStub(dir, {
      implementWrites: (d) => {
        mkdirSync(join(d, 'tools', 'orchestration', 'prompts'), { recursive: true });
        writeFileSync(join(d, PROMPTS_PROBE), '# probe template\n');
      },
    });

    const result = asResult(await resumePipeline(optionsFor(dir, { client })));

    expect(result.status).toBe('escalated');
    // Nothing stands for the pause: a run with a committed `ESCALATION.md`
    // never carries the record the CLI maps exit 3 from.
    expect(result.restartRequired).toBeUndefined();

    const reportPath = join(dir, DESIGN_DIR, 'ESCALATION.md');
    expect(existsSync(reportPath)).toBe(true);
    expect(readFileSync(reportPath, 'utf8')).toContain('checkpoint save failed');

    // The pause never finalized: no boundary commit anywhere in the history.
    expect(gitOut(dir, ['log', '--format=%s'])).not.toContain('restart boundary');

    // The escalated checkpoint the next resume reads is the ADR-006 §3 split
    // posture — the newest committed checkpoint restored, the terminal record
    // carried by the marker beside it — and it holds no stale pause record.
    const checkpoint = join(dir, DESIGN_DIR, '.pipeline.json');
    expect(statSync(checkpoint).isFile()).toBe(true);
    const restored = JSON.parse(readFileSync(checkpoint, 'utf8')) as StateWithBoundary;
    expect(restored.status).toBe('running');
    expect(restored.restartRequired).toBeUndefined();
    // The same record's absence in the driver-facing view: an escalated run's
    // status output carries no boundary line beside the escalation
    // (compatibility r1 #2).
    expect(statusLines(restored, false).join('\n')).not.toMatch(/restart/i);
  });
});

function statusOfHead(dir: string): string {
  return gitOut(dir, ['log', '--format=%s', '-1']).trim();
}

describe('resume across a standing restart boundary', () => {
  // proves: Test strategy #5 — a `resume` launched while the record stands
  // gate-logs the boundary it is crossing, clears the field, and drives on:
  // the next stage runs in the fresh process and the run completes.
  it('clears the record at entry and keeps driving', async () => {
    const dir = lightRepo(lightBoundaryState());
    const first = boundaryStub(dir, {
      implementWrites: (d) => {
        mkdirSync(join(d, 'tools', 'orchestration', 'prompts'), { recursive: true });
        writeFileSync(join(d, PROMPTS_PROBE), '# probe template\n');
      },
    });
    const r1 = asResult(await resumePipeline(optionsFor(dir, { client: first.client })));
    expect(r1.restartRequired).toBeDefined();

    const second = boundaryStub(dir);
    const r2 = asResult(await resumePipeline(optionsFor(dir, { client: second.client })));

    expect(r2.status).toBe('done');
    expect(r2.restartRequired).toBeUndefined();
    const saved = savedState(dir) as StateWithBoundary;
    expect(saved.status).toBe('done');
    expect(saved.restartRequired).toBeUndefined();
    // The pause entry and the crossing entry are both in the run's record
    // (mirror of the split-record posture resume already keeps).
    expect(saved.gateLog.filter((g) => /restart/i.test(g.gate)).length).toBeGreaterThanOrEqual(2);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
  });

  // proves: Test strategy #5 — `pipeline status` gains exactly one boundary
  // line while the record stands, built the way the plan-stop line is built:
  // the stage that opened it, how many harness files it changed, and the
  // resume posture. With no record, it prints nothing about one.
  it('statusLines prints the boundary line only while the record stands', () => {
    const paused = baseState({
      status: 'running',
      restartRequired: {
        stage: 'implement',
        paths: [PROMPTS_PROBE, SRC_PROBE],
        at: 1,
      },
    } as unknown as PipelineState);

    const lines = statusLines(paused, false).filter((line) => /restart/i.test(line));
    expect(lines).toHaveLength(1);
    expect(lines[0]).toContain('implement');
    // The count phrase, not a bare digit: the run id `002-x` satisfies `2`.
    expect(lines[0]).toContain('changed 2 harness file(s)');
    expect(lines[0]).toContain('pipeline resume 002-x');

    const plain = statusLines(baseState({ status: 'running' }), false).join('\n');
    expect(plain).not.toMatch(/restart/i);
  });

  // proves: Test strategy #5 (security r1 #2) — the record's values are
  // re-sanitized at every render, not only where this process wrote them: a
  // record a later-era runner or an operator planted in the checkpoint with
  // control bytes in its stage and paths still crosses as a printable gate-log
  // entry and a printable status line.
  it('renders a planted record printable at the crossing and the status line', async () => {
    const planted: RestartRecord = {
      stage: 'imple\u0007ment',
      paths: ['tools/orchestration/prompts/probe\u0007.md'],
      at: 7,
    };
    const state = lightBoundaryState() as StateWithBoundary;
    state.restartRequired = planted;
    const dir = lightRepo(state);
    const { client } = boundaryStub(dir);

    const result = asResult(await resumePipeline(optionsFor(dir, { client })));

    // The resume crosses the boundary and carries the run on.
    expect(result.status).toBe('done');
    expect(result.restartRequired).toBeUndefined();
    const saved = savedState(dir) as StateWithBoundary;
    expect(saved.restartRequired).toBeUndefined();
    const crossing = saved.gateLog.filter((entry) => entry.gate.includes('crossed'));
    expect(crossing).toHaveLength(1);
    for (const entry of saved.gateLog) {
      expect(hasControlChar(`${entry.gate} ${entry.reason}`), JSON.stringify(entry)).toBe(false);
    }
    // The same loaded value in the status line: sanitized, and still readable
    // as the pause it is.
    const rendered = statusLines(
      baseState({ status: 'running', restartRequired: planted } as unknown as PipelineState),
      false,
    ).join('\n');
    expect(rendered).toMatch(/restart boundary/i);
    // Each rendered line, not the joined block: the join's own `\n` is a
    // control byte, so a whole-block check can never be true.
    for (const line of rendered.split('\n')) {
      expect(hasControlChar(line), JSON.stringify(line)).toBe(false);
    }
    // Non-vacuous: the raw loaded value never reaches the terminal.
    expect(rendered).not.toContain(planted.stage);
  });
});

describe('the per-attempt template provenance', () => {
  // proves: Test strategy #8 (Decision 4) — a stage's transcript names the
  // template and the sha computed from the template text loaded at stage
  // entry, and attempt 2 of the same stage carries the identical sha: the
  // entry pin is the record each stage is judged by across a boundary. The
  // template is rewritten on disk between the attempts, so the identical sha
  // can only come from the entry-pinned text — a stage that re-reads its
  // template per attempt hashes the new text and fails here.
  it('carries the same template sha across attempts', async () => {
    const dir = tmpRepo(baseState());
    const promptsDir = mkdtempSync(join(tmpdir(), 'bindforge-provenance-'));
    const templateText = 'implement the plan at {{designDir}}\n';
    writeFileSync(join(promptsDir, 'implement.md'), templateText);

    const s = baseState();
    const { client } = approveClient(dir);
    const opts: RunOptions = {
      repoDir: dir,
      sourcePath: 'docs/adrs/002-x.md',
      promptsDir,
      client,
    };
    const handle = {
      stage: 'implement',
      stageKey: 'implement',
      attempt: 1,
      round: 0,
      specialty: undefined,
      vars: { designDir: 'docs/designs/002-x' },
    };

    const first = await executeStage(opts, s, handle, FAST);
    expect(first.ok).toBe(true);

    // The surface moves under the stage between its attempts — exactly what a
    // self-change run does across a boundary. The entry pin is what keeps
    // attempt 2 judged by the text attempt 1 was given.
    const rewritten = 'implement something else entirely at {{designDir}}\n';
    writeFileSync(join(promptsDir, 'implement.md'), rewritten);

    const second = await executeStage(
      opts,
      s,
      // The stage-entry-pinned text, handed down exactly as the drive loop
      // hands it to every attempt of the stage.
      { ...handle, attempt: 2, templateText } as unknown as typeof handle,
      FAST,
    );
    expect(second.ok).toBe(true);
    expect(s.results).toHaveLength(2);

    const expected = templateHash(templateText);
    const shas = s.results.map((r) => {
      const text = readFileSync(join(dir, r.transcript), 'utf8');
      // The provenance names the template path as the run displays it.
      const found = /- template: \S*implement\.md @ ([0-9a-f]{12})/.exec(text);
      expect(found, r.transcript).not.toBeNull();
      return found?.[1];
    });
    expect(shas[0]).toBe(expected);
    expect(shas[1]).toBe(expected);
    // Non-vacuous: the text on disk at attempt 2 hashes elsewhere, so the
    // identical sha above is the entry pin, not two readings of one file.
    expect(templateHash(rewritten)).not.toBe(expected);
  });
});
