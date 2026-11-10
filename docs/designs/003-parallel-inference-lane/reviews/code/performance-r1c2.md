# Code review — performance seat — round 1, ratification pass c2

Scope: `git diff 91dfbea2f0ab34c063a14036b3afdc1f4b4227d9..HEAD -- ':(exclude)docs/designs'`,
read against the full current `02-proposal.md` and `docs/adrs/003-parallel-inference-lane.md`.
This pass checks only that the post-approval revisions (`5e12f5f` fix, `3745f3c` test-update)
did not break what this seat approved in round 1.

## Verdict: approve

## Disposition of round-1 findings (this seat)

1. **nit — double `git status` spawn per commit (`runner.ts` commitStage/formatTouchedFiles): RESOLVED.**
   The whole `formatTouchedFiles` step was dropped (security-driven). `commitStage`
   (runner.ts:295) now runs `dirtyFiles(repoDir)` exactly once, and no prettier
   spawn precedes the commit. Net spawn count per commit strictly decreased;
   the round-1 fix suggestion is moot and superseded.

2. **nit — semaphore permit held across `await writer.enqueue(state)`: CARRIED OVER, UNCHANGED.**
   `runSeat` still awaits `enqueueSnapshot()` (runner.ts:1248, 1282, 1286) inside the
   `try`, releasing the permit in `finally` (runner.ts:1287-1288). The later fix only
   prepends `state.updatedAt = Date.now()` to the same enqueue (runner.ts:1213-1216) —
   no new await, no new fs work. Bounded and negligible exactly as characterized in
   round 1; the ratification delta neither worsens nor fixes it. Still acceptable,
   still a nit (see findings below).

3. **nit — undocumented `structuredClone` + misleading "shells out of it" comment: RESOLVED in substance.**
   `state-store.ts:213-214` carries the required documentation ("Snapshot synchronously:
   the caller's state keeps mutating while this write waits its turn"), and the runner
   comment (runner.ts:758) — "the independence audit reads the tool calls and shells out
   of it" — now reads accurately against the widened audit, which genuinely consumes
   shell messages (`m.type === 'shell'`, independence.ts) out of `lastSeen`. No misleading
   subprocess implication remains under the natural parse. Withdrawing.

## Ratification checks (performance checklist)

1. **Event/polling path:** unchanged. `enqueueSnapshot` adds one `Date.now()` +
   field assignment per checkpoint write — no new fs, network, or spawn. The audit's
   new segment regexes (`TRANSCRIPTS_SEGMENT`/`REVIEWS_SEGMENT`/`REVIEWS_KEY`,
   independence.ts) run once per attempt over `lastSeen`, replacing substring tests;
   negligible against a per-attempt cost of session polling.
2. **Complexity:** `printable` (`/\p{Cc}/gu` replace), `reportKey` (single exec), and
   `boundedFindings` (slice ≤ 41 items) are all O(size-of-record); no unbounded
   nesting introduced.
3. **Unbounded growth — improved.** `boundedFindings` (runner.ts:1808-1850) caps
   recorded findings per seat at 40 and each field at 1,000 chars at the record site,
   and `reviewVars` re-bounds on the way out for pre-cap ledgers. This is exactly the
   checklist's item 3 answer for the seat board that is cloned into every later
   checkpoint and re-injected into prompts. `stageCommits` remains capped at 200.
   No new persisted field, matching the proposal's "no checkpoint schema change."
4. **Startup:** untouched by this range.
5. **Cache/stamp invalidation:** the stamp moved to the write sites (`checkpoint()`
   runner.ts:417-430, `enqueueSnapshot()` runner.ts:1213) with the writer still
   snapshotting synchronously at enqueue and `drain()` as the barrier — checkpoint
   write count per pass is unchanged, and enqueue order still matches timestamp order.
   Covered by the new assertion in `pipeline-parallel-integration.test.ts:230-235`.
6. **Cheap check run:** executed the package suite — `npm test` in
   `tools/orchestration`: **26 files / 224 tests passed, Duration 2.69s**. The new
   `updatedAt > startedAt` assertion is a stamp-semantics pin, not a benchmark; no
   timing flake risk (fixture stamps are 0, stamp site uses wall clock).

## Findings

| severity | location | problem | fix |
| --- | --- | --- | --- |
| nit | tools/orchestration/src/pipeline/runner.ts:1248,1282,1286,1287-1288 | Carried over from round 1, unchanged by this range: a seat holds its semaphore permit across `await enqueueSnapshot()`, so up to |panel| tmp+rename fs completions serialize inside slot turnover at N > 1. Bounded (≤3 enqueues, 5 writes worst case per seat) so the effect is negligible — acceptable as approved, listed for continuity. | Release the permit before awaiting the seat's own write promise (the writer snapshots synchronously at enqueue and `drain()` preserves the durability barrier), keeping rejection semantics via the existing `Promise.allSettled`. |

## FYI (out of specialty, no severity)

The formatter drop moves `format:check` clearing onto the `fix`/`test-update`
ownership route; that adds one stage round-trip when a stage leaves files
prettier-dirty, but the runner-side spawn savings are larger, so on net this
range is performance-positive.
