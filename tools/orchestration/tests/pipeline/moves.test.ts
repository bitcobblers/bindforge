import { execFileSync } from 'node:child_process';
import {
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it, vi } from 'vitest';

import { type MoveApplyFailure, applyMoves, validateMoves } from '../../src/pipeline/moves.js';
import type { MovePair } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-005 / run 004 (A12): 23 test-file moves were inexpressible to any
// stage, so fix and test-update livelocked six rounds and the operator
// landed the moves by hand. The fix makes moves data in the reply:
// `validateMoves(repoDir, headBefore, stageName, moves)` checks the whole
// list against `headBefore` plus the disk before anything is touched, and
// `applyMoves(repoDir, headBefore, moves)` lands it in reply order with
// `fs.renameSync`, rolling back in reverse on the first failure. Every
// rejection must name the offending pair; every case below is one clause of
// the validator rules (proposal, Chosen design: Validator 1-7) or of the
// apply contract.
//
// The failure representation is fixed (03-build-notes.md, deviation 3):
// `validateMoves` returns the issue text naming the offending pair or null, and
// `applyMoves` returns `MoveApplyFailure` (`{ error, rollbackFailed }`) or null.
// `rollbackFailed` is the discriminator `finalizeStage` branches on for the
// terminal `moves-rollback-failed` class, so every apply case asserts it. Both
// contracts are imported as types, so a drift in `MovePairSchema` or
// `MoveApplyFailure` breaks the test build instead of passing unnoticed.
// ---------------------------------------------------------------------------

// Fault injection for the one state the proposal calls "the one state nothing
// in-process can undo" — a rollback rename that itself fails. `rename(2)` is
// symmetric, so no plain filesystem condition makes a forward rename succeed
// while its exact reverse fails; `renameSync` passes through to the real fs
// until armed, then fails (the partial-mock route `tests/cli.test.ts:29` uses).
const fsFault = vi.hoisted(() => ({
  allowRenames: Number.POSITIVE_INFINITY,
  renames: [] as string[],
}));

vi.mock('node:fs', async (importOriginal) => {
  const actual = await importOriginal<typeof import('node:fs')>();
  return {
    ...actual,
    renameSync: (...args: Parameters<typeof actual.renameSync>) => {
      fsFault.renames.push(`${String(args[0])} -> ${String(args[1])}`);
      if (fsFault.renames.length > fsFault.allowRenames) {
        throw Object.assign(new Error(`EIO: injected rename failure ${String(args[0])}`), {
          code: 'EIO',
        });
      }
      return actual.renameSync(...args);
    },
  };
});

function gitAt(dir: string, args: string[]): string {
  return execFileSync(
    'git',
    ['-c', 'user.name=fixture', '-c', 'user.email=fixture@example', ...args],
    { cwd: dir, encoding: 'utf8' },
  );
}

/**
 * Repo with everything the validator rules need to be pinned one at a time:
 * tracked regular files under src/, a tracked symlink (mode 120000), README.md +
 * notes.md for the case-fold cases, a tracked tests/ file, a design dir, and a
 * tracked filename that is itself a git pathspec glob (`bindings[en].json`)
 * plus its glob siblings (`ab.ts`, `bb.ts`) for the literal-lookup pins.
 * `headBefore` is the single base commit.
 */
function movesRepo(): { dir: string; headBefore: string } {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-moves-'));
  gitAt(dir, ['init', '-q', '-b', 'main']);
  mkdirSync(join(dir, 'src'), { recursive: true });
  mkdirSync(join(dir, 'tests'), { recursive: true });
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  writeFileSync(join(dir, 'src', 'old.ts'), 'one\n');
  writeFileSync(join(dir, 'src', 'beta.ts'), 'two\n');
  writeFileSync(join(dir, 'src', 'ab.ts'), 'three\n');
  writeFileSync(join(dir, 'src', 'bb.ts'), 'four\n');
  writeFileSync(join(dir, 'src', 'bindings[en].json'), '{}\n');
  symlinkSync('old.ts', join(dir, 'src', 'link.ts'));
  writeFileSync(join(dir, 'README.md'), '# readme\n');
  writeFileSync(join(dir, 'notes.md'), 'plain notes\n');
  writeFileSync(join(dir, 'tests', 'x.test.ts'), "test('x', () => {});\n");
  writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), 'v1\n');
  gitAt(dir, ['add', '-A']);
  gitAt(dir, ['commit', '-q', '-m', 'docs: base']);
  return { dir, headBefore: gitAt(dir, ['rev-parse', 'HEAD']).trim() };
}

