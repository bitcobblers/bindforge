import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { describe, expect, it, vi } from 'vitest';

import { escalationPhase, hasPlanStop, reportFindings } from '../../src/pipeline/report.js';
import { finalizeEscalate } from '../../src/pipeline/runner.js';
import { DEFAULT_CONFIG } from '../../src/pipeline/stages.js';
import type { PipelineState, StageResult } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

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

  // proves: Test strategy #10 — a light run stopped at `plan` reports the code
  // phase: a light run has no design phase, so "the design phase" in
  // ESCALATION.md would name a phase that cannot have failed. `plan` joins the
  // reporting set for both flows, and the gate — not this function — decides
  // which paths the stage owns.
  it('reports a plan-stage stop as the code phase', () => {
    expect(escalationPhase({ ...stateWith(), lastStage: 'plan', flow: 'light' })).toBe('code');
    expect(escalationPhase({ ...stateWith(), lastStage: 'plan' })).toBe('code');
    // The design-phase attribution the light flow must not lose: an entry-stage
    // stop in the full flow is still design.
    expect(escalationPhase({ ...stateWith(), lastStage: 'proposal' })).toBe('design');
  });
});

// ---------------------------------------------------------------------------
// Run 010, Test strategy #14 (and #8's other half): the light promotion
// suffix is appended at exactly one site — this function — gated on the flow
// and on the escalating **stage**, never on `escalationPhase` (which reports a
// light `plan` stop as `code`). One append covers the pass-backstop, fix-round
// and session-budget records; the plan-size stop's own string already names
// the promotion, so it must not gain the suffix a second time.
// ---------------------------------------------------------------------------

