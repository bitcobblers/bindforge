# Code review — testing seat, round 1 dissent pass c1

Scope: `git diff 29a6bc141f8a6420fb372271ff48d81dcb83202e..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md` and `docs/adrs/005-runner-executed-file-moves.md`,
re-checking this seat's round-1 findings first.

## Verdict: approve

Zero open blocker/major findings. `npm run check` in `tools/orchestration` is green from this seat.

## Re-check of my round-1 findings

1. **major — `moves-rollback-failed` escalation unbacked; rollbackFailed path never executed — RESOLVED.**
   Production: `escalatesWithoutRetry` (runner.ts:1700-1702) is consulted in the driver's
   `runWithRetry` (runner.ts:2044-2053) *before* attempt 2 is built, and
   `finalizeStage` (runner.ts:1588-1607) now classifies via `failure.rollbackFailed`, amends the
   ledger entry, and returns without sweeping/staging/committing. Tests:
   `moves-rollback.test.ts` drives the real driver with a fault injected at the `renameSync`
   boundary (the armed undo of `src/moved.ts → src/old.ts`, undoAttempts=1 asserting the fault
   really ran) and pins the claim by counting, not by prefix: exactly one fix result, one fix
   session created, one stageFailure gate entry, one code-budget unit, escalation reason naming
   the class, the half-applied worktree preserved and never committed (subjects ==
   base + escalated). The paired suite disarms the fault and pins the contrast:
   `moves-apply-failed` *does* get a fresh-session attempt 2 (2 fix results, 2 fix sessions) —
   which is what makes the terminal-class assertion a claim rather than a tautology.
   `escalatesWithoutRetry` also has a unit case (reply-contract.test.ts:240-255). Round-1 code
   (no terminal branch, attempt 2 always running) fails the terminal suite.

2. **major — validator/applier contracts unpinned; `MoveApplyFailure.rollbackFailed` asserted nowhere; `MovePair` redeclared — RESOLVED.**
   `moves.test.ts` now imports `MoveApplyFailure` and `MovePair` as types from the production
   modules (lines 18-19), so contract drift breaks the test build; `issueOf`/`applyFailureOf`
   (106-117) are thin typed wrappers over the real signatures, no shape-flexing.
   `rollbackFailed` is asserted on all four apply outcomes: guard refusal (line 591), exact
   rollback (608), EXDEV (635), and the injected failed undo (663, plus the half-applied-disk
   assertions 667-669).

3. **minor — two shared-inference-lane edits unpinned (baseline position, no re-baseline after invalid-reply nudge) — RESOLVED by revert.**
   `967dd3c` ("revert the out-of-scope changes") removed them: against **main**, the branch no
   longer moves the `assistantBaseline` read or removes the invalid-nudge re-baseline — both
   now sit exactly where mainline had them (the main..HEAD diff shows only comment rewording on
   both hunks). No unproposed shared-lane behavior change remains; the pacing floor now also
   applies to moves rejections, matching the proposal's "behave exactly as for any invalid
   reply", and is pinned end-to-end by reply-contract.test.ts:609-631 (pre-change code that
   skipped the floor fails it). One *new* moves-specific branch is still untested — see open
   finding 1 below.

4. **minor — rule clauses implemented with no unit case (trailing dot/space, empty segment, self-nesting, cross-pair nesting) — RESOLVED.**
   moves.test.ts adds all five: segment ending in `.` (322), in space (331), empty segment (340),
   `to` nesting inside its own `from` (483 — the shape that passes rules 4/5/7 and is rejected
   only by the nesting check), and a pair nesting inside another pair's endpoint (495).

