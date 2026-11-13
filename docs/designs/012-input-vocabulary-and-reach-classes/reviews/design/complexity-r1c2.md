# complexity review r1c2

verdict: request-changes

Ratification pass: scope is the diff between the two `02-proposal.md` blobs
(`17aee59` → `b08752d`) plus the current document. r1 had no complexity
findings from this seat, so only defects introduced by the r1 revisions are
reported.

## Findings

- [major][regression] Placement legality (`placement.rs`, predicate clause 4)
  — "The `PlacementCheck` result carries the shape delta when the moves agree
  and the per-move deltas when they do not, so a caller (ADR-015's
  precondition, 015's policy layer) can explain a refusal instead of seeing a
  bool." — the r1 rewrite replaced "the per-action degradation" with "the
  per-move deltas". "Delta" in this predicate is a coordinate delta (clause 2
  defines it: `coordinate(target) − coordinate(source)`), so the revised
  sentence leaves the shipped result type carrying only coordinates. A bound
  refusal — clause (b), the refusal 015 actually gates on — is now explainable
  only as "some move degraded", not by how much or which action; the sentence's
  own purpose clause ("explain a refusal instead of seeing a bool") is false
  for that case, and the implementer is left to invent the field of a public
  type 015 preconditions on. The Revision log's entry ("one `PlacementCheck`
  delta shape") describes unifying the shape, not deleting the degradation, so
  this reads as an editing loss rather than a decision. Fix: keep the single
  delta shape and put the degradation back in the same sentence —
  "carries the per-move coordinate deltas, one common `(dx, dy)` when the moves
  agree, and each move's degradation" — which is what the r1 base said and is
  not an added mechanism.

- [minor][new] The model (`reach.rs`) — "`load_reach_tables` gains two checks,
  the first for **every** reach table at load, v1 directories included" now
  sits in the same paragraph as "the v1 path keep their exact shipped behavior
  (the existing `reach_tables.rs` suite stays green unchanged)" — broadening
  the version-segment equality check to v1 loads adds a refusal to the v1 load
  path that shipped code did not have (a document declaring a version other
  than its directory's segment), so "exact shipped behavior" is now false in
  the paragraph that introduces the change, and it strains the compatibility
  matrix's "v1 refusals stay byte-exact" promise and Rollout 6's "none". The
  universal form is the *simpler* rule (no branch on directory version), so the
  claim, not the check, is the problem. Fix: keep the universal check and
  narrow the promise to what is actually pinned — "no v1 document in the corpus
  changes outcome (every v1 document declares `v1`, so v1-on-v1 passes
  trivially); the existing `reach_tables.rs` suite stays green unchanged" —
  and let Test strategy #3's existing sentence ("v1 directories included")
  stand as the only case that changes.

## Value check (checklist 7)

Proportionate. Every r1 addition traces to a stated Goal and reuses existing
machinery: `ReachClass` for `InputGrade.class` reuses the shipped type instead
of a `String` (ADR-012 Context names it in `reach.rs`); the geometry `name`
grammar check reuses the entry-set's own `[a-z][a-z0-9-]*` rule and mirrors the
shipped `hand_matches_file_name` pattern; the `0..=999` coordinate bound plus
signed deltas is one comparison and a type choice, and it retires the r1 base's
looser "non-negative integer" wording. New dependencies: none (std-only,
`Cargo.lock` byte-unchanged). No reusable in-repo alternative exists for the
geometry table or the predicate; the smaller design the checklist asks about —
Rust constants for coordinates — is rejected on ADR-012's own Consequences
("new table files, not new logic"), and deferring the predicate would leave ADR
Decision Four unimplemented while 015 preconditions on it. The two new modules
mirror `action.rs`'s existing loader shape rather than adding a layer.

## FYI (no severity)

- Worked examples / Test strategy #5: the pinned case "WASD→OKL ... uniform
  deltas `(7,0)`" names three targets for four moves; the fourth
  (`keyboard:;` at `9 2`, which is what makes `(7,0)` hold) is unstated, so the
  pinned delta is not checkable as written. Naming the four `(source, target)`
  pairs would close it. (Test-strategy seat.)
- Compatibility matrix, non-Rust row: `yes (data only)` is a support promise
  whose enforcement half ("the runner contract stays Deferred") is not in this
  document, so the row and the Deferred entry should be read together by the
  compatibility seat.
- Test strategy #3 asserts the version/directory-segment refusal for "v1
  directories included", but the delta inventory adds no v1 fixture pinning it;
  the conformance corpus already pins every other rule in this design.
  (Test-strategy seat.)
