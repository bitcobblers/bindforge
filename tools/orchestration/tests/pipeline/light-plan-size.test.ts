import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { configFor } from '../../src/pipeline/flows.js';
import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import { decideNext } from '../../src/pipeline/states.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// The one plan stop (A7 reverses the proposal-size lesson, deliberately): a
// plan over `planMaxLines` is a scope signal, so the run stops at the stage-ok
// point — before the implement session is spent — with a message that names the
// flow to move to. Over length is not a prose-police warning here: the plan is
// a fixed five-section document written once, before any code.
//
// The stop fails closed: a plan the meter cannot measure — the document the
// stage did not write — is a refusal, never a pass, since the run's one
// artifact is what the seat stamp and the reviewers' faithfulness check read.
//
// The stop is sticky (correctness r1c2 finding A): the record it leaves names
// `plan` as the stage that stopped, and the light graph's next transition from
// `plan` is `implement` — so the stop has to survive in the state, the record
// has to print one repair line (the promotion recipe, never the generic
// `pipeline resume`), and the refused plan must never become the document
// `implement` builds against. A `plan` pass that lands under the cap is what
// clears it.
// ---------------------------------------------------------------------------

const DESIGN_DIR = 'docs/designs/002-x';
const BRIEF = 'docs/prompts/002-x.md';
const STOP = 'plan outgrew the light flow (120 lines, cap 80)';
const PROMOTION = 'set flow: full and re-run with --force';

const BASE: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

/** A git command run in a temp repo, its output as text. */
function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

function tmpRepo(): string {
  const dir = mkTmpDir('bindforge-light-plan-size-');
  const git = (args: string[]) => gitAt(dir, args);
  mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
  writeFileSync(join(dir, BRIEF), '---\nflow: light\n---\n\nBuild the thing.\n', 'utf8');
  const state: PipelineState = {
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
  mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
  writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), `${JSON.stringify(state, null, 2)}\n`);
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** A `plan` session that writes a plan of `planLines` lines, then replies.
 * `null` writes no document at all: the stage replies ok and the run's one
 * artifact is missing. */
