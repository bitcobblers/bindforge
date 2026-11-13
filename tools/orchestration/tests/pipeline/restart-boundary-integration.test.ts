import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { exitCodeFor } from '../../src/pipeline/cli.js';
import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-020 restart boundary, integration (design 020 Test strategy #9/#10/#11):
// run 016's scenario end to end against a mock client in a temp git repo. A
// light run whose `implement` window changes a stage template and the renderer
// in one commit must pause clean — checkpoint record, `restart-boundary` gate
// entry, boundary commit, no ESCALATION.md — and a fresh `resumePipeline`
// process, having read the committed tree, crosses the boundary, clears the
// record and renders the changed template with the current renderer instead of
// escalating on `{{gateEvidence}}`. The driver posture (`run; while exit 3:
// resume`) and the uncapped two-boundary run ride the same fixture.
//
// The run's promptsDir is inside the repo (`tools/orchestration/prompts/`),
// as in a real self-change run, so the templates the window edits are the
// templates the next process renders. `{{gateEvidence}}` is the run-016 class
// placeholder: the merged renderer supplies it, the stale one would not.
// ---------------------------------------------------------------------------

const BRIEF = 'docs/prompts/002-x.md';
const DESIGN_DIR = 'docs/designs/002-x';
const PROMPTS_REL = join('tools', 'orchestration', 'prompts');
const GATE_EVIDENCE_LINE = 'Evidence: {{gateEvidence}}';

interface RestartRecord {
  stage: string;
  paths: string[];
  at: number;
}
type StateWithBoundary = PipelineState & { restartRequired?: RestartRecord };
/** The widened run/resume result (§3); the CLI maps 3 from the standing record. */
interface DriveResult {
  status: 'running' | 'done' | 'escalated';
  restartRequired?: RestartRecord;
}

// The run's effective flow numbers are the registry's for a light run, not this
// object's — `light-budget.test.ts` pins that resolution — so the light run here
// drives the 2-seat code panel with 3 passes per round. This object supplies the
// timeouts and pacing the fixture relies on.
const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

function planDoc(): string {
  return [
    '# Plan: 002-x',
    '',
    '## Problem',
    'the thing is missing',
    '',
    '## Chosen approach',
    'one module, one test file',
    '',
    '## Files to touch',
    '- src/thing.ts',
    '- tests/thing.test.ts',
    '',
    '## Test plan',
    '- the check gate stays green',
    '',
    '## Done criteria',
    '- the module exists and the gate passes',
    '',
  ].join('\n');
}

function lightState(): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'brief', path: BRIEF },
    status: 'escalated',
    lastStage: '',
    flow: 'light',
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
  };
}

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

/** Light run whose templates live under `tools/orchestration/prompts/` — the
 * self-change run's tree. `checkExitCode` drives the red-gate sibling. */
function tmpRepo(checkExitCode: 0 | 1 = 0): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-restart-e2e-'));
  mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
  writeFileSync(join(dir, BRIEF), '---\nflow: light\n---\n\nBuild the thing.\n', 'utf8');
  mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
  writeFileSync(
    join(dir, DESIGN_DIR, '.pipeline.json'),
    `${JSON.stringify(lightState(), null, 2)}\n`,
  );
  writeFileSync(
    join(dir, 'package.json'),
    `{"name":"t","scripts":{"check":"node -e \\"process.exit(${String(checkExitCode)})\\""}}\n`,
    'utf8',
  );
  mkdirSync(join(dir, PROMPTS_REL), { recursive: true });
  // Every stage template the light route can reach, rendered from inside the
  // repo — a self-change run's tree. Only the ones the route renders need to
  // carry anything; the rest are placeholders so no stage ENOENTs.
  writeFileSync(join(dir, PROMPTS_REL, 'plan.md'), 'Write the plan.\n');
  writeFileSync(join(dir, PROMPTS_REL, 'implement.md'), 'Implement the plan.\n');
  writeFileSync(join(dir, PROMPTS_REL, 'fix.md'), 'Fix what the review found.\n');
  writeFileSync(join(dir, PROMPTS_REL, 'fix-tests.md'), 'Fix the tests.\n');
  writeFileSync(
    join(dir, PROMPTS_REL, 'code-review.md'),
    'Review the code. Write your report to {{reportPath}}\n',
  );
  for (const other of [
    'design-review.md',
    'doc-sync.md',
    'proposal-final.md',
    'proposal-revise.md',
    'proposal.md',
    'research.md',
    'test-authoring.md',
    'test-update.md',
  ]) {
    writeFileSync(join(dir, PROMPTS_REL, other), 'Placeholder stage prompt.\n');
  }
  gitAt(dir, ['init', '-q', '-b', 'main']);
  gitAt(dir, ['add', '-A']);
  gitAt(dir, ['commit', '-q', '-m', 'docs: base']);
  return dir;
}

