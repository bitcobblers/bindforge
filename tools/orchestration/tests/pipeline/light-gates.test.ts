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
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import {
  type ResumeOptions,
  type RunOptions,
  resumePipeline,
  runPipeline,
} from '../../src/pipeline/runner.js';
import { DESIGN_SPECIALTIES, type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// The pre-lock posture (Selecting the flow): an unreadable `flow` value, a
// `--stage` the flow does not declare, and a caller-supplied empty seat panel
// are usage errors. A usage error never half-starts a run: no lock file, no
// design directory, no entry checkpoint, no ESCALATION.md, and zero client
// calls. A half-started run leaves an escalated checkpoint the operator has to
// reason about, which is exactly what a typo should not cost.
// ---------------------------------------------------------------------------

const DESIGN_DIR = 'docs/designs/002-x';

function lightState(partial: Partial<PipelineState> = {}): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'brief', path: 'docs/prompts/002-x.md' },
    status: 'escalated',
    lastStage: 'code-review',
    flow: 'light',
    designRound: 1,
    codeRound: 1,
    designBudgetUsed: 0,
    codeBudgetUsed: 2,
    editorialDone: false,
    testsGreen: true,
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

/** Repo with briefs in `docs/prompts/`, an optional committed light checkpoint,
 * and NO design directory of its own: an entry that works creates one. */
