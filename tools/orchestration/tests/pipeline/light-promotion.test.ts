import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { OpenCodeClient, SessionMessageInfo } from '@opencode/client';
import { describe, expect, it } from 'vitest';

import { deriveIdentity, identityClash } from '../../src/pipeline/identity.js';
import {
  type ResumeOptions,
  type RunOptions,
  resumePipeline,
  runPipeline,
} from '../../src/pipeline/runner.js';
import { DEFAULT_CONFIG, type PipelineConfig } from '../../src/pipeline/stages.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-010 §Promotion: promotion is an operator action with no state migration.
// Set `flow: full` in the source document and re-run with `--force` on the same
// identity (A19): `identityClash` reports the light run's checkpoint and branch,
// `--force` proceeds, the full `proposal` stage rewrites `02-proposal.md`, and
// the light plan survives in the branch's git history (A26).
//
// `--force` is the entry's identity-clash bypass, so the story has to start from
// a source whose identity is stable — an ADR under `docs/adrs/`, whose `NNN`
// comes from its file name (identity.ts). A brief carries no `NNN`, so a brief's
// `flow: full` entry takes the counter's next free number: the checkpoint-identity
// rule is the light flow's alone (A17), which the allocation tests pin below.
// ---------------------------------------------------------------------------

const ADR = 'docs/adrs/002-x.md';
const DESIGN_DIR = 'docs/designs/002-x';
const PLAN_MARK = 'the light plan: one module, one test file';
const FULL_MARK = 'the full proposal: every section the design review reads';
const LIGHT_SEATS = ['correctness', 'testing'];
const CLASH_CHECKPOINT = `checkpoint exists at ${DESIGN_DIR}/.pipeline.json`;
const FORCE_HINT = 'pass --force to continue on the existing identity anyway';

const BASE: PipelineConfig = {
  ...DEFAULT_CONFIG,
  pollIntervalMs: 1,
  stageTimeoutMs: 5_000,
  requestTimeoutMs: 1_000,
};

function stateFor(partial: Partial<PipelineState> = {}): PipelineState {
  return {
    version: 1,
    slug: 'x',
    nnn: '002',
    branch: 'pipeline/002-x',
    source: { kind: 'adr', path: ADR },
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
    ...partial,
  };
}

/** Repo with the ADR source, a green `npm run check` gate, and one commit.
 *  `flow` is the source's frontmatter: the promotion under test is exactly the
 *  edit that changes it. */
function tmpRepo(opts: { flow: string; checkpoint?: PipelineState }): string {
  const dir = mkdtempSync(join(tmpdir(), 'bindforge-light-promotion-'));
  const git = (args: string[]) =>
    execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
  mkdirSync(join(dir, 'docs', 'adrs'), { recursive: true });
  writeSource(dir, opts.flow);
  if (opts.checkpoint !== undefined) {
    mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
    const checkpoint = `${JSON.stringify(opts.checkpoint, null, 2)}\n`;
    writeFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), checkpoint);
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

function writeSource(dir: string, flow: string): void {
  const text =
    `---\nflow: ${flow}\nstatus: proposed\n---\n\n` +
    `# ADR 002: x\n\n## Context\n\nctx\n\n## Decision\n\ndec\n`;
  writeFileSync(join(dir, ADR), text, 'utf8');
}

function entryOptions(dir: string, extra: object = {}): RunOptions {
  return {
    repoDir: dir,
    sourcePath: join(dir, ADR),
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: BASE,
    ...extra,
  };
}

function optionsFor(dir: string, extra: object = {}): ResumeOptions {
  return {
    repoDir: dir,
    slug: '002-x',
    promptsDir: fileURLToPath(new URL('../../prompts/', import.meta.url)),
    cfg: BASE,
    ...extra,
  };
}

/** Every stage does on disk what the real agent does: the light `plan` writes
 *  the plan at the proposal path, the full `proposal` rewrites that same path,
 *  a seat writes the report its prompt names, and every stage replies once. */
