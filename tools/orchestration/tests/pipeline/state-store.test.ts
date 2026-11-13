import { mkdir, mkdtemp, readFile, readdir, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { deriveIdentity } from '../../src/pipeline/identity.js';
import { designDirOf, loadState, saveState } from '../../src/pipeline/state-store.js';
import type { PipelineState } from '../../src/pipeline/types.js';

const base: PipelineState = {
  version: 1,
  slug: 'feat',
  nnn: '002',
  branch: 'pipeline/002-feat',
  source: { kind: 'adr', path: 'docs/adrs/002-feat.md' },
  status: 'running',
  lastStage: 'research',
  designRound: 0,
  codeRound: 0,
  designBudgetUsed: 0,
  codeBudgetUsed: 0,
  editorialDone: false,
  testsGreen: false,
  testsNeedUpdate: false,
  stageFailures: {},
  forceNext: null,
  results: [],
  gateLog: [],
  startedAt: 1,
  updatedAt: 1,
};

describe('state store', () => {
  it('round-trips a checkpoint', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-state-'));
    await saveState(dir, base);
    const loaded = await loadState(dir);
    expect(loaded).not.toBeNull();
    expect(loaded?.slug).toBe('feat');
    expect(loaded?.lastStage).toBe('research');
  });

  it('returns null when no checkpoint exists', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-state-'));
    expect(await loadState(dir)).toBeNull();
  });

  it('parses a checkpoint written before the proposal size meter', async () => {
    // Run 002-run2's own checkpoint: results with no size fields. A resume must
    // not fail on the fields the fix adds.
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-state-'));
    const run2 = structuredClone(base);
    run2.results = [
      {
        stage: 'proposal-revise',
        round: 8,
        sessionID: 'ses_old',
        agent: 'design/designer',
        at: 1,
        reply: { changed: true },
        transcript: 'docs/designs/002-run2/transcripts/proposal-revise-r8.md',
        durationMs: 1000,
        nudges: 0,
      },
    ];
    await writeFile(join(dir, '.pipeline.json'), JSON.stringify(run2), 'utf8');
    const loaded = await loadState(dir);
    expect(loaded?.results[0]?.docLines).toBeUndefined();
  });

  it('round-trips the proposal size meter', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-state-'));
    const sized = structuredClone(base);
    sized.results = [
      {
        stage: 'proposal-revise',
        round: 3,
        sessionID: 'ses_new',
        agent: 'design/designer',
        at: 1,
        reply: { changed: true },
        transcript: 'docs/designs/002-feat/transcripts/proposal-revise-r3.md',
        docLines: 660,
        docLinesAdded: 300,
        docLinesRemoved: 41,
      },
    ];
    await saveState(dir, sized);
    const loaded = await loadState(dir);
    expect(loaded?.results[0]?.docLines).toBe(660);
    expect(loaded?.results[0]?.docLinesAdded).toBe(300);
    expect(loaded?.results[0]?.docLinesRemoved).toBe(41);
  });

  it('round-trips a failed attempt stored without a reply', async () => {
    // Run 003 became unresumable this way: a failed attempt has
    // `reply: undefined`, JSON.stringify drops the key, and a required
    // `reply` in the persisted schema rejected the run's own checkpoint.
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-state-'));
    const failed = structuredClone(base);
    failed.results = [
      {
        stage: 'test-authoring',
        round: 0,
        sessionID: 'ses_failed',
        agent: 'code/test-engineer',
        at: 1,
        // no reply: the attempt failed before one validated
        transcript: 'docs/designs/002-feat/transcripts/test-authoring-r0:attempt1.md',
        durationMs: 5200,
        nudges: 0,
        failure: 'independence violation: read a review report: …',
      },
    ];
    await saveState(dir, failed);
    const loaded = await loadState(dir);
    expect(loaded?.results[0]?.reply).toBeUndefined();
    expect(loaded?.results[0]?.failure).toContain('independence violation');
  });

  it('refuses unsupported state versions', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-state-'));
    await writeFile(join(dir, '.pipeline.json'), JSON.stringify({ version: 99 }), 'utf8');
    await expect(loadState(dir)).rejects.toThrow(/unsupported pipeline state version/);
  });

  it('leaves no temp files behind after saving', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-state-'));
    await saveState(dir, base);
    const files = await readdir(dir);
    expect(files.filter((f) => f.endsWith('.tmp'))).toEqual([]);
    expect(files).toEqual(['.pipeline.json']);
  });

  it('derives the design directory from nnn and slug', () => {
    expect(designDirOf(base)).toBe(join('docs', 'designs', '002-feat'));
  });
});

