# Code review — correctness (r1, pass c0)

Diff under review: `git diff 881d3b48641d7dbb6114de03c5770bc12073bc45...HEAD -- ':(exclude)docs/designs'`
(32 files, all additions). Inputs: that diff, `02-proposal.md`,
`docs/adrs/008-rust-for-the-product.md`. No prior-round findings to re-check
(first pass of this seat).

## Checklist walk-through

1. **Named components/interfaces exist and are wired.** Pass. Root
   `Cargo.toml` matches the proposal's snippet (resolver 3, `members =
   ["crates/*", "apps/*"]`, `[workspace.package]` version/edition 2024/
   rust-version 1.85/Apache-2.0, `[workspace.dependencies]` with
   `bindforge-core` + `serde` and the same "enter when their consumer lands"
   comment). `crates/core` → package `bindforge-core` (naming rule applied,
   depends on nothing); `rust-toolchain.toml` channel `1.85.0` equals
   `rust-version`; `.gitignore` gains `/target`; `Cargo.lock` committed; the
   three gate commands appear verbatim in `.github/workflows/check.yml` plus
   the `orchestration` job with `node-version: "22.23.4"`;
   `.github/workflows/advisories.yml` + root `deny.toml` match Gates/Rollout 4
   (job-level `permissions: contents: read`, actions pinned to SHAs); the
   `repo-practices` Rust section carries all five Rollout-5 items; README
   carries the rustup line and the A16 system-deps list. `schemas/` layout
   matches the tree in Formats contract, with the scaffold scoped to
   `profile/v1` + `reach-classes/v1`. `resolve_chain`, `MAX_CHAIN_DEPTH`,
   `Cycle`/`ChainTooDeep`, the 1 MiB / 16-level budget, the version gate and
   `supported_majors()` all exist with the proposal's names.
2. **Assumption register honored.** Pass on the rows the code touches: A7
   (inheritance, globs), A8 (committed lock + `--locked`), A9 (clippy/fmt
   gate), A10/A11 (edition 2024 + pin), A12 (`bindforge-` prefix), A19/A20
   (untouched), A22 (fixture corpus as currency), A23 (nothing in
   `tools/orchestration` touched — confirmed: no `tools/` path in the diff),
   A26 (`env!("CARGO_MANIFEST_DIR")` in `tests/common/mod.rs:36-43`).
3. **Deviations declared.** Not verifiable inside this seat's inputs
   (`03-build-notes.md` lives under the excluded `docs/designs/`), so the two
   gaps found are reported as findings with "declare it" in the fix — see
   F2.
4. **Scope creep.** No behavior beyond the design. The in-tree TOML reader is
   the only way to satisfy Test #1 with `crates/core` dependency-free, so it
   is necessity, not creep; its grammar narrowing is F2.
5. **Build-notes claims.** Out of input (FYI at the end).
6. **Error paths exist and fail closed.** Mostly yes — every refusal is a
   typed `CoreError`, `resolve_chain` is iterative, the size/nesting checks
   run before any model value is built, `load_reach_tables` fails closed on a
   hand/file-name conflict. Two holes: F1 (unknown entries accepted) and
   F4 (budget guards unexercised).
7. **Previous round's blocker/majors.** None to check.

## Findings

### F1 — major — parser accepts entries the shipped spec forbids

`crates/core/src/document.rs:144-201` (`parse_profile`, `parse_reach_classes`)
read only the keys they need and never inspect the rest of the top-level
map, so `format`, `version`, plus any number of unknown entries parse
cleanly. Both grammar tables shipped in this same diff say the opposite:
`schemas/profile/v1/spec.md:22` and `schemas/reach-classes/v1/spec.md:30` —
"Any other entry is rejected." This is the one place the reference
implementation is *more permissive* than the contract it is the reference
for, and it runs against the Adversarial-guard premise ("the invalid-fixture
corpus is the contract for that behavior"): no invalid fixture exercises an
unknown entry, so the harness at `crates/core/tests/conformance.rs:77-93`
cannot notice, and a third-party reader written from `spec.md` rejects
documents `bindforge-core` accepts — the divergence the ADR-008 contract
exists to prevent, and a fail-open path into the ADR's "hub validates
uploads with the same compiled types".

Fix: after the required entries are read, reject every key outside the
format's table set (`document.rs`), with a typed
`CoreError::InvalidValue { key, message: "not part of the <format>/v1
grammar" }` (or a dedicated `UnknownEntry` variant), and add
`schemas/profile/v1/fixtures/invalid/unknown-entry.toml` and
`schemas/reach-classes/v1/fixtures/invalid/unknown-entry.toml` so the corpus
pins it for every implementer.

### F2 — major — reference reader narrows the format away from TOML v1.1.0

The design fixes the contract as "Formats are TOML (ADR-009) over TOML
v1.1.0 (A20)". `crates/core/src/toml.rs:38-83` instead accepts a hand-rolled
line-oriented subset and *rejects legal TOML v1.1.0*: table headers
(`toml.rs:52-58`), dotted keys, multi-line arrays (`toml.rs:251`), floats,
hex integers. The narrowing is stated only in the new contract-tree docs
(`schemas/README.md:31-38`, the two `spec.md` grammar sections); it is a gap
against the proposal's format claim and against A20, and it changes which
documents the language-neutral contract admits — anyone using a standard
TOML parser (every ADR-009/ADR-012 author, every third-party reader) accepts
a `derives = [\n "a",\n]` profile that `bindforge-core` rejects with
`Malformed`.

Fix (either half, then re-test): record the subset as a design deviation in
`03-build-notes.md` with the reason (dependency-free scaffold), and make
`schemas/README.md` say the v1 grammar is *this subset of* TOML v1.1.0 rather
than "TOML over TOML v1.1.0"; or parse with the real `toml` crate and keep
the subset as fixture guidance only. If the build notes already record it,
this reduces to the README wording.

### F3 — minor — reach-classes fixtures are asserted by parse, not by load

`crates/core/tests/conformance.rs:56-60` skips re-serialization for
`reach-classes` and then `continue`s, so a valid reach-classes fixture is
only ever parsed; the load half never sees it. Both the proposal ("Reach-class
table fixtures are asserted by load + distinctness (Test strategy #4)") and
the spec shipped in this diff (`schemas/reach-classes/v1/spec.md:41-44`:
fixtures are "asserted by load and per-hand distinctness") promise a load
assertion, while `crates/core/tests/reach_tables.rs:26-31` loads only the
version directory's `left.toml`/`right.toml` data. The per-hand distinctness
promise is kept for the data files and lost for the corpus, which is the
half ADR-009 will extend.

Fix: for each `reach-classes/v1/fixtures/valid/` fixture, assert the parsed
document is a `ReachClassTable` whose `hand` matches what the harness
expectation for that file is (or drive the loader over a temp dir), and pair
that with the invalid-fixture rejection; or state in `03-build-notes.md` that
this scaffold asserts those fixtures by parse + guard only, and correct the
spec wording.

### F4 — minor — the parse-budget guards have no test at all

`MAX_DOCUMENT_BYTES`/`MAX_NESTING_DEPTH` and the `DocumentTooLarge` /
`NestingTooDeep` variants (`crates/core/src/toml.rs:23-26,39-43,229-233`) are
reached from nothing: no fixture, no unit test (only `src/` references these
names). The design decides this budget here as a constraint ADR-009 must
adopt and publishes it for third-party readers in `schemas/README.md:46-47`,
yet only the chain bound is boundary-tested. A regression that drops the
size check or off-by-ones the nesting limit keeps every test green, and the
harness deliberately cannot carry a 1 MiB fixture.

Fix: add `crates/core/tests/parse_budget.rs` (unit, no fixture) asserting
`parse` on 1 MiB + 1 bytes is `DocumentTooLarge` and on 1 MiB is not, and
that a 17-level nested array is `NestingTooDeep` while 16 levels is not —
written against the constants, as `chain_guard.rs` does for
`MAX_CHAIN_DEPTH`.

### F5 — minor — gate greenness is unproven in this change

The design's deliverable is a workspace "compiling green behind one cargo
gate" (Context; Test strategy #5). `cargo fmt --all -- --check` and
`cargo clippy --workspace --all-targets -- -D warnings` were not run by this
seat (no toolchain access), and two spots look hand-wrapped rather than
rustfmt-canonical: `crates/core/src/document.rs:100-103` and
`document.rs:174-180` each fit inside 100 columns and rustfmt normally
collapses them; and `crates/core/tests/conformance.rs:159-163` passes the
simple binding `kind` to `{}` in
`"fixtures/{} of {} holds names that differ only by case"`, which
`clippy::uninlined_format_args` (style, on by default) flags under
`-D warnings`.

Fix: run the three gate commands from a clean checkout (Rollout 7 / Test #5)
and apply their own output — `cargo fmt --all`, and inline `{kind}` at
`conformance.rs:161`.

### F6 — nit — reader is laxer than its own "accepts exactly" claim

`crates/core/src/toml.rs:1-11` says the reader "accepts exactly that grammar
and rejects everything else", but `toml.rs:117` admits `-` inside bare keys
(and quoted keys as keys) and `toml.rs:206` accepts a trailing underscore, so
`1_` parses as the integer 1 although TOML forbids it. Neither widens the
model surface (typed entry reads reject non-strings), so this is
documentation/precision only. Fix: tighten `parse_key`/`parse_integer` to the
bare-key and integer rules, or soften the module doc.

### F7 — nit — budget constants are unreachable from entry points

`crates/core/src/toml.rs:23,26` are `pub` but the module is private
(`crates/core/src/lib.rs:21`), so `MAX_DOCUMENT_BYTES` is not exported while
`MAX_CHAIN_DEPTH` is (`lib.rs:23`). The untrusted-input premise wants a
future CLI/hub to cap size before it allocates a `String`; today it can only
re-read the README's number. Fix: re-export both constants from `lib.rs`.

## FYI (out of specialty, no severity)

Checklist items 3 and 5 (deviations and claims in `03-build-notes.md`) are
outside this seat's stated inputs and outside the diff (`docs/designs/` is
excluded), so they were checked only against the contract-tree docs shipped
in the diff; F2's fix is written to close either half. Separately: the diff
touches no path under `tools/orchestration`, consistent with the proposal's
non-goal.
