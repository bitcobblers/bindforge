import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { type ResumeOptions, resumePipeline } from '../src/pipeline/runner.js';
import { type PipelineConfig } from '../src/pipeline/stages.js';
import type { PipelineState } from '../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-002 funnel loop, driven end to end against stub sessions: pass naming
// (rNcM), dissent-only re-review, one ratification pass for the approvers,
// the revise stage receiving only the dissenting seat's report, the deadlock
// stop, the session budget, the pass backstop, and resume mid-round with a
// partially-recorded panel. The whole loop must fit one 40-session design
// budget with slack — run 002-run2 burned 40 sessions and never converged.
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

const DESIGN_DOC = join('docs', 'designs', '002-x', '02-proposal.md');

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

function tmpRepo(state: PipelineState): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-funnel-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(
    join(dir, 'docs', 'adrs', '002-x.md'),
    '---\nstatus: proposed\n---\n\n# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n',
  );
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  writeFileSync(join(dir, DESIGN_DOC), '# proposal\n\nfirst body\n');
  // Green test gate without a real package: `npm run check` must exit 0.
  writeFileSync(
    join(dir, 'package.json'),
    '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
  );
  writeFileSync(
    join(dir, 'docs', 'designs', '002-x', '.pipeline.json'),
    `${JSON.stringify(state, null, 2)}\n`,
  );
  git(['init', '-q', '-b', 'main']);
  git(['add', '-A']);
  git(['commit', '-q', '-m', 'docs: base']);
  return dir;
}

interface StubOptions {
  /** stageKey -> session ordinals that answer request-changes (0-based). */
  rejects: Record<string, number[]>;
  /** revise sessions write a fresh document (new revision stamp). */
  reviseWrites: boolean;
  /** revise replies `changed: false` (the deadlock input). */
  reviseNoChange?: boolean;
}

function stubClient(dir: string, opts: StubOptions) {
  const created: Record<string, number> = {};
  const sessions = new Map<
    string,
    { key: string; ordinal: number; prompts: number; lists: number }
  >();
  let clock = 0;
  let revisions = 0;
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
        if (!s) {
          return;
        }
        s.prompts += 1;
        if (s.key === 'proposal-revise' && opts.reviseWrites) {
          revisions += 1;
          writeFileSync(join(dir, DESIGN_DOC), `# proposal\n\nrevision ${String(revisions)}\n`);
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
            content: [{ type: 'text', text: replyFor(s.key, s.ordinal) }],
          });
          clock += 1;
          messages.push({ id: `m_i${String(clock)}`, time: { created: clock }, type: 'idle' });
        }
        return { data: messages, cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  const rejected = (key: string, ordinal: number) => (opts.rejects[key] ?? []).includes(ordinal);
  const replyFor = (key: string, ordinal: number): string => {
    if (key.endsWith('-review')) return 'impossible'; // stage keys here always carry :specialty
    if (key.includes('-review:')) return rejected(key, ordinal) ? reject : approve;
    if (key === 'proposal-revise') {
      return opts.reviseNoChange
        ? JSON.stringify({ changed: false, summary: 'the finding is already resolved in Decision' })
        : JSON.stringify({
            changed: true,
            commitMessage: 'docs: revise the design',
            summary: 'closed it',
          });
    }
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 0, commitMessage: 'test: pin the contract' });
    if (key === 'implement')
      return JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok' });
  };
  const sessionsCreated: string[] = [];
  const origCreate = client.session.create;
  client.session.create = async (input: { title?: string }) => {
    sessionsCreated.push((input.title ?? '').split(': ').slice(1).join(': '));
    return origCreate(input);
  };
  return { client, sessionsCreated };
}

function optionsFor(dir: string, extra: object = {}): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../prompts/', import.meta.url)),
    cfg: FAST,
    ...extra,
  };
}

function savedState(dir: string): PipelineState {
  return JSON.parse(
    readFileSync(join(dir, 'docs', 'designs', '002-x', '.pipeline.json'), 'utf8'),
  ) as PipelineState;
}

function readTranscript(dir: string, name: string): string {
  return readFileSync(join(dir, 'docs', 'designs', '002-x', 'transcripts', name), 'utf8');
}

