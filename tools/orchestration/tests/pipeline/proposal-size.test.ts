import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import {
  escalationMarkdown,
  proposalGrowth,
  proposalGrowthLine,
} from '../../src/pipeline/report.js';
import { type RunOptions, executeStage } from '../../src/pipeline/runner.js';
import type { PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState, StageResult } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// Run 002-run2 grew its design document from 401 lines to 2,696 over 8 revise
// rounds, and every one of those rounds reported closing its findings. Nothing
// the operator could see - checkpoint, `pipeline status`, ESCALATION.md - said
// the design had outgrown its ADR. The meter is taken by the runner from the
// worktree, because the stages that did the growing are not a source an
// operator can trust about their own growth.
// ---------------------------------------------------------------------------

const FAST: PipelineConfig = {
  maxRounds: 5,
  maxReviews: { design: 40, code: 20 },
  maxPassesPerRound: 6,
  blockingSeverities: ['blocker', 'major'],
  pollIntervalMs: 5,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

const TIMEOUT_FAST: PipelineConfig = { ...FAST, stageTimeoutMs: 40 };

const DESIGN_DIR_REL = 'docs/designs/002-x';

function state(): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: 'docs/adrs/002-x.md' },
    status: 'running',
    lastStage: 'proposal',
    designRound: 1,
    codeRound: 0,
    designBudgetUsed: 1,
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

function sizedResult(stage: string, added: number, removed: number, docLines: number): StageResult {
  return {
    stage,
    round: 1,
    sessionID: 'ses_x',
    agent: 'design/designer',
    at: 1,
    reply: {},
    transcript: 'transcripts/x.jsonl',
    docLines,
    docLinesAdded: added,
    docLinesRemoved: removed,
  };
}

/** A real repo: every stage runs git, and the meter reads git numstat. */
function tmpRepo(committedDoc?: string): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-proposal-size-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, DESIGN_DIR_REL), { recursive: true });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'adrs', '002-x.md'), 'x\n');
  if (committedDoc !== undefined) {
    writeFileSync(join(dir, DESIGN_DIR_REL, '02-proposal.md'), committedDoc);
  }
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

/**
 * Stage session that writes the design document while it works and then replies.
 * The reply claims the opposite of what the file shows, so a meter read from the
 * reply would fail these tests.
 */
