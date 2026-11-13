# Code review — testing seat, round 1 pass c0

Range: `git diff 1af12026893971772c5ac8384ceae348d63cd064...HEAD -- ':(exclude)docs/designs'`
Judged against: `02-proposal.md` + ADR-015. First pass of this seat — no prior findings to re-check.
Note on method: the review environment denied all shell execution, so the diff was reviewed
from the post-change working tree of exactly the paths the range covers, and gate commands
could not be run from this seat (checklist item 6 — recorded as a finding, not a pass).

## Checklist, item by item

**1. Every numbered Test strategy item has a citing test — PASS (with the gaps below).**
Citations found:
- #1 `crates/core/tests/policy.rs` (file header + per-case `proves:` on every grammar case:
  unknown kind, empty row, few fields, past `MAX_RELOCATE_TARGETS`, bad hand, bound letter/
  past-scale, unknown input, extra polarity field, mouse-row arguments, four tag shapes,
  missing/empty/past-cap `transforms`, uppercase/64-octet id) and
  `crates/core/tests/grammar_closure.rs:478,494` (entry set + `transforms_` alias).
- #2 `policy.rs:670` round-trip of every valid stack + `conformance.rs` walk over the real
  fixtures (order preserved, canonical re-spelling).
- #3 `policy.rs:786,866,896,931,959` — grammar-before-filesystem (incl. the absolute-path
  `Path::join` hazard), stem↔id, directory-major, format arm, happy-path load.
- #4–#8 `crates/transforms/tests/apply.rs` — every named relocate outcome, polarity, mouse,
  sequential-stack both directions, all-inapplicable, `report.index`, chain-citizen shape,
  pin/id/patch entry refusals, determinism (`serialize`-identical bytes), atomicity pinned on
  every failure shape.
- #9 `conformance.rs:395` + `policy.rs:992` (corpus ships the 6+30 named fixtures; the walk
  itself is untouched — only the expected-directory list gained `policy/v1`).
- #10 `version_gate.rs:329,359` (v2-declaring policy refuses naming policy's majors; the
  majors-equality invariant unchanged) + both `zz_api_probe.rs` (core names/constants,
  transforms names + the ten reason spellings).
- #11 is platform evidence (CI `rust` job, one Windows run) — no citable test by nature; it
  belongs to the merge PR, not this diff.

**2. Unit and integration layers — PASS.** Unit: `core/tests/policy.rs`,
`transforms/tests/apply.rs`. Integration: the conformance walk, `version_gate.rs`, and both
api probes, as the proposal promises.

**3. Revision rounds — N/A.** Round 1, first pass of this seat; no previously rejected
blocker/major to pin.

**4. Determinism — PASS.** No wall-clock or random value reaches an assertion; the shipped
contract tree is read through `CARGO_MANIFEST_DIR`; scratch dirs follow the pre-existing
`crates/core/tests/common` convention (pid + counter + nanosecond nonce only in the directory
*name*). No network, no live server, no ports anywhere in the change.

**5. Test names describe behavior — PASS.** E.g.
`refuses_a_transform_row_naming_an_unknown_kind_naming_the_row`,
`a_second_row_sees_the_layout_the_row_above_it_produced`,
`swap_polarity_applied_twice_is_the_identity`.

**6. Gate commands — COULD NOT EVALUATE (finding 5).** `npm run check` does not exist in this
repo; the equivalent is `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D
warnings`, `cargo test --workspace --locked`. Every shell call from this review environment was
rejected (`Permission denied: shell`), so no `test result:` tail can be pasted here. API
surface was spot-checked by hand instead (test imports vs `lib.rs` re-exports, `GradeBound.0`,
`PlacementCheck.degradations/worst_degradation`, `TagPattern::spelling`, `Transform::kind`):
consistent. The runner verifies this independently per the skill's own note.

**7. Untestable additions — findings 1–4 below** (one untested grammar refusal, one dead
message arm, two reachable-but-unpinned guard/verdict arms).

## Findings

