import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
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
import { DEFAULT_CONFIG, type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDir } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// The decision-unit ledger gate (design 021, Test strategy #9, #11, #12):
// `checkLedgers` end to end over fixture ledgers written into the scratch
// repos of `tests/pipeline/tmp-dir.ts` (no committed markdown for prettier to
// reformat out from under a shape pin), the `ledgerGate` denial bounded by
// `MAX_LEDGER_FINDINGS` with the elided count stated, the entry hook beside
// `sourceAdmissible` in `runPipeline`, and the finalize hook before
// `flipAdrStatus` — 021-R2's machine-verified acceptance: a run cannot complete
// while one of its own units is unhandled.
//
// The ledger import is dynamic on purpose — `src/pipeline/ledger.ts` is what
// the design adds; the runner-level suites (#11, #12) drive the real
// `runPipeline`/`resumePipeline` and go red on their own assertions.
// ---------------------------------------------------------------------------

interface LedgerFinding {
  file: string;
  line: number;
  code: string;
  detail: string;
}

interface GateResult {
  proceed: boolean;
  reason: string;
}

interface LedgerSurface {
  checkLedgers?: (repoDir: string) => LedgerFinding[];
  ledgerGate?: (repoDir: string) => GateResult;
  MAX_LEDGER_FINDINGS?: number;
}

const LEDGER = (await import('../../src/pipeline/ledger.js').catch(
  (): LedgerSurface => ({}),
)) as LedgerSurface;

function checkLedgers(): NonNullable<LedgerSurface['checkLedgers']> {
  if (typeof LEDGER.checkLedgers !== 'function') {
    throw new Error('ledger.js exports no checkLedgers yet');
  }
  return LEDGER.checkLedgers;
}

function ledgerGate(): NonNullable<LedgerSurface['ledgerGate']> {
  if (typeof LEDGER.ledgerGate !== 'function') {
    throw new Error('ledger.js exports no ledgerGate yet');
  }
  return LEDGER.ledgerGate;
}

function maxLedgerFindings(): number {
  if (typeof LEDGER.MAX_LEDGER_FINDINGS !== 'number') {
    throw new Error('ledger.js exports no MAX_LEDGER_FINDINGS yet');
  }
  return LEDGER.MAX_LEDGER_FINDINGS;
}

const FINDING_NOTE =
  '**Unit handles.** A line whose final code span holds unit ids is a handle for those units.';

function corpusRepo(prefix: string, files: Record<string, string>): string {
  const dir = mkTmpDir(`bindforge-ledger-gates-${prefix}-`);
  for (const [rel, text] of Object.entries(files)) {
    const path = join(dir, rel);
    mkdirSync(join(path, '..'), { recursive: true });
    writeFileSync(path, text, 'utf8');
  }
  return dir;
}

// ===========================================================================
// Test strategy #9 — checkLedgers end to end, and the denial's bounded reason
// ===========================================================================

