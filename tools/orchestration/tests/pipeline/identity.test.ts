import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  type RunIdentity,
  deriveIdentity,
  identityClash,
  sanitizeSlug,
  slugFromSource,
} from '../../src/pipeline/identity.js';
import { SLUG_RE } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// A run's identity names its branch and its design directory. Run 1's branch
// was `pipeline/002-parallel-inference-lane`; starting a fresh run of the same
// ADR derived the same name, and ensureBranch checked the old branch out —
// the new run would have inherited 9 design rounds of artifacts and a
// baseRef that is not an ancestor of HEAD.
// ---------------------------------------------------------------------------

const ID: RunIdentity = {
  slug: 'parallel-inference-lane',
  nnn: '002',
  kind: 'adr',
  branch: 'pipeline/002-parallel-inference-lane',
};

function tmpRepo(withBranch?: string): { dir: string } {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-identity-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
      cwd: dir,
      encoding: 'utf8',
    });
  git(['init', '-q', '-b', 'main']);
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'adrs', '001-first.md'), 'x\n');
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  if (withBranch) {
    git(['branch', withBranch]);
  }
  return { dir };
}

describe('slugFromSource', () => {
  it('drops the NNN prefix', () => {
    expect(slugFromSource('/repo/docs/adrs/002-parallel-inference-lane.md')).toBe(
      'parallel-inference-lane',
    );
  });

  it('forces names into the slug grammar the validator enforces', () => {
    expect(sanitizeSlug('2026-11 WIP!')).toBe('2026-11-wip');
    expect(sanitizeSlug('--x--')).toBe('x');
    expect(sanitizeSlug('   ')).toBe('untitled');
    expect(slugFromSource('/repo/docs/prompts/004-Some Brief!.md')).toBe('some-brief');
    // The contract that matters: whatever the source document is called, the
    // generated slug is one the checkpoint validator accepts and that cannot
    // escape its path segment.
    for (const hostile of ['../escape', 'a\nb', 'Über Brief', '..', '0', 'x//y']) {
      expect(SLUG_RE.test(sanitizeSlug(hostile))).toBe(true);
    }
    expect(sanitizeSlug('../escape')).toBe('escape');
  });
});

describe('deriveIdentity', () => {
  it('takes nnn from the ADR file name', () => {
    const { dir } = tmpRepo();
    expect(deriveIdentity(dir, join(dir, 'docs/adrs/002-parallel-inference-lane.md'))).toEqual(ID);
  });

  it('pins the identity for a second attempt at the same ADR', () => {
    const { dir } = tmpRepo();
    const id = deriveIdentity(dir, join(dir, 'docs/adrs/002-parallel-inference-lane.md'), {
      slug: 'run2',
    });
    expect(id).toEqual({
      slug: 'run2',
      nnn: '002',
      kind: 'adr',
      branch: 'pipeline/002-run2',
    });
  });

  it('allocates the next nnn for a brief', () => {
    const { dir } = tmpRepo();
    mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
    const id = deriveIdentity(dir, join(dir, 'docs/prompts/add-timeout.md'));
    expect(id.kind).toBe('brief');
    expect(id.nnn).toBe('002');
  });

  it('rejects malformed pins instead of writing an unusable identity', () => {
    const { dir } = tmpRepo();
    const source = join(dir, 'docs/adrs/002-x.md');
    expect(() => deriveIdentity(dir, source, { nnn: '2' })).toThrow(/invalid --nnn/);
    expect(() => deriveIdentity(dir, source, { slug: '../escape' })).toThrow(/invalid --slug/);
    expect(() => deriveIdentity(dir, source, { slug: 'Run Two' })).toThrow(/invalid --slug/);
  });
});

describe('identityClash', () => {
  it('is empty for a fresh identity', () => {
    const { dir } = tmpRepo();
    expect(identityClash(dir, ID)).toEqual([]);
  });

  it('names the taken branch and the existing checkpoint', () => {
    const { dir } = tmpRepo(ID.branch);
    const designDir = join(dir, 'docs', 'designs', '002-parallel-inference-lane');
    mkdirSync(designDir, { recursive: true });
    writeFileSync(join(designDir, '.pipeline.json'), '{}\n');

    const reasons = identityClash(dir, ID);
    expect(reasons).toHaveLength(2);
    expect(reasons[0]).toContain(ID.branch);
    expect(reasons[1]).toContain('resume');
  });

  it('accepts a prepared branch the operator is already standing on', () => {
    const { dir } = tmpRepo();
    execFileSync('git', ['checkout', '-q', '-b', ID.branch], { cwd: dir });
    expect(identityClash(dir, ID)).toEqual([]);
  });

  it('flags a design directory left behind without a checkpoint', () => {
    const { dir } = tmpRepo();
    mkdirSync(join(dir, 'docs', 'designs', '002-parallel-inference-lane'), { recursive: true });
    expect(identityClash(dir, ID)).toEqual([
      expect.stringContaining(
        'design directory docs/designs/002-parallel-inference-lane already exists',
      ),
    ]);
  });
});
