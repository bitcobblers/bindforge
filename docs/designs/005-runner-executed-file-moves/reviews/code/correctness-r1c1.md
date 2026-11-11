# Code review — correctness — r1 dissent pass c1

Range: `29a6bc1..HEAD` excluding `docs/designs/`, read against `02-proposal.md`
and `docs/adrs/005-runner-executed-file-moves.md`. Specialty: correctness
(faithful to the design). This is the dissent pass on the revision written to
answer my round-1 findings.

## Verdict: approve — all seven round-1 findings are genuinely fixed

### Round-1 finding-by-finding (checklist item 7)

1. **major — staging sweep on the failed-apply path — FIXED.**
   `finalizeStage` (runner.ts:1586-1608) now returns
   `{ ...outcome, ok: false, failure: text }` with no `git add`; the comment at
   1603-1606 states the correct reason (`dirtyFiles` passes
   `--untracked-files=all`, verified at runner.ts:207-211). This is exactly the
   preserved-worktree state a failed reply leaves. Declared as removed in
   03-build-notes.md ("Reverted after code review round 1").

2. **major — `moves-rollback-failed` did not stop the run — FIXED.**
   `escalatesWithoutRetry` (runner.ts:1700-1702) is checked in `runWithRetry`
   *before* attempt 2 is built (runner.ts:2044-2053): stageFailures and the
   `stageFailure` gate entry are recorded first (2032-2037), then
   `finalizeEscalate` and `StopDriving`. The false comment is gone. Pinning:
   the old prefix-only assertion is replaced by a real driver-level suite,
   `moves-rollback.test.ts`, which drives `resumePipeline` with an injected
   filesystem fault and counts one attempt, one `session.create`, one
   `stageFailure` entry, `codeBudgetUsed` 1, an escalation naming the class,
   and the half-moved disk; the control suite (fault disarmed) counts two
   attempts for `moves-apply-failed`, which is what makes the terminal claim
   non-tautological. `moves.test.ts:649` now pins `rollbackFailed: true` at the
   `applyMoves` level (the old `it.skip` is gone; the only skips anywhere are
   `describe.runIf` capability probes for tmpfs/EXDEV and macOS-only suites).

3. **major — poll-baseline rewrite — FIXED.**
   `assistantBaseline` is read again *after* the stage/resume prompt
   (runner.ts:1168-1175, original anti-masquerade comment restored), re-baselined
   after the narration nudge (1406-1408), and re-baselined after the
   reply-rejection nudge including the moves branch (1464-1470). The
   idle-count distinction at 1232/1242 is intact. Revert declared in build notes.

4. **major — design-review prompt rewrite — FIXED for this branch.**
   967dd3c deleted the added sentences from `prompts/design-review.md`, and
   `git diff HEAD main` shows the branch's own delta no longer touches
   `design-review.md`, `test-authoring.md`, `independence.ts`,
   `opencode-runner.ts`, `cli.ts`, or any child-session/007 test file. The
   apparent changes to those paths in the caller's two-dot range come entirely
   from `8f98292 chore: merge main (007-audit-by-deeds)`: HEAD is byte-identical
   to main for every 007-touched path (verified with
   `git diff --stat HEAD main -- <paths>`). They are main's landed change, not
   this branch's, and are not counted against this branch.

5. **minor — nudge pacing floor skipped for moves rejections — FIXED.**
   The moves rejection and schema mismatch share one code path
   (runner.ts:1420-1474): `nudges < MAX_NUDGES ? await nudgeGate() : 'go'` at
   1439, with `over` → `stage-timeout` at 1440-1443. Corroboration that the
   floor really applies: `budget.test.ts:331` had to give the moves case
   `minNudgeIntervalMs: 0` so the nudge budget can burn before the stage
   timeout — exactly the behavior my fix asked to restore.

6. **nit — `git ls-tree` pathspec glob-matching — FIXED.**
   `treeMode` passes `:(literal)<relPath>` (moves.ts:57-73), with `--` also
   present and rule 1 already rejecting `:`. Pinned: a tracked
   `bindings[en].json` is accepted and an untracked `a?.ts` glob-matching a
   tracked sibling is rejected (moves.test.ts:204-227). Declared as deviation 5
   in 03-build-notes.md; the git-1.9 magic-signature dating keeps the
   "git, any version" matrix row honest.

7. **nit — `movesValidationIssue` lenient on missing `headBefore` — FIXED.**
   runner.ts:1068-1070 returns an issue when a non-empty moves list arrives
   with no base revision, fail-closed; only `executeStage` passes `headBefore`
   (1640) and `runReviewPass` passes none (1815, 1835), so a moves-bearing
   reply on the pass fan-out is now rejected, not applied unvalidated.

### New code in the fix commits — reviewed, no new correctness defects

- Deviation 7 (build notes): `lastMovesIssue` labels the exhausted-nudge
  empty-reply failure `invalid-reply: moves — <issue>` (runner.ts:1186-1191,
  1411-1417). This serves the proposal's own final-failure naming, is declared,
  and the schema-mismatch branch at 1473 still reports its own poll's problem,
  so no mislabeling path exists.
- The ledger amend in `finalizeStage` (1597-1602) walks `state.results` from
  the newest entry, so a same-session attempt 2 amends its own entry, not
  attempt 1's. Consistent with the proposal making the failure string the
  operator-visible outcome.
- Wiring re-verified against the proposal's Chosen design: schemas
  (`MovePairSchema`, `.max(64).default([])`, the three stage entries),
  validator rule set, `applyMoves` re-running rules 4-7 as one guard before the
  first rename (moves.ts:333-338), call site ahead of `enforceOwnership` and
  `commitStage`, `git add -A` staging only in the pre-existing `commitStage`,
  four prompt sentences pinned by `prompts.test.ts` and `fix-trigger.test.ts`
  (Test strategy #7), engines bump as the matrix states.
- Review-pass lane stays out of the moves machinery: review schemas carry no
  `moves`, `finalizeReviewPass` never applies moves, and the terminal class can
  only be produced on the single-stage driver path — consistent with the ADR's
  "Untouched: review protocol, gates" clause.

### Assumptions and deviations (checklist items 2, 3, 5)

A2/A4/A5/A8/A11/A13 all hold in the current code. Deviations 1-7 in
03-build-notes.md are each declared and each verified true against the code.
The notes' live-gate record (lint 0, `npm test` exit 0, format-only red then
cleared by `107c167`) could not be re-executed here — shell was denied for test
runs in this seat — but every pinned case named there was located in the test
files and traced statically against the implementation.

### FYI (no severity)

All escalations, including the new terminal branch, end in
`finalizeEscalate` → `commitStage`, whose pre-existing `git add -A` lands the
half-moved pair inside the terminal `docs: … pipeline escalated` commit
(pinned by `moves-rollback.test.ts:251`); the damage is visible to the
operator, but on a branch commit as well as the worktree. This is today's
escalation machinery unchanged — flag it to the editor only if the operator
preference is that a known-damaged tree be left uncommitted.
