import { describe, expect, it } from 'vitest';

import { createSemaphore } from '../../src/pipeline/concurrency.js';

// ---------------------------------------------------------------------------
// ADR-003's ~20-line bounded-concurrency primitive for the review-pass fan-out:
// a running counter plus a FIFO waiter queue, no new dependency. The lane in
// `drive` depends on exactly three properties: never more than N concurrent
// permits, waiters woken in the order the seats were handed, and a release
// waking the oldest waiter. (src/pipeline/concurrency.ts in isolation.)
// ---------------------------------------------------------------------------

const tick = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

describe('createSemaphore', () => {
  // proves: Test strategy #1 — never more than N concurrent permits
  it('runs at most limit acquisitions concurrently across a ten-seat burst', async () => {
    const sem = createSemaphore(3);
    let inFlight = 0;
    let observed = 0;

    const tasks = Array.from({ length: 10 }, async () => {
      const release = await sem.acquire();
      inFlight += 1;
      observed = Math.max(observed, inFlight);
      await tick();
      await tick();
      inFlight -= 1;
      release();
    });
    await Promise.all(tasks);

    // Exactly 3: the cap binds (no more than N) and every permit is used (a
    // semaphore that over-serialises fails the fan-out just as hard).
    expect(observed).toBe(3);
  });

  // proves: Test strategy #1 — FIFO waiter order; a release wakes the oldest waiter
  it('wakes queued waiters in the order they arrived', async () => {
    const sem = createSemaphore(1);
    const woken: number[] = [];

    const held = await sem.acquire();
    const waiters = [0, 1, 2, 3, 4].map(async (k) => {
      const release = await sem.acquire();
      woken.push(k);
      release();
    });
    await tick(); // every waiter is now in the queue
    expect(woken).toEqual([]); // blocked: no permit free

    held();
    await Promise.all(waiters);

    expect(woken).toEqual([0, 1, 2, 3, 4]);
  });

  // proves: Test strategy #1 — acquire blocks while the limit is held, and the
  // release function returns the permit to the pool
  it('holds a late acquirer until a permit is released back', async () => {
    const sem = createSemaphore(1);
    const first = await sem.acquire();

    let secondAcquired = false;
    const second = sem.acquire().then((release) => {
      secondAcquired = true;
      return release;
    });

    await tick();
    expect(secondAcquired).toBe(false);

    first();
    const secondRelease = await second;
    expect(secondAcquired).toBe(true);
    secondRelease();
  });
});
