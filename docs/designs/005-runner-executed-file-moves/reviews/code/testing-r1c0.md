# Code review — testing seat — round 1, pass c0

Scope reviewed: `git diff f523ce7e273a434fb052fa4b146245b2155bd28d...HEAD -- ':(exclude)docs/designs'`
(17 files, +1776/-41). Judged against `02-proposal.md` and `docs/adrs/005-runner-executed-file-moves.md`.
Checklist: `.opencode/skills/code-review/references/testing.md` (items 1-7). No other seat's report read.

## Checklist walk

**1. Every Test strategy item cited.** All 13 items carry at least one
`// proves: Test strategy #N` comment: #1 and #6 and #9 in `reply-contract.test.ts`,
#2/#3/#5/#12/#13 in `moves.test.ts`, #4 in `ownership.test.ts`, #7 in
`prompts.test.ts` + `fix-trigger.test.ts`, #8/#11 in `moves-stage.test.ts`, #10 in
`budget.test.ts`. Citation coverage is complete; the gaps below are depth, not labels.

**2. Unit + integration layers.** Both present: validator/applier/classification units,
and `executeStage`/`resumePipeline` round trips (#8, #9, #10, #11). Good.

**3. Revision rounds.** No prior code-review blocker/major exists for this run, so
nothing to re-pin. Design-round contracts that changed (`r4`: `git ls-tree`
regular-file mode instead of `cat-file`; `lstatSync` dentry instead of `existsSync`;
parent-required; case-folded disjointness) are each pinned by a named case
(`moves.test.ts:139`, `:169`, `:189`, `:374`, `:383`, `:394`). Pass.

**4. Determinism.** No wall-clock assertions, no network, no live server, no ports:
stubs fake the message list with a monotonically increasing fake clock and turn
visibility counters; git runs against `mkdtemp` fixtures. One environment-dependent
case (tmpfs mount, `moves.test.ts:493-515`) is uid-gated — see finding 7.

**5. Test names describe behavior.** Consistently behavior-phrased
("rejects a to that is a dangling symlink", "lands the declared moves as R entries in
exactly one stage commit"). Pass.

**6. `npm run check` — NOT EVALUATED in this seat.** I could not run it: the review
sandbox's shell policy allowed only `git` for this session — `npm run check`,
`npx`, `node`, `ls`, and `which` were each denied — and no `node_modules` exists in
this worktree, so no green tail can be pasted here. Per the checklist, an item I
could not evaluate is a finding, not a pass (finding 6). The runner must confirm green
independently; nothing in the diff is obviously un-typeable, but note that
`moves.test.ts:37-41` redeclares `MovePair` rather than importing it, which is what
lets the contract drift unnoticed (finding 2).

**7. Untestable additions / dead branches.** Three found and reported
(finding 1's `it.skip`, finding 2's shape-tolerant helpers, finding 8's throw-guard).

## Findings

### 1. major — the `moves-rollback-failed` escalation class has no execution test, and the only test that names it asserts a property the code does not provide

- Location: `tools/orchestration/tests/pipeline/reply-contract.test.ts:183-197`;
  `tools/orchestration/tests/pipeline/moves.test.ts:517-527` (`it.skip`);
  production branch `tools/orchestration/src/pipeline/moves.ts:338-347` and
  `tools/orchestration/src/pipeline/runner.ts:1136-1140`.
- Problem: Test strategy #6 requires that a `moves-rollback-failed: …` outcome
  "escalates to the operator and starts no attempt 2". The only pin is
  `retryInSameSession('moves-rollback-failed: …', 'ses_1') === false`. That function
  (`runner.ts:1218-1223`) only chooses same-session vs fresh-session; `runWithRetry`
  (`runner.ts:1550-1591`) unconditionally runs attempt 2 for any non-ok outcome, and
  nothing in `src/` special-cases the `moves-rollback-failed` prefix (the only other
  mention is the comment at `runner.ts:1216`). So the behavior the comment and the
  test comment both claim — "starts nothing at all" — is neither implemented nor
  observed. The whole `rollbackFailed === true` path (moves.ts's reverse-undo failure,
  finalizeStage's decision to skip `git add -A` for it) is executed by zero tests:
  the one test written for it is `it.skip`, so #5's last clause and #6's last clause
  are both unbacked. This is the one state the proposal itself calls "the one state
  nothing in-process can undo", i.e. the highest-consequence branch in the change.
- Fix: make the skipped case real. The repo already mocks modules in tests
  (`tests/cli.test.ts:29` uses `vi.mock('../src/opencode-runner.js', importOriginal)`),
  so a partial mock of `node:fs` whose second `renameSync` call (the undo) throws gives
  a deterministic fault injection; assert `applyMoves` returns
  `{ rollbackFailed: true }` and that disk holds the half-applied state. Then add one
  pipeline-level case (`moves-stage.test.ts`-style, or `budget.test.ts`-style through
  `resumePipeline`) whose fix/implement stage yields `moves-rollback-failed: …` and
  assert the run escalates with `codeBudgetUsed` unchanged for a second entry and with
  no second `session.create` for that stage — i.e. actually pin "starts no attempt 2".
  If the implementation keeps starting a fresh attempt 2, that is the correctness
  seat's fix; either way the pin must exist, and today's test comment must stop
  claiming a property no test checks.

### 2. major — the validator and applier contracts are not pinned: tests accept any return shape and drop the one field the runner branches on

- Location: `tools/orchestration/tests/pipeline/moves.test.ts:37-41` (local
  `interface MovePair`), `:66-88` (`issueOf`), `:90-104` (`applyFailureOf`).
- Problem: `issueOf` accepts `null`/`string`/`{issue}`/`{message}`/thrown-error as
  "a rejection", and `applyFailureOf` accepts `null`/`string`/`{error}`/`{message}`/
  thrown-error as "a failure". Because the helpers then flatten the result to a
  string, `MoveApplyFailure.rollbackFailed` — the exact discriminator
  `finalizeStage` switches on (`runner.ts:1136`) — is asserted nowhere in the suite,
  and the tests would keep passing if `applyMoves` returned a plain string, if
  `rollbackFailed` were hardcoded `false`, or if the field were renamed. The test file
  also redeclares `MovePair` locally, so `types.ts`'s `MovePairSchema`/`MovePair`
  could drift (bounds, field names) with no compile error in the tests. The
  shape-tolerance comment ("the proposal leaves the failure representation open")
  describes the pre-implementation state; the implementation now has a fixed contract
  that the tests should pin rather than dodge.
- Fix: import `type MovePair` from `../../src/pipeline/types.js` and
  `type MoveApplyFailure` from `../../src/pipeline/moves.js`, drop the local
  interface, and type the helpers as `string | null` and `MoveApplyFailure | null`
  (keeping only a try/catch if the throw path is genuinely contractual — see
  finding 8). Assert `failure.rollbackFailed` explicitly in every `applyMoves` case:
  `false` for the guard refusal and the mid-list ENOTDIR rollback, `true` in the
  fault-injected case from finding 1.

### 3. minor — changes to the shared inference lane are unpinned: reverting either one keeps the suite green

- Location: `tools/orchestration/src/pipeline/runner.ts:770-780` (assistant baseline
  read before the stage/resume prompt) and `runner.ts:993-1000` (no re-baseline after
  the invalid-reply nudge).
- Problem: both edits change reply-polling behavior for *every* stage, review seats
  included, and both are justified only by prose comments about backends whose turn is
  already visible on the very next `message.list` call. No fixture in the suite models
  that: every stub delays visibility by at least one poll
  (`reply-contract.test.ts:137` and `:376` use `visibleFrom = lists + 2`;
  `budget.test.ts`'s new stub uses `visible = min(lists - 1, prompts)`). I traced the
  new #9 and #10 tests against the pre-change code: both pass with the old ordering
  and with the old re-baseline. So the fix has zero regression protection, and neither
  edit appears in the proposal's Test strategy or Revision log.
- Fix: add a stub variant with zero-delay visibility (`visible = min(lists, prompts)`)
  and assert the attempt reads the reply instead of ending as `stage-timeout`
  (pins the pre-prompt baseline); add one case where a session answers each invalid
  reply instantly and assert the attempt burns the nudge budget down to
  `invalid-reply:` rather than stalling (pins the no-re-baseline decision). If the
  edits are load-bearing for the live backends, they also belong in the proposal's
  revision log so the next seat can judge them against a stated intent.

### 4. minor — validator rule clauses implemented with no unit case, despite #2's "one case per rule clause"

- Location: `tools/orchestration/src/pipeline/moves.ts:107-115` (segment ending in
  `.` or a space; empty segment) and `moves.ts:238-246` + `:252-258` (rule 6's nesting
  clauses: a move that nests inside itself, and one pair's path containing another's).
- Problem: `moves.test.ts:196-275` pins rule 1's backslash/colon/control/absolute/`.`/
  `..`/Windows-reserved clauses, but not `segment.endsWith('.') || segment.endsWith(' ')`
  or the empty-segment branch, and no test exercises `matchingIssue`'s `nests(...)`
  branches. Those are reachable: `validateMoves(dir, head, 'implement',
  [{from: 'src/old.ts', to: 'src/old.ts/inner.ts'}])` passes rules 4-5-7 (the parent is
  a tracked regular file, hence a dentry) and is rejected only by the self-nesting
  check; the cross-pair nesting check is what keeps
  `{from:'src/old.ts',to:'src/a.ts'}, {from:'src/beta.ts',to:'src/old.ts/inner.ts'}`
  out. Untested, an edit to `nests()` (e.g. dropping the trailing `/`) silently turns
  those into apply-time `ENOTDIR` failures instead of reply errors.
- Fix: add validator cases for `{to: 'src/x./new.ts'}`, `{to: 'src/x /new.ts'}`,
  `{to: 'src//new.ts'}`, the self-nesting pair above, and the cross-pair nesting pair,
  each `// proves: Test strategy #2`.

### 5. minor — the motivating payload (test-file moves by `test-update`/`fix`) is never landed end to end

- Location: `tools/orchestration/tests/pipeline/moves-stage.test.ts:164-199` (single
  `implement` pair); `moves.test.ts` unit fixtures never run through a stage.
- Problem: A12/ADR-005's whole reason for existing is run 004's 23 test-file moves
  owned by the test engineer. Every integration pin moves one `src/**` or `docs/**`
  file as `implement`; no test has `test-update` land a multi-pair list into
  `tests/**`, which is where rule 3 (`inDocs || inTests`), the sweep interaction (#4),
  and rename detection in one commit have to hold together. `fix` is only tested on
  the rejection side (#10).
- Fix: extend #8 with one `test-update` round trip carrying 3+ pairs into `tests/**`
  (mirroring run 004's shape), asserting one stage commit whose
  `git show --name-status -M` reports an `R` entry per pair and no
  `stagePathAllowed` revert.

### 6. minor — checklist item 6 could not be executed in this seat (no green tail to paste)

- Location: report-level; affected package `tools/orchestration` (`npm run check` =
  lint + `format:check` + vitest).
- Problem: the seat's shell policy permitted only `git` invocations; `npm run check`,
  `npx`, `node`, `ls`, `which` were all denied, and the worktree has no `node_modules`,
  so the suite's green status is unverified from here. The checklist treats an
  unevaluable item as a finding, and the seat that follows (or the gate) must supply
  the output tail.
- Fix: nothing for the branch author beyond ensuring the gate's `npm run check` output
  is recorded in `03-build-notes.md`; the next testing pass in this run should paste
  the tail (the commit `test: finish prettier formatting of the ADR-005 moves test
  files` suggests formatting was chased by the gate, not by a local `check`).

### 7. nit — the EXDEV case can report "passed" while asserting nothing, and mutates host mount state

- Location: `tools/orchestration/tests/pipeline/moves.test.ts:493-515`.
- Problem: `it.runIf(process.getuid() === 0)` then `catch { return; }` after
  `mount -t tmpfs`: on a machine that is root but unprivileged (the common container
  case) the test runs, mounts nothing, returns, and is recorded as passed. When the
  mount does succeed, the test performs a real system mount/umount inside the suite.
- Fix: probe the capability once (`describe.runIf(canMountTmpfs())`, probe in a
  helper that mounts and immediately unmounts a scratch dir) so the case is reported
  as skipped rather than passed, and keep `finally { umount }` for the live case.

### 8. nit — unreachable throw-guard whose coverage is claimed by a test that never throws

- Location: `tools/orchestration/src/pipeline/runner.ts:757-762`
  (`movesValidationIssue`'s `try/catch` and its `headBefore === undefined` guard);
  claimed at `tools/orchestration/tests/pipeline/reply-contract.test.ts:465-468`.
- Problem: `validateMoves`/`endpointsIssue` are total functions under their own
  contract: every git failure reads as "not tracked" (`moves.ts:52-56`), a missing
  `repoDir` returns an issue (`moves.ts:281-284`), and `spawnSync` does not throw on a
  non-zero exit. The test that claims to pin "a validator rule that throws internally
  … lands as `invalid-reply: moves`" uses `from: 'CON'`, which `shapeIssue` rejects
  normally — no throw is exercised, so the `catch` is an untested branch and the
  comment overstates the pin.
- Fix: either delete the `try/catch` (the contract is a returned issue) and correct the
  test comment, or pin it honestly with a case that forces a throw (mock `node:fs` or
  `node:child_process` so a rule throws) and assert the failure still arrives as
  `invalid-reply: moves — <message>`.

## What this change gets right (for the record)

Rule-by-rule validator coverage is dense and each rejection names the offending pair
(`expectRejectionNaming`), the sweep invariant is pinned in both directions
(`ownership.test.ts:168-208`), the invalid-reply ride-through is pinned at both unit
and pipeline level including the one-budget-event claim (#10), and the apply-failure
integration case (#11) pins the three things that matter at once: no commit, byte-
identical pair paths, and an un-swept preserved worktree listed path-by-path.

## FYI (out of specialty, no severity)

The diff also edits `prompts/design-review.md` (the "listing, globbing, or grepping
counts as opening them" sentence) — outside the proposal's Prompts section, and not
pinned by any prompt-content test, though the audit it describes already exists and is
matched by `independence.ts:132-142` for review seats. `finalizeStage`'s new
`git add -A` on the failed-apply path is a deviation from the proposal's flat
"returns without sweeping or committing", but its observable effect (paths listed
individually for attempt 2) is pinned by #11. The `engines` bump to `>=22.23.4` is in
the Compatibility matrix but no test can pin an engines field; flagging only so the
panel sees it as an unpinable, not an omission.

## Verdict

request-changes — 2 open major findings (1, 2).
