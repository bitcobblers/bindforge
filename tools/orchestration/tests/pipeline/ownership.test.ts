import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { enforceOwnership, porcelainPaths } from '../../src/pipeline/runner.js';
import { loadState, recordRunnerDelete, saveState } from '../../src/pipeline/state-store.js';
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

// ---------------------------------------------------------------------------
// ADR-006 §1: the checkpoint is runner-reserved. The ownership sweep is
// author-agnostic, so a stage write to `.pipeline.json`, to the `saveState`
// tmp shape, or to a directory planted at the checkpoint path is a content
// mismatch against what the runner last wrote — reverted before the stage's
// `git add -A` can commit it. The runner's own save is the one match the
// sweep skips, so boundary commits keep advancing the committed checkpoint.
// ---------------------------------------------------------------------------

/** Repo with a committed checkpoint under the design dir. */
function tmpRepoWithCheckpoint(): { dir: string; base: string; checkpoint: string } {
  // The sweep's `headBefore` is the checkpoint commit below, not the base
  // commit `tmpRepo()` returns, so only `dir` carries forward here.
  const { dir } = tmpRepo();
  const checkpoint = join(dir, 'docs', 'designs', '002-x', '.pipeline.json');
  writeFileSync(
    checkpoint,
    `${JSON.stringify({ ...state(), lastStage: 'proposal', updatedAt: 1 }, null, 2)}\n`,
  );
  gitAt(dir, ['add', '-A']);
  gitAt(dir, ['commit', '-q', '-m', 'chore: commit the checkpoint']);
  return { dir, base: gitAt(dir, ['rev-parse', 'HEAD']).trim(), checkpoint };
}