/** The fixed validator contract: the issue text naming the pair, or null. */
function issueOf(dir: string, headBefore: string, stage: string, pairs: MovePair[]): string | null {
  return validateMoves(dir, headBefore, stage, pairs);
}

/** The fixed applier contract: `MoveApplyFailure`, or null when the list landed. */
function applyFailureOf(
  dir: string,
  headBefore: string,
  pairs: MovePair[],
): MoveApplyFailure | null {
  return applyMoves(dir, headBefore, pairs);
}

/** A rejection is only a rejection if it names the offending path. */
function expectRejectionNaming(issue: string | null, path: string): void {
  expect(issue, 'validator must reject the pair, not accept it').not.toBeNull();
  expect(issue as string).toContain(path);
}

describe('validateMoves — rule 4: from must be a tracked regular file', () => {
  // proves: Test strategy #2 (untracked from)
  it('rejects a from that is not tracked at headBefore', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/ghost.ts', to: 'src/landed.ts' }]),
      'src/ghost.ts',
    );
  });

  // proves: Test strategy #2 (from a tree — the file-only rule)
  it('rejects a from that names a tracked tree', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src', to: 'docs/designs/002-x/src' }]),
      'src',
    );
  });

  // proves: Test strategy #2 (from a tracked symlink — mode 120000 fails the regular-file mode)
  it('rejects a from that is a tracked symlink', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/link.ts', to: 'src/unlinked.ts' }]),
      'src/link.ts',
    );
  });

  // proves: Test strategy #2 (from deleted on disk)
  it('rejects a from that is tracked at headBefore but deleted on disk', () => {
    const { dir, headBefore } = movesRepo();
    rmSync(join(dir, 'src', 'old.ts'));
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/moved.ts' }]),
      'src/old.ts',
    );
  });
});

describe('validateMoves — rule 5: to must exist nowhere, while its parent does', () => {
  // proves: Test strategy #2 (to on disk)
  it('rejects a to that already exists on disk', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/beta.ts' }]),
      'src/beta.ts',
    );
  });

  // proves: Test strategy #2 (to a dangling symlink — a dentry rename(2) would replace silently)
  it('rejects a to that is a dangling symlink', () => {
    const { dir, headBefore } = movesRepo();
    symlinkSync('never-existed.ts', join(dir, 'src', 'dangling.ts'));
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/dangling.ts' }]),
      'src/dangling.ts',
    );
  });

  // proves: Test strategy #2 (to tracked at headBefore)
  it('rejects a to that is tracked at headBefore even when deleted from disk', () => {
    const { dir, headBefore } = movesRepo();
    rmSync(join(dir, 'src', 'beta.ts'));
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/beta.ts' }]),
      'src/beta.ts',
    );
  });

  // proves: Test strategy #2 (to's parent missing — a reply error, not an apply failure)
  it('rejects a to whose parent does not exist', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'nodir/new.ts' }]),
      'nodir/new.ts',
    );
  });
});

