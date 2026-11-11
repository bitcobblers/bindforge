# Code review — correctness seat, round 1 dissent pass (c1)

Scope: `git diff ea54086ca6562fb2b1bf69a25bf071479cb09b27..HEAD -- ':(exclude)docs/designs'`, read
against `02-proposal.md`, `03-build-notes.md`, and `docs/adrs/008-rust-for-the-product.md`.
This pass checks first whether the last revision answers my round-1 findings, then reports only
what the changed code newly makes true.

## Round-1 findings — verification

- **F1 (major, closed entry sets)** — *addressed in substance.* `document.rs:39-43,151-165`
  defines the closed `PROFILE_ENTRIES`/`REACH_CLASSES_ENTRIES` sets and `reject_unknown_entry`
  refuses any key outside them with a typed `InvalidValue { key, message }` naming the key and
  format — exactly the fix I asked for (the `InvalidValue` spelling was the allowed alternative to
  `UnknownEntry`). Both corpora now ship `unknown-entry.toml` (profile: the `derives_` alias case;
  reach-classes: `scope`), and `grammar_closure.rs` pins the behavior in-memory and against the
  corpus. **However**, the refactor that landed it broke the crate's build — see blocker 1.
- **F2 (major, TOML-subset contract wording)** — *addressed.* `schemas/README.md:31-39` now states
  "TOML (ADR-009) in a documented subset of TOML v1.1.0", both `spec.md` grammars enumerate the
  excluded constructs, the build notes record it as a deviation, and every excluded construct has
  an invalid fixture pinned by `grammar_closure.rs:130-152`.
- **F3 (minor, reach-classes corpus load assertion)** — *addressed in substance.* The test pass
  added `conformance.rs:88-124` `load_corpus_table`: each valid corpus table is loaded through
  `load_reach_tables` in a scratch directory and its classes asserted filed under its declared
  hand. That closes the gap I reported — but the suite does not compile (blockers 2 and 3), so the
  assertion never runs.
- **F4 (minor, untested parse budget)** — *addressed.* `tests/parse_budget.rs` boundary-tests
  exactly-`MAX_DOCUMENT_BYTES` accepted / +1 refused (`DocumentTooLarge` naming the limit) on both
  the text and the pre-read file path, and 16 vs 17 array levels against `MAX_NESTING_DEPTH`;
  `nesting-past-limit.toml` puts the 17-level case in the language-neutral corpus.
- **F5 (minor, gate commands never run)** — *partially addressed.* `src/` hand-wrapping was
  collapsed and `{kind}` is now inlined at `conformance.rs:305`. The three gate commands have
  still never been run (the build-notes checklist boxes are open), and this revision introduces
  three hard compile errors — the unrun gate is exactly what would have caught them.
- **F6 (nit, module-doc precision)** — *addressed.* `parse_integer` now follows TOML's decimal rule
  (`toml.rs:279-315`: `_` only between digits, no trailing/doubled `_`, no leading zero, no radix
  prefix), and the module doc (`toml.rs:1-18`) states the subset honestly.
- **F7 (nit, budget constants not public)** — *addressed.* `lib.rs:35` re-exports
  `MAX_DOCUMENT_BYTES` / `MAX_NESTING_DEPTH` beside `MAX_CHAIN_DEPTH`, and `lib.rs:17-21` lists
  them; `parse_file`/`parse_reader` apply the cap before allocation, matching `schemas/README.md:51-57`.

## Findings

