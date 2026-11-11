import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { enforceOwnership, porcelainPaths } from '../../src/pipeline/runner.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// Run 1, 10:31: the operator landed `78a0c8c` (a harness fix) while a review
// stage was in flight; enforceOwnership diffed `headBefore..HEAD`, blamed the
// stage for it, and reverted it — and the revert commit also swallowed that
// round's own stage artifacts. A later pass logged "reverted 3 file(s)" with
// no paths and no commit, and could not be reconstructed afterwards.
// ---------------------------------------------------------------------------

function gitAt(dir: string, args: string[]): string {
  return execFileSync(
    'git',
    ['-c', 'user.name=fixture', '-c', 'user.email=fixture@example', ...args],
    { cwd: dir, encoding: 'utf8' },
  );
}

/** Repo with one base commit holding a docs artifact and an agent skill. */
function tmpRepo(): { dir: string; base: string } {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-ownership-'));
  gitAt(dir, ['init', '-q', '-b', 'main']);
  mkdirSync(join(dir, 'docs', 'designs', '002-x', 'reviews'), { recursive: true });
  mkdirSync(join(dir, '.opencode', 'skills', 'design-proposal'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), 'v1\n');
  writeFileSync(join(dir, '.opencode', 'skills', 'design-proposal', 'SKILL.md'), 'old text\n');
  gitAt(dir, ['add', '-A']);
  gitAt(dir, ['commit', '-q', '-m', 'docs: base']);
  return { dir, base: gitAt(dir, ['rev-parse', 'HEAD']).trim() };
}

function state(stageCommits?: string[]): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'running',
    lastStage: 'design-review',
    designRound: 9,
    codeRound: 0,
    designBudgetUsed: 9,
    codeBudgetUsed: 0,
    editorialDone: false,
    testsGreen: false,
    testsNeedUpdate: false,
    stageFailures: {},
    ...(stageCommits === undefined ? {} : { stageCommits }),
    forceNext: null,
    results: [],
    gateLog: [],
    startedAt: 0,
    updatedAt: 0,
  };
}

describe('porcelainPaths', () => {
  it('reads NUL-separated entries and leaves odd names intact', () => {
    const porcelain = [' M docs/a.md', '?? odd name.md', '?? a -> b.md', '', 'A  x/y.ts'].join(
      '\0',
    );
    expect(porcelainPaths(porcelain)).toEqual(['docs/a.md', 'odd name.md', 'a -> b.md', 'x/y.ts']);
  });

  it('lists untracked directory contents instead of the collapsed directory', () => {
    const { dir } = tmpRepo();
    mkdirSync(join(dir, 'scratch', 'nested'), { recursive: true });
    writeFileSync(join(dir, 'scratch', 'nested', 'note.md'), 'x\n');
    writeFileSync(join(dir, 'scratch', 'loose.md'), 'y\n');
    const listing = gitAt(dir, [
      'status',
      '--porcelain=v1',
      '-z',
      '--untracked-files=all',
      '--no-renames',
    ]);
    expect(porcelainPaths(listing).sort()).toEqual(['scratch/loose.md', 'scratch/nested/note.md']);
    // The old parse produced one `scratch/` entry; deleting it threw
    // ERR_FS_EISDIR and killed the run mid-finalize.
    expect(porcelainPaths(gitAt(dir, ['status', '--porcelain']))).not.toContain('scratch/');
  });
});