describe('validateMoves — rules 4 and 5 look up the literal path, never a git glob', () => {
  // proves: Test strategy #2 (03-build-notes.md deviation 5) — a legitimate
  // Linux/macOS filename that is itself a pathspec glob (`bindings[en].json`)
  // is tracked and regular, so rule 4 must accept it. A bare pathspec is
  // fnmatch'd by git even after `--`, and the class `[en]` then matches no
  // tracked path at all, rejecting the pair the design allows.
  it('accepts a tracked from whose name is a git pathspec glob', () => {
    const { dir, headBefore } = movesRepo();
    expect(
      issueOf(dir, headBefore, 'implement', [
        { from: 'src/bindings[en].json', to: 'src/bindings-en.json' },
      ]),
    ).toBeNull();
  });

  // proves: Test strategy #2 (deviation 5) — the file-only rule evaluates the
  // literal name: an untracked `a?.ts` that glob-matches the tracked sibling
  // `ab.ts` is not tracked, and must fail rule 4 instead of borrowing its mode.
  it('rejects an untracked from that glob-matches a tracked sibling', () => {
    const { dir, headBefore } = movesRepo();
    writeFileSync(join(dir, 'src', 'a?.ts'), 'never committed\n');
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/a?.ts', to: 'src/a-landed.ts' }]),
      'src/a?.ts',
    );
  });

  // proves: Test strategy #2 (deviation 5) — same shape as a directory: with a
  // glob-matched mode lookup it would pass the file-only gate and `renameSync`
  // would move a whole directory, the case the proposal defers.
  it('rejects an untracked directory that glob-matches a tracked sibling', () => {
    const { dir, headBefore } = movesRepo();
    mkdirSync(join(dir, 'src', 'b?.ts'));
    writeFileSync(join(dir, 'src', 'b?.ts', 'inner.ts'), 'inside the untracked directory\n');
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/b?.ts', to: 'src/b-landed.ts' }]),
      'src/b?.ts',
    );
  });
});

describe('validateMoves — rule 1: shape', () => {
  // proves: Test strategy #2 (from === to)
  it('rejects a pair whose endpoints are identical', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/old.ts' }]),
      'src/old.ts',
    );
  });

  // proves: Test strategy #2 (absolute path)
  it('rejects an absolute endpoint', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: '/etc/passwd', to: 'src/stolen' }]),
      '/etc/passwd',
    );
  });

  // proves: Test strategy #2 (`.` segment)
  it('rejects a `.` segment', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/./new.ts' }]),
      'src',
    );
  });

  // proves: Test strategy #2 (`..` segment)
  it('rejects a `..` segment', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: '../outside.ts' }]),
      '..',
    );
  });

  // proves: Test strategy #2 (endpoint with `\`)
  it('rejects an endpoint containing a backslash', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src\\old.ts', to: 'src/new.ts' }]),
      'src\\old.ts',
    );
  });

  // proves: Test strategy #2 (endpoint with a control character)
  it('rejects an endpoint containing a control character', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/a\u0001b.ts', to: 'src/new.ts' }]),
      'src/a',
    );
  });

  // proves: Test strategy #2 (endpoint with `:`)
  it('rejects an endpoint containing a colon', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/a:b.ts', to: 'src/new.ts' }]),
      'src/a:b.ts',
    );
  });

  // proves: Test strategy #2 (Windows reserved name). Pinned on Linux for every
  // platform (Compatibility matrix); rule 1 rejects it as an ordinary returned
  // issue, and the end-to-end `invalid-reply: moves` wiring of that rejection is
  // pinned in reply-contract.test.ts (Test strategy #9).
  it('rejects a segment that is a Windows reserved name', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'CON', to: 'src/new.ts' }]),
      'CON',
    );
  });

  // proves: Test strategy #2 (segment ending in `.`, a Windows-illegal name)
  it('rejects a segment that ends in a dot', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/x./new.ts' }]),
      'src/x./new.ts',
    );
  });

  // proves: Test strategy #2 (segment ending in a space, a Windows-illegal name)
  it('rejects a segment that ends in a space', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/x /new.ts' }]),
      'src/x /new.ts',
    );
  });

  // proves: Test strategy #2 (empty segment)
  it('rejects an endpoint with an empty segment', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src//new.ts' }]),
      'src//new.ts',
    );
  });
});

