# Code review — security — round 1, dissent pass c1

Scope: `git diff 29a6bc141f8a6420fb372271ff48d81dcb83202e..HEAD -- ':(exclude)docs/designs'`,
read against `docs/designs/005-runner-executed-file-moves/02-proposal.md` and
`docs/adrs/005-runner-executed-file-moves.md`. Dissent pass: this pass checks the
revision written against my round-1 findings first, and reports anything else
only where the changed code makes it true.

## Verdict: approve

## Round-1 finding re-check

### [was: major] `moves-rollback-failed` had no retry/escalate branch — RESOLVED

My round-1 finding: `runWithRetry` had no branch for the terminal class; a fresh
attempt 2 ran on the half-applied worktree and a successful attempt 2 committed
the half-moved pair inside a normal stage commit; the doc comment claimed
"starts nothing at all"; the class was unpinned.

Checked in the changed code:

- `escalatesWithoutRetry` (runner.ts:1700-1702, exported) matches the
  `moves-rollback-failed` prefix — a decision function distinct from
  `retryInSameSession`, which round 1 showed cannot tell the two classes apart.
- `runWithRetry` (runner.ts:2044-2053) checks it immediately after the
  `stageFailure` gate log and checkpoint and before attempt 2 is built: it calls
  `finalizeEscalate` and stops the driver via `StopDriving` (caught at
  runner.ts:2338). No second session is ever pointed at the damaged worktree —
  exactly the fix I prescribed, matching the proposal's Apply paragraph and
  Risks row 3 ("stops at the operator as moves-rollback-failed").
- Failure text reaches the gate keyed correctly: `finalizeStage`
  (runner.ts:1588-1607) builds the class-prefixed text and amends the attempt's
  ledger entry (reverse scan by sessionID — the entry pushed for the current
  attempt is the newest match), so the gate log, checkpoint, and operator-facing
  results all name the class.
- The misleading comment at the `retryInSameSession` docstring is corrected: it
  now states the third class never reaches the same/fresh-session choice
  because `escalatesWithoutRetry` stops it first (runner.ts:1677-1684).
- Pinning, both levels I asked for:
  - Unit pin: `reply-contract.test.ts` `describe('escalatesWithoutRetry')`
    distinguishes rollback (true) from `moves-apply-failed`,
    `invalid-reply: moves`, and `stage-timeout` (false) — the distinction
    `retryInSameSession === false` could not make.
  - Fault-injected integration suite: new `tests/pipeline/moves-rollback.test.ts`
    drives the real driver through `resumePipeline` with the undo rename armed
    to throw EIO: exactly one attempt, one session, one `stageFailure` gate
    entry, one code-budget unit, an escalation naming the class, the damaged
    pair left for the operator, and no stage commit ("the only history past the
    base is the terminal record"). Its control suite runs the same fixture with
    the fault disarmed and counts two fresh sessions for
    `moves-apply-failed`, making the terminal claim falsifiable rather than
    tautological. The previously `it.skip`-ed escalation test is gone; no
    `it.skip` remains in the moves suites.

Design goals "a failed apply never leaves a half-applied list inside finalize"
and the Risks-row mitigation are now enforced end to end: the half-applied pair
can never ride into a stage commit.

## Security checklist over the changed code (things the revision makes true)

1. **Boundary validation** — zod bounds plus `validateMoves` unchanged in
   strength; the new fail-closed branch in `movesValidationIssue`
   (runner.ts:1062-1072) rejects a moves-bearing reply that arrives without a
   `headBefore` instead of applying it unvalidated in finalize. Improvement.
2. **Path handling** — no new sinks; all git calls stay array-argv `spawnSync`
   (no shell). Validator rules 1/2/7 untouched and still pinned.
3. **Injection** — `treeMode` (moves.ts:56-73) now passes the pathspec as
   `:(literal)<path>`: closes the git glob-matching gap where an untracked
   `a?.ts` or a directory could mode-match a tracked sibling and pass rule 4,
   and legitimate `bindings[en].json`-style names failed it. `--` does not
   disable globbing; rule 1's colon ban keeps the magic prefix unpickable.
   Pinned by three validator tests (moves.test.ts:204-243), including the
   untracked-directory case that would have handed a whole tree to
   `renameSync`.
4. **Secrets/PII** — `basicAuthHeader` (opencode-runner.ts) is a refactor of the
   existing header construction; it is never logged, and the direct-HTTP
   fallback's error text carries only the URL pathname and status. New log/gate
   lines carrying failure or error text pass through `printable`/`brief`;
   `finalizeEscalate` sanitizes its reason.
5. **New dependencies** — none in the diff.
6. **Filesystem atomicity** — apply stays all-or-nothing (guard re-runs rules
   4-7 as one sweep before the first rename; reverse rollback); the
   post-failure `git add -A` staging was dropped, and moves-stage.test.ts now
   pins the preserved state as unstaged (`diff --cached --name-only` empty) with
   attempt 2's sweep seeing every path through `--untracked-files=all`
   (`dirtyFiles`, runner.ts:207-211). The rollback class cannot commit a
   half-applied list (moves-rollback.test.ts).
7. **Clean rejection** — the throw-from-a-rule shape is now fault-injected
   (reply-contract.test.ts git-fault mock) and lands as
   `invalid-reply: moves — …`, not a crash; the exhausted-nudge final failure
   now names the moves issue instead of `not JSON`, which keeps the class and
   the same-session retry semantics consistent for validator rejections.

Merged-in design-007 content (child-session walk, independence scan bounds)
reads clean for this checklist: bounded page crawl and node budgets,
`encodeURIComponent` on the URL path segment, deadline-clamped requests, no
auth material in any log line. It is 007's already-reviewed work arriving via
the main merge and is not judged further here.

## FYI (no severity)

- `finalizeEscalate`'s terminal `docs: … pipeline escalated` commit runs
  `commitStage`, whose `git add -A` folds the known-damaged half-moved worktree
  into that terminal record. This is the pre-existing escalation semantics the
  design equates to ("the preserved-worktree state that a failed reply leaves
  today") and is what my round-1 fix prescribed, so it is not a defect of this
  change — but operator tooling reading the escalate commit should know the
  damaged pair's endpoints sit inside it.

## Open findings

None. Zero blocker, zero major.
