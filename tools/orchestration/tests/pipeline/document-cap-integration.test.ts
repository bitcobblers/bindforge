import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { FLOWS } from '../../src/pipeline/flows.js';
import { type RunOptions, runPipeline } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import { DEFAULT_CONFIG } from '../../src/pipeline/stages.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// ADR-016 Decision 4 end to end: `01-research.md` joins the metered document
// stages, so the full flow's `docMaxLines` gate now runs against research too.
// Over the cap the research attempt becomes a normal stage failure — the retry
// (same-session reply or attempt 2) instructs the shrink and the existing
// one-retry-then-escalate budget terminates it — with the cap named in the gate
// log. Under the cap the stage passes and the run moves on. This is not the
// plan stop: an over-length full-flow document is shrinkable by the next revise,
// so it takes the repair loop, not a terminal stop.
// ---------------------------------------------------------------------------

const SOURCE = 'docs/adrs/016-oversized.md';

// The cap the fixture drives against: the full flow's registry value, the same
// one `flows.test.ts` and `document-cap.test.ts` pin at the registry seam. A
// run's cap resolves from `FLOWS.full.config` (the registry leads a run's own
// config for registry-pinned keys, as the light registry's 80 leads in
// `light-plan-size`), so the `cfg` below carries it without ever overriding it.
const CAP = 900;

const FULL: PipelineConfig = {
  ...DEFAULT_CONFIG,
  maxRounds: 1,
  maxPassesPerRound: 1,
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
  docMaxLines: CAP,
};

function tmpRepo(): string {
  const dir = mkTmpDir('bindforge-doc-cap-e2e-');
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, SOURCE),
    [
      '---',
      'status: proposed',
      '---',
      '# ADR 016: oversized',
      '## Context',
      'why.',
      '## Decision',
      'what.',
    ].join('\n'),
    'utf8',
  );
  // A check script that stays green, so the run's only failure signal is the
  // document cap, never the gate.
  writeFileSync(
    join(dir, 'package.json'),
    JSON.stringify({ name: 't', scripts: { check: 'node -e "process.exit(0)"' } }),
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** A `research` session that writes `01-research.md` at `researchLines`, then
 * replies with the research shape. The design directory is resolved when the
 * stage prompts, not when the stub is built — the runner creates it at run
 * start, so an eager resolution would land the document at the repo root. */
function researchStub(
  dir: string,
  researchLines: number,
): { client: OpenCodeClient; sessionsCreated: string[] } {
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
        if (s.key === 'research') {
          const lines: string[] = [];
          for (let i = 0; i < researchLines; i += 1) {
            lines.push(`research line ${String(i + 1)}`);
          }
          const rel = designDirRel(dir);
          if (rel !== '') {
            writeFileSync(join(dir, rel, '01-research.md'), `${lines.join('\n')}\n`, 'utf8');
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
  return { client, sessionsCreated };
}

function designDirRel(dir: string): string {
  const designs = join(dir, 'docs', 'designs');
  if (!existsSync(designs)) return '';
  const entry = readdirSync(designs)[0];
  return entry === undefined ? '' : join('docs', 'designs', entry);
}

function optionsFor(dir: string, client: OpenCodeClient): RunOptions {
  return {
    repoDir: dir,
    sourcePath: SOURCE,
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: FULL,
    client,
  };
}

function gateLogReasons(dir: string, rel: string): string {
  const file = join(dir, rel, '.pipeline.json');
  if (!existsSync(file)) return '';
  const state = JSON.parse(readFileSync(file, 'utf8')) as { gateLog?: { reason?: string }[] };
  return (state.gateLog ?? []).map((g) => g.reason ?? '').join('\n');
}

describe('the oversized research document (ADR-016 Decision 4)', () => {
  // proves: Test strategy #12 — over the cap the research attempt fails with the
  // cap named, retries once, and escalates with the cap in the gate log.
  it('fails, retries and escalates a research document over the cap', async () => {
    const dir = tmpRepo();
    const { client } = researchStub(dir, CAP + 100);

    const { status } = await runPipeline(optionsFor(dir, client));

    const rel = designDirRel(dir);
    expect(status, 'an over-cap research stage must not finish').toBe('escalated');
    expect(existsSync(join(dir, rel, 'ESCALATION.md'))).toBe(true);
    const reasons = gateLogReasons(dir, rel);
    expect(reasons).toContain('document outgrew');
    expect(reasons).toContain(String(CAP));
  });

  // proves: Test strategy #12 — under the cap the same stage passes and the run
  // moves past research to the next document stage.
  it('passes a research document under the cap', async () => {
    const dir = tmpRepo();
    const { client, sessionsCreated } = researchStub(dir, 300);

    await runPipeline(optionsFor(dir, client));

    expect(sessionsCreated, 'the run moves past research to proposal').toContain('proposal');
  });

  // proves: Test strategy #12 — the cap is registry config, not code: raising
  // `docMaxLines` lets the very same over-900 fixture pass research and move on.
  // The run seam's cap is the flow registry's (the shipped `FLOWS.full.config`
  // 900 governs the run above, exactly as the light registry's 80 governs
  // `light-plan-size`), so this raises it the way the operator's repair does —
  // edit the registry value — and restores it after.
  it('passes the same fixture once the cap is raised', async () => {
    const dir = tmpRepo();
    const { client, sessionsCreated } = researchStub(dir, CAP + 100);

    const shipped = FLOWS.full.config;
    FLOWS.full.config = { ...shipped, docMaxLines: CAP + 200 };
    try {
      await runPipeline(optionsFor(dir, client));
    } finally {
      FLOWS.full.config = shipped;
    }

    expect(sessionsCreated, 'with the raised cap the run moves past research').toContain(
      'proposal',
    );
  });
});
