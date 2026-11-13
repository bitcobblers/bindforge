import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { configFor, seatsFor } from '../../src/pipeline/flows.js';
import { reviewSessionsUsed } from '../../src/pipeline/panel.js';
import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import { DEFAULT_CONFIG, type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// The light budgets, bound in a run: maxReviews.code 6 and maxPassesPerRound 3
// bind at the same point — three passes of a 2-seat panel — so the pass-backstop
// record is the terminal record for a stubborn panel, and it says which flow it
// stopped. `--extend-budget` buys sessions on the STATE, never in the config,
// and a resume with the CLI's own config `{ ...DEFAULT_CONFIG, maxParallelReviews: 1 }`
// must drive the light numbers, not the five-seat full panel.
//
// The two counters keep their own units (A16, and the round-1 correctness
// finding 1): `state.codeBudgetUsed` counts the mutating rounds each `fix`
// charges, `maxRounds: 2` is its cap, and the sessions a phase spent are
// `reviewSessionsUsed(state, 'code')`, which is what `maxReviews.code: 6` caps.
// Reading a session count off the round counter is what unbounded the red-tests
// loop and cut the dissent loop to one round.
// ---------------------------------------------------------------------------

const DESIGN_DIR = 'docs/designs/002-x';
const BRIEF = 'docs/prompts/002-x.md';
const LIGHT_SEATS = ['correctness', 'testing'];
const LIGHT_SUFFIX = 'the light flow (flow: light); promote with flow: full and re-run --force';
/** The shared fix-round backstop, with the light `maxRounds: 2` spent. */
const FIX_ROUND_STOP = 'dissent and the fix-round backstop is exhausted (2/2)';
/** The red-tests stop: the round budget, not a session cap, ends it. */
const RED_TESTS_STOP = 'tests not green at the code-review budget limit';

/** The CLI's config shape: `cli.ts` spreads DEFAULT_CONFIG and adds the lane. */
function cliConfig(): PipelineConfig {
  return { ...DEFAULT_CONFIG, maxParallelReviews: 1 };
}

/** A base config whose every number is the full flow's: a light run that reads
 * this literal instead of `configFor` fails these tests. */
const BASE: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

function lightState(partial: Partial<PipelineState> = {}): PipelineState {
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
    ...partial,
  };
}

