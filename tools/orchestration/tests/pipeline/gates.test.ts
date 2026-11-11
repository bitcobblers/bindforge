import { execFileSync } from 'node:child_process';
import { mkdtemp, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { afterEach, describe, expect, it } from 'vitest';

import {
  allReviewsApproved,
  flipAdrStatus,
  isSemanticCommit,
  nextNnn,
  packageDirs,
  parseSource,
  sourceAdmissible,
  stagePathAllowed,
} from '../../src/pipeline/gates.js';
import { DEFAULT_CONFIG } from '../../src/pipeline/stages.js';
import type { PipelineState, VerdictReply } from '../../src/pipeline/types.js';

const approve: VerdictReply = { verdict: 'approve', findings: [] };
const rejectBlocker: VerdictReply = {
  verdict: 'request-changes',
  findings: [{ severity: 'blocker', location: '§2', problem: 'broken' }],
};
const approveWithNit: VerdictReply = {
  verdict: 'approve',
  findings: [{ severity: 'nit', location: '§3', problem: 'wording' }],
};

function stateWith(
  phase: string,
  round: number,
  verdicts: Record<string, VerdictReply>,
): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '001',
    branch: 'pipeline/001-x',
    source: { kind: 'brief', path: 'docs/prompts/x.md' },
    status: 'running',
    lastStage: `${phase}-review`,
    designRound: round,
    codeRound: round,
    designBudgetUsed: 1,
    codeBudgetUsed: 1,
    editorialDone: false,
    testsGreen: false,
    testsNeedUpdate: false,
    stageFailures: {},
    forceNext: null,
    results: Object.entries(verdicts).map(([specialty, reply]) => ({
      stage: `${phase}-review:${specialty}`,
      round,
      sessionID: 'ses',
      agent: 'a',
      at: 0,
      reply,
      transcript: 't',
    })),
    gateLog: [],
    startedAt: 0,
    updatedAt: 0,
  };
}

describe('sourceAdmissible', () => {
  let dir: string;

  afterEach(async () => {
    // fresh temp dir per test via mkdtemp below
    void dir;
  });

  async function tempFile(name: string, content: string): Promise<string> {
    dir = await mkdtemp(join(tmpdir(), 'bindforge-src-'));
    const path = join(dir, name);
    await writeFile(path, content, 'utf8');
    return path;
  }

  it('admits a proposed ADR with Context and Decision', async () => {
    const path = await tempFile(
      '007-thing.md',
      [
        '---',
        'status: proposed',
        '---',
        '# ADR 007: thing',
        '## Context',
        'why.',
        '## Decision',
        'what.',
      ].join('\n'),
    );
    const g = await sourceAdmissible(path, 'adr');
    expect(g.proceed).toBe(true);
  });

  it('denies an already-accepted ADR', async () => {
    const path = await tempFile(
      '007-thing.md',
      ['---', 'status: accepted', '---', '## Context', 'x', '## Decision', 'y'].join('\n'),
    );
    const g = await sourceAdmissible(path, 'adr');
    expect(g.proceed).toBe(false);
    expect(g.reason).toContain('accepted');
  });

  it('denies an ADR with an empty Decision section', async () => {
    const path = await tempFile(
      '007-thing.md',
      [
        '---',
        'status: proposed',
        '---',
        '## Context',
        'x',
        '## Decision',
        '   ',
        '## Consequences',
        'z',
      ].join('\n'),
    );
    const g = await sourceAdmissible(path, 'adr');
    expect(g.proceed).toBe(false);
    expect(g.reason).toContain('Decision');
  });

  it('admits a non-empty brief and denies an empty one', async () => {
    const ok = await sourceAdmissible(
      await tempFile('brief.md', '# Do the thing\nbecause'),
      'brief',
    );
    expect(ok.proceed).toBe(true);
    const empty = await sourceAdmissible(await tempFile('empty.md', '   \n'), 'brief');
    expect(empty.proceed).toBe(false);
  });

  it('parses frontmatter status and sections', () => {
    const p = parseSource(['---', 'status: proposed', '---', '## Context', 'a'].join('\n'), 'adr');
    expect(p.status).toBe('proposed');
    expect(p.hasContext).toBe(true);
    expect(p.hasDecision).toBe(false);
  });

  it('a section body starting with a blank line is still non-empty', () => {
    const doc = ['## Context', '', 'real content', '', '## Decision', 'yes'].join('\n');
    const p = parseSource(doc, 'adr');
    expect(p.hasContext).toBe(true);
    expect(p.hasDecision).toBe(true);
    // a genuinely empty section stays empty
    const empty = ['## Context', '', '## Decision', 'yes'].join('\n');
    expect(parseSource(empty, 'adr').hasContext).toBe(false);
  });
});

