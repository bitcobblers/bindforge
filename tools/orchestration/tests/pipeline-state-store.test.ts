import { mkdtemp, readdir, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

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
