import { describe, expect, it } from 'vitest';

import config from '../../vitest.config.js';

// ---------------------------------------------------------------------------
// The suite's timeouts and worker cap are not cosmetics: an unconfigured run
// defaults to a 5s per-test timeout and one isolated worker per file, and the
// post-merge gate for the 020 run went red under exactly that shape — 25 tests
// timed out on a loaded box while the same tests had passed inside the run
// minutes earlier. Nothing else in the build fails if vitest.config.ts is
// deleted or its numbers are trimmed, so this pins the floor that keeps a busy
// host from being reported as a broken change.
// ---------------------------------------------------------------------------

const test = config.test ?? {};

describe('the test runner posture (vitest.config.ts)', () => {
  // proves: a loaded host cannot push an integration test past the timeout and
  // have the failure read as a regression in the change under review.
  it('gives process-bound tests real time to run', () => {
    expect(test.testTimeout).toBeGreaterThanOrEqual(15_000);
    expect(test.hookTimeout).toBeGreaterThanOrEqual(15_000);
  });

  // proves: the suite never opens one worker per test file on a shared box,
  // which is how the suite itself became the source of the load it failed under.
  it('caps concurrent workers below the file count', () => {
    expect(test.maxWorkers).toBeLessThanOrEqual(8);
  });
});
