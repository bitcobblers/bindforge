# Build notes — ADR-005 runner-executed file moves

Implementation of `02-proposal.md` in `tools/orchestration`. What landed:

- `src/pipeline/moves.ts` — `validateMoves` (rules 1–7, whole list against
  `headBefore` plus disk, nothing touched) and `applyMoves` (re-runs rules 4–7
  as one guard, then `renameSync` per pair in reply order, reverse rollback).
  The rule 4/5 tree lookup asks git for a `:(literal)` pathspec, so it is an
  exact-path lookup and not a glob match (deviation 5).
- `src/pipeline/types.ts` — `MovePairSchema` / `MovePair`; a `moves` field
  (default `[]`) on the `implement`, `fix`, and `test-update` reply schemas;
  `LOCK_FILE` moved here next to `STATE_FILE`.
- `src/pipeline/runner.ts` — `executeInference` gained an optional
  `headBefore` parameter (passed only by `executeStage`); a schema-valid reply
  with a non-empty `moves` list is validated before acceptance and a rejection
  rides the existing invalid-reply path as `invalid-reply: moves — <issue>`
  (same pacing floor, nudge, same-session attempt 2, then escalate). A
  moves-bearing reply with no base revision is rejected instead of applied
  unvalidated. `finalizeStage` applies the moves ahead of
  `enforceOwnership`/`commitStage`, and on apply failure returns
  `moves-apply-failed:` (fresh attempt 2) or `moves-rollback-failed:` without
  sweeping, staging, or committing. `runWithRetry` treats `moves-rollback-failed`
  as terminal: one `stageFailure` gate entry, `finalizeEscalate`, no attempt 2 —
  classified by the exported `escalatesWithoutRetry`, next to
  `retryInSameSession`.
- `prompts/implement.md`, `fix.md`, `fix-tests.md`, `test-update.md` — the one
  sentence each the proposal asked for.
- `package.json` — the `node: ">=22.23.4"` engines bump the proposal carries.

## Deviations from the proposal

1. **Schema chain order.** The proposal writes
   `z.array(MovePairSchema).default([]).max(64)`; in zod v4 `.max` does not
   chain after `.default`. Implemented as
   `z.array(MovePairSchema).max(64).default([])` — same validation (max 64
   pairs, absent means `[]`), order swapped.
2. **`LOCK_FILE` relocated.** `runner.ts` → `types.ts` (next to `STATE_FILE`)
   so `moves.ts` can name it in rule 2 without importing `runner.ts` (a cycle).
3. **Failure representations chosen** where the proposal left them open.
   `validateMoves` returns `string | null` (the issue naming the offending
   pair) rather than throwing — the proposal itself allows "a returned issue or
   a throw from a rule", and the runner additionally catches a throw
   (`movesValidationIssue`) so either lands as `invalid-reply: moves — …`.
   `applyMoves` returns `MoveApplyFailure | null`
   (`{ error, rollbackFailed }`), the structured form the runner needs to
   separate the environmental apply failure from the failed rollback.
4. **`finalizeStage` now returns the `StageOutcome`.** It previously returned
   nothing; the moves outcome (`ok: false` + failure string) must reach
   `executeStage`'s caller so the retry/escalate classification sees it.
5. **`:(literal)` on the `git ls-tree` pathspec** (correctness nit 6,
   compatibility major 1). The proposal writes `git ls-tree <headBefore> --
   <from>`, and a bare pathspec is glob-matched by git even after `--`:
   `a?.ts` matches a tracked `aX.ts` (rule 4's regular-file test then passes on
   a path that is not itself tracked, and an untracked directory of that shape
   reaches `renameSync` as a directory move), while a legitimately named
   `bindings[en].json` fails rule 4. The magic prefix makes the lookup exact.
   Rule 1 has already rejected `:` in every endpoint, so nothing else can reach
   the pathspec, and magic signatures date to git 1.9 — the matrix's "git, any
   version" row still holds.
