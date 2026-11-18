# Code review — compatibility seat, round 1 pass c0

- **Design:** `docs/designs/024-game-slug-identity/02-proposal.md`; source ADR
  `docs/adrs/024-game-slug-identity.md`.
- **Scope (caller's, authoritative):** `git diff e786de8f6820d71dbe919300e247f72bd59ef7f3...HEAD -- ':(exclude)docs/designs'`
  — 46 files, +2733/−287: `crates/core/src`, `crates/core/tests`,
  `crates/discovery/src`, `crates/discovery/tests`, `apps/discovery-cli`,
  `Cargo.lock`, `schemas/` (specs + fixtures), `docs/adrs/024-*.md`.
- **Checklist applied:** `references/compatibility.md` only. No other seat's
  report or transcript was opened.
- **Inputs used:** the diff; the proposal; ADR-024; the runner's gate record
  (2026-11-18T19:08:59Z, green: `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace --locked` ending `test result: ok. 0 passed; 0 failed;`);
  surrounding source read only to establish the findings below
  (`.github/workflows/check.yml`, `Cargo.toml`, `crates/discovery/Cargo.toml`,
  `apps/discovery-cli/Cargo.toml`, `crates/core/src/policy.rs`,
  `docs/adrs/013-game-manifests.md`).
- **Prior findings:** none (first pass).

## Checklist walkthrough

**1. Platform code matches the Compatibility matrix — satisfied.**
The matrix's Windows row claims compile verification at
Test strategy #11's level, and that leg exists and is workspace-wide:
`.github/workflows/check.yml:29-32` runs
`cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`, so the
new `crates/discovery/tests/common/mod.rs` and the re-pinned
`steam_windows_tests.rs` are type-checked for that target even though no test
binary runs there. Nothing in the diff adds a `cfg(target_os)`/`cfg(unix)`/
`cfg(darwin)` path: `grep` over the diff's added lines finds no new `#[cfg(`
guard and no new platform-conditional code, and the only platform guards in
play are the pre-existing ones (`crates/discovery/src/steam_windows.rs:88,196`
for the registry reader; `apps/discovery-cli/src/main.rs:58-65` for which
plugin answers `--steam-root`). The new derivation, collision pass and boundary
check are std-only and platform-neutral (`BTreeMap<&str,&str>` gives byte-order
iteration with no locale collation), matching the matrix's
"derivation, collision and refusal results are platform-independent" claim.
macOS: still no product code path and no leg, as stated.
Note for the record: the runner's gate record contains the cargo trio and the
npm check but not the Windows compile leg, so that leg's status is established
from the workflow file, not from gate evidence.

**2. Node floor — not applicable.** No file under `tools/orchestration` is in
scope; `npm run check` is green in the gate record. The matrix's claim that
nothing there reads the changed surface holds: `tools/orchestration/src`
mentions only its own `nnn-slug` run-identity grammar
(`pipeline/types.ts:281-291`), which shares nothing with `MAX_SLUG_OCTETS`.

**3. Paths — satisfied.** No new path is built by concatenation:
`install_path` still composes through `Path::join`
(`crates/discovery/src/steam_common.rs:242-256`), the fixture-corpus test joins
with `.join("fixtures").join("invalid")`
(`crates/core/tests/slug_bound_corpus.rs:72,132`), and the temp-dir helpers join
(`crates/discovery/tests/common/mod.rs`, `cli_slug_refusal_tests.rs`). Case
sensitivity stays safe: the bound rise cannot introduce a case collision,
because `is_slug_name` is lowercase-ASCII-only, which is what the policy spec's
"case-unique on a case-insensitive filesystem" requirement
(`schemas/policy/v1/spec.md:170`) leans on; `is_slug_name`'s refusal set
(`.`/`..`/`/`/`../elden`) is pinned at `crates/core/tests/placement.rs:864-887`.

**4. Guarded platform imports — satisfied.** `winreg` remains behind
`[target.'cfg(windows)'.dependencies]` (`crates/discovery/Cargo.toml:21-22`);
the two new edges (`discovery → bindforge-core`, `discovery-cli → bindforge-core`)
are unconditional workspace paths (`Cargo.toml:21`) and carry no platform-only
module. `serde_json` is added to discovery as a dev-dependency only, pinned the
way `discovery-cli` already pins it.

**5. Locale/layout reality — satisfied, and materially strengthened.** IDs and
formats survive non-ASCII input: `derive_slug` emits pure ASCII by construction
(`crates/discovery/src/slug.rs:69-99`: fold table → ASCII case-fold → `-` for
everything else), so octets and chars agree at the bound, and the fold table's
covered ranges carry no stray code point (I walked each range: `0128..=012F` and
`0130` hold only i-family letters with base `i`, and dotless `ı` U+0131 is
correctly outside the table, as its doc comment claims). Non-ASCII punctuation
folds to `-` (`’`, `™`, `：`), non-Latin-only names derive `None` and are
refused rather than emitted, and the CLI echo runs through the escaped renderer.
`crates/discovery/tests/slug_tests.rs:262-289` pins the ASCII-only promise plus
the no-separator promise.

**6. Native dependencies introduced — not applicable.** `Cargo.lock` gains only
workspace-internal edges plus `serde_json` (pure Rust) for a dev-target. No C
toolchain, no build script, no CI matrix change needed.

**7. Platform claims of the tests — satisfied, with two nits below.** No test in
the diff is `#[ignore]`d or guarded into never running; every new test executes
on the Linux leg. Test strategy #11's reserved-name pin is real
(`slug_tests.rs:241-257`, `emission_tests.rs:237-250`) and asserts only
platform-independent behaviour. The spec-boundary corpus is exercised for all
four trees (`slug_bound_corpus.rs`), including the new `manifests/v1`
`id-past-cap.toml`, and the superseded
`segment-past-63-octets.toml` name is pinned out of the tree.

**Contract-amendment cross-check (matrix "Third-party schema readers").** The
amendment really is relaxation-only and really is stated once: `grep` for `63`
across `schemas/**/*.md` returns exactly one hit — the README's own
"an id valid under the 63-octet text stays valid" clause — so no published spec
restates a number; all four specs cross-reference the README bullet, and the
four boundary fixtures sit at 129 octets per the corpus test.

## Findings

### minor — `apps/discovery-cli/tests/cli_slug_refusal_tests.rs:25-26, 33, 113`

The CLI-side boundary pin retypes the shared bound as a file-local
`const SLUG_BOUND: usize = 128` and re-implements the grammar as
`fn is_slug_shaped(slug: &str) -> bool`, on the stated premise that "this
crate's tests deliberately keep no `bindforge-core` dependency". That premise
is false for this change: `apps/discovery-cli/Cargo.toml:8-11` adds
`bindforge-core = { workspace = true }`, and Cargo makes a package's
`[dependencies]` available to its `tests/` targets — which is exactly how
`crates/discovery/tests/slug_tests.rs:23` and `game_tests.rs:131-140` reach
`bindforge_core::{MAX_SLUG_OCTETS, is_slug_name}` with the same dependency
shape. Consequence: the one pin that proves "every slug reaching a rendering is
inside the shared bound" asserts a number the product no longer reads, so the
next bound amendment (which `024-L3` explicitly plans for) leaves this
corpus-style pin enforcing 128 while `main.rs:89-96` enforces whatever the
constant says; the hand-rolled predicate has the same drift risk in the other
direction, and it is a second grammar in the tree — the thing the proposal's
Options section rejected for `discovery`.
**Fix:** delete `SLUG_BOUND` and `is_slug_shaped` and assert against
`bindforge_core::MAX_SLUG_OCTETS` / `bindforge_core::is_slug_name`, as the
discovery suites already do; drop the `//!` note that justifies the copy.

### nit — `apps/discovery-cli/tests/cli_slug_refusal_tests.rs:258, 328`

Both assertions hard-code the Linux answer to a `cfg`-dependent choice:
`apps/discovery-cli/src/main.rs:58-65` feeds `--steam-root` to
`SteamLinuxPlugin` on non-Windows and `SteamWindowsPlugin` on Windows, so the
expected `"error: plugin steam-linux failed: "` prefix and the
`"steam-linux"` Platform cell are platform-specific outcomes, while Test
strategy #11 states that "no test asserts a platform-specific outcome". Nothing
breaks in today's CI (no Windows or macOS test leg), so this is a forward
hazard for the leg the matrix's Windows row eventually wants, and it follows a
pre-existing pattern in `cli_tests.rs` rather than inventing one.
**Fix:** derive the expected plugin name (or accept either spelling) and say in
the file header that these two assertions ride the Linux leg only.

### nit — `crates/discovery/src/slug.rs:19`

The stroke-letter sample in the fold table's coverage comment lists `ł` twice —
`` (`ł`, `đ`, `ħ`, `ŧ`, `ł`) `` — where the fifth entry was presumably meant to
be `ŀ` (U+0140) or `Ł` (U+0141). This comment is the stated boundary that Test
strategy #1 pins, so its sample set should be the real one; the code's ranges
are correct either way.
**Fix:** correct the duplicated code point in the doc comment.

## FYI (out of specialty, no severity)

`013-L5` in `docs/adrs/013-game-manifests.md:114-116` carries no
`amended-by 024-B1` annotation and the diff never touches that file, so the
ledger step of Rollout item 3 (ADR-024 Rollout 5, per `021-L4`) is unrecorded —
the mechanism by which a third-party reader traces which bound now governs the
`id` it parses. Flagging for whichever seat owns ledger bookkeeping.

## Verdict

No blocker or major finding: the platform story, the Node floor, path handling,
guarded imports, the locale/ID-fidelity story and the dependency surface all
match the proposal's Compatibility matrix, and the gate record is green. Two
minor/nit drift hazards inside the new test surface, neither of which changes
what ships.
