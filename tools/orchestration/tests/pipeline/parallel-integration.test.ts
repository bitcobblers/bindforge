import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import { type PipelineConfig } from '../../src/pipeline/stages.js';
import { loadState } from '../../src/pipeline/state-store.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// End-to-end integration (Test strategy #9): a real review pass driven over a
// temp git repo with a mock client at N = 2. Every seat's report and
// transcript lands on disk, results and the seat ledger record each seat
// exactly once, the pass produces ONE commit carrying the whole panel, the
// checkpoint loads with loadState, the gate log has one pass-gate entry, and
// no tmp file survives the drained writer. The same fixture at N = 1 must
// show identical control flow — result order included (fan-out is a
// degenerate case of the same code path, not a branch).
// ---------------------------------------------------------------------------

const DESIGN_SEATS = ['correctness', 'compatibility', 'security', 'performance', 'complexity'];

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

function baseState(): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'escalated',
    lastStage: 'proposal',
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

const designDirOf = (repo: string): string => join(repo, 'docs', 'designs', '002-x');

function tmpRepo(): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-lane-e2e-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  mkdirSync(designDirOf(dir), { recursive: true });
  writeFileSync(join(designDirOf(dir), '02-proposal.md'), '# proposal\n\nbody\n');
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  writeFileSync(
    join(designDirOf(dir), '.pipeline.json'),
    `${JSON.stringify(baseState(), null, 2)}\n`,
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/**
 * Stub whose review seats dissent and write their own report files during the
 * prompt, staggered so completion order is deterministic (seat k stays open
 * for 15 + 20k ms).
 */
function dissentingStub(dir: string) {
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  const created: Record<string, number> = {};
  let clock = 0;
  let revisions = 0;
  const reject = JSON.stringify({
    verdict: 'request-changes',
    findings: [{ severity: 'major', location: 'Decision', problem: 'still broken' }],
  });
  const revise = JSON.stringify({ changed: true, summary: 'closed it' });

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created[key] = (created[key] ?? 0) + 1;
        const id = `ses_${String(Object.values(created).reduce((a, b) => a + b, 0))}`;
        sessions.set(id, { key, ordinal: created[key] - 1, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        const seat = s.key.startsWith('design-review:') ? s.key.split(':')[1] : undefined;
        if (seat !== undefined) {
          const reports = join(designDirOf(dir), 'reviews', 'design');
          mkdirSync(reports, { recursive: true });
          writeFileSync(join(reports, `${seat}-r1c0.md`), `report from ${seat}\n`);
          const ordinal = DESIGN_SEATS.indexOf(seat);
          if (ordinal >= 0) {
            await new Promise((resolve) => setTimeout(resolve, 15 + ordinal * 20));
          }
        }
        if (s.key === 'proposal-revise') {
          revisions += 1;
          writeFileSync(
            join(designDirOf(dir), '02-proposal.md'),
            `# proposal\n\nrevision ${String(revisions)}\n`,
          );
        }
      },
      remove: async () => undefined,
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
            content: [
              {
                type: 'text',
                text: s.key.includes('-review:')
                  ? reject
                  : s.key === 'proposal-revise'
                    ? revise
                    : JSON.stringify({ summary: 'ok' }),
              },
            ],
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

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

async function runPassAt(maxParallelReviews: number): Promise<string> {
  const dir = tmpRepo();
  // The c0 panel (5 sessions) exhausts the design budget: the run escalates
  // right after the pass, so the repo ends with exactly one review pass.
  const cfg: PipelineConfig = { ...FAST, maxReviews: { design: 5, code: 20 }, maxParallelReviews };
  const options: ResumeOptions = {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg,
    client: dissentingStub(dir),
  };
  const status = await resumePipeline(options);
  expect(status).toBe('escalated');
  return dir;
}

describe('end-to-end review pass at N = 2', () => {
  // proves: Test strategy #9 — every report and transcript exists, one commit
  // for the pass, and the checkpoint loads after the drained writer
  it('writes every seat report and transcript and commits the whole pass once', async () => {
    const dir = await runPassAt(2);

    for (const seat of DESIGN_SEATS) {
      expect(existsSync(join(designDirOf(dir), 'reviews', 'design', `${seat}-r1c0.md`))).toBe(true);
      expect(
        existsSync(join(designDirOf(dir), 'transcripts', `design-review-${seat}-r1c0.md`)),
      ).toBe(true);
    }

    const passCommit = 'docs: 002-x design-review stage';
    const messages = gitAt(dir, ['log', '--format=%s']).trim().split('\n');
    expect(messages.filter((s) => s.includes('design-review'))).toEqual([passCommit]);
    const sha = gitAt(dir, ['log', '--format=%H', `--grep=^${passCommit}$`]).trim();
    const files = gitAt(dir, ['show', '--name-only', '--format=', sha]).trim().split('\n');
    for (const seat of DESIGN_SEATS) {
      expect(files).toContain(`docs/designs/002-x/reviews/design/${seat}-r1c0.md`);
    }

    // Writer drained before finalize: no stray tmp file where `git add -A`
    // could have swept it, and the checkpoint is readable by loadState.
    const entries = readdirSync(designDirOf(dir));
    expect(entries.filter((f) => f.includes('.tmp'))).toEqual([]);
    const loaded = await loadState(designDirOf(dir));
    expect(loaded).not.toBeNull();
    expect(loaded?.slug).toBe('x');

    // The checkpoint on disk carries the moment it was TAKEN, not the frozen
    // run-creation stamp (accepted correctness r1 minor 3 / compatibility r1
    // minor 5): the fixture checkpoint holds startedAt = updatedAt = 0, so a
    // save that stamps at the write site must land strictly newer.
    expect((loaded?.updatedAt ?? 0) > (loaded?.startedAt ?? 0)).toBe(true);
  });

  // proves: Test strategy #9 — results and seat verdicts are recorded exactly
  // once per seat, and the gate log carries one pass-gate entry
  it('records each seat once, in results and the seat ledger, one pass-gate entry', async () => {
    const dir = await runPassAt(2);
    const saved = (await loadState(designDirOf(dir))) as PipelineState;

    const seatStages = saved.results.map((r) => r.stage).filter((s) => s.includes('design-review'));
    expect(seatStages.sort()).toEqual(DESIGN_SEATS.map((s) => `design-review:${s}`).sort());
    for (const seat of DESIGN_SEATS) {
      const ledger = saved.seats?.design?.[seat];
      expect(ledger?.verdict).toBe('request-changes');
      expect(ledger?.dissents).toBe(1);
    }

    const passGates = saved.gateLog.filter((g) => g.gate === 'design-review');
    expect(passGates).toHaveLength(1);
    expect(passGates[0]?.proceed).toBe(false);
  });

  // proves: Test strategy #9 — identical control flow to the same fixture at
  // N = 1: same result order (including seat launch order), same commits
  it('matches the N = 1 fixture result-for-result, including result order', async () => {
    const dir1 = await runPassAt(1);
    const dir2 = await runPassAt(2);

    const saved1 = (await loadState(designDirOf(dir1))) as PipelineState;
    const saved2 = (await loadState(designDirOf(dir2))) as PipelineState;

    // At N = 1 the seat stages land in handed order (today's control flow)…
    const stages1 = saved1.results.map((r) => r.stage);
    expect(stages1.filter((s) => s.includes('design-review'))).toEqual(
      DESIGN_SEATS.map((s) => `design-review:${s}`),
    );
    // …and the N = 2 run produces the exact same sequence.
    expect(saved2.results.map((r) => r.stage)).toEqual(stages1);
    expect(saved2.status).toBe(saved1.status);

    expect(gitAt(dir2, ['log', '--format=%s'])).toBe(gitAt(dir1, ['log', '--format=%s']));
  });
});
