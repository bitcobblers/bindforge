import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  type ParsedSource,
  flipTaskStatus,
  parseSource,
  sourceAdmissible,
  taskDependenciesGate,
} from '../../src/pipeline/gates.js';
import {
  type RunIdentity,
  deriveIdentity,
  designDirOfIdentity,
  identityClash,
  nextTaskNnn,
  slugFromSource,
  sourceKind,
} from '../../src/pipeline/identity.js';
import {
  type PipelineState,
  designDirCandidates,
  designDirOf,
  loadState,
  loadStateForSlug,
  saveState,
} from '../../src/pipeline/state-store.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// ADR-023: tasks are a second source kind. These tests pin the contract:
// the T-NNN naming and the separate number series (023-C2/C5), the task
// admission gate and the doc_type/path agreement rule (023-C1/C4), the
// segregated artifact root and the two-root slug lookup (023-C3), the
// pending-to-completed flip (023-L3), and the dependency gate (023-L4).
// ---------------------------------------------------------------------------

const TASK_ID: RunIdentity = {
  slug: 'add-auth-service',
  nnn: '001',
  kind: 'task',
  branch: 'pipeline/001-add-auth-service',
};

function tmpRepo(): string {
  return mkTmpDir('bindforge-task-');
}

/** A minimal admissible task document at docs/tasks/T-001-add-auth-service.md. */
function writeTask(
  dir: string,
  name = 'T-001-add-auth-service.md',
  front = [
    'doc_type: task',
    'status: pending',
    'type: feature',
    'functional_area: backend',
    'associated_adr: ADR-023',
  ].join('\n'),
  body = 'Build the auth service endpoint.\n',
): string {
  mkdirSync(join(dir, 'docs', 'tasks'), { recursive: true });
  const path = join(dir, 'docs', 'tasks', name);
  writeFileSync(path, `---\n${front}\n---\n\n${body}`);
  return path;
}

function taskState(partial: Partial<PipelineState> = {}): PipelineState {
  return {
    version: 1,
    slug: 'add-auth-service',
    nnn: '001',
    branch: 'pipeline/001-add-auth-service',
    source: { kind: 'task', path: 'docs/tasks/T-001-add-auth-service.md' },
    status: 'running',
    lastStage: '',
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
    ...partial,
  };
}

describe('slugFromSource', () => {
  it('drops the T-NNN prefix of a task source', () => {
    expect(slugFromSource('/repo/docs/tasks/T-001-add-auth-service.md')).toBe('add-auth-service');
  });

  it('still drops the NNN prefix of an ADR source', () => {
    expect(slugFromSource('/repo/docs/adrs/002-parallel-inference-lane.md')).toBe(
      'parallel-inference-lane',
    );
  });
});

describe('sourceKind', () => {
  it('classifies docs/tasks as task, docs/adrs as adr, everything else as brief', () => {
    const dir = tmpRepo();
    expect(sourceKind(dir, join(dir, 'docs/tasks/T-001-x.md'))).toBe('task');
    expect(sourceKind(dir, join(dir, 'docs/adrs/002-x.md'))).toBe('adr');
    expect(sourceKind(dir, join(dir, 'docs/prompts/x.md'))).toBe('brief');
  });
});

describe('deriveIdentity (tasks)', () => {
  it('derives kind, nnn and slug from the task file name', () => {
    const dir = tmpRepo();
    writeTask(dir);
    expect(deriveIdentity(dir, join(dir, 'docs/tasks/T-001-add-auth-service.md'))).toEqual(TASK_ID);
  });

  it('counts task numbers in their own series, from 001', () => {
    const dir = tmpRepo();
    // ADRs go up to 005: the task series must not see them.
    mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
    writeFileSync(join(dir, 'docs/adrs/005-some-decision.md'), 'x\n');
    expect(nextTaskNnn(dir)).toBe('001');
  });

  it('allocates the next free task number from sources and artifacts', () => {
    const dir = tmpRepo();
    writeTask(dir, 'T-001-first.md');
    writeTask(dir, 'T-003-third.md');
    // T-002's source was deleted, but its artifacts still reserve the number.
    mkdirSync(join(dir, 'docs', 'designs', 'tasks', '002-second'), { recursive: true });
    expect(nextTaskNnn(dir)).toBe('004');
  });

  it('keeps the shared ADR/brief counter independent of the task series', () => {
    const dir = tmpRepo();
    writeTask(dir, 'T-001-first.md');
    writeTask(dir, 'T-002-second.md');
    mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
    writeFileSync(join(dir, 'docs/adrs/003-decision.md'), 'x\n');
    // A brief takes the shared counter's next free number: 004 — it does not
    // stop at the task series' 002, and it is not blocked by it either.
    mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
    const id = deriveIdentity(dir, join(dir, 'docs/prompts/add-timeout.md'));
    expect(id.kind).toBe('brief');
    expect(id.nnn).toBe('004');
  });
});