interface StubSpec {
  /** seat stage key → session ordinals that answer request-changes. */
  rejects: Record<string, number[]>;
  /** the implement window adds a prompt variable to the next stage's template
   * AND a renderer line in the same window — run 016's exact shape. */
  implementChangesSurface: boolean;
  /** the fix window also changes a surface path (two-boundary runs). */
  fixChangesSurface?: boolean;
}

/** Seat key → sessions already created for that seat in the whole run. Every
 * resume builds a fresh stub, so a per-stub ordinal re-dissents in every
 * process; `rejects` is meant to name a seat's attempts in the run, so the
 * ordinal stream is shared across the run's stubs. */
type RunOrdinals = Record<string, number>;

function boundaryStub(dir: string, spec: StubSpec, ordinals: RunOrdinals = {}) {
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  const created: Record<string, number> = {};
  const sessionsCreated: string[] = [];
  const prompts: Array<{ key: string; text: string }> = [];
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const reject = JSON.stringify({
    verdict: 'request-changes',
    findings: [{ severity: 'major', location: 'Test plan', problem: 'nothing covers it' }],
  });
  let clock = 0;

  const replyFor = (key: string, ordinal: number): string => {
    if (key.includes('-review:')) {
      return (spec.rejects[key] ?? []).includes(ordinal) ? reject : approve;
    }
    if (key === 'plan') return JSON.stringify({ summary: 'planned', commitMessage: 'docs: plan' });
    if (key === 'implement' || key === 'fix')
      return JSON.stringify({
        testsNeedUpdate: false,
        commitMessage: key === 'fix' ? 'fix: address the findings' : 'feat: implement it',
      });
    return JSON.stringify({ summary: 'ok' });
  };

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        const ordinal = ordinals[key] ?? 0;
        ordinals[key] = ordinal + 1;
        created[key] = (created[key] ?? 0) + 1;
        sessionsCreated.push(key);
        const id = `ses_${String(Object.values(created).reduce((a, b) => a + b, 0))}`;
        sessions.set(id, { key, ordinal, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string; text?: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        prompts.push({ key: s.key, text: input.text ?? '' });

        if (s.key === 'plan') {
          writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), `${planDoc()}\n`, 'utf8');
        }
        if (s.key === 'implement') {
          mkdirSync(join(dir, 'src'), { recursive: true });
          writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n', 'utf8');
          writeFileSync(join(dir, DESIGN_DIR, '03-build-notes.md'), '# Build notes\n\nshipped\n');
          if (spec.implementChangesSurface) {
            // Template variable in the NEXT stage's template...
            writeFileSync(
              join(dir, PROMPTS_REL, 'code-review.md'),
              `Review the code. Write your report to {{reportPath}}\n${GATE_EVIDENCE_LINE}\n`,
            );
            // ...and the renderer that supplies it, in the same window.
            mkdirSync(join(dir, 'tools', 'orchestration', 'src', 'pipeline'), { recursive: true });
            writeFileSync(
              join(dir, 'tools', 'orchestration', 'src', 'pipeline', 'render.ts'),
              'export const TEMPLATE_DEFAULTS = { gateEvidence: "no gate evidence" };\n',
            );
          }
        }
        if (s.key === 'fix') {
          // A fix window that changes nothing has no commit to finalize, so the
          // round's commit gets a real (non-surface) change — the posture
          // `light-budget.test.ts`'s stub keeps.
          writeFileSync(
            join(dir, DESIGN_DIR, '02-proposal.md'),
            `${planDoc()}\nfixed in window ${String(sessionsCreated.length)}\n`,
            'utf8',
          );
          if (spec.fixChangesSurface === true) {
            writeFileSync(
              join(dir, PROMPTS_REL, 'implement.md'),
              'Implement the plan.\nA second window, another surface line.\n',
            );
            writeFileSync(
              join(dir, 'tools', 'orchestration', 'src', 'pipeline', 'render.ts'),
              'export const TEMPLATE_DEFAULTS = { gateEvidence: "boundary evidence" };\n',
            );
          }
        }
        if (s.key.includes('-review:')) {
          const found = /docs\/designs\/[^\s`]+\/reviews\/code\/[\w.-]+\.md/.exec(input.text ?? '');
          if (found !== null) {
            const path = join(dir, found[0]);
            mkdirSync(join(path, '..'), { recursive: true });
            writeFileSync(path, `report from ${s.key}\n`, 'utf8');
          }
        }
      },
      remove: async () => undefined,
      // The descendant walk's enumeration endpoint: no child sessions here.
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { key: '', ordinal: 0, prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          clock += 1;
          messages.push({
            id: `m_a${String(clock)}`,
            time: { created: clock },
            type: 'assistant',
            content: [{ type: 'text', text: replyFor(s.key, s.ordinal) }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(clock)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, sessionsCreated, prompts };
}

function optionsFor(dir: string, client: OpenCodeClient): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: join(dir, PROMPTS_REL),
    cfg: FAST,
    client,
  };
}

function savedState(dir: string): StateWithBoundary {
  return JSON.parse(
    readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8'),
  ) as StateWithBoundary;
}

const asResult = (raw: unknown): DriveResult => raw as DriveResult;

describe('run 016 end to end across a restart boundary', () => {
  // proves: Test strategy #9 (first half) — the first process stops clean: the
  // record names the stage and the template path, the `restart-boundary` gate
  // entry and the boundary commit are minted, the COMMITTED checkpoint carries
  // the record, the worktree is clean, and no escalation machinery runs: no
  // ESCALATION.md, and the next stage was never opened in this process.
  it('stops at the boundary when the implement window changes template and renderer', async () => {
    const dir = tmpRepo();
    const { client, sessionsCreated } = boundaryStub(dir, {
      rejects: {},
      implementChangesSurface: true,
    });

    const result = asResult(await resumePipeline(optionsFor(dir, client)));

    // The exit-3 posture, at the seam: status stays `running`, the record
    // stands, and §3's integer for that result is 3.
    expect(result.status).toBe('running');
    expect(result.restartRequired).toBeDefined();
    expect(exitCodeFor(result)).toBe(3);
    expect(result.restartRequired?.stage).toBe('implement');
    expect(result.restartRequired?.paths).toContain(
      join('tools', 'orchestration', 'prompts', 'code-review.md'),
    );

    const saved = savedState(dir);
    expect(saved.status).toBe('running');
    expect(saved.restartRequired?.stage).toBe('implement');

    const lastGate = saved.gateLog[saved.gateLog.length - 1];
    expect(lastGate?.gate).toBe('restart-boundary');
    expect(lastGate?.reason).toContain('implement');
    expect(lastGate?.reason).toContain(join('tools', 'orchestration', 'prompts', 'code-review.md'));

    expect(gitAt(dir, ['log', '--format=%s', '-1']).trim()).toBe('chore: 002-x restart boundary');
    expect(gitAt(dir, ['show', '-s', '--format=%B'])).toContain(
      join('tools', 'orchestration', 'prompts', 'code-review.md'),
    );

    // The record is in the COMMITTED checkpoint — the pause survives as run
    // history, the save-before-commit posture. Git paths are `/`-separated.
    const committed = JSON.parse(
      gitAt(dir, ['show', 'HEAD:docs/designs/002-x/.pipeline.json']),
    ) as StateWithBoundary;
    expect(committed.restartRequired?.stage).toBe('implement');

    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
    expect(gitAt(dir, ['status', '--porcelain']).trim()).toBe('');

    // No further stage window opened in the paused process.
    expect(sessionsCreated).toEqual(['plan', 'implement']);
  });

  // proves: Test strategy #9 (second half) — `resume` IS the restart: the fresh
  // process gate-logs the boundary it crosses, clears the record before
  // driving, and the next stage renders the committed template — the new
  // `{{gateEvidence}}` placeholder is substituted, not escalated on.
  it('crosses the boundary on resume and renders the changed template green', async () => {
    const dir = tmpRepo();
    const first = boundaryStub(dir, { rejects: {}, implementChangesSurface: true });
    const r1 = asResult(await resumePipeline(optionsFor(dir, first.client)));
    expect(r1.restartRequired).toBeDefined();

    const second = boundaryStub(dir, { rejects: {}, implementChangesSurface: true });
    const r2 = asResult(await resumePipeline(optionsFor(dir, second.client)));

    expect(r2.status).toBe('done');
    expect(r2.restartRequired).toBeUndefined();
    // The run goes green on the fresh process: exit 0, not 2 (compatibility
    // r1 #1 — the integers, not only the fields).
    expect(exitCodeFor(r2)).toBe(0);

    const saved = savedState(dir);
    expect(saved.restartRequired).toBeUndefined();
    const boundaryEntries = saved.gateLog.filter((g) => /restart/i.test(g.gate));
    // The pause entry and the crossing entry are both in the record.
    expect(boundaryEntries.length).toBeGreaterThanOrEqual(2);

    // The seat prompts came from the committed template, rendered: the
    // placeholder is gone and its line is there.
    const seatPrompts = second.prompts.filter((p) => p.key.startsWith('code-review:'));
    expect(seatPrompts.length).toBeGreaterThan(0);
    for (const p of seatPrompts) {
      expect(p.text).toContain('Evidence:');
      expect(p.text).not.toContain('{{gateEvidence}}');
    }
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
  });
});

describe('the driver loop', () => {
  // proves: Test strategy #10 — `run; while exit 3: resume` carries a
  // self-change run to the done (exit-0) posture: exactly two processes, and
  // the loop turns on §3's integer for the result, so the driver the design
  // promises is the one the suite pins (compatibility r1 #1, testing r1 #1).
  it('carries a self-change run to done across restarts', async () => {
    const dir = tmpRepo();
    const spec = { rejects: {}, implementChangesSurface: true };
    const drive = async (): Promise<DriveResult> =>
      asResult(await resumePipeline(optionsFor(dir, boundaryStub(dir, spec).client)));
    let result = await drive();
    let code = exitCodeFor(result);
    let processes = 1;
    while (code === 3 && processes < 5) {
      result = await drive();
      code = exitCodeFor(result);
      processes += 1;
    }
    expect(code).toBe(0);
    expect(result.status).toBe('done');
    expect(processes).toBe(2);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
  });

  // proves: Test strategy #10 — the two stop kinds stay distinguishable in the
  // other direction: a red check gate on a window that changed no harness path
  // is still the escalation posture — exit 2 keeps its exact meaning, "read
  // ESCALATION.md" — with no restart record a driver could misread. §1's rule is
  // why this fixture writes no surface path: an ordinary product run never opens
  // a boundary.
  it('still escalates a red check gate that changed no harness path', async () => {
    const dir = tmpRepo(1);
    const result = asResult(
      await resumePipeline(
        optionsFor(dir, boundaryStub(dir, { rejects: {}, implementChangesSurface: false }).client),
      ),
    );
    expect(result.status).toBe('escalated');
    expect(result.restartRequired).toBeUndefined();
    // Exit 2 with its exact meaning: a driver that resumes on 3 never picks up
    // an escalation (security r1 #1).
    expect(exitCodeFor(result)).toBe(2);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(true);
  });

  // proves: Test strategy #10 (§1's mandated check point) — when the red-gate
  // window also changed the surface, the pause wins the ordering: the record was
  // set inside the finalize window, and the loop-top check runs before
  // `decideNext`, so the fix routing never opens a window in that process. The
  // pause still reads as no failure: exit 3, the record standing, no marker.
  it('pauses at the boundary before the fix routing on a red gate', async () => {
    const dir = tmpRepo(1);
    const { client, sessionsCreated } = boundaryStub(dir, {
      rejects: {},
      implementChangesSurface: true,
    });

    const result = asResult(await resumePipeline(optionsFor(dir, client)));

    expect(result.status).toBe('running');
    expect(result.restartRequired?.stage).toBe('implement');
    // The pause reads as a pause on the integer too: 3, beside a red gate the
    // escalation routing never reached.
    expect(exitCodeFor(result)).toBe(3);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
    // The stop is the loop-top check refusing the next window — the fix round
    // the red gate routes to is never entered in this process.
    expect(sessionsCreated).toEqual(['plan', 'implement']);
  });
});

describe('consecutive boundaries', () => {
  // proves: Test strategy #11 — two self-change windows open two boundaries,
  // each gate-logged and cleared by its own resume, and the run completes on
  // the third process: v1 is uncapped, the count lives only in the gate log.
  it('opens one boundary per self-change window and finishes on the third process', async () => {
    const dir = tmpRepo();
    const spec = {
      rejects: { 'code-review:testing': [0] },
      implementChangesSurface: true,
      fixChangesSurface: true,
    };
    // Seat ordinals stream across the whole run, not per process: each resume
    // builds a fresh stub, and a per-process ordinal re-dissents in every
    // process. The pass cap that binds a light run is the flow registry's 3
    // passes per code round (`light-budget.test.ts` pins the resolution), so
    // three dissents spend the round's passes and the run escalates on the
    // backstop. One dissent in the round is what the light funnel converges on.
    const ordinals: RunOrdinals = {};

    const r1 = asResult(
      await resumePipeline(optionsFor(dir, boundaryStub(dir, spec, ordinals).client)),
    );
    expect(r1.restartRequired?.stage).toBe('implement');
    expect(exitCodeFor(r1)).toBe(3);

    const r2 = asResult(
      await resumePipeline(optionsFor(dir, boundaryStub(dir, spec, ordinals).client)),
    );
    expect(r2.restartRequired?.stage).toBe('fix');
    expect(exitCodeFor(r2)).toBe(3);
    const afterSecond = savedState(dir);
    expect(afterSecond.restartRequired?.stage).toBe('fix');
    expect(
      afterSecond.gateLog.filter((g) => /restart/i.test(g.gate)).length,
    ).toBeGreaterThanOrEqual(3);

    const r3 = asResult(
      await resumePipeline(optionsFor(dir, boundaryStub(dir, spec, ordinals).client)),
    );
    expect(r3.status).toBe('done');
    expect(r3.restartRequired).toBeUndefined();
    expect(exitCodeFor(r3)).toBe(0);

    const saved = savedState(dir);
    expect(saved.restartRequired).toBeUndefined();
    expect(
      (gitAt(dir, ['log', '--format=%s']).match(/chore: 002-x restart boundary/g) ?? []).length,
    ).toBe(2);
    expect(saved.gateLog.filter((g) => /restart/i.test(g.gate)).length).toBeGreaterThanOrEqual(4);
  });
});
