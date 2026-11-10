// ---------------------------------------------------------------------------
// The bounded-concurrency primitive behind ADR-003's review-pass fan-out: a
// running counter plus a FIFO waiter queue, ~20 lines, no dependency. The
// lane in `drive` depends on exactly three properties: never more than N
// concurrent permits, waiters woken in the order the seats were handed, and a
// release waking the oldest waiter. Node's single-threaded event loop makes
// the counter and queue non-tearing; races exist only at `await` boundaries,
// which this module keeps inside `acquire`.
// ---------------------------------------------------------------------------

export interface Semaphore {
  /** Resolves with this permit's release function; FIFO while the cap is held. */
  acquire(): Promise<() => void>;
}

export function createSemaphore(limit: number): Semaphore {
  if (!Number.isInteger(limit) || limit < 1) {
    throw new Error(`semaphore limit must be an integer >= 1, got ${String(limit)}`);
  }
  let available = limit;
  const waiters: Array<(release: () => void) => void> = [];

  const wake = (): void => {
    const next = waiters.shift();
    if (next) {
      // Hand the permit straight to the oldest waiter: never free + re-grab,
      // so the FIFO order of the waiter queue is the order of acquisition.
      next(finish);
    } else {
      available += 1;
    }
  };

  const finish = (): void => {
    wake();
  };

  const acquire = (): Promise<() => void> => {
    if (available > 0) {
      available -= 1;
      return Promise.resolve(finish);
    }
    return new Promise<() => void>((resolve) => {
      waiters.push(resolve);
    });
  };

  return { acquire };
}
