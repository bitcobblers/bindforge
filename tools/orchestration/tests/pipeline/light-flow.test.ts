import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// ADR-010's light flow end to end: `plan → implement → code-review`, a 2-seat
// code panel, and nothing else. The full flow's stages must not appear at all:
// no research artifact, no design panel, no editorial round, no doc-sync
// commit. The whole run fits 2 + seats × 3 sessions, against the 43–96 sessions
// the full flow spent on the same shape of change.
// ---------------------------------------------------------------------------

const DESIGN_DIR = 'docs/designs/002-x';
const BRIEF = 'docs/prompts/002-x.md';
const LIGHT_SEATS = ['correctness', 'testing'];

/** Base config: the CLI's own shape. The light registry replaces the budgets,
 * so every number a light assertion reads is the flow's, not this literal's. */
const BASE: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

/** A plan inside the cap, carrying the five plan-note sections. */
function planDoc(marker: string): string {
  return [
    '# Plan: 002-x',
    '',
    '## Problem',
    `the thing is missing (${marker})`,
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

function lines(n: number): string {
  return Array.from({ length: n }, (_, i) => `plan line ${String(i + 1)}`).join('\n');
}

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

/** Full-flow sibling: the same repo, an ADR source, no `flow` key. */
function fullState(partial: Partial<PipelineState> = {}): PipelineState {
  return {
    ...lightState(),
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    flow: undefined,
    lastStage: 'proposal',
    ...partial,
  };
}

/** Repo with a brief carrying `flow: light`, a committed checkpoint, and a
 * `npm run check` gate that exits 0. The design dir holds the checkpoint only:
 * every artifact inside it has to be written by a stage to show up. */
function tmpRepo(state: PipelineState): string {
  const dir = mkTmpDir('bindforge-light-flow-');
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(join(dir, BRIEF), `---\nflow: light\n---\n\nBuild the thing.\n`, 'utf8');
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
    'utf8',
  );
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

function optionsFor(dir: string, extra: object = {}): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: BASE,
    ...extra,
  };
}

interface StubOptions {
  /** seat stage key → session ordinals that answer request-changes. */
  rejects: Record<string, number[]>;
  /** the plan stage writes a plan of this many lines (default: the five sections) */
  planLines?: number;
  /** the fix stage rewrites the plan and commits, so `HEAD` — the code-phase
   * stamp a pass entry takes (A5) — moves under the approvals just recorded. */
  fixRewritesPlan?: boolean;
}

/**
 * Stub client for a light run. Each stage does on disk what the real agent
 * does: the plan writes `02-proposal.md`, the coder writes source and build
 * notes, a seat writes the report at the path its prompt names. Replies appear
 * one poll after the prompt, the way a real turn lands.
 */
