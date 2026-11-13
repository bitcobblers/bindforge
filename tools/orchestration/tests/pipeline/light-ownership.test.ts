import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { enforceOwnership } from '../../src/pipeline/runner.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// The security spine, in a real sweep (A8/A9/A28): the ownership sweep reads
// `flowOf(state)` and hands it to `stagePathAllowed`, so the widening is exactly
// the light coder's test paths, the light `plan` keeps the document stage's
// `docs/**` surface — including the accepted widening over the decision record
// — and every path git reports is judged by its real, repo-root-relative
// spelling. The predicate never sees an un-normalized `docs/../src/...`.
// ---------------------------------------------------------------------------

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

function tmpRepo(): { dir: string; base: string } {
  const dir = mkTmpDir('bindforge-light-ownership-');
  gitAt(dir, ['init', '-q', '-b', 'main']);
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  mkdirSync(join(dir, 'src'), { recursive: true });
  mkdirSync(join(dir, 'tests'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), '# plan\n');
  writeFileSync(join(dir, 'src', 'kept.ts'), 'export const kept = 1;\n');
  writeFileSync(join(dir, 'tests', 'kept.test.ts'), "test('kept', () => {});\n");
  gitAt(dir, ['add', '-A']);
  gitAt(dir, ['commit', '-q', '-m', 'docs: base']);
  return { dir, base: gitAt(dir, ['rev-parse', 'HEAD']).trim() };
}

function state(stage: string, flow: 'light' | 'full'): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'brief', path: 'docs/prompts/002-x.md' },
    status: 'running',
    lastStage: stage,
    flow,
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

describe('ownership in a real sweep', () => {
  // proves: Test strategy #18 — the widening, at the sweep: a light `implement`
  // that writes source and tests keeps both.
  it('keeps a light coder source and test paths', () => {
    const { dir, base } = tmpRepo();
    writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n');
    writeFileSync(join(dir, 'tests', 'thing.test.ts'), "test('thing', () => {});\n");

    const report = enforceOwnership(dir, base, 'implement', state('implement', 'light'));

    expect(report.reverted).toEqual([]);
    expect(existsSync(join(dir, 'src', 'thing.ts'))).toBe(true);
    expect(existsSync(join(dir, 'tests', 'thing.test.ts'))).toBe(true);
  });

  // proves: Test strategy #18 (A8; correctness r1 finding 3) — the widening is
  // by the test-path predicate, not by directory: a co-located test beside its
  // source is the light coder's, so the sweep leaves it — and the prompt that
  // tells the coder it owns the test files is never contradicted by a revert.
  // The same sweep under `full` takes the file back.
  it('keeps a light coder a co-located test file', () => {
    const { dir, base } = tmpRepo();
    writeFileSync(join(dir, 'src', 'thing.test.ts'), "test('thing', () => {});\n");

    const light = enforceOwnership(dir, base, 'implement', state('implement', 'light'));
    expect(light.reverted).toEqual([]);
    expect(existsSync(join(dir, 'src', 'thing.test.ts'))).toBe(true);

    const full = enforceOwnership(dir, base, 'implement', state('implement', 'full'));
    expect(full.reverted).toEqual(['src/thing.test.ts']);
    expect(existsSync(join(dir, 'src', 'thing.test.ts'))).toBe(false);
  });

  // proves: Test strategy #18 — the same sweep under `full` reverts the test
  // paths: the widening is one flow's, not a global loosening.
  it('reverts a full coder test paths', () => {
    const { dir, base } = tmpRepo();
    writeFileSync(join(dir, 'tests', 'thing.test.ts'), "test('thing', () => {});\n");

    const report = enforceOwnership(dir, base, 'implement', state('implement', 'full'));

    expect(report.reverted).toEqual(['tests/thing.test.ts']);
    expect(existsSync(join(dir, 'tests', 'thing.test.ts'))).toBe(false);
  });

  // proves: Test strategy #18 — `plan` is a document stage: its source write is
  // reverted, its document write stays.
  it('reverts a light plan source write and keeps its plan', () => {
    const { dir, base } = tmpRepo();
    writeFileSync(join(dir, 'src', 'sneaky.ts'), 'export const sneaky = 1;\n');
    writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), '# plan\n\nv2\n');

    const report = enforceOwnership(dir, base, 'plan', state('plan', 'light'));

    expect(report.reverted).toEqual(['src/sneaky.ts']);
    expect(existsSync(join(dir, 'src', 'sneaky.ts'))).toBe(false);
    const plan = readFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), 'utf8');
    expect(plan).toBe('# plan\n\nv2\n');
  });

  // proves: Test strategy #18 (A28) — paths reach the predicate from git
  // plumbing: a write that spells a docs prefix over source is seen as git's
  // real path, so no un-normalized spelling can match the `docs/` prefix.
  it('reverts a light plan write that spells docs/../src as src', () => {
    const { dir, base } = tmpRepo();
    mkdirSync(join(dir, 'docs'), { recursive: true });
    // The stage's own spelling walks out of docs/; git reports the real path.
    writeFileSync(join(dir, 'docs', '..', 'src', 'escape.ts'), 'export const escape = 1;\n');

    const report = enforceOwnership(dir, base, 'plan', state('plan', 'light'));

    expect(report.reverted).toEqual(['src/escape.ts']);
    expect(report.reverted).not.toContain('docs/../src/escape.ts');
    expect(existsSync(join(dir, 'src', 'escape.ts'))).toBe(false);
  });

  // proves: Test strategy #18 (#7's accepted widening, asserted NOT reverted) —
  // the light plan's `docs/**` surface includes the decision record: the sweep
  // leaves it applied, and the design records that as accepted.
  it('keeps a light plan write to the decision record', () => {
    const { dir, base } = tmpRepo();
    mkdirSync(join(dir, 'docs', 'designs', '003-sibling'), { recursive: true });
    const adr = '---\nstatus: proposed\n---\n\n# ADR\n';
    writeFileSync(join(dir, 'docs', 'adrs', '009-side.md'), adr);
    writeFileSync(join(dir, 'docs', 'designs', '003-sibling', 'note.md'), 'sibling\n');

    const report = enforceOwnership(dir, base, 'plan', state('plan', 'light'));

    expect(report.reverted).toEqual([]);
    expect(existsSync(join(dir, 'docs', 'adrs', '009-side.md'))).toBe(true);
  });
});
