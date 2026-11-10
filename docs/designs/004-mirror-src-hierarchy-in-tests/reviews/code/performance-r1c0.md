# Performance review — 004 mirror-src-hierarchy-in-tests (r1c0)

Seat: performance. Range: `d5092f3...HEAD -- ':(exclude)docs/designs'`.
Inputs: that diff, `02-proposal.md`, `docs/adrs/004-mirror-src-hierarchy-in-tests.md`.
Prior findings: none (first pass at this seat).

## What the diff changes, perf-wise

The diff touches no file under `tools/orchestration/src/` — only the 23 test
renames with depth rewrites, two new test blocks (`gates.test.ts`,
`prompts.test.ts`), and two skill-file convention lines. No runtime code path
is added or moved, so the hot-path items below are evaluated as "nothing to
regress," and the only new execution cost is in the test tree.

## Checklist

1. **Event/keystroke path — pass.** No event/keystroke or fs-on-input path
   exists in the diff; `src/` is untouched (verified from the diff file list).
   The only new synchronous calls are `execFileSync('git', ...)` in the two
   new test blocks (`gates.test.ts:369-370`, `prompts.test.ts:316`) — test
   harness, not a product path. See nit 1 for the duplicated spawns.
2. **Complexity — pass.** All new loops are bounded by fixed sets: 26 tracked
   test files (`MOVED_NAMES` 23 + 3 root), ≤31 files scanned by the
   flat-glob grep in `prompts.test.ts:335-345`, and a 3×23 `not.toContain`
   loop (`gates.test.ts:394`). No O(n²) over an unbounded n; no
   identity-lookup-as-array-scan pattern that a map/set would fix — the
   set sizes make it moot.
3. **Unbounded growth — pass.** No cache, registry, or accumulating array is
   introduced anywhere in the diff.
4. **Startup — pass.** Nothing on any boot path changes; the added work runs
   only under vitest, and the recursive `readdir`/`readFile` scan in
   `prompts.test.ts:335-345` walks a bounded ~31-file, all-markdown set.
5. **Cache invalidation — n/a.** No cache is added, so there is nothing to
   invalidate or to cover.
6. **Benchmark-like checks — n/a; nothing to run.** The diff contains no
   timing assertions, benchmark tests, or timing notes, so there is no
   number to quote. (Attempting `npx vitest run` to time the new suites was
   blocked by sandbox permissions; with no benchmark in the diff this does
   not gate the checklist.)

## Findings

1. **nit — `tools/orchestration/tests/pipeline/gates.test.ts:368-371,381,393`
   (and `prompts.test.ts:315-316,324,337`): the tracked-file listing and repo
   root are recomputed with fresh synchronous git subprocess spawns per test.**
   `trackedTestFiles()` spawns two `execFileSync` git processes
   (`rev-parse` + `ls-files`) on every call, and both new `it` blocks call it,
   so this one suite pays 4 subprocess spawns per run; `prompts.test.ts`
   spawns `rev-parse` again in each of its two tests. The data is
   write-stable within a run — it is exactly the listing the tests assert —
   so it is recomputation with zero invalidation value. Fix: compute the root
   and listing once at describe scope (e.g. `let files: string[]` populated in
   `beforeAll`, or a module-level memoized helper) and have both tests read
   the cached value; share a single `gitRoot()` result in `prompts.test.ts`.
   Impact is milliseconds — this is a tidiness nit, not a gate item.

## FYI (no severity, out of specialty)

The two new suites shell out to `git`, so `npm run check` now presumes the
test process runs inside a git checkout (worktrees included). That is a
correctness/portability question for other seats, not a performance one.

## Verdict

**approve** — zero open blocker/major findings; one nit.
