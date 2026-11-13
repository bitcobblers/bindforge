# correctness review r1 (pass c0, scope: whole document)

verdict: approve

Inputs: `02-proposal.md`, `docs/adrs/011-action-taxonomy-and-core-skeleton.md`, the
Revision log (r0: no reviews yet), plus webfetch URL spot-checks required by the
correctness checklist. No earlier findings exist to carry or re-check.

## Checklist evidence

- **Assumption register** — `verified` rows checked against their cited basis:
  A1 (`Cargo.toml:12` `members = ["crates/*"]`; `crates/core/Cargo.toml:2`
  `name = "bindforge-core"`; `schemas/` tree and `conformance.rs` walk all
  present); A2 (`008/03-build-notes.md:533` "five seats, five approves, zero
  blockers, zero majors"); A4 (`012:6` `depends-on: [011]`; `013:36-37` action
  table "game-native action name ↔ taxonomy id (011) + axis kind"; ADR-011:52-54
  polarity clause); A8/A9 (`check.yml:33` `cargo test --workspace --locked`);
  A10 (`toml.rs:169-189` bare keys admit `-`; `basic-profile.toml:5` empty
  one-line array; `chain-past-max-depth.toml:8` 33-link one-line array — all
  confirmed); A11 (`SKILL.md:52-55`); A16 (`008/03-build-notes.md:48-63`
  toolchain-unavailable blocker). The three `unverified` rows (A3, A15, A17)
  are honestly marked and the body defers to them rather than asserting them.
- **URL spot-checks (4)** — RFC 1035 §2.3.1/§2.3.4 (letters/digits/hyphen,
  letter-first, letter-or-digit-last, ≤63 octets, case-insensitive) supports
  A5; toml-test README ("test one thing and one thing only … named after the
  fault") supports A6; semver.org FAQ supports A12's announce-in-minor /
  remove-in-major vocabulary; the evdev header's "a new KEY_FOO define MUST be
  added" and the frozen-LED comment support A14. None dead or contradicting.
- **Arithmetic and grammar self-consistency** — the registry holds exactly 40
  ids (4 + 1 + 35); namespace counts (move 8, aim 3, combat 8, interact 3,
  item 8, ui 6, system 4) sum to 40 and match the enumeration; every shipped id
  satisfies the stated segment grammar; `discrete-lateral` is exactly the four
  ids Test strategy #2 pins. Options, spec, model interfaces
  (`axis_kind → Option<AxisKind>`), error variants (`InvalidValue`,
  `MissingEntry` both exist in `error.rs`), and the four re-exported names are
  consistent across sections.

## Findings

- [minor][new] Test strategy #3 — "a document with exactly the four defined
  entries parses" — the format defines five entries: the spec entry table lists
  `format`, `version`, and the three axis arrays (all required), and the
  `registry.toml` shown in Chosen design carries exactly those five keys; the
  same test even says a "fourth axis array" is an unknown entry, implying three
  defined arrays plus `format` and `version`. As written, a document with only
  four of the five required entries would have to parse, which contradicts the
  entry table and the `missing-axis-arrays` fixture. Fix: change "four" to
  "five defined entries".
- [minor][new] Delta inventory — "Unchanged: `conformance.rs`, `common/mod.rs`,
  `chain.rs`, `reach.rs`, …" — contradicts Placeholders and lineage and Rollout
  5, which edit "the `ADR-009` references in the module docs of
  `crates/core/src/lib.rs`, `document.rs`, and `chain.rs`" (`chain.rs:3` and
  `:16` do hold such references). `chain.rs` is an edited file listed as
  unchanged. Fix: move `chain.rs` into the Edited list annotated "(module docs
  only)", or split the Unchanged list into "unchanged" and "docs-only edits".
- [minor][new] Goals / Non-goals — "the stale ADR-009 lineage in the contract
  docs retargeted to the 011-015 series" — the delta inventory and Rollout 5
  retarget only `schemas/README.md` plus the `lib.rs`/`document.rs`/`chain.rs`
  module docs, but stale ADR-009 ownership lineage also sits in
  `schemas/reach-classes/v1/spec.md` ("class *content* … owned by ADR-009 …
  placeholders pending ADR-009"), `schemas/profile/v1/spec.md` ("owned by
  ADR-009"), and `crates/core/src/reach.rs` ("ADR-009 owns the taxonomy the ids
  name"). The Risks row "Stale ADR-009 lineage misleads the next contributor"
  therefore under-covers its own mitigation. Fix: add those three files'
  lineage sentences to the Rollout 5 retarget list (doc-only, no deferred
  substance touched), or scope the Goal to the files actually edited.
- [nit][new] Chosen design — "fixtures/invalid/ … missing-axis-arrays.toml" —
  the proposal commits to toml-test's rule ("one fault each, file named after
  the fault (A6)"), but the plural name implies one document missing several
  required axis arrays. Fix: make the fixture omit exactly one required axis
  array and name it `missing-axis-array.toml`; the all-arrays-absent case is
  already covered by unit Test strategy #3.

## FYI (no severity)

- Test strategy #2 pins `discrete-lateral` to exactly four ids, while Stability
  allows id additions within a major by PR: a legitimate future
  `discrete-lateral` addition will fail this test and must update it too. A
  wording nuance for the testing seat.
- Deferred names "ADR-016 (not yet written)" — no such ADR file exists under
  `docs/adrs/`; that is consistent with the proposal's own "not yet written"
  note and with ADR-011 (which names 017 but never assigns seed sets to 016),
  so it is not a contradiction finding, just a reference no named input
  corroborates.
