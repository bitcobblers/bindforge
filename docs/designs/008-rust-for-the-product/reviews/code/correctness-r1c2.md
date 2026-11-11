# Code review — correctness, round 1 dissent pass c2

Scope: `git diff a8b3aeda622ff4da60863c46e704091e6a74707b..HEAD -- ':(exclude)docs/designs'`,
read against `02-proposal.md`, `docs/adrs/008-rust-for-the-product.md`, and my own
round-1 findings. Shell/git were unavailable in this environment, so the current
tree was reviewed file-by-file and each prior finding re-verified at HEAD by
hand, line by line (no `cargo` run is possible from this seat either — the
dated gate evidence stays an outstanding checklist item, honestly disclosed in
the build notes).

## Round-1 findings — re-check

1. **Blocker — `toml.rs` missing `use std::collections::BTreeMap;`.** **Fixed.**
   `crates/core/src/toml.rs:28` imports `std::collections::BTreeMap` in
   rustfmt's sorted std position, matching `document.rs:23`; the two use sites
   (`read_document`'s return type at toml.rs:98 and `BTreeMap::new()` at :105)
   now have a binding. The lib target's E0412 is gone.
2. **Blocker — `conformance.rs:119` `class.to_string()` on `ReachClass`.**
   **Fixed.** The corpus load assertion now maps to ids —
   `table.classes().iter().map(|class| class.id.clone()).collect()`
   (conformance.rs:122), compared against the textually-read `classes` list
   (`common::string_array`), with a comment stating why id-spelling is not
   public surface — exactly the fix my round-1 report prescribed. `ReachClass.id`
   is `pub` (reach.rs:62-64), so the `conformance` target compiles.
3. **Blocker — `reach_tables.rs:111-125` `contains(&"near")` on `&[ReachClass]`.**
   **Fixed, and strengthened.** The content assertion now collects
   `class.id.as_str()` (reach_tables.rs:112-113) and asserts *exact* expected
   lists: left `["near", "mid", "extended"]`, right `["near", "mid", "cross"]`
   (:114-123) plus `assert_ne!` (:124-128) and the `hand()` checks. I verified
   those literals against the shipped `schemas/reach-classes/v1/left.toml` /
   `right.toml` — they match. The Test #4 distinctness assertion now compiles
   and pins per-hand content.
4. **Nit — shipped contract text understating the harness.** **Half fixed, now
   stale the other way — see finding 1.** `schemas/reach-classes/v1/spec.md:55-58`
   now states that the walk also loads each valid corpus table and files its
   classes under the declared hand in table order; the harness matches
   (conformance.rs:88-127, exercised by `basic-table.toml`, which declares
   `hand = "right"`).

## Correctness checklist pass over the changed code

1. **Components/interfaces named in the proposal:** root `Cargo.toml`
   (resolver 3, `[workspace.package]` 2024/1.85/Apache-2.0,
   `[workspace.dependencies]`), `rust-toolchain.toml` 1.85.0, committed
   `Cargo.lock` (v3, single path member), `.gitignore` `/target`,
   `crates/core` as `bindforge-core` (depends on nothing),
   `crates/core/tests/conformance.rs` walking `schemas/*/v*/fixtures/{valid,invalid}`
   via `env!("CARGO_MANIFEST_DIR")`, `resolve_chain` + `MAX_CHAIN_DEPTH = 32` +
   `Cycle`/`ChainTooDeep`, the 1 MiB/16-level budget constants re-exported,
   both workflows pinned by SHA with job-level `permissions: contents: read`,
   root `deny.toml`, README and `repo-practices` updates: all present and wired.
2. **Assumption register:** honored; the members-glob narrowing and the
   in-tree-parser/grammar-subset are the two conscious gaps, both disclosed.
3. **Deviations:** the shipped deviations (members glob, minimal reader +
   documented TOML subset, error-API spellings, reach-table file names,
   `deny.toml` `ignore`, scanner's own 1.88.0 toolchain, `--format` before
   `check`) are all recorded in `03-build-notes.md`. No undeclared deviation
   found in the changed paths.
4. **Scope creep:** none in this pass's diff — the new/changed test bodies
   implement exactly Test #1–#4 and the Gates pins; `hand_matches_file_name`'s
   truth table and the reach-loader budget tests pin loader rules `spec.md`
   states.
5. **Build-notes claims vs code:** verified except the now-stale test status —
   finding 1. The cargo-deny fix is real and machine-pinned: the workflow runs
   `cargo deny --format json check` (advisories.yml:65) and
   `gate_contract.rs:298-305` asserts that exact form plus the no-flag-after-
   `check` and no-section-narrowing rules — build-notes item 3 landed. Item 4
   also landed: the stream half of the input budget is in
   `parse_budget.rs:126-157`; the hand truth table landed in
   `reach_tables.rs:138-155` rather than `parse_budget.rs` — content complete,
   file placement differs from the note's wording.
6. **Error paths fail closed:** re-traced the whole invalid corpus (13 fixtures)
   against `toml::read_document`/`parse`/`resolve_chain`: each produces the
   typed rejection the harness and `chain_guard`/`version_gate`/
   `grammar_closure`/`parse_budget` expect (33-link chain → `ChainTooDeep`
   before the cycle scan; 17-level nesting → `NestingTooDeep`; hex →
   `unexpected 'x' after the value`; forged newline → truncated one-line
   `UnknownFormat`). Valid corpus round-trips through `serialize`/`quoted`
   including the `\b`/`\f` escape fixture; the corpus load half replays
   `basic-table.toml` into a stand-in directory and compares ids in order.
7. **Round-1 blocker/major findings addressed:** yes — all three compile
   errors are gone, each fixed at exactly the site my round-1 report named.

I also re-checked the changed test targets for the failure class that bit
rounds 0 and 1: import closure (every `use` in every `tests/*.rs` resolves to a
lib re-export or a `common` helper that exists), pattern types (`{ limit, .. }`
on single-field variants is legal; `Vec<&str>` vs `[&str; 3]` comparisons have
a std `PartialEq` impl), temporary lifetime of `for dir in &format_dirs()`,
and unused-import warnings under `-D warnings`. Nothing found. This is hand
verification — the dated gate evidence remains the outstanding checklist in the
build notes, as it must be run on a first-party toolchain.

## Findings

1. **minor** — `docs/designs/008-rust-for-the-product/03-build-notes.md:62-66,
   118-120, 252-296` (status text about `crates/core/tests/`). The tree now
   contains the four test edits the notes list as pending: `conformance.rs:122`
   uses `class.id.clone()`, `reach_tables.rs:112-128` asserts exact id lists,
   `gate_contract.rs:301` pins the fixed cargo-deny form, and `parse_budget.rs`
   gained the stream budget half. The notes still say the load assertion
   "does not compile yet", that clippy/test "stay red" until the test pass, and
   carry the whole "Test changes the next stage must make" section in the
   future tense. The changed code makes those statements false about HEAD.
   Fix: mark items 1–4 as landed (noting the truth table landed in
   `reach_tables.rs`), drop the `conformance.rs:119` caveat, and leave only the
   dated toolchain checklist outstanding.

FYI (no severity, outside this seat): the stream budget accept case uses a
small document rather than exactly `MAX_DOCUMENT_BYTES` as the build notes
prescribed — coverage detail for the testing seat.

## Verdict

All three round-1 blockers are fixed at the exact sites, the nit's contract
half landed, and no new correctness gap was introduced in the changed source
and test paths. One open minor.

verdict: approve