function stageStub(dir: string) {
  const sessionsCreated: string[] = [];
  const sessions = new Map<string, { key: string; prompts: number; lists: number }>();
  let clock = 0;
  const approve = JSON.stringify({ verdict: 'approve', findings: [] });
  const writeDoc = (text: string) =>
    writeFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), text, 'utf8');
  const client = {
    session: {
      create: async (input: { title?: string }) => {
        const key = (input.title ?? '').split(': ').slice(1).join(': ');
        sessionsCreated.push(key);
        const id = `ses_${String(sessionsCreated.length)}`;
        sessions.set(id, { key, prompts: 0, lists: 0 });
        return { id };
      },
      prompt: async (input: { sessionID: string; text?: string }) => {
        const s = sessions.get(input.sessionID);
        if (!s) return;
        s.prompts += 1;
        mkdirSync(join(dir, DESIGN_DIR), { recursive: true });
        if (s.key === 'plan') writeDoc(`# Plan\n\n${PLAN_MARK}\n`);
        if (s.key === 'proposal') writeDoc(`# Proposal\n\n${FULL_MARK}\n`);
        if (s.key === 'implement') {
          mkdirSync(join(dir, 'src'), { recursive: true });
          writeFileSync(join(dir, 'src', 'thing.ts'), 'export const thing = 1;\n', 'utf8');
        }
        if (s.key.includes('-review:')) {
          const found = /docs\/designs\/[^\s`]+\/reviews\/(design|code)\/[\w.-]+\.md/.exec(
            input.text ?? '',
          );
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
        const s = sessions.get(input.sessionID) ?? { key: '', prompts: 0, lists: 0 };
        s.lists += 1;
        const visible = Math.max(0, Math.min(s.lists - 1, s.prompts));
        const text = s.key.includes('-review:')
          ? approve
          : s.key === 'implement'
            ? JSON.stringify({ testsNeedUpdate: false, commitMessage: 'feat: implement it' })
            : s.key === 'test-authoring'
              ? JSON.stringify({ testsAdded: 1, commitMessage: 'test: pin the contract' })
              : s.key === 'proposal-final'
                ? JSON.stringify({ ambiguities: [] })
                : s.key === 'doc-sync'
                  ? JSON.stringify({ drifted: false })
                  : JSON.stringify({ summary: 'ok', commitMessage: `docs: ${s.key}` });
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

/** Client that records every call: a clean refusal calls it zero times. */
function countingClient() {
  const calls: string[] = [];
  const client = {
    session: {
      create: async (input: { title?: string }) => {
        calls.push(`create:${input.title ?? ''}`);
        return { id: `ses_${String(calls.length)}` };
      },
      prompt: async () => undefined,
      remove: async () => undefined,
      list: async () => ({ data: [], cursor: { previous: null, next: null } }),
    },
    message: {
      list: async () => {
        calls.push('message:list');
        return { data: [], cursor: { previous: null, next: null } };
      },
    },
  } as unknown as OpenCodeClient;
  return { client, calls };
}

function gitAt(dir: string, args: string[]): string {
  return execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], {
    cwd: dir,
    encoding: 'utf8',
  });
}

function savedState(dir: string): PipelineState {
  return JSON.parse(readFileSync(join(dir, DESIGN_DIR, '.pipeline.json'), 'utf8')) as PipelineState;
}

async function failureOf(promise: Promise<unknown>): Promise<string> {
  try {
    await promise;
    return '';
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
}

describe("promotion: --force on a light run's identity (A19, A26)", () => {
  // proves: Test strategy #22 — a full run with `--force` on a light run's
  // identity completes, `02-proposal.md` is the full proposal, and the light
  // plan is recoverable from the branch's git history.
  it('completes as a full run and keeps the light plan in the branch history', async () => {
    const dir = tmpRepo({ flow: 'light' });
    const light = stageStub(dir);
    expect((await runPipeline(entryOptions(dir, { client: light.client }))).status).toBe('done');
    expect(light.sessionsCreated.filter((k) => k.startsWith('code-review:'))).toEqual(
      LIGHT_SEATS.map((seat) => `code-review:${seat}`),
    );

    // The light run's own commit is the plan, on the run's branch.
    const planCommit = gitAt(dir, ['rev-parse', 'HEAD']).trim();
    expect(gitAt(dir, ['log', '--format=%s', 'pipeline/002-x'])).toContain('docs: plan');

    // The operator promotes: one frontmatter edit, then the `--force` re-entry.
    writeSource(dir, 'full');
    const full = stageStub(dir);
    const fullRun = await runPipeline(entryOptions(dir, { client: full.client, force: true }));
    expect(fullRun.status).toBe('done');

    const saved = savedState(dir);
    expect(saved.flow).toBe('full');
    expect(saved.nnn).toBe('002');

    // `02-proposal.md` is the full proposal: the full stage rewrote the path the
    // plan occupied, on the same identity (A26, accepted).
    const doc = readFileSync(join(dir, DESIGN_DIR, '02-proposal.md'), 'utf8');
    expect(doc).toContain(FULL_MARK);
    expect(doc).not.toContain(PLAN_MARK);

    // The light plan survives where the design says it does: the branch history.
    const history = gitAt(dir, ['show', `${planCommit}:${DESIGN_DIR}/02-proposal.md`]);
    expect(history).toContain(PLAN_MARK);
    expect(gitAt(dir, ['log', '--name-only', '--format=', 'pipeline/002-x'])).toContain(
      '02-proposal.md',
    );

    // The promoted run is the full flow: the design panel it skipped ran, and so
    // did the doc-sync stage the light flow does not have.
    expect(full.sessionsCreated.some((k) => k.startsWith('design-review:'))).toBe(true);
    expect(full.sessionsCreated).toContain('doc-sync');
  });

  // proves: Test strategy #22 (A19) — `--force` is the identity-clash bypass, and
  // without it the promoted entry names the light run's checkpoint and starts
  // nothing: no session, no new design directory beside the plan.
  it('refuses the promoted entry without --force, naming the checkpoint', async () => {
    const dir = tmpRepo({ flow: 'light' });
    const light = stageStub(dir);
    expect((await runPipeline(entryOptions(dir, { client: light.client }))).status).toBe('done');

    writeSource(dir, 'full');
    const { client, calls } = countingClient();
    const message = await failureOf(runPipeline(entryOptions(dir, { client })));
    expect(message).toContain(CLASH_CHECKPOINT);
    expect(message).toContain(FORCE_HINT);

    expect(calls).toEqual([]);
    expect(existsSync(join(dir, 'docs', 'designs', '003-x'))).toBe(false);
    const saved = savedState(dir);
    expect(saved.flow).toBe('light');
    expect(saved.status).toBe('done');
  });

  // proves: Test strategy #22 (the Risks row: the two flow records can disagree)
  // — the checkpoint is authoritative for a run in flight: a resume of the light
  // run keeps the light panel after the source says `flow: full`, so promotion
  // is the entry, never a mid-run migration.
  it('keeps a light run in flight light after the source is promoted', async () => {
    const checkpoint = stateFor({ status: 'running', lastStage: '' });
    const dir = tmpRepo({ flow: 'light', checkpoint });
    writeSource(dir, 'full');
    const { client, sessionsCreated } = stageStub(dir);

    expect((await resumePipeline(optionsFor(dir, { client }))).status).toBe('done');
    expect(savedState(dir).flow).toBe('light');
    expect(sessionsCreated.filter((k) => k.startsWith('design-review:'))).toEqual([]);
    expect(sessionsCreated.filter((k) => k.startsWith('code-review:'))).toEqual(
      LIGHT_SEATS.map((seat) => `code-review:${seat}`),
    );
  });
});

describe("identity allocation stays the full flow's (A17)", () => {
  // The checkpoint-identity rule in `deriveIdentity` is scoped to a light entry,
  // so a brief's full entry keeps A17's next-free number: the light flow gains
  // its re-entry rule and the shared counter is untouched.

  /** A repo where a brief already drives `003-light-thing`, branch and all. */
  function briefRepo(): string {
    const dir = mkdtempSync(join(tmpdir(), 'bindforge-light-identity-'));
    const git = (args: string[]) =>
      execFileSync('git', ['-c', 'user.name=f', '-c', 'user.email=f@f', ...args], { cwd: dir });
    mkdirSync(join(dir, 'docs', 'prompts'), { recursive: true });
    mkdirSync(join(dir, 'docs', 'designs', '003-light-thing'), { recursive: true });
    writeFileSync(
      join(dir, 'docs', 'prompts', 'light-thing.md'),
      '---\nflow: light\n---\n\nDo it.\n',
      'utf8',
    );
    const checkpoint = stateFor({
      nnn: '003',
      slug: 'light-thing',
      branch: 'pipeline/003-light-thing',
      source: { kind: 'brief', path: 'docs/prompts/light-thing.md' },
    });
    writeFileSync(
      join(dir, 'docs', 'designs', '003-light-thing', '.pipeline.json'),
      `${JSON.stringify(checkpoint, null, 2)}\n`,
      'utf8',
    );
    writeFileSync(
      join(dir, 'package.json'),
      '{"name":"t","scripts":{"check":"node -e \\"process.exit(0)\\""}}\n',
    );
    git(['init', '-q', '-b', 'main']);
    git(['add', '-A']);
    git(['commit', '-q', '-m', 'docs: base']);
    git(['branch', 'pipeline/003-light-thing']);
    return dir;
  }

  /** The brief the light run was entered from: a path the runner hands on, so
   *  absolute, with the repo-relative path in the checkpoint's `source.path`. */
  function briefSource(dir: string, name = 'light-thing.md'): string {
    return join(dir, 'docs', 'prompts', name);
  }

  // proves: Test strategy #22 (A17, deviation 4) — the light re-entry lands on
  // the identity that holds the plan, not beside it: the design directory whose
  // checkpoint names the document IS that document's run.
  it('re-enters a light run at the identity its checkpoint records', () => {
    const dir = briefRepo();
    const id = deriveIdentity(dir, briefSource(dir), {}, 'light');
    expect(id).toEqual({
      slug: 'light-thing',
      nnn: '003',
      kind: 'brief',
      branch: 'pipeline/003-light-thing',
    });
  });

  // proves: Test strategy #22 (A17) — the same brief entered as `flow: full`
  // allocates exactly as it always did: the counter's next free number, so the
  // promoted brief opens a new identity beside the light run's.
  it("keeps the full flow's next-free allocation for a brief", () => {
    const dir = briefRepo();
    expect(deriveIdentity(dir, briefSource(dir), {}, 'full').nnn).toBe('004');
  });

  // proves: Test strategy #22 (A19) — `identityClash` reports the light run's
  // checkpoint and branch, the two names `--force` is asked to override.
  it('reports the checkpoint and the branch it is about to reuse', () => {
    const dir = briefRepo();
    const reasons = identityClash(dir, deriveIdentity(dir, briefSource(dir), {}, 'light')).join(
      '\n',
    );
    expect(reasons).toContain('docs/designs/003-light-thing/.pipeline.json');
    expect(reasons).toContain('pipeline/003-light-thing');
  });

  // proves: Test strategy #22 (A17) — a light entry on a document that drives no
  // run yet takes the shared counter's number, so a light run consumes an `NNN`
  // from the same counter a decision does.
  it('allocates the next free number for a first light entry', () => {
    const dir = briefRepo();
    writeFileSync(
      join(dir, 'docs', 'prompts', 'other-thing.md'),
      '---\nflow: light\n---\n\nDo it.\n',
      'utf8',
    );
    const id = deriveIdentity(dir, briefSource(dir, 'other-thing.md'), {}, 'light');
    expect(id.slug).toBe('other-thing');
    expect(id.nnn).toBe('004');
  });
});
