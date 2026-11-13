import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it, vi } from 'vitest';

import { exitCodeFor } from '../../src/pipeline/cli.js';
import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// ADR-020 §3 (design 020 Decision 3): the exit-code table — `done → 0`, a run
// ending with the restart record standing `→ 3`, any other non-done `→ 2`,
// thrown error `→ 1` — is carried to the CLI by widening the run/resume seam
// from a bare `RunStatus` to `{ status, restartRequired? }`. These tests pin
// that seam end of the seam: what `cli.ts` maps its integers from, so `2`
// keeps its exact "read ESCALATION.md" meaning and a pause can never collide
// with an escalation. The light fixture drives the smallest run that owns a
// coder window (plan → implement → code-review).
// ---------------------------------------------------------------------------

interface RestartRecord {
  stage: string;
  paths: string[];
  at: number;
}
/** The widened run/resume result the CLI maps exit codes from (§3). */
interface DriveResult {
  status: 'running' | 'done' | 'escalated';
  restartRequired?: RestartRecord;
}

/** Dynamic on purpose: only the thrown row of §3's table needs the entry
 * point, and a named static import of a CLI that renames it would fail this
 * file at load, before any behavior is judged. */
const pipelineCli = (await import('../../src/pipeline/cli.js')) as {
  main?: (argv: string[]) => unknown;
};

const BRIEF = 'docs/prompts/002-x.md';
const DESIGN_DIR = 'docs/designs/002-x';

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

/** Repo with a light brief, a committed checkpoint, and a `check` gate whose
 * exit code the fixture controls (green by default, red for the 2-case). */
function tmpRepo(checkExitCode: 0 | 1 = 0): string {
  const dir = mkTmpDir('bindforge-cli-seam-');
  gitAt(dir, ['init', '-q', '-b', 'main']);
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
  gitAt(dir, ['add', '-A']);
  gitAt(dir, ['commit', '-q', '-m', 'docs: base']);
  return dir;
}

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

/** Light stub: plan writes the plan, implement writes the module, build notes
 * and — for the pause case — a harness template file; seats approve. */