describe('design directories (tasks)', () => {
  it('routes a task identity to the segregated root', () => {
    expect(designDirOfIdentity(TASK_ID)).toBe(
      join('docs', 'designs', 'tasks', '001-add-auth-service'),
    );
  });

  it('leaves ADR and brief identities on the top-level root', () => {
    expect(
      designDirOfIdentity({
        slug: 'x',
        nnn: '002',
        kind: 'adr',
        branch: 'pipeline/002-x',
      }),
    ).toBe(join('docs', 'designs', '002-x'));
  });

  it('designDirOf follows the checkpoint source kind', () => {
    expect(designDirOf(taskState())).toBe(join('docs', 'designs', 'tasks', '001-add-auth-service'));
    expect(
      designDirOf(
        taskState({
          source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
          slug: 'x',
          nnn: '002',
        }),
      ),
    ).toBe(join('docs', 'designs', '002-x'));
  });

  it('identityClash sees the task design directory', () => {
    const dir = tmpRepo();
    const designDir = join(dir, 'docs', 'designs', 'tasks', '001-add-auth-service');
    mkdirSync(designDir, { recursive: true });
    writeFileSync(join(designDir, '.pipeline.json'), '{}\n');
    const reasons = identityClash(dir, TASK_ID);
    expect(reasons.some((r) => r.includes('checkpoint exists'))).toBe(true);
  });
});

describe('parseSource (tasks)', () => {
  it('carries the ADR-023 keys verbatim', () => {
    const doc: ParsedSource = parseSource(
      '---\ndoc_type: task\nstatus: pending\ntype: feature\nfunctional_area: backend\ndepends_on: [T-001-a, "T-002-b"]\n---\n\nbody\n',
      'task',
    );
    expect(doc.docType).toBe('task');
    expect(doc.status).toBe('pending');
    expect(doc.type).toBe('feature');
    expect(doc.functionalArea).toBe('backend');
    expect(doc.dependsOn).toEqual(['T-001-a', 'T-002-b']);
  });

  it('answers undefined and [] for absent keys', () => {
    const doc = parseSource('---\nstatus: pending\n---\n\nbody\n', 'task');
    expect(doc.docType).toBeUndefined();
    expect(doc.type).toBeUndefined();
    expect(doc.functionalArea).toBeUndefined();
    expect(doc.dependsOn).toEqual([]);
  });
});

