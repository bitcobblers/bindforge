# complexity review r1 (dissent pass c3)

verdict: approve

Scope: the blob diff b08752d..92beeaa of `02-proposal.md` (The model sentence,
predicate clause 4, Revision log r2 entries), read against the full current
document and `docs/adrs/012-input-vocabulary-and-reach-classes.md`.

## Closures verified (my r1 findings)

- [closed] **Placement legality** — clause 4 now reads: "The `PlacementCheck`
  result carries the per-move coordinate deltas — one common `(dx, dy)` when
  the moves agree — and each move's degradation, so a caller (ADR-015's
  precondition, 015's policy layer) can explain a refusal instead of seeing a
  bool." (r1: major) The degradation is restored in the same sentence while
  the single delta shape is kept — this is my r1 fix verbatim in substance.
  Coherent with clause 2 ("delta" = coordinate difference), clause 3
  (degradation defined there), Test strategy #5 ("the check reports per-move
  deltas"), and the r2 revision-log entry "PlacementCheck carries coordinate
  deltas and each move's degradation". A clause-(b) bound refusal stays
  explainable (which action, by how many ranks). No mechanism added beyond
  what r1 approved.
- [closed] **The model** — "Hand and hand_matches_file_name are untouched;
  every v1 document declares `v1`, so the check adds no corpus refusal;
  `reach_tables.rs` stays green." (r1: minor) The self-contradictory promise
  ("the v1 path keep their exact shipped behavior") is gone, replaced by the
  narrowed, pinned claim. The universal version-segment check is kept (the
  branch-free rule I endorsed), v1-on-v1 passes trivially, and Test strategy
  #3's "v1 directories included" refusal stands as the only changed case —
  it tests a synthetic document, not corpus data, so it does not conflict
  with "no corpus refusal".

## Findings

None open. Checklist over the changed text: no new package/module/interface
(the closures only restate existing ones); every changed part traces to a
stated Goal (`PlacementCheck` payload → 015's precondition rationale;
version-segment equality → A2's contract versioning rule); no new
dependency; no configuration knob or one-implementation abstraction; the
changed sentences are net-shorter than what they replaced, so the design did
not grow. Value: proportionate.

## FYI (no severity)

- Compatibility seat: the matrix row "v1 refusals stay byte-exact" now
  coexists with a check that *could* refuse a malformed v1 directory
  placement; The model qualifies it ("every v1 document declares `v1`, so the
  check adds no corpus refusal"), which reads consistent to me, but the
  byte-exactness promise is their call, not mine.
