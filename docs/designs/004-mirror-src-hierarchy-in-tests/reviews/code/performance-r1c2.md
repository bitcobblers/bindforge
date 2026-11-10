# Code review — performance — r1 ratification c2

Scope: `git diff 1a66bda6a543508cf60bf792107a6d8b38bbfeee..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and ADR-004.
This pass checks only that the post-r1 revisions did not break what the
performance seat approved in round 1.

Note on method: the shell tool was permission-denied in this environment, so
the diff was reconstructed by direct inspection of the in-scope current files
(`tools/orchestration/tests/pipeline/*`, `.opencode/skills/*`). The move has
landed (23 files under `tests/pipeline/`, 3 at `tests/` root; both skill
surfaces now state `tests/**/*.test.ts`), matching Chosen design ¶1–¶4.

## Verdict: approve

The revisions since r1 touch only comments and assertion wording inside the
tree-pin blocks (`gates.test.ts` describe `test tree mirrors src/ hierarchy`)
and keep the same structure the performance seat approved: the fail-closed
check is one `git ls-files` listing plus linear filters over ~26 tracked
paths; the #3 check simulates the path-substring filter over that same
listing instead of spawning a nested vitest run (Test strategy #3, as
amended). No runtime `src/` path, config, or startup behavior changed — the
move is test-tree plus two documentation lines, so checklist items 1 and 4
have nothing to bite on.

## Checklist

1. **Event/keystroke path** — N/A: no runtime hot path in scope; the diff is
   test-tree moves, relative-depth rewrites, and two convention lines.
2. **Complexity** — the new pins are linear filters over the 26-entry tracked
   listing; the prompts convention check scans two small, repo-bounded
   directories. No O(n²) over unbounded n. Pass.
3. **Unbounded growth** — no caches/registries introduced; the recursive
   `readdir` in `prompts.test.ts:338` is bounded to `.opencode/skills` and
   `tools/orchestration/prompts`, not the growing `docs/designs/` tree. Pass.
4. **Startup** — vitest default recursive include; no config change, no added
   boot work. Pass.
5. **Cache invalidation** — no cache added; see the carried nit: the
   write-stable listing is recomputed instead. Nit unchanged since r1.
6. **Benchmark/timing note to run** — none present in the diff; nothing to
   measure or quote. (The A17 acceptance gate `npm run check` is an editor
   step, not a timing assertion; not runnable here — shell denied — but it is
   not a benchmark-like test, so this item does not count as unevaluated.)

## Findings

### nit — carried from r1, still open, unchanged in kind or magnitude
**Location:** `tools/orchestration/tests/pipeline/gates.test.ts:370-377,386,398`;
`tools/orchestration/tests/pipeline/prompts.test.ts:315-317,324,337`
**Problem:** identical to the r1 finding. `trackedTestFiles()` spawns two
synchronous git subprocesses (`rev-parse` + `ls-files`) per call and both new
`it` blocks call it (4 spawns per run); `gitRoot()` in `prompts.test.ts` is
invoked once per loop iteration — two iterations per test — so 4 more
`rev-parse` spawns. Total 8 write-stable-listing recomputations per suite
run; millisecond-scale in test code, so still nit. The post-r1 edits to these
blocks are comment-only at the performance-relevant sites (gates lines shifted
from 368–371/381/393 to 370–377/386/398 by an added explanatory comment;
prompts sites sit at the same lines r1 cited). Nothing was broken.
**Fix:** as recommended in r1: compute the repo root and the `git ls-files`
listing once at describe scope (shared variable in `beforeAll`, or memoize
`trackedTestFiles()`/`gitRoot()`) and hoist `gitRoot()` out of the two loops
in `prompts.test.ts`.

## FYI (no severity)
The revisions correctly implemented the testing seat's r1 correction the
proposal's revision log records (subset assertions instead of hard 26/228
pins, simulated #3 filter); from the performance side this is neutral to
positive — the permanent assertions now avoid spawning a nested vitest run.
