import { spawnSync } from 'node:child_process';
import { lstatSync, realpathSync, renameSync } from 'node:fs';
import { isAbsolute, join, posix, relative } from 'node:path';

import { isStageInFlow } from './flows.js';
import type { FlowId } from './flows.js';
import { stagePathAllowed } from './gates.js';
import { LOCK_FILE, type MovePair, STATE_FILE } from './types.js';

// ---------------------------------------------------------------------------
// ADR-005: a file move is data in a stage reply, never a shell command the
// stage runs. `validateMoves` checks a whole list against `headBefore` plus the
// current disk BEFORE anything is touched; `applyMoves` lands it with
// `fs.renameSync` under the `.pipeline.lock` single-writer invariant and rolls
// a failed list back in reverse, so a failed apply is all-or-nothing too.
//
// A rejection is a plain issue string naming the offending pair, not a thrown
// error: the runner turns it into `invalid-reply: moves — <issue>`, and a rule
// that threw internally must still land as an invalid reply, never a crash.
// `applyMoves` returns a structured failure because the runner must tell an
// environmental apply failure (retry with a fresh session) from a failed
// rollback (escalate to the operator on a known-damaged worktree).
// ---------------------------------------------------------------------------

/** A failed apply. The two classes pick the runner's two different outcomes. */
export interface MoveApplyFailure {
  /** What went wrong, naming the pair. */
  error: string;
  /**
   * A rollback rename failed too. Nothing in-process can undo that, so the
   * runner stops at the operator instead of starting attempt 2 on a half-moved
   * worktree.
   */
  rollbackFailed: boolean;
}

/** Tree modes of regular files. `040000` trees and `120000` symlinks are out. */
const REGULAR_MODES = new Set(['100644', '100755']);

/** Windows device names, bare or with an extension: never a legal segment. */
const WINDOWS_RESERVED = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i;