describe('validateMoves — rule 2: runner-reserved paths and .git', () => {
  // proves: Test strategy #2 (STATE_FILE segment)
  it('rejects an endpoint with a .pipeline.json segment', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [
        { from: 'src/old.ts', to: 'docs/designs/002-x/.pipeline.json' },
      ]),
      '.pipeline.json',
    );
  });

  // proves: Test strategy #2 (LOCK_FILE segment)
  it('rejects an endpoint with a .pipeline.lock segment', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'docs/.pipeline.lock' }]),
      '.pipeline.lock',
    );
  });

  // proves: Test strategy #2 (.git endpoint)
  it('rejects .git itself as an endpoint', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: '.git', to: 'src/anything' }]),
      '.git',
    );
  });

  // proves: Test strategy #2 (.git/config endpoint)
  it('rejects an endpoint inside .git', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: '.git/config', to: 'src/config' }]),
      '.git/config',
    );
  });

  // proves: Test strategy #2 (.GIT/hooks/pre-commit endpoint — the case-folded .git rule)
  it('rejects a case-folded .git endpoint', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: '.GIT/hooks/pre-commit' }]),
      '.GIT/hooks/pre-commit',
    );
  });

  // proves: Test strategy #2 (src/.git/config endpoint — a nested .git in an owned dir)
  it('rejects a nested .git endpoint inside an owned directory', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/.git/config' }]),
      'src/.git/config',
    );
  });
});

describe('validateMoves — rule 3: ownership via stagePathAllowed', () => {
  // proves: Test strategy #2 (test-update out of src/**)
  it('rejects test-update moving a source file', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'test-update', [{ from: 'src/old.ts', to: 'src/renamed.ts' }]),
      'src/old.ts',
    );
  });

  // proves: Test strategy #2 (implement into tests/**)
  it('rejects implement moving a source file into tests', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'tests/old.test.ts' }]),
      'tests/old.test.ts',
    );
  });
});

describe('validateMoves — rule 6: disjoint pairs, case-folded on every platform', () => {
  // proves: Test strategy #2 (duplicate endpoints)
  it('rejects duplicate froms and duplicate tos', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [
        { from: 'src/old.ts', to: 'src/first.ts' },
        { from: 'src/old.ts', to: 'src/second.ts' },
      ]),
      'src/old.ts',
    );
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [
        { from: 'src/old.ts', to: 'src/shared.ts' },
        { from: 'src/beta.ts', to: 'src/shared.ts' },
      ]),
      'src/shared.ts',
    );
  });

  // proves: Test strategy #2 (lone case-only pair)
  it('rejects the case-only move README.md -> readme.md', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'README.md', to: 'readme.md' }]),
      'README.md',
    );
  });

  // proves: Test strategy #2 (swap A→B, B→A)
  it('rejects a swap of two tracked files', () => {
    const { dir, headBefore } = movesRepo();
    const issue = issueOf(dir, headBefore, 'implement', [
      { from: 'src/old.ts', to: 'src/beta.ts' },
      { from: 'src/beta.ts', to: 'src/old.ts' },
    ]);
    expect(issue).not.toBeNull();
    expect(issue as string).toMatch(/src\/(old|beta)\.ts/);
  });

  // proves: Test strategy #2 (chain A→B, B→C)
  it('rejects a chain where one to equals another from', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [
        { from: 'src/old.ts', to: 'src/beta.ts' },
        { from: 'src/beta.ts', to: 'src/gamma.ts' },
      ]),
      'src/beta.ts',
    );
  });

  // proves: Test strategy #2 (no listed path a path-prefix of another) — a pair
  // that nests inside itself: rules 4, 5 and 7 all pass, since the `to`'s parent
  // is a dentry (the tracked `from` file), so only the nesting check rejects it.
  // An unguarded `nests()` would turn this into an apply-time ENOTDIR.
  it('rejects a pair whose to nests inside its own from', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/old.ts/inner.ts' }]),
      'src/old.ts',
    );
  });

  // proves: Test strategy #2 (no listed path a path-prefix of another) — one
  // pair nesting inside a different pair's endpoint, which is what keeps
  // `src/old.ts -> src/delta.ts` together with
  // `src/beta.ts -> src/old.ts/inner.ts` out of one reply.
  it('rejects a list where a pair nests inside another pair endpoint', () => {
    const { dir, headBefore } = movesRepo();
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [
        { from: 'src/old.ts', to: 'src/delta.ts' },
        { from: 'src/beta.ts', to: 'src/old.ts/inner.ts' },
      ]),
      'src/old.ts',
    );
  });
});

