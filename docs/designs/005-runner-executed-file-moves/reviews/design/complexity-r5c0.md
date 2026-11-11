# Complexity review r5 (ratification pass c0)

verdict: approve

Scope: `git diff 2168fac7 967d5e86` over `02-proposal.md`, then the full current
document against `docs/adrs/005-runner-executed-file-moves.md` and the Revision
log. Ratification bar applied: only report where the revisions broke something
approved in r5 or made a cited section false.

What the revisions did to the complexity shape (all checked, nothing broken):
- Rules 4–5 swapped `git cat-file -t` for `git ls-tree` and `existsSync` for
  `lstatSync` try/catch. Still one git subprocess per `from` plus one path
  lookup per `to`; no new module, no new dependency, no new knob. `git ls-tree`
  and `lstatSync` are as old as the previously cited primitives, so the
  "no new git floor" and "no new runtime dependency" claims in the Compatibility
  matrix still hold for the mechanism.
- The `git mv`-subprocess rejection (Options 2, "only multiplies A6's config
  surface") is unchanged and now better supported: the design still adds exactly
  one runner-side git call, not one per pair.
- A10 was reworded to `lstatSync` + dentry, matching rule 5 and Test #12. The
  assumption register stays internally consistent, so the sweep/case-fold layer
  I approved is untouched.
- Failure classes, the no-new-sweep-branch decision, the disjointness matching,
  and the rollout (no flag, default `[]`) are unchanged; the diff's remaining
  edits are comma/wording only.
- Value check: complexity is still proportionate to what the feature buys (run
  004's 23-move livelock, A12). No design 30–50% smaller is available that still
  meets Goals 1–3; the validator's rule count is what makes apply
  all-or-nothing, and nothing in the diff inflated it.

## Findings

- [minor][new] Compatibility matrix, Node row + Revision log, "Node ≥ 22.23.4
  (engines raised from ≥22.12.0, tools/orchestration/package.json:30-32)" and
  "Node floor raised to ≥22.23.4 (r4: … engines included)" — this is a second,
  unrelated change riding in the moves branch. No moving part of the design
  traces to it: `fs.renameSync`, `lstatSync` and `git ls-tree` are all far older
  than the current floor, so nothing in Goals 1–3 needs 22.23.4 — the
  justification is a security release, which is a different goal. It also
  contradicts the document's own scope statement, "this change ships items 1-4
  only" (Context), and Rollout #1 merges it as one branch, so the ADR's
  file-moves capability and a repo-wide runtime floor rise can no longer land or
  be reverted independently, and the r5 text I approved said the opposite ("no
  new floor … operators expected on the current patched 22.x"). Fix: take the
  `engines` raise out of this change (separate `chore:` commit/run) and leave the
  row as the security-release note only — or, if it must ride along, add one
  sentence to Rollout naming it as a prerequisite outside items 1-4 and saying
  what happens to a stage shell that runs below the floor. Wording/branching
  only; no mechanism change.

- [nit][re-raised] Assumption register, A6 — "The four read-only-named commands
  carry write/exec-capable config … `git --no-optional-locks` aside" — unchanged
  by the revisions, so it is still open: "the four read-only-named commands" has
  no antecedent inside this document (the four are spelled out only in ADR-005's
  deferred item 5), and `--no-optional-locks` is a spelling that occurs nowhere
  else in the current text. The diff slightly worsens the ambiguity, because the
  only git commands the body now names are runner-side (`git ls-tree`, `git add
  -A`, `git show --name-status -M`) plus `git diff*`/`git log*` in Deferred, so
  nothing left invites the reader to count four. The assumption itself must stay:
  Options 1 and 2 still cite A6. Fix: reword the lead-in to "the read-only git
  commands ADR-005 defers to a follow-up ADR (`status`, `log`, `diff`, `show`)"
  and drop the "`git --no-optional-locks` aside" clause; wording only.

## FYI (no severity)

- Rule 4's new `git ls-tree <headBefore> -- <from>` is a runner-side git call in
  the stage window; it does not refresh the index, so A6's `core.fsmonitor` sink
  and the pager/diff sinks do not reach it — worth a line of confirmation from
  the security seat, since it is the one git invocation the design adds.
- Rule 4 says `from` "still exists on disk" without naming the primitive, while
  rule 5 now names `lstatSync`; the dropped parenthetical ("its dirty content is
  what rides along") was the only place that explained the check. Implementation
  detail, not a design gap.
