# Code review — performance seat — r1c0

Diff under review: `git diff 73200b44...HEAD -- ':(exclude)docs/designs'`
(25 files, +2521/−73). Judged against `02-proposal.md` and
`docs/adrs/003-parallel-inference-lane.md`.

## Verdict: approve

Zero blocker/major findings. Three nits (below). The performance core of the
design — template preloaded once per pass, semaphore-only await ahead of
`session.create`, drained writer, retry-inside-permit — is implemented as
specified and its benchmark-like tests pass with the expected numbers.

## Checklist

**1. Event/keystroke path (here: per-seat attempt path).** No new synchronous
fs or network on the polling/seat path. `executeInference` dropped a
`git rev-parse HEAD` per attempt (moved to the pass/stage window — a net
removal on the concurrent path). `loadTemplate` now runs once per pass
(`runner.ts` `runReviewPass`, template preload) instead of per seat attempt —
strictly less fs than before. The new independence audit
(`independence.ts`) is pure in-memory over the already-fetched `lastSeen`
message list (runner.ts:845); it spawns nothing and fetches nothing.
`formatTouchedFiles` does add a sync `git status` + prettier `spawnSync` per
commit (runner.ts:282, :317), but finalize runs only after the pass barrier —
with nothing else in flight — and matches the runner's existing all-`spawnSync`
git posture (A6). Pass.

**2. Complexity.** Semaphore acquire/release O(1) with direct permit handoff
(no free+regrab). Audit is O(turns × messages × parts) per attempt, bounded by
one session's message list (≤ 1 + MAX_NUDGES audits per attempt); no re-fetch,
no per-tick scan. Writer chain is O(writes) per pass. `packageDirs` is a
bounded probe (root + one level + `tools/*`/`packages/*`), not a tree walk. No
O(n²) over an unbounded dimension. Pass.

**3. Unbounded growth.** Semaphore waiter queue bounded by |seats| ≤ 5 per
pass; one writer per pass, discarded at pass end; `lastSeen` is replaced, not
appended; `state.results`/`gateLog` growth stays bounded by ADR-002 session
budgets. New per-pass objects are scoped to the pass. Pass.

**4. Startup.** `resolveMaxParallel` is pure string checks, run before any
side effect (cli.ts comment + placement before `createClient`); boot reads no
new file. Templates are still read lazily per stage/pass, not eagerly at boot.
Pass.

**5. Cache invalidation.** No cache is introduced: `templateText` is loaded
per pass and dies with the pass (so a mid-run prompt-template edit is picked
up by the next pass — no staleness); the checkpoint writer is an ordered queue,
not a cache, and its landing-order invariant is covered by
`tests/pipeline-checkpoint-writer.test.ts` (enqueue order, slower-first-write,
no stray `.tmp`, invalid-state refusal — Test strategy #7). Pass.

**6. Benchmark-like tests — run, numbers quoted.** I ran the suite myself:
`npm test` in `tools/orchestration`: **26 files, 224 tests, all green,
Duration 2.55s**. The fan-out tests observe and assert max in-flight ==
min(N, 5) at N ∈ {1,2,3,5}
(`pipeline-parallel-fanout.test.ts:220–229`); per-test wall clocks (verbose
reporter): full-panel resume at **N = 1: 438ms vs N = 2: 305ms, N = 3: 266ms,
N = 5: 229ms** — the fan-out shortens the staggered review phase as the ADR's
ceil(|panel|/N) claim predicts (stub windows 15/35/55/75/95ms: serial ≈275ms
vs parallel ≈95ms). Escalation test confirms unstarted seats are skipped
(they release without prompting): 194ms at N = 1.

## Findings (all nits)

1. **nit — runner.ts:282, :317–318 — `git status` spawned twice per commit.**
   `formatTouchedFiles` runs `dirtyFiles(repoDir)` (a sync `git status` spawn)
   and `commitStage` immediately recomputes `dirtyFiles(repoDir)`. Every commit
   (≈30/run) pays one extra subprocess. Fix: have `formatTouchedFiles` return
   (or accept) the dirty list and reuse it for the emptiness check.
2. **nit — runner.ts:1238, :1272, :1276 — seat holds its semaphore permit
   across `await writer.enqueue(state)`.** The permit is only released in
   `finally` after the queued `saveState` chain has caught up to this seat's
   write, so up to |panel| fs write completions serialize inside slot
   turnover. The writer already snapshots synchronously and the barrier
   `drain()` guarantees durability, so the seat could release before awaiting
   its own write promise (same rejection semantics via `allSettled`), or simply
   not await it while the chain's rejection still ends the run at drain time.
   Cost today is ≤5 small tmp+rename writes per pass — irrelevant against
   minutes-long seats — hence nit, not minor.
3. **nit — state-store.ts:207 / runner.ts:765 — overhead notes on new code.**
   `structuredClone(state)` adds one full deep copy per checkpoint write on top
   of `saveState`'s existing `safeParse` + `JSON.stringify` (bounded, fine; a
   comment saying so would stop a future "optimize" pass from racing the writer
   with a lazy snapshot). Separately, the runner.ts:764–765 comment claims the
   audit "shells out of it" — it never shells out (pure in-memory); the wording
   invites a future maintainer to assume a subprocess sits in the polling loop.

## FYI (out of specialty, no severity)

`docs/workflows.md` still reads "Seats execute one at a time (single local
inference lane)" (line 83) and ADR-003 still sits under "Proposed, not
implemented" (lines 87–91); neither workflows.md nor any README gained
`--max-parallel` / `$OPENCODE_MAX_PARALLEL`, the `N ≤ server slots` guidance,
or the timeout/queueing note — all required by the proposal's Configuration +
Rollout #3 ("docs land with the change"). Likely the compatibility/rollout
seat's finding; noted here only because the guidance is the operator-facing
half of this lane's performance contract.
