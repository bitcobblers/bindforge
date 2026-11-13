import { existsSync, readFileSync, statSync } from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

// ---------------------------------------------------------------------------
// ADR-020's promised honesty test (§1, design 020 Decision 1): the executable
// surface is exactly the two prefixes — `tools/orchestration/src/` (what the
// runner imports at process start) and `tools/orchestration/prompts/` (what it
// renders at stage time). This test walks the import graph from the pipeline's
// entry points and pins that every in-repo module reached, and the stage-time
// prompt read root, fall under a prefix the rule names: a new importable
// location or read root the rule does not cover turns the suite red, exactly
// as the ADR demands. The module-scope imports are dynamic on purpose — the
// rule (`touchesExecutableSurface`) and the exported read root (`PROMPTS_DIR`)
// are what the design adds; a named static import of a missing export would
// fail this file at load for the wrong reason, before any behavior is judged.
// ---------------------------------------------------------------------------

const PKG_ROOT = resolve(fileURLToPath(new URL('../../', import.meta.url))); // tools/orchestration
const REPO_ROOT = resolve(fileURLToPath(new URL('../../../../', import.meta.url)));

const gatesModule = (await import('../../src/pipeline/gates.js')) as {
  touchesExecutableSurface?: (path: string) => boolean;
};
const pipelineCli = (await import('../../src/pipeline/cli.js')) as {
  PROMPTS_DIR?: string;
};

const ENTRY_POINTS = [
  'tools/orchestration/src/pipeline/cli.ts',
  'tools/orchestration/src/cli.ts',
  'tools/orchestration/src/opencode-runner.ts',
];

/** Every specifier this file imports from or imports(), relative or not. */
const IMPORT_RE = /(?:\bfrom\s*|\bimport\s*\(\s*|\bimport\s+)['"]([^'"]+)['"]/g;

/** Resolve one ESM specifier (always `.js` in this NodeNext package) to the
 * TypeScript source file on disk, or null when nothing backs it. */
function resolveSpecifier(fromFile: string, spec: string): string | null {
  if (!spec.startsWith('.')) {
    return null; // bare specifiers (`yargs`, `zod`, `node:*`) are not in-repo
  }
  const base = resolve(dirname(fromFile), spec);
  const candidates = [base];
  if (base.endsWith('.js')) {
    candidates.push(`${base.slice(0, -3)}.ts`);
  }
  candidates.push(join(base, 'index.ts'));
  for (const candidate of candidates) {
    if (existsSync(candidate) && statSync(candidate).isFile()) {
      return candidate;
    }
  }
  return null;
}

/** BFS the module graph from the entry points; returns every reached file
 * path that lives inside this repository, repo-root-relative. */
function walkImportGraph(): string[] {
  const seen = new Set<string>();
  const queue = ENTRY_POINTS.map((entry) => resolve(REPO_ROOT, entry));
  for (const entry of queue) {
    if (!existsSync(entry)) {
      throw new Error(`pipeline entry point missing: ${entry}`);
    }
  }
  while (queue.length > 0) {
    const file = queue.shift() as string;
    if (seen.has(file)) {
      continue;
    }
    seen.add(file);
    const text = readFileSync(file, 'utf8');
    for (const match of text.matchAll(IMPORT_RE)) {
      const resolved = resolveSpecifier(file, match[1] ?? '');
      if (resolved !== null && relative(REPO_ROOT, resolved).split(/[\\/]/)[0] !== '..') {
        queue.push(resolved);
      }
    }
  }
  return [...seen].map((file) => relative(REPO_ROOT, file)).sort();
}

describe('the executable surface covers everything the runner runs and renders', () => {
  const surfaceRule = (): ((path: string) => boolean) => {
    if (typeof gatesModule.touchesExecutableSurface !== 'function') {
      throw new Error('gates.js exports no touchesExecutableSurface yet');
    }
    return gatesModule.touchesExecutableSurface;
  };

  // proves: Test strategy #7 — walking the import graph from `src/pipeline/cli.ts`,
  // `src/cli.ts` and `src/opencode-runner.ts`, every in-repo module reached is
  // under `tools/orchestration/src/`, and the surface rule answers true for
  // every one of them. A new importable location (say `tools/orchestration/
  // scripts/`) appears in the walk and fails it twice over: outside the src
  // prefix, and outside the rule.
  it('covers every module the pipeline entry points import', () => {
    const reached = walkImportGraph();
    // The graph is real, not an empty walk: the research read counted 15 TS
    // files under src/, all imported from these entry points at process start.
    expect(reached.length).toBeGreaterThanOrEqual(10);

    const outsideSrc = reached.filter(
      (f) => !f.startsWith(join('tools', 'orchestration', 'src') + '/'),
    );
    expect(outsideSrc).toEqual([]);

    const touches = surfaceRule();
    for (const module of reached) {
      expect(touches(module), `surface rule misses ${module}`).toBe(true);
    }
  });

  // proves: Test strategy #7 — the stage-time read root is pinned too: the
  // prompt read root (`PROMPTS_DIR`, cli.ts:20) resolves under
  // `tools/orchestration/prompts/`, the templates are really there, and the
  // rule answers true for the templates it renders. Together with the import
  // walk this names both §1 prefixes: a new read root or a rule that forgets
  // one of the two goes red here.
  it('covers the stage-time prompt read root', () => {
    const promptsDir = pipelineCli.PROMPTS_DIR;
    if (typeof promptsDir !== 'string' || promptsDir === '') {
      throw new Error('src/pipeline/cli.ts exports no PROMPTS_DIR read root yet');
    }
    // A package-relative read root resolves from the module that declares it.
    const abs = resolve(PKG_ROOT, 'src', 'pipeline', promptsDir);
    const promptsRoot = join(PKG_ROOT, 'prompts');
    const rel = relative(promptsRoot, abs);
    expect(rel.startsWith('..'), `PROMPTS_DIR escapes ${promptsRoot}`).toBe(false);
    expect(isAbsolute(rel), `PROMPTS_DIR escapes the package prompts root`).toBe(false);

    const repoRel = relative(REPO_ROOT, abs);
    const aRealTemplate = join(repoRel, 'implement.md');
    expect(existsSync(resolve(REPO_ROOT, aRealTemplate)), aRealTemplate).toBe(true);
    const touches = surfaceRule();
    expect(touches(aRealTemplate)).toBe(true);
  });
});