describe('validateMoves — rule 7: containment of resolved paths', () => {
  // proves: Test strategy #2 (containment — a symlinked to-parent escaping the repo)
  it('rejects a to whose parent resolves outside the repo', () => {
    const { dir, headBefore } = movesRepo();
    symlinkSync('..', join(dir, 'out'));
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'out/escape.ts' }]),
      'out/escape.ts',
    );
  });

  // proves: Test strategy #2 (a symlinked endpoint landing in the real .git)
  it('rejects a to whose parent is a symlink into .git', () => {
    const { dir, headBefore } = movesRepo();
    symlinkSync('.git', join(dir, 'glink'));
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'glink/config' }]),
      'glink/config',
    );
  });
});

// proves: Test strategy #3 — the validator works against headBefore, not the
// index/worktree contents: uncommitted edits to `from` must not invalidate it.
it('accepts a legal pair whose from carries uncommitted content edits', () => {
  const { dir, headBefore } = movesRepo();
  writeFileSync(join(dir, 'src', 'old.ts'), 'one\n// dirty edit from the stage\n');
  expect(
    issueOf(dir, headBefore, 'implement', [{ from: 'src/old.ts', to: 'src/moved.ts' }]),
  ).toBeNull();
});

/**
 * Probes the tmpfs-mount capability once, in a scratch directory that is
 * mounted and immediately unmounted, so the EXDEV case is reported as skipped
 * where the capability is absent (the common unprivileged container) instead of
 * running, mounting nothing, and asserting nothing.
 */
function canMountTmpfs(): boolean {
  if (process.platform !== 'linux') return false;
  const probe = mkdtempSync(join(tmpdir(), 'bindforge-mnt-probe-'));
  try {
    execFileSync('mount', ['-t', 'tmpfs', 'tmpfs', probe]);
  } catch {
    rmSync(probe, { recursive: true, force: true });
    return false;
  }
  try {
    execFileSync('umount', [probe]);
  } catch {
    // The probe only reports the capability; a lingering mount is not a failure.
  }
  rmSync(probe, { recursive: true, force: true });
  return true;
}

