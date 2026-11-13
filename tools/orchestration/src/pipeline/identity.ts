import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { basename, join, relative } from 'node:path';

import type { Flow } from './flows.js';
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
  flow: Flow = 'full',
): RunIdentity {
  if (pin.nnn !== undefined && !NNN_RE.test(pin.nnn)) {
    throw new Error(`invalid --nnn "${pin.nnn}": expected three digits, e.g. 002`);
  }
  if (pin.slug !== undefined && !SLUG_RE.test(pin.slug)) {
    throw new Error(`invalid --slug "${pin.slug}": expected ${String(SLUG_RE)}`);
  }
  const rel = relative(repoDir, sourcePath).split(/[\\/]/).join('/');
  const kind = kindOfRel(rel);
  const fromName = /^docs\/(adrs|prompts)\/(\d{3})-/.exec(rel);
  const slug = pin.slug ?? slugFromSource(sourcePath);
  let nnn: string;
  if (pin.nnn !== undefined) {
    nnn = pin.nnn;
  } else if (kind === 'adr' && fromName?.[2]) {
    nnn = fromName[2];
  } else {
    // A17, unchanged: the shared counter's next free number. The
    // checkpoint-identity rule below is the light flow's alone, so a
    // `flow: full` entry allocates exactly as it always did.
    const existing = flow === 'light' ? nnnOfExistingRun(repoDir, rel, slug) : undefined;
    nnn = existing ?? nextNnn(takenNames(repoDir));
  }
  return { slug, nnn, kind, branch: `pipeline/${nnn}-${slug}` };
}

/** The rule both readers share: `docs/adrs/**` is a decision, anything else is a brief. */
function kindOfRel(rel: string): 'adr' | 'brief' {
  return rel.startsWith('docs/adrs/') ? 'adr' : 'brief';
}

/**
 * Which source gate a document faces: an ADR is under `docs/adrs/`, everything
 * else is a brief. `deriveIdentity` derives the identity's kind from the same
 * rule, and `runPipeline` needs the value to parse the source's `flow:` key
 * before the identity exists — one rule, so the parse and the identity can
 * never disagree.
 */
export function sourceKind(repoDir: string, sourcePath: string): 'adr' | 'brief' {
  return kindOfRel(relative(repoDir, sourcePath).split(/[\\/]/).join('/'));
}

/** Every name the shared `NNN` counter has consumed: decisions and designs. */
function takenNames(repoDir: string): string[] {
  const adrs = listDir(join(repoDir, 'docs', 'adrs'));
  const designs = listDir(join(repoDir, 'docs', 'designs'));
  return [...adrs, ...designs];
}

/**
 * The `NNN` of the run this source document already drives, by its checkpoint.
 *
 * The light flow's rule only (`deriveIdentity`'s `flow` parameter). A brief's
 * file name carries no `NNN`, so a light run entered from the document that
 * already drives one would take the counter's next free number and write a
 * second design directory beside the plan — a light entry is re-run by the
 * same operator on the same document often enough that the checkpoint has to
 * be the authority: the design directory whose checkpoint names this document
 * IS this document's run, `identityClash` reports it taken, and the re-entry
 * lands on the identity that holds the plan.
 *
 * The full flow keeps A17's allocation untouched (its own compatibility claim:
 * "every existing document and checkpoint behaves exactly as it does today"),
 * so a brief's full entry keeps taking the next free number. A rule for that
 * path is a change to shared entry behaviour and needs its own design entry;
 * it is not this design's.
 */
function nnnOfExistingRun(repoDir: string, sourceRel: string, slug: string): string | undefined {
  for (const entry of listDir(join(repoDir, 'docs', 'designs'))) {
    const match = /^(\d{3})-(.+)$/.exec(entry);
    if (match === null) {
      continue;
    }
    const [, nnn, dirSlug] = match;
    if (nnn === undefined || !NNN_RE.test(nnn) || dirSlug !== slug) {
      continue;
    }
    try {
      const raw = JSON.parse(
        readFileSync(join(repoDir, 'docs', 'designs', entry, STATE_FILE), 'utf8'),
      ) as { source?: { path?: unknown } };
      if (raw.source?.path === sourceRel) {
        return nnn;
      }
    } catch {
      // No readable checkpoint: a design directory left behind is not a run.
    }
  }
  return undefined;
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