6. **`escalatesWithoutRetry` as a named predicate.** The proposal states the
   terminal class in prose ("escalates to the operator instead of starting
   attempt 2"); the driver needs something to branch on, and a named export
   beside `retryInSameSession` is the shape the classification tests can pin.
7. **A nudged-then-silent session keeps the moves label on its failure.** The
   proposal names the end of a rejected list "final failure `invalid-reply:
   moves — <issue>`". The reply loop only reached that string while the session
   kept answering with rejected replies; a session that answered the
   moves-rejection nudge with narration, or with nothing at all, exhausted the
   nudge budget on the empty-reply branch and logged `invalid-reply: not JSON`,
   which loses the pair the attempt died on — the gate log and attempt 2's repair
   prompt then name nothing (and `budget.test.ts`, Test strategy #10, pins the
   moves label on exactly that shape). `executeInference` now remembers the last
   issue `validateMoves` returned for the attempt and, at the exhausted-nudge
   branch of the empty-reply path, reports `invalid-reply: moves — <issue>`
   instead. Nothing else changes: a schema mismatch or a narration-only stage
   with no moves in the reply still ends `invalid-reply: not JSON`
   (`reply-contract.test.ts:244-254`), and the rejection-nudge branch still
   reports the problem of its own poll.

## Reverted after code review round 1

Earlier notes here listed four further deviations and a prompt edit. All five
were out of scope for ADR-005 and are gone; nothing in this section is a
deviation any more:

- **Staging on the failed-apply path (`git add -A`, never a commit) — removed**
  (correctness major 1). `finalizeStage` now returns without sweeping, staging,
  or committing, exactly the preserved-worktree state a failed reply leaves
  today. The stated justification was false against the production path:
  `dirtyFiles` already passes `--untracked-files=all`, so nothing is collapsed
  and attempt 2's sweep sees every preserved path whether or not the index
  holds them. Staging also handed the ownership sweep an index it had never
  vetted.
- **The min-nudge floor skip for moves rejections — removed** (correctness
  minor 5). A moves rejection now takes the same `nudgeGate()` as a schema
  mismatch, so it keeps all three ways that path ends (`go` / `wait` /
  `over` → `stage-timeout`), which is what "behave exactly as for any invalid
  reply" (proposal, Wiring into retry) means.
- **The poll-baseline change — reverted to the pre-branch discipline**
  (correctness major 3, testing minor 3). The assistant-count baseline is again
  read *after* the stage/resume prompt, and again re-read after every nudge,
  the reply-rejection nudge included. Both edits changed reply polling for
  every stage and every review seat with nothing in the proposal authorizing
  it, and removing the re-baseline destroyed the anti-masquerade guard the
  comment described (an idle session with no new turn had its previous reply
  re-read as the repair). No assumption-register entry backs either change;
  if a live-backend stall is ever observed, it belongs there first.
- **`prompts/design-review.md` — the added browsing-discipline sentence is
  out of this branch** (correctness major 4). The proposal's Prompts section
  names exactly four files. Note for the next seat: shell was denied to this
  stage, so the revert reconstructs the earlier wording by deleting the added
  sentences ("Listing, globbing, or grepping those directories counts as
  opening them … never browse for one") rather than by `git show`; the
  resulting paragraph matches `code-review.md`'s wording for the same rule,
  and no `prompts.test.ts` assertion pins the removed text.
- **The `headBefore === undefined` leniency in `movesValidationIssue` — now
  fails closed** (correctness minor 7): a moves-bearing reply whose window
  carries no base revision is an issue, not a silent pass.

## Test pins that must change (this stage routed `testsNeedUpdate: true`)

Test paths belong to `test-update`, and this stage cannot edit them. The pins
below are what the review round asked for; the two that are red against the
reverted implementation are marked.

Status after the `test-update` round: pins 1-10 landed — `moves-stage.test.ts`
now asserts the un-swept state through `--untracked-files=all` (its :250-257),
`budget.test.ts` carries `minNudgeIntervalMs: 0`, `reply-contract.test.ts` pins
the class through `escalatesWithoutRetry`, `moves-rollback.test.ts` is the new
driver-level pin, and `moves.test.ts` took the contracts, the pathspec pins and
the rule-clause gaps with the `node:fs` fault injection. Pin 11 is unverified:
no stage in this run has had shell, so the formatter never ran.

1. `moves-stage.test.ts:250` **(red)** — `git status --porcelain` containing
   `tests/preserved.md` held only because of the deleted staging step: at the
   default `-untracked-files=normal` git collapses a wholly-untracked `tests/`
   to `?? tests/`. Re-assert the design's shape instead — the file still on
   disk, HEAD still `headBefore`, no commit, and the path listed individually
   under `--untracked-files=all` (the flag the sweep itself uses).
2. `budget.test.ts` moves case **(red)** — `FAST` keeps the default 20 s
   `minNudgeIntervalMs` against a 4 s `stageTimeoutMs`, so with the pacing
   floor restored a moves rejection ends `stage-timeout` before the nudge
   budget can burn. Give that case `minNudgeIntervalMs: 0` (as
   `reply-contract.test.ts`'s `reviewConfig` does); it then fails
   `invalid-reply: moves`, resumes the same session for attempt 2, and leaves
   `codeBudgetUsed` at 1.
3. `reply-contract.test.ts:183-198` — `retryInSameSession(...) === false`
   cannot distinguish "fresh attempt 2" from "no attempt 2", and the comment
   claims an escalation the old code never ran. Pin the class with
   `escalatesWithoutRetry('moves-rollback-failed: …') === true` and
   `escalatesWithoutRetry('moves-apply-failed: …') === false`.
4. New driver-level pin for Test strategy #6: a stage whose apply reports a
   rollback failure must leave exactly one `stageFailure` gate entry, one
   `session.create` for that stage, `codeBudgetUsed` unchanged, and an
   `escalate` reason naming `moves-rollback-failed`.
5. `moves.test.ts:524` — make the skipped rollback-fault case real with the
   partial `node:fs` mock the repo already uses (`tests/cli.test.ts:29`): the
   second `renameSync` throws, so `applyMoves` returns
   `{ rollbackFailed: true }` and the disk holds the half-applied pair.
   `MoveApplyFailure.rollbackFailed` is the discriminator `finalizeStage`
   branches on, and no test asserts it today.
6. `moves.test.ts` contracts — import `type MovePair` from
   `../../src/pipeline/types.js` and `type MoveApplyFailure` from
   `../../src/pipeline/moves.js`, drop the local `interface MovePair`, and type
   `issueOf`/`applyFailureOf` as `string | null` / `MoveApplyFailure | null`
   instead of accepting any shape.
7. `moves.test.ts` pathspec pins for deviation 5 — a tracked
   `bindings[en].json` is accepted by rule 4; an untracked glob-ambiguous
   `from` (`a?.ts` with a tracked `a.ts`) is rejected, and so is a directory of
   that shape.
8. `moves.test.ts` rule-clause gaps under #2's "one case per rule clause" —
   `{to: 'src/x./new.ts'}`, `{to: 'src/x /new.ts'}`, `{to: 'src//new.ts'}`,
   the self-nesting `src/old.ts → src/old.ts/inner.ts`, and the cross-pair
   nesting `src/old.ts → src/a.ts` with `src/beta.ts → src/old.ts/inner.ts`.
9. `moves.test.ts:493-515` — probe the tmpfs capability in a helper behind
   `describe.runIf` so the EXDEV case reports skipped rather than passed on an
   unprivileged root, keeping the `finally { umount }` for the live case.
10. `reply-contract.test.ts:465-468` — the comment claims a throwing rule
    lands as `invalid-reply: moves`; `from: 'CON'` is rejected by rule 1
    without throwing. Either force a throw (mock `node:fs` or
    `node:child_process`) and assert the message, or drop the claim. The
    `try/catch` in `movesValidationIssue` stays: the proposal's Wiring
    paragraph names both shapes.
11. Formatting — `prettier --check` still flags the committed wrapping in
    `moves.test.ts` and `moves-stage.test.ts`; `npx prettier --write` on just
    those two files, since the earlier `test:` formatting commit did not clear
    them.

## Gate status

Shell is denied to every stage of this run, so `npm run check` never ran here and
the implementation side is verified statically, case by case, against the test
files as they now stand:

- `moves.test.ts` — each validator case re-derived against `moves.ts` rules 1-7
  (`:(literal)` lookup for `bindings[en].json`, `a?.ts`/`b?.ts`, the
  mode-`040000` and mode-120000 refusals, `ENOTDIR` under rule 5's `lstatSync`,
  the self-nesting and cross-pair nesting pins, the realpath escapes) and the
  four `applyMoves` cases including the injected `rollbackFailed` fault.
- `moves-stage.test.ts` — both round trips traced end to end: reply accepted →
  apply → sweep sees only in-ownership paths → one commit with an `R` entry; the
  ENOTDIR pair → `moves-apply-failed`, nothing staged
  (`git diff --cached --name-only` empty), `tests/preserved.md` still listed
  under the sweep's own `--untracked-files=all`.
- `moves-rollback.test.ts` — one stage result, one `stageFailure` entry,
  `codeBudgetUsed` 1, no attempt 2, and the escalated commit: `drive`'s
  `runWithRetry` takes the `escalatesWithoutRetry` branch before attempt 2 is
  built (runner.ts:1567), and `finalizeEscalate` commits
  `docs: <nnn>-<slug> pipeline escalated` without sweeping the half-applied pair.
- `reply-contract.test.ts` — schema caps (200/64) and the two driver cases hold
  under the re-baseline discipline, because `sequenceStubServer` appends one turn
  per prompt, visible one poll later.
- `budget.test.ts` — the moves case is the reason for deviation 7. Its message
  stub is the saturating one (`visible = max(0, min(lists - 1, prompts))`), which
  under the re-baseline discipline can never show a turn after the first nudge;
  the attempt therefore dies on the empty-reply branch, and only deviation 7 puts
  `invalid-reply: moves` on that branch's failure. With the `minNudgeIntervalMs:
  0` the test now carries, the run escalates with `codeBudgetUsed` 1 and one
  `fix` session, as pinned.

Two things this stage could not verify and leaves open for the next gate run:

- Formatting (test pin 11 above). `prettier --check` was named against
  `moves.test.ts` and `moves-stage.test.ts` before the pins landed; both files
  are now rewritten, every code line sits inside printWidth 100 (the long
  `it(...)` titles and comments are string/comment text prettier will not
  reflow), and the import groups and specifiers match
  `importOrderSortSpecifiers`. Nothing in this stage could run the formatter to
  confirm it.
- `budget.test.ts` carries an edit made by an earlier turn of this stage in the
  working tree (a `fixPrompts`/`keyOf` block in the stub's `prompt` handler that
  references a helper the file never defines). It is a test path, so this stage
  will not touch it again: the runner's revert of test paths restores the file.
  Test-update should keep the saturating stub (the pinned behavior it models —
  a session that answers once and then goes quiet — is the one deviation 7 is
  about) and drop the unused `validFix` reply if it reworks the case.

Review passes remain untouched: `runReviewPass` passes no `headBefore`, review
schemas carry no `moves`, and the terminal moves classification lives only on
the single-stage driver path that can produce it.

## Fix round: gate run live — format-only red, both files test-owned

This fix round reproduced the gate (`npm run check` in `tools/orchestration`,
exit 1):

- `npm run lint` — exit 0, no findings.
- `npx prettier --check .` — exit 1, flagging exactly two files:
  `tests/pipeline/moves-rollback.test.ts` (the `subjects()` method-chain
  wrapping near :139, which prettier wants collapsed to one line) and
  `tests/pipeline/moves.test.ts` (the `applyFailureOf` argument array near
  :633, likewise). `moves-stage.test.ts` — the other half of the stale pin-11
  list — is now clean; `moves-rollback.test.ts` is the newly flagged file.
- `npm test` — exit 0: 31 files, 316 passed, 2 skipped (the skipped pair is
  the tmpfs/EXDEV capability probe, degrading gracefully on an unprivileged
  host, as pin 9 intends).

So the only red is `format:check`, and both offending files are under
`tests/`. Per the fix-tests contract this stage does not touch them — running
`npm run format` package-wide here would rewrite only test paths, which the
runner reverts, leaving the gate identically red. This reply sets
`testsNeedUpdate: true`; test-update owns `prettier --write` on exactly those
two files. Nothing in `src/`, `prompts/`, or `package.json` was touched, and
no test asserts anything wrongly — the suite is fully green. No deviation
from the proposal.

## Fix round after the test-update pins (no change to src)

This stage again had shell denied, so `npm run check` was not run here either.
Every moves case in `moves.test.ts`, `moves-stage.test.ts`,
`moves-rollback.test.ts`, `reply-contract.test.ts`, `budget.test.ts`, and
`ownership.test.ts` was re-derived against `moves.ts`, `types.ts`, and
`runner.ts` line by line — validator rules 1-7, the `:(literal)` pathspec
lookups, the retry/escalation classification, the apply-failure and
rollback-fault fixtures, the budget and sweep pins — and no implementation
divergence was found. The one known red the run's own record names is
`prettier --check` (test pin 11): the gate's `format:check` named
`moves.test.ts` and `moves-stage.test.ts`, and the earlier
`test: finish prettier formatting of the ADR-005 moves test files` commit did
not clear them; both files are test-owned. Per the fix-tests contract this
reply sets `testsNeedUpdate: true` so `test-update` can run the formatter on
the files it owns; nothing in `src/`, `prompts/`, or `package.json` was
touched. No deviation from the proposal.

## Close-out: where the run landed (2026-11-10)

This section closes the record. It changes nothing above it; it records what
happened after the last fix round wrote its entry, verifies the pins that
section listed, and adds the one deviation the earlier sections never declared.
The proposal now carries a matching dated As-built note
(`02-proposal.md`, "As-built (2026-11-10)") summarizing the same drift.

### Source is final, and narrow

Only three commits of this run touch `tools/orchestration/src`: `7ae67d7`
(the feature), `967dd3c` (rollback made terminal + the out-of-scope reverts),
`17710f6` (deviation 7); `moves.ts` appears in just the first two. The four
design-007 commits that also change `runner.ts` (`29087ca`, `ac7b03e`,
`95b7356`, `aff51e8`) add no line mentioning `moves`, `applyMoves`,
`validateMoves`, `headBefore` or `escalatesWithoutRetry`, so the moves path is
this run's alone. `gates.ts`, `state-store.ts` and `enforceOwnership` are
untouched across the whole set, so the sweep still has no move-specific branch
(Sweep interaction, as designed). `prompts/design-review.md` and
`prompts/test-authoring.md` — and the second sentence in `prompts/test-update.md`
("You own the test files you write … run the package formatter") — are design
007's `29087ca`, which entered through the `8f98292` merge of main;
`git blame` attributes every one of those lines to it. This run's prompt edits
are exactly the four files the proposal names.

### Deviation 8 — the ledger backfill (declared here for the first time)

`finalizeStage` (runner.ts:1592-1602) walks `state.results` backwards and
rewrites the accepted attempt's `failure` to the `moves-apply-failed:` /
`moves-rollback-failed:` text when the apply fails. The proposal never mentions
the results ledger. It is needed because that entry is pushed when the reply
validates — before the apply window runs — and the retry decision, the gate log
and the operator-facing results all key on it. No checkpoint shape change:
`failure` is an existing optional field, `version` stays 1 (A13 holds). The
round-1 code reviews read it (correctness-r1c1:82, security-r1c1:33,
testing-r1c1:17) but no stage had written it into this file.

### Pin 11 and the test-pin list, resolved

- Pins 1-10 all landed, in `ddd707b`, `0b16fd0`, `582b488`, `2848581`.
  Confirmed at HEAD: `moves-stage.test.ts:349-357` re-asserts the preserved
  state through `git status --porcelain --untracked-files=all`;
  `budget.test.ts:331` carries `minNudgeIntervalMs: 0`;
  `reply-contract.test.ts:240-253` pins the class through
  `escalatesWithoutRetry`; `moves-rollback.test.ts` is the driver-level pin;
  `moves.test.ts` carries the contracts, the `:(literal)` pathspec pins, the
  rule-clause gaps and the `node:fs` fault injection.
- Pin 10 (the "a rule that throws" claim) went the force-the-throw way:
  `reply-contract.test.ts:37-63` arms a `node:child_process` fault that makes
  the validator's `git ls-tree` spawn throw, and `:574-596` asserts the attempt
  fails `invalid-reply: moves — …`. The `from: 'CON'` case stays at `:525-561`
  as a plain rule-1 rejection whose comment now says `CON` alone never throws,
  which is the correction pin 10 asked for.
- Pin 11 (formatting) cleared in `107c167`, which reflowed `moves.test.ts` and
  `moves-rollback.test.ts`; `moves-stage.test.ts` was already clean. The
  formatting-only history is `ddd707b` → `29a6bc1` → `107c167`.
- The `budget.test.ts` working-tree hazard named above (a `fixPrompts`/`keyOf`
  block referencing an undefined helper) is gone: the moves case is now the
  hand-written saturating stub at `:285-347` with one `invalidFix` reply.
- Pin 9 landed as `describe.runIf(canMountTmpfs())` (`moves.test.ts:626`), so
  the EXDEV pair is reported skipped rather than passed on an unprivileged host.
  `moves.test.ts:675-697` are the platform-gated A10 blocks: the Linux
  case-variant contrast runs here, the darwin one does not, so A10 stays
  unverified exactly as the assumption register says.

### Gate history since the last entry

The run escalated once and the operator resumed it; that sequence, not a new
implementation problem, is what closed the gap:

- `testsGreen` red repeatedly (`npm run check red`, format-only) — the state the
  previous section left.
- one `stageFailure`: `test-update: independence violation: execute accessed run
  records` — the seat quoted test source out of a run-record path in its own
  command; the round retried.
- `escalate: tests not green at the code-review budget limit`, which wrote
  `ESCALATION.md` (`fe9c2d2`).
- `operatorExtension`: the operator extended the review budget by 3 review
  sessions over the 40/20 caps (and 3 revise rounds over the backstop of 5);
  `ESCALATION.md` was removed in `99caa1b`.
- `8f98292` merged main (design 007, `audit-by-deeds`) into this branch; that
  merge is why the range's diff lists 007's files, and nothing in it touches the
  moves code.
- `107c167` landed the formatting; `testsGreen` then went green.
- `code-review` r1 ratified: "all five seats approve the current revision
  stamp" at `b1c66ae5`.

State now: `status: running`, `lastStage: code-review`, `codeRound` 1 /
`codeCycle` 3, `designBudgetUsed` 3, `codeBudgetUsed` 6, `testsGreen: true`,
`testsNeedUpdate: false`. The testing seat ran the gate itself at r1c2: lint
clean, prettier clean, `31 files, 316 passed | 2 skipped` — the two skips are
the tmpfs/EXDEV probe.

### The testing seat's two carry-forwards, settled here

1. *minor — the deviation-7 relabel is untested.* Not accurate as filed. That
   seat checked `reply-contract.test.ts`'s `sequenceStubServer` stubs, which
   every time answer each prompt with parseable JSON and so exit through the
   parse/schema path. `budget.test.ts:275-347` uses its own saturating stub: it
   answers once with a validator-rejected moves reply, then goes quiet, so the
   attempt dies on the exhausted-nudge/empty-reply branch — and `:343` requires
   `invalid-reply: moves` among the `fix` attempts' ledger failures, which only
   the `lastMovesIssue` relabel can produce. The branch is pinned, budget
   strategy #10 being the reason it exists.
2. *nit — the no-base-revision fail-closed guard (runner.ts:1068-1069) is
   unreachable and unpinned.* Recorded here as intentional, per the seat's
   requested remedy: only `executeStage` passes `headBefore`, and it always has
   one, so the guard is defense-in-depth against a future caller. It fails
   closed rather than applying an unvalidated list. No pin added.

### Still open at HEAD

- `docs/adrs/005-runner-executed-file-moves.md` is still `status: proposed`;
  the flip belongs to this run's done-gate (Rollout 1), which has not run.
- The dogfood acceptance (Rollout 4) needs the next file-moving run.
- A10 (case-insensitive checkouts) remains unverified: the darwin block never
  executes on this host, and the "Repository spanning mount points" row still
  rests on a host with mount capability for the EXDEV case.
- The run's `.pipeline.json` carries an uncommitted working-tree delta; that is
  a run record, not source.
