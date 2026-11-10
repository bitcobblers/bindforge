import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync } from 'node:fs';
import { basename, join, relative } from 'node:path';

import { nextNnn } from './gates.js';
import { NNN_RE, SLUG_RE, STATE_FILE } from './types.js';

// ---------------------------------------------------------------------------
// Run identity: `NNN-slug` names the design directory, the pipeline branch
// and the checkpoint's own id, so it is derived in one place and validated
// before any state is written.
// ---------------------------------------------------------------------------

export interface RunIdentity {
  slug: string;
  nnn: string;
  kind: 'adr' | 'brief';
  branch: string;
}

/** Operator-supplied identity, e.g. `--nnn 002 --slug run2`. */
export interface IdentityPin {
  nnn?: string;
  slug?: string;
}

/**
 * Slug from the source file name, forced into SLUG_RE. The generator and the
 * checkpoint validator must agree on the grammar: run 1's own compatibility
 * reviewer flagged a validator that rejected slugs the generator produced,
 * which would have made those runs unresumable.
 */
export function slugFromSource(sourcePath: string): string {
  return sanitizeSlug(basename(sourcePath, '.md').replace(/^\d{3}-/, ''));
}

export function sanitizeSlug(raw: string): string {
  const slug = raw
    .toLowerCase()
    .replace(/[^a-z0-9-]+/g, '-')
    .replace(/-{2,}/g, '-')
    .replace(/^-+|-+$/g, '');
  return slug.length > 0 ? slug : 'untitled';
}

export function deriveIdentity(
  repoDir: string,
  sourcePath: string,
  pin: IdentityPin = {},
): RunIdentity {
  if (pin.nnn !== undefined && !NNN_RE.test(pin.nnn)) {
    throw new Error(`invalid --nnn "${pin.nnn}": expected three digits, e.g. 002`);
  }
  if (pin.slug !== undefined && !SLUG_RE.test(pin.slug)) {
    throw new Error(`invalid --slug "${pin.slug}": expected ${String(SLUG_RE)}`);
  }
  const rel = relative(repoDir, sourcePath).split(/[\\/]/).join('/');
  const kind: 'adr' | 'brief' = rel.startsWith('docs/adrs/') ? 'adr' : 'brief';
  const fromName = /^docs\/(adrs|prompts)\/(\d{3})-/.exec(rel);
  let nnn: string;
  if (pin.nnn !== undefined) {
    nnn = pin.nnn;
  } else if (kind === 'adr' && fromName?.[2]) {
    nnn = fromName[2];
  } else {
    nnn = nextNnn([
      ...listDir(join(repoDir, 'docs', 'adrs')),
      ...listDir(join(repoDir, 'docs', 'designs')),
    ]);
  }
  const slug = pin.slug ?? slugFromSource(sourcePath);
  return { slug, nnn, kind, branch: `pipeline/${nnn}-${slug}` };
}

function listDir(dir: string): string[] {
  try {
    return readdirSync(dir);
  } catch {
    return [];
  }
}

/** Repo-relative design directory for an identity. */
export function designDirOfIdentity(id: RunIdentity): string {
  return join('docs', 'designs', `${id.nnn}-${id.slug}`);
}

/**
 * Why this identity is already taken. Without this check `run` writes a fresh
 * checkpoint and re-bases on the current HEAD, then `ensureBranch` checks out
 * the existing branch: the new run continues on the old run's history with
 * its artifacts and stale review rounds still on disk.
 */
export function identityClash(repoDir: string, id: RunIdentity): string[] {
  const reasons: string[] = [];
  const current = spawnSync('git', ['rev-parse', '--abbrev-ref', 'HEAD'], {
    cwd: repoDir,
    encoding: 'utf8',
  });
  // A branch the operator prepared and is already standing on is the intended
  // base for the run (e.g. a fix-only branch): ensureBranch would be a no-op,
  // so there is nothing to resurrect.
  const onBranch = current.status === 0 && current.stdout.trim() === id.branch;
  const ref = spawnSync('git', ['show-ref', '--verify', '--quiet', `refs/heads/${id.branch}`], {
    cwd: repoDir,
  });
  if (ref.status === 0 && !onBranch) {
    reasons.push(
      `branch ${id.branch} already exists — archive it (git branch -m ${id.branch} archive/...) ` +
        'or pin a different --slug/--nnn, or check it out before starting',
    );
  }
  const dirRel = designDirOfIdentity(id);
  const dir = join(repoDir, dirRel);
  if (existsSync(join(dir, STATE_FILE))) {
    reasons.push(
      `checkpoint exists at ${join(dirRel, STATE_FILE)} — resume it, or remove ${dirRel} for a from-scratch run`,
    );
  } else if (existsSync(dir)) {
    reasons.push(`design directory ${dirRel} already exists — remove it for a from-scratch run`);
  }
  return reasons;
}
