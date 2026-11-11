import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type RunOptions, executeStage, retryInSameSession } from '../../src/pipeline/runner.js';
import { type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-005 end to end: an `implement` reply that carries `moves: {from, to}[]`
// must land exactly ONE stage commit whose `git show --name-status -M` reports
// an `R` entry per pair (rename detection is diff-time and on by default, A5),
// alongside the stage's content edits — the runner applies the renames during
// finalize, ahead of the ownership sweep and `commitStage`. The same round trip
// runs as `test-update` over a multi-pair list inside `tests/**`, the payload
// run 004 could not express (A12) and where rule 3's ownership direction, the
// sweep invariant (#4) and rename detection must hold together. A pair that can
// no longer be applied between validation and apply must fail the stage with
// `moves-apply-failed`, leave the worktree byte-identical, unstaged and UN-swept
// (the same preserved state a failed reply leaves today), and mint no commit.
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

const IMPL_HANDLE = {
  stage: 'implement',
  stageKey: 'implement',
  attempt: 1,
  round: 1,
  specialty: undefined,
  vars: { designDir: 'docs/designs/002-x' },
};

const TEST_UPDATE_HANDLE = {
  stage: 'test-update',
  stageKey: 'test-update',
  attempt: 1,
  round: 1,
  specialty: undefined,
  vars: { designDir: 'docs/designs/002-x', round: 1, reviewFiles: '' },
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

/** Repo with two tracked source files plus the design artifact; returns the base sha. */
function tmpRepo(): { dir: string; base: string } {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-moves-stage-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
      cwd: dir,
      encoding: 'utf8',
    });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  mkdirSync(join(dir, 'src'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'adrs', '002-x.md'), 'x\n');
  writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), 'v1\n');
  writeFileSync(join(dir, 'src', 'old.ts'), 'one\n');
  writeFileSync(join(dir, 'src', 'beta.ts'), 'two\n');
  writeFileSync(join(dir, 'src', 'gamma.ts'), 'gamma\n');
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return { dir, base: git(['rev-parse', 'HEAD']).trim() };
}

/**
 * Repo shaped like run 004's payload (A12): the test engineer's own tracked
 * test files, plus an on-disk-only `tests/pipeline/` subtree the reply moves
 * them into. Every endpoint satisfies `stagePathAllowed('test-update', …)`, so
 * the validator's rule 3 and the ownership sweep agree on the whole list.
 */
function tmpTestRepo(): { dir: string; base: string } {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-moves-test-update-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
      cwd: dir,
      encoding: 'utf8',
    });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  mkdirSync(join(dir, 'src'), { recursive: true });
  mkdirSync(join(dir, 'tests', 'pipeline'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'adrs', '002-x.md'), 'x\n');
  writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), 'v1\n');
  writeFileSync(join(dir, 'src', 'old.ts'), 'one\n');
  for (const name of ['one', 'two', 'three']) {
    writeFileSync(join(dir, 'tests', `${name}.test.ts`), `test('${name}', () => {});\n`);
  }
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

function options(client: OpenCodeClient, repoDir: string): RunOptions {
  return {
    repoDir,
    sourcePath: 'docs/adrs/002-x.md',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    client,
  };
}

/**
 * Stub whose move-capable stage session (implement or test-update) performs its
 * work inside `prompt` (like a real agent writing files) and answers one poll
 * after the prompt with `replyText`.
 */
