# Code review — correctness (round 1, seat c0)

Scope: `git diff 0179375b8d037833c6b6c5150312d88af0162211...HEAD -- ':(exclude)docs/designs'`.
Judged against `02-proposal.md` (ADR-020) and the correctness checklist.

## Verdict: approve

One minor finding; no blocker or major. The change is a faithful implementation
of the design.

## Checklist

1. **Named components exist and match.** `touchesExecutableSurface` beside
   `stagePathAllowed` with the two §1 prefixes (gates.ts:541–570);
   `RestartBoundary { stage, paths, at }` and `PipelineState.restartRequired?`
   beside `planStop` (types.ts); `RestartBoundarySchema.optional()` in
   `StateSchema` with `status` and `version` untouched (state-store.ts:69–73,
   119); `surfacePathsInWindow` (runner.ts:1845–1851), the detection hooks
   after `recordStageCommit` in `finalizeStage` (1957) and `finalizeReviewPass`
   (2032, commit-path only); the loop-top pause before `decideNext`
   (2549–2557); `finalizeRestartBoundary` beside `finalizeDone` (3561) with
   gate entry, terminal-save guard, `chore: <nnn>-<slug> restart boundary`
   commit, and printed summary; resume gate-log + clear (2477–2487);
   `restartBoundaryStatusLines` in `statusLines` (3797–3816, 3850);
   `PipelineRunResult` widening and `exitCodeFor` done→0 / record→3 / other→2
   / throw→1 (cli.ts:43–52, 245, 258) with the header comment and the
   workflows.md operator section carrying the table. All match the proposal's
   names and shapes.
2. **Assumption register honored.** Net diff is exactly `git diff -z
   --no-renames --name-only <headBefore>..HEAD` through `nulPaths` (§1); the
   A3/A4 sweep-revert net rule is pinned by a test (runner.test.ts:891); the
   A6 grammar move is one optional field, `version` 1; the A7 inert-unknown-key
   posture is pinned (state-store.test.ts); the A10 `planStop` shape is
   mirrored; `states.ts` gained no transition; the lock posture is untouched
   (A9).
3. **Deviations all declared and resolved.** Deviation 1 (seam-pin migration)
   is done across the 17 old test suites; Deviation 2's fixture now pins both
   postures — red gate with no surface change escalates (exit-2 direction),
   red gate with a surface-touching window pauses at the loop top per §1's
   mandated check point (restart-boundary-integration.test.ts:438, :455);
   Deviation 3 converged via the build-notes' own fix (run-shared seat
   ordinals, :490). Deviations 4–5 (PROMPTS_DIR export; record drop before
   escalation) are declared and harmless.
4. **No scope creep** of substance (see FYIs for two output-only extras).
5. **Build-notes claims** checked against the code: what-was-built and the
   five deviations are true at HEAD. (The quoted gate legs predate the test-fix
   commit and no longer describe HEAD — FYI only, docs are out of this seat's
   scope.)
6. **Error paths fail closed** — with one narrow gap, finding 1.
   `finalizeRestartBoundary`'s `CheckpointWriteFailure` arm restores the
   committed copy, escalates, and keeps the record out of the committed
   checkpoint (pinned, runner.test.ts:1011–1030).
7. **No previous-round findings** to re-check (first pass of round 1).

Gate record (green at 2026-11-13T21:02:48Z) is consistent with HEAD: I re-ran
the new suites (`restart-boundary-integration`, `executable-surface`,
`runner`, `cli`, `gates`, `state-store`) — 115 tests, all pass.

## Findings

1. **[minor] `finalizeEscalate` leaves a standing `restartRequired` on an
   escalated run, which maps to exit 3, not 2.**
   Location: `tools/orchestration/src/pipeline/runner.ts:3656` (and the
   escalation catch at :3016–3028).
   Problem: a surface-touching coder window sets the record in `finalizeStage`
   (:1957); if the very next checkpoint save (:3015) throws
   `CheckpointWriteFailure`, `finalizeEscalate` runs with the record still in
   state — its terminal checkpoint then carries `restartRequired` alongside
   `status: 'escalated'`, ESCALATION.md is written, and `runResult`/
   `exitCodeFor` return 3, not 2. A `while [ $? -eq 3 ]; do resume; done`
   driver carries the escalated run on without the operator ever reading the
   marker — exactly the code-collision §3 says `2` exists to prevent.
   Reachable only through a save failure in the same iteration as a
   surface-touching window, hence minor, not major; the code is literally
   faithful to §3's canonical mapping sentence, so this is hardening, not a
   proposal gap.
   Fix: clear `state.restartRequired = undefined` at the top of
   `finalizeEscalate` — the same posture the pause's own save-failure arm
   already takes and Deviation 5 declares ("the pause did not happen, so its
   record must not ride into the checkpoint the escalation rewrites").

## FYI (no severity)

- `recordRestartBoundary` prints a `console.warn` line at classification time
  (runner.ts:1889–1893) in addition to §3's summary line from
  `finalizeRestartBoundary`; the design names only the latter. Harmless, but
  a driver's log now sees the pause twice per boundary.
- Test strategy #6 pins the seam as the *result shape* (status + record);
  `exitCodeFor`'s integers 0/3/2/1 have no direct pin in cli.test.ts. Left to
  the testing seat.
- `03-build-notes.md`'s gate-legs section quotes the pre-fix red suite and is
  stale against HEAD's green gate; it is run record, outside this diff.