describe('applyMoves', () => {
  // proves: Test strategy #5 (pairs land in order carrying dirty content)
  it('lands every pair in reply order, dirty content included', () => {
    const { dir, headBefore } = movesRepo();
    writeFileSync(join(dir, 'src', 'old.ts'), 'one\n// stage edit\n');
    const pairs: MovePair[] = [
      { from: 'src/old.ts', to: 'docs/designs/002-x/old.ts' },
      { from: 'src/beta.ts', to: 'src/gamma.ts' },
    ];
    expect(applyFailureOf(dir, headBefore, pairs)).toBeNull();
    expect(readFileSync(join(dir, 'docs', 'designs', '002-x', 'old.ts'), 'utf8')).toBe(
      'one\n// stage edit\n',
    );
    expect(existsSync(join(dir, 'src', 'old.ts'))).toBe(false);
    expect(readFileSync(join(dir, 'src', 'gamma.ts'), 'utf8')).toBe('two\n');
    expect(existsSync(join(dir, 'src', 'beta.ts'))).toBe(false);
  });

  // proves: Test strategy #5 (the rules 4-7 guard refuses a from untracked at the
  // headBefore handed in; the refusal is an apply failure, not a validation nudge —
  // retryInSameSession on its message must therefore be false. Nothing was
  // renamed, so no undo failed: this is the retryable class, not the terminal one.)
  it('refuses a list whose from is untracked at the headBefore handed in, leaving disk untouched', () => {
    const { dir, headBefore } = movesRepo();
    writeFileSync(join(dir, 'src', 'untracked.ts'), 'never committed\n');
    const pairs: MovePair[] = [{ from: 'src/untracked.ts', to: 'src/moved.ts' }];
    const failure = applyFailureOf(dir, headBefore, pairs);
    expect(failure, 'the guard must refuse the list').not.toBeNull();
    expect(failure?.rollbackFailed, 'no rename ran, so no undo can have failed').toBe(false);
    expect(existsSync(join(dir, 'src', 'untracked.ts'))).toBe(true);
    expect(existsSync(join(dir, 'src', 'moved.ts'))).toBe(false);
  });

  // proves: Test strategy #5 (a pair failing mid-list rolls back in reverse, leaving
  // the disk byte-identical). The second pair is crafted to pass rules 4-7 (its
  // to-parent exists as a dentry) but fail at rename(2) with ENOTDIR. The reverse
  // undo succeeded, so this stays the environmental, retryable class.
  it('rolls back completed renames in reverse when a later pair fails, byte-identical', () => {
    const { dir, headBefore } = movesRepo();
    const pairs: MovePair[] = [
      { from: 'src/old.ts', to: 'docs/designs/002-x/old.ts' },
      { from: 'src/beta.ts', to: 'README.md/inner.ts' },
    ];
    const failure = applyFailureOf(dir, headBefore, pairs);
    expect(failure).not.toBeNull();
    expect(failure?.rollbackFailed, 'the reverse rollback was exact').toBe(false);
    expect(readFileSync(join(dir, 'src', 'old.ts'), 'utf8')).toBe('one\n');
    expect(readFileSync(join(dir, 'src', 'beta.ts'), 'utf8')).toBe('two\n');
    expect(readFileSync(join(dir, 'README.md'), 'utf8')).toBe('# readme\n');
    expect(existsSync(join(dir, 'docs', 'designs', '002-x', 'old.ts'))).toBe(false);
    let innerExists = true;
    try {
      lstatSync(join(dir, 'README.md', 'inner.ts'));
    } catch {
      innerExists = false;
    }
    expect(innerExists).toBe(false);
  });

  // proves: Test strategy #5 (EXDEV across a mount point). Operator-run per the
  // Compatibility matrix: mounting a tmpfs needs a capability the vitest suite
  // lacks, so the capability is probed once and an unprivileged host reports the
  // case as skipped instead of mounting nothing and recording a pass.
  describe.runIf(canMountTmpfs())('on a host that can mount a tmpfs', () => {
    it('fails EXDEV for a pair whose to-parent is on another mount', () => {
      const { dir, headBefore } = movesRepo();
      const mnt = join(dir, 'mnt');
      mkdirSync(mnt);
      execFileSync('mount', ['-t', 'tmpfs', 'tmpfs', mnt]);
      try {
        const failure = applyFailureOf(dir, headBefore, [{ from: 'src/old.ts', to: 'mnt/old.ts' }]);
        expect(failure, 'rename(2) across a mount point fails EXDEV').not.toBeNull();
        expect(failure?.rollbackFailed, 'the failed pair moved nothing').toBe(false);
        expect(readFileSync(join(dir, 'src', 'old.ts'), 'utf8')).toBe('one\n');
        expect(existsSync(join(dir, 'mnt', 'old.ts'))).toBe(false);
      } finally {
        execFileSync('umount', [mnt]);
      }
    });
  });

  // proves: Test strategy #5 (a rollback rename that itself fails yields the
  // escalation class of #6). Nothing on a plain filesystem makes a forward rename
  // succeed while its exact reverse fails — rename(2) is symmetric — so the fault
  // is injected: rename #1 lands pair 1, rename #2 is the mid-list ENOTDIR
  // failure, rename #3 is the armed undo.
  it('reports rollbackFailed when a rollback rename fails, leaving the disk half-applied', () => {
    const { dir, headBefore } = movesRepo();
    fsFault.renames = [];
    fsFault.allowRenames = 2;
    let failure: MoveApplyFailure | null;
    try {
      failure = applyFailureOf(dir, headBefore, [
        { from: 'src/old.ts', to: 'docs/designs/002-x/old.ts' },
        { from: 'src/beta.ts', to: 'README.md/inner.ts' },
      ]);
    } finally {
      fsFault.allowRenames = Number.POSITIVE_INFINITY;
    }
    expect(failure, 'a failed undo must be reported, never swallowed').not.toBeNull();
    expect(failure?.rollbackFailed, 'a failed undo is the terminal escalation class').toBe(true);
    // The half-applied list survives — the one state nothing in-process can
    // undo, and the reason the run stops at the operator: pair 1 is still
    // moved, the failing pair left nothing behind.
    expect(existsSync(join(dir, 'src', 'old.ts'))).toBe(false);
    expect(readFileSync(join(dir, 'docs', 'designs', '002-x', 'old.ts'), 'utf8')).toBe('one\n');
    expect(existsSync(join(dir, 'src', 'beta.ts'))).toBe(true);
  });
});