describe('finalizeEscalate: the light promotion suffix', () => {
  function lightRepo(): string {
    const dir = mkTmpDir('bindforge-escalate-light-');
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

  const SUFFIX = 'the light flow (flow: light); promote with flow: full and re-run --force';
  const designDirOf = (dir: string) => join(dir, 'docs', 'designs', '002-x');

  async function recordFor(partial: Partial<PipelineState>, reason: string): Promise<string> {
    const dir = lightRepo();
    const state = { ...stateWith(), ...partial };
    await finalizeEscalate(dir, state, reason, DEFAULT_CONFIG);
    return readFileSync(join(designDirOf(dir), 'ESCALATION.md'), 'utf8');
  }

  // proves: Test strategy #14 — every code-phase escalation carries the
  // suffix, appended once.
  it('appends the suffix once to each code-phase record', async () => {
    for (const lastStage of ['implement', 'fix', 'code-review']) {
      const report = await recordFor(
        { lastStage, flow: 'light' },
        'code review: the pass backstop (3/3 passes)',
      );
      expect(report, `${lastStage} escalates with the promotion recipe`).toContain(SUFFIX);
      expect(report.match(/promote with flow: full/g)).toHaveLength(1);
    }
  });

  // proves: Test strategy #14 — the plan-size stop is the design's one plan
  // stop, and its string already carries the recipe: the suffix is not added.
  it('keeps the plan-size stop recipe singular', async () => {
    const planReason =
      'plan outgrew the light flow (81 lines, cap 80): set flow: full and re-run with --force';
    const report = await recordFor({ lastStage: 'plan', flow: 'light' }, planReason);
    expect(report).toContain(planReason);
    expect(report).not.toContain(SUFFIX);
  });

  // proves: Test strategy #14/#19 (correctness r1c2 finding A) — the plan stop's
  // record is single-voiced: its repair line is the promotion recipe, not the
  // generic `pipeline resume`, because the resume that footer prints lands on
  // `case 'plan'` → `implement` and builds the plan the flow refused. Every
  // other light stop is repaired by a resume, so its footer keeps naming one.
  // The state carries the stop the gate records (`planStop`), which is what the
  // repair selects on: the stage is not the stop (correctness r1c3 finding 1).
  it('repairs a plan stop with promotion, not with a plain resume', async () => {
    const planReason =
      'plan outgrew the light flow (120 lines, cap 80): set flow: full and re-run with --force';
    const report = await recordFor(
      { lastStage: 'plan', flow: 'light', planStop: planReason },
      planReason,
    );
    expect(report).toContain('set flow: full and re-run with --force');
    expect(report).toContain('resume --stage plan');
    expect(report).not.toContain('pipeline resume 002-x');

    const budget = await recordFor(
      { lastStage: 'code-review', flow: 'light' },
      'code review: the pass backstop (3/3 passes)',
    );
    expect(budget).toContain('pipeline resume 002-x');
  });

  // proves: Test strategy #14/#19 (correctness r1c3 finding 1) — the light run
  // with no stop recorded gets the repair the state machine honors. This is the
  // shape a failing `implement` leaves: the plan that cleared the cap persisted
  // `lastStage: 'plan'`, the gate cleared the stop, and the coder's own failure
  // escalated with `lastStage` never advanced. Nothing was refused, so
  // `case 'plan'` falls to `implement` and a plain resume retries the coder; a
  // record that claims a plan stop sends the operator to a promotion and a
  // wasted re-plan for a run that never reached the gate.
  it('repairs a light stop with no plan stop recorded by a plain resume', async () => {
    const report = await recordFor(
      { lastStage: 'plan', flow: 'light' },
      'implement: stage failed (2 attempts)',
    );
    expect(report).toContain('pipeline resume 002-x');
    expect(report).not.toMatch(/plan stop/i);
    expect(report).not.toMatch(/re-escalat/i);
    expect(report).not.toContain('resume --stage plan');
    expect(report).not.toContain('flow: full and re-run with --force');
  });

  // proves: Test strategy #14 — a full run's record gains nothing: the append
  // is gated on the flow, and an absent flow key is full.
  it('adds nothing to a full run', async () => {
    const report = await recordFor(
      { lastStage: 'code-review' },
      'code review: the pass backstop (3/3 passes)',
    );
    expect(report).not.toContain('light flow');
    expect(report).not.toContain('--force');
  });
});

// ---------------------------------------------------------------------------
// Run 010, correctness r1c3 finding 1: the plan-stop record is selected by the
// stop the gate recorded, not by the stage a plan stop happens to have left.
// The stage is not the stop — a light run that clears the cap and a light
// `implement` that fails both stand at `lastStage: 'plan'` with nothing
// refused, and the state's documented reading of a missing stop is "no stop
// standing". The predicate is the one selector for the ESCALATION.md repair
// lines and the `pipeline status` plan-stop line, so it is pinned here.
// ---------------------------------------------------------------------------

describe('hasPlanStop', () => {
  const PLAN_STOP =
    'plan outgrew the light flow (120 lines, cap 80): set flow: full and re-run with --force';

  // proves: Test strategy #19 (correctness r1c3 finding 1) — a recorded stop
  // reads as a stop.
  it('reads the stop the gate recorded', () => {
    expect(
      hasPlanStop({ ...stateWith(), flow: 'light', lastStage: 'plan', planStop: PLAN_STOP }),
    ).toBe(true);
  });

  // proves: Test strategy #19 (correctness r1c3 finding 1) — the stage a plan
  // stop leaves is not the stop: a running light run past its plan and a light
  // `implement` that failed both sit at `plan` with no stop on the state, and
  // neither is stopped.
  it('reads the stage alone as no stop', () => {
    expect(hasPlanStop({ ...stateWith(), flow: 'light', lastStage: 'plan' })).toBe(false);
    expect(
      hasPlanStop({ ...stateWith(), flow: 'light', lastStage: 'plan', status: 'escalated' }),
    ).toBe(false);
  });

  // proves: Test strategy #19/#5 (the plan gate is light-only: `planMaxLines`
  // is absent in the full flow's config, so no full run has a plan stop to
  // print) — a full run never claims the light flow's plan stop.
  it('reads no plan stop for a full run', () => {
    expect(hasPlanStop({ ...stateWith(), lastStage: 'plan', planStop: PLAN_STOP })).toBe(false);
    expect(
      hasPlanStop({ ...stateWith(), flow: 'full', lastStage: 'plan', planStop: PLAN_STOP }),
    ).toBe(false);
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
    const dir = mkTmpDir('bindforge-escalate-');
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
