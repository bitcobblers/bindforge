# Code review — compatibility — round 1, pass c0

Seat: compatibility. Diff: `git diff 4aeb57c2188fbdfc1ca68d99080e4d2f3950d945...HEAD -- ':(exclude)docs/designs'`
(30 files, +1860/−37). Judged against `02-proposal.md` and
`docs/adrs/011-action-taxonomy-and-core-skeleton.md`. Previous seat findings: none.

## Checklist result

| # | Item | Result |
|---|---|---|
| 1 | Platform code matches the Compatibility matrix | pass |
| 2 | Toolchain floor | pass |
| 3 | Paths built with joins; case-sensitivity assumptions flagged | finding 1, finding 4 |
| 4 | Guarded platform imports | pass (nothing platform-only) |
| 5 | Locale / layout reality | pass, with finding 3 on the corpus leg |
| 6 | Native dependencies | pass (none introduced) |
| 7 | Tests run on the platforms they claim | finding 2 (unevaluable → finding) |

### What passes, and on what evidence

- **Item 1 — matrix.** No OS-specific code anywhere in the diff: no `cfg`, no syscall,
  no env probing, no path convention. The two `yes` rows have real legs on the platform
  they name. Linux CI (`check.yml:17-33`, untouched by this diff — the A9 "no CI change"
  claim holds): `cargo test --workspace --locked` auto-discovers the two new targets
  `tests/action.rs` and `tests/action_ids.rs`, and the `schemas/*/v*/` walk in
  `crates/core/tests/common/mod.rs:73-101` picks up `schemas/action-ids/v1` with no
  harness change, which `conformance.rs:47-78` and `:138-154` then exercise generically
  — the round-trip half works because `Document::ActionIds` derives `PartialEq` and
  `serialize` writes the three arrays back in `ids()` order. The macOS / Steam Deck /
  non-Rust-reader rows say `no (v1)` and the diff adds nothing for them — consistent.
  "std-only, no new dependency, `Cargo.lock` byte-unchanged" (matrix line 289): the diff
  touches no manifest, and `action.rs` imports only `std::collections::BTreeMap` and
  `std::path::Path`; `document.rs`/`lib.rs` add no `use` outside the crate.
- **Item 2 — floor.** Floor is `rust-version = "1.85"` + `channel = "1.85.0"`, edition
  2024. Every construct the new code uses predates it by a wide margin: let-else
  (`action.rs:113`), `str::split_once`/`strip_prefix`, `u8::is_ascii_*`, array
  by-value `IntoIterator` (edition 2021), inline format args, `BTreeMap`. The new
  fixtures are read with the existing `fs::read_to_string`; nothing calls an API above
  1.85. No native or workspace dependency enters the tree, so the pinned lock story and
  `--locked` gate are unaffected.
- **Item 3 — paths.** `load_action_registry` builds its file path with
  `dir.join("registry.toml")` (`action.rs:221`), never concatenation; tests build paths
  with `join` chains (`action_ids.rs` `dir.join("fixtures").join("valid").join(fixture)`)
  and with the pre-existing `ScratchDir::write`, which joins then creates the parent.
  `serialize`'s one-line array form is a document format, not a path. CRLF is safe on a
  Windows contributor's checkout: the reader takes `text.lines()` and trims each line
  (`toml.rs:106-108`), so a `core.autocrlf` checkout of `registry.toml` or of the new
  fixtures parses identically.
- **Item 4.** No platform-only module is imported, lazily or otherwise.
- **Item 5 — locale/layout.** The id grammar is byte-exact ASCII and the refusal path is
  non-ASCII-safe: `is_well_formed_segment` (`action.rs:275-286`) tests bytes and fails on
  any non-ASCII byte, the bound is counted in octets exactly as `spec.md:31-35` states,
  and `CoreError`'s `one_line` truncates by `chars()` and escapes control characters
  (`error.rs:155-169`), so the echoed id can neither split a scalar value nor forge a
  line. Both directions are pinned: unit cases (`action.rs` `rejects_an_id_that_breaks_the_segment_grammar`
  covers `möve.forward`, `move.stéer`) and a corpus case (`fixtures/invalid/non-ascii-id.toml`)
  — so an id that survives the format does not exist, which is the design's answer rather
  than an omission. Ids are semantic, not physical-key names, so non-QWERTY layouts have
  no surface to bite.