function writingClient(repoDir: string, doc: string) {
  const sessions = new Map<string, { prompts: number; lists: number; writesDoc: boolean }>();
  let clock = 0;
  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const id = `ses_${String(sessions.size + 1)}`;
        sessions.set(id, {
          prompts: 0,
          lists: 0,
          writesDoc: (input.title ?? '').includes('proposal'),
        });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) {
          return;
        }
        s.prompts += 1;
        if (s.writesDoc) {
          writeFileSync(join(repoDir, DESIGN_DIR_REL, '02-proposal.md'), doc);
        }
      },
      remove: async () => undefined,
      // Test strategy #9: the descendant walk's enumeration endpoint. This
      // fixture creates no child sessions, so an empty page is the truth.
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID) ?? { prompts: 0, lists: 0, writesDoc: false };
        s.lists += 1;
        // The runner reads a message baseline before its reply lands, so the
        // first listing of a session shows nothing.
        const messages: SessionMessageInfo[] = [];
        if (Math.min(s.lists - 1, s.prompts) > 0) {
          clock += 1;
          messages.push({
            id: `m_a${String(clock)}`,
            time: { created: clock },
            type: 'assistant',
            content: [
              {
                type: 'text',
                text: sessions.get(input.sessionID)?.writesDoc
                  ? '{"changed": true, "commitMessage": "docs: revise the proposal", "summary": "the document did not change size"}'
                  : '{"verdict": "approve", "findings": []}',
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

/** Session that never finishes, so the attempt fails. */
function busyClient(): OpenCodeClient {
  let turn = 0;
  return {
    session: {
      create: async () => ({ id: 'ses_busy' }),
      prompt: async () => undefined,
      remove: async () => undefined,
      // Test strategy #9: the descendant walk's enumeration endpoint. This
      // fixture creates no child sessions, so an empty page is the truth.
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async () => {
        turn += 1;
        return {
          data: [
            {
              id: `m_${String(turn)}`,
              time: { created: turn },
              type: 'assistant',
              content: [{ type: 'tool' }],
            } as unknown as SessionMessageInfo,
          ],
          cursor: { previous: null, next: null },
        };
      },
    },
  } as unknown as OpenCodeClient;
}

function options(client: OpenCodeClient, repoDir: string): RunOptions {
  return {
    repoDir,
    sourcePath: 'docs/adrs/002-x.md',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    client,
  };
}

function handleFor(stage: 'proposal' | 'proposal-revise' | 'design-review') {
  return {
    stage,
    stageKey: stage === 'design-review' ? 'design-review:correctness' : stage,
    attempt: 1,
    round: 1,
    specialty: stage === 'design-review' ? 'correctness' : undefined,
    vars: {
      designDir: DESIGN_DIR_REL,
      sourcePath: 'docs/adrs/002-x.md',
      round: 1,
      specialty: 'correctness',
      scope: 'the whole document',
      passNote: 'full-panel pass c0',
      prevFindings: '(none — test fixture)',
      reportPath: `${DESIGN_DIR_REL}/reviews/design/correctness-r1c0.md`,
      reviewFiles: '',
    },
  };
}

function lines(n: number): string {
  return Array.from({ length: n }, (_, i) => `line ${String(i + 1)}`).join('\n') + '\n';
}

describe('proposal size meter', () => {
  it('measures the first proposal from the worktree, not the stage reply', async () => {
    const repoDir = tmpRepo();
    const s = state();

    const outcome = await executeStage(
      options(writingClient(repoDir, lines(12)), repoDir),
      s,
      handleFor('proposal'),
      FAST,
    );

    expect(outcome.ok).toBe(true);
    const result = s.results[0];
    expect(result?.docLines).toBe(12);
    // Nothing at HEAD for this path: the whole document is the addition.
    expect(result?.docLinesAdded).toBe(12);
    expect(result?.docLinesRemoved).toBe(0);
  });

  it('measures a revise round that deletes content as net negative', async () => {
    const repoDir = tmpRepo(lines(20));
    const s = state();

    const outcome = await executeStage(
      // The stage cuts 8 lines from the committed document while its reply
      // claims the size did not change.
      options(writingClient(repoDir, lines(12)), repoDir),
      s,
      handleFor('proposal-revise'),
      FAST,
    );

    expect(outcome.ok).toBe(true);
    const result = s.results[0];
    expect(result?.docLines).toBe(12);
    expect(result?.docLinesAdded).toBe(0);
    expect(result?.docLinesRemoved).toBe(8);
  });

  it('leaves review stages without a size record', async () => {
    const repoDir = tmpRepo(lines(20));
    const s = state();

    const outcome = await executeStage(
      options(writingClient(repoDir, lines(20)), repoDir),
      s,
      handleFor('design-review'),
      FAST,
    );

    expect(outcome.ok).toBe(true);
    expect(s.results[0]?.docLines).toBeUndefined();
    expect(s.results[0]?.docLinesAdded).toBeUndefined();
  });

  it('records no size for a failed attempt', async () => {
    const repoDir = tmpRepo(lines(20));
    const s = state();

    const outcome = await executeStage(
      options(busyClient(), repoDir),
      s,
      handleFor('proposal-revise'),
      TIMEOUT_FAST,
    );

    expect(outcome.ok).toBe(false);
    expect(s.results[0]?.docLines).toBeUndefined();
  });
});

describe('proposalGrowth', () => {
  it('is absent when no design stage reported a size', () => {
    const s = state();
    s.results = [{ ...sizedResult('proposal', 1, 1, 10), docLines: undefined }];
    expect(proposalGrowth(s)).toBeUndefined();
    expect(proposalGrowthLine(s)).toBeNull();
  });

  it('sums the run and reports the count after the last design stage', () => {
    const s = state();
    s.results = [
      sizedResult('proposal', 401, 0, 401),
      sizedResult('proposal-revise', 300, 41, 660),
      sizedResult('proposal-revise', 270, 60, 870),
    ];
    expect(proposalGrowth(s)).toEqual({
      docLines: 870,
      added: 971,
      removed: 101,
      stages: 3,
      // The first write counts too: every stage here grew the document.
      netPositiveStreak: 3,
    });
  });

  it('stops the streak at the first round that did not grow the document', () => {
    const s = state();
    s.results = [
      sizedResult('proposal-revise', 300, 10, 691),
      sizedResult('proposal-revise', 200, 260, 631),
      sizedResult('proposal-revise', 270, 280, 621),
    ];
    expect(proposalGrowth(s)?.netPositiveStreak).toBe(0);
  });

  it('warns once the proposal has grown for three design rounds running', () => {
    const s = state();
    s.results = [
      sizedResult('proposal-revise', 300, 41, 660),
      sizedResult('proposal-revise', 270, 60, 870),
    ];
    expect(proposalGrowthLine(s)).toContain('proposal:  870 lines (+570/-101');
    expect(proposalGrowthLine(s)).not.toContain('WARNING');

    s.results.push(sizedResult('proposal-revise', 260, 50, 1080));
    expect(proposalGrowthLine(s)).toContain(
      'WARNING: the proposal has grown for 3 design rounds running',
    );
  });

  it('shows design scope in the escalation report an operator reads', () => {
    const s = state();
    s.status = 'escalated';
    s.results = [sizedResult('proposal-revise', 300, 41, 660)];
    s.gateLog = [
      { gate: 'design-review-round', round: 1, proceed: false, reason: 'open majors', at: 1 },
    ];

    const md = escalationMarkdown(s, 'design', ['blocker', 'major']);
    expect(md).toContain('## Proposal size');
    expect(md).toContain('proposal:  660 lines (+300/-41 across 1 design stage(s))');
  });

  // proves: Test strategy #10 — the record names the flow that ran, with the
  // effective light numbers from the same `configFor` the runner drives with: a
  // light escalation must not read as a full run whose design phase vanished.
  it('prints the light flow line with the effective numbers', () => {
    const s = state();
    s.status = 'escalated';
    s.flow = 'light';
    s.lastStage = 'code-review';
    s.gateLog = [{ gate: 'code-review', round: 1, proceed: false, reason: 'open majors', at: 1 }];

    const md = escalationMarkdown(s, 'code', ['blocker', 'major']);
    expect(md).toContain(
      'light (plan → implement → code review, seats: correctness, testing; rounds 2, code reviews 6)',
    );
    expect(md).toMatch(/\*\*code\*\* phase|code phase/);
  });

  // proves: Test strategy #10 — a full run's report gains that one line and
  // keeps every existing section byte-for-byte, including the size section.
  it('keeps the full report intact beside the flow line', () => {
    const s = state();
    s.status = 'escalated';
    s.results = [sizedResult('proposal-revise', 300, 41, 660)];
    s.gateLog = [
      { gate: 'design-review-round', round: 1, proceed: false, reason: 'open majors', at: 1 },
    ];

    const md = escalationMarkdown(s, 'design', ['blocker', 'major']);
    expect(md).toMatch(/flow:\s+full/);
    expect(md).toContain('## Proposal size');
    expect(md).toContain('proposal:  660 lines (+300/-41 across 1 design stage(s))');
  });
});
