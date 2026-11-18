# Code review — testing — round 1, pass c0

Seat: testing. Scope: `git diff e786de8f6820d71dbe919300e247f72bd59ef7f3...HEAD -- ':(exclude)docs/designs'`
(46 files, +2733/−287). Inputs: that diff, `docs/designs/024-game-slug-identity/02-proposal.md`,
`docs/adrs/024-game-slug-identity.md`, the runner's gate record. No prior findings on this seat.

## Evidence

**Supplied (runner gate, 2026-11-18T19:08:59Z, all green, exit 0):** `npm run check`;
`cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`;
`cargo test --workspace --locked`. The one captured `test result:` line reads
`ok. 0 passed; 0 failed`. Read against the suite, that line is the
`Running unittests src/main.rs` target of `discovery-cli` — the binary's unit-test target, which
still holds **zero** tests. It is a renderer artifact (a green leg, not a failing one), but it is
worth naming because that empty target is exactly where this report's main finding belongs
(finding 1).

**Executed by this seat:** `cargo test --workspace --locked` at HEAD. Green; the visible tails
agree with the gate and with the new suites existing and running: `steam_windows_tests` 26 passed,
`crates/discovery/tests/zz_api_probe.rs` 2 passed, `cli_dual_plugin_tests` 2, `cli_output_polish_tests` 4,
`cli_slug_refusal_tests` 4, `cli_tests` 3, doc-tests 0, `discovery-cli` unittests 0 — no failures,
no ignored tests. No required gate is failing or missing, so item 6 of the checklist passes on
evidence; the findings below are coverage findings, not gate failures.

## Checklist walk

**1 — Test-strategy coverage.** Every numbered item has at least one test citing
`proves: design 024's Test strategy #N`:

| # | Citing tests | Status |
|---|---|---|
| 1 | `crates/discovery/tests/slug_tests.rs:37,68,94,119,138,162,189` (+ `#1, #11` at `emission_tests.rs:242`) | covered — survey slugs (incl. the 64-octet one), `g-` set, `é ü ñ Å É` folding, `ø æ ß` → `-`, `ḗ` → `-`, non-Latin → `None` all present |
| 2 | `crates/core/tests/placement.rs:825,839,866`; `discovery/tests/slug_tests.rs:209`; `emission_tests.rs:81,128` | covered |
| 3 | `emission_tests.rs:61,128,169,189,227` | covered |
| 4 | `collision_tests.rs:54,89,116,135,159,195,225` | covered, except the "and counted" clause → finding 2 |
| 5 | `game_tests.rs:42,65,85,118`; `emission_tests.rs:260` | covered, incl. field order and the `Platform`/`Namespace` spelling divergence |
| 6 | `core/tests/manifest.rs:786,845`; `core/tests/policy.rs:634`; `core/tests/chain_ops.rs:696,717`; `core/tests/action.rs:244`; `core/tests/action_ids.rs:124` (fixture literal) | covered; the constant-interpolation half is pinned for all four formats by `core/tests/slug_bound_corpus.rs:124` |
| 7 | `core/tests/error.rs:24,36,61,89` | covered at the renderer's definition site |
| 8 | `cli_tests.rs:72,139`; `cli_output_polish_tests.rs`; `cli_dual_plugin_tests.rs:84`; `plugin_tests.rs:34,47`; `steam_linux_tests.rs:336,364`; `steam_windows_tests.rs:494,637`; `game_tests.rs:114`; `cli_slug_refusal_tests.rs:283` | covered — the eight `unknown/` files are re-pinned, the old "absence of `unknown/` on paths" pin is inverted to "slugs contain no `/`", and a repo-wide grep for `unknown/` in `crates/**` now returns only negative pins |
| 9 | `cli_slug_refusal_tests.rs:138,181,233` | **partial** — the CLI's own named refusal + escaped echo is unasserted → finding 1 |
| 10 | `core/tests/slug_bound_corpus.rs:62,104,124,161,193` | covered, incl. the superseded-name absence and the new `manifests/v1` fixture |
| 11 | `slug_tests.rs:242,263`; `emission_tests.rs:242` | covered — reserved names pinned as known current behaviour, no platform-specific assertion anywhere |
| 12 | `discovery/tests/zz_api_probe.rs:37,101` | covered |

