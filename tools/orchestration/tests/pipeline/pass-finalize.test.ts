import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import { type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// Pass-level finalize (ADR-003 / ADR-002 ownership rule): a review pass runs
// ONE enforceOwnership + ONE commitStage for the whole pass — never per seat.
// `commitStage`'s `git add -A` would otherwise sweep a sibling's half-written
// file into the wrong commit, and the ownership sweep would revert siblings'
// in-flight edits. The pass commit carries the pass's stage key, so its
// message has the fallback shape with one value: `docs: <nnn-slug> design-
// review stage` (review replies carry no commitMessage field). N = 1 differs
// from the pre-change behaviour exactly here, and that is the point.
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

function baseState(partial: Partial<PipelineState> = {}): PipelineState {
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
    ...partial,
  };
}

/** Temp git repo; an untracked out-of-policy file probes the ownership sweep. */
function tmpRepo(state: PipelineState): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-finalize-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'designs', '002-x', '02-proposal.md'), '# proposal\n\nbody\n');
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  writeFileSync(
    join(dir, 'docs', 'designs', '002-x', '.pipeline.json'),
    `${JSON.stringify(state, null, 2)}\n`,
  );
  mkdirSync(join(dir, 'tools'), { recursive: true });
  writeFileSync(join(dir, 'tools', 'stray.ts'), 'export const stray = 1;\n');
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A', '--', 'docs', 'package.json']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** Stub whose review sessions write their own report files, as real seats do. */
function finalizeStub(dir: string, opts: { allDissent: boolean }) {
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  const created: Record<string, number> = {};
  let clock = 0;
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const reject = JSON.stringify({
    verdict: 'request-changes',
    findings: [{ severity: 'major', location: 'Decision', problem: 'still broken' }],
  });

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
          const reports = join(dir, 'docs', 'designs', '002-x', 'reviews', 'design');
          mkdirSync(reports, { recursive: true });
          writeFileSync(join(reports, `${seat}-r1c0.md`), `report from ${seat}\n`);
        }
      },
      remove: async () => undefined,
      // Test strategy #9: the descendant walk's enumeration endpoint. This
      // fixture creates no child sessions, so an empty page is the truth.
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

  function replyFor(key: string): string {
    if (key.endsWith('-review')) return 'impossible'; // stage keys here always carry :specialty
    if (key.includes('-review:')) return opts.allDissent ? reject : approve;
    if (key === 'proposal-revise') return JSON.stringify({ changed: true, summary: 'closed it' });
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 0, commitMessage: 'test: pin the contract' });
    if (key === 'implement')
      return JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok' });
  }

  return client;
}

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

function subjects(dir: string): string[] {
  return gitAt(dir, ['log', '--format=%s']).trim().split('\n');
}

/** File list of the single commit whose subject equals `subject`. */
function committedFiles(dir: string, subject: string): string[] {
  const sha = gitAt(dir, ['log', '--format=%H', `--grep=^${subject}$`]).trim();
  expect(sha).not.toBe('');
  return gitAt(dir, ['show', '--name-only', '--format=', sha]).trim().split('\n');
}

async function runPass(maxParallelReviews: number, allDissent: boolean): Promise<string> {
  const cfg: PipelineConfig = allDissent
    ? { ...FAST, maxReviews: { design: 5, code: 20 }, maxParallelReviews }
    : { ...FAST, maxParallelReviews };
  const dir = tmpRepo(baseState());
  const options: ResumeOptions = {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg,
    client: finalizeStub(dir, { allDissent }),
  };
  await resumePipeline(options);
  return dir;
}

describe('one commit and one ownership sweep per review pass', () => {
  // proves: Test strategy #6 — exactly one pass commit (and one ownership
  // sweep, whose effect is visible: the out-of-policy stray never reaches a
  // commit and the tree ends clean) at N = 1 and at N = 3, with the exact
  // fallback message `docs: <nnn-slug> design-review stage`
  for (const n of [1, 3]) {
    it(`makes exactly one design-review pass commit at N = ${n}`, async () => {
      const dir = await runPass(n, true);

      const reviewSubjects = subjects(dir).filter((s) => s.includes('design-review'));
      expect(reviewSubjects).toEqual(['docs: 002-x design-review stage']);

      // The one pass commit carries every seat's complete report and the
      // runner's transcript — the pass-level sweep, not five per-seat ones.
      const files = committedFiles(dir, 'docs: 002-x design-review stage');
      for (const seat of DESIGN_SEATS) {
        expect(files).toContain(`docs/designs/002-x/reviews/design/${seat}-r1c0.md`);
      }
      expect(files).toContain('docs/designs/002-x/transcripts/design-review-correctness-r1c0.md');

      // The ownership sweep ran before the commit: the untracked out-of-policy
      // file was reverted, not swept into the pass commit, and nothing dirty
      // survives the pass.
      expect(files).not.toContain('tools/stray.ts');
      expect(gitAt(dir, ['status', '--porcelain'])).toBe('');
    });
  }

  // proves: Test strategy #6 — the editorial pass (kind: 'review') is in
  // scope too: one commit for the whole pass, and its reports/transcripts
  // keep the `-rN` (no pass number) naming
  it('makes exactly one editorial-review pass commit at N = 3 and keeps -rN naming', async () => {
    const dir = await runPass(3, false);

    const editorialSubjects = subjects(dir).filter((s) => s.includes('editorial-review'));
    expect(editorialSubjects).toEqual(['docs: 002-x editorial-review stage']);

    // One commit carries the WHOLE editorial pass: every seat's transcript is
    // in the single editorial commit — no per-seat editorial commits.
    const editorialFiles = committedFiles(dir, 'docs: 002-x editorial-review stage');
    for (const seat of DESIGN_SEATS) {
      expect(editorialFiles.some((f) => f.includes(`editorial-review-${seat}-r`))).toBe(true);
    }

    const transcripts = readdirSync(join(dir, 'docs', 'designs', '002-x', 'transcripts'));
    const editorialTranscripts = transcripts.filter((f) => f.startsWith('editorial-review'));
    expect(editorialTranscripts.length).toBeGreaterThan(0);
    // `-rN` naming: no c<pass> segment anywhere in editorial artifact names.
    for (const name of editorialTranscripts) {
      expect(name).toMatch(/^editorial-review-[a-z-]+-r\d+\.md$/);
    }
    expect(transcripts.some((f) => /-r\d+c\d+\.md$/.test(f))).toBe(true); // design passes keep rNcM
  });
});
