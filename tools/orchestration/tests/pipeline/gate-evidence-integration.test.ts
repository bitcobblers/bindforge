import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-016 Decision 2 end to end. The gate record is captured into the
// checkpoint after the code stages run, and it reaches the next stage's prompt:
// a red gate's failing-leg tail lands in the fix prompt (the coder acts on the
// actual failure, not a two-word "cargo gate red"), and a green gate's
// `test result:` line lands in a review seat's prompt so a behaviour claim has a
// citable line. The light flow is the shortest driver that reaches implement,
// the check gate, the fix loop and a code panel.
// ---------------------------------------------------------------------------

const DESIGN_DIR = 'docs/designs/002-x';
const BRIEF = 'docs/prompts/002-x.md';

const BASE: PipelineConfig = {
  maxRounds: 2,
  maxReviews: { design: 0, code: 6 },
  maxPassesPerRound: 3,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

/** The `check` script's generator: green emits a cargo-shaped summary, red
 * fails with a tail that a seat or coder must be able to read, and dirty fails
 * the same way with a tail that additionally carries the two line classes the
 * design refuses to commit — a credential-bearing line and an absolute host
 * path. */
function gateScript(): string {
  return [
    "const mode = process.argv[2] ?? 'green';",
    "if (mode === 'green') {",
    "  console.log('running integration suite');",
    "  console.log('test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured');",
    '  process.exit(0);',
    '}',
    "if (mode === 'dirty') {",
    "  console.log('test result: FAILED. 2 passed; 1 failed; 0 ignored');",
    "  console.error('npm ERR! code E401 //registry.npmjs.org/:_authToken=REGSECRETabc123');",
    "  console.error('npm ERR! GET https://ci-bot:hunter2pw@registry.example.com/token=DEADBEEF');",
    "  console.error('error: could not compile /home/ci/work/bindforge/crates/core/src/lib.rs');",
    '  process.exit(1);',
    '}',
    "console.log('test result: FAILED. 2 passed; 1 failed; 0 ignored');",
    "console.error('boom at src/lib.rs:9:13');",
    'process.exit(1);',
  ].join('\n');
}

/** The line classes planted in the `dirty` leg's output: the credential shapes
 * the design names (`_authToken`, `://<user>:<pass>@`, `token=`) and an
 * absolute host path. */
const PLANTED = [
  'REGSECRETabc123',
  '_authToken',
  'hunter2pw',
  'token=DEADBEEF',
  '/home/ci/work/bindforge',
];

function tmpGateRepo(mode: 'green' | 'red' | 'dirty'): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-gate-e2e-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });

  mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
  mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
  writeFileSync(join(dir, BRIEF), '---\nflow: light\n---\n\nBuild the thing.\n', 'utf8');
  writeFileSync(join(dir, 'gate.mjs'), gateScript(), 'utf8');
  writeFileSync(
    join(dir, 'package.json'),
    JSON.stringify({ name: 't', scripts: { check: `node gate.mjs ${mode}` } }),
  );

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
  writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), `${JSON.stringify(state, null, 2)}\n`);
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** A stub session that writes the plan document, then answers each stage with
 * that stage's reply shape, so a green run drives through the code panel. */