1. **minor — `crates/core/src/document.rs:1430-1433` (with `schemas/policy/v1/spec.md:89-90`).**
   The parse-level "no repeats" rule for `relocate-cluster` targets — stated in the proposal's
   contract table and in the shipped `spec.md`, enforced in code ("`{field}` is named twice by
   the target list") — has no test anywhere and no invalid fixture, breaking the corpus
   invariant the proposal states for this format ("every grammar refusal has its fixture",
   A22) that every other refusal here honors.
   Fix: add `fixtures/invalid/relocate-target-repeats.toml` (a 16-inside-cap row with one
   repeated input), cite it in `spec.md`'s fixture index, and add it to
   `INVALID_FIXTURE_NAMES` in `crates/core/tests/policy.rs` (or at minimum an inline
   `value_refusal` case there).

2. **minor — `crates/transforms/src/apply.rs:495-496`.** Dead branch: the `None` arm of the
   `axis_kind` match ("`{name}` is not an id the loaded registry lists") is unreachable — the
   match set is built from `registry.ids()`, and `registry_from_entries`
   (`action.rs:179-210`) gives every listed id one of the three kinds, so `axis_kind` answers
   `Some` for every matched id. An untestable branch is a finding per checklist item 7.
   Fix: collapse the `match kind` to the `Some` spelling (the arm's text belongs to no rule)
   or replace it with a `debug_assert!`, keeping one message path.

3. **minor — `crates/transforms/src/apply.rs:508-510`.** The untouchable guard for
   `swap-polarity` is reachable (the guard is "unconditional, not author-declared" over every
   row, spec.md:176-179) but has no test: guard cases exist for relocate (four shapes) and
   mouse (docked), none for polarity. An author docking `move.steer-left` gets a code path no
   suite pins.
   Fix: add one `apply.rs` case — e.g. `docked-actions = ["move.steer-left keyboard:a"]` with
   the standard `swap-polarity move.steer-*` row → `reserved-input` naming the offender,
   layout byte-equal.

4. **minor — `crates/transforms/src/apply.rs:724-727`.** The `not-keyboard` rule is pinned only
   on the source arm (the gamepad-bound cluster); the target arm of
   `first_not_keyboard` — promised by Chosen design → The stack ("any source **or target**
   input not `keyboard:*`") — is untested, even though a `mouse:`/`gamepad:` target parses
   cleanly (it is closed-vocabulary grammar-legal).
   Fix: add a relocate row with `mouse:left` as a target → `not-keyboard` naming `mouse:left`.

5. **minor — checklist item 6 (gate commands), whole change.** Could not run
   `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, or
   `cargo test --workspace --locked` from this seat: the review environment rejects every
   shell invocation. No output tail to paste; the item counts as a finding per the review rule,
   and the runner's independent `rust` job is the evidence. Nothing observed in the reviewed
   files suggests a red gate (imports, types, and public names line up with the shipped and
   new API surfaces), so this is recorded as an unverifiable-from-seat item, not a failure.

6. **nit — `crates/transforms/tests/apply.rs:499-523`.** Test strategy #5 promised "a match
   that includes `aim.camera` → `axis-kind` naming the id"; the case instead places
   `move.steer-right` in a scratch registry's `continuous-lateral` array. The deviation is
   substantively forced (the prefix tag form requires a literal `-` in the second segment, and
   the exact form matches one id → `match-count` fires first, so a two-id match containing
   `aim.camera` is not expressible with the shipped grammar), but no test ever touches the
   shipped `continuous-lateral` id, and the `` `none` ``-kind spelling of the same message
   (`apply.rs:491-494`) is likewise unexercised.
   Fix: one sentence in the test's docstring recording why `aim.camera` itself cannot appear,
   and (optionally) a scratch registry with two `none`-array ids under one prefix tag to pin
   the `none` spelling.

## FYI (out of specialty)

`crates/transforms/tests/common/mod.rs` is a new shared test helper not named in the proposal's
Delta inventory (which lists only `apply.rs` and `zz_api_probe.rs` under that `tests/`); it
mirrors the shipped `crates/core/tests/common` pattern and reads clean — completeness of the
delta inventory is the correctness seat's, noted here only for the record.

## Verdict

**approve** — zero blocker and zero major findings; the suite covers Test strategy #1–#10 with
behavior-named, deterministic, channel-faithful tests, and the six findings above are gaps and
one dead branch worth a follow-up pass, not design violations.
