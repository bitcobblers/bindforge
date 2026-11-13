# Code review — compatibility seat, round 1 pass c0

Diff under review: `git diff c26bf7f02371a1535a4d956d00294479b94d5532...HEAD -- ':(exclude)docs/designs'`
(38 files, +3496/−79: `crates/core/src/{input,placement,document,reach,lib}.rs`, seven test
targets, `schemas/reach-classes/v2/`, `schemas/keyboard-geometry/v1/`,
`schemas/reach-classes/v1/fixtures/invalid/keys-entry.toml`, `schemas/README.md`).

Inputs used: the diff, `02-proposal.md`, `docs/adrs/012-input-vocabulary-and-reach-classes.md`.
No other review report or transcript was read.

## Verdict

`request-changes` — one open major (the version gate stops being per-format, so two
untouched formats widen what the reference reader accepts). Everything else on the
checklist passes; the rest are minor/nit.

## Checklist walk

1. **Platform code vs. the Compatibility matrix** — pass. No `#[cfg(target_os = …)]`,
   no OS-specific branch, no platform-only module anywhere in the diff (`grep` for
   `cfg(` / `target_os` over `crates/core` returns nothing), so nothing sits behind a
   guard the matrix does not claim, and every "supported" row has a code path:
   the Linux row is real (below), the Windows row's claim is a data claim
   (`left`/`right`/`ansi-us` stems and every new fixture name are lowercase ASCII,
   and `conformance.rs:326` `fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique`
   iterates `format_dirs()`, so it does cover both new directories).
   Rows the matrix marks `no`/`not applicable` add nothing in the diff — no macOS or
   SteamOS-specific code, no binary, no runner.