describe('allReviewsApproved', () => {
  const specialties = ['correctness', 'compatibility', 'security', 'performance', 'complexity'];

  it('denies when a specialty has no verdict (missing != passed)', () => {
    const g = allReviewsApproved(
      stateWith('design', 1, { correctness: approve, security: approve }),
      'design',
      1,
      specialties,
      DEFAULT_CONFIG,
    );
    expect(g.proceed).toBe(false);
    expect(g.reason).toContain(
      'reviews missing or invalid: compatibility, performance, complexity',
    );
  });

  it("a retry's verdict supersedes the failed attempt it retried", () => {
    const s = stateWith('design', 1, {
      security: approve,
      compatibility: approve,
      performance: approve,
      complexity: approve,
    });
    s.results.push({
      stage: 'design-review:correctness',
      round: 1,
      sessionID: 'ses-retry',
      agent: 'a',
      at: 1,
      reply: approve,
      transcript: 't',
    });
    s.results.unshift({
      stage: 'design-review:correctness',
      round: 1,
      sessionID: 'ses-failed',
      agent: 'a',
      at: 0,
      transcript: 't',
    });
    const g = allReviewsApproved(s, 'design', 1, specialties, DEFAULT_CONFIG);
    expect(g.proceed).toBe(true);
  });

  it('allows when all approve with no blocking findings', () => {
    const all = Object.fromEntries(specialties.map((s) => [s, approve]));
    const g = allReviewsApproved(
      stateWith('design', 1, all),
      'design',
      1,
      specialties,
      DEFAULT_CONFIG,
    );
    expect(g.proceed).toBe(true);
  });

  it('denies on any request-changes verdict', () => {
    const all = Object.fromEntries(specialties.map((s) => [s, approve]));
    all['security'] = rejectBlocker;
    const g = allReviewsApproved(
      stateWith('design', 1, all),
      'design',
      1,
      specialties,
      DEFAULT_CONFIG,
    );
    expect(g.proceed).toBe(false);
  });

  it('allows approve + nit-only findings (nits do not block)', () => {
    const all = Object.fromEntries(specialties.map((s) => [s, approveWithNit]));
    const g = allReviewsApproved(
      stateWith('design', 1, all),
      'design',
      1,
      specialties,
      DEFAULT_CONFIG,
    );
    expect(g.proceed).toBe(true);
  });
});

describe('isSemanticCommit', () => {
  it('accepts Conventional Commits forms', () => {
    expect(isSemanticCommit('feat: add timeout flag')).toBe(true);
    expect(isSemanticCommit('feat(orchestration): add timeout flag')).toBe(true);
  });

  it('rejects non-conventional messages', () => {
    expect(isSemanticCommit('added a thing')).toBe(false);
    expect(isSemanticCommit('Feat: capital type')).toBe(false);
    expect(isSemanticCommit('feat:nospace')).toBe(false);
  });
});

describe('flipAdrStatus', () => {
  it('flips proposed to accepted', () => {
    const out = flipAdrStatus(['---', 'status: proposed', '---', '# x'].join('\n'));
    expect(out).not.toBeNull();
    expect(out).toContain('status: accepted');
  });

  it('refuses when the input was not proposed', () => {
    expect(flipAdrStatus(['---', 'status: accepted', '---', '# x'].join('\n'))).toBeNull();
  });

  it('refuses without frontmatter', () => {
    expect(flipAdrStatus('# x\nstatus: proposed')).toBeNull();
  });
});

describe('nextNnn', () => {
  it('starts at 001', () => {
    expect(nextNnn([])).toBe('001');
  });

  it('takes the next number after the maximum', () => {
    expect(nextNnn(['007-a.md', '003-b', 'README.md'])).toBe('008');
  });
});