5. **minor — the motivating payload never landed end to end — RESOLVED.**
   moves-stage.test.ts:251-297 lands a `test-update` three-pair move list into `tests/pipeline/`
   (run-004's A12 shape) in exactly one commit, asserts one `R` line per pair, the stage's own
   content edit in the same commit, and that no endpoint appears as a standalone `D`/`M` (the
   #4 sweep invariant seen end to end).

6. **report-level — `npm run check` unverifiable in this seat — RESOLVED; shell policy permits it now.** Tail:

   ```
   npm run @bindforge/orchestration@1.0.0 format:check
   Checking formatting...
   All matched files use Prettier code style!
    Test Files  31 passed (31)
         Tests  316 passed | 2 skipped (318)
      Duration  3.42s
   ```
   (lint clean; the 2 skips are the EXDEV case — `canMountTmpfs()` probe returns false on this
   unprivileged host — and the darwin half of the A10 pair. The `mount: must be superuser`
   line is the probe's own stderr, not a test failure.)

7. **nit — EXDEV case "passed" while asserting nothing on an unprivileged host — RESOLVED.**
   `canMountTmpfs()` probes once in a scratch dir and `describe.runIf(canMountTmpfs())` gates
   the case (moves.test.ts:545-561, 626-642), so an unprivileged host reports it skipped, and
   the real mount/umount happens only inside the capability-gated case with a `finally` umount.

8. **nit — unreachable throw-guard presented as tested — MOSTLY RESOLVED.** The throw half is now
   genuinely exercised: reply-contract.test.ts:37-63 arms a throwing `spawnSync`/`execFileSync`
   keyed to `ls-tree` + `boom.md`, and the test at 574-601 pins that a rule letting a throw
   escape still lands as `invalid-reply: moves — <error>`, not a crash. The `headBefore ===
   undefined` guard half remains untested — see open finding 2.

## Open findings

1. **minor** — `tools/orchestration/src/pipeline/runner.ts:1411-1417` (and `lastMovesIssue` at
   1191, 1436-1438). The exhausted-nudge fallback that renames `invalid-reply: not JSON` to
   `invalid-reply: moves — <lastMovesIssue>` (added by `17710f6` "name the rejected moves list
   when a nudged session goes silent") has no test reaching it: every moves-rejection stub in
   reply-contract.test.ts answers *each* prompt with parseable moves JSON, so exhaustion always
   exits through the parse/schema path at runner.ts:1473, and no stub mixes a moves rejection
   with subsequent narration. The branch is reachable in a real shape (a session that answers
   the moves nudge with prose, exhausting the budget). Fix: one `sequenceStubServer` case whose
   texts are `[moves-invalid, ...narration×8]`, asserting the failure is
   `invalid-reply: moves — <pair-naming issue>` rather than `invalid-reply: not JSON`.

2. **nit** — `tools/orchestration/src/pipeline/runner.ts:1068-1069`. The fail-closed issue for a
   moves-bearing reply arriving without a base revision is still unreachable under today's
   schemas (only `executeStage` calls with moves-capable replies, always passing `headBefore`;
   review replies cannot carry `moves`) and unpinned — checklist item 7. Defensible as
   defense-in-depth for a future schema slip; if kept, pin it or note it in build notes as an
   intentional untestable guard. (Round-1 finding 8's throw half is fixed; this is its residue.)

## Checklist notes

1. Every numbered strategy item has a citing test: #1/#9 (reply-contract.test.ts), #2/#3/#5/#12/#13
   (moves.test.ts), #4 (ownership.test.ts, predating this range, unchanged and still green),
   #6 (reply-contract + moves-rollback.test.ts), #7 (prompts.test.ts:321-331, fix-trigger.test.ts:70-74),
   #8/#11 (moves-stage.test.ts), #10 (budget.test.ts:264-347).
2. Unit + integration layers both present as promised.
3. Both round-1 majors (1, 2) now have tests that round-1 code would fail (attempt-counting
   driver suites; typed contract imports + rollbackFailed assertions).
4. Deterministic: stub servers, temp-dir repos, fixed clocks inside stubs; the only capability-
   dependent case is gated by a probe; no network/ports.
5. Test names describe behavior throughout.
6. `npm run check` green (tail above).
7. Untestable additions: the `headBefore === undefined` guard (finding 2).

FYI (no severity): this diff range also carries the merged 007-audit-by-deeds work (main merge
`8f98292`: independence.ts, child-session deeds tests, audit-by-deeds prompt). It belongs to
design 007's own reviews, cites no 005 strategy item, and is green; no 005 seat should judge it.