describe('funnel loop end to end', () => {
  it('dissent-only re-review, one ratification pass, then done — 10 design sessions', async () => {
    const dir = tmpRepo(baseState());
    const { client, sessionsCreated } = stubClient(dir, {
      rejects: { 'design-review:security': [0] }, // only the first security session dissents
      reviseWrites: true,
    });

    const status = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('done');

    // One round, three passes: full panel c0, security alone at c1, the four
    // approvers at ratification c2. Run 2 would have spent 15 sessions here.
    const designSessions = sessionsCreated.filter((k) => k.startsWith('design-review:'));
    expect(designSessions).toHaveLength(10);
    expect(sessionsCreated.filter((k) => k === 'design-review:security')).toHaveLength(2);
    expect(sessionsCreated.filter((k) => k === 'design-review:performance')).toHaveLength(2); // c0 + ratify

    const saved = savedState(dir);
    // designRound reads 2 only because the editorial pass continues the rN
    // numbering; the whole funnel converged inside round 1.
    expect(saved.designRound).toBe(2);
    expect(saved.designCycle).toBe(3);
    expect(saved.designBudgetUsed).toBe(1); // ONE revise round bought all of it
    const stamp = saved.designStamp;
    for (const seat of Object.values(saved.seats?.design ?? {})) {
      expect(seat.verdict).toBe('approve');
      expect(seat.stamp).toBe(stamp);
    }

    // Pass-numbered reports and transcripts, and the dissent/ratify wording
    // reaching the reviewer prompts.
    expect(
      existsSync(
        join(dir, 'docs', 'designs', '002-x', 'transcripts', 'design-review-security-r1c1.md'),
      ),
    ).toBe(true);
    expect(
      existsSync(
        join(dir, 'docs', 'designs', '002-x', 'transcripts', 'design-review-correctness-r1c2.md'),
      ),
    ).toBe(true);
    expect(readTranscript(dir, 'design-review-security-r1c1.md')).toContain('dissent pass c1');
    // Review independence: the dissent prompt carries the seat's OWN c0
    // findings re-injected from the checkpoint — no report file to read —
    // and never names another seat's report.
    const c1 = readTranscript(dir, 'design-review-security-r1c1.md');
    expect(c1).toContain('Your own previous findings (security seat)');
    expect(c1).toContain('still broken');
    expect(c1).toContain('Review independently');
    expect(c1).not.toContain('correctness-r1c0.md');
    expect(readTranscript(dir, 'design-review-security-r1c1.md')).toContain('security-r1c1.md');
    expect(readTranscript(dir, 'design-review-correctness-r1c2.md')).toContain(
      'ratification pass c2',
    );

    // The designer was handed ONLY the dissenting seat's report.
    const revise = readTranscript(dir, 'proposal-revise-r1.md');
    expect(revise).toContain('security-r1c0.md');
    expect(revise).not.toContain('correctness-r1c0.md');
  });

  it('deadlock-stops the round when the designer replies changed:false', async () => {
    const dir = tmpRepo(baseState());
    const { client, sessionsCreated } = stubClient(dir, {
      rejects: { 'design-review:security': [0] },
      reviseWrites: false,
      reviseNoChange: true,
    });

    const status = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('escalated');
    // One full panel, one revise, then stop — no second dissent pass is spent.
    expect(sessionsCreated.filter((k) => k.startsWith('design-review:'))).toHaveLength(5);
    const report = readFileSync(join(dir, 'docs', 'designs', '002-x', 'ESCALATION.md'), 'utf8');
    expect(report).toContain('deadlock');
    expect(report).toContain('security still dissents');
  });

  it('the session budget caps review cost, and --extend-budget buys sessions', async () => {
    const cfg: PipelineConfig = { ...FAST, maxReviews: { design: 6, code: 20 } };
    const dir = tmpRepo(baseState());
    const { client, sessionsCreated } = stubClient(dir, {
      rejects: { 'design-review:security': [0, 1, 2, 3, 4, 5] },
      reviseWrites: true,
    });

    const status = await resumePipeline(optionsFor(dir, { client, cfg }));
    expect(status).toBe('escalated');
    // c0 spent 5 sessions; c1 (dissent: security alone) spent 1 more; entry to
    // c2 sees 6 >= 6 and stops. The funnel spends what dissent costs.
    expect(sessionsCreated.filter((k) => k.startsWith('design-review:'))).toHaveLength(6);
    const report = readFileSync(join(dir, 'docs', 'designs', '002-x', 'ESCALATION.md'), 'utf8');
    expect(report).toContain('review session budget exhausted (6/6 sessions)');

    // Same run extended: one granted session admits the next pass entry.
    const dir2 = tmpRepo(baseState());
    const stub2 = stubClient(dir2, {
      rejects: { 'design-review:security': [0, 1, 2, 3, 4, 5] },
      reviseWrites: true,
    });
    const status2 = await resumePipeline(
      optionsFor(dir2, { client: stub2.client, cfg, extendBudget: 1 }),
    );
    expect(status2).toBe('escalated');
    // The grant admits exactly one more pass entry (1 dissent session), then
    // the next entry sees 7 >= 6 with no grant left.
    expect(stub2.sessionsCreated.filter((k) => k.startsWith('design-review:'))).toHaveLength(7);
    const report2 = readFileSync(join(dir2, 'docs', 'designs', '002-x', 'ESCALATION.md'), 'utf8');
    expect(report2).toContain('review session budget exhausted (7/6 sessions)');
    const saved2 = savedState(dir2);
    const consents = saved2.gateLog.filter((g) => g.gate === 'operatorExtension');
    expect(consents.map((g) => g.reason).join('\n')).toContain(
      'extended the review budget by 1 review session(s) over the caps of 6/20',
    );
    expect(consents.map((g) => g.reason).join('\n')).toContain(
      'over-cap design review pass entered under operator extension',
    );
  });

  it('the pass backstop stops a stubborn seat inside one round', async () => {
    const cfg: PipelineConfig = { ...FAST, maxPassesPerRound: 3 };
    const dir = tmpRepo(baseState());
    const { client, sessionsCreated } = stubClient(dir, {
      rejects: {
        'design-review:security': [0, 1, 2, 3, 4, 5],
        'design-review:correctness': [0, 1, 2, 3, 4, 5],
        'design-review:compatibility': [0, 1, 2, 3, 4, 5],
        'design-review:performance': [0, 1, 2, 3, 4, 5],
        'design-review:complexity': [0, 1, 2, 3, 4, 5],
      },
      reviseWrites: true,
    });

    const status = await resumePipeline(optionsFor(dir, { client, cfg }));
    expect(status).toBe('escalated');
    // Three passes (c0, c1, c2), not an unbounded dissent cycle.
    expect(sessionsCreated.filter((k) => k.startsWith('design-review:'))).toHaveLength(15);
    const saved = savedState(dir);
    const denied = saved.gateLog
      .filter((g) => !g.proceed)
      .map((g) => g.reason)
      .join('\n');
    expect(denied).toContain('pass backstop');
    expect(denied).toContain('cap 3 passes per round');
  });

  it('resumes mid-round from a partially recorded panel', async () => {
    // Crash after c0 with two of five verdicts recorded: the resume must pick
    // up the funnel from the seat ledger, not restart the round.
    const dir = tmpRepo(
      baseState({
        lastStage: 'design-review',
        designRound: 1,
        designBudgetUsed: 1,
        designCycle: 1,
        designStamp: 'gone',
        seats: {
          design: {
            security: { verdict: 'request-changes', stamp: 'gone', dissents: 1 },
            correctness: { verdict: 'approve', stamp: 'gone', dissents: 0 },
          },
        },
        lastReviewPass: { phase: 'design', round: 1, cycle: 0, seats: ['security'] },
        results: [
          {
            stage: 'design-review:security',
            round: 1,
            sessionID: 'ses_old',
            agent: 'review/design-security',
            at: 0,
            reply: {
              verdict: 'request-changes',
              findings: [{ severity: 'major', location: 'x', problem: 'p' }],
            },
            transcript: 't',
          },
          {
            stage: 'design-review:correctness',
            round: 1,
            sessionID: 'ses_old2',
            agent: 'review/design-correctness',
            at: 0,
            reply: { verdict: 'approve', findings: [] },
            transcript: 't',
          },
        ],
      }),
    );
    const { client, sessionsCreated } = stubClient(dir, { rejects: {}, reviseWrites: true });

    const status = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('done');

    // The dissenter is re-reviewed alone; the three never-seen seats and the
    // stale approver are caught up in one ratification pass.
    const designSessions = sessionsCreated.filter((k) => k.startsWith('design-review:'));
    expect(designSessions.filter((k) => k === 'design-review:security')).toHaveLength(1);
    expect(designSessions.filter((k) => k === 'design-review:performance')).toHaveLength(1);
    expect(sessionsCreated[0]).toBe('proposal-revise');

    const saved = savedState(dir);
    expect(saved.designRound).toBe(2); // editorial numbering; funnel stayed in r1
    expect(saved.designCycle).toBe(3); // c1 dissent + c2 ratify after the crashed c0
    expect(Object.values(saved.seats?.design ?? {})).toHaveLength(5);
  });

  it('a checkpoint written before seats existed still loads and runs', async () => {
    // Additive checkpoint fields: version stays 1, old files parse.
    const old = baseState();
    const dir = tmpRepo(old);
    const { client } = stubClient(dir, { rejects: {}, reviseWrites: true });
    const status = await resumePipeline(optionsFor(dir, { client }));
    expect(status).toBe('done');
    const saved = savedState(dir);
    expect(saved.version).toBe(1);
    expect(saved.seats?.design?.security?.verdict).toBe('approve');
  });
});