describe('checkpoint validation', () => {
  async function tmpDir(): Promise<string> {
    return mkdtemp(join(tmpdir(), 'bindforge-validate-'));
  }

  it('turns an interrupted write into a recoverable instruction', async () => {
    const dir = await tmpDir();
    // Exactly what a crash between write and rename leaves behind.
    await writeFile(join(dir, '.pipeline.json'), '{"version":1,"slug":"fea', 'utf8');
    await expect(loadState(dir)).rejects.toThrow(/checkpoint is not readable/);
    await expect(loadState(dir)).rejects.toThrow(/git checkout HEAD --/);
  });

  it('rejects an identity the run could never resolve to a branch or directory', async () => {
    for (const [field, value] of [
      ['slug', '../escape'],
      ['slug', 'Run Two'],
      ['nnn', '2'],
    ] as const) {
      const dir = await tmpDir();
      await writeFile(
        join(dir, '.pipeline.json'),
        JSON.stringify({ ...base, [field]: value }),
        'utf8',
      );
      await expect(loadState(dir)).rejects.toThrow(/checkpoint is invalid/);
      await expect(loadState(dir)).rejects.toThrow(new RegExp(field));
    }
  });

  it('reads a checkpoint written before the newer fields existed', async () => {
    const dir = await tmpDir();
    const run1 = {
      ...base,
      // run 1's shape: no stageCommits, no extensionRounds, no per-stage
      // duration/nudge record, results with only the original keys.
      results: [
        {
          stage: 'design-review:correctness',
          round: 9,
          sessionID: 'ses_x',
          agent: 'review/design-correctness',
          at: 1,
          reply: { verdict: 'approve', findings: [] },
          transcript: 'docs/designs/002-feat/transcripts/r9.md',
        },
      ],
    };
    await writeFile(join(dir, '.pipeline.json'), JSON.stringify(run1), 'utf8');
    const loaded = await loadState(dir);
    expect(loaded?.results).toHaveLength(1);
  });

  it('never writes a checkpoint it would refuse to read back', async () => {
    const dir = await tmpDir();
    const broken = { ...base, results: 'not-an-array' } as unknown as PipelineState;
    await expect(saveState(dir, broken)).rejects.toThrow(
      /refusing to write an invalid pipeline checkpoint/,
    );
  });

  it('accepts whatever the identity generator produces', async () => {
    // The compatibility finding run 1's own reviewer raised: a validator that
    // rejects a slug the generator emits makes a run unresumable.
    const repo = await mkdtemp(join(tmpdir(), 'bindforge-gen-'));
    for (const source of ['004-Some Brief!.md', '010-.._escape.md', '011-Über.md']) {
      const id = deriveIdentity(repo, join(repo, 'docs', 'prompts', source), {
        slug: undefined,
      });
      const dir = await tmpDir();
      await saveState(dir, { ...base, nnn: id.nnn, slug: id.slug, branch: id.branch });
      const loaded = await loadState(dir);
      expect(loaded?.slug).toBe(id.slug);
    }
  });
});

// ---------------------------------------------------------------------------
// ADR-006 §2/§3: the cross-version contract's refusal path and the replaced-
// path repair. A refused version names the supported version and the ADR's
// two interim operator postures (restore the committed copy, or start a
// fresh run) and touches nothing on disk; a directory at the checkpoint path
// is reported as the repair it needs, not as a bare errno.
// ---------------------------------------------------------------------------

describe('cross-version refusal and path repair messages', () => {
  async function tmpDir(): Promise<string> {
    return mkdtemp(join(tmpdir(), 'bindforge-refuse-'));
  }

  async function refusalMessage(dir: string): Promise<string> {
    try {
      await loadState(dir);
    } catch (error) {
      return error instanceof Error ? error.message : String(error);
    }
    throw new Error('expected loadState to refuse the checkpoint');
  }

  // proves: Test strategy #4 — version 0, 2, and 99 all refuse; the message
  // keeps today's stem, names the supported version and both operator
  // postures, and the file on disk is byte-identical after the refusal —
  // refusal never writes or deletes anything.
  it('refuses versions 0, 2, and 99 naming the supported version and both recovery postures', async () => {
    for (const version of [0, 2, 99]) {
      const dir = await tmpDir();
      const raw = JSON.stringify({ ...base, version });
      await writeFile(join(dir, '.pipeline.json'), raw, 'utf8');

      const message = await refusalMessage(dir);

      expect(message).toMatch(/unsupported pipeline state version/);
      expect(message).toContain('version 1');
      expect(message).toContain('git checkout HEAD --');
      expect(message).toMatch(/fresh run/i);

      expect(await readFile(join(dir, '.pipeline.json'), 'utf8')).toBe(raw);
      expect((await readdir(dir)).filter((f) => f.endsWith('.tmp'))).toEqual([]);
    }
  });

  // proves: Test strategy #5 — a directory occupying the checkpoint path is
  // loadState'd into the rm -r + git checkout HEAD -- repair instruction,
  // not a bare errno (A5: the replacement is not repairable inside the
  // write call).
  it('names the rm -r plus git checkout repair for a directory at the checkpoint path', async () => {
    const dir = await tmpDir();
    await mkdir(join(dir, '.pipeline.json'));
    await writeFile(join(dir, '.pipeline.json', 'planted.md'), 'x', 'utf8');

    const message = await refusalMessage(dir);

    expect(message).toContain('rm -r');
    expect(message).toContain('git checkout HEAD --');
  });

  // proves: ADR-006 §2 / security r1 #5 (deviation 12) — the raw read is
  // size-capped before any parse: the committed copy is the recovery anchor
  // for pre-fence history that §Risks admits can carry model-authored bytes,
  // so an oversized hostile checkpoint is refused with the same recovery
  // postures instead of spending unbounded memory in JSON.parse — and the
  // refusal stays read-only like every other one.
  it('refuses an oversized checkpoint before parsing, naming the recovery postures', async () => {
    const dir = await tmpDir();
    const huge = {
      ...base,
      gateLog: [
        { gate: 'x', round: 0, proceed: true, reason: 'a'.repeat(66 * 1024 * 1024), at: 0 },
      ],
    } as unknown as PipelineState;
    const raw = JSON.stringify(huge);
    await writeFile(join(dir, '.pipeline.json'), raw, 'utf8');

    const message = await refusalMessage(dir);

    expect(message).toMatch(/too large/i);
    expect(message).toContain('git checkout HEAD --');
    expect(message).toMatch(/fresh run/i);

    expect(await readFile(join(dir, '.pipeline.json'), 'utf8')).toBe(raw);
    expect((await readdir(dir)).filter((f) => f.endsWith('.tmp'))).toEqual([]);
  });
});