function lightStub(dir: string, implementWritesHarness: boolean) {
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  const created: Record<string, number> = {};
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  let clock = 0;

  const replyFor = (key: string): string => {
    if (key.includes('-review:')) return approve;
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
        created[key] = (created[key] ?? 0) + 1;
        const id = `ses_${String(Object.values(created).reduce((a, b) => a + b, 0))}`;
        sessions.set(id, { key, ordinal: created[key] - 1, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string; text?: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        if (s.key === 'plan') {
          writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), `${planDoc()}\n`, 'utf8');
        }
        if (s.key === 'implement') {
          mkdirSync(join(dir, 'src'), { recursive: true });
          writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n', 'utf8');
          writeFileSync(join(dir, DESIGN_DIR, '03-build-notes.md'), '# Build notes\n\nshipped\n');
          if (implementWritesHarness) {
            // A self-change run: this window adds a stage template.
            mkdirSync(join(dir, 'tools', 'orchestration', 'prompts'), { recursive: true });
            writeFileSync(
              join(dir, 'tools', 'orchestration', 'prompts', 'implement.md'),
              '# implement\n\ncarry {{gateEvidence}} into the brief\n',
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
            content: [{ type: 'text', text: replyFor(s.key) }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(clock)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return client;
}

function optionsFor(dir: string, client: OpenCodeClient): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: FAST,
    client,
  };
}

describe('the run/resume exit-code seam', () => {
  // proves: Test strategy #6 — `done → 0`: a run that touches no harness path
  // finishes as a plain done result with no restartRequired on it; the pause
  // integer must never ride a finished run.
  it('returns a done result with no restart record for an untouched run', async () => {
    const dir = tmpRepo();
    const result = (await resumePipeline(
      optionsFor(dir, lightStub(dir, false)),
    )) as unknown as DriveResult;
    expect(result.status).toBe('done');
    expect(result.restartRequired).toBeUndefined();
  });

  // proves: Test strategy #6 — a run ending with the record standing `→ 3`:
  // the result carries the restart record, and `status` itself stays
  // `running` (§2 — the pause never enters the status enum, so the old
  // non-done `→ 2` mapping cannot claim it).
  it('returns the standing restart record when a window touches the surface', async () => {
    const dir = tmpRepo();
    const result = (await resumePipeline(
      optionsFor(dir, lightStub(dir, true)),
    )) as unknown as DriveResult;
    expect(result.status).toBe('running');
    expect(result.restartRequired).toBeDefined();
    expect(result.restartRequired?.stage).toBe('implement');
    expect(result.restartRequired?.paths).toContain('tools/orchestration/prompts/implement.md');
  });

  // proves: Test strategy #6 — an escalated run `→ 2`: the code that today
  // means "read ESCALATION.md" keeps its exact meaning; a stopped-with-no-
  // boundary run carries no record to confuse a driver.
  it('returns an escalated result with no restart record on a red check gate', async () => {
    const dir = tmpRepo(1);
    const result = (await resumePipeline(
      optionsFor(dir, lightStub(dir, false)),
    )) as unknown as DriveResult;
    expect(result.status).toBe('escalated');
    expect(result.restartRequired).toBeUndefined();
  });

  // proves: Test strategy #6 — a thrown error `→ 1`: an unreadable checkpoint
  // escapes the seam as a rejection (the CLI's 1), not as a status a driver
  // could map to a stop posture.
  it('rejects when the run cannot start at all', async () => {
    const dir = tmpRepo();
    writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), '{"version":1,"slug":"fea', 'utf8');
    await expect(resumePipeline(optionsFor(dir, lightStub(dir, false)))).rejects.toThrow();
  });
});

// ---------------------------------------------------------------------------
// The integers themselves — compatibility r1 #1, testing r1 #1. §3's table is
// the driver-facing contract and `exitCodeFor` is the whole of it: the seam
// tests above pin what the run/resume result carries, the pins below pin the
// four integers the documented driver loop
// (`pipeline run …; while [ $? -eq 3 ]; do pipeline resume …; done`) turns on,
// so a swap of 3 and 2, or a pause that exits 0, cannot keep the suite green
// while every driver automates past an escalation.
// ---------------------------------------------------------------------------

describe('the exit-code table', () => {
  const record: RestartRecord = {
    stage: 'implement',
    paths: ['tools/orchestration/prompts/implement.md'],
    at: 3,
  };

  // proves: Test strategy #6 — `done → 0`: the record arm is consulted only
  // after `done`, so the pause integer never rides a finished run.
  it('answers a done result with 0, record beside it or not', () => {
    expect(exitCodeFor({ status: 'done' })).toBe(0);
    expect(exitCodeFor({ status: 'done', restartRequired: record })).toBe(0);
  });

  // proves: Test strategy #6 — a run ending with the record standing `→ 3`:
  // the code the driver loop resumes on.
  it('answers a standing restart record with 3', () => {
    expect(exitCodeFor({ status: 'running', restartRequired: record })).toBe(3);
  });

  // proves: Test strategy #6 — any other non-done `→ 2`: a stopped run with no
  // boundary is not a pause, so 3 stays reserved for the restart posture.
  it('answers a running result with no record with 2', () => {
    expect(exitCodeFor({ status: 'running' })).toBe(2);
  });

  // proves: Test strategy #6 (security r1 #1, compatibility r1 #2) — an
  // escalated run `→ 2` with a record beside it or without one: `2` keeps its
  // exact "read ESCALATION.md" meaning, so no existing driver automates past an
  // escalation by code collision.
  it('answers an escalated result with 2, record beside it or not', () => {
    expect(exitCodeFor({ status: 'escalated' })).toBe(2);
    expect(exitCodeFor({ status: 'escalated', restartRequired: record })).toBe(2);
  });

  // proves: Test strategy #6 — a thrown error `→ 1`: an invocation whose run
  // cannot start at all ends on the error integer, never on a status posture a
  // driver could loop on. The integer is read from whichever channel the CLI
  // answers on — a returned code, a `process.exit` call, or `process.exitCode`.
  // The argv shape follows the sibling CLI's `main(process.argv)` convention;
  // under either convention this invocation cannot open a run.
  it('answers an invocation whose run cannot start with 1', async () => {
    const main = pipelineCli.main;
    if (typeof main !== 'function') {
      throw new Error('src/pipeline/cli.ts exports no main entry point yet');
    }
    // Nothing to resume onto: an empty directory, no checkpoint, no git repo.
    const dir = mkTmpDir('bindforge-cli-error-');
    const cwdBefore = process.cwd();
    const exitCodeBefore = process.exitCode;
    const exitCalls: number[] = [];
    const exitMock = (...args: unknown[]): never => {
      const [code] = args;
      if (typeof code === 'number') {
        exitCalls.push(code);
      }
      return undefined as never;
    };
    const exitSpy = vi.spyOn(process, 'exit').mockImplementation(exitMock);
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => undefined);
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => undefined);
    process.exitCode = 0;
    let returned: unknown;
    let escaped = false;
    try {
      process.chdir(dir);
      returned = await main(['node', 'pipeline', 'resume', '002-x']);
    } catch {
      escaped = true;
    } finally {
      process.chdir(cwdBefore);
      exitSpy.mockRestore();
      logSpy.mockRestore();
      errorSpy.mockRestore();
    }
    let observed = exitCalls[0];
    if (observed === undefined && typeof returned === 'number') {
      observed = returned;
    }
    if (observed === undefined && typeof process.exitCode === 'number') {
      observed = process.exitCode;
    }
    process.exitCode = exitCodeBefore;

    // One assertion, whichever channel the CLI answers on: a run that could
    // not start is the error integer, never 0 (a driver that stops cleanly)
    // and never a status code it could loop on.
    expect(
      escaped ? undefined : observed,
      'the thrown-error row of the table is exit 1: a failed run answers with the error integer',
    ).toBe(1);
  });
});