function lightStub(dir: string, opts: StubOptions) {
  const sessionsCreated: string[] = [];
  const prompts: Array<{ key: string; text: string }> = [];
  const reports: string[] = [];
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  let clock = 0;

  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const reject = JSON.stringify({
    verdict: 'request-changes',
    findings: [{ severity: 'major', location: 'Test plan', problem: 'nothing covers it' }],
  });

  const replyFor = (key: string, ordinal: number): string => {
    if (key.includes('-review:')) {
      return (opts.rejects[key] ?? []).includes(ordinal) ? reject : approve;
    }
    if (key === 'plan') return JSON.stringify({ summary: 'planned', commitMessage: 'docs: plan' });
    if (key === 'implement' || key === 'fix')
      return JSON.stringify({
        testsNeedUpdate: false,
        commitMessage: key === 'fix' ? 'fix: address the findings' : 'feat: implement it',
      });
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'proposal-revise')
      return JSON.stringify({ changed: true, summary: 'ok', commitMessage: 'docs: revise' });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 1, commitMessage: 'test: pin the contract' });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok', commitMessage: 'docs: ok' });
  };

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        sessionsCreated.push(key);
        const ordinal = sessionsCreated.filter((k) => k === key).length - 1;
        const id = `ses_${String(sessionsCreated.length)}`;
        sessions.set(id, { key, ordinal, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string; text?: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        const text = input.text ?? '';
        prompts.push({ key: s.key, text });

        if (s.key === 'plan') {
          const body = opts.planLines === undefined ? planDoc('v1') : lines(opts.planLines);
          writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), `${body}\n`, 'utf8');
        }
        if (s.key === 'implement') {
          mkdirSync(join(dir, 'src'), { recursive: true });
          writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n', 'utf8');
          writeFileSync(
            join(dir, DESIGN_DIR, '03-build-notes.md'),
            '# Build notes\n\nshipped it\n',
            'utf8',
          );
        }
        if (s.key === 'fix' && opts.fixRewritesPlan) {
          writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), `${planDoc('fixed')}\n`, 'utf8');
        }
        if (s.key.includes('-review:')) {
          // A seat writes the report at the path its prompt names, so a missing
          // or mis-numbered report path is a missing file here, not a guess.
          const found = /docs\/designs\/[^\s`]+\/reviews\/code\/[\w.-]+\.md/.exec(text);
          if (found !== null) {
            const path = join(dir, found[0]);
            mkdirSync(join(path, '..'), { recursive: true });
            writeFileSync(path, `report from ${s.key}\n`, 'utf8');
            reports.push(found[0]);
          }
        }
      },
      remove: async () => undefined,
      // The independence walk's enumeration endpoint: no child sessions here.
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

  return { client, sessionsCreated, prompts, reports };
}

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

function savedState(dir: string): PipelineState {
  return JSON.parse(readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8')) as PipelineState;
}

function transcripts(dir: string): string[] {
  const tdir = join(dir, DESIGN_DIR, 'transcripts');
  return existsSync(tdir) ? readdirSync(tdir) : [];
}

describe('light run end to end', () => {
  // proves: Test strategy #12 — the light route runs exactly plan → implement →
  // code-review r1c0 → done, on the run's own branch.
  it('routes plan to implement to code review round 1 and ends done', async () => {
    const dir = tmpRepo(lightState());
    const { client, sessionsCreated } = lightStub(dir, { rejects: {} });

    const { status } = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('done');

    expect(sessionsCreated).toEqual([
      'plan',
      'implement',
      'code-review:correctness',
      'code-review:testing',
    ]);

    const saved = savedState(dir);
    expect(saved.flow).toBe('light');
    expect(saved.status).toBe('done');
    expect(saved.results.map((r) => r.stage)).toEqual([
      'plan',
      'implement',
      'code-review:correctness',
      'code-review:testing',
    ]);
    expect(gitAt(dir, ['rev-parse', '--abbrev-ref', 'HEAD']).trim()).toBe('pipeline/002-x');
  });

  // proves: Test strategy #12 — the plan is the run's only document, written at
  // the full flow's proposal path (the design stamp, the size meter and the
  // `reviews/code/` reports all hang off it), and the size meter measured it.
  it('writes the plan, the build notes and both seat reports', async () => {
    const dir = tmpRepo(lightState());
    const { client } = lightStub(dir, { rejects: {} });

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('done');

    expect(existsSync(join(dir, DESIGN_DIR, '02-proposal.md'))).toBe(true);
    expect(existsSync(join(dir, DESIGN_DIR, '03-build-notes.md'))).toBe(true);
    for (const seat of LIGHT_SEATS) {
      expect(existsSync(join(dir, DESIGN_DIR, 'reviews', 'code', `${seat}-r1c0.md`))).toBe(true);
    }

    const saved = savedState(dir);
    const plan = saved.results.find((r) => r.stage === 'plan');
    expect(plan?.docLines).toBeGreaterThan(0);
  });

  // proves: Test strategy #12 — the full flow's stages are absent: no research
  // artifact, no design review, no editorial round, no doc-sync commit.
  it('writes nothing from the full flow', async () => {
    const dir = tmpRepo(lightState());
    const { client, sessionsCreated } = lightStub(dir, { rejects: {} });

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('done');

    expect(existsSync(join(dir, DESIGN_DIR, '01-research.md'))).toBe(false);
    expect(existsSync(join(dir, DESIGN_DIR, 'reviews', 'design'))).toBe(false);
    expect(existsSync(join(dir, DESIGN_DIR, 'editorial'))).toBe(false);

    for (const name of transcripts(dir)) {
      expect(name).not.toMatch(/^(research|proposal|editorial|doc-sync|test-authoring)/);
    }
    expect(gitAt(dir, ['log', '--format=%s'])).not.toMatch(/doc-sync|editorial/i);
    expect(sessionsCreated.filter((k) => k.includes('review'))).toHaveLength(LIGHT_SEATS.length);
  });

  // proves: Test strategy #12 — the session bound: 2 stage sessions plus at most
  // 3 passes of the 2-seat panel.
  it('spends at most 2 + seats × 3 sessions', async () => {
    const dir = tmpRepo(lightState());
    const { client, sessionsCreated } = lightStub(dir, { rejects: {} });

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('done');

    expect(sessionsCreated.length).toBeLessThanOrEqual(2 + LIGHT_SEATS.length * 3);
  });
});

describe('the 2-seat code funnel', () => {
  // proves: Test strategy #13 — one dissent runs `fix`, the dissent pass prompts
  // only the dissenter, and the stale approver runs one ratification pass on the
  // current code stamp before the panel converges.
  it('re-reviews only the dissenter, then one ratification pass', async () => {
    const dir = tmpRepo(lightState());
    const { client, sessionsCreated } = lightStub(dir, {
      rejects: { 'code-review:testing': [0] },
      fixRewritesPlan: true,
    });

    const { status } = await resumePipeline(optionsFor(dir, { client, cfg: BASE }));
    expect(status).toBe('done');

    expect(sessionsCreated.filter((k) => k === 'code-review:testing')).toHaveLength(2);
    expect(sessionsCreated.filter((k) => k === 'code-review:correctness')).toHaveLength(2);
    expect(sessionsCreated.filter((k) => k === 'fix')).toHaveLength(1);

    const names = transcripts(dir);
    expect(names).toContain('code-review-testing-r1c1.md');
    expect(names).toContain('code-review-correctness-r1c2.md');

    const saved = savedState(dir);
    expect(saved.seats?.code?.testing?.verdict).toBe('approve');
    expect(saved.seats?.code?.correctness?.verdict).toBe('approve');
    expect(saved.seats?.code?.testing?.stamp).toBe(saved.seats?.code?.correctness?.stamp);
  });

  // proves: Test strategy #13 — the dissent pass is seat-scoped: it names the
  // dissenting seat's own report and carries its findings re-injected, and never
  // points the seat at another seat's report.
  it('prompts the dissent pass for one seat only', async () => {
    const dir = tmpRepo(lightState());
    const { client, prompts } = lightStub(dir, {
      rejects: { 'code-review:testing': [0] },
      fixRewritesPlan: true,
    });

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('done');

    const dissent = prompts.filter(
      (p) => p.key === 'code-review:testing' && p.text.includes('reviews/code/testing-r1c1.md'),
    );
    expect(dissent.length).toBeGreaterThan(0);
    const text = dissent.map((p) => p.text).join('\n');
    expect(text).toContain('dissent pass c1');
    expect(text).toContain('nothing covers it');
    expect(text).not.toContain('correctness-r1c0.md');

    const ratify = prompts.filter(
      (p) =>
        p.key === 'code-review:correctness' && p.text.includes('reviews/code/correctness-r1c2.md'),
    );
    expect(ratify.length).toBe(1);
    expect(ratify[0]?.text).toContain('ratification pass c2');
  });
});

describe('the ownership line reaches the coder prompt', () => {
  // proves: Test strategy #11 — the light run's `implement` and `fix` prompts
  // carry the light ownership line: the coder owns the test files and no test
  // engineer exists in this flow.
  it('tells the light coder it owns the test files', async () => {
    const dir = tmpRepo(lightState());
    const { client, prompts } = lightStub(dir, { rejects: {} });

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('done');

    const coderPrompts = prompts.filter((p) => p.key === 'implement');
    expect(coderPrompts.length).toBeGreaterThan(0);
    const text = coderPrompts.map((p) => p.text).join('\n');
    expect(text).toMatch(/test files/i);
    expect(text).toMatch(/no test engineer/i);
    expect(text).not.toContain('Never edit test files');
  });

  // proves: Test strategy #11 — the full flow keeps the standing rule and its
  // escape: the same template rendered for a full run names the test engineer
  // and the `testsNeedUpdate` route.
  it('keeps the standing rule in a full run', async () => {
    const dir = tmpRepo(fullState());
    const { client, prompts } = lightStub(dir, { rejects: {} });

    await resumePipeline(optionsFor(dir, { client, forceStage: 'implement' }));

    const coderPrompts = prompts.filter((p) => p.key === 'implement');
    expect(coderPrompts.length).toBeGreaterThan(0);
    const text = coderPrompts.map((p) => p.text).join('\n');
    expect(text).toContain('Never edit test files');
    expect(text).toContain('testsNeedUpdate');
  });
});
