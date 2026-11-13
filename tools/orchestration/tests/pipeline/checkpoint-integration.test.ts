import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-006 acceptance scenarios end to end — mock client, temp git repos, the
// shape of parallel-integration.test.ts / stage-lifecycle.test.ts: an older-
// era checkpoint resumes on current code (the self-update guarantee), a run
// interrupted between a stage commit and the next save resumes from the
// committed checkpoint, and a checkpoint path replaced by a directory
// mid-run ends the run escalated with a record — never crashed — and resumes
// from the committed copy after the printed repair.
// ---------------------------------------------------------------------------

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

const DESIGN_DIR = join('docs', 'designs', '002-x');

function baseState(partial: Partial<PipelineState> = {}): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'running',
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

/** Repo with a committed checkpoint, the design doc, and a green check gate. */
function tmpRepo(state: PipelineState): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-adr006-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
  writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), '# proposal\n\nfirst body\n');
  // The real repo gitignores the run lock (ADR-006 §4, deviation 9): the
  // boundary re-assertion must never make it a commit-visible file here.
  writeFileSync(join(dir, '.gitignore'), '.pipeline.lock\n');
  // Green test gate without a real package: `npm run check` must exit 0.
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), `${JSON.stringify(state, null, 2)}\n`);
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/** Stub client: every stage answers with a valid reply, so a run reaches done.
 * The session grows the way a real one does: the read the runner takes right
 * after the stage prompt is the baseline — no assistant turn yet — and the turn
 * only shows up on a later poll (`visible = max(0, min(lists - 1, prompts))`,
 * as in `funnel.test.ts`). A stub that answers every read from the first one
 * gives the runner no growth to detect, so its reply contract treats the stage
 * as an empty reply and the attempt ends at the deadline as `stage-timeout`:
 * the fixture would then drive the retry/escalate path instead of the resume
 * path under test here.
 */
function approveClient(
  dir: string,
  hooks: { onFirstPrompt?: () => void } = {},
): { client: OpenCodeClient; created: string[] } {
  const created: string[] = [];
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  let clock = 0;
  let promptsSent = 0;
  let revisions = 0;

  const replyFor = (key: string): string => {
    if (key.endsWith('-review') || key.includes('-review:')) return approve;
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'proposal-revise')
      return JSON.stringify({ changed: true, summary: 'ok', commitMessage: 'docs: revise' });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 0, commitMessage: 'test: pin the contract' });
    if (key === 'implement')
      return JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok', commitMessage: 'docs: ok' });
  };

  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created.push(key);
        const id = `ses_${String(created.length)}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        promptsSent += 1;
        if (promptsSent === 1) hooks.onFirstPrompt?.();
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        // A revise stage that claims `changed: true` has to leave a new
        // revision stamp behind, exactly as funnel.test.ts's stub does.
        if (s.key === 'proposal-revise') {
          revisions += 1;
          writeFileSync(
            join(dir, DESIGN_DIR, '02-proposal.md'),
            `# proposal\n\nrevision ${String(revisions)}\n`,
          );
        }
      },
      remove: async () => undefined,
      // The independence walk's enumeration endpoint: this fixture creates no
      // child sessions, so an empty page is the truth.
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { key: '', prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const text = replyFor(s.key);
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

function optionsFor(dir: string, extra: object = {}): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: FAST,
    ...extra,
  };
}