2. **Toolchain floor (1.85.0, edition 2024; the proposal's "std-only, no new
   dependency, `Cargo.lock` byte-unchanged")** — pass by inspection. The newest std
   API the diff introduces is `Option::is_none_or` (`placement.rs:197`, stable 1.82);
   everything else (`let-else`, `u32::try_from`, `str::strip_prefix`, `BTreeMap`,
   `Path::join`) predates 1.85. `Cargo.toml`, `crates/core/Cargo.toml` and
   `Cargo.lock` are absent from the diff, and `crates/core/Cargo.toml` still declares
   no dependencies. (I could not compile against 1.85.0 specifically — `rustc`/`rustup`
   are not reachable from this seat — so this is an inspection result, not a build leg.)
3. **Paths** — pass. `load_geometry` joins (`dir.join(format!("{name}.toml"))`,
   `placement.rs:151`) after grammar-checking `name`; `load_reach_tables` keeps its
   existing join; `check_dir_version` reads the segment through `dir.file_name()`.
   No hand-concatenated path reaches `std::fs` (the harness's
   `format!("{major}/{other}.toml")` at `conformance.rs:126-127` goes through
   `ScratchDir::write` → `Path::join`, and `/` is a Windows separator, so it is fine).
   Case sensitivity is byte-exact everywhere (`geometry_matches_file_name`,
   `check_dir_version`), and every loader-side name grammar is ASCII-lowercase, so a
   case-insensitive filesystem cannot alias two geometries or two tables.
4. **Guarded platform imports** — pass, vacuously: no platform-only module is imported.
5. **Locale/layout reality** — mostly pass, one gap (finding 2). Ids are ASCII-only
   (`is_keyboard_member` uses `is_ascii_lowercase`/`is_ascii_digit`, so `keyboard:é`
   is refused rather than silently accepted); geometry names are `[a-z][a-z0-9-]*`, so
   file names stay portable; the backslash glyph round-trips as `\\` in every shipped
   file (`left.toml`, `right.toml`, `ansi-us.toml`, `punctuation-glyphs.toml`,
   `sparse-grid.toml`) and through `serialize`'s `quoted`; the ANSI-US assumption the
   non-QWERTY risk turns on is stated where a reader sees it
   (`reach-classes/v2/spec.md:30-37`, `keyboard-geometry/v1/spec.md:12-14`).
   Refusal echoes stay UTF-8-safe because `error.rs::one_line` walks `chars()`.
6. **Native dependencies** — pass: none introduced (no `Cargo.toml`/`Cargo.lock` hunk).
7. **Tests run where they claim** — pass for the leg the diff can evidence. I ran the
   three gate commands on this Linux checkout: `cargo fmt --all -- --check` clean,
   `cargo clippy --workspace --all-targets --locked -- -D warnings` clean, and
   `cargo test --workspace --locked` green — 14 targets, 118 tests, **0 ignored**, so
   no platform-specific test is skipped-everywhere and none is `#[cfg]`-gated away.
   The new targets all run: `input` 6, `reach_v2` 17, `placement` 17, `zz_api_probe` 1,
   and the untouched `reach_tables` (5) and `version_gate` (4) stay green, which is the
   matrix's "third-party v1-only reader: v1 refusals stay byte-exact" row for the
   table data. The Windows leg of Test strategy #8 is a human leg; the plan already
   routes it to a contributor machine, and nothing in the diff blocks it.

## Findings

### 1. major — the version gate is no longer per-format, so `profile/v2` and
`action-ids/v2` documents are now accepted under the v1 grammar

Location: `crates/core/src/document.rs:186-191` (the gate, now fed by
`supported_majors()` = `[v1, v2]` at `document.rs:100`), the `profile` and
`action-ids` arms at `crates/core/src/document.rs:194-208`,
`parse_profile` at `document.rs:379`, `registry_from_entries` at
`crates/core/src/action.rs:179`.

Problem: the gate is format-blind, and only the new `keyboard-geometry` arm checks
the format's own majors (`parse_keyboard_geometry`, `document.rs:531-536`:
`if version != MAJOR_V1 → UnsupportedVersion`). Adding `MAJOR_V2` to the crate-wide
list therefore changes the acceptance of two formats this diff declares untouched
(delta inventory: "Unchanged: … `action.rs`, `profile/v1`"). A document that
declares `format = "profile"` (or `action-ids`) and `version = "v2"` now passes the
gate, passes `reject_unknown_entry` against the *v1* entry list
(`reach_classes_entries` returns the v1 set for any version that is not `v2`, and
`profile`/`action-ids` never reach that function), and parses as a v1 profile — where
before this diff it was refused as `UnsupportedVersion` naming the supported majors.
That is exactly the fail-open `tests/version_gate.rs:8-12` says the gate exists to
prevent ("a document written against a future major parsed with v1 semantics"), it
contradicts the ADR's "extending them means a new major, never widening v1", and it
is unpinned: `version_gate.rs:45` only tests `v99`, and no corpus fixture declares a
major its directory does not carry. The next format major in the 012-015 series
(manifest, patch, policy — README already advertises `v1`, `v2`, … per format)
inherits the hole.

Fix: make the gate per-format and keep it in one place. Either a
`format_majors(format) -> &'static [&'static str]` consulted right after the format
lookup (`profile` → `["v1"]`, `action-ids` → `["v1"]`, `keyboard-geometry` → `["v1"]`,
`reach-classes` → `["v1", "v2"]`) with the same `UnsupportedVersion { declared,
supported: format_majors }` refusal, or mirror the geometry arm's guard in the other
two arms. Then pin it in the contract, not only in Rust: a
`profile/v1/fixtures/invalid/`-style refusal plus a unit case that a `profile`
document declaring `v2` is refused as `UnsupportedVersion`, alongside the existing
`rejects_a_document_declaring_an_unsupported_major…`.

### 2. minor — the ASCII-only half of the id grammar is nowhere in the corpus, so a
non-Rust reader can accept `keyboard:é`