function planStub(dir: string, planLines: number | null) {
  const sessionsCreated: string[] = [];
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  let clock = 0;
  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        sessionsCreated.push(key);
        const id = `ses_${String(sessionsCreated.length)}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        if (planLines === null) return;
        // `planLines: 0` writes an empty file: the meter reads no lines, and the
        // gate must not treat "no measurement" as "within cap" (it measures 0).
        const body = Array.from({ length: planLines }, (_, i) => `plan line ${String(i + 1)}`);
        const text = planLines === 0 ? '' : `${body.join('\n')}\n`;
        writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), text, 'utf8');
      },
      remove: async () => undefined,
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { key: '', prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        // Every stage answers with its own reply shape — the plan
        // `summary`/`commitMessage`, the coder `testsNeedUpdate`, a seat a
        // verdict — so a run past the cap reaches the stages after `plan`.
        const text = s.key.includes('-review:')
          ? '{"verdict": "approve", "findings": []}'
          : s.key === 'implement'
            ? '{"testsNeedUpdate": false, "commitMessage": "feat: implement it"}'
            : '{"summary": "planned", "commitMessage": "docs: plan"}';
        const messages: SessionMessageInfo[] = [];
        for (let i = 0; i < visible; i += 1) {
          clock += 1;
          messages.push({
            id: `m_a${String(clock)}`,
            time: { created: clock },
            type: 'assistant',
            content: [{ type: 'text', text }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(clock)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, sessionsCreated };
}

function optionsFor(dir: string, extra: object = {}): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: BASE,
    ...extra,
  };
}

function savedState(dir: string): PipelineState {
  return JSON.parse(readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8')) as PipelineState;
}

function denialReasons(saved: PipelineState): string {
  return saved.gateLog
    .filter((g) => !g.proceed)
    .map((g) => g.reason ?? '')
    .join('\n');
}

describe('the oversized-plan stop', () => {
  // proves: Test strategy #19 (correctness r1 findings 2 and 6) — a 120-line plan
  // escalates with the size, the cap and the promotion recipe, and the stop is a
  // terminal record. It reports the stage and phase that stopped the run — a
  // light run has no design phase — and the refused plan survives in the worktree
  // and in the branch history, where the recipe it prints tells the operator to
  // promote from (A26).
  it('escalates a plan over the cap with the size, the cap and the recipe', async () => {
    const dir = tmpRepo();
    const { client } = planStub(dir, 120);

    const { status } = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('escalated');

    const md = readFileSync(join(dir, DESIGN_DIR, 'ESCALATION.md'), 'utf8');
    expect(md).toContain(STOP);
    expect(md).toContain(PROMOTION);

    // The record names the stage that stopped and the phase that stop belongs
    // to (A15, Reporting): `plan` reports `code`, never the design phase a light
    // run does not have.
    expect(md).toMatch(/Run stopped after stage `plan`/);
    expect(md).toContain('in the **code** phase');
    expect(md).not.toContain('in the **design** phase');

    // The stop sits at the stage-ok/pre-commit point, so the stage's own commit
    // is skipped — and the refused bytes stay where the stage wrote them, unstaged
    // for the operator to read and promote from. The terminal escalation commit
    // tracks them, so the plan lives in the branch history A26 sells promotion on.
    expect(existsSync(join(dir, DESIGN_DIR, '02-proposal.md'))).toBe(true);
    const planText = readFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), 'utf8');
    expect(planText).toContain('plan line 120');
    expect(gitAt(dir, ['ls-files', `${DESIGN_DIR}/02-proposal.md`]).trim()).toBe(
      `${DESIGN_DIR}/02-proposal.md`,
    );
    expect(gitAt(dir, ['log', '--format=%s'])).not.toMatch(/docs: plan/);

    const saved = savedState(dir);
    expect(saved.status).toBe('escalated');
    expect(denialReasons(saved)).toContain(STOP);
  });

  // proves: Test strategy #19/#5 (performance r1 finding 4, correctness r1
  // finding 5) — one line-count authority: the number in the denial is the
  // meter's own count as the stage recorded it, so the gate and the meter cannot
  // drift apart, and the gate does not meter the document a second time.
  it("denies on the meter's own count, as the stage recorded it", async () => {
    const dir = tmpRepo();
    const { client } = planStub(dir, 120);

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('escalated');

    const saved = savedState(dir);
    const plan = saved.results.find((r) => r.stage === 'plan');
    expect(plan?.docLines).toBe(120);
    expect(denialReasons(saved)).toContain(
      `plan outgrew the light flow (${String(plan?.docLines)} lines, cap 80)`,
    );
  });

  // proves: Test strategy #19 — the gate runs at the stage-ok point, so the stop
  // costs the plan session and nothing else: no `implement` session is spent.
  it('stops before any implement session', async () => {
    const dir = tmpRepo();
    const { client, sessionsCreated } = planStub(dir, 120);

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('escalated');
    expect(sessionsCreated.filter((k) => k !== 'plan')).toEqual([]);
  });

  // proves: Test strategy #19 (The `plan` stage) — the promotion recipe on this
  // record is the gate's own line: `finalizeEscalate` appends the light suffix
  // only for `implement`/`fix`/`code-review`, so a plan stop does not gain it.
  it('names the promotion once, from the gate', async () => {
    const dir = tmpRepo();
    const { client } = planStub(dir, 120);

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('escalated');

    const md = readFileSync(join(dir, DESIGN_DIR, 'ESCALATION.md'), 'utf8');
    expect(md).toContain(PROMOTION);
    expect(md).not.toContain('promote with flow: full and re-run --force');
  });

  // proves: Test strategy #19 (correctness r1c2 finding A) — the record is
  // single-voiced. Its footer is the promotion recipe, not the generic
  // `pipeline resume`: the resume that footer prints lands on `case 'plan'` →
  // `implement`, so following it would build the 120-line plan the flow just
  // refused. A record that prints two opposite instructions is the bug; the
  // promotion is the design's only remedy for this stop (Promotion).
  it('sends a plan stop to promotion, not to a plain resume', async () => {
    const dir = tmpRepo();
    const { client } = planStub(dir, 120);

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('escalated');

    const md = readFileSync(join(dir, DESIGN_DIR, 'ESCALATION.md'), 'utf8');
    expect(md).toContain(PROMOTION);
    expect(md).not.toContain('pipeline resume 002-x');
  });

  // proves: Test strategy #19 (correctness r1c2 finding A) — the stop survives
  // the checkpoint it writes. The state graph is the pin: `decideNext` on the
  // state the stop leaves (`lastStage: 'plan'`, `flow: 'light'`, no code budget
  // spent) escalates on the promotion record instead of handing off to
  // `implement`, and the runner agrees — resuming the stopped run spends no
  // `implement` session on the refused plan.
  it('refuses to build the refused plan when the run is resumed', async () => {
    const dir = tmpRepo();
    const first = planStub(dir, 120);
    const firstRun = await resumePipeline(optionsFor(dir, { client: first.client }));
    expect(firstRun.status).toBe('escalated');

    // The shape that made the bypass reachable: the stop records its stage.
    const saved = savedState(dir);
    expect(saved.status).toBe('escalated');
    expect(saved.lastStage).toBe('plan');

    const next = decideNext(saved, configFor('light', BASE));
    expect(next.kind).toBe('escalate');
    const reason = (next as { reason?: string }).reason ?? '';
    expect(reason).toMatch(/promot|flow: full|--force/);

    const second = planStub(dir, 120);
    const secondRun = await resumePipeline(optionsFor(dir, { client: second.client }));
    expect(secondRun.status).toBe('escalated');
    expect(second.sessionsCreated).not.toContain('implement');
    expect(existsSync(join(dir, DESIGN_DIR, '03-build-notes.md'))).toBe(false);

    // The second record says the same single thing the first one did.
    expect(readFileSync(join(dir, DESIGN_DIR, 'ESCALATION.md'), 'utf8')).toContain(PROMOTION);
  });

  // proves: Test strategy #19 (correctness r1c2 finding A) — the sticky stop is
  // the state of the stop, not a one-way door: the next `plan` pass that clears
  // `planOutgrewFlow` clears it, so a re-plan under the cap hands off to
  // `implement` and the run finishes. The stop gates the plan; it does not end
  // the identity.
  it('clears the stop when a re-plan lands under the cap', async () => {
    const dir = tmpRepo();
    const first = planStub(dir, 120);
    const firstRun = await resumePipeline(optionsFor(dir, { client: first.client }));
    expect(firstRun.status).toBe('escalated');

    const replan = planStub(dir, 60);
    const { status } = await resumePipeline(
      optionsFor(dir, { client: replan.client, forceStage: 'plan' }),
    );
    expect(status).toBe('done');
    expect(replan.sessionsCreated).toContain('implement');
    expect(readFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), 'utf8')).toContain('plan line 60');
  });

  // proves: Test strategy #19/#5 — the cap is the light registry's 80: a plan of
  // exactly 80 lines is not over length, so the run goes on past `implement` to
  // the code panel and ends `done`: the stop is the cap, not the plan's presence.
  it('runs on when the plan is exactly at the cap', async () => {
    const dir = tmpRepo();
    const { client, sessionsCreated } = planStub(dir, 80);

    const { status } = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('done');
    expect(sessionsCreated).toContain('implement');
    expect(sessionsCreated.filter((k) => k.startsWith('code-review:'))).toHaveLength(2);
  });

  // proves: Test strategy #19 (security r1 finding 4, correctness r1 finding 6)
  // — a deterministic stop is never satisfied by the absence of its own input:
  // a `plan` stage that replies ok and writes no `02-proposal.md` escalates at
  // the same pre-commit point, naming the document it needs (the seat stamp and
  // the reviewers' faithfulness check are built from it), and spends no
  // `implement` session.
  it('escalates a plan stage that writes no document', async () => {
    const dir = tmpRepo();
    const { client, sessionsCreated } = planStub(dir, null);

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('escalated');
    expect(sessionsCreated.filter((k) => k !== 'plan')).toEqual([]);

    const md = readFileSync(join(dir, DESIGN_DIR, 'ESCALATION.md'), 'utf8');
    expect(md).toContain('plan document not written');
    expect(md).toContain('02-proposal.md');

    const saved = savedState(dir);
    expect(saved.status).toBe('escalated');
    expect(denialReasons(saved)).toContain('plan document not written');
  });

  // proves: Test strategy #19/#5 — the cap is a cap, not a floor: a plan file
  // that exists with no bytes measures 0, 0 is within 80, and the gate stays
  // silent. Only the unmeasurable document — the one that is not there — is the
  // refusal.
  it('runs on when the plan measures zero lines', async () => {
    const dir = tmpRepo();
    const { client, sessionsCreated } = planStub(dir, 0);

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('done');
    expect(sessionsCreated).toContain('implement');
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
  });
});