describe('sourceAdmissible (tasks)', () => {
  const dir = tmpRepo();

  it('admits a pending, complete task', async () => {
    const path = writeTask(dir);
    const verdict = await sourceAdmissible(path, 'task');
    expect(verdict).toEqual({ proceed: true, reason: 'task is pending and complete' });
  });

  it('refuses a misnamed task file (023-L2)', async () => {
    const path = writeTask(dir, 'add-auth-service.md');
    const verdict = await sourceAdmissible(path, 'task');
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/must be named T-NNN-slug\.md/);
  });

  it('refuses a task without doc_type: task', async () => {
    const path = writeTask(
      dir,
      'T-001-no-type.md',
      'status: pending\ntype: feature\nfunctional_area: backend',
    );
    const verdict = await sourceAdmissible(path, 'task');
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/no doc_type: task/);
  });

  it('refuses a task whose status is not pending', async () => {
    const path = writeTask(
      dir,
      'T-001-done.md',
      'doc_type: task\nstatus: completed\ntype: feature\nfunctional_area: backend',
    );
    const verdict = await sourceAdmissible(path, 'task');
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/expected "pending"/);
  });

  it('refuses a task missing type or functional_area', async () => {
    const noType = writeTask(
      dir,
      'T-002-no-type.md',
      'doc_type: task\nstatus: pending\nfunctional_area: backend',
    );
    expect((await sourceAdmissible(noType, 'task')).reason).toMatch(/"type:"/);
    const noArea = writeTask(
      dir,
      'T-003-no-area.md',
      'doc_type: task\nstatus: pending\ntype: feature',
    );
    expect((await sourceAdmissible(noArea, 'task')).reason).toMatch(/"functional_area:"/);
  });

  it('refuses a task whose body is empty', async () => {
    const path = writeTask(
      dir,
      'T-004-empty.md',
      'doc_type: task\nstatus: pending\ntype: feature\nfunctional_area: backend',
      '',
    );
    const verdict = await sourceAdmissible(path, 'task');
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/body is empty/);
  });

  it('refuses a doc_type that disagrees with the path', async () => {
    const path = writeTask(
      dir,
      'T-005-mismatch.md',
      'doc_type: adr\nstatus: pending\ntype: feature\nfunctional_area: backend',
    );
    const verdict = await sourceAdmissible(path, 'task');
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/disagrees with the source path/);
  });

  it('admits a backfilled ADR that carries doc_type: adr (regression)', async () => {
    mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
    const path = join(dir, 'docs/adrs/006-backfilled.md');
    writeFileSync(
      path,
      '---\ndoc_type: adr\nstatus: proposed\n---\n\n# ADR 006\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
    );
    const verdict = await sourceAdmissible(path, 'adr');
    expect(verdict.proceed).toBe(true);
  });

  it('refuses an ADR or brief whose doc_type disagrees with the path', async () => {
    mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
    const adr = join(dir, 'docs/adrs/007-mismatch.md');
    writeFileSync(
      adr,
      '---\ndoc_type: task\nstatus: proposed\n---\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
    );
    expect((await sourceAdmissible(adr, 'adr')).reason).toMatch(/disagrees with the source path/);

    mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
    const brief = join(dir, 'docs/prompts/mismatch.md');
    writeFileSync(brief, '---\ndoc_type: adr\n---\n\na brief\n');
    expect((await sourceAdmissible(brief, 'brief')).reason).toMatch(
      /disagrees with the source path/,
    );
  });

  it('still admits a brief without any doc_type (regression)', async () => {
    mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
    const brief = join(dir, 'docs/prompts/plain-brief.md');
    writeFileSync(brief, 'a plain brief\n');
    expect((await sourceAdmissible(brief, 'brief')).proceed).toBe(true);
  });
});

describe('taskDependenciesGate (023-L4)', () => {
  const dir = tmpRepo();

  /** A task design directory checkpointing the named source. */
  function taskCheckpoint(dep: string, status: string, dirSlug?: string): void {
    const nnn = dep.slice(2, 5);
    const nameSlug = dep.slice(5);
    const designDir = join(dir, 'docs', 'designs', 'tasks', `${nnn}-${dirSlug ?? nameSlug}`);
    mkdirSync(designDir, { recursive: true });
    writeFileSync(
      join(designDir, '.pipeline.json'),
      JSON.stringify({
        version: 1,
        source: { kind: 'task', path: `docs/tasks/${dep}.md` },
        status,
      }),
    );
  }

  it('allows with no dependencies', () => {
    expect(taskDependenciesGate(dir, [])).toEqual({
      proceed: true,
      reason: 'no dependencies declared',
    });
  });

  it('refuses a malformed entry', () => {
    const verdict = taskDependenciesGate(dir, ['001-no-prefix']);
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/not a T-NNN-slug task name/);
  });

  it('refuses a dependency whose source file is missing', () => {
    const verdict = taskDependenciesGate(dir, ['T-009-ghost']);
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/does not exist/);
  });

  it('refuses a dependency that has never been run', () => {
    writeTask(dir, 'T-007-never.md');
    const verdict = taskDependenciesGate(dir, ['T-007-never']);
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/no run/);
  });

  it('refuses a dependency whose run is not done', () => {
    writeTask(dir, 'T-008-running.md');
    taskCheckpoint('T-008-running', 'running');
    const verdict = taskDependenciesGate(dir, ['T-008-running']);
    expect(verdict.proceed).toBe(false);
    expect(verdict.reason).toMatch(/not done \(status: running\)/);
  });

  it('allows every satisfied dependency', () => {
    writeTask(dir, 'T-010-done.md');
    taskCheckpoint('T-010-done', 'done');
    const verdict = taskDependenciesGate(dir, ['T-010-done']);
    expect(verdict).toEqual({ proceed: true, reason: '1 dependency(ies) satisfied' });
  });

  it('resolves a --slug-pinned run through the checkpoint source path', () => {
    writeTask(dir, 'T-011-pinned.md');
    taskCheckpoint('T-011-pinned', 'done', 'run2');
    const verdict = taskDependenciesGate(dir, ['T-011-pinned']);
    expect(verdict.proceed).toBe(true);
  });
});