**2 — Unit and integration layers.** Both exist as promised: unit (pure `derive_slug`,
`escape_and_truncate`, serde over in-memory rows, `is_slug_name` shapes) and integration (plugin
driven through `SteamLinuxPlugin::from_roots` over scratch roots; CLI driven as a subprocess via
`CARGO_BIN_EXE_discovery-cli`).

**3 — Revision rounds.** Not applicable: first pass, no previously rejected blocker/major.

**4 — Determinism.** Passes. No network, no live server, no ports, no wall-clock assertions
(`crates/discovery/tests/common/mod.rs:37` reads `SystemTime` only as a temp-dir nonce, never as an
expected value). Scratch roots are pid/nonce/nanosecond-tagged and removed on `Drop`; the CLI
suite's label+pid scheme (`cli_slug_refusal_tests.rs:45`) is collision-free because each test in
the file uses a distinct label and each file is its own process. Ordering assertions rest on
`BTreeMap` byte order and the shipped `(slug, install_path)` sort, not on `HashMap` iteration. Host
Steam is never probed (`--steam-root`, or the fake-registry seam in `steam_windows_tests.rs`).

**5 — Test names.** Passes. Names state behaviour (`a_slug_at_the_bound_renders_and_one_octet_past_it_does_not`,
`derives_windows_reserved_device_names_unchanged_as_known_current_behaviour`), not steps.

**6 — Gate evidence.** Assessed above; all four legs green and reproduced locally.

**7 — Untestable additions.** Two, reported as findings 1 and 2.

## Findings

1. **[major]** `apps/discovery-cli/src/main.rs:88-102` — the CLI's emit-boundary refuse-and-echo
   branch has no test at any layer. Test strategy #9 (`02-proposal.md:288-290`) promises
   specifically: "a plugin emitting a row whose slug violates the grammar or bound gets that row
   dropped by the CLI **with one named stderr refusal containing the escaped echo**, while valid
   rows still render." What the suite actually pins is the *plugin*-side refusal:
   `cli_slug_refusal_tests.rs:138,181,233` build scratch Steam roots, so the row is refused inside
   `steam_common::discovered_game` and never reaches `main.rs`'s filter — the file's own header
   (`cli_slug_refusal_tests.rs:5-19`) states the injection seam does not exist because the binary
   builds its two plugins itself. `grep` for `refused game with invalid slug` returns exactly two
   hits outside `docs/`: the `eprintln!` itself (`main.rs:99`) and a prose mention in a doc comment
   (`crates/core/tests/error.rs:87`). So the branch is currently dead code with zero assertions:
   an `eprintln!`→`println!` slip, an echo of `install_path` instead of `slug`, a missing
   `escape_and_truncate`, or moving the filter behind `render_table`/`serde_json::to_string` — each
   of which breaks `024-L1` at the boundary ADR-024 adds it for (third-party plugins) — would ship
   green. `error.rs`'s renderer tests pin only what an echo *can* look like, not that the CLI's
   line is built from it. **Fix:** hoist the inline closure into a named function in `main.rs`
   (e.g. `fn refuse_invalid_slugs(games: &mut Vec<DiscoveredGame>) -> Vec<String>` returning the
   formatted `error: refused game with invalid slug: <escape_and_truncate(slug)>` lines), then
   unit-test it in a `#[cfg(test)] mod tests` inside `main.rs` — the target already exists and
   currently reports `0 passed` — over hand-built rows (one invalid slug, one over-bound slug, one
   valid), asserting the valid row survives, each invalid row is dropped, and each refusal line is
   the exact named prefix plus the escaped value and no install path. That is the seam the
   repository can reach; the subprocess tests keep the end-to-end half they already pin.