describe('checkLedgers and ledgerGate over a scratch corpus', () => {
  // proves: Test strategy #9 — fixture ledgers built in this module and
  // written into the scratch repo drive `checkLedgers` end to end: the walk
  // covers `docs/adrs/**` and `schemas/**` recursively, names each finding's
  // file and line, and `ledgerGate` denies with the findings in its reason.
  it('drives the corpus walk and names every fixture finding by file and line', () => {
    const specLines = [
      '# Manifests spec',
      '',
      FINDING_NOTE,
      '',
      'A grammar claim. `013-C1`',
      'A dangling claim. `013-C9`',
      '',
    ];
    const dir = corpusRepo('e2e-', {
      'docs/adrs/013-fixture.md': [
        '---',
        'status: proposed',
        'ledger: yes',
        '---',
        '',
        '# ADR 013: fixture',
        '',
        '## Context',
        '',
        'why.',
        '',
        '## Decision',
        '',
        '### Contracts',
        '',
        '- **013-C1** A handled contract. → spec: Grammar.',
        '',
        '### Reservations',
        '',
        '- **013-R5** The codec slot waits. → unassigned. Owner: unassigned.',
        '',
      ].join('\n'),
      'docs/adrs/014-fixture.md': [
        '---',
        'status: proposed',
        'depends-on: [021]',
        '---',
        '',
        '# ADR 014: fixture',
        '',
        'why.',
        '',
      ].join('\n'),
      'schemas/manifests/v1/spec.md': specLines.join('\n'),
      // A second schema doc carries a handle but no declaration note.
      'schemas/manifests/v1/notes.md': '# Notes\n\nOne more handle. `013-C1`\n',
    });

    const findings = checkLedgers()(dir).sort((a, b) => a.code.localeCompare(b.code));
    expect(findings.map((f) => f.code)).toEqual([
      'depends-order',
      'handle-dangling',
      'note-missing',
    ]);
    expect(findings.find((f) => f.code === 'handle-dangling')?.file).toBe(
      'schemas/manifests/v1/spec.md',
    );
    expect(findings.find((f) => f.code === 'handle-dangling')?.line).toBe(
      specLines.indexOf('A dangling claim. `013-C9`') + 1,
    );
    expect(findings.find((f) => f.code === 'depends-order')?.file).toBe('docs/adrs/014-fixture.md');
    expect(findings.find((f) => f.code === 'note-missing')?.file).toBe(
      'schemas/manifests/v1/notes.md',
    );

    const gate = ledgerGate()(dir);
    expect(gate.proceed).toBe(false);
    expect(gate.reason).toContain('handle-dangling');
    expect(gate.reason).toContain('depends-order');
    expect(gate.reason).toContain('note-missing');
  });

  // proves: Test strategy #9 — a denial lists at most `MAX_LEDGER_FINDINGS`
  // findings (the way `MAX_GATE_LEGS` bounds a gate record), each as a
  // `file:line code detail` line, and states the elided count as `+N more`.
  it('bounds a denial reason at MAX_LEDGER_FINDINGS and states the elided count', () => {
    const max = maxLedgerFindings();
    const claims: string[] = [];
    for (let i = 0; i < max + 3; i += 1) {
      claims.push(`Dangling claim ${String(i)}. \`099-C1\``);
    }
    const dir = corpusRepo('cap-', {
      'docs/adrs/099-many.md': [
        '---',
        'status: proposed',
        '---',
        '',
        '# ADR 099: many',
        '',
        FINDING_NOTE,
        '',
        ...claims,
        '',
      ].join('\n'),
    });

    const gate = ledgerGate()(dir);
    expect(checkLedgers()(dir)).toHaveLength(max + 3);
    expect(gate.proceed).toBe(false);
    const listed = gate.reason.split('\n').filter((line) => line.includes('handle-dangling'));
    expect(listed).toHaveLength(max);
    expect(gate.reason).toContain('+3 more');
  });

  // proves: Test strategy #9 — the same gate allows a conforming tree.
  it('allows a conforming corpus', () => {
    const dir = corpusRepo('green-', {
      'docs/adrs/013-fixture.md': [
        '---',
        'status: proposed',
        'ledger: yes',
        '---',
        '',
        '# ADR 013: fixture',
        '',
        '## Context',
        '',
        'why.',
        '',
        '## Decision',
        '',
        '### Contracts',
        '',
        '- **013-C1** A handled contract. → spec: Grammar.',
        '',
        '### Reservations',
        '',
        '- **013-R5** The codec slot waits. → unassigned. Owner: unassigned.',
        '',
      ].join('\n'),
      'schemas/manifests/v1/spec.md': [
        '# Manifests spec',
        '',
        FINDING_NOTE,
        '',
        'A grammar claim. `013-C1`',
        '',
      ].join('\n'),
    });
    expect(checkLedgers()(dir)).toEqual([]);
    expect(ledgerGate()(dir).proceed).toBe(true);
  });
});

// ===========================================================================
// Test strategy #11 — the entry hook in runPipeline (¶7, A9)
// ===========================================================================

const ENTRY_CFG: PipelineConfig = {
  ...DEFAULT_CONFIG,
  maxRounds: 1,
  maxPassesPerRound: 1,
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
  docMaxLines: 900,
};

const RED_ADR = [
  '---',
  'status: proposed',
  'ledger: yes',
  '---',
  '',
  '# ADR 002: x',
  '',
  '## Context',
  '',
  'why.',
  '',
  '## Decision',
  '',
  '### Contracts',
  '',
  '- **002-C1** A bounded choice nobody handles. → spec: Grammar.',
  '',
].join('\n');

const GREEN_ADR = [
  '---',
  'status: proposed',
  'ledger: yes',
  '---',
  '',
  '# ADR 002: x',
  '',
  '## Context',
  '',
  'why.',
  '',
  '## Decision',
  '',
  '### Contracts',
  '',
  '- **002-C1** A bounded choice. → spec: Grammar.',
  '',
  '## Notes',
  '',
  FINDING_NOTE,
  '',
  '002-C1 is handled by its own file. `002-C1`',
  '',
].join('\n');

/** A committed scratch repo: the fixture ADR under docs/adrs, an optional
 * light brief, and a green check script so the ledger gate is the only
 * failure signal. */