describe('flipTaskStatus (023-L3)', () => {
  it('flips pending to completed', () => {
    const out = flipTaskStatus(
      '---\ndoc_type: task\nstatus: pending\ntype: feature\n---\n\nbody\n',
    );
    expect(out).not.toBeNull();
    expect(out).toContain('status: completed');
    expect(out).not.toContain('status: pending');
  });

  it('refuses to touch a task that is not pending', () => {
    expect(flipTaskStatus('---\nstatus: completed\n---\n\nbody\n')).toBeNull();
    expect(flipTaskStatus('---\nstatus: proposed\n---\n\nbody\n')).toBeNull();
  });

  it('refuses content without frontmatter', () => {
    expect(flipTaskStatus('no frontmatter here\n')).toBeNull();
  });
});

describe('slug lookup across design roots (023-C3)', () => {
  it('names the ADR root first, the task root second', () => {
    expect(designDirCandidates('001-x')).toEqual([
      join('docs', 'designs', '001-x'),
      join('docs', 'designs', 'tasks', '001-x'),
    ]);
  });

  it('finds a task checkpoint in the segregated root', async () => {
    const dir = tmpRepo();
    const designDir = join(dir, 'docs', 'designs', 'tasks', '001-add-auth-service');
    mkdirSync(designDir, { recursive: true });
    await saveState(designDir, taskState());
    const loaded = await loadStateForSlug(dir, '001-add-auth-service');
    expect(loaded).not.toBeNull();
    expect(loaded?.dir).toBe(join('docs', 'designs', 'tasks', '001-add-auth-service'));
    expect(loaded?.state.source.kind).toBe('task');
  });

  it('still finds an ADR checkpoint in the top-level root', async () => {
    const dir = tmpRepo();
    const designDir = join(dir, 'docs', 'designs', '002-x');
    mkdirSync(designDir, { recursive: true });
    await saveState(
      designDir,
      taskState({
        slug: 'x',
        nnn: '002',
        source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
      }),
    );
    const loaded = await loadStateForSlug(dir, '002-x');
    expect(loaded?.dir).toBe(join('docs', 'designs', '002-x'));
    expect(loaded?.state.source.kind).toBe('adr');
  });

  it('returns null when neither root holds the slug', async () => {
    const dir = tmpRepo();
    expect(await loadStateForSlug(dir, '003-nowhere')).toBeNull();
  });

  it('refuses a slug present in both roots', async () => {
    const dir = tmpRepo();
    const top = join(dir, 'docs', 'designs', '004-x');
    const under = join(dir, 'docs', 'designs', 'tasks', '004-x');
    mkdirSync(top, { recursive: true });
    mkdirSync(under, { recursive: true });
    await saveState(
      top,
      taskState({ nnn: '004', slug: 'x', source: { kind: 'adr', path: 'docs/adrs/004-x.md' } }),
    );
    await saveState(under, taskState({ nnn: '004', slug: 'x' }));
    await expect(loadStateForSlug(dir, '004-x')).rejects.toThrow(/ambiguous slug/);
  });
});

describe('checkpoint round-trip (tasks)', () => {
  it('saves and reloads a task-kind checkpoint', async () => {
    const dir = tmpRepo();
    const designDir = join(dir, 'docs', 'designs', 'tasks', '001-add-auth-service');
    mkdirSync(designDir, { recursive: true });
    await saveState(designDir, taskState());
    const loaded = await loadState(designDir);
    expect(loaded).not.toBeNull();
    expect(loaded?.source).toEqual({ kind: 'task', path: 'docs/tasks/T-001-add-auth-service.md' });
  });

  it('loads a pre-023 checkpoint with an adr source unchanged (compat)', async () => {
    const dir = tmpRepo();
    const designDir = join(dir, 'docs', 'designs', '005-legacy');
    mkdirSync(designDir, { recursive: true });
    // The raw JSON is what a pre-023 runner wrote: no `task` member anywhere.
    writeFileSync(
      join(designDir, '.pipeline.json'),
      JSON.stringify(
        taskState({
          nnn: '005',
          slug: 'legacy',
          source: { kind: 'adr', path: 'docs/adrs/005-legacy.md' },
        }),
      ),
    );
    const loaded = await loadState(designDir);
    expect(loaded?.source.kind).toBe('adr');
  });
});
