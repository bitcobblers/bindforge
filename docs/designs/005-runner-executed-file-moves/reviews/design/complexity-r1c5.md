# complexity review r1 (ratification pass c5)

verdict: approve

Scope: the blob diff 188d705..44c52fa of 02-proposal.md, plus the full current
document, checked only for regressions against what the complexity seat approved
in round 1. Inputs: 02-proposal.md, docs/adrs/005-runner-executed-file-moves.md,
the Revision log, and the seat's own round-1 findings.

## Checklist walk-through

1. **New modules/interfaces vs existing reuse.** One new module (`moves.ts`,
   `validateMoves`/`applyMoves`); the revisions continue to lean on existing
   machinery rather than duplicating it: the shared `stagePathAllowed` predicate
   (validator rule 2/3, apply guard, sweep — "one predicate"), the existing
   schema-failure branch for validation errors, the extended (not new)
   `auditSessionIndependence`, and the existing `git()` helper's env widened to
   all runner git calls. No reuse miss found in the changed text.
2. **Moving parts trace to a Goal.** Every part added since r1 traces: the
   schema bounds (`.max(200)`/`.max(64)`) bound what one reply can make the
   runner spend, serving the fail-before-touching goal; rule 5's parent check
   reclassifies ENOENT as a reply error, serving the same goal; the rules 4–7
   apply guard replaces the old per-rename TOCTOU re-check (not additive — it
   subsumes it); `moves-rollback-failed` and `git-plant-detected` reuse the
   existing escalate-to-operator path rather than inventing a new one; the
   shared config-free git env and the `subagent` deny serve the item-5 goal,
   which ADR-005 Decision item 5 and its Context (worktree-planted config
   executing through permitted git) put in scope. No YAGNI part found.
3. **A 30–50% smaller design?** A variant that drops the audit matcher and the
   shared env, keeping only the `.git` deny, is smaller but fails the item-5
   ship case: in a regular (non-linked) checkout the sweep's dirty-path listing
   never sees `.git/**` content, so a planted `.git/config` survives the sweep
   and the escalation's own `ESCALATION.md` commit would execute it at runner
   privilege. The ADR's own Alternatives paragraph ("plant hooks or repo-local
   config that any permitted `git` invocation then executes") makes that the
   threat model to close. The added pieces are containment for a hole the
   design itself proves real; not a reducible slice.
4. **New dependencies.** None. The git ≥ 2.32 floor is a compatibility cost of
   `GIT_CONFIG_GLOBAL`, which has no pre-2.32 stdlib substitute (per-invocation
   `-c` cannot suppress config read from `~/.gitconfig`/includes). Reasonable.
5. **Deferrable second half?** The second half (item 5) is mandated by ADR-005
   Decision item 5; deferring it entirely contradicts the ADR and the proposal
   already rejected that (Options, "Land `moves` without item 5"). The runner-env
   hardening moved from the r1 Deferred list into Chosen design — that is a
   scope pull-in, but a consistent one: r1 deferred it only on the premise that
   the `.git` write-deny "closes the repo-local half," and the r3 audit path
   establishes a plant can exist anyway. Not a finding; noted so the reviser
   knows why the seat no longer treats it as deferrable.
6. **Knobs without consumers; one-implementation abstractions.** The schema
   caps are validator bounds, not configuration; the single `stagePathAllowed`
   predicate is the opposite of an abstraction layer. None found.
7. **Value check.** The design grew since r1 (audit matcher, shared git env,
   `subagent` deny, extra failure classes), but the growth closes demonstrated
   gaps (audit-invisible `.git` plants; `~/.gitconfig`/system config at runner
   privilege; child-agent route) rather than adding speculative generality —
   and the r3 revisions actually *shrank* one r1-approved area by dropping the
   argv deny refinements ("flag patterns do not survive abbreviation").
   Complexity looks proportionate to what the feature brings.

## Regression check against r1-approved text

- Item-4-as-invariant, no-`git-mv`, rollback, checkpoint additivity, the
  ownership sweep untouched: all still hold in the changed text; nothing in the
  diff contradicts ADR-005 Decision items 1–5 or its Alternatives.
- The r1 nit on the directory-moves rationale survives the compression of that
  option (below).
- New rule 6 case-fold, rule 5 parent check, `git --no-optional-locks status`
  spelling, git-floor row, and Test strategy #12/#13 are internally consistent
  with the Goals, Risks, and Deferred sections (the case-fold *rejects*
  case-only moves, which matches the Deferred entry deferring only *support*;
  #12's `notes.md` → `readme.md` pair is a rule-5 disk-check case, not a
  rule-6 fold case, and the matrix/Risks text says exactly that).

## Findings

- [nit][re-raised] Options considered — "Allow directory moves" — "file-only
  drops that whole rule class." — the r1 wording ("File-only removes that
  entire rule class.") was compressed but the overclaim survived: the rule
  class is not dropped whole, because validator rule 6 still carries "no
  listed path a path-prefix of another," which is needed for file pairs
  (`to1=a`, `to2=a/b`), and rule 4's blob-at-`headBefore` check still exists in
  file form. An implementer could read the rationale as license to drop the
  prefix clause. The Revision log has no entry recording this finding as
  rejected, so it is reported unchanged at the new anchor. Fix: "file-only drops
  the self-containment-as-directory-rule, the cascade rules, and the per-child
  tracked-at-`headBefore` check; rule 6's path-prefix clause stays for file
  pairs."

## FYI (no severity)

- The `auditSessionIndependence` matcher over raw shell command text will
  false-positive on read-only commands that merely mention a `.git` path
  (e.g. `git log -- .git/...`), escalating stages that committed no plant —
  false-positive/miss analysis of the text matcher is the security/correctness
  seats' ground, not this one.
- The design concedes the child-agent `.git` route is covered only by an
  advisory `subagent` deny while calling agent-config permissions advisory;
  whether that residual is acceptable is the security seat's call.
