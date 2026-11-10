import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it, vi } from 'vitest';

import { escalationPhase, reportFindings } from '../src/pipeline/report.js';
import { finalizeEscalate } from '../src/pipeline/runner.js';
import { DEFAULT_CONFIG } from '../src/pipeline/stages.js';
import type { PipelineState, StageResult } from '../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// Run 1 escalated six times. Each ESCALATION.md listed every finding of the
// last round at every severity with no history, and nothing was printed to the
// terminal, so the operator could not tell which findings the last revise had
// failed to close from which the closure itself introduced.
// ---------------------------------------------------------------------------

function verdict(
  findings: Array<{ severity: string; location: string; problem: string }>,
  approve = false,
): object {
  return { verdict: approve ? 'approve' : 'request-changes', findings };
}

function review(
  specialty: string,
  round: number,
  findings: Array<{ severity: string; location: string; problem: string }>,
): StageResult {
  return {
    stage: `design-review:${specialty}`,
    round,
    sessionID: `ses_${specialty}_${String(round)}`,
    agent: `review/design-${specialty}`,
    at: round,
    reply: verdict(findings),
    transcript: `docs/designs/002-x/transcripts/r${String(round)}-${specialty}.md`,
  };
}

function stateWith(...results: StageResult[]): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'running',
    lastStage: 'design-review',
    designRound: 3,
    codeRound: 0,
    designBudgetUsed: 3,
    codeBudgetUsed: 0,
    editorialDone: false,
    testsGreen: false,
    testsNeedUpdate: false,
    stageFailures: {},
    stageCommits: [],
    forceNext: null,
    results,
    gateLog: [
      {
        gate: 'design-review',
        round: 3,
        proceed: false,
        reason: '2 blocking findings open',
        at: 0,
      },
    ],
    startedAt: 0,
    updatedAt: 0,
  };
}

describe('escalationPhase', () => {
  it('names the phase the run died in', () => {
    expect(escalationPhase(stateWith())).toBe('design');
    expect(escalationPhase({ ...stateWith(), lastStage: 'code-review' })).toBe('code');
    expect(escalationPhase({ ...stateWith(), lastStage: 'doc-sync' })).toBe('code');
    expect(escalationPhase({ ...stateWith(), lastStage: 'test-update' })).toBe('code');
    expect(escalationPhase({ ...stateWith(), lastStage: 'editorial-review' })).toBe('editorial');
  });
});

describe('reportFindings', () => {
  // r1 raises A and B; r2 carries A and raises C; r3 carries A and raises D,
  // while B — closed at r2 — comes back inside the text the r2 revise added.
  const state = stateWith(
    review('performance', 1, [
      { severity: 'major', location: 'Test strategy 13', problem: 'bar unsatisfiable' },
      { severity: 'major', location: 'Rollout', problem: 'no rollback path' },
    ]),
    review('performance', 2, [
      { severity: 'major', location: 'Test strategy 13:1180', problem: 'bar unsatisfiable' },
      { severity: 'major', location: 'Cost note', problem: 'cost model wrong' },
    ]),
    review('performance', 3, [
      {
        severity: 'blocker',
        location: 'Test strategy 13 (lines 1204-1220)',
        problem: 'still wrong',
      },
      { severity: 'major', location: 'Rollout', problem: 'still no rollback' },
      { severity: 'minor', location: 'Risk', problem: 'typo in the table' },
      { severity: 'nit', location: 'Context', problem: 'heading capitalisation' },
    ]),
  );

  it('splits a round into new, re-raised and previously-closed themes', () => {
    const found = reportFindings(state, 'design', 3, DEFAULT_CONFIG.blockingSeverities);
    const byLocation = Object.fromEntries(found.map((f) => [f.location, f]));

    // Same theme despite the shifted line anchor.
    expect(byLocation['Test strategy 13 (lines 1204-1220)'].status).toBe('re-raised');
    expect(byLocation['Test strategy 13 (lines 1204-1220)'].firstSeen).toBe(1);
    // Open at r1, gone at r2, back at r3: the closure introduced the defect.
    expect(byLocation.Rollout.status).toBe('previously-closed');
  });

  it('lists blocking severities only', () => {
    const severities = reportFindings(state, 'design', 3, DEFAULT_CONFIG.blockingSeverities).map(
      (f) => f.severity,
    );
    expect(severities.sort()).toEqual(['blocker', 'major']);
  });

  it('labels a first round as all-new', () => {
    const found = reportFindings(state, 'design', 1, DEFAULT_CONFIG.blockingSeverities);
    expect(found).toHaveLength(2);
    expect(new Set(found.map((f) => f.status))).toEqual(new Set(['new']));
  });
});

describe('finalizeEscalate', () => {
  function tmpRepo(): string {
    const dir = mkdtempSync(join(tmpdir(), 'bindforge-escalate-'));
    const git = (args: string[]) =>
      execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
        cwd: dir,
        encoding: 'utf8',
      });
    mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
    writeFileSync(join(dir, 'docs', 'adrs', '002-x.md'), '---\nstatus: proposed\n---\n');
    git(['init', '-q', '-b', 'main']);
    git(['add', '-A']);
    git(['commit', '-q', '-m', 'docs: base']);
    return dir;
  }

  it('writes an atomic checkpoint, a blocking-only report, and prints the cause', async () => {
    const dir = tmpRepo();
    const designDirRel = join('docs', 'designs', '002-x');
    const state = stateWith(
      review('performance', 3, [
        { severity: 'blocker', location: 'Test strategy 13', problem: 'bar unsatisfiable' },
        { severity: 'nit', location: 'Context', problem: 'heading capitalisation' },
      ]),
    );
    const errors: string[] = [];
    const spy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
      errors.push(args.join(' '));
      return undefined;
    });

    await finalizeEscalate(dir, state, 'design review: 1 blocking findings open', DEFAULT_CONFIG);
    spy.mockRestore();

    const designDir = join(dir, designDirRel);
    // No half-written checkpoint: tmp+rename, nothing left behind.
    expect(readdirSync(designDir)).not.toContain(expect.stringContaining('.tmp'));
    expect(readdirSync(designDir).some((f) => f.endsWith('.tmp'))).toBe(false);
    const saved = JSON.parse(
      readFileSync(join(designDir, '.pipeline.json'), 'utf8'),
    ) as PipelineState;
    expect(saved.status).toBe('escalated');

    const report = readFileSync(join(designDir, 'ESCALATION.md'), 'utf8');
    expect(report).toContain('**design** phase');
    expect(report).toContain('[blocker] (performance, first seen r3) Test strategy 13');
    expect(report).not.toContain('heading capitalisation');
    expect(report).toContain('1 minor/nit findings from this round live in the review reports');
    expect(report).toContain('design-review (r3): 2 blocking findings open');

    expect(errors.join('\n')).toContain('pipeline ESCALATED');
    expect(errors.join('\n')).toContain(join(designDirRel, 'ESCALATION.md'));
    expect(errors.join('\n')).toContain('pipeline resume 002-x');

    // Terminal record is committed, and the runner knows it authored that commit.
    expect(state.stageCommits).toHaveLength(1);
    expect(existsSync(join(dir, '.pipeline.lock'))).toBe(false);
    const subjects = execFileSync('git', ['log', '--format=%s', '-1'], {
      cwd: dir,
      encoding: 'utf8',
    }).trim();
    expect(subjects).toBe('docs: 002-x pipeline escalated');
  });
});
