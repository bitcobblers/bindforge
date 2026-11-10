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
} from '../src/pipeline/gates.js';
import { DEFAULT_CONFIG } from '../src/pipeline/stages.js';
import type { PipelineState, VerdictReply } from '../src/pipeline/types.js';

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
  it('coder stages may not touch tests but may touch source and docs', () => {
    expect(stagePathAllowed('implement', 'tools/orchestration/src/cli.ts')).toBe(true);
    expect(stagePathAllowed('implement', 'docs/designs/002-x/03-build-notes.md')).toBe(true);
    expect(stagePathAllowed('fix', 'tools/orchestration/tests/cli.test.ts')).toBe(false);
    expect(stagePathAllowed('fix', 'packages/x/tests/a.ts')).toBe(false);
  });

  it('test engineer owns tests and docs but never source', () => {
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