function tmpRepo(opts: { checkpoint?: PipelineState; briefs?: Record<string, string> }): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-light-gates-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
  mkdirSync(join(dir, 'docs', 'designs'), { recursive: true });
  for (const [name, text] of Object.entries(opts.briefs ?? {})) {
    writeFileSync(join(dir, 'docs', 'prompts', name), text, 'utf8');
  }
  if (opts.checkpoint !== undefined) {
    mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
    const checkpoint = `${JSON.stringify(opts.checkpoint, null, 2)}\n`;
    writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), checkpoint);
  }
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** Client that records every call: a clean usage-error exit calls it zero times. */
function countingClient() {
  const calls: string[] = [];
  const client = {
    session: {
      create: async (input: { title?: string }) => {
        calls.push(`create:${input.title ?? ''}`);
        return { id: `ses_${String(calls.length)}` };
      },
      prompt: async () => undefined,
      remove: async () => undefined,
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async () => {
        calls.push('message:list');
        return { data: [], cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, calls };
}

function entryOptions(dir: string, sourcePath: string, extra: object = {}): RunOptions {
  return {
    repoDir: dir,
    sourcePath,
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: { ...DEFAULT_BASE },
    ...extra,
  } as RunOptions;
}

function resumeOptions(dir: string, extra: object = {}): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: { ...DEFAULT_BASE },
    ...extra,
  } as ResumeOptions;
}

const DEFAULT_BASE: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

async function failureOf(promise: Promise<unknown>): Promise<string> {
  try {
    await promise;
    return '';
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
}

function designDirs(dir: string): string[] {
  return readdirSync(join(dir, 'docs', 'designs'));
}

describe('pre-lock gates leave nothing behind', () => {
  // proves: Test strategy #17 — an unreadable `flow` value is a usage error that
  // names the value, the source and the accepted set.
  it('refuses a flow value the registry does not define', async () => {
    const dir = tmpRepo({ briefs: { 'sideways.md': '---\nflow: sideways\n---\n\nDo it.\n' } });
    const { client, calls } = countingClient();

    const source = 'docs/prompts/sideways.md';
    const run = runPipeline(entryOptions(dir, source, { client }));
    const message = await failureOf(run);
    expect(message).toContain('invalid flow "sideways"');
    expect(message).toContain('docs/prompts/sideways.md');
    expect(message).toContain('expected full or light');

    // The run never started: nothing to reason about afterwards.
    expect(designDirs(dir)).toEqual([]);
    expect(existsSync(join(dir, '.pipeline.lock'))).toBe(false);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
    expect(calls).toEqual([]);
  });

  // proves: Test strategy #17 (Selecting the flow, A18) — the check is
  // pre-lock, ahead of the `sourceAdmissible` gate: an admissible brief with a
  // case-variant value is the same clean refusal, never a silent full run.
  it('refuses a case-variant flow value with the same usage error', async () => {
    const dir = tmpRepo({ briefs: { 'capital.md': '---\nflow: Light\n---\n\nDo it.\n' } });
    const { client, calls } = countingClient();

    const source = 'docs/prompts/capital.md';
    const message = await failureOf(runPipeline(entryOptions(dir, source, { client })));
    expect(message).toContain('invalid flow');
    expect(message).toContain('Light');
    expect(designDirs(dir)).toEqual([]);
    expect(calls).toEqual([]);
  });

  // proves: Test strategy #17 — `--stage` is checked against the flow at the
  // resume's validation points, so a light run never gains a research artifact.
  it('refuses a stage the checkpoint flow does not declare', async () => {
    const dir = tmpRepo({ checkpoint: lightState() });
    const { client, calls } = countingClient();

    const resume = resumePipeline(resumeOptions(dir, { client, forceStage: 'research' }));
    const message = await failureOf(resume);
    expect(message).toContain('research');
    expect(message).toContain('light');

    expect(existsSync(join(dir, DESIGN_DIR, '01-research.md'))).toBe(false);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
    expect(existsSync(join(dir, '.pipeline.lock'))).toBe(false);
    expect(calls).toEqual([]);

    // The checkpoint is untouched: a usage error does not write a terminal record.
    const saved = JSON.parse(
      readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8'),
    ) as PipelineState;
    expect(saved.status).toBe('escalated');
    expect(saved.flow).toBe('light');
    expect(saved.gateLog).toEqual([]);
  });

  // proves: Test strategy #17 (Budget resolution, security r1 closure) — a
  // caller-supplied panel is validated pre-lock, not at the post-lock config
  // read: an empty code panel is the same clean usage error, on the entry path.
  it('refuses an empty caller-supplied code panel before the lock', async () => {
    const dir = tmpRepo({ briefs: { '002-x.md': '---\nflow: light\n---\n\nDo it.\n' } });
    const { client, calls } = countingClient();
    const cfg: PipelineConfig = {
      ...DEFAULT_BASE,
      reviewSeats: { design: [...DESIGN_SPECIALTIES], code: [] },
    };

    const source = 'docs/prompts/002-x.md';
    const message = await failureOf(runPipeline(entryOptions(dir, source, { client, cfg })));
    expect(message).toContain('reviewSeats');

    expect(designDirs(dir)).toEqual([]);
    expect(existsSync(join(dir, '.pipeline.lock'))).toBe(false);
    expect(existsSync(join(dir, DESIGN_DIR, '.pipeline.json'))).toBe(false);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
    expect(calls).toEqual([]);
  });

  // proves: Test strategy #17 — the same panel is refused on the resume path,
  // and the run it refuses to start keeps its checkpoint exactly as it was.
  it('refuses an empty caller-supplied panel on a resume', async () => {
    const dir = tmpRepo({ checkpoint: lightState() });
    const { client, calls } = countingClient();
    const cfg: PipelineConfig = {
      ...DEFAULT_BASE,
      reviewSeats: { design: [...DESIGN_SPECIALTIES], code: [] },
    };

    const message = await failureOf(resumePipeline(resumeOptions(dir, { client, cfg })));
    expect(message).toContain('reviewSeats');

    expect(existsSync(join(dir, '.pipeline.lock'))).toBe(false);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
    expect(calls).toEqual([]);
    const saved = JSON.parse(
      readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8'),
    ) as PipelineState;
    expect(saved.codeBudgetUsed).toBe(2);
    expect(saved.gateLog).toEqual([]);
  });
});

// ---------------------------------------------------------------------------
// `FlowDef.stages` is the `--stage` gate (Selecting the flow), and the flow
// check is the only gate it needs: `plan` is in `FLOWS.light.stages` and in no
// full-flow path, so a light run can be forced into the light flow's first
// stage — the stage a plan stop ended — while a full run keeps the usage error.
// ---------------------------------------------------------------------------

/** A light run whose every stage answers its own reply shape: a forced `plan`
 *  runs the plan and the run goes on to the 2-seat panel and ends `done`. */
function planStub(dir: string, designDir: string) {
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
        if (s.key === 'plan') {
          mkdirSync(join(dir, designDir), { recursive: true });
          writeFileSync(join(dir, designDir, '02-proposal.md'), '# Plan\n\n## Problem\n\nx\n');
        }
        if (s.key === 'implement') {
          mkdirSync(join(dir, 'src'), { recursive: true });
          writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n');
          writeFileSync(join(dir, designDir, '03-build-notes.md'), '# Notes\n\nshipped\n');
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

describe('the flow registry is the --stage gate', () => {
  // proves: Test strategy #17 (correctness r1 finding 5, deviation 21) — the
  // light flow's own first stage is forceable on the entry path: the registry
  // check accepts it, and the run runs the plan and goes on.
  it('accepts the light plan stage on the entry path', async () => {
    const dir = tmpRepo({ briefs: { 're-plan.md': '---\nflow: light\n---\n\nDo it.\n' } });
    const designDir = join('docs', 'designs', '001-re-plan');
    const { client, sessionsCreated } = planStub(dir, designDir);

    const source = 'docs/prompts/re-plan.md';
    const status = await runPipeline(entryOptions(dir, source, { client, forceStage: 'plan' }));
    expect(status).toBe('done');

    expect(sessionsCreated[0]).toBe('plan');
    expect(existsSync(join(dir, designDir, '02-proposal.md'))).toBe(true);
  });

  // proves: Test strategy #17 (correctness r1 finding 5) — the same stage is
  // forceable on a resume: the run a plan stop ended is re-planned from its own
  // checkpoint, not refused as an unknown stage.
  it('accepts the light plan stage on a resume', async () => {
    const dir = tmpRepo({
      checkpoint: lightState({ lastStage: '' }),
      briefs: { '002-x.md': '---\nflow: light\n---\n\nDo it.\n' },
    });
    const { client, sessionsCreated } = planStub(dir, DESIGN_DIR);

    const resume = resumePipeline(resumeOptions(dir, { client, forceStage: 'plan' }));
    expect(await resume).toBe('done');
    expect(sessionsCreated[0]).toBe('plan');
  });

  // proves: Test strategy #17 (deviation 21) — the flow check is what keeps
  // `plan` out of a full run: a full checkpoint resumed with it is the same
  // pre-lock usage error, with nothing written and no session spent.
  it('refuses plan on a full checkpoint', async () => {
    const dir = tmpRepo({ checkpoint: lightState({ flow: 'full' }) });
    const { client, calls } = countingClient();

    const resume = resumePipeline(resumeOptions(dir, { client, forceStage: 'plan' }));
    const message = await failureOf(resume);
    expect(message).toContain('plan');
    expect(message).toContain('full');

    expect(existsSync(join(dir, '.pipeline.lock'))).toBe(false);
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
    expect(calls).toEqual([]);
  });
});
