# Code review — performance seat, round 1 cycle 2 (ratification)

Scope: `git diff 29a6bc141f8a6420fb372271ff48d81dcb83202e..HEAD -- ':(exclude)docs/designs'`,
read against the current `02-proposal.md` and `docs/adrs/005-runner-executed-file-moves.md`.
This pass checks only that the post-approval revisions did not break what this seat approved.

Note on method: the sandbox refused shell execution and the package has no `node_modules`
(`glob tools/orchestration/node_modules` → no files), so the vitest suite could not be run
and no wall-clock numbers could be quoted (checklist item 6; see finding 2). The git range
could not be enumerated mechanically either; the review is against the full current state
of the diff's source and test paths (`moves.ts`, the moves wiring in `runner.ts`, and the
moves test files), which is what the range's revisions touched.

## What the revisions changed on this seat's axis, and whether it broke anything

1. **Child-session walk cost gate** (`runner.ts:1195-1200, 1292-1300`): `walkedDelegations`
   per-attempt counter + `countDelegationParts` guard bound the descendant walk to one
   network pass per delegation growth, and the walk runs only for attempts that delegated.
   This is the cost shape this seat wanted, and it is pinned by tests
   (`child-session-deeds.test.ts:217-221`, `child-session-deeds-retry.test.ts:316-319`).
   Not broken; improved.
2. **Moves rejection now rides the uniform nudge pacing floor** (`runner.ts:1439`, the
   `nudgeGate()` call that was round 1's floor skip): rejected lists now wait the floor
   like schema mismatches — bounded (MAX_NUDGES=8), and now pinned by tests at
   `reply-contract.test.ts:269` (schema mismatch) and `reply-contract.test.ts:609` (moves
   rejection, same shape: 1 prompt, `stage-timeout` at the deadline), plus the
   `minNudgeIntervalMs: 0` pin at `budget.test.ts:325-331`. The pacing half of last
   pass's finding 2 is answered by the new tests.
3. **Re-baseline after an invalid-reply nudge** (`runner.ts:1464-1470`): adds one
   `listSessionMessages` round trip per nudge — bounded at 8 per attempt, negligible.
   Not a cost problem; still not pinned by a distinguishing test (finding 2).
4. **`:(literal)` pathspec in `treeMode`** (`moves.ts:67`): same single `git ls-tree`
   spawn per lookup, no added subprocess. Fine.

## Open findings (all carried unchanged from the prior pass; nothing worse now)

### 1. minor — validator still spawns one `git ls-tree` per endpoint, per pass
`tools/orchestration/src/pipeline/moves.ts:66-73, 190, 203, 303, 334`
`treeMode` spawns a subprocess per endpoint; `validateMoves` runs it twice per pair
(rules 4 and 5), and `applyMoves` re-runs the same per-pair guard across the
inference→finalize window. A schema-capped 64-pair reply costs ~128 spawns per
validation and another ~128 per apply guard; a session that repeatedly answers the
moves nudge with a rejected list re-validates up to 9 times per attempt (stage prompt
+ 8 nudges), i.e. >1000 spawns, all against one immutable tree. Unchanged since the
prior pass. Fix as before: one `git ls-tree -r <headBefore>` per pass, parsed into a
Map; only the lstat/realpath disk checks stay per-call.

### 2. minor — checklist item 6 not evaluable; the baseline-growth claim still has no distinguishing test
`tools/orchestration/src/pipeline/runner.ts:1173-1175, 1405-1408, 1464-1470`;
`tools/orchestration/tests/pipeline/reply-contract.test.ts:174, 435`,
`moves-stage.test.ts:171`
The pacing-floor claims are now pinned (see item 2 above — prior finding 2 shrunk).
But every stub still makes each nudge produce a visible turn at `lists + 2`, so no
test models "nudge, session silent": reverting either re-baseline
(`runner.ts:1406-1408`, `runner.ts:1468-1470`) leaves the suite green, and the suite
itself could not be run here to quote any duration (no `node_modules`, shell refused).
Fix: one stub variant with `visibleFrom = Infinity` (no turn after the nudge)
asserting the attempt ends at the deadline as `stage-timeout` rather than re-reading
the already-rejected reply; quote the measured suite duration in the done-gate run.

### 3. nit — `matchingIssue` still re-folds the same endpoints in the inner loop
`tools/orchestration/src/pipeline/moves.ts:249-278`
`fold()` is re-invoked per comparison and the `endsA`/`endsB` arrays are allocated
inside the `(i, j)` loop: ~36k lowercase strings at the 64-pair cap where 128 exist.
Precompute `const ends = moves.map(m => [fold(m.from), fold(m.to)])` once and compare
against it.

## Out-of-specialty FYI
None.

## Verdict
approve — no blocker/major findings open; the post-approval revisions did not break
anything this seat approved (walk cost gate present and pinned, pacing uniform and
tested, no new subprocess or allocation pressure).
