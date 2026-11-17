import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type RunOptions, runPipeline } from '../../src/pipeline/runner.js';
import { type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// ADR-023, integration: the task entry path, end to end, with a stub client.
// The dependency gate (023-L4) is the runner's own wiring: a task whose
// depends_on is not done escalates at the source step — before any session
// exists — with the refusal in the gate log. A task whose dependencies are
// done runs the light flow to completion, under the segregated design root,
// with the source flipped pending -> completed (023-L3).
// ---------------------------------------------------------------------------

const DEFAULT_BASE: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

const DEP_TASK = 'T-001-dep';
const DEP_CHECKPOINT_DIR = join('docs', 'designs', 'tasks', '001-dep');

function depCheckpoint(status: string): PipelineState {
  return {
    version: 1,
    slug: 'dep',
    nnn: '001',
    branch: 'pipeline/001-dep',
    source: { kind: 'task', path: `docs/tasks/${DEP_TASK}.md` },
    status,
    lastStage: status === 'done' ? 'doc-sync' : 'implement',
    designRound: 0,
    codeRound: status === 'done' ? 1 : 0,
    designBudgetUsed: 0,
    codeBudgetUsed: status === 'done' ? 1 : 0,
    editorialDone: false,
    testsGreen: status === 'done',
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

/**
 * Scratch repo with the dependency task, its run checkpoint, and the task
 * under test — a light-flow task that depends on the dependency.
 */
function tmpTaskRepo(depStatus: 'running' | 'done'): string {
  const dir = mkTmpDir('bindforge-task-entry-');
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'tasks'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'tasks', `${DEP_TASK}.md`),
    '---\ndoc_type: task\nstatus: pending\ntype: feature\nfunctional_area: backend\n---\n\nBuild the dependency.\n',
  );
  mkdirSync(join(dir, DEP_CHECKPOINT_DIR), { recursive: true });
  writeFileSync(
    join(dir, DEP_CHECKPOINT_DIR, '.pipeline.json'),
    `${JSON.stringify(depCheckpoint(depStatus), null, 2)}\n`,
  );
  writeFileSync(
    join(dir, 'docs', 'tasks', 'T-002-needs-dep.md'),
    [
      '---',
      'doc_type: task',
      'status: pending',
      'type: feature',
      'functional_area: backend',
      `depends_on: [${DEP_TASK}]`,
      'flow: light',
      '---',
      '',
      'Build the feature that needs the dependency.\n',
    ].join('\n'),
  );
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** A light-flow stub: plan writes the plan, implement ships code, reviews approve. */
function lightTaskStub(dir: string) {
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
          mkdirSync(join(dir, 'docs', 'designs', 'tasks', '002-needs-dep'), { recursive: true });
          writeFileSync(
            join(dir, 'docs', 'designs', 'tasks', '002-needs-dep', '02-proposal.md'),
            '# Plan\n\n## Problem\n\nx\n',
          );
        }
        if (s.key === 'implement') {
          mkdirSync(join(dir, 'src'), { recursive: true });
          writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n');
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

function entryOptions(dir: string, extra: object = {}): RunOptions {
  return {
    repoDir: dir,
    sourcePath: join(dir, 'docs', 'tasks', 'T-002-needs-dep.md'),
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: { ...DEFAULT_BASE },
    ...extra,
  } as RunOptions;
}

describe('the task entry path (integration)', () => {
  it('refuses a task whose dependency is not done, before any session exists', async () => {
    const dir = tmpTaskRepo('running');
    const { client, sessionsCreated } = lightTaskStub(dir);

    const { status } = await runPipeline(entryOptions(dir, { client }));
    expect(status).toBe('escalated');
    expect(sessionsCreated).toEqual([]);
    expect(existsSync(join(dir, '.pipeline.lock'))).toBe(false);

    const checkpoint = JSON.parse(
      readFileSync(
        join(dir, 'docs', 'designs', 'tasks', '002-needs-dep', '.pipeline.json'),
        'utf8',
      ),
    ) as PipelineState;
    expect(checkpoint.status).toBe('escalated');
    expect(checkpoint.source).toEqual({
      kind: 'task',
      path: 'docs/tasks/T-002-needs-dep.md',
    });
    const depGate = checkpoint.gateLog.find((entry) => entry.gate === 'taskDependencies');
    expect(depGate).toBeDefined();
    expect(depGate?.proceed).toBe(false);
    expect(depGate?.reason).toMatch(/not done \(status: running\)/);
  });

  it('runs a task with satisfied dependencies to done, under the task root', async () => {
    const dir = tmpTaskRepo('done');
    const { client, sessionsCreated } = lightTaskStub(dir);

    const { status } = await runPipeline(entryOptions(dir, { client }));
    expect(status).toBe('done');
    expect(sessionsCreated[0]).toBe('plan');

    // The run's artifacts sit under the segregated root, not the ADR root.
    const designDir = join(dir, 'docs', 'designs', 'tasks', '002-needs-dep');
    expect(existsSync(join(designDir, '.pipeline.json'))).toBe(true);
    expect(existsSync(join(dir, 'docs', 'designs', '002-needs-dep'))).toBe(false);

    // The source flip: pending -> completed, runner-written (023-L3).
    const source = readFileSync(join(dir, 'docs', 'tasks', 'T-002-needs-dep.md'), 'utf8');
    expect(source).toContain('status: completed');
    expect(source).not.toMatch(/^status:[ \t]*pending[ \t]*$/m);

    const checkpoint = JSON.parse(
      readFileSync(join(designDir, '.pipeline.json'), 'utf8'),
    ) as PipelineState;
    const depGate = checkpoint.gateLog.find((entry) => entry.gate === 'taskDependencies');
    expect(depGate?.proceed).toBe(true);
    expect(depGate?.reason).toBe('1 dependency(ies) satisfied');
  });
});
