# Code review — correctness, round 1, pass c0

Range: `git diff f523ce7e273a434fb052fa4b146245b2155bd28d...HEAD -- ':(exclude)docs/designs'`
Inputs used: that diff, `02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`.
No previous findings from this seat (first pass).

Verdict: **request-changes** — 4 major findings.

## Checklist walk-through

1. **Components/interfaces named by the proposal exist and match.** PASS.
   `MovePairSchema` + `moves: z.array(MovePairSchema).max(64).default([])` on the
   `implement`, `fix`, `test-update` entries (types.ts:29-44, 62-64);
   `validateMoves(repoDir, headBefore, stageName, moves)` and
   `applyMoves(repoDir, headBefore, moves)` in the new
   `tools/orchestration/src/pipeline/moves.ts`; wiring in `executeInference`
   (runner.ts:950-957, 681-696) with `headBefore` handed only by `executeStage`
   (runner.ts:1173); apply ahead of `enforceOwnership` in `finalizeStage`
   (runner.ts:1118-1143); `LOCK_FILE` relocated to types.ts:168 with the same
   name; the four prompt sentences are present. Validator rules 1-7 are all
   implemented (shape, reserved/`.git`, `stagePathAllowed`, tracked-regular-file
   `from`, no-dentry `to` + parent, case-folded disjointness, realpath containment
   with the `.git` test on the resolved path), and `replyFormat` renders the
   schema so the field self-documents.
2. **Assumption register honored.** PASS on A1/A2/A3/A4/A13 as far as the code
   shows: sweep untouched, `retryInSameSession` prefix test unchanged
   (runner.ts:1218-1223), checkpoint `version` still 1, `state-store.ts` untouched,
   `fs.renameSync` (A8) is the primitive, `git ls-tree` is the only new git call.
   A10's contrast is pinned as designed (moves.test.ts:530-561).
3. **Deviations declared in `03-build-notes.md`.** NOT EVALUABLE at this seat: the
   caller scoped this pass's inputs to the diff, the proposal and the ADR, and the
   diff excludes `docs/designs`. Findings 1, 3 and 4 below are undeclared *relative
   to the proposal*; if `03-build-notes.md` declares them, the finding narrows to
   "declared but out-of-scope", not to zero.
4. **Scope creep.** FAIL — findings 1, 3, 4.
5. **Build-notes claims.** NOT EVALUABLE (same scope note as item 3). One claim
   *inside the diff's own comments* is checkable and is false — see finding 1.
6. **Error paths from the design exist and fail closed.** PARTIAL — the
   `invalid-reply: moves` and `moves-apply-failed` classes behave as designed;
   the `moves-rollback-failed` class does not (finding 2).
7. **Previous round's blocker/major findings addressed.** N/A — first code pass
   for this seat.

## Findings

### 1. major — `runner.ts:1122-1132`: the failed-apply path stages the whole worktree; the design says it leaves today's unstaged state

The design is explicit: "Finalize returns without sweeping or committing, leaving
the preserved-worktree state that a failed reply leaves today (runner.ts:1046-1048)",
and Test strategy #11 says the same shape. The code adds a `git add -A` that the
proposal never asked for. Its stated justification — that "`git status --porcelain`
collapses a wholly-untracked directory to a single entry" — is contradicted by the
production path it is supposed to protect: the sweep reads paths through
`dirtyFiles` (runner.ts:184-188), which already passes
`--untracked-files=all`, so nothing is collapsed and attempt 2's sweep sees the
same paths either way. The staging also mutates shared state the design leaves
alone: the index now holds content the ownership sweep has never vetted, so a
crash right after the failure leaves a staged worktree an operator (or a later
`commitStage` after a different failure class) can commit without the sweep ever
having run on it.

Fix: drop the staging block and return the outcome as designed. If the coder
wants the preserved paths listed individually, that is a separate change with its
own justification in the proposal/build notes.

### 2. major — `runner.ts:1134-1140` + `runner.ts:1550-1594`: `moves-rollback-failed` starts attempt 2 anyway

