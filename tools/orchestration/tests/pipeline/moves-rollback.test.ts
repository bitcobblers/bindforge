import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it, vi } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import { type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-005, the terminal failure class (Test strategy #6): "A rollback rename
// that itself fails is the one state nothing in-process can undo:
// `moves-rollback-failed: <error>` escalates to the operator instead of
// starting attempt 2 on a known-damaged worktree." `retryInSameSession(...) ===
// false` is true of both moves failure classes, so it cannot tell a fresh
// attempt 2 from no attempt 2 at all — these suites drive the real driver and
// count what it actually does: for the terminal class, exactly one attempt, one
// session, one stage-failure gate entry, one code-budget unit, and an escalation
// naming the class, with the damaged worktree left for the operator and never
// committed as a normal stage result.
//
// The fault is injected at the filesystem boundary: pair 1's forward rename
// lands, pair 2 fails at rename(2) with ENOTDIR (its to-parent `src/gamma.ts`
// is a regular file, so it passes rules 4-7 at validation), and the armed
// reverse undo of pair 1 throws. The second suite runs the same fixture with
// the fault disarmed, where the undo succeeds: that is the environmental
// `moves-apply-failed` class, which the proposal keeps on today's retry, and
// counting its attempts is what makes the terminal class above a claim rather
// than a tautology.
// ---------------------------------------------------------------------------

const fsFault = vi.hoisted(() => ({ armed: false, undoAttempts: 0 }));

vi.mock('node:fs', async (importOriginal) => {
  const actual = await importOriginal<typeof import('node:fs')>();
  return {
    ...actual,
    renameSync: (...args: Parameters<typeof actual.renameSync>) => {
      const from = String(args[0]);
      const to = String(args[1]);
      // Only the undo of the landed pair (`src/moved.ts` back to `src/old.ts`)
      // is armed, and only while the terminal-class suite arms it. The forward
      // rename, the checkpoint's own tmp+rename, and every other rename in the
      // run still go to the real filesystem.
      if (fsFault.armed && from.endsWith('src/moved.ts') && to.endsWith('src/old.ts')) {
        fsFault.undoAttempts += 1;
        throw Object.assign(new Error(`EIO: injected rollback failure ${from}`), { code: 'EIO' });
      }
      return actual.renameSync(...args);
    },
  };
});

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 4_000,
  requestTimeoutMs: 1_000,
};

const APPROVE = JSON.stringify({ verdict: 'approve', findings: [] });
const MOVES_REPLY = JSON.stringify({
  testsNeedUpdate: false,
  commitMessage: 'fix: land the moves',
  moves: [
    { from: 'src/old.ts', to: 'src/moved.ts' },
    { from: 'src/beta.ts', to: 'src/gamma.ts/nested.ts' },
  ],
});

/** A run whose code budget has one entry left, forced straight into `fix`. */
function state(): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'escalated',
    lastStage: 'code-review',
    designRound: 5,
    codeRound: 1,
    designBudgetUsed: 5,
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

/** Repo with the tracked source files the injected apply failure needs. */
function tmpRepo(s: PipelineState): { dir: string; base: string } {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-rollback-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
      cwd: dir,
      encoding: 'utf8',
    });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  const designDir = join(dir, 'docs', 'designs', '002-x');
  mkdirSync(designDir, { recursive: true });
  mkdirSync(join(dir, 'src'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  writeFileSync(join(designDir, '02-proposal.md'), '# proposal\n\nbody\n');
  writeFileSync(join(designDir, '.pipeline.json'), `${JSON.stringify(s, null, 2)}\n`);
  writeFileSync(join(dir, 'src', 'old.ts'), 'one\n');
  writeFileSync(join(dir, 'src', 'beta.ts'), 'two\n');
  writeFileSync(join(dir, 'src', 'gamma.ts'), 'gamma\n');
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return { dir, base: git(['rev-parse', 'HEAD']).trim() };
}

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

function subjects(dir: string): string[] {
  return gitAt(dir, ['log', '--reverse', '--format=%s']).trim().split('\n');
}

function savedState(dir: string): PipelineState {
  const text = readFileSync(join(dir, 'docs', 'designs', '002-x', '.pipeline.json'), 'utf8');
  return JSON.parse(text) as PipelineState;
}

/**
 * Stub whose `fix` session replies with the moves list; every turn becomes
 * visible one poll after the prompt that asked for it, the same visibility the
 * other pipeline suites use.
 */
function replyClient(created: string[]): OpenCodeClient {
  const sessions = new Map<string, { prompts: number; lists: number; reply: string }>();
  let clock = 0;
  return {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created.push(key);
        const id = `ses_stage_${String(created.length)}`;
        sessions.set(id, { prompts: 0, lists: 0, reply: key === 'fix' ? MOVES_REPLY : APPROVE });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (s) s.prompts += 1;
      },
      remove: async () => undefined,
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { prompts: 0, lists: 0, reply: '{}' };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          clock += 1;
          messages.push({
            id: `m_a${String(i)}`,
            time: { created: clock },
            type: 'assistant',
            content: [{ type: 'text', text: s.reply }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(i)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
}

describe('a rollback rename that itself fails stops the run at the operator', () => {
  // proves: Test strategy #6 — a `moves-rollback-failed` outcome escalates to the
  // operator and starts no attempt 2: one stage result, one session for that
  // stage, one stageFailure gate entry, one code-budget unit, and an escalation
  // naming the class; and the half-applied worktree is left as the operator's
  // decision rather than committed as a normal stage result.
  it('escalates the rollback failure without starting a second attempt', async () => {
    const { dir } = tmpRepo(state());
    const created: string[] = [];
    const options: ResumeOptions = {
      repoDir: dir,
      slug: '002-x',
      promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
      cfg: FAST,
      client: replyClient(created),
      forceStage: 'fix',
    };

    const errors: string[] = [];
    const spy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
      errors.push(args.join(' '));
      return undefined;
    });
    fsFault.armed = true;
    const { status } = await resumePipeline(options).finally(() => {
      fsFault.armed = false;
    });
    spy.mockRestore();

    expect(status).toBe('escalated');
    expect(fsFault.undoAttempts, 'the injected undo failure really ran').toBe(1);

    const saved = savedState(dir);
    // Exactly one attempt ran: the terminal class never re-enters the stage, so
    // no fresh session is ever pointed at the known-damaged worktree.
    expect(saved.results.filter((r) => r.stage === 'fix')).toHaveLength(1);
    expect(created.filter((k) => k === 'fix')).toHaveLength(1);
    // One `fix` entry burned exactly one code-budget unit (A3), and the terminal
    // decision spent nothing further.
    expect(saved.codeBudgetUsed).toBe(1);
    // One stage-failure entry, and the escalate reason the operator sees names
    // the class: the run stops where the damage is, with the reason.
    expect(saved.gateLog.filter((g) => g.gate === 'stageFailure')).toHaveLength(1);
    const fixFailure = saved.results.filter((r) => r.stage === 'fix')[0]?.failure ?? '';
    expect(fixFailure.startsWith('moves-rollback-failed')).toBe(true);
    expect(errors.join('\n')).toContain('pipeline ESCALATED');
    expect(errors.join('\n')).toContain('moves-rollback-failed');

    // The half-applied list survives exactly as the operator must find it: pair 1
    // is still moved, the failing pair left nothing behind. Both endpoints are
    // in-ownership for `fix`, so the escalation sweep leaves them alone.
    expect(existsSync(join(dir, 'src', 'moved.ts'))).toBe(true);
    expect(readFileSync(join(dir, 'src', 'moved.ts'), 'utf8')).toBe('one\n');
    expect(existsSync(join(dir, 'src', 'old.ts'))).toBe(false);
    expect(existsSync(join(dir, 'src', 'gamma.ts', 'nested.ts'))).toBe(false);

    // No stage commit: the only history past the base is the terminal record.
    expect(subjects(dir)).toEqual(['docs: base', 'docs: 002-x pipeline escalated']);
  });
});

describe('an environmental apply failure keeps the fresh-session retry', () => {
  // proves: Test strategy #6 — the control for the terminal class above. Pair 2's
  // rename still fails with ENOTDIR, but nothing is armed, so pair 1's reverse
  // undo succeeds and the outcome is the environmental class the proposal keeps
  // on today's retry: "the failure being environmental, attempt 2 starts a fresh
  // session". Both classes report `retryInSameSession(...) === false`, so only
  // counting attempts and `session.create` calls tells them apart.
  it('runs a fresh-session attempt 2 for a moves-apply-failed outcome', async () => {
    const { dir } = tmpRepo(state());
    const created: string[] = [];
    const options: ResumeOptions = {
      repoDir: dir,
      slug: '002-x',
      promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
      cfg: FAST,
      client: replyClient(created),
      forceStage: 'fix',
    };

    const undoBefore = fsFault.undoAttempts;
    const spy = vi.spyOn(console, 'error').mockImplementation(() => undefined);
    const { status } = await resumePipeline(options);
    spy.mockRestore();

    expect(status).toBe('escalated');
    expect(fsFault.undoAttempts, 'the reverse undo ran, and it succeeded').toBe(undoBefore);

    const saved = savedState(dir);
    const fixResults = saved.results.filter((r) => r.stage === 'fix');
    // Both attempts ran, each in a session of its own — the driver stopped at
    // nothing, because this failure is not the rollback one.
    expect(fixResults).toHaveLength(2);
    expect(created.filter((k) => k.startsWith('fix'))).toHaveLength(2);
    for (const result of fixResults) {
      expect(result.failure ?? '').toMatch(/^moves-apply-failed/);
    }
    // A3 is untouched by the driver's terminal branch: one stage entry spends
    // exactly one code-budget unit, however many attempts it takes.
    expect(saved.codeBudgetUsed).toBe(1);

    // The worktree is what a failed reply leaves: pair 1 rolled back, the failing
    // pair left nothing behind, and the only commit past the base is the terminal
    // record.
    expect(existsSync(join(dir, 'src', 'moved.ts'))).toBe(false);
    expect(readFileSync(join(dir, 'src', 'old.ts'), 'utf8')).toBe('one\n');
    expect(existsSync(join(dir, 'src', 'gamma.ts', 'nested.ts'))).toBe(false);
    expect(subjects(dir)).toEqual(['docs: base', 'docs: 002-x pipeline escalated']);
  });
});
