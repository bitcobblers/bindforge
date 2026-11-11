import { mkdtemp, readFile, readdir, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { loadState } from '../../src/pipeline/state-store.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-006 §2, the load side of the cross-version resume contract: every
// checkpoint shape that exists in the wild must load, unchanged, under
// current runner code — `version` stays 1 and new fields are additive-
// optional. The golden fixtures in fixtures/checkpoints/ pin the era shapes:
// the run-1 file (no stageCommits, no baseRef, no seats, results with only
// the original keys), the run-2-era file (still no stageCommits; a failed
// attempt persisted without a reply survives), and the live run-003
// checkpoint (seats, lastReviewPass, reply-less failure records). A future
// breaking field change must keep these loading or move to version 2 with a
// reader per era at the state-store version seam.
// ---------------------------------------------------------------------------

const FIXTURES = fileURLToPath(new URL('./fixtures/checkpoints/', import.meta.url));

async function loadFixture(name: string): Promise<{ file: PipelineState; loaded: PipelineState }> {
  const raw = await readFile(join(FIXTURES, name), 'utf8');
  const dir = await mkdtemp(join(tmpdir(), 'bindforge-compat-'));
  await writeFile(join(dir, '.pipeline.json'), raw, 'utf8');
  const loaded = await loadState(dir);
  if (loaded === null) {
    throw new Error(`fixture ${name} loaded as null`);
  }
  return { file: JSON.parse(raw) as PipelineState, loaded };
}

describe('checkpoint compatibility fixtures', () => {
  // proves: Test strategy #3 — every fixture in fixtures/checkpoints/ loads
  // via loadState unchanged: the loaded state carries exactly the file's
  // content, at schema version 1.
  it('loads every golden checkpoint fixture unchanged', async () => {
    const names = (await readdir(FIXTURES)).filter((f) => f.endsWith('.json')).sort();
    expect(names).toEqual(['run-1-shape.json', 'run-2-era.json', 'run-3-live.json']);
    for (const name of names) {
      const { file, loaded } = await loadFixture(name);
      expect(loaded).toEqual(file);
      expect(loaded.version).toBe(1);
    }
  });

  // proves: Test strategy #3 — run-1 shape: fields a newer runner added are
  // simply absent, and absence loads as absence (the additive-optional rule;
  // no post-run-1 field defaults into existence on read).
  it('loads the run-1 shape with every post-run-1 field absent', async () => {
    const { loaded } = await loadFixture('run-1-shape.json');
    expect(loaded.stageCommits).toBeUndefined();
    expect(loaded.baseRef).toBeUndefined();
    expect(loaded.seats).toBeUndefined();
    expect(loaded.lastReviewPass).toBeUndefined();
    for (const result of loaded.results) {
      expect(result.durationMs).toBeUndefined();
      expect(result.nudges).toBeUndefined();
    }
  });

  // proves: Test strategy #3 — run-2 era: no stageCommits, and a failed
  // attempt persisted without a reply still loads (making `reply` required
  // once made run 003 unresumable — this fixture keeps that scar pinned).
  it('loads the run-2-era checkpoint with its reply-less failure record', async () => {
    const { loaded } = await loadFixture('run-2-era.json');
    expect(loaded.stageCommits).toBeUndefined();
    expect(loaded.baseRef).toBeUndefined();
    const failed = loaded.results.filter((r) => r.failure !== undefined);
    expect(failed).toHaveLength(1);
    expect(failed[0].reply).toBeUndefined();
    expect(failed[0].failure).toContain('invalid reply');
  });

  // proves: Test strategy #3 — the live run-003 checkpoint with seats,
  // lastReviewPass, and reply-less failure records loads unchanged.
  it('loads the live run-003 checkpoint with seats and reply-less failures', async () => {
    const { loaded } = await loadFixture('run-3-live.json');
    expect(Object.keys(loaded.seats?.design ?? {})).toHaveLength(5);
    expect(Object.keys(loaded.seats?.code ?? {})).toHaveLength(5);
    expect(loaded.lastReviewPass?.phase).toBe('code');
    const failed = loaded.results.filter((r) => r.failure !== undefined && r.reply === undefined);
    expect(failed.length).toBeGreaterThan(0);
  });
});
