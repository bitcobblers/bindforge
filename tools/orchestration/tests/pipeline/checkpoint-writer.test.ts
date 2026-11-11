import { mkdir, mkdtemp, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  CheckpointWriteFailure,
  createStateWriter,
  loadState,
  saveState,
} from '../../src/pipeline/state-store.js';
import type { PipelineState } from '../../src/pipeline/types.js';

// ---------------------------------------------------------------------------
// ADR-003 §Self-update: every checkpoint write made from inside a concurrent
// review pass goes through ONE in-process write path — a one-at-a-time chain
// over saveState keyed to enqueue order — drained before pass finalize. Two
// racing tmp+rename writers can land the OLDER snapshot last (lost reviewer
// record, resume replays settled work), and a stray `.pipeline.json.*.tmp`
// would sit where the pass's `git add -A` sweeps it.
// ---------------------------------------------------------------------------

function base(updatedAt: number): PipelineState {
  return {
    version: 1,
    slug: 'feat',
    nnn: '002',
    branch: 'pipeline/002-feat',
    source: { kind: 'adr', path: 'docs/adrs/002-feat.md' },
    status: 'running',
    lastStage: 'design-review',
    designRound: 1,
    codeRound: 0,
    designBudgetUsed: 1,
    codeBudgetUsed: 0,
    editorialDone: false,
    testsGreen: false,
    testsNeedUpdate: false,
    stageFailures: {},
    forceNext: null,
    results: [],
    gateLog: [],
    startedAt: 1,
    updatedAt,
  };
}

async function tmpDir(): Promise<string> {
  return mkdtemp(join(tmpdir(), 'bindforge-writer-'));
}

describe('drained checkpoint writer', () => {
  // proves: Test strategy #7 — concurrent enqueue calls land in enqueue order:
  // after drain(), the file on disk is the newest snapshot, not whichever
  // tmp+rename happened to win the race
  it('lands three concurrent enqueues in enqueue order', async () => {
    const dir = await tmpDir();
    const writer = createStateWriter(dir);

    // Enqueued back to back without awaiting: the writer, not the filesystem,
    // decides the landing order.
    void writer.enqueue(base(1000));
    void writer.enqueue(base(2000));
    void writer.enqueue(base(3000));
    await writer.drain();

    const loaded = await loadState(dir);
    expect(loaded?.updatedAt).toBe(3000);
  });

  // proves: Test strategy #7 — a slower first write cannot land last: the
  // first enqueue's promise marks the point where its snapshot is the one on
  // disk, and the later write has not overwritten it yet
  it('keeps the first snapshot authoritative until the next one is written', async () => {
    const dir = await tmpDir();
    const writer = createStateWriter(dir);

    const first = writer.enqueue(base(1000));
    void writer.enqueue(base(2000));

    await first;
    // Writes are one at a time: while the first was still in flight, the
    // second could not start, so the first rename is what is on disk.
    expect((await loadState(dir))?.updatedAt).toBe(1000);

    await writer.drain();
    expect((await loadState(dir))?.updatedAt).toBe(2000);
  });

  // proves: Test strategy #7 — drain() resolves only once no checkpoint write
  // is in flight, so no half-written `.pipeline.json.*.tmp` can sit in the
  // design dir when the pass's `git add -A` runs
  it('leaves no tmp file in flight after drain', async () => {
    const dir = await tmpDir();
    const writer = createStateWriter(dir);

    void writer.enqueue(base(1000));
    void writer.enqueue(base(2000));
    await writer.drain();

    const files = await readdir(dir);
    expect(files.filter((f) => f.includes('.tmp'))).toEqual([]);
    expect(files).toEqual(['.pipeline.json']);
  });

  // proves: Test strategy #7 — the writer writes through the same validating
  // saveState path (a corrupted state is refused, nothing is written)
  it('refuses to write an invalid checkpoint through the queue', async () => {
    const dir = await tmpDir();
    const writer = createStateWriter(dir);
    await saveState(dir, base(1000)); // a valid baseline to compare against

    const broken = { ...base(2000), results: 'not-an-array' } as unknown as PipelineState;
    await expect(writer.enqueue(broken)).rejects.toThrow(
      /refusing to write an invalid pipeline checkpoint/,
    );
    await writer.drain();

    const loaded = await loadState(dir);
    expect(loaded?.updatedAt).toBe(1000); // untouched
  });
});

describe('checkpoint write failures', () => {
  // proves: Test strategy #6 — a write-path failure (a directory sits at the
  // checkpoint path) surfaces as CheckpointWriteFailure, not a bare errno, so
  // the drive loop can tell an escalation-worthy failure from a crash
  it('wraps a write-path failure in CheckpointWriteFailure', async () => {
    const dir = await tmpDir();
    await mkdir(join(dir, '.pipeline.json')); // the directory-replacement posture

    const err = await saveState(dir, base(1000)).then(
      () => undefined,
      (reason: unknown) => reason,
    );
    expect(err).toBeInstanceOf(CheckpointWriteFailure);
  });

  // proves: Test strategy #6 — one failed write does not wedge the queue: the
  // failed enqueue rejects its caller, and once the path is repaired the
  // writer keeps landing later writes
  it('keeps landing writes after a write-path failure', async () => {
    const dir = await tmpDir();
    const writer = createStateWriter(dir);

    await mkdir(join(dir, '.pipeline.json'));
    const err = await writer.enqueue(base(1000)).then(
      () => undefined,
      (reason: unknown) => reason,
    );
    expect(err).toBeInstanceOf(CheckpointWriteFailure);

    await rm(join(dir, '.pipeline.json'), { recursive: true }); // operator repair
    await writer.enqueue(base(2000));
    await writer.drain();

    const loaded = await loadState(dir);
    expect(loaded?.updatedAt).toBe(2000);
  });

  // proves: Test strategy #6 — a refused invalid snapshot rejects only its
  // caller (§3 chain semantics): a later enqueue queued behind it still lands
  it('lets the queue land a later write after a refused invalid snapshot', async () => {
    const dir = await tmpDir();
    const writer = createStateWriter(dir);

    const broken = { ...base(1000), gateLog: 'not-an-array' } as unknown as PipelineState;
    const refused = writer.enqueue(broken);
    void writer.enqueue(base(2000));

    await expect(refused).rejects.toThrow(/refusing to write an invalid pipeline checkpoint/);
    await writer.drain();

    const loaded = await loadState(dir);
    expect(loaded?.updatedAt).toBe(2000);
  });
});
