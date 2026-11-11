# correctness review r4

verdict: request-changes

## Scope

- Named scope target: `docs/designs/005-runner-executed-file-moves/reviews/design/editorial-r4.diff`.
  A direct read of that exact path (absolute and repo-relative) returns "File not found" — the
  round-4 editorial diff is not present in this worktree. No listing, glob, or grep of the
  `reviews/` tree was performed, and no report or transcript was opened.
- Inputs read: `02-proposal.md`, `docs/adrs/005-runner-executed-file-moves.md`, the Revision log
  inside the proposal. To settle whether the pass's subject exists under a sibling name, one exact
  adjacent path was read: `reviews/design/editorial-r3.diff` (an editorial diff artifact, not a
  report or transcript). Nothing else in that directory was opened.
- Subject coverage where it could be established: the r3 language diff is applied verbatim in the
  current proposal — every `+` side is present in Context, Non-goals, Reply field, validator rule 5,
  rule 6, Wiring into retry, and Apply. All seven edits are meaning-preserving: the comma before
  "`because its 23 test-file moves`", "`at any scope`", "`are added to the … entries`", "`a list over
  either cap`", rule 5's "`does not exist anywhere`", rule 6's "`What remains is a matching`", the
  validator-failure apposition, and the Apply rewrap ("`as that of a rename error`", "`which is exact
  because the pairs are disjoint`", "`leaving the preserved-worktree state that a failed reply leaves
  today`"). None of them changes the rules, failure classes, or call sites, and no hunk is half-applied.

## Findings

- [major][new] Scope (checklist 7 — "are the previous round's findings closed for real") — the caller
  named `reviews/design/editorial-r4.diff` as this pass's subject and that file does not exist, so
  there is no round-4 diff to check and no way to confirm that the round-4 revision left the design
  correct rather than only the r3 text. Fix: re-emit the round-4 editorial diff at the named path
  (or restate the named path to the diff that was actually produced) and re-run this seat against it.
  Not a proposal defect — a run-artifact gap; the gate fails because an unevaluated checklist item
  is a finding, not a pass.
- [nit][new] Chosen design / Reply field — "a list over either cap fails `safeParse` like any invalid
  reply" — a list cannot exceed the per-endpoint bound, so "either cap" attributes an endpoint
  property to the list; Test strategy #1 already states it correctly ("an endpoint over the length
  cap, and a list over the pair cap"). Fix: write "an endpoint over the length cap or a list over the
  pair cap fails `safeParse`", replacing the current clause rather than adding to it.

## Checklist coverage

1. **Assumption register** — for every row touching the diff-touched passages: A4 (sweep with
   `--no-renames`, revert only on `stagePathAllowed`, restore from `headBefore`) is used consistently
   in Sweep interaction, Apply, and #4; A5, A8 spot-checked below; A12 matches the ADR Context
   (`tests not green at the code-review budget limit`, six rounds, 23 files). No `verified` row in
   the touched text is contradicted by its basis, and A10 remains honestly `unverified` with the run
   004 precedent cited rather than asserted as fact.
2. **Source spot-check (anti-hallucination gate)** — three cited URLs fetched live:
   `https://man7.org/linux/man-pages/man2/rename.2.html` supports A8 (atomic replacement; `EISDIR`,
   `EINVAL` — "The new pathname contained a path prefix of the old", which is also Options 4's
   directory-move argument — and `EXDEV` across mount points, matching the Compatibility matrix row);
   `https://git-scm.com/docs/git-diff` supports A5 ("The default similarity index is 50%", and
   `--no-renames` "even when the configuration file gives the default to do so");
   `https://git-scm.com/docs/git-config` supports A6's multi-source claim (`.git/config`,
   `~/.gitconfig`, `$XDG_CONFIG_HOME/git/config`, `/etc/gitconfig`, `include.path`) and A10's
   mechanism (`core.ignoreCase`, default false, probed at clone/init). No dead or contradicting source.
3. **Internal contradictions** — none introduced: rule 5's parent-exists requirement, rule 7's
   "of `to`'s parent (rule 5)", the Apply guard ("re-runs rules 4–7"), and #5/#11 agree; the failure
   vocabulary `invalid-reply: moves` / `moves-apply-failed` / `moves-rollback-failed` is identical in
   Apply, Risks, #6, #9 and #11; "see Options 1" and "(Options 5, #4)" resolve to real Options
   entries; #12's macOS contrast matches the Compatibility matrix and A10.
4. **Missing edge cases** — the touched text adds no new state; empty list, swaps, chains, case-only
   pairs, missing parents, symlinks escaping the repo, and mount-crossing pairs remain covered by
   rules 1–7 and #2/#5.
5. **Interfaces** — `MovePairSchema`, `moves`, `validateMoves(repoDir, headBefore, stageName, moves)`
   and `applyMoves(repoDir, headBefore, moves)` each appear once in each shape; only the cap wording
   above is imprecise (nit).
6. **Source document** — ADR items 1–4 are all decided, and the deferrals (directory moves, case-only
   renames, OS-level protection, checkpoint trust, read-only git item 5) match the ADR's own Deferred
   and Consequences clauses. No deferral contradicts the ADR.
7. **Revision rounds** — cannot be evaluated for round 4; see the scope finding.

## FYI (no severity)

- ADR item 3 says the runner performs the renames "through its own git plumbing", while the design's
  primitive is `fs.renameSync` plus the existing `commitStage` `git add -A` (Options 2, Revision log
  r0/OQ1). The r0 rebuttal (a per-pair subprocess only multiplies A6's config surface; detection is
  diff-time per A5) is sound, so this is not re-litigated — a half-sentence naming `git add -A` as
  the plumbing would just close the wording gap between the two documents.
- Rule 1's blanket ban on `:` in endpoints is stricter than POSIX requires; it is deliberate
  cross-platform parity and is pinned by #2. Compatibility/complexity seat's call, not mine.
