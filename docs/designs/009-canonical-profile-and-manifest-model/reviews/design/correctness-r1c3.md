# correctness review r1 (dissent pass c3)

verdict: approve

Scope: `git diff 5bbcc09d f7521536` of `02-proposal.md`, then the full current
document, `docs/adrs/009-canonical-profile-and-manifest-model.md`, the Revision
log, and my own r1 findings. Checklist: assumption register, source spot-check,
internal contradiction, edge cases, interface identity, ADR coverage, closure
quality.

## Closure check (my r1 findings)

- r1 #1 (major) — **closed for real.** §4 now declares
  `apply_policy(...) -> Result<PolicyOutcome, CoreError>` with `skipped`
  reserved for the inapplicable-transform report, the whole-`stack` check
  before the first invocation, the rejection as `InvalidValue` in the `Err`
  channel, Adversarial posture says "the invocation table's arity and
  grammars as `apply_policy`'s `Err` (§4)", and Test #10 reads "is an
  `Err(InvalidValue)` from `apply_policy`, before the first invocation runs".
  `bindforge-transforms → bindforge-core` puts `CoreError` in scope, so no new
  type is invented. The Revision log records the r3 reason accurately.
- r1 #2 (major) — **closed for real.** §3 states `Resolved.manifest` is `Some`
  whenever the profile declares `for`, and the Id → file paragraph makes
  `UnresolvedAncestor` one rule over "any document reference with no document
  — a `derives` entry, or a `for` pin whose `game` + `revision` names no
  manifest document in the index"; Test #9 asserts the pin case, Adversarial
  posture maps `UnresolvedAncestor` to #9, and the drift gate runs through
  `resolve`, so a dangling pin fails CI instead of answering `None`. One
  residual is left by the new wording (finding C below), not the r1 hole.
- r1 #3 (minor) — **closed for real**, with two loose ends (findings A, B).
  Adversarial posture now pins "only where `parse` can see the condition" and
  maps every listed variant to a named suite; all 13 variants are accounted
  for (index limits → #1, `GeometryConflict` → #3,
  `PlacementNotIsometric`/`PlacementDegraded` → #4, `DuplicateDelta` → #6,
  `AxisKindConflict`/`UnknownAction`/`UnknownInput` → #8,
  `DuplicateId`/`UnresolvedAncestor` → #9, `BadPin` → fixture,
  `MergeConflict` → ADR-012); the authoring bar's "one invalid fixture per
  rejected condition" and Test #10's fixture parenthetical are gone, and the
  Risks row's "per-condition invalid fixtures" mitigation was deleted with
  them. The r3 reason in the log matches what the text now says.
- Spot-check: `https://toml.io/en/v1.1.0` fetched — bare-key charset
  `A-Za-z0-9_-`, quoted keys, dotted keys as nested-table syntax, and nested
  arrays (`nested_arrays_of_ints = [ [ 1, 2 ], [3, 4, 5] ]`) all as A1/A12/A13
  claim. The changed text's load-bearing citations (A13 nested-array rows,
  A15 authored grading, A18 id-index rule, A25 seed-scale headroom) are
  consistent with their basis. ADR coverage: all five contracts decided; the
  `docked` honoring, `fork`/`merge` implementation and CLI deferrals match the
  ADR's own "apps/cli lands in ADR-012, not here" and "Not decided here", so
  no Deferred item contradicts the source document.

## Findings

- [minor][new] Chosen design → Reach classes and placements → Authoring bar (§2)
  — "CI enforces what it can: the graded set … is graded in both hands,
  coordinates agree across hands, **grades are non-decreasing in grid distance
  from that hand's home-graded keys**, every rejected condition pinned by Test
  strategy #3's model tests" — the r3 closure makes Test #3 the sole pin for
  "every rejected condition", and two items in that same CI-enforced list break
  it. (a) The monotonicity condition appears in no test anywhere: Test #3's
  clauses are per-hand distinctness, row grammar + class vocabulary,
  cross-hand coordinates, mirrored asymmetry, graded-set coverage, `off-reach`
  default, admitted-list `InvalidValue` — grid-distance monotonicity is not
  among them, and no other Test strategy item names it, so the condition the
  authoring bar says CI enforces is exactly the un-pinned one the r1 finding
  was about. (b) The bar governs both `placement/v1/spec.md` and
  `reach-classes/v1/spec.md`, and Adversarial posture maps
  `PlacementNotIsometric`/`PlacementDegraded` to **#4**, so naming only #3
  contradicts the posture's own mapping. Fix: write "every rejected condition
  pinned by Test strategy #3 and #4's model tests", and add the monotonicity
  clause to Test #3 ("grades are non-decreasing in grid distance from that
  hand's home-graded keys") — or drop it from the CI-enforced list to
  review-only, which is what "extended only by review" already implies for the
  graded set.
