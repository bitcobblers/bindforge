import { execFileSync } from 'node:child_process';
import { mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type RunOptions, runPipeline } from '../../src/pipeline/runner.js';
import { DEFAULT_CONFIG } from '../../src/pipeline/stages.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// The pipeline binds every stage session to the model its role declares: the
// run tree's `.opencode/agents/<id>.md` `role:` frontmatter joins through the
// run tree's `.opencode/roles.json`, and the parsed model — `#variant` suffix
// included — rides the session.create body. Before this contract, the runner
// created sessions with `{title, agent, location}` alone and roles.json — the
// reasoning levels the operator assigned — reached no pipeline session at all.
// An agent whose role the map omits keeps the server default (no `model` key),
// which is also the guard against a slash-bearing agent id being fabricated
// into a provider/model pair by the direct-string path.
// ---------------------------------------------------------------------------

const SOURCE = 'docs/adrs/001-roles.md';

const CFG = {
  ...DEFAULT_CONFIG,
  maxRounds: 1,
  maxPassesPerRound: 1,
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

type Created = { key: string; model?: unknown };

function tmpRepo(rolesMap: Record<string, string>): string {
  const dir = mkTmpDir('bindforge-role-model-');
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, SOURCE),
    [
      '---',
      'status: proposed',
      '---',
      '# ADR 001: roles',
      '## Context',
      'why.',
      '## Decision',
      'what.',
    ].join('\n'),
    'utf8',
  );
  mkdirSync(join(dir, '.opencode', 'agents', 'research'), { recursive: true });
  writeFileSync(
    join(dir, '.opencode', 'agents', 'research', 'researcher.md'),
    '---\nrole: researcher\n---\n# researcher\n',
    'utf8',
  );
  writeFileSync(join(dir, '.opencode', 'roles.json'), JSON.stringify({ roles: rolesMap }), 'utf8');
  writeFileSync(
    join(dir, 'package.json'),
    JSON.stringify({ name: 't', scripts: { check: 'node -e "process.exit(0)"' } }),
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** The research stage writes a three-line document and replies with the
 * research shape; every other session replies, reviews approve. The create
 * body's `model` — the thing under test — is captured per session. */
function stub(dir: string): { client: OpenCodeClient; created: Created[] } {
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  const created: Created[] = [];
  let clock = 0;
  let counter = 0;
  const client = {
    session: {
      create: async (input: { title?: string; model?: unknown }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created.push(input.model === undefined ? { key } : { key, model: input.model });
        const id = `ses_${String(++counter)}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        if (s.key === 'research') {
          const designs = join(dir, 'docs', 'designs');
          const entry = readdirSync(designs)[0] as string | undefined;
          if (entry !== undefined) {
            writeFileSync(join(designs, entry, '01-research.md'), 'a\nb\nc\n', 'utf8');
          }
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
          : '{"summary": "researched", "commitMessage": "docs: research"}';
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
  return { client, created };
}

function optionsFor(dir: string, client: OpenCodeClient): RunOptions {
  return {
    repoDir: dir,
    sourcePath: SOURCE,
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: CFG,
    client,
  };
}

describe('role-model binding at pipeline session create', () => {
  // proves: the declared role model reaches the create body — provider, id,
  // and the `#variant` reasoning level — for a stage session.
  it('binds the role model with its variant to the research session', async () => {
    const dir = tmpRepo({ researcher: 'basalt/test-model#medium' });
    const { client, created } = stub(dir);

    await runPipeline(optionsFor(dir, client));

    const research = created.find((c) => c.key === 'research');
    expect(research, 'the run created a research session').toBeDefined();
    expect(research?.model).toEqual({ providerID: 'basalt', id: 'test-model', variant: 'medium' });
  });

  // proves: an agent whose declared role the roles map omits is created
  // without a `model` key at all — the server default, never a model
  // fabricated from the agent id's own slash.
  it('omits the model for a role the map does not carry', async () => {
    const dir = tmpRepo({ planner: 'basalt/other-model#low' });
    const { client, created } = stub(dir);

    await runPipeline(optionsFor(dir, client));

    const research = created.find((c) => c.key === 'research');
    expect(research, 'the run created a research session').toBeDefined();
    expect(research?.model).toBeUndefined();
  });
});