function entryRepo(adr: string, brief: string | null): string {
  const dir = mkTmpDir('bindforge-ledger-entry-');
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'adrs', '002-x.md'), adr, 'utf8');
  if (brief !== null) {
    mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
    writeFileSync(join(dir, 'docs', 'prompts', '002-brief.md'), brief, 'utf8');
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

// The scratch repo the entry suites point at: the runner creates the design
// directory at run start, so the stub resolves it only when a window prompts.
let ENTRY_REPO = '';

/** A client that records every created session key: the entry gate refusing a
 * red tree must never open a window. */
function recordingClient() {
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
        if (s !== undefined) {
          s.prompts += 1;
          // The research window produces an under-cap document so a green
          // tree moves past research to the next document stage. The ADR
          // `002-x.md` gives the run the identity `002-x`.
          if (s.key === 'research') {
            const designDir = join(ENTRY_REPO, 'docs', 'designs', '002-x');
            if (existsSync(designDir)) {
              const lines: string[] = [];
              for (let i = 0; i < 300; i += 1) {
                lines.push(`research line ${String(i + 1)}`);
              }
              writeFileSync(join(designDir, '01-research.md'), `${lines.join('\n')}\n`, 'utf8');
            }
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
          : '{"summary": "ok", "commitMessage": "docs: ok"}';
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

interface GateLogEntry {
  gate?: string;
  proceed?: boolean;
  reason?: string;
}

function gateLog(dir: string, designRel: string): GateLogEntry[] {
  const file = join(dir, designRel, '.pipeline.json');
  if (!existsSync(file)) return [];
  const state = JSON.parse(readFileSync(file, 'utf8')) as { gateLog?: GateLogEntry[] };
  return state.gateLog ?? [];
}

function entryOptions(dir: string, sourcePath: string, client: OpenCodeClient): RunOptions {
  return {
    repoDir: dir,
    sourcePath,
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: ENTRY_CFG,
    client,
  };
}

describe('the entry hook (runPipeline)', () => {
  // proves: Test strategy #11 — a temp repo with a `ledger: yes` ADR carrying
  // an unhandled unit refuses the run before the first stage and gate-logs
  // the finding under `ledger-join`: no window opens, the gate denial names
  // the code and the file, and the run escalates.
  it('escalates a red tree before the first stage and gate-logs under ledger-join', async () => {
    const dir = entryRepo(RED_ADR, null);
    ENTRY_REPO = dir;
    const { client, sessionsCreated } = recordingClient();

    const { status } = await runPipeline(entryOptions(dir, 'docs/adrs/002-x.md', client));

    expect(status).toBe('escalated');
    expect(sessionsCreated).toEqual([]);
    const entries = gateLog(dir, join('docs', 'designs', '002-x')).filter(
      (g) => g.gate === 'ledger-join',
    );
    expect(entries.length).toBeGreaterThanOrEqual(1);
    expect(entries.some((g) => g.proceed === false)).toBe(true);
    expect(entries.map((g) => g.reason ?? '').join('\n')).toContain('unit-unhandled');
    expect(entries.map((g) => g.reason ?? '').join('\n')).toContain('docs/adrs/002-x.md');
    expect(existsSync(join(dir, 'docs', 'designs', '002-x', 'ESCALATION.md'))).toBe(true);
  });

  // proves: Test strategy #11 (¶7) — the join is a tree invariant, so a brief
  // source is refused against the same red tree: the light-flow brief run
  // never opens a window either.
  it('refuses a brief source in the same red tree', async () => {
    const dir = entryRepo(RED_ADR, '---\nflow: light\n---\n\nDo it.\n');
    ENTRY_REPO = dir;
    const { client, sessionsCreated } = recordingClient();

    const { status } = await runPipeline(entryOptions(dir, 'docs/prompts/002-brief.md', client));

    expect(status).toBe('escalated');
    expect(sessionsCreated).toEqual([]);
    // The brief's NNN comes from the shared counter (next free after the
    // fixture's 002), not from its filename, so the run's design directory is
    // `003-brief` — the allocation `light-promotion.test.ts` pins.
    const entries = gateLog(dir, join('docs', 'designs', '003-brief')).filter(
      (g) => g.gate === 'ledger-join',
    );
    expect(entries.some((g) => g.proceed === false)).toBe(true);
  });

  // proves: Test strategy #11 — a green tree proceeds: the entry gate allows
  // under `ledger-join` and the run opens its first stage past the checkpoint.
  it('proceeds when the tree conforms', async () => {
    const dir = entryRepo(GREEN_ADR, null);
    ENTRY_REPO = dir;
    const { client, sessionsCreated } = recordingClient();

    await runPipeline(entryOptions(dir, 'docs/adrs/002-x.md', client));

    expect(sessionsCreated).toContain('research');
    expect(sessionsCreated).toContain('proposal');
    const entries = gateLog(dir, join('docs', 'designs', '002-x')).filter(
      (g) => g.gate === 'ledger-join',
    );
    expect(entries.length).toBeGreaterThanOrEqual(1);
    expect(entries.every((g) => g.proceed === true)).toBe(true);
  });
});

// ===========================================================================
// Test strategy #12 — the finalize hook (¶7, A10): 021-R2's acceptance
// ===========================================================================

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

function funnelState(partial: Partial<PipelineState> = {}): PipelineState {
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

function funnelRepo(adr: string): string {
  const dir = mkTmpDir('bindforge-ledger-finalize-');
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeFileSync(join(dir, 'docs', 'adrs', '002-x.md'), adr, 'utf8');
  mkdirSync(join(dir, 'docs', 'designs', '002-x'), { recursive: true });
  writeFileSync(join(dir, DESIGN_DOC), '# proposal\n\nfirst body\n');
  writeFileSync(
    join(dir, 'docs', 'designs', '002-x', '.pipeline.json'),
    `${JSON.stringify(funnelState(), null, 2)}\n`,
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

/** The funnel's green-panel stub: every seat approves, the coder windows
 * answer their contracts, so the run reaches finalize (the `funnel.test.ts`
 * posture). Per the compatibility matrix the entry hook is not re-run on
 * resume, so a red tree reaches the finalize hook instead of the entry one. */
function funnelStub() {
  const created: Record<string, number> = {};
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  let clock = 0;
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const replyFor = (key: string): string => {
    if (key.includes('-review:')) return approve;
    if (key === 'proposal-final') return JSON.stringify({ ambiguities: [] });
    if (key === 'test-authoring')
      return JSON.stringify({ testsAdded: 0, commitMessage: 'test: pin the contract' });
    if (key === 'implement' || key === 'fix')
      return JSON.stringify({
        testsNeedUpdate: false,
        commitMessage: key === 'fix' ? 'fix: address the findings' : 'feat: implement it',
      });
    if (key === 'doc-sync') return JSON.stringify({ drifted: false });
    return JSON.stringify({ summary: 'ok' });
  };
  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        created[key] = (created[key] ?? 0) + 1;
        const id = `ses_${String(Object.values(created).reduce((a, b) => a + b, 0))}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string }) => {
        const s = sessions.get(input.sessionID);
        if (s !== undefined) {
          s.prompts += 1;
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
  return { client };
}

function resumeOptionsFor(dir: string, client: OpenCodeClient): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: FAST,
    client,
  };
}

describe('the finalize hook (finalizeDone)', () => {
  // proves: Test strategy #12 — finalize escalates rather than flipping the
  // ADR status when the run's own ledger carries an unhandled unit: the source
  // ADR stays `status: proposed`, the escalation marker exists, and the denial
  // is gate-logged under `ledger-join`.
  it('escalates instead of flipping the ADR status on an unhandled unit', async () => {
    const dir = funnelRepo(RED_ADR);
    const { client } = funnelStub();

    const { status } = await resumePipeline(resumeOptionsFor(dir, client));

    expect(status).toBe('escalated');
    expect(readFileSync(join(dir, 'docs', 'adrs', '002-x.md'), 'utf8')).toContain(
      'status: proposed',
    );
    expect(existsSync(join(dir, 'docs', 'designs', '002-x', 'ESCALATION.md'))).toBe(true);
    const denied = gateLog(dir, join('docs', 'designs', '002-x')).filter(
      (g) => g.gate === 'ledger-join' && g.proceed === false,
    );
    expect(denied.length).toBeGreaterThanOrEqual(1);
    expect(denied.map((g) => g.reason ?? '').join('\n')).toContain('unit-unhandled');
  });

  // proves: Test strategy #12 — and flips it when the tree is green: 021-R2's
  // machine-verified acceptance — a run whose units all resolve completes.
  it('flips the ADR status to accepted when the tree conforms', async () => {
    const dir = funnelRepo(GREEN_ADR);
    const { client } = funnelStub();

    const { status } = await resumePipeline(resumeOptionsFor(dir, client));

    expect(status).toBe('done');
    expect(readFileSync(join(dir, 'docs', 'adrs', '002-x.md'), 'utf8')).toContain(
      'status: accepted',
    );
    const entries = gateLog(dir, join('docs', 'designs', '002-x')).filter(
      (g) => g.gate === 'ledger-join',
    );
    expect(entries.some((g) => g.proceed === true)).toBe(true);
  });
});