describe('enforceOwnership', () => {
  it('reports an operator commit landed mid-stage and leaves it alone', () => {
    const { dir, base } = tmpRepo();
    writeFileSync(join(dir, '.opencode', 'skills', 'design-proposal', 'SKILL.md'), 'fixed text\n');
    gitAt(dir, ['add', '-A']);
    gitAt(dir, ['commit', '-q', '-m', 'chore: fix edit and code-mode guidance in stage skills']);

    const report = enforceOwnership(dir, base, 'design-review', state());

    expect(report.reverted).toEqual([]);
    expect(report.foreign).toHaveLength(1);
    expect(report.foreign[0].subject).toBe(
      'chore: fix edit and code-mode guidance in stage skills',
    );
    expect(
      readFileSync(join(dir, '.opencode', 'skills', 'design-proposal', 'SKILL.md'), 'utf8'),
    ).toBe('fixed text\n');
    // No revert commit was minted, so the stage artifacts stay intact.
    expect(gitAt(dir, ['rev-list', '--count', `${base}..HEAD`]).trim()).toBe('1');
  });

  it('reverts paths from a runner-authored commit and records the revert', () => {
    const { dir, base } = tmpRepo();
    mkdirSync(join(dir, 'tools'), { recursive: true });
    writeFileSync(join(dir, 'tools', 'sneaky.ts'), 'export const x = 1;\n');
    gitAt(dir, ['add', '-A']);
    gitAt(dir, ['commit', '-q', '-m', 'feat: stage wrote source outside its ownership']);
    const sha = gitAt(dir, ['rev-parse', 'HEAD']).trim();

    const s = state([sha]);
    const report = enforceOwnership(dir, base, 'design-review', s);

    expect(report.reverted).toEqual(['tools/sneaky.ts']);
    expect(report.foreign).toEqual([]);
    expect(s.stageCommits).toHaveLength(2);
    expect(gitAt(dir, ['status', '--porcelain'])).toBe('');
  });

  it('reverts dirty out-of-ownership work and names every path', () => {
    const { dir, base } = tmpRepo();
    mkdirSync(join(dir, 'tools'), { recursive: true });
    writeFileSync(join(dir, 'tools', 'leaked.ts'), 'x\n');
    writeFileSync(join(dir, '.opencode', 'skills', 'design-proposal', 'SKILL.md'), 'clobbered\n');

    const report = enforceOwnership(dir, base, 'proposal', state());

    expect(report.reverted.sort()).toEqual([
      '.opencode/skills/design-proposal/SKILL.md',
      'tools/leaked.ts',
    ]);
    expect(
      readFileSync(join(dir, '.opencode', 'skills', 'design-proposal', 'SKILL.md'), 'utf8'),
    ).toBe('old text\n');
  });

  it('survives a renamed tracked file without absorbing it into one entry', () => {
    const { dir, base } = tmpRepo();
    renameSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), join(dir, 'MOVED.md'));
    writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), 'v2\n');

    const report = enforceOwnership(dir, base, 'design-review', state());

    expect(report.reverted).toContain('MOVED.md');
    expect(readFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), 'utf8')).toBe(
      'v2\n',
    );
  });

  // proves: Test strategy #4 — the sweep needs no reply-moves branch: a
  // runner-applied move pair (delete of a tracked file + untracked add) whose
  // endpoints both satisfy `stagePathAllowed` for the executing stage is absent
  // from `report.reverted` and stays applied. This pins the invariant that the
  // moves validator and the sweep share one `stagePathAllowed` predicate; a
  // future fork of the predicate breaks this test instead of silently breaking
  // production.
  it('leaves an in-ownership move pair applied and out of the reverted list', () => {
    const { dir, base } = tmpRepo();
    renameSync(
      join(dir, 'docs', 'designs', '002-x', '02-proposal.md'),
      join(dir, 'docs', 'designs', '002-x', '03-renamed.md'),
    );

    const report = enforceOwnership(dir, base, 'design-review', state());

    expect(report.reverted).toEqual([]);
    expect(existsSync(join(dir, 'docs', 'designs', '002-x', '03-renamed.md'))).toBe(true);
    expect(existsSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'))).toBe(false);
  });

  // proves: Test strategy #4 — the same delete+add shape whose endpoints fail
  // `stagePathAllowed` is unsanctioned: the deleted path is still restored from
  // `headBefore` and the planted add removed. Here `implement` (everything but
  // tests) renames a tracked test file, so both endpoints are violations.
  it('restores an unsanctioned delete+add pair from headBefore', () => {
    const { dir } = tmpRepo();
    mkdirSync(join(dir, 'tests'), { recursive: true });
    writeFileSync(join(dir, 'tests', 'old.test.ts'), "test('old', () => {});\n");
    gitAt(dir, ['add', '-A']);
    gitAt(dir, ['commit', '-q', '-m', 'test: add a tracked test file']);
    const base = gitAt(dir, ['rev-parse', 'HEAD']).trim();
    renameSync(join(dir, 'tests', 'old.test.ts'), join(dir, 'tests', 'new.test.ts'));

    const report = enforceOwnership(dir, base, 'implement', state());

    expect(report.reverted.sort()).toEqual(['tests/new.test.ts', 'tests/old.test.ts']);
    expect(readFileSync(join(dir, 'tests', 'old.test.ts'), 'utf8')).toContain("test('old'");
    expect(existsSync(join(dir, 'tests', 'new.test.ts'))).toBe(false);
  });
});