function stageStub(dir: string): { client: OpenCodeClient; sessionsCreated: string[] } {
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  const sessionsCreated: string[] = [];
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
        if (s.key === 'plan') {
          writeFileSync(
            join(dir, DESIGN_DIR, '02-proposal.md'),
            Array.from({ length: 30 }, (_, i) => `plan line ${String(i + 1)}`).join('\n') + '\n',
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
          ? '{"verdict": "approve", "findings": []}'
          : s.key === 'implement' || s.key === 'fix'
            ? '{"testsNeedUpdate": false, "commitMessage": "feat: make it so"}'
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

function optionsFor(dir: string, client: OpenCodeClient): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: BASE,
    client,
  };
}

function savedState(dir: string): PipelineState {
  return JSON.parse(readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8')) as PipelineState;
}

/** The rendered prompt of the last stage result whose stage key starts with
 * `stage`, read back from the transcript the runner wrote. */
function promptOfStage(dir: string, saved: PipelineState, stage: string): string | undefined {
  const hit = saved.results.filter((r) => r.stage.startsWith(stage)).at(-1);
  if (hit?.transcript === undefined) return undefined;
  try {
    return readFileSync(join(dir, hit.transcript), 'utf8');
  } catch {
    return undefined;
  }
}

describe('gate evidence round trip (ADR-016 Decision 2)', () => {
  // proves: Test strategy #9 — a red gate: the checkpoint's `gateEvidence.legs`
  // names the failing command with a nonzero exit and a tail carrying the
  // failure, and the next fix prompt carries that tail.
  it('records a red gate leg and feeds its tail to the fix prompt', async () => {
    const dir = tmpGateRepo('red');
    const { client } = stageStub(dir);
    await resumePipeline(optionsFor(dir, client));

    const saved = savedState(dir);
    const evidence = saved.gateEvidence;
    expect(evidence, 'the red gate persisted a record').toBeDefined();
    const failing = evidence?.legs.find((leg) => leg.exit !== 0);
    expect(failing, 'a leg records the nonzero exit').toBeDefined();
    expect(failing?.exit).not.toBe(0);
    expect(failing?.tail).toContain('FAILED');

    const fixPrompt = promptOfStage(dir, saved, 'fix');
    expect(fixPrompt, 'a fix session ran').toBeDefined();
    expect(fixPrompt).toContain('boom at src/lib.rs');
  });

  // proves: Test strategy #10 — a green gate: each green leg's `test result:`
  // line renders into the code-review seat's prompt, so a behaviour claim has a
  // citable line.
  it('renders a green gate test-result line into the code-review prompt', async () => {
    const dir = tmpGateRepo('green');
    const { client } = stageStub(dir);
    await resumePipeline(optionsFor(dir, client));

    const saved = savedState(dir);
    const seatPrompt = promptOfStage(dir, saved, 'code-review');
    expect(seatPrompt, 'a code-review seat ran').toBeDefined();
    expect(seatPrompt).toContain('test result: ok. 3 passed; 0 failed');
  });
});

describe('the gate record commits no secrets and no host paths (ADR-016 Decision 2)', () => {
  // proves: security r2c0 finding 1 (accepted) — the design's line-selective
  // citation rule cannot live only in the build note, because this repo commits
  // the checkpoint (`git add -A` takes `.pipeline.json`, no .gitignore entry) and
  // repeats the same block in every later prompt. A credential-bearing line and
  // an absolute-path line in a leg's output must not survive into the persisted
  // record — capture or write, the runner's choice — while the citable failure
  // line the coder and the seats must act on still does.
  it('keeps secrets and host paths out of the record and the prompts', async () => {
    const dir = tmpGateRepo('dirty');
    const { client } = stageStub(dir);
    await resumePipeline(optionsFor(dir, client));

    const saved = savedState(dir);
    const evidence = saved.gateEvidence;
    expect(evidence, 'the red gate persisted a record').toBeDefined();
    const failing = evidence?.legs.find((leg) => leg.exit !== 0);
    expect(failing, 'a leg records the nonzero exit').toBeDefined();
    expect(failing?.tail, 'the citable failure line survives').toContain('test result: FAILED');

    const record = JSON.stringify(evidence);
    for (const planted of PLANTED) {
      expect(record, `the persisted record keeps ${planted} out`).not.toContain(planted);
    }

    const fixPrompt = promptOfStage(dir, saved, 'fix');
    expect(fixPrompt, 'a fix session ran').toBeDefined();
    for (const planted of PLANTED) {
      expect(fixPrompt, `the fix prompt keeps ${planted} out`).not.toContain(planted);
    }
    expect(fixPrompt).toContain('test result: FAILED');
  });
});

describe('resuming a run without a gate record (ADR-016 Decision 2)', () => {
  // proves: Test strategy #13 — a run resumed from a checkpoint with no
  // `gateEvidence` field completes a stage without prompt breakage: the field is
  // absent, the prompts default it, and the run converges.
  it('runs a stage against a checkpoint that carries no gateEvidence', async () => {
    const dir = tmpGateRepo('green');
    const { client } = stageStub(dir);
    const { status } = await resumePipeline(optionsFor(dir, client));

    const saved = savedState(dir);
    // No pre-existing gateEvidence in the seeded checkpoint.
    expect(status).toBe('done');
    expect(saved.results.some((r) => r.transcript !== undefined)).toBe(true);
  });
});