function savedState(dir: string): PipelineState {
  return JSON.parse(readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8')) as PipelineState;
}

function fixture(name: string): PipelineState {
  return JSON.parse(
    readFileSync(fileURLToPath(new URL(`./fixtures/checkpoints/${name}`, import.meta.url)), 'utf8'),
  ) as PipelineState;
}

describe('cross-era checkpoint resumes on current code', () => {
  // proves: Test strategy #11 — the self-update resume guarantee end to end:
  // a run-2-era checkpoint (no stageCommits, no baseRef, a reply-less failure
  // record) resumes without refusal, driving continues after lastStage, and
  // baseRef backfills.
  it('resumes the run-2-era fixture with no refusal and backfills baseRef', async () => {
    const checkpoint = fixture('run-2-era.json');
    expect(checkpoint.stageCommits).toBeUndefined();
    expect(checkpoint.baseRef).toBeUndefined();

    const dir = tmpRepo(checkpoint);
    const { client, created } = approveClient(dir);

    const { status } = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('done');

    // §2's promise is that driving continues *after* the checkpoint's
    // lastStage and that the old shape is not refused. Which stage the funnel
    // picks next is the funnel's own, pre-existing choice — ADR-006 changes no
    // gates, rounds, or fan-out — so this pins the promise rather than the
    // funnel's pick: the run resumed past design-review instead of restarting
    // the phase from the beginning.
    expect(created.length).toBeGreaterThan(0);
    expect(created[0]).not.toBe('research');
    expect(created[0]).not.toBe('proposal');
    const saved = savedState(dir);
    // The checkpoint itself advanced past the stage the old runner stopped in.
    expect(saved.lastStage).not.toBe(checkpoint.lastStage);
    expect(typeof saved.baseRef).toBe('string');
    expect(saved.baseRef).not.toBe('');
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
  });
});

describe('interrupted between a stage commit and the next save', () => {
  // proves: Test strategy #12 — the committed checkpoint is the recovery
  // anchor: the stage commit landed but the next save did not, so resume
  // continues after lastStage with no surviving seat records, and after the
  // run nothing dangles — the checkpoint moved only through the runner's own
  // write path (the fence kept every copy runner-authored).
  it('resumes from the committed checkpoint with the fence untouched', async () => {
    const dir = tmpRepo(
      baseState({
        status: 'running',
        lastStage: 'design-review',
        designRound: 1,
        designBudgetUsed: 1,
      }),
    );
    const { client, created } = approveClient(dir);

    const { status } = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('done');

    // The checkpoint stopped inside the design-review loop, so resume's first
    // inference is a later stage of that loop — never a stage before it. The
    // funnel's exact choice is not ADR-006's to pin (no gate, round, or
    // fan-out change), so the assertion is the guarantee itself.
    expect(created.length).toBeGreaterThan(0);
    expect(created[0]).not.toBe('research');
    expect(created[0]).not.toBe('proposal');
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
    const saved = savedState(dir);
    expect(saved.status).toBe('done');
    // The resume ran on past the stage the interrupted run recorded.
    expect(saved.lastStage).not.toBe('design-review');
    // Every stage boundary committed its runner-authored checkpoint: the
    // worktree copy is clean, so no unsanctioned copy waits at `git add -A`.
    expect(execFileSync('git', ['status', '--porcelain'], { cwd: dir, encoding: 'utf8' })).toBe('');
  });
});

describe('checkpoint path replaced by a directory mid-run', () => {
  // proves: Test strategy #13 — the run-2 blocker scenario: a stage replaces
  // the checkpoint path with a directory mid-run. The run ends escalated
  // with a record — not crashed — the report names the failed save and the
  // git recovery posture, and after the operator repair (rm -r the directory,
  // restore the committed copy) resume proceeds from that committed
  // checkpoint.
  it('ends escalated with a record, then resumes after the repair', async () => {
    const dir = tmpRepo(baseState({ status: 'running' }));
    const checkpoint = join(dir, DESIGN_DIR, '.pipeline.json');
    const sabotaged = approveClient(dir, {
      onFirstPrompt: () => {
        rmSync(checkpoint);
        mkdirSync(checkpoint);
        writeFileSync(join(checkpoint, 'planted.md'), 'x\n');
      },
    });

    const { status } = await resumePipeline(optionsFor(dir, { client: sabotaged.client }));
    // Escalated, not a crash: the caller gets the terminal status, and the
    // run's terminal record exists even though the checkpoint could not be
    // written.
    expect(status).toBe('escalated');

    const report = readFileSync(join(dir, DESIGN_DIR, 'ESCALATION.md'), 'utf8');
    expect(report).toContain('checkpoint save failed');
    expect(report).toContain('git checkout HEAD --');

    // testing r1 #5: the in-process restore (deviation 1) ran before the
    // escalation commit — the planted directory is gone, the path holds the
    // newest runner-authored save (the committed one: the failed save never
    // landed, so the split record stands), and the escalation commit carried
    // no checkpoint-shaped path, planted residue included.
    expect(statSync(checkpoint).isFile()).toBe(true);
    expect(existsSync(join(checkpoint, 'planted.md'))).toBe(false);
    const restored = JSON.parse(readFileSync(checkpoint, 'utf8')) as PipelineState;
    expect(restored.version).toBe(1);
    expect(restored.status).toBe('running');
    expect(restored.lastStage).toBe('proposal');
    const committedFiles = execFileSync('git', ['show', '--name-only', '--format=', 'HEAD'], {
      cwd: dir,
      encoding: 'utf8',
    })
      .split('\n')
      .map((f) => f.trim())
      .filter((f) => f !== '');
    expect(committedFiles).toContain(join(DESIGN_DIR, 'ESCALATION.md'));
    expect(committedFiles.filter((f) => f.includes('.pipeline'))).toEqual([]);

    // The §3 operator posture: clear the path, restore the newest committed
    // checkpoint, and resume from it.
    rmSync(checkpoint, { recursive: true, force: true });
    const cpRel = join(DESIGN_DIR, '.pipeline.json');
    const lastGood = execFileSync('git', ['rev-list', '-n', '1', 'HEAD', '--', cpRel], {
      cwd: dir,
      encoding: 'utf8',
    }).trim();
    expect(lastGood).not.toBe('');
    execFileSync('git', ['checkout', lastGood, '--', cpRel], { cwd: dir });

    const clean = approveClient(dir);
    const { status: resumed } = await resumePipeline(optionsFor(dir, { client: clean.client }));
    expect(resumed).toBe('done');

    // testing r1 #4: the marker was committed by the escalation commit, and
    // the second resume's delete of it is runner-authored — the sweep must
    // not resurrect it from headBefore at the next boundary.
    expect(existsSync(join(dir, DESIGN_DIR, 'ESCALATION.md'))).toBe(false);
  });
});
