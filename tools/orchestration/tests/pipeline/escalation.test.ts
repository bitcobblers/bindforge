import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it, vi } from 'vitest';

import { escalationPhase, reportFindings } from '../../src/pipeline/report.js';
import { finalizeEscalate } from '../../src/pipeline/runner.js';
import { DEFAULT_CONFIG } from '../../src/pipeline/stages.js';
import type { PipelineState, StageResult } from '../../src/pipeline/types.js';

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

  // proves: Test strategy #7 — finalizeEscalate with an unwritable checkpoint
  // (a directory occupies the checkpoint path): no error escapes, ESCALATION.md
  // still records the gate reason plus the `checkpoint save failed` line, the
  // commit is still made, and the repair printed names the rm -r posture.
  it('escalates with a full record when its own checkpoint save fails', async () => {
    const dir = tmpRepo();
    const designDir = join(dir, 'docs', 'designs', '002-x');
    mkdirSync(designDir, { recursive: true });
    mkdirSync(join(designDir, '.pipeline.json')); // directory replaces the file path

    const state = stateWith(
      review('performance', 3, [
        { severity: 'blocker', location: 'Test strategy 13', problem: 'bar unsatisfiable' },
      ]),
    );
    const errors: string[] = [];
    const spy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
      errors.push(args.join(' '));
      return undefined;
    });

    let escaped: unknown;
    try {
      await finalizeEscalate(
        dir,
        state,
        'checkpoint save failed: EISDIR: illegal operation on a directory, rename',
        DEFAULT_CONFIG,
      );
    } catch (err) {
      escaped = err;
    } finally {
      spy.mockRestore();
    }

    // A failing save turns the escalation into a crash no more.
    expect(escaped).toBeUndefined();

    // The terminal record exists next to the broken path and names both the
    // findings that opened the escalation and the save that failed during it.
    const report = readFileSync(join(designDir, 'ESCALATION.md'), 'utf8');
    expect(report).toContain('2 blocking findings open');
    expect(report).toContain('checkpoint save failed');
    // testing r1 #6 / correctness r1 #5: the appended line is the failure
    // message alone — its stem already begins `checkpoint save failed:`, so
    // the stem never appears doubled on one line (deviation 13).
    expect(report).toMatch(/^checkpoint save failed: (?!checkpoint save failed)/m);
    expect(report).not.toMatch(/checkpoint save failed: checkpoint save failed/);

    // The escalation commit is still recorded — the run is not left without a
    // terminal commit just because the checkpoint save failed.
    expect(state.stageCommits).toHaveLength(1);
    const subject = execFileSync('git', ['log', '--format=%s', '-1'], {
      cwd: dir,
      encoding: 'utf8',
    }).trim();
    expect(subject).toBe('docs: 002-x pipeline escalated');

    // The operator is printed the repair, rm -r variant included, and the
    // planted directory is left exactly where it is for them to act on.
    const printed = errors.join('\n');
    expect(printed).toContain('rm -r');
    expect(printed).toContain('git checkout HEAD --');
    expect(statSync(join(designDir, '.pipeline.json')).isDirectory()).toBe(true);
  });

  // proves: ADR-006 §3 (security r1 #1/#2, deviation 10) — the marker defends
  // its own path. A stage owning docs/** plants `ESCALATION.md/probe`: the
  // sweep reverts the file but never lists the (then-empty, untracked)
  // directory residue, so escalation must clear the directory, write the
  // marker atomically through a tmp+rename pair, and land a full record —
  // not crash with no terminal record at all.
  it('escalates with a full record when a directory occupies the marker path', async () => {
    const dir = tmpRepo();
    const designDir = join(dir, 'docs', 'designs', '002-x');
    mkdirSync(join(designDir, 'ESCALATION.md'), { recursive: true });
    writeFileSync(join(designDir, 'ESCALATION.md', 'probe'), 'planted\n', 'utf8');

    const state = stateWith(
      review('performance', 3, [
        { severity: 'blocker', location: 'Test strategy 13', problem: 'bar unsatisfiable' },
      ]),
    );
    const errors: string[] = [];
    const spy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
      errors.push(args.join(' '));
      return undefined;
    });

    let escaped: unknown;
    try {
      await finalizeEscalate(dir, state, 'design review: 1 blocking findings open', DEFAULT_CONFIG);
    } catch (err) {
      escaped = err;
    } finally {
      spy.mockRestore();
    }

    // An EISDIR at the marker path escapes to the drive loop no more: the
    // run keeps its terminal record either way.
    expect(escaped).toBeUndefined();

    const marker = join(designDir, 'ESCALATION.md');
    expect(statSync(marker).isFile()).toBe(true); // the planted directory is cleared
    const report = readFileSync(marker, 'utf8');
    expect(report).toContain('**design** phase');
    expect(report).toContain('2 blocking findings open');

    // The marker write is atomic (tmp+rename like saveState): no tmp residue
    // and no planted residue survive beside it.
    expect(
      readdirSync(designDir).filter((f) => f.startsWith('ESCALATION') && f !== 'ESCALATION.md'),
    ).toEqual([]);

    // The escalation commit still lands, as for every other escalation.
    expect(state.stageCommits).toHaveLength(1);
  });
});