// proves: Test strategy #12 — the A10 contrast, one describe per checkout shape
// so a host runs exactly one half of it.
describe.runIf(process.platform !== 'darwin')('case-sensitive checkout (Linux, A10)', () => {
  // Linux half: the disk check passes and apply lands the case-variant sibling.
  it('lands notes.md -> readme.md as a case-variant sibling', () => {
    const { dir, headBefore } = movesRepo();
    const casePair: MovePair[] = [{ from: 'notes.md', to: 'readme.md' }];
    expect(issueOf(dir, headBefore, 'implement', casePair)).toBeNull();
    expect(applyFailureOf(dir, headBefore, casePair)).toBeNull();
    expect(readFileSync(join(dir, 'readme.md'), 'utf8')).toBe('plain notes\n');
    expect(readFileSync(join(dir, 'README.md'), 'utf8')).toBe('# readme\n');
  });
});

describe.runIf(process.platform === 'darwin')('case-insensitive checkout (macOS, A10)', () => {
  // macOS half: one move round trips, and the disk check rejects the case-only
  // name where README.md is tracked.
  it('round trips one move and rejects notes.md -> readme.md at the disk check', () => {
    const { dir, headBefore } = movesRepo();
    const legalPair: MovePair[] = [{ from: 'src/old.ts', to: 'src/kept.ts' }];
    expect(applyFailureOf(dir, headBefore, legalPair)).toBeNull();
    expect(readFileSync(join(dir, 'src', 'kept.ts'), 'utf8')).toBe('one\n');
    expectRejectionNaming(
      issueOf(dir, headBefore, 'implement', [{ from: 'notes.md', to: 'readme.md' }]),
      'readme.md',
    );
  });
});

// proves: Test strategy #13 — linked worktree checkout: `.git` is a `gitdir:`
// pointer file (A11), the move still applies inside the worktree, and the
// pointer stays a byte-identical regular file.
it('applies a move inside a linked worktree and leaves the .git pointer file untouched', () => {
  const { dir, headBefore } = movesRepo();
  const wtParent = mkdtempSync(join(tmpdir(), 'bindforge-wt-'));
  const wt = join(wtParent, 'linked');
  gitAt(dir, ['worktree', 'add', '-q', '--detach', wt]);

  expect(statSync(join(wt, '.git')).isFile()).toBe(true);
  const pointerBefore = readFileSync(join(wt, '.git'), 'utf8');
  expect(pointerBefore).toContain('gitdir:');

  const pairs: MovePair[] = [{ from: 'src/old.ts', to: 'src/kept.ts' }];
  expect(issueOf(wt, headBefore, 'implement', pairs)).toBeNull();
  expect(applyFailureOf(wt, headBefore, pairs)).toBeNull();
  expect(readFileSync(join(wt, 'src', 'kept.ts'), 'utf8')).toBe('one\n');

  expect(statSync(join(wt, '.git')).isFile()).toBe(true);
  expect(readFileSync(join(wt, '.git'), 'utf8')).toBe(pointerBefore);
});
