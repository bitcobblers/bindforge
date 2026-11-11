# Code review — performance seat, round 1, pass c0

Diff reviewed: `git diff f523ce7e273a434fb052fa4b146245b2155bd28d...HEAD -- ':(exclude)docs/designs'`
(17 files: `moves.ts` new, `runner.ts`, `types.ts`, `package.json` engines, 4 prompts, `design-review.md`, 6 test files).
Judged against `02-proposal.md` and ADR-005.

## Checklist verdicts

1. **Event/keystroke path (hot paths):** pass. `movesValidationIssue` runs only after a schema-valid
   reply candidate — not per poll tick — and returns immediately when `headBefore` is undefined
   (review fan-out) or `moves` is empty/default, so review passes pay nothing. `applyMoves` is
   synchronous `renameSync`, exactly as the proposal chose, under the `.pipeline.lock` single-writer
   invariant, off the polling loop. The polling-loop changes (pre-prompt baseline at
   runner.ts:778-780, removed re-baseline after an invalid-reply nudge) *remove* one
   `listSessionMessages` round trip per invalid-reply nudge — a net improvement.
2. **Complexity:** pass with one subprocess-spawn finding (below). `matchingIssue` is O(n²) but n ≤ 64
   is schema-enforced (`moves.max(64)`, types.ts) and the proposal names the cap as the work bound —
   not flagged. No identity-lookup-should-be-map cases: the O(n²) comparisons are pairwise
   prefix/equality tests, not keyed lookups.
3. **Unbounded growth:** pass. No new caches, registries, or arrays that outlive one call;
   `REGULAR_MODES`/`WINDOWS_RESERVED` are fixed module constants; `done` in `applyMoves` is
   bounded by the pair cap.
4. **Startup:** pass. `moves.ts` executes nothing at import beyond two small constants and the zod
   schema; no fs/git work added to boot.
5. **Cache invalidation:** n/a — the change introduces no cache. The rules 4–7 re-run in
   `applyMoves` is a fresh TOCTOU guard, not a cache revalidation, and it is covered by the
   guard tests in moves.test.ts and moves-stage.test.ts.
6. **Benchmark-like tests / timing notes:** partially evaluated — see finding 2. The change carries
   three explicit timing claims (runner.ts:770-777, 958-965, 994-1000) but no runnable timing pin
   distinguishes them, and the sandbox has no installed dependencies (no `node_modules`) with the
   shell restricted, so I could not execute the suite to quote a number.

## Findings

### Minor — per-endpoint `git ls-tree` subprocess spawns (moves.ts:57, 181, 194; guard re-run at 324-328)
Every `endpointsIssue` call spawns one `git ls-tree` for `from` (line 181) and one for `to`
(line 194); `validateMoves` runs it per pair and `applyMoves` re-runs the same per-pair lookups
as the rules 4–7 guard. A 64-pair reply therefore costs up to 2×64 = 128 subprocess spawns per
pass and up to ~256 per stage attempt (validation + guard), and a session that repeats a
rejected list pays it again per nudge cycle (the reply-contract test's nine-answer loop models
exactly that). At a few ms to ~15 ms per small-repo `ls-tree`, that is on the order of a second
or several of wall time burned per capped attempt on process startup alone, all for lookups in
one immutable tree.
Fix: batch per pass — one `git ls-tree -r <headBefore>` (or a multi-pathspec
`git ls-tree <headBefore> -- <p1> <p2> …`), parse `mode … \tpath` lines into a Map, and answer
every mode question from the Map. Semantics are unchanged: `headBefore` is a fixed commit, so
the fresh-look requirement of the TOCTOU guard does not apply to the tree side (only the
`lstat`/`realpath` disk checks need to stay per-call).

### Minor — the three polling-loop timing claims have no distinguishing test (runner.ts:770-777, 969, 994-1000; reply-contract.test.ts:35, 137, 376)
Each rewritten comment asserts a latency property: (a) the floor skip at runner.ts:969 keeps a
repairable moves reply from becoming a stage-timeout in production (default floor
`DEFAULT_MIN_NUDGE_INTERVAL_MS = 20_000`, stages.ts:73); (b) skipping the re-baseline prevents a
fast backend's corrected reply from folding into the baseline and stalling the attempt; (c) the
pre-prompt baseline keeps the idle-count check honest on fast backends. None of the new tests
can falsify any of the three: every moves test uses `reviewConfig()` with
`minNudgeIntervalMs: 0` (reply-contract.test.ts:35), so keeping `await nudgeGate()` on a moves
rejection would pass identically — and every stub makes a turn visible only `lists + 2` after
its prompt (lines 137, 376), so a post-prompt re-baseline would never count the answer and both
baseline placements behave the same. Reverting any of the three changes keeps the suite green.
Checklist item 6 ("run it and quote the number") therefore could not be satisfied for this
change in this sandbox (no node_modules; shell restricted), and no equivalent pin exists for a
later run to lean on.
Fix: copy the existing pin pattern at reply-contract.test.ts:219 — run one moves-rejection test
with `reviewConfig({ minNudgeIntervalMs: 30_000, stageTimeoutMs: 3_000 })` and assert attempt 1
still ends `invalid-reply: moves …`, not `stage-timeout` (pins the floor skip); add a stub
variant with `visibleFrom: lists + 1` and assert the nudge cycle still burns the nudge budget
rather than stalling (pins (b) and (c)). Quote the measured duration in the done-gate run.

### Nit — repeated case-folding in the disjointness loop (moves.ts:240-266)
`matchingIssue` calls `fold()` on the same four endpoints ~18 times per (i, j) pair — including
inside each `nests(x, y)` argument — allocating ~36k lowercase strings at the 64-pair cap where
128 exist. Bounded and microseconds in absolute terms, but trivially avoidable.
Fix: precompute `const ends = moves.map(m => [fold(m.from), fold(m.to)])` once and compare
against the array.

## Out-of-specialty FYI (no severity)

`prompts/design-review.md` gains a browsing-audit sentence beyond the proposal's four-prompt
Prompts section; whether that belongs to this change is a scope question for another seat.

## Verdict

**approve** — zero open blocker/major findings. The moves path is bounded everywhere the
proposal promises (schema caps, single-writer lock, reverse rollback), adds nothing to startup
or review fan-out, and the polling-loop edits are round-trip-neutral to positive. The three
findings are bounded-work and pin-coverage refinements.
