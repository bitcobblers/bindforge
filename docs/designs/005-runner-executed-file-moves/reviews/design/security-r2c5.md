# security review r2 (ratification pass c5)

verdict: approve

Scope: the blob diff 82b346f → 2168fac of 02-proposal.md (the item-5 removal
and its ripple), plus a full re-read of the current document against the
security checklist. Inputs: the current 02-proposal.md,
docs/adrs/005-runner-executed-file-moves.md, the Revision log, and my own
round-2 findings (none). No review reports or transcripts opened.

## Checks that hold after the revision

- **Deferral is ADR-consistent.** The removal of item 5 (audit matcher,
  git-env pin, config allowlists, `subagent` deny) matches the ADR's explicit
  clause "Deferred — read-only git in stage shells … It is deferred to a
  follow-up ADR. Items 1-4 stand without it" (ADR-005). The proposal's
  Deferred entry restates the same reasons ("separable and non-terminating").
  This is the source document handing the concern away, not the proposal
  dropping it on its own — not re-openable per the deferral rule.
- **`.git` protection for the moves surface survives the split intact.**
  Round 2 leaned on the item-5 `stagePathAllowed` first-line `.git` rule for
  validator rule 2; the revision replaces that dependency with a
  validator-local case-folded `.git` segment rule (rule 2) and — critically —
  keeps the apply-time coverage: rule 7 ("rule 2's `.git` segment test then
  runs on the resolved path relative to that realpath") is inside the guard
  set "rules 4–7" that `applyMoves` re-runs immediately before the first
  rename, so the TOCTOU symlink-plant window is still covered, pinned by
  #2's "a symlinked endpoint escaping the repo or landing in `.git`". Test
  #2 keeps the `.git`, `.GIT/hooks/pre-commit` and `src/.git/config`
  endpoint rejections. Dropping those assertions from #4 is correct — they
  pinned a `stagePathAllowed` change that no longer ships.
- **Failure behavior stays fail-closed.** Removing `git-plant-detected` from
  #6 and the risk table removes only item-5 outcomes; the moves classes
  (`invalid-reply:`, `moves-apply-failed`, `moves-rollback-failed`) and the
  validate-all-before-touching / rollback / no-half-apply properties are
  unchanged.
- **Dependencies (checklist 5).** The only git call the change still adds is
  `git cat-file -t` (long-stable plumbing, rev argument prefixed by
  `headBefore` so endpoint text cannot be parsed as an option); dropping the
  2.32 floor row is truthful once `GIT_CONFIG_GLOBAL` is gone. No new
  runtime dependency.
- **A6/A7 kept and still load-bearing** for the Options-1/2 rejections; A9
  removed cleanly with no dangling reference.

## Findings

- [nit][new] Assumption register (A6) — "The four read-only-named commands
  carry write/exec-capable config" — the four commands were item 5's allowlist;
  with item 5 removed, no section of this document names four read-only git
  commands anymore, leaving a dangling descriptor on an otherwise live
  assumption (still cited by Options 1 and 2). Fix: reword A6's opening to
  something like "The read-only-spelled git commands (`status`, `log`,
  `diff`, `show`) carry write/exec-capable config" — an assumption about git
  itself, not about a shipped allowlist. Non-blocking.

## FYI (no severity)

- A7's pre-existing exposure (stage agents can write `.git/**` today) is now
  mitigated for this change only by the validator-local `.git` rule on the
  moves surface; the sweep-level `.git` refusal that round 2's design added
  now travels with the deferred follow-up ADR. That is exactly what the
  ADR's Deferred clause sanctions, so it is not a finding here — but the
  follow-up ADR should inherit A7 as a live premise.
- A9's removal leaves the register numbering skipping A9 while keeping A10+;
  harmless stability of old ids, worth leaving as-is (correctness/style
  seat's call if they care).
