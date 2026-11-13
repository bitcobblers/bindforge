import { readFile, readdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { loadState, saveState } from '../../src/pipeline/state-store.js';
import type { GateEvidence, PipelineState } from '../../src/pipeline/types.js';
import { mkTmpDirAsync } from './tmp-dir.js';

// ---------------------------------------------------------------------------
// ADR-016 Decision 2, checkpoint-grammar side. The run gains ONE new field,
// `gateEvidence`, and it obeys the shipped cross-version rule (A6): added after
// run 1, so it is optional with a behavior-safe default, nothing is removed or
// retyped, and the loader still sizes the file against `MAX_CHECKPOINT_BYTES`
// before parsing. A state that carries the record parses; every golden
// checkpoint fixture — written before the field existed — still parses, with no
// `gateEvidence` materialised out of absence.
// ---------------------------------------------------------------------------

const FIXTURES = fileURLToPath(new URL('./fixtures/checkpoints/', import.meta.url));

const base: PipelineState = {
  version: 1,
  slug: 'feat',
  nnn: '016',
  branch: 'pipeline/016-feat',
  source: { kind: 'adr', path: 'docs/adrs/016-feat.md' },
  status: 'running',
  lastStage: 'implement',
  designRound: 1,
  codeRound: 1,
  designBudgetUsed: 1,
  codeBudgetUsed: 1,
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

const EVIDENCE: GateEvidence = {
  at: 1_000_000,
  green: false,
  legs: [
    {
      command: 'cargo test --workspace --locked',
      cwd: '/repo',
      exit: 101,
      tail: 'test result: FAILED. 4 passed; 1 failed',
    },
    { command: 'cargo fmt --all -- --check', cwd: '/repo', exit: 0, tail: '' },
  ],
};

describe('gateEvidence in the checkpoint grammar (ADR-016 Decision 2)', () => {
  // proves: Test strategy #2 — a state carrying `gateEvidence` round-trips
  // through the loader with its legs, exits and tails intact.
  it('round-trips a state carrying gateEvidence', async () => {
    const dir = await mkTmpDirAsync('bindforge-gate-evidence-');
    const withEvidence = { ...base, gateEvidence: EVIDENCE } as PipelineState;
    await saveState(dir, withEvidence);

    const loaded = await loadState(dir);
    expect(loaded).not.toBeNull();
    expect(loaded?.gateEvidence?.green).toBe(false);
    expect(loaded?.gateEvidence?.legs).toHaveLength(2);
    expect(loaded?.gateEvidence?.legs[0]?.command).toBe('cargo test --workspace --locked');
    expect(loaded?.gateEvidence?.legs[0]?.exit).toBe(101);
    expect(loaded?.gateEvidence?.legs[0]?.tail).toContain('test result: FAILED');
  });

  // proves: Test strategy #2 — every existing golden checkpoint fixture parses
  // with no `gateEvidence` key: the additive-optional rule (A6), absence loads
  // as absence, nothing is invented at the read seam.
  it('loads every golden checkpoint fixture with no gateEvidence materialised', async () => {
    const names = (await readdir(FIXTURES)).filter((f) => f.endsWith('.json')).sort();
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) {
      const raw = await readFile(join(FIXTURES, name), 'utf8');
      const dir = await mkTmpDirAsync('bindforge-gate-compat-');
      await writeFile(join(dir, '.pipeline.json'), raw, 'utf8');
      const loaded = await loadState(dir);
      expect(loaded, name).not.toBeNull();
      expect(loaded?.gateEvidence, name).toBeUndefined();
    }
  });

  // proves: Test strategy #13 — a run resumed from a pre-016 checkpoint carries
  // no `gateEvidence`; the absence round-trips unchanged so the resume can drive
  // a stage against it. A state without the field still saves and loads.
  it('round-trips a pre-016 state with the field simply absent', async () => {
    const dir = await mkTmpDirAsync('bindforge-gate-absent-');
    await saveState(dir, base);
    const loaded = await loadState(dir);
    expect(loaded?.gateEvidence).toBeUndefined();
  });
});