### Blocker 1 — `bindforge-core` does not compile: `BTreeMap` used without an import
- **Severity:** blocker
- **Location:** `crates/core/src/toml.rs:97,104` (imports at `toml.rs:28-32`)
- **Problem:** `read_document` returns and builds a `BTreeMap<String, Value>`, but the module's
  only `use` statements are `std::fs`, `std::io::Read`, `std::path::Path`, and
  `crate::error::CoreError` — there is no `use std::collections::BTreeMap;` (grep confirms
  `BTreeMap` appears at exactly the two use sites). `BTreeMap` is not in the 2024-edition prelude,
  so the `bindforge-core` lib target fails with E0412. Every downstream target fails with it:
  `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace --locked`
  both error out, zero tests run, and the proposal's deliverable — "compiling green behind one
  cargo gate" — is unmet. This is the same failure mode round 1 flagged in `tests/common/mod.rs`,
  now in `src/`, introduced by the F1/F7 refactor that moved `read_document` into `toml.rs` (the
  import previously lived where `read_document` did, `document.rs`).
- **Fix:** add `use std::collections::BTreeMap;` to `toml.rs`'s import block, and finally tick the
  still-open build-notes box by actually running the three gate commands.

### Blocker 2 — the F3 load assertion cannot run: `class.to_string()` on a type with no `Display`
- **Severity:** blocker
- **Location:** `crates/core/tests/conformance.rs:119`
- **Problem:** `load_corpus_table` maps the loaded classes with `|class| class.to_string()`, but
  `ReachClass` (`reach.rs:61-64`) derives only `Clone, Debug, PartialEq, Eq` — no `Display`, hence
  no `ToString`. The `conformance` test target fails to build with E0599, so the harness walk
  (Test #1) and the newly added corpus-load half (the F3 fix, Test #4) never execute.
- **Fix:** compare ids directly — `.map(|class| class.id.clone())` — or add a `Display` impl for
  `ReachClass` if the id-spelling is meant to be part of the public surface.

### Blocker 3 — the Test #4 distinctness assertion cannot run: `contains` on `&[ReachClass]` with `&&str`
- **Severity:** blocker
- **Location:** `crates/core/tests/reach_tables.rs:111-125`
- **Problem:** the content half of the per-hand distinctness fix — the assertion that replaced
  round 1's tautological `!std::ptr::eq` — calls `left_classes.contains(&"near")` (and the five
  sibling literals `mid`, `extended`, `near`, `mid`, `cross`) on a `&[ReachClass]`.
  `slice::contains` takes `&ReachClass`, and `&"near"` is `&&str`; `ReachClass` implements
  `PartialEq` only with itself. The `reach_tables` target fails to build with E0308, so the one
  assertion that pins per-hand content (Test strategy #4) never runs.
- **Fix:** assert on ids, e.g. map `left.classes().iter().map(|c| c.id.as_str())` and check
  containment of the literals, or pass `&ReachClass { id: "near".to_string() }`.

### Nit 4 — spec and build notes now understate what the shipped harness asserts
- **Severity:** nit
- **Location:** `schemas/reach-classes/v1/spec.md:52-56`; `03-build-notes.md:96-104,228-230`
- **Problem:** after the test pass added `conformance.rs`'s `load_corpus_table`, both the contract
  text (the corpus walk checks "parse, grammar, and the closed entry set"; the load rules are
  asserted "against this directory's own table data") and the notes ("The corpus's load half needs
  a harness change", listed under "Test changes this stage could not make") describe the pre-fix
  state. The harness now asserts more than these documents say it does; the drift is harmless
  directionally but it is drift in artifacts this diff ships.
- **Fix:** one clause in `spec.md`'s Fixtures section ("the harness also loads each valid corpus
  table and files its classes under the hand it declares"), and update the notes' round-1-fix /
  outstanding-test lists to record that the test pass landed.

## Out-of-specialty FYI

No `DATABASE_URL`/Postgres references exist in the workflows, and the gate_contract pins match the
two workflow files I read — informational only, no severity.

## Verdict

`request-changes`. The round-1 majors are substantively answered (closed entry sets, subset
contract wording, load assertion, budget tests, public constants), but the revision as shipped
does not compile — one `src/` import error and two test-target type errors — so no gate command
can be green and the assertions that close F3 and Test #4 never run.