describe('enforceOwnership: runner-reserved checkpoint paths', () => {
  // proves: Test strategy #2 — a stage that rewrites the checkpoint gets its
  // bytes reverted, even though every stage may write `docs/**`.
  it('reverts a stage-written checkpoint', () => {
    const { dir, base, checkpoint } = tmpRepoWithCheckpoint();
    const tampered = { ...state(), lastStage: 'doc-sync', slug: 'x' };
    writeFileSync(checkpoint, `${JSON.stringify(tampered, null, 2)}\n`);

    const report = enforceOwnership(dir, base, 'design-review', state());

    expect(report.reverted).toContain('docs/designs/002-x/.pipeline.json');
    const restored = JSON.parse(readFileSync(checkpoint, 'utf8')) as PipelineState;
    expect(restored.lastStage).toBe('proposal');
  });

  // proves: Test strategy #2 — a planted writer-shaped tmp file matches
  // nothing the runner wrote, so the sweep removes it and `git add -A` stays
  // clean of stray checkpoint tmp files.
  it('removes a planted saveState-shaped tmp file', () => {
    const { dir, base } = tmpRepoWithCheckpoint();
    const planted = join(dir, 'docs', 'designs', '002-x', '.pipeline.json.a1b2c3d4.tmp');
    writeFileSync(planted, '{"slug":"x"}\n');

    const report = enforceOwnership(dir, base, 'design-review', state());

    expect(report.reverted).toContain('docs/designs/002-x/.pipeline.json.a1b2c3d4.tmp');
    expect(existsSync(planted)).toBe(false);
  });

  // proves: Test strategy #2 — the directory replacement of the checkpoint
  // path (the renameSync fixture pattern of this file, inverted) is reverted
  // the same way: the planted directory goes, the committed file returns.
  it('restores the checkpoint over a directory planted at its path', () => {
    const { dir, base, checkpoint } = tmpRepoWithCheckpoint();
    const planted = join(dir, 'docs', 'designs', '002-x', 'moved-proposal.md');
    renameSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), planted);
    rmSync(checkpoint);
    mkdirSync(checkpoint);
    writeFileSync(join(checkpoint, 'inside.md'), 'planted\n');

    const report = enforceOwnership(dir, base, 'design-review', state());

    expect(report.reverted).toContain('docs/designs/002-x/.pipeline.json');
    expect(readFileSync(checkpoint, 'utf8')).toContain('"lastStage": "proposal"');
    expect(existsSync(join(checkpoint, 'inside.md'))).toBe(false);
  });

  // proves: Test strategy #2 — the sweep skips what saveState last wrote, so
  // a boundary whose only checkpoint change is the runner's own save commits
  // that save rather than restoring the checkpoint to `headBefore`.
  it('keeps the runner’s own checkpoint save instead of restoring headBefore', async () => {
    const { dir, base, checkpoint } = tmpRepoWithCheckpoint();
    const advanced = { ...state(), lastStage: 'design-review', updatedAt: 2 };
    await saveState(join(dir, 'docs', 'designs', '002-x'), advanced);

    const report = enforceOwnership(dir, base, 'design-review', advanced);

    expect(report.reverted).not.toContain('docs/designs/002-x/.pipeline.json');
    const saved = JSON.parse(readFileSync(checkpoint, 'utf8')) as PipelineState;
    expect(saved.lastStage).toBe('design-review');
  });

  // proves: Test strategy #2 — the authorship skip compares content, not map
  // presence: bytes the runner recorded and a stage then rewrote mismatch the
  // record, so a stage edit made on top of the runner's own save is still
  // reverted (testing r1 #3 — an implementation that skipped whenever the
  // record merely existed would keep the rest of the suite green).
  it('reverts a stage edit made on top of the runner’s own save', async () => {
    const { dir, base, checkpoint } = tmpRepoWithCheckpoint();
    const advanced = { ...state(), lastStage: 'design-review', updatedAt: 2 };
    await saveState(join(dir, 'docs', 'designs', '002-x'), advanced);
    writeFileSync(
      checkpoint,
      `${JSON.stringify({ ...advanced, lastStage: 'implement' }, null, 2)}\n`,
    );

    const report = enforceOwnership(dir, base, 'design-review', advanced);

    expect(report.reverted).toContain('docs/designs/002-x/.pipeline.json');
    const restored = JSON.parse(readFileSync(checkpoint, 'utf8')) as PipelineState;
    expect(restored.lastStage).toBe('proposal'); // back to what headBefore holds
  });

  // proves: ADR-006 §1 (correctness r1 #1) — the authorship record survives
  // the process boundary: `loadState` seeds it with the bytes it read, so the
  // first sweep of a resumed run skips the run's own dirty checkpoint instead
  // of reporting it as tampering. The bytes here never passed through this
  // process's saveState: they are what the interrupted previous process left
  // on disk, which is the ADR's central resume scenario.
  it('skips the checkpoint bytes a fresh loadState just read', async () => {
    const { dir, base, checkpoint } = tmpRepoWithCheckpoint();
    writeFileSync(
      checkpoint,
      `${JSON.stringify({ ...state(), lastStage: 'design-review', updatedAt: 2 }, null, 2)}\n`,
    );
    await loadState(join(dir, 'docs', 'designs', '002-x')); // the resume's load

    const report = enforceOwnership(dir, base, 'design-review', {
      ...state(),
      lastStage: 'design-review',
      updatedAt: 2,
    });

    expect(report.reverted).not.toContain('docs/designs/002-x/.pipeline.json');
    const kept = JSON.parse(readFileSync(checkpoint, 'utf8')) as PipelineState;
    expect(kept.lastStage).toBe('design-review');
  });

  // proves: ADR-006 §1 (correctness r1 #1, stated bound) — the seed keeps the
  // fence's teeth: a stage write that follows the load mismatches the seeded
  // record and is still reverted.
  it('still reverts a stage write that follows a fresh loadState', async () => {
    const { dir, base, checkpoint } = tmpRepoWithCheckpoint();
    await loadState(join(dir, 'docs', 'designs', '002-x'));
    writeFileSync(
      checkpoint,
      `${JSON.stringify({ ...state(), lastStage: 'implement' }, null, 2)}\n`,
    );

    const report = enforceOwnership(dir, base, 'design-review', state());

    expect(report.reverted).toContain('docs/designs/002-x/.pipeline.json');
    const restored = JSON.parse(readFileSync(checkpoint, 'utf8')) as PipelineState;
    expect(restored.lastStage).toBe('proposal');
  });

  // proves: ADR-006 §1 (correctness r1 #2) — a new run's checkpoint is
  // untracked until its first commitStage; when a stage tampers with it, the
  // sweep restores the bytes the runner recorded for it instead of deleting
  // the run's only checkpoint copy because headBefore holds no blob.
  it('restores recorded bytes over a tampered untracked checkpoint', async () => {
    const { dir, base } = tmpRepo();
    const designDir = join(dir, 'docs', 'designs', '002-x');
    const checkpoint = join(designDir, '.pipeline.json');
    const advanced = { ...state(), lastStage: 'proposal', updatedAt: 1 };
    await saveState(designDir, advanced); // the entry save — nothing committed yet
    const recorded = readFileSync(checkpoint, 'utf8');
    writeFileSync(
      checkpoint,
      `${JSON.stringify({ ...advanced, lastStage: 'implement' }, null, 2)}\n`,
    );

    const report = enforceOwnership(dir, base, 'design-review', advanced);

    expect(report.reverted).toContain('docs/designs/002-x/.pipeline.json');
    // Not deleted: the run keeps its checkpoint, rewritten to the bytes the
    // runner recorded — whatever headBefore holds.
    expect(readFileSync(checkpoint, 'utf8')).toBe(recorded);
  });

  // proves: ADR-006 §1 (testing r1 #4) — resume deletes the escalation marker
  // after gate-logging the split record; that delete is runner-authored, so a
  // committed marker's deletion survives the next sweep instead of being
  // resurrected from headBefore.
  it('leaves a runner-deleted committed marker deleted instead of restoring it', () => {
    const { dir } = tmpRepoWithCheckpoint();
    const marker = join(dir, 'docs', 'designs', '002-x', 'ESCALATION.md');
    writeFileSync(marker, '# pipeline escalated\n\ncheckpoint save failed: EISDIR\n', 'utf8');
    gitAt(dir, ['add', '-A']);
    gitAt(dir, ['commit', '-q', '-m', 'docs: 002-x escalated']);
    const headBefore = gitAt(dir, ['rev-parse', 'HEAD']).trim();

    rmSync(marker); // resume's marker delete, recognized through recordRunnerDelete
    recordRunnerDelete(marker);

    const report = enforceOwnership(dir, headBefore, 'design-review', state());

    expect(report.reverted).not.toContain('docs/designs/002-x/ESCALATION.md');
    expect(existsSync(marker)).toBe(false);
  });
});