- [minor][new] Test strategy #4 and #8 — "a non-isometric **fixture** is
  rejected `PlacementNotIsometric`; a fixture exceeding `max-degrade` is
  rejected `PlacementDegraded`" / "… is rejected, **each with its own
  fixture**" — the r3 closure fixed this wording for reach-classes only
  (#3's new "in the loader, not in a fixture document"); #4's and #8's
  conditions are produced by `validate`, which `parse` cannot see, so the bare
  word "fixture" points an implementer at `schemas/{placement,manifest}/v1/
  fixtures/invalid/`. A document there is asserted by the shipped
  fixture-walking harness (Test #1: "each `fixtures/invalid/` case is rejected
  with the pinned `CoreError` variant"; Test #13: "requires no change to
  `crates/core/tests/conformance.rs`") — it parses clean, so the harness
  assertion fails and the "no harness change" claim breaks. Fix: carry #3's
  clause across, once per site: in #4 "a non-isometric placement pair is
  rejected `PlacementNotIsometric` by `validate`, not in a fixture document"
  (same for the degraded and ungraded-key cases), and in #8 "each over its own
  manifest data, not in `fixtures/invalid/`". Adversarial posture's general
  rule already says this; the two test-strategy lines are all that is loose.
- [minor][new] Chosen design → Profiles are chains (`resolve`) — "`Resolved {
  …, manifest: Option<ManifestPin> }`, whose `manifest` is `Some` whenever the
  profile declares `for`" and "`for` is required at parse and a `for` naming no
  manifest document in the index is `UnresolvedAncestor` …, so the check never
  runs against nothing" — the new `Some`-whenever claim is stated of one
  document, while a chain carries the entry on every rung: §3 gives `patch/v1`
  "the same optional chain entries", so a base can pin `remnant2@0.1`
  (`seeded-from` beside it) under a leaf patch that pins nothing, or pins
  `0.2`. `Resolved.manifest` is one slot for N pins, and the sentence's
  guarantee ("never runs against nothing") follows only if the `seeded-from`
  check is read per-document, against that document's own `for`; read against
  the rooted document's pin — the natural reading of "`the profile`" in
  `resolve(index, root, …)`, and the value the drift gate's "resolves every
  profile … against the manifest revision **it** pins" hands to `validate` — a
  chain whose base carries `seeded-from` and whose leaf carries no `for`
  answers `manifest: None` and the dangling-preset case my r1 #2 raised is
  vacuously green again, for chains. Fix: one clause — state that
  `Resolved.manifest` is the rooted document's pin, that the `seeded-from`
  check runs per-document against that document's own `for` (parse already
  pairs them), and that a chain document whose `for` is present and differs
  from the rooted document's is a rejection naming both document ids. No new
  mechanism; it picks, in one sentence, the reading the two sections already
  imply.

## FYI (no severity)

- Goals promise "Concrete Rust: modules, types, function signatures", and the
  r3 format-admission sentence enumerates only the two indexed trees
  (`data/profiles/`, `data/manifests/`). The walks that fill
  `Ctx.placements` and read `data/policies/` — the loader behind Test #4's
  "every placement in `data/placements/`" and Test #11's
  `apply_policy(game-default, …)` — have no named signature and no stated
  format-admission rule, so a `manifest` document under `data/placements/` is
  unruled. Completeness seat's line, not a correctness contradiction.
- `UnknownAction`/`UnknownInput` are mapped "by #8" in Adversarial posture,
  while #4 and #6 also exercise them over their own fixture data; the mapping
  is a producer attribution, so naming the extra suites is wording, not a gap.
