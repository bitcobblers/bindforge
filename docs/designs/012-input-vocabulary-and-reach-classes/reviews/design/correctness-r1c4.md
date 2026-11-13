# correctness review r1 (ratification pass c4)

verdict: approve

Scope: `git diff b08752d2 92beeaa7` on `02-proposal.md` (three hunks: the v1-path
sentence in *The model*; predicate rule 4 in *Placement legality*; the last two
Revision log entries), then the full current document and
`docs/adrs/012-input-vocabulary-and-reach-classes.md`.

## Findings

None. Checks performed on the changed text:

- **The model** — "Hand and hand_matches_file_name are untouched; every v1
  document declares `v1`, so the check adds no corpus refusal;
  `reach_tables.rs` stays green." Consistent with the version-segment equality
  rule stated immediately above, with Test strategy #3 ("a document whose
  declared version differs from its loading directory's segment is refused, v1
  directories included"), and with Test strategy #7 ("`reach_tables.rs`'s v1
  expectations pass byte-unchanged"). The replacement is strictly more precise
  than the sentence it replaces: the equality check does now run on v1 tables,
  and the new wording states exactly why v1 corpus outcomes are unchanged.
  Nothing previously approved becomes false.
- **Predicate rule 4** — "the per-move coordinate deltas — one common
  `(dx, dy)` when the moves agree — and each move's degradation". Agrees with
  rule 2's shared-`(dx, dy)` shape clause, with the `Result<PlacementCheck,
  CoreError>` interface (definedness is the structured refusal; shape/bound
  failures are verdicts the deltas explain), and with Test strategy #5
  ("legal with uniform deltas `(6,0)`, `(7,0)`, `(1,0)`"; "the check reports
  per-move deltas" for the swap/rotation case). Re-verified the delta and
  degradation arithmetic for WASD→IJKL, WASD→OKL, WASD→ESDF against the shipped
  tables and `classes = [home, thumb, stretch, far, off-reach]`: deltas (6,0),
  (7,0), (1,0) and all degradations 0, so "all three are legal at bound 0"
  remains true. No same-thing-two-shapes violation (checklist item 5): the
  common delta is explicitly the degenerate case of the per-move deltas.
- **Revision log** — both r2 entries record decisions actually present in the
  current text (grammar-checked `name`, bounded signed deltas, `PlacementCheck`
  content, version-segment equality, typed `InputGrade.class`); no closure
  contradicts the body or the ADR.

## Prior finding disposition

- My r1 minor finding (worked examples + Test strategy #5 — the
  "`keyboard:;`, `off-reach` for the source hand" example is hand-pair
  dependent while rule 3 grades the target by the to hand) sits in text this
  diff does not change; the diff context lines confirm the sentences are
  byte-identical to the state approved. The revisions neither break it further
  nor make a cited section newly false, so per this pass's rules it is not
  re-reported.

## FYI (no severity)

- The revision log relabels two entries from r1 to r2 and merges the earlier
  r1 wording into them; round-attribution bookkeeping is editorial, not
  correctness.