function tmpRepo(state: PipelineState, checksGreen = true): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-light-budget-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
  writeFileSync(join(dir, BRIEF), '---\nflow: light\n---\n\nBuild the thing.\n', 'utf8');
  mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
  writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), `${JSON.stringify(state, null, 2)}\n`);
  // The test gate the runner reads as `testsGreen`: a red gate is the light
  // flow's `fix` loop entered from `implement`, the path no session cap gates.
  const exit = checksGreen ? '0' : '1';
  writeFileSync(
    join(dir, 'package.json'),
    `{"name":"t","scripts":{"check":"node -e \\"process.exit(${exit})\\""}}\n`,
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
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

/** Every seat dissents every pass; `dissenters` names the seats that do. */
function dissentStub(dir: string, dissenters: string[]) {
  const sessionsCreated: string[] = [];
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  let clock = 0;
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const reject = JSON.stringify({
    verdict: 'request-changes',
    findings: [{ severity: 'major', location: 'Test plan', problem: 'nothing covers it' }],
  });

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        sessionsCreated.push(key);
        const id = `ses_${String(sessionsCreated.length)}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string; text?: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        if (s.key === 'plan') {
          const plan = '# Plan\n\n## Problem\n\nx\n';
          writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), plan, 'utf8');
        }
        if (s.key === 'fix') {
          // The code-phase stamp is `HEAD` (A5), taken at the pass entry that
          // follows this mutating stage: a fix commit moves it, which is what
          // sends the stale approvals into a ratification pass.
          writeFileSync(
            join(dir, DESIGN_DIR, '02-proposal.md'),
            `# Plan\n\n## Problem\n\nfixed at ${String(sessionsCreated.length)}\n`,
            'utf8',
          );
        }
      },
      remove: async () => undefined,
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { key: '', prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const text = s.key.includes('-review:')
          ? dissenters.includes(s.key.split(':')[1] ?? '')
            ? reject
            : approve
          : s.key === 'implement' || s.key === 'fix'
            ? JSON.stringify({ testsNeedUpdate: false, commitMessage: 'fix: address findings' })
            : JSON.stringify({ summary: 'ok', commitMessage: 'docs: plan' });
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

function savedState(dir: string): PipelineState {
  return JSON.parse(readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8')) as PipelineState;
}

function marker(dir: string): string {
  return readFileSync(join(dir, DESIGN_DIR, 'ESCALATION.md'), 'utf8');
}

function codeSessions(created: string[]): string[] {
  return created.filter((k) => k.startsWith('code-review:'));
}

function denialReasons(saved: PipelineState): string {
  return saved.gateLog
    .filter((g) => !g.proceed)
    .map((g) => g.reason ?? '')
    .join('\n');
}

/** The grant count an over-cap consumption line prints as left (A27, A16). */
function grantLeft(reason: string): string | null {
  return /(\d+) session grant/.exec(reason)?.[1] ?? null;
}

describe('the light review budget, bound in a run', () => {
  // proves: Test strategy #14(a) — a panel that dissents every pass stops on the
  // PASS-BACKSTOP record at 6 code sessions: the light caps, not full's 20. The
  // sessions are `reviewSessionsUsed`; the round counter is the `fix` charges.
  it('stops a panel that dissents every pass at 6 sessions on the pass backstop', async () => {
    const dir = tmpRepo(lightState());
    const { client, sessionsCreated } = dissentStub(dir, LIGHT_SEATS);

    const status = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('escalated');
    expect(codeSessions(sessionsCreated)).toHaveLength(6);

    const saved = savedState(dir);
    expect(reviewSessionsUsed(saved, 'code')).toBe(6);
    expect(denialReasons(saved)).toContain('pass backstop');
    expect(denialReasons(saved)).toContain('cap 3 passes per round');

    // proves: Goals (`fix` bounded at 2 rounds), correctness r1 finding 1 — the
    // dissenter gets its SECOND `fix` round, and the round counter reads the two
    // charges, not the six sessions: `maxRounds: 2` is a round budget.
    expect(sessionsCreated.filter((k) => k === 'fix')).toHaveLength(2);
    expect(saved.codeBudgetUsed).toBe(2);

    // The record says which flow stopped it (finalizeEscalate appends the suffix).
    const escalate = saved.gateLog.filter((g) => g.gate === 'escalate');
    expect(escalate.map((g) => g.reason ?? '').join('\n')).toContain(LIGHT_SUFFIX);
    expect(marker(dir)).toContain(LIGHT_SUFFIX);
  });

  // proves: Test strategy #14(a) — the two caps bind at the same point, and the
  // pass backstop is the record that prints: the session-budget string stays out
  // of the round the pass cap stopped.
  it('does not print the session-budget string for the pass-cap stop', async () => {
    const dir = tmpRepo(lightState());
    const { client } = dissentStub(dir, LIGHT_SEATS);

    expect(await resumePipeline(optionsFor(dir, { client }))).toBe('escalated');
    expect(marker(dir)).not.toContain('review session budget exhausted');
  });

  // proves: Test strategy #14(b) — one dissenter runs single-seat passes, so the
  // same pass backstop binds at 4 sessions, well under the 6-session budget.
  it('stops one dissenter every pass at 4 sessions', async () => {
    const dir = tmpRepo(lightState());
    const { client, sessionsCreated } = dissentStub(dir, ['testing']);

    const status = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('escalated');
    expect(codeSessions(sessionsCreated)).toHaveLength(4);

    const saved = savedState(dir);
    expect(reviewSessionsUsed(saved, 'code')).toBe(4);
    expect(denialReasons(saved)).toContain('pass backstop');
    expect(marker(dir)).toContain(LIGHT_SUFFIX);
    expect(marker(dir)).not.toContain('review session budget exhausted');

    // proves: correctness r1 finding 1 — the single dissenter is bounded the same
    // way: two `fix` charges, and the sessions are the phase's own count.
    expect(sessionsCreated.filter((k) => k === 'fix')).toHaveLength(2);
    expect(saved.codeBudgetUsed).toBe(2);
  });

  // proves: Test strategy #14(c) — the escape hatch reads the same budget:
  // resuming the run above with `--stage code-review` re-enters a pass at 6/6
  // and stops on the SESSION-BUDGET record, with the light suffix on it too.
  it('escalates a forced code-review resume on the session budget at 6/6', async () => {
    const dir = tmpRepo(lightState());
    const first = dissentStub(dir, LIGHT_SEATS);
    expect(await resumePipeline(optionsFor(dir, { client: first.client }))).toBe('escalated');
    expect(codeSessions(first.sessionsCreated)).toHaveLength(6);

    const second = dissentStub(dir, LIGHT_SEATS);
    const status = await resumePipeline(
      optionsFor(dir, { client: second.client, forceStage: 'code-review' }),
    );
    expect(status).toBe('escalated');

    // No session was spent: the budget check is at pass entry.
    expect(codeSessions(second.sessionsCreated)).toHaveLength(0);
    const md = marker(dir);
    expect(md).toContain('review session budget exhausted (6/6 sessions)');
    expect(md).toContain(LIGHT_SUFFIX);
  });

  // proves: Goals (`fix` bounded at 2 rounds) + Budget resolution (A16) —
  // correctness r1 finding 1's first shape: the `fix` loop entered from a red
  // `testsGreen` gate is bound by the round budget the design states it in. No
  // session cap gates that path (the pass-entry cap gates review passes), so
  // with the round counter repurposed into sessions it read `0 < 2` forever:
  // the run spent `fix` sessions unboundedly with no escalation at all. Now it
  // charges two rounds and stops on the round-limit record.
  it('escalates a light run whose tests stay red after two fix rounds', async () => {
    const dir = tmpRepo(lightState(), false);
    const { client, sessionsCreated } = dissentStub(dir, LIGHT_SEATS);

    const status = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('escalated');

    // The loop terminates: plan, implement, and exactly two `fix` sessions.
    expect(sessionsCreated).toEqual(['plan', 'implement', 'fix', 'fix']);

    const saved = savedState(dir);
    expect(saved.codeBudgetUsed).toBe(2);
    expect(reviewSessionsUsed(saved, 'code')).toBe(0);
    expect(denialReasons(saved)).toContain(RED_TESTS_STOP);

    // The stop is the run's terminal record and names the flow, so an operator
    // reads a light-flow limit and the promotion recipe.
    const md = marker(dir);
    expect(md).toContain(RED_TESTS_STOP);
    expect(md).toContain(LIGHT_SUFFIX);
  });

  // proves: Test strategy #14(a) (A16, A27) — `--extend-budget` is state-side: it
  // lands as `extensionSessions` and reaches no config key. The pass cap binds
  // first inside the round, so the session cap is never reached and the grant is
  // never spent: it buys nothing the round did not already stop at.
  it('leaves a grant unspent when the round stops at the pass backstop', async () => {
    const dir = tmpRepo(lightState());
    const { client, sessionsCreated } = dissentStub(dir, LIGHT_SEATS);

    const status = await resumePipeline(optionsFor(dir, { client, extendBudget: 1 }));
    expect(status).toBe('escalated');
    expect(codeSessions(sessionsCreated)).toHaveLength(6);

    const saved = savedState(dir);
    expect(saved.extensionSessions).toBe(1);
    expect(denialReasons(saved)).toContain('pass backstop');
    // No over-cap pass entry happened, so no grant was spent and logged.
    expect(saved.gateLog.filter((g) => g.gate === 'operatorExtension')).toHaveLength(0);
    expect(marker(dir)).not.toContain('review session budget exhausted');
  });

  // proves: Test strategy #14(a)/(c) (A16, A27) — a grant binds at the over-cap
  // pass entry and is consumed there: one grant buys exactly one 2-seat pass, so
  // the sessions go 6 → 8, the state's grant reads 0, and the consent names the
  // single session it paid for. An unspent grant would unbind the flow's stop for
  // the rest of the run, which is the one deterministic stop the light flow has.
  // The run then stops on the budget the design bounds it with: the first run
  // spent its two `fix` rounds, so the dissent that follows the granted pass
  // escalates on the fix-round record, not on the session cap (deviation 10).
  it('spends one grant per over-cap pass and stops on the spent fix budget', async () => {
    const dir = tmpRepo(lightState());
    const first = dissentStub(dir, LIGHT_SEATS);
    expect(await resumePipeline(optionsFor(dir, { client: first.client }))).toBe('escalated');
    expect(codeSessions(first.sessionsCreated)).toHaveLength(6);

    const second = dissentStub(dir, LIGHT_SEATS);
    const status = await resumePipeline(
      optionsFor(dir, { client: second.client, forceStage: 'code-review', extendBudget: 1 }),
    );
    expect(status).toBe('escalated');
    expect(codeSessions(second.sessionsCreated)).toHaveLength(2);

    const saved = savedState(dir);
    expect(reviewSessionsUsed(saved, 'code')).toBe(8);
    expect(saved.extensionSessions).toBe(0);
    expect(saved.codeBudgetUsed).toBe(2);

    const consents = saved.gateLog.filter((g) => g.gate === 'operatorExtension');
    expect(consents).toHaveLength(1);
    expect(consents.map((g) => g.reason ?? '').join('\n')).toContain(
      'extended the review budget by 1 review session(s) over the caps of 0/6',
    );

    const md = marker(dir);
    expect(md).toContain(FIX_ROUND_STOP);
    expect(md).not.toContain('review session budget exhausted');
    expect(md).toContain(LIGHT_SUFFIX);
  });

  // proves: Test strategy #14(a)/(c) (A16, A27; security r1 finding 2) — the
  // grant count the run prints is the count the state carries, and it goes down:
  // `--extend-budget 2` prints the grant left at the entry it pays for, buys
  // exactly one 2-seat pass (the flow's own stop is reached after it), and leaves
  // the second grant on the state unspent. A grant that lifted the cap for the
  // rest of the run would unbind the flow's deterministic stop.
  it('prints a grant count that decreases and leaves the surplus unspent', async () => {
    const dir = tmpRepo(lightState());
    const first = dissentStub(dir, LIGHT_SEATS);
    expect(await resumePipeline(optionsFor(dir, { client: first.client }))).toBe('escalated');
    expect(codeSessions(first.sessionsCreated)).toHaveLength(6);

    const second = dissentStub(dir, LIGHT_SEATS);
    const status = await resumePipeline(
      optionsFor(dir, { client: second.client, forceStage: 'code-review', extendBudget: 2 }),
    );
    expect(status).toBe('escalated');
    // One grant spent, one 2-seat pass: the fix-round budget ends the run, so
    // the second over-cap entry the wider grant could buy is never reached.
    expect(codeSessions(second.sessionsCreated)).toHaveLength(2);

    const saved = savedState(dir);
    expect(reviewSessionsUsed(saved, 'code')).toBe(8);
    expect(saved.extensionSessions).toBe(1);
    expect(saved.codeBudgetUsed).toBe(2);

    const consents = saved.gateLog.filter((g) => g.gate === 'operatorExtension');
    expect(consents).toHaveLength(1);
    const printed = consents.map((g) => grantLeft(g.reason ?? ''));
    expect(printed, 'each entry names the grant left, and the count goes down').toEqual(['1']);

    const md = marker(dir);
    expect(md).toContain(FIX_ROUND_STOP);
    expect(md).not.toContain('review session budget exhausted');
    expect(md).toContain(LIGHT_SUFFIX);
  });
});

describe('the effective config on a light resume', () => {
  // proves: Test strategy #15 — the failure this design can hide: a resume with
  // the CLI-shaped config must run the light numbers. The panel is the registry
  // pair — the seat list, not only the numbers — the pass cap is the light 3,
  // and the code budget is the light 6, not DEFAULT_CONFIG's 20.
  it('drives a resumed light run with the light panel and budgets', async () => {
    const dir = tmpRepo(lightState());
    const { client, sessionsCreated } = dissentStub(dir, LIGHT_SEATS);
    const cliCfg = cliConfig();

    const status = await resumePipeline(optionsFor(dir, { client, cfg: cliCfg }));
    expect(status).toBe('escalated');

    const launched = codeSessions(sessionsCreated);
    expect(launched.slice(0, 2)).toEqual(LIGHT_SEATS.map((seat) => `code-review:${seat}`));
    expect(launched).toHaveLength(6);

    // The panel the drive loop used is the panel the resolver resolves.
    expect(seatsFor('code', configFor('light', cliCfg))).toEqual(
      launched.slice(0, 2).map((key) => key.split(':')[1] ?? ''),
    );

    const saved = savedState(dir);
    expect(reviewSessionsUsed(saved, 'code')).toBe(6);
    expect(saved.codeBudgetUsed).toBe(2);
    expect(denialReasons(saved)).toContain('cap 3 passes per round');

    // The record an operator reads carries the effective numbers.
    const md = marker(dir);
    expect(md).toContain('rounds 2, code reviews 6');
    expect(md).toContain(LIGHT_SUFFIX);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(true);
  });

  // proves: Test strategy #15 (Risks, budget-drift row) — the config the run
  // resolves is `configFor(flow, base)`: the full-flow literal's numbers are
  // gone, the shared knobs and the operator's lane survive.
  it('resolves the light numbers out of the CLI-shaped config', () => {
    const cfg = configFor('light', cliConfig());
    expect(cfg.maxRounds).toBe(2);
    expect(cfg.maxReviews).toEqual({ design: 0, code: 6 });
    expect(cfg.maxPassesPerRound).toBe(3);
    expect(seatsFor('code', cfg)).toEqual(LIGHT_SEATS);
    expect(cfg.maxParallelReviews).toBe(1);
  });
});