2. **[minor]** `crates/discovery/src/steam_common.rs:325` — Test strategy #4's last clause ("a name
   with no in-bound candidate is refused **and counted**") is unasserted, and as written it is not
   assertable through the public API. `collision_tests.rs:159`
   (`a_name_with_no_in_bound_candidate_is_refused_rather_than_pushed_past_the_bound`) pins the
   *removal* and keeps a third in-bound row alive so the run still succeeds — which is precisely
   the configuration that hides the counter: `skipped` is only ever printed by the diagnostic at
   `steam_common.rs:107`, guarded by `games.is_empty()`, and a collision refusal can never empty
   `games` (the byte-order-first name for any base always keeps its in-bound base, since
   `discovered_game` already enforced the bound). So `*skipped += before - games.len()` has no
   observable consequence and no test; `resolve_collisions` is private, so nothing reaches it
   either. **Fix:** either make the delta observable and pinned — `#[cfg(test)]`-reachable
   `resolve_collisions` (or a thin `pub(crate)` wrapper) with a unit test asserting `skipped` grows
   by exactly the refused count over a hand-built `Vec<DiscoveredGame>` — or, if the count is
   deliberately unobservable, say so where the claim lives: cite it in `03-build-notes.md` and
   soften the test-suite's implied coverage in `collision_tests.rs:14-16` so the record does not
   read as if the counting half were pinned at the emission seam.

3. **[minor]** `apps/discovery-cli/tests/cli_slug_refusal_tests.rs:25-33` — the suite retypes the
   bound as `const SLUG_BOUND: usize = 128` on a stated premise this diff invalidated. The comment
   reads "this crate's tests deliberately keep no `bindforge-core` dependency, so the bound is
   stated below as the number the README bullet states", but `apps/discovery-cli/Cargo.toml` gains
   `bindforge-core = { workspace = true }` in this very change, and the same pattern already works
   in `crates/discovery/tests/` (which reads `bindforge_core::MAX_SLUG_OCTETS` from `[dependencies]`
   with no dev-dependency entry). `024-B1`'s rule is that the bound is "referenced rather than
   retyped"; a test that hard-codes 128 in four places (`113-122`, `139-144`, `182-183`) is the
   second home ADR-024 exists to prevent, and its rationale comment is now false. **Fix:** replace
   `SLUG_BOUND` with `bindforge_core::MAX_SLUG_OCTETS` (a bin-only package's test target sees the
   package's dependencies, as `crates/discovery/tests/` demonstrates; if it does not, add it under
   `[dev-dependencies]` — either way, delete the stale premise from the comment). The independent
   `is_slug_shaped` oracle can stay: a black-box grammar oracle is a different thing from a retyped
   budget constant.

## Not applicable / not a finding

- Checklist 3 (revision rounds) — N/A, first pass.
- Missing `crates/core/tests/zz_api_probe.rs` update: FYI only, no severity. ADR-024's core-side
  exports are resolved in test code elsewhere (`crates/discovery/tests/zz_api_probe.rs:101` at the
  `bindforge-core` root; `core/tests/placement.rs:825` for the value; `core/tests/error.rs` for the
  renderer), and Test strategy #12 scopes the new probe to `crates/discovery`, so no checklist item
  or proposal line requires the core probe to grow. A one-line addition there would keep that
  file's own "every name the crate promises" habit whole.
- Fold-table correctness (e.g. `İ` → `i`, absent `ł`/`đ`) was checked only as far as the tests'
  expectations match the proposal's stated coverage boundary; all 40-plus derivation expectations
  recompute correctly against the table. Grammar/spec-seam questions belong to other seats.
- No wall-clock, network, or port dependence found; no test weakened without replacement found —
  every removed assertion in `cli_tests.rs` / `cli_output_polish_tests.rs` / `manifest.rs` has a
  stronger prefix-free or bound-relative replacement.

Verdict: request-changes — one open major (finding 1).
