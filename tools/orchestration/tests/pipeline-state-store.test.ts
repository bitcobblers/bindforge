import { mkdtemp, readdir, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { deriveIdentity } from '../src/pipeline/identity.js';
import { designDirOf, loadState, saveState } from '../src/pipeline/state-store.js';
import type { PipelineState } from '../src/pipeline/types.js';

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