// ---------------------------------------------------------------------------
// ADR-020 §2 (design 020): `restartRequired` is the checkpoint's one additive-
// optional companion field — `status` untouched, `version` still 1, the ADR-006
// §2 grammar's sanctioned move. A checkpoint carrying the record writes and
// reads back; a record with a key the current schema does not name still loads
// (a later-era writer stays readable here, exactly as the pre-020 reader stays
// reading here — A7); and no era fixture gains the key by defaulting, the same
// absence rule as `flow`, `planStop` and `gateEvidence`.
// ---------------------------------------------------------------------------

interface RestartRecord {
  stage: string;
  paths: string[];
  at: number;
}
type StateWithBoundary = PipelineState & { restartRequired?: RestartRecord };

const CHECKPOINT_FIXTURES = fileURLToPath(new URL('./fixtures/checkpoints/', import.meta.url));

describe('the restartRequired checkpoint record', () => {
  // proves: Test strategy #2 — a checkpoint carrying restartRequired:
  // { stage, paths, at } writes and reads back with the record intact and
  // `status` untouched.
  it('round-trips a checkpoint carrying the restartRequired record', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-restart-record-'));
    const withRecord = {
      ...base,
      restartRequired: {
        stage: 'implement',
        paths: ['tools/orchestration/prompts/implement.md'],
        at: 1_700_000_000_000,
      },
    } as unknown as PipelineState;
    await saveState(dir, withRecord);
    const loaded = (await loadState(dir)) as StateWithBoundary | null;
    expect(loaded).not.toBeNull();
    expect(loaded?.status).toBe('running');
    expect(loaded?.restartRequired).toEqual({
      stage: 'implement',
      paths: ['tools/orchestration/prompts/implement.md'],
      at: 1_700_000_000_000,
    });
  });

  // proves: Test strategy #2 (A7) — a record carrying a key this schema does
  // not name still parses and loads, the unknown key inert to every consumer:
  // the known fields read exactly, and nothing else about the checkpoint moves.
  it('loads a record carrying an unrecognized sibling key with that key inert', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'bindforge-restart-record-'));
    await writeFile(
      join(dir, '.pipeline.json'),
      JSON.stringify({
        ...base,
        restartRequired: {
          stage: 'fix',
          paths: ['tools/orchestration/src/pipeline/gates.ts'],
          at: 2,
          crossReferencedBy: 'a later era',
        },
      }),
      'utf8',
    );
    const loaded = (await loadState(dir)) as StateWithBoundary | null;
    expect(loaded).not.toBeNull();
    expect(loaded?.slug).toBe('feat');
    expect(loaded?.status).toBe('running');
    expect(loaded?.restartRequired?.stage).toBe('fix');
    expect(loaded?.restartRequired?.paths).toEqual(['tools/orchestration/src/pipeline/gates.ts']);
    expect(loaded?.restartRequired?.at).toBe(2);
  });

  // proves: Test strategy #2 — the record defaults to absence, not to a
  // default: every golden era fixture in fixtures/checkpoints/ loads with no
  // restartRequired key materialised (the Rollout's "every existing checkpoint
  // reads as no boundary standing").
  it('loads every era fixture with no restartRequired key materialised', async () => {
    const names = (await readdir(CHECKPOINT_FIXTURES)).filter((f) => f.endsWith('.json')).sort();
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) {
      const dir = await mkdtemp(join(tmpdir(), 'bindforge-restart-era-'));
      const raw = await readFile(join(CHECKPOINT_FIXTURES, name), 'utf8');
      const file = JSON.parse(raw) as StateWithBoundary;
      expect(file.restartRequired, name).toBeUndefined();
      await writeFile(join(dir, '.pipeline.json'), raw, 'utf8');
      const loaded = (await loadState(dir)) as StateWithBoundary | null;
      expect(loaded, name).not.toBeNull();
      expect(loaded?.restartRequired, name).toBeUndefined();
      expect(Object.keys(loaded ?? {}).includes('restartRequired'), name).toBe(false);
    }
  });
});