describe('packageDirs', () => {
  it('finds nested tool packages and skips node_modules', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-pkgs-'));
    await writeFile(join(dir, 'package.json'), '{}', 'utf8');
    const { mkdir } = await import('node:fs/promises');
    await mkdir(join(dir, 'tools', 'x'), { recursive: true });
    await mkdir(join(dir, 'node_modules', 'y'), { recursive: true });
    await writeFile(join(dir, 'tools', 'x', 'package.json'), '{}', 'utf8');
    await writeFile(join(dir, 'node_modules', 'y', 'package.json'), '{}', 'utf8');
    const found = packageDirs(dir).map((d) => d.slice(dir.length + 1) || '.');
    expect(found.sort()).toEqual(['.', 'tools/x']);
  });
});

describe('stagePathAllowed', () => {
  // The real post-move path of this file's own successor under ADR-004. #4:
  // nested-path classification is asserted on a path that will actually exist
  // after the move, not inferred from the inert fixture strings below (A19).
  const nestedTest = 'tools/orchestration/tests/pipeline/gates.test.ts';

  it('coder stages may not touch tests but may touch source and docs', () => {
    // proves: Test strategy #4 — nested tests/pipeline/ path denied to coder stages
    expect(stagePathAllowed('implement', nestedTest)).toBe(false);
    expect(stagePathAllowed('fix', nestedTest)).toBe(false);
    expect(stagePathAllowed('implement', 'tools/orchestration/src/cli.ts')).toBe(true);
    expect(stagePathAllowed('implement', 'docs/designs/002-x/03-build-notes.md')).toBe(true);
    expect(stagePathAllowed('fix', 'tools/orchestration/tests/cli.test.ts')).toBe(false);
    expect(stagePathAllowed('fix', 'packages/x/tests/a.ts')).toBe(false);
  });

  it('test engineer owns tests and docs but never source', () => {
    // proves: Test strategy #4 — nested tests/pipeline/ path allowed to test stages
    expect(stagePathAllowed('test-authoring', nestedTest)).toBe(true);
    expect(stagePathAllowed('test-update', nestedTest)).toBe(true);
    expect(stagePathAllowed('test-authoring', 'tools/orchestration/tests/cli.test.ts')).toBe(true);
    expect(stagePathAllowed('test-update', 'docs/designs/002-x/transcripts/x.md')).toBe(true);
    expect(stagePathAllowed('test-authoring', 'tools/orchestration/src/cli.ts')).toBe(false);
  });

  it('document and review stages are confined to docs/', () => {
    expect(stagePathAllowed('research', 'docs/designs/002-x/01-research.md')).toBe(true);
    expect(stagePathAllowed('design-review', 'docs/designs/002-x/reviews/design/x.md')).toBe(true);
    expect(stagePathAllowed('doc-sync', 'docs/designs/002-x/02-proposal.md')).toBe(true);
    expect(stagePathAllowed('proposal', 'tools/orchestration/src/cli.ts')).toBe(false);
    expect(stagePathAllowed('code-review', 'README.md')).toBe(false);
  });
});

describe('stagePathAllowed: runner-reserved checkpoint paths', () => {
  // proves: Test strategy #1 — the reserved-name rule precedes the per-stage
  // rules: the checkpoint, its saveState tmp shape, ESCALATION.md, and the
  // run lock are denied to every stage class at any depth, and case-
  // insensitively the way the moves validator already compares these names
  // (ADR-006 §1). The lock joins the reserved set per security r1 #4: §1's
  // trust posture leans on §4's single-flight premise, so a coder stage must
  // not be able to delete or rewrite `.pipeline.lock`.
  type StageArg = Parameters<typeof stagePathAllowed>[0];

  const STAGE_CLASSES: StageArg[] = [
    'research',
    'proposal',
    'design-review',
    'doc-sync',
    'editorial-review',
    'test-authoring',
    'test-update',
    'implement',
    'fix',
    'code-review',
  ];

  const RESERVED_PATHS = [
    '.pipeline.json',
    'docs/designs/002-x/.pipeline.json',
    'docs/designs/002-x/round/reviews/.pipeline.json',
    '.pipeline.json.a1b2c3d4.tmp',
    'docs/designs/002-x/.pipeline.json.a1b2c3d4.tmp',
    'ESCALATION.md',
    'docs/designs/002-x/ESCALATION.md',
    '.PIPELINE.JSON',
    'docs/designs/002-x/.Pipeline.Json.A1B2C3D4.TMP',
    'docs/designs/002-x/Escalation.md',
    '.pipeline.lock',
    'docs/designs/002-x/.pipeline.lock',
    'docs/designs/002-x/.PIPELINE.LOCK',
  ];

  for (const stage of STAGE_CLASSES) {
    it(`denies every reserved checkpoint name to the ${stage} stage`, () => {
      for (const path of RESERVED_PATHS) {
        expect(stagePathAllowed(stage, path)).toBe(false);
      }
    });
  }

  // proves: Test strategy #1 — the fence matches whole `/`-separated
  // segments: a neighboring docs path that merely carries a reserved name
  // stays inside the stage's write set.
  it('leaves neighboring docs paths allowed', () => {
    expect(stagePathAllowed('research', 'docs/designs/002-x/.pipeline.json-notes.md')).toBe(true);
    expect(stagePathAllowed('doc-sync', 'docs/designs/002-x/pipeline.json')).toBe(true);
    expect(stagePathAllowed('research', 'docs/designs/002-x/ESCALATION-notes.md')).toBe(true);
    expect(stagePathAllowed('implement', 'docs/designs/002-x/escalation-summary.md')).toBe(true);
  });
});

