import { defineConfig } from 'vitest/config';

// ---------------------------------------------------------------------------
// This suite drives real repositories: each file builds a scratch repo under
// /tmp and runs git inside it, so its tests are process-bound rather than CPU-
// bound. With no configuration Vitest defaults to a 5s per-test timeout and one
// isolated worker per file — 54 workers at once on this host. Under load the
// slowest integration tests crossed 5s and the suite failed for reasons that
// had nothing to do with the change under test: the post-merge gate for the
// 020 run went red exactly that way, 25 timeouts, while the same tests passed
// green twice inside the run minutes earlier.
//
// The timeouts below are sized for a loaded machine, and `hookTimeout` covers
// the scratch-repo cleanup in tests/pipeline/tmp-dir.ts, which deletes those
// trees recursively when a file's suite finishes. The worker cap exists so a
// 54-way process storm is never itself what loads the box: the suite is small
// enough that four workers cost about four seconds of wall time.
// ---------------------------------------------------------------------------

export default defineConfig({
  test: {
    testTimeout: 20_000,
    hookTimeout: 20_000,
    maxWorkers: 8,
  },
});
