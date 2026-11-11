# Code review — security — ratification pass c2

Scope: `git diff 107c1672b6f33b0cae9117cdc751abb77f75bc68..HEAD -- ':(exclude)docs/designs'`, read
against the full current `02-proposal.md` and ADR-005. This pass checks only that the revisions
landed after the r1c1 approval did not break what the security seat approved. Prior security
findings: none.

Note on method: shell/git was unavailable in this environment (permission denied), so the review was
performed on the full current contents of the paths the diff range touches — `moves.ts`, the
`types.ts` schema block, the `runner.ts` moves wiring (`movesValidationIssue`, `executeInference`,
`finalizeStage`, `retryInSameSession`, `escalatesWithoutRetry`, the driver stop), `gates.ts`
unchanged `stagePathAllowed`, `package.json`, the four prompts, and the moves test suites.

## Checklist evaluation

1. **Boundary validation.** `MovePairSchema` (`types.ts:32-35`) min 1 / max 200 per endpoint, list
   max 64 with default `[]` (`types.ts:44`), exactly the proposal's bounds; a missing field parses
   to `[]` and over-cap inputs fail `safeParse` (reply-contract.test.ts:337-371). No code
   evaluation anywhere on the reply path. Pass.
2. **Path handling.** Rule 1 rejects absolute paths, `.`/`..`, empty segments, `\`, `:`, control
   chars incl. NUL, Windows reserved names (`con`, `com1`-style with and without extension), and
   trailing dot/space segments — pinned per clause in moves.test.ts:245-347. Rule 2 rejects
   `STATE_FILE`/`LOCK_FILE` segments and case-folded `.git` segments (moves.ts:134-147), including
   the nested-`.git`-in-owned-dir and `.GIT/hooks` cases (moves.test.ts:349-405). Rule 7 confines
   the realpath of `from` and of `to`'s parent to the repo realpath and re-runs the `.git` test on
   the resolved path (moves.ts:211-233), with the symlink-escape and symlink-into-`.git` pins
   (moves.test.ts:507-527) and the linked-worktree pointer pin (moves.test.ts:705-722). No
   string-interpolated shell: the only git call is `spawnSync('git', argvArray)` (moves.ts:51-54).
   Pass.
3. **Injection.** No shell. The one git pathspec is `:(literal)`-prefixed (moves.ts:66-73): this is
   the post-approval deviation-5 fix, and it is a security improvement — it closes the earlier
   glob-borrow where an untracked `a?.ts` could pass rule 4 on a tracked sibling's mode and an
   untracked directory could move through a glob-matched mode lookup; both are now pinned
   (moves.test.ts:204-243). Escaping the magic prefix is unreachable because rule 1 has already
   rejected any `:` in an endpoint; older git without magic signatures fails closed (no match →
   reject). No regex with backtracking risk; no untrusted regex construction. Pass.
4. **Secrets/PII.** None in source or fixtures (test emails are `fixture@example`); log/nudge lines
   echo only schema-bounded path text. Pass.
5. **Dependencies.** No new runtime or dev dependency in the touched package layout. The
   `engines: >=22.23.4` bump (package.json:30-32) was verified against the Node.js advisories:
   22.23.4 is the 22.x line of the 2026-10-21 security release fixing CVE-2026-111497 (Brotli
   decoder OOB write / RCE) among others; the bump raises the floor to the fixed line, which is a
   hardening, not an exposure. Pass.
6. **Filesystem writes.** `applyMoves` re-runs rules 4-7 as one guard immediately before the first
   rename (moves.ts:333-338), renames with atomic `renameSync` under the `.pipeline.lock`
   single-writer invariant, rolls completed renames back in reverse on the first error
   (exactness rests on the rule-6 disjointness matching), and touches nothing outside the repo
   realpath. Matches the approved Apply design. Pass.
7. **Clean rejection.** `validateMoves` returns a plain issue naming the offending pair; an internal
   throw is caught in `movesValidationIssue` (runner.ts:1071-1075) and lands as
   `invalid-reply: moves — <issue>` — pinned by the throwing-validator test
   (reply-contract.test.ts:574-596). Guard refusal and rename errors take `moves-apply-failed`
   with no sweep, no staging, no commit, and a byte-identical worktree after rollback
   (moves-stage.test.ts:306-330, moves.test.ts:600-620); a failed rollback is terminal
   (`moves-rollback-failed` → escalate, no attempt 2 on the damaged tree;
   runner.ts:2044-2053, moves-rollback.test.ts:195-262). Failures never reach users as stack
   traces. Pass.

## Post-approval fixes checked

- Failure representation (returned string / `MoveApplyFailure{error, rollbackFailed}` instead of
  throwing): preserves the invalid-retry/apply-fail/escalate trichotomy exactly as approved; the
  discriminator and both retry classes are pinned. No regression.
- `:(literal)` pathspec: strictly narrows rule 4; no new surface; fail-closed on old git. No
  regression.
- `LOCK_FILE` constant moved to `types.ts:168` to break the validator↔runner import cycle: all
  three call sites (runner.ts:384, 408; moves.ts:139) reference the same constant; lock behavior
  and reserved-segment rule unchanged. No regression.
- Ledger amend on apply failure (runner.ts:1597-1602): keeps the checkpoint honest about a failed
  attempt next to a preserved worktree; an `ok` ledger on a known-damaged tree would have been a
  trust defect — this is the correct direction. No regression.

## Verdict

No security regression introduced by the post-approval revisions. Every checklist item passes on
the current tree.

verdict: approve
findings: none