- **Item 7 second half.** No new test is `#[ignore]`d or `#[cfg]`-gated (grep across
  `crates/core/tests` finds none), so no platform-specific test is skipped everywhere.
  The Windows half of the name rule the suite claims to cover is real and generic:
  `conformance.rs:277-316` asserts lowercase-ASCII **and** case-unique fixture names for
  every walked format, and all 17 new fixture names pass `is_fixture_name`; every
  `ScratchDir` label the new suites use is ASCII lowercase and hits no Windows reserved
  device name, and the deepest scratch path stays well under 260 characters.

## Findings

### 1. minor — the registry file name's case rule is the filesystem's, not the contract's

`crates/core/src/action.rs:221` resolves the registry with `dir.join("registry.toml")`
and never inspects the name it opened. On a case-insensitive filesystem — the platform
the matrix calls `yes` (02-proposal.md:287) — a version directory holding
`Registry.toml` loads as the registry; the same tree on the Linux CI runner is refused
with an `Io` refusal. The directory half of the same identity check *is* byte-exact
(`is_major_dir`, `action.rs:289`, refuses `V1` on every platform), so the loader's name
matching is asymmetric within one function, and `spec.md:14-19` ("`registry.toml`,
sitting directly in this version directory") never says which rule a third-party reader
implements. No fixture or test pins either behaviour.

Fix: pick one and state it. Either pin the name byte-exact — add the sentence to
`spec.md` "Registry data" and compare `file_name()` in the loader — or state that file
lookup follows the filesystem's case rules and the tree ships lowercase names, and let
the matrix's Windows row name that as the case rule for this format.

### 2. minor — Test strategy #7's platform legs stay unevaluated, so the Windows matrix row has no evidence leg

Checklist item 7 asks whether the tests named in the plan actually run on the platforms
they claim. `02-proposal.md:328-331` names two legs — the three gate commands on the
Linux CI runner, and the suite "run once on a Windows contributor machine" (Rollout 5,
:350) — and A16 (:65) records that no stage of this run has ever invoked a toolchain, so
neither the new `action`/`action_ids` targets nor the new corpus has executed anywhere
yet. The diff correctly declines a fake test-file leg (`tests/action_ids.rs:20-26`), so
nothing in the change either runs the Windows pass or names who runs it and when. Until
one of the two legs produces output, "Windows 10/11 contributor: yes" rests on
inspection only, and this checklist item cannot be scored as a pass.

Fix: post the CI `test result:` lines for the `action` and `action_ids` targets into the
run's build notes, and give the Windows run an owner and a date (or state in the matrix
note that the row is inspection-only until that run is filed). The suite is Windows-safe
by reading — see the item 5 / item 7 evidence above — this finding asks for the leg, not
for a code change.

### 3. nit — the new format restates the shared TOML exclusions but ships no fixture for them

`schemas/action-ids/v1/spec.md:21-35` tells a reader of this format that table headers,
dotted keys, multi-line arrays, floats and hex integers are "legal TOML and outside this
grammar, and are rejected". None of the 14 invalid fixtures carries one of those cases,
and the suite that pins them — `crates/core/tests/grammar_closure.rs:131-152` — walks
`profile/v1/fixtures/invalid/` only. Since fixtures are the conformance currency for the
language-neutral corpus, an implementer reading only `action-ids/v1/` gets the claim with
no leg to check it against.

Fix: point the construct walk at every `schemas/*/v*/fixtures/invalid/` (or add the five
cases to `action-ids/v1/`), or add one sentence to `spec.md` naming
`schemas/profile/v1/fixtures/invalid/` as where the shared-construct cases live.

### 4. nit — an undecodable directory name is reported as a wrong naming rule

`crates/core/src/action.rs:229-232` reads the expected major with
`dir.file_name().and_then(|name| name.to_str()).unwrap_or_default()`. For a caller path
whose final component is not valid UTF-8 — ordinary on Linux, reachable on Windows for a
name with an unpaired surrogate — `major` becomes `""` and the refusal claims "a registry
directory is named for a major, like `v1`", pointing the operator at the file's contents
rather than at their path.

Fix: keep the `OsStr` and compare bytes for the expected major, or split the two cases so
an undecodable name gets a message that says the directory name is not usable text.

## FYI (out of specialty, no severity)

`document.rs:446-454` `axis_array` calls `registry.ids()` and re-queries `axis_kind` per
id inside a per-kind filter — three full scans of a `BTreeMap` per serialize; fine at 40
ids, worth a note for whoever reads the perf seat's pass.

## Verdict

approve — zero open blocker and major findings. Findings 1-4 are minor/nit.