function implementClient(replyText: string, onPrompt: () => void) {
  const turns = new Map<string, { text: string; visibleFrom: number }[]>();
  const lists = new Map<string, number>();
  let clock = 0;
  let created = 0;
  const client = {
    session: {
      create: async () => {
        created += 1;
        const id = `ses_impl_${String(created)}`;
        turns.set(id, []);
        lists.set(id, 0);
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        onPrompt();
        const mine = turns.get(input.sessionID) ?? [];
        mine.push({ text: replyText, visibleFrom: (lists.get(input.sessionID) ?? 0) + 2 });
      },
      remove: async () => undefined,
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const n = (lists.get(input.sessionID) ?? 0) + 1;
        lists.set(input.sessionID, n);
        const messages: SessionMessageInfo[] = [];
        for (const t of turns.get(input.sessionID) ?? []) {
          if (n < t.visibleFrom) {
            continue;
          }
          clock += 1;
          messages.push({
            id: `m_a${String(messages.length)}`,
            time: { created: clock },
            type: 'assistant',
            content: [{ type: 'text', text: t.text }],
          });
          clock += 1;
          messages.push({
            id: `m_i${String(messages.length)}`,
            time: { created: clock },
            type: 'idle',
          });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, created: () => created };
}

describe('executeStage round trip with a moves reply', () => {
  // proves: Test strategy #8 — one stage commit, an `R` entry per pair at the
  // default 50% similarity threshold, and the stage's own content edit inside
  // that same commit.
  it('lands the declared moves as R entries in exactly one stage commit', async () => {
    const { dir, base } = tmpRepo();
    const reply = JSON.stringify({
      testsNeedUpdate: false,
      commitMessage: 'refactor: rename the module and add a note',
      moves: [{ from: 'src/old.ts', to: 'src/moved.ts' }],
    });
    let wrote = false;
    const { client } = implementClient(reply, () => {
      if (!wrote) {
        wrote = true;
        writeFileSync(join(dir, 'docs', 'designs', '002-x', 'note-added.md'), 'stage edit\n');
      }
    });

    const outcome = await executeStage(options(client, dir), state(), IMPL_HANDLE, FAST);
    expect(outcome.ok).toBe(true);

    // Exactly one commit rides on top of the base.
    const subjects = gitAt(dir, ['log', '--format=%s', `${base}..HEAD`])
      .trim()
      .split('\n');
    expect(subjects).toEqual(['refactor: rename the module and add a note']);

    // The single commit shows the pair as a rename, plus the content edit.
    const nameStatus = gitAt(dir, ['show', '--name-status', '-M', 'HEAD']);
    expect(nameStatus).toMatch(/^R\d+\tsrc\/old\.ts\tsrc\/moved\.ts$/m);
    expect(nameStatus).toMatch(/^A\tdocs\/designs\/002-x\/note-added\.md$/m);

    // The moved content survived the round trip.
    expect(readFileSync(join(dir, 'src', 'moved.ts'), 'utf8')).toBe('one\n');
    expect(existsSync(join(dir, 'src', 'old.ts'))).toBe(false);
  });

  // proves: Test strategy #8 — the motivating payload, run 004's shape (A12):
  // a `test-update` reply moves three of its own test files into a mirrored
  // subtree (testing review r1c0, finding 5: every other integration case moves
  // one src/** or docs/** file as `implement`). Exactly one stage commit carries
  // an `R` entry per pair and the stage's own content edit; no pair degrades
  // into a separate delete+add, which is exactly what an ownership revert of the
  // endpoints would leave behind — the #4 sweep invariant pinned at unit level in
  // ownership.test.ts, seen end to end for the stage that owns `tests/**`.
  it('lands a test-update round trip of three test-file moves in one commit', async () => {
    const { dir, base } = tmpTestRepo();
    const reply = JSON.stringify({
      commitMessage: 'test: mirror the src hierarchy in the tests tree',
      moves: [
        { from: 'tests/one.test.ts', to: 'tests/pipeline/one.test.ts' },
        { from: 'tests/two.test.ts', to: 'tests/pipeline/two.test.ts' },
        { from: 'tests/three.test.ts', to: 'tests/pipeline/three.test.ts' },
      ],
    });
    let wrote = false;
    const { client } = implementClient(reply, () => {
      if (!wrote) {
        wrote = true;
        // The stage's own work: a new test file, in ownership for test-update,
        // that must ride the same commit as the moves.
        writeFileSync(join(dir, 'tests', 'four.test.ts'), "test('four', () => {});\n");
      }
    });

    const outcome = await executeStage(options(client, dir), state(), TEST_UPDATE_HANDLE, FAST);
    expect(outcome.ok, outcome.failure).toBe(true);

    // Exactly one commit rides on top of the base.
    const subjects = gitAt(dir, ['log', '--format=%s', `${base}..HEAD`])
      .trim()
      .split('\n');
    expect(subjects).toEqual(['test: mirror the src hierarchy in the tests tree']);

    // Each pair is a rename in that one commit, at the default 50% threshold.
    const nameStatus = gitAt(dir, ['show', '--name-status', '-M', 'HEAD']);
    expect(nameStatus).toMatch(/^R\d+\ttests\/one\.test\.ts\ttests\/pipeline\/one\.test\.ts$/m);
    expect(nameStatus).toMatch(/^R\d+\ttests\/two\.test\.ts\ttests\/pipeline\/two\.test\.ts$/m);
    expect(nameStatus).toMatch(/^R\d+\ttests\/three\.test\.ts\ttests\/pipeline\/three\.test\.ts$/m);
    expect(nameStatus).toMatch(/^A\ttests\/four\.test\.ts$/m);
    // No endpoint survived as an unreverted delete or add of its own.
    expect(nameStatus).not.toMatch(/^[DM]\ttests\//m);

    // The worktree matches the commit: the subtree holds all three files and the
    // flat names are gone.
    for (const name of ['one', 'two', 'three']) {
      expect(existsSync(join(dir, 'tests', `${name}.test.ts`)), name).toBe(false);
      expect(readFileSync(join(dir, 'tests', 'pipeline', `${name}.test.ts`), 'utf8')).toBe(
        `test('${name}', () => {});\n`,
      );
    }
  });

  // proves: Test strategy #11 — a pair that can no longer be applied between
  // validation and apply (the second pair passes rules 4-7 — its to-parent
  // `src/gamma.ts` exists as a dentry, and no listed path is a path prefix of
  // another — yet rename(2) fails with ENOTDIR because that parent is a
  // regular file) produces no commit, a byte-identical worktree for the pair
  // paths, and an UN-swept preserved worktree for attempt 2, the same shape a
  // failed reply leaves today.
  it('fails with moves-apply-failed: no commit, pair rolled back, worktree preserved un-swept', async () => {
    const { dir, base } = tmpRepo();
    const reply = JSON.stringify({
      testsNeedUpdate: false,
      commitMessage: 'refactor: impossible rename',
      moves: [
        { from: 'src/old.ts', to: 'docs/designs/002-x/old.ts' },
        { from: 'src/beta.ts', to: 'src/gamma.ts/nested.ts' },
      ],
    });
    let wrote = false;
    const { client } = implementClient(reply, () => {
      if (!wrote) {
        wrote = true;
        // Out-of-ownership for implement: had the sweep run, this file would be
        // reverted — its survival proves finalize returned without sweeping.
        mkdirSync(join(dir, 'tests'), { recursive: true });
        writeFileSync(join(dir, 'tests', 'preserved.md'), 'stage wrote this\n');
      }
    });

    const outcome = await executeStage(options(client, dir), state(), IMPL_HANDLE, FAST);
    expect(outcome.ok).toBe(false);
    const failure = outcome.failure ?? '';
    expect(failure.startsWith('moves-apply-failed')).toBe(true);
    // Environmental failure: attempt 2 starts a fresh session, not the same one.
    expect(retryInSameSession(failure, outcome.sessionID)).toBe(false);

    // No stage commit, and HEAD is still the base the stage started from.
    expect(gitAt(dir, ['rev-list', '--count', `${base}..HEAD`]).trim()).toBe('0');
    expect(gitAt(dir, ['rev-parse', 'HEAD']).trim()).toBe(base);

    // The first pair rolled back; the failed pair left nothing behind: the
    // worktree for the pair paths is byte-identical to before finalize.
    expect(readFileSync(join(dir, 'src', 'old.ts'), 'utf8')).toBe('one\n');
    expect(readFileSync(join(dir, 'src', 'beta.ts'), 'utf8')).toBe('two\n');
    expect(existsSync(join(dir, 'docs', 'designs', '002-x', 'old.ts'))).toBe(false);
    expect(existsSync(join(dir, 'src', 'gamma.ts', 'nested.ts'))).toBe(false);
    expect(readFileSync(join(dir, 'src', 'gamma.ts'), 'utf8')).toBe('gamma\n');

    // Preserved, un-swept, and unstaged: the out-of-ownership stage edit is
    // still on disk and uncommitted, and the index holds no content the
    // ownership sweep has not vetted — exactly the state a failed reply leaves
    // today. `--untracked-files=all` is the flag the sweep itself reads paths
    // through (`dirtyFiles`), so attempt 2 still sees every preserved path by
    // name rather than one collapsed directory entry.
    expect(existsSync(join(dir, 'tests', 'preserved.md'))).toBe(true);
    expect(gitAt(dir, ['diff', '--cached', '--name-only'])).toBe('');
    const preserved = gitAt(dir, ['status', '--porcelain', '--untracked-files=all']);
    expect(preserved).toContain('?? tests/preserved.md');
  });
});
