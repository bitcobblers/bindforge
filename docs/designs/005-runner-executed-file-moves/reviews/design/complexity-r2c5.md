# complexity review r2 — ratification pass c5

verdict: approve

Scope: diff 82b346f..2168fac of 02-proposal.md, plus the full current document,
checked against docs/adrs/005-runner-executed-file-moves.md and the Revision log.

## Round-2 finding status

- My r2 finding (finalizeEscalate branch keyed on the `git-plant-detected`
  reason string) is **resolved by deletion**: the entire Read-only git
  (item 5) section, the audit matcher, the `git-plant-detected` class, the
  git-env pin, and the config allowlists are gone from the document. `grep`
  over the current text finds no surviving reference to any of them, and
  Test #6 now names only `invalid-reply:`, `moves-apply-failed` and
  `moves-rollback-failed`. Nothing in the changed text reintroduces a
  third stringly-typed classifier.

## Diff coherence (what the revisions touched)

- The deferral matches the ADR verbatim in substance: ADR-005, Decision,
  "Deferred — read-only git in stage shells ... deferred to a follow-up ADR.
  Items 1-4 stand without it." Context, Non-goals, Deferred, and the Revision
  log all state items 1-4 only. No contradiction with the source ADR — per
  checklist step 5, the deferral is not re-opened.
- Removals are complete and mutually consistent: item-5 goal bullet, the
  "Land moves without item 5" option, the A9 assumption (only item 5's
  allowlist ordering consumed it), the git-config risk row, the #4
  stagePathAllowed `.git` assertions, the #6 `git-plant-detected` clause, the
  #13 ship-gate block, and the 2.32 git floor all disappear together; no
  dangling "(see Read-only git)" cross-reference survives.
- The `.git` endpoint denial survives as validator-local rule 2, re-run
  resolved by rule 7 (so the apply guard's "rules 4–7" re-check covers it in
  resolved form), and its test coverage moved intact from #4 to #2's
  `.git`/`.GIT`/`src/.git/config` cases — the smaller mechanism is still
  pinned. This is a strict simplification of what r2 approved: one shared
  predicate edit became one validator rule.
- "the only git call this change adds is the validator's `git cat-file -t`"
  (Compatibility) is true for the surviving design (apply uses
  `fs.renameSync`; staging stays the existing `commitStage` `git add -A`),
  and `cat-file -t` predates any git this repo runs, so "no new floor" holds.

## Checklist

1. No new package/module beyond `moves.ts`; the `.git` check no longer edits
   the shared `stagePathAllowed`, which is the smaller shape. OK.
2. Every surviving moving part traces to the three Goals; the item-5
   machinery that didn't trace to the move goal is now Deferred. OK.
3. The c5 revision *is* the smaller design this checklist asked for. OK.
4. No new dependencies. OK.
5. Second half deferred: exactly what happened (items 1-4 now, item 5 in a
   follow-up ADR). OK.
6. No knob or abstraction left without a consumer — one residual dangling
   phrase, see finding.
7. Value check: proportionate — schema field, one validator, apply with
   rollback, four one-sentence prompts.

## Findings

- [nit][new] Assumption register, A6 — "The four read-only-named commands
  carry write/exec-capable config ... `git --no-optional-locks` aside" —
  after item 5's removal the proposal never lists those four commands, so
  "the four read-only-named commands" has no antecedent in this document,
  and the `--no-optional-locks` aside names a spelling that no longer occurs
  anywhere in it. A6 itself must stay: Options 1 and 2 still cite it to
  justify rejecting agent git access and `git mv`. Fix: reword the lead-in
  to "the read-only git commands ADR-005 defers to a follow-up ADR (`status`,
  `log`, `diff`, `show`)" and drop the "`git --no-optional-locks` aside"
  clause; wording only.

## FYI (no severity)

- The Deferred sandbox bullet still reads "config writes landing outside the
  worktree or at its root with no `.git` segment" — the phrase outlives the
  audit matcher it was carved against but remains coherent against the
  validator-local `.git` rule; wording taste, not a finding.