/** Case-folded comparison: the disjointness rules are folded on every platform. */
function fold(path: string): string {
  return path.toLowerCase();
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** git plumbing for the validator: a non-zero exit reads as "not there", never a throw. */
function gitOut(repoDir: string, args: string[]): string | null {
  const res = spawnSync('git', args, { cwd: repoDir, encoding: 'utf8' });
  return res.status === 0 ? res.stdout : null;
}

/**
 * The mode of `relPath` in `rev`'s tree, or null when it is not tracked there.
 * The pathspec is `:(literal)`: git glob-matches a bare pathspec by default, so
 * `a?.ts` would match a tracked `aX.ts` and rule 4's regular-file test would
 * pass on a path that is not itself tracked — and a legitimately named file
 * such as `bindings[en].json` would fail it. `--` does not disable the globbing;
 * the magic prefix does. Rule 1 has already rejected any `:` in an endpoint, so
 * nothing else can reach the pathspec, and magic signatures date to git 1.9, so
 * this adds no git floor (Compatibility matrix: "git, any version").
 */
function treeMode(repoDir: string, rev: string, relPath: string): string | null {
  const out = gitOut(repoDir, ['ls-tree', rev, '--', `:(literal)${relPath}`]);
  if (out === null) {
    return null;
  }
  const mode = /^\S+/.exec(out)?.[0];
  return mode === undefined ? null : mode;
}

/**
 * Any dentry, a dangling symlink included: `rename(2)` replaces one silently,
 * and on a case-insensitive checkout the lookup itself is the guard that a
 * case-variant name is already taken.
 */
function existsAsDentry(absPath: string): boolean {
  try {
    lstatSync(absPath);
    return true;
  } catch {
    return false;
  }
}

function realpathOrNull(absPath: string): string | null {
  try {
    return realpathSync(absPath);
  } catch {
    return null;
  }
}

function hasControlChar(path: string): boolean {
  for (const ch of path) {
    const code = ch.codePointAt(0) ?? 0;
    if (code < 0x20 || code === 0x7f) {
      return true;
    }
  }
  return false;
}

/** Rule 1: repo-relative POSIX shape, judged identically on every platform. */
function shapeIssue(side: 'from' | 'to', pair: MovePair): string | null {
  const path = pair[side];
  if (path.includes('\\') || path.includes(':') || hasControlChar(path)) {
    return `${side} names a path with a backslash, a colon, or a control character: ${path}`;
  }
  if (path.startsWith('/')) {
    return `${side} must be repo-relative, not absolute: ${path}`;
  }
  for (const segment of path.split('/')) {
    if (segment === '') {
      return `${side} has an empty path segment: ${path}`;
    }
    if (segment === '.' || segment === '..') {
      return `${side} has a . or .. segment: ${path}`;
    }
    if (WINDOWS_RESERVED.test(segment) || WINDOWS_RESERVED.test(segment.split('.')[0] ?? '')) {
      return `${side} has a Windows reserved-name segment: ${path}`;
    }
    if (segment.endsWith('.') || segment.endsWith(' ')) {
      return `${side} has a segment ending in a dot or a space: ${path}`;
    }
  }
  return null;
}

/** Rule 2: the runner's own files and the git directory are never endpoints. */
function reservedIssue(side: 'from' | 'to', pair: MovePair): string | null {
  for (const segment of pair[side].split('/')) {
    if (segment === STATE_FILE) {
      return `${side} names the run checkpoint: ${pair[side]}`;
    }
    if (segment === LOCK_FILE) {
      return `${side} names the run lock: ${pair[side]}`;
    }
    if (segment.toLowerCase() === '.git') {
      return `${side} names the git directory: ${pair[side]}`;
    }
  }
  return null;
}

/**
 * Rule 3: both endpoints must sit inside the stage's ownership, the same
 * `stagePathAllowed` predicate the ownership sweep applies. That shared
 * predicate is why the sweep needs no move-specific branch: a validated pair's
 * delete and its add are both sanctioned by construction.
 */
function ownershipIssue(stage: string, pair: MovePair, flow: FlowId = 'full'): string | null {
  // A move a flow never runs is not an ownership question: the stage does not
  // exist in this flow at all, so neither endpoint is reachable. `stagePathAllowed`
  // alone would not catch this (a light `test-update` owns `tests/**` by the
  // shared rule), so the flow's stage list is the validator's source of truth.
  if (!isStageInFlow(stage, flow)) {
    return `stage ${stage} does not run in flow ${flow} (${pair.from} -> ${pair.to})`;
  }
  for (const side of ['from', 'to'] as const) {
    if (!stagePathAllowed(stage, pair[side], flow)) {
      return `stage ${stage} does not own the ${side} path: ${pair[side]}`;
    }
  }
  return null;
}

/** True when `target` is `repoReal` itself or below it. */
function insideRepo(repoReal: string, target: string): boolean {
  const rel = relative(repoReal, target);
  return rel === '' || (rel !== '..' && !rel.startsWith('../') && !isAbsolute(rel));
}

function namesGitDir(repoReal: string, target: string): boolean {
  const rel = relative(repoReal, target);
  if (rel === '') {
    return false;
  }
  return rel.split(/[\\/]/).some((segment) => segment.toLowerCase() === '.git');
}

/**
 * Rules 4-7: the endpoints against `headBefore` and the disk. Split out from
 * rules 1-3 because `applyMoves` re-runs exactly these as one guard across the
 * inference-to-finalize window.
 */
function endpointsIssue(
  repoDir: string,
  repoReal: string,
  headBefore: string,
  pair: MovePair,
): string | null {
  // Rule 4: `from` is a tracked regular file at headBefore and still on disk.
  const mode = treeMode(repoDir, headBefore, pair.from);
  if (mode === null || !REGULAR_MODES.has(mode)) {
    return `from is not a regular file tracked at the stage's base commit: ${pair.from}`;
  }
  const fromAbs = join(repoDir, pair.from);
  if (!existsAsDentry(fromAbs)) {
    return `from is tracked but not on disk: ${pair.from}`;
  }
  // Rule 5: `to` exists nowhere, while its parent does — so a missing parent is
  // a reply error here, not an environmental apply failure later.
  if (existsAsDentry(join(repoDir, pair.to))) {
    return `to already exists on disk: ${pair.to}`;
  }
  if (treeMode(repoDir, headBefore, pair.to) !== null) {
    return `to is tracked at the stage's base commit: ${pair.to}`;
  }
  const toParentRel = posix.dirname(pair.to);
  const toParentAbs = join(repoDir, toParentRel);
  if (!existsAsDentry(toParentAbs)) {
    return `to's parent directory does not exist: ${pair.to}`;
  }
  // Rule 7: both resolved endpoints stay inside the repo's realpath, which also
  // catches a symlink escaping into the real gitdir; the `.git` test then runs
  // on the resolved path.
  const fromReal = realpathOrNull(fromAbs);
  const parentReal = realpathOrNull(toParentAbs);
  if (fromReal === null) {
    return `from does not resolve inside the repository: ${pair.from}`;
  }
  if (parentReal === null) {
    return `to does not resolve inside the repository: ${pair.to}`;
  }
  const resolved: Array<['from' | 'to', string]> = [
    ['from', fromReal],
    ['to', parentReal],
  ];
  for (const [side, target] of resolved) {
    if (!insideRepo(repoReal, target)) {
      return `${side} resolves outside the repository: ${pair[side]}`;
    }
    if (namesGitDir(repoReal, target)) {
      return `${side} resolves inside the git directory: ${pair[side]}`;
    }
  }
  return null;
}

/** True when `shorter` is a proper path prefix of `longer`. */
function nests(shorter: string, longer: string): boolean {
  return longer.startsWith(`${shorter}/`);
}

/**
 * Rule 6: the list must be a matching whose apply order is irrelevant — all
 * `from`s distinct, all `to`s distinct, no `to` equals any `from`, no listed
 * path a path-prefix of another, every comparison case-folded. The fold is
 * deliberate conservatism: a repo with case-variant filenames cannot move both
 * in one reply, and case-only renames are deferred.
 */
function matchingIssue(moves: readonly MovePair[]): string | null {
  const label = (pair: MovePair): string => `${pair.from} -> ${pair.to}`;
  for (let i = 0; i < moves.length; i++) {
    const a = moves[i] as MovePair;
    if (fold(a.from) === fold(a.to)) {
      return `a move differs only in case: ${label(a)}`;
    }
    if (nests(fold(a.from), fold(a.to)) || nests(fold(a.to), fold(a.from))) {
      return `a move nests inside itself: ${label(a)}`;
    }
    for (let j = i + 1; j < moves.length; j++) {
      const b = moves[j] as MovePair;
      if (fold(a.from) === fold(b.from)) {
        return `two moves share one from: ${label(a)} and ${label(b)}`;
      }
      if (fold(a.to) === fold(b.to)) {
        return `two moves land on one to: ${label(a)} and ${label(b)}`;
      }
      if (fold(a.to) === fold(b.from) || fold(b.to) === fold(a.from)) {
        return `one move lands where another moves away from: ${label(a)} and ${label(b)}`;
      }
      const endsA = [fold(a.from), fold(a.to)];
      const endsB = [fold(b.from), fold(b.to)];
      if (endsA.some((x) => endsB.some((y) => nests(x, y) || nests(y, x)))) {
        return `one move's path contains another's: ${label(a)} and ${label(b)}`;
      }
    }
  }
  return null;
}

/**
 * The whole-list validation: rules 1-7, every check against `headBefore` plus
 * the current disk, nothing touched. Returns null when the list is acceptable
 * and otherwise an issue naming the offending pair.
 */
export function validateMoves(
  repoDir: string,
  headBefore: string,
  stage: string,
  moves: readonly MovePair[],
  flow: FlowId = 'full',
): string | null {
  const repoReal = realpathOrNull(repoDir);
  if (repoReal === null) {
    return `the repository directory is not readable: ${repoDir}`;
  }
  for (const pair of moves) {
    for (const side of ['from', 'to'] as const) {
      const issue = shapeIssue(side, pair) ?? reservedIssue(side, pair);
      if (issue !== null) {
        return issue;
      }
    }
    const issue =
      ownershipIssue(stage, pair, flow) ?? endpointsIssue(repoDir, repoReal, headBefore, pair);
    if (issue !== null) {
      return issue;
    }
  }
  return matchingIssue(moves);
}

/**
 * Land a validated list, in reply order, with `rename(2)`. Immediately before
 * the first rename it re-runs rules 4-7 as one guard across the inference-to-
 * finalize window; a guard refusal is the same apply failure as a rename error.
 * On the first error the completed renames roll back in reverse, which is exact
 * because the pairs are disjoint.
 */
export function applyMoves(
  repoDir: string,
  headBefore: string,
  moves: readonly MovePair[],
): MoveApplyFailure | null {
  if (moves.length === 0) {
    return null;
  }
  const repoReal = realpathOrNull(repoDir);
  if (repoReal === null) {
    return {
      error: `the repository directory is not readable: ${repoDir}`,
      rollbackFailed: false,
    };
  }
  for (const pair of moves) {
    const issue = endpointsIssue(repoDir, repoReal, headBefore, pair);
    if (issue !== null) {
      return { error: `the moves list no longer applies: ${issue}`, rollbackFailed: false };
    }
  }
  const done: MovePair[] = [];
  for (const pair of moves) {
    try {
      renameSync(join(repoDir, pair.from), join(repoDir, pair.to));
      done.push(pair);
    } catch (error) {
      const failure = `rename ${pair.from} -> ${pair.to}: ${messageOf(error)}`;
      for (const moved of done.reverse()) {
        try {
          renameSync(join(repoDir, moved.to), join(repoDir, moved.from));
        } catch (rollbackError) {
          return {
            error: `${failure}; undo ${moved.to} -> ${moved.from}: ${messageOf(rollbackError)}`,
            rollbackFailed: true,
          };
        }
      }
      return { error: failure, rollbackFailed: false };
    }
  }
  return null;
}