describe('test tree mirrors src/ hierarchy', () => {
  // The baseline, named once as the amended Rollout 1 words it: the pre-move
  // test-only commit keeps the 26-file count and adds cases, so the
  // 26-file/228-test pair at the move commit is that run's acceptance
  // measurement (Rollout 3's reported evidence), not a lasting suite
  // invariant — new mirrored test files are expected from the merge on. What
  // below stays permanent: the zero-flat-name listing check plus the subset
  // assertions. Test strategy #1's "still exists and passes unmodified
  // against that baseline" is what the other 25 suites prove across the move.
  // The 23 files ADR-004 Decision ¶1 moves, minus the pipeline- prefix.
  const MOVED_NAMES = [
    'budget',
    'checkpoint-writer',
    'cli-max-parallel',
    'concurrency',
    'escalation',
    'fix-trigger',
    'funnel',
    'gates',
    'identity',
    'independence',
    'ownership',
    'panel',
    'parallel-escalation',
    'parallel-fanout',
    'parallel-integration',
    'pass-finalize',
    'prompts',
    'proposal-size',
    'reply-contract',
    'runner',
    'stage-lifecycle',
    'state-store',
    'states',
  ];
  const ROOT_TESTS = ['cli.test.ts', 'opencode-runner.test.ts', 'opencode-runner-timeout.test.ts'];
  const nested = MOVED_NAMES.map((n) => `tools/orchestration/tests/pipeline/${n}.test.ts`);
  const rootTier = ROOT_TESTS.map((n) => `tools/orchestration/tests/${n}`);

  // #2's prescribed fail-closed check: the tracked file listing, not a content
  // grep, so in-file fixture strings naming the old flat form are not files.
  function trackedTestFiles(): string[] {
    const root = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
    const listing = execFileSync('git', ['ls-files', '--full-name', 'tools/orchestration/tests/'], {
      cwd: root,
      encoding: 'utf8',
    });
    return listing.split('\n').filter((f) => f.endsWith('.test.ts'));
  }

  // proves: Test strategy #1 (every previously existing test file is still
  // tracked) and #2's lasting assertions: zero leftover flat
  // pipeline-*.test.ts names, so a half-applied move fails closed, and the
  // 23 moved and 3 root-tier paths as a subset of the listing. The exact
  // 26-file/228-test figures are the move commit's acceptance measurement,
  // deliberately not pinned here (code-review r1, testing, major).
  it('tracks every moved and root-tier test file, and fails closed on leftover flat names', () => {
    const files = trackedTestFiles();
    expect(files.filter((f) => /(^|\/)pipeline-[^/]*\.test\.ts$/.test(f))).toEqual([]);
    for (const expected of [...nested, ...rootTier]) expect(files).toContain(expected);
  });

  // proves: Test strategy #3 — the suite's own assertion simulates vitest's
  // path-substring selection filter (A6) over the tracked listing rather
  // than spawning a nested vitest run (code-review r1, testing, minor): it
  // matches every moved file and none of the three root-tier tests. The
  // real `vitest run tests/pipeline` selection is reported acceptance
  // evidence.
  it('simulated tests/pipeline selection matches every moved file and no root-tier test', () => {
    const selected = trackedTestFiles().filter((f) => f.includes('tests/pipeline'));
    for (const moved of nested) expect(selected).toContain(moved);
    for (const rootFile of rootTier) expect(selected).not.toContain(rootFile);
  });
});
