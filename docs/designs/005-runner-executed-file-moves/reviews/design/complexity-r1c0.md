# complexity review r1

verdict: approve

## Checklist walk

1. **Reuse.** One new module (`moves.ts`, two exported functions). The design
   reuses the existing choke points instead of duplicating them:
   `stagePathAllowed` (verified at gates.ts:260 — currently `implement`/`fix`
   return true for anything non-test, so the `.git` first-line is the right
   place to add the deny), the existing git helper for `cat-file`, the
   `invalid-reply:` retry path, and `commitStage`'s `git add -A`. No
   pre-existing path-validation or realpath utility exists in `tools/`
   (grep for `realpath` finds nothing), so rule 7 is new-but-unavoidable,
   not duplicate code. `finalizeStage` already receives `headBefore`
   (runner.ts:1041), so the apply hook needs no new plumbing at that end.
2. **Trace to Goals.** Every moving part maps: reply field + validator +
   retry wiring → Goals 1–2; apply + reverse rollback + disjointness → Goal 3;
   `.git` deny + allowlist → Goal 4 (item 5 is ADR-decided, not YAGNI). The
   deny refinements are load-bearing, not decoration: `--output=<file>` is a
   direct write lever that the `.git` deny does not disarm.
3. **Smaller design?** The obvious 30%-smaller variant — drop the rollback and
   accept a half-applied list on apply failure — was considered (Options 3) and
   the rejection is sound: with `.pipeline.lock` single-writer the rollback is
   ~15 lines and its exactness is already guaranteed by rule 6's disjointness.
   Splitting item 5 into a second merged slice is possible but the moves
   validator rule 2 leans on the same `.git` deny line, so one branch is the
   thinner total. No smaller design survives.
4. **New dependencies.** None; stdlib `fs.renameSync` plus the existing git
   helper. Compatible with the pinned Node ≥ 22.12.
5. **Deferrable second half?** No meaningful seam: item 5's precondition
   (`.git` deny) is shared with moves validation, and the allowlist itself is
   ~6 config lines.
6. **Knobs / one-impl abstractions.** Explicitly none — Rollout 3 states no
   flag; `MovePairSchema` is shared across three schemas rather than three
   copies; no interface is introduced with a single implementation.
7. **Value check.** Proportionate. The feature unblocks a documented
   livelock that burned a full code budget (run 004), and the added machinery
   is one small module, one parameter, one first-line predicate change, and
   config lines — no new subsystem.

## Findings

- [nit][new] Options considered, "Allow directory moves" — "File-only removes
  that entire rule class." — overclaims against the design's own validator
  rule 6, which keeps "no listed path is a path-prefix of another" (a
  superset of the self-containment check the sentence says is removed; it is
  still needed for file pairs, e.g. `to1=a` with `to2=a/b`). An implementer
  could read the rationale as license to drop the prefix clause. Fix: narrow
  the claim to what file-only actually removes — "removes the cascade rules
  for children and the per-child `tracked-at-headBefore` check; the
  path-prefix disjointness clause stays in rule 6 for file pairs."

## FYI (no severity)

- Validator rules 4/5 spawn one git subprocess per endpoint (`git cat-file -t
  <headBefore>:<from>` plus the `to`-tracked check); a single
  `git ls-tree -r <headBefore>` snapshot would answer both halves in one call
  — performance seat's territory, not a complexity objection.
