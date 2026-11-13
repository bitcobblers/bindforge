# correctness review r1

verdict: request-changes

Inputs: `02-proposal.md` (whole document), `docs/adrs/012-input-vocabulary-and-reach-classes.md`,
the (empty) Revision log. Assumption rows spot-checked against their cited bases
(`schemas/reach-classes/v1/spec.md`, `left.toml`, `document.rs:35,45,61-63,355-377`,
`schemas/README.md:16-26,68-74`, `version_gate.rs:92-105`, `conformance.rs:101-104`,
`error.rs`, `reach_tables.rs:176-195`, `common/mod.rs:50-62`, `check.yml:29-33`,
ADR-015, 011 proposal:475-478) — all support their rows. Four research URLs webfetched:
W3C `uievents-code` (US-layout naming confirmed), W3C `gamepad`, W3C Pointer Events 4
(button 1 = middle/wheel press confirmed), Wikipedia Touch typing (home row ASDF / JKL;
confirmed). The three shipped data listings were counted by hand: 48 entries each, each id
once per hand, the two hands' id sets equal, `ansi-us.toml` covers exactly the graded set,
ANSI row/column layout correct, and deltas `(6,0)`, `(7,0)`, `(1,0)` for WASD→IJKL /
WASD→OKL / WASD→ESDF check out. A8/A9/A10 are honestly labelled unverified and the body
hedges them; no unverified assumption is stated as fact.

## Findings

- [major][new] Test strategy #5 — "a one-step degradation (`home`→`stretch`) is legal at `(1)` and illegal at `(0)`" — the degradation is two rank steps, not one: rank is "the class's index in the shared `classes` order" (predicate rule 3) and the shipped order is `["home", "thumb", "stretch", "far", "off-reach"]`, so `home`→`stretch` = 2−0 = 2, which is illegal at `GradeBound(1)`, not legal. The pair of claims pins a suite case that fails as written (and matches the ADR's set-listing order, not this design's pinned severity order — exactly the confusion A9 warns about). Fix: make the example a real one-step move — `home`→`thumb` (e.g. source home key → `keyboard:space`) legal at `(1)`, illegal at `(0)` — or relabel and move the bounds: `home`→`stretch` legal at `(2)`, illegal at `(1)`.
- [major][new] Chosen design, Worked examples — "WASD→ESDF are legal at bound 0, which is the left-hander's WASD-to-the-right-hand move (the research's folklore rationale for the IJKL cluster...)" — the hand attribution contradicts the shipped tables: with to-hand `right`, `A`(left `home`, rank 0)→`S`(right `off-reach`, rank 4) degrades 4, so WASD→ESDF is *illegal* at bound 0 left→right; it is legal at bound 0 only as a same-left-hand one-column shift (E/S/D/F are left-hand keys; the folklore right-hand move is WASD→IJKL, which *is* legal at bound 0 to `right`). No single (from, to) pair makes all three pinned examples legal, and Test strategy #5 never states each example's hands — implemented from the prose it pins a failing case. Fix: attach the folklore clause to WASD→IJKL (from `left`, to `right`), describe WASD→ESDF as a same-hand (left→left) cluster shift one column right, and state each example's `(from, to)` hand pair in Test strategy #5.
- [minor][new] The model / reach.rs — "`load_reach_tables` gains two checks for a `v2` directory: the declared `version` must equal the loading directory's version segment" — scoping the check to "a `v2` directory" misses the load-bearing direction: a *v2* document sitting in a *v1* directory passes the version gate (v2 is now supported) and would load unchecked, while Test strategy #3 pins the general rule ("a document whose declared version differs from its loading directory's segment is refused"). Fix: state the version-segment equality check as running for every reach-classes table at load, v1 directories included (the v1-on-v1 case passes trivially, keeping the `reach_tables.rs` suite green).
- [nit][new] The model — "`InputGrade` is the parsed pair: `input: InputId`, `class: String`" beside "`grade(&InputId) -> Option<&ReachClass>`" — the class id appears under two shapes; the shipped code already carries `ReachClass` (and the conformance note says nothing makes an id-shaped string part of `ReachClass`'s surface). Fix: make `InputGrade.class` a `ReachClass` (or say in one clause that the loader validates the pair's class id against the canonical taxonomy and converts), so one concept has one type.
- [nit][new] Placement legality, rule 4 — "The `PlacementCheck` result carries the shape delta and the per-action degradation" (singular) while Test strategy #5 pins "a non-translation move set ... the check reports both deltas". Fix: say rule 4's shape delta is reported when the moves agree, and the per-move deltas when they do not — the shape #5 already pins.

## FYI (no severity)

- `gamepad:left-stick` / `gamepad:right-stick` are in the v1 closed member set while per-stick-direction naming is deferred: what a directionless stick id binds is under-specified — wording/spec seat.
- The 48 authored grades themselves (e.g. right `]` far vs left `]` off-reach) have no external benchmark (A8); plausibility auditing of individual grades is a data-review concern, not this seat's; the proposal flags it honestly.
- The compatibility matrix rows inherited from design 008 (macOS/SteamOS "no (v1)") are the compatibility seat's territory.