Location: `schemas/reach-classes/v2/spec.md:24` ("An id is `<namespace>:<member>`:
lowercase, no spaces, exactly one colon") and its invalid-fixture set
(`fixtures/invalid/uppercase-member.toml` is the only case-spelling fixture);
`crates/core/src/input.rs:170-181` (`is_keyboard_member`, ASCII-only);
`crates/core/tests/input.rs:110-144` (no non-ASCII case).

Problem: checklist item 5 is about non-ASCII input names, and this is the format
that states the grammar every later format reuses. The Rust reader refuses a
non-ASCII member, but "lowercase" is Unicode-lowercase to a reader implementing the
spec in another language (`é`, `ß`, `İ` are all lowercase under a Unicode-aware
check), and the corpus — which the contract names as "the conformance currency"
(`schemas/README.md:46-48`) — never exercises the case. So the first non-US keyboard
user gets two readers that disagree about the same document. The precedent already
exists two directories away: `schemas/action-ids/v1/fixtures/invalid/non-ascii-id.toml`.

Fix: add `schemas/reach-classes/v2/fixtures/invalid/non-ascii-member.toml` (one
fault, `keys = ["keyboard:é home"]`, refused naming the member), a matching case in
`tests/input.rs::rejects_an_id_that_breaks_the_namespace_or_member_grammar`, and the
word "ASCII" in the spec line: "a lowercase ASCII glyph of the ANSI-US set".

### 3. minor — `reach-classes/v2/spec.md` still tells implementers the stand-in
directory is not named for a major, which the shipped harness now contradicts

Location: `schemas/reach-classes/v2/spec.md:150-154` (the parenthetical "(A stand-in
directory is not named for a major, so the directory/version rule above does not
apply to it; the version gate that already ran does.)") against
`crates/core/tests/conformance.rs:121-130`, which now writes
`{major}/{other}.toml` and loads from `scratch.path().join(major)` precisely because
`check_dir_version` (`document.rs:623`) does apply.

Problem: the spec is the canonical statement a third-party implementer mirrors, and
the reference harness it mirrors now names the stand-in directory with the fixture's
own major — the diff changed the harness (`load_corpus_table(label, text, major)`)
and left the sentence that describes the old layout. The rule itself is stated
correctly earlier (`spec.md:58-60`, `keyboard-geometry/v1/spec.md:21-22`); it is the
fixtures paragraph that now describes behavior the corpus does not have, which is
the kind of drift that makes two implementers build different stand-in directories.

Fix: reword the parenthetical to what the harness does: the stand-in directory is
named for the fixture's own major, and the table that completes the pair is taken
from the same major directory, so the version/directory rule holds there too.

### 4. nit — the "directory is named for a major" rule now exists twice, with two
different strictnesses

Location: `crates/core/src/document.rs:645` (`is_major_dir`, new) beside
`crates/core/src/action.rs:229-238` (`is_major_dir` + the requirement that a registry
directory *must* be a major directory); the new lenient use is
`check_dir_version` at `document.rs:623-644`.

Problem: the same contract rule ("a document sits directly in a `vN/` directory") is
defined in two private helpers that can drift, and they answer differently for the
same input — a non-major directory is a refusal for the registry and a pass-through
for reach tables and geometries. That divergence is deliberate (harness stand-ins),
but it is currently carried only by two copies of the predicate.

Fix: one `pub(crate) fn major_dir_segment(dir: &Path) -> Option<&str>` in `document`,
used by both loaders, with each loader deciding what `None` means.

### 5. nit — `crates/core/tests/zz_api_probe.rs` is a new test target the proposal's
delta inventory does not list

Location: `crates/core/tests/zz_api_probe.rs` (new, 107 lines);
`02-proposal.md:127-146` names exactly three new test targets
(`tests/input.rs`, `tests/placement.rs`, `tests/reach_v2.rs`).

Problem: the target is worth having from this seat — it resolves every name ADR-015
compiles against at the crate root, which is where a rename would break a consumer
that has not been written yet — but it is outside the inventory the build was
measured against, and ADR-004's module-mirroring is the repo's stated rule for test
targets.

Fix: record it (one line in the delta inventory / Rollout 3, via the doc-sync leg),
or fold the probe into `tests/placement.rs`.

## FYI (no severity, out of specialty)

- `serialize`'s reach arm (`document.rs:312`) writes a `keys` array for a
  v1-declaring table whenever `keys` is non-empty, which would emit a v1 document
  its own reader refuses. Unreachable from `parse`, so it is only a
  hand-built-model hazard — flagging for the correctness seat.
- `tests/common/mod.rs::quoted_array` (new) is a second, hand-written reader of
  contract files next to `string_array`; it correctly splits outside quotes (the
  `,` glyph makes a naive split wrong), but it is one more place the grammar lives.