The design's Apply section makes the rollback failure the one terminal class:
"`moves-rollback-failed: <error>` escalates to the operator instead of starting
attempt 2 on a known-damaged worktree". What the code produces is a failure string
whose only distinguishing effect is `retryInSameSession(...) === false`
(runner.ts:1218-1223) — and a *false* there means attempt 2 runs **with a fresh
session**. `runWithRetry` (runner.ts:1551-1581) unconditionally re-runs the stage
after any first-attempt failure, so a half-moved worktree gets a new agent session
pointed at it, and if that attempt replies validly the damage is committed and the
run continues. The comment at runner.ts:1213-1216 asserts the opposite ("and
`moves-rollback-failed:` starts nothing at all"), so the code's own documentation
is wrong. Test strategy #6 asks for "escalates to the operator and starts no
attempt 2"; the only test written for it asserts the prefix
(reply-contract.test.ts:187-196), which does not express that requirement, and the
unit-level construction is `it.skip`'d (moves.test.ts:524-527).

Fix: make the class terminal at the driver — in `runWithRetry`, before the attempt-2
call, treat a failure beginning `moves-rollback-failed` as `finalizeEscalate(...)`
plus `StopDriving` (recording the gate entry as today). Pin it with a driver-level
test that counts sessions/attempts, not with a prefix assertion.

### 3. major — `runner.ts:770-780` and `runner.ts:994-1000`: the reply-polling loop for every stage was rewritten, and one documented guard was removed

Two changes to shared inference machinery that the proposal never asked for:
`assistantBaseline` moved from after the stage prompt to before it, and the
re-baseline after the invalid-reply nudge deleted. Both affect every stage and
every review seat, including the pre-existing schema-mismatch path, not just
moves. The removal specifically deletes the protection the previous comment
described — "without growth past this baseline the poll below would re-read the
previous reply and burn the whole nudge budget in seconds" — and the idle-count
test at runner.ts:829 can no longer tell "the session produced a new turn" from
"idle with no new turn" (the case the branch at runner.ts:839-846 exists for):
on the latter the runner now re-reads the *previous* reply, re-validates it as if
it were the repair, and burns the nudge budget on a stale turn. Nothing in the
proposal authorizes this; the new comments argue it from backend behavior the
design never registered as an assumption.

Fix: restore the post-prompt baseline and the post-nudge re-baseline (the moves
path works with them — the stubs in moves-stage.test.ts and reply-contract.test.ts
answer one turn per prompt either way). If a real backend stall was observed, it
belongs in the assumption register and `03-build-notes.md` as its own change, not
folded into an ADR-005 branch.

### 4. major — `tools/orchestration/prompts/design-review.md:6-10`: a design-review prompt change that ADR-005 never asked for

The proposal's Prompts section names exactly four files (`implement.md`, `fix.md`,
`fix-tests.md`, `test-update.md`). This branch additionally rewrites the
design-review seat's prompt (auditing of listing/globbing/grepping run-record
directories). It changes the instructions every design-review seat in every future
run receives — a behavior change of the pipeline, unrelated to file moves, with no
proposal text behind it and no test pin added for it.

Fix: take it out of this branch (its own change), or add it to the proposal scope
and pin it in `prompts.test.ts` like the four moves sentences are.

### 5. minor — `runner.ts:969`: a moves rejection skips the nudge pacing floor and the timeout classification

`const gate = checked.success || nudges >= MAX_NUDGES ? 'go' : await nudgeGate();`
means a schema-valid-but-moves-invalid reply never consults `nudgeGate()`, so it
never takes the `'over'` → `stage-timeout` branch and never waits the
`minNudgeIntervalMs` floor. The design says the moves failure behaves "exactly as
for any invalid reply" through the same nudge/retry/escalate path; here one of the
three ways that path can end has been removed for moves only. The reasoning in the
comment is plausible but is a divergence, and it is what turns finding 3's
stale-turn read into a fast burn.

Fix: keep `nudgeGate()` on this branch (the floor is a wall-clock floor, not a
"silent session" special case), or record the divergence in the proposal/build notes.

### 6. nit — `moves.ts:57-64`: `git ls-tree` pathspecs are glob-matched

`git ls-tree <rev> -- <relPath>` applies git pathspec globbing, so a `from`/`to`
like `src/*.ts` matches a tracked sibling and the mode test passes on a path that
is not itself tracked. It fails closed today only incidentally, because rule 4's
disk check then rejects it ("tracked but not on disk"). Fix: pass the pathspec as
`:(literal)<relPath>` (or assert no `*?[]` in rule 1) so rule 4 means what it says.

### 7. nit — `runner.ts:687-690`: a missing `headBefore` silently skips moves validation

`movesValidationIssue` returns `null` when `headBefore === undefined`. That is
unreachable for the three move-capable schemas today (only `executeStage` passes
it, runner.ts:1173), but the shape is permissive-by-default: a future caller that
runs a move-capable stage through the pass fan-out would accept a moves-bearing
reply at inference and then apply it in finalize unvalidated. Fix: fail closed —
an issue string ("moves declared without a base revision") for that branch.

## FYI (out of specialty, no severity)

- `validateMoves` spawns up to ~2 git subprocesses per pair (plus realpath/lstat
  per endpoint) inside the reply-acceptance path, and `applyMoves` re-runs the same
  rules as its guard — up to ~4×64 git spawns per accepted reply. Belongs to the
  performance seat.
- Test strategy #5's rollback-fault case is `it.skip`'d (moves.test.ts:524) and
  #6's "starts no attempt 2" is pinned only as a prefix assertion; that gap is what
  let finding 2 through. Belongs to the testing seat.
- Checklist items 3 and 5 (deviations and claims in `03-build-notes.md`) could not
  be evaluated at this seat: the caller's input list for this pass excludes
  `docs/designs`, and the diff excludes it by design.
