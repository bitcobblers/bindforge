# complexity review r1 (pass c0)

verdict: approve

Scope: whole document (`docs/designs/tasks/001-steam-linux-discovery/02-proposal.md`).
Inputs: the proposal, `docs/tasks/T-001-steam-linux-discovery.md`, the Revision
log, and read-only repo checks to verify the design's claims about the code.

## Checklist walk

1. **Reuse.** Verified: no KeyValues/VDF/Steam parsing exists anywhere in
   `crates/` (grep over `*.rs`/`*.toml`/lock finds only the `Platform`
   variants); `tools/` is a pure-TypeScript orchestration package with nothing
   reusable from Rust. The design reuses the existing seam unchanged —
   `DiscoveryPlugin` (`crates/discovery/src/plugin.rs:4-7`), `DiscoveryEngine`
   (`engine.rs:20-33`, `Err` → single `eprintln!` + `Ok` confirmed verbatim),
   `normalize_slug` (`slug.rs:1-35`, `unknown/` prefix confirmed), the 5-field
   `DiscoveredGame` (`game.rs:20-27`), and `Platform::SteamLinux`.
2. **Trace to Goals.** Every moving part traces: the reader subset, root
   candidate list, always-scan of the root's own `steamapps`, the `installdir`
   rejects, the `(slug, install_path)` sort, and the `--steam-root` seam each
   map to a Goal or a recorded r0 Revision-log decision (q1, q8/q14, q12,
   q15). Flatpak/Snap probing is the r0 q1 decision with a sound stated basis;
   not re-litigated.
3. **30–50% smaller.** The realistic cut is dropping the Flatpak/Snap
   candidate groups (~7 literal paths plus the `$SNAP_USER_DATA` branch) and
   deferring them like Deck. That trims little code, and the r0 q1 rebuttal
   (coexisting layouts; the `steamapps` gate drops sandbox-only paths cheaply;
   the matrix records the partial result) is not unsound. No major.
4. **Dependencies.** Zero new; verified `Cargo.lock` holds exactly 32
   `[[package]]` entries with no parser/`dirs`/`walkdir`/`tempfile`, and both
   `Cargo.toml`s gain nothing — the "byte-identical lock" claim is checkable
   and currently true.
5. **Defer the second half.** Done well: filtering, `StateFlags`, error
   taxonomy, legacy shapes, benchmark harness, workspace-version churn are all
   deferred with named owners; the v1 slice is thin.
6. **Knobs/abstractions without consumers.** `--steam-root`, `from_roots`,
   `host_candidates`' env closure and both caps all have named consumers
   (tests, CLI arm, bounded-work risk). One justification sentence is false —
   finding below.
7. **Value check.** Proportionate. A first real probe for the product, one
   hand-rolled reader whose dependency alternative was rejected on verified
   lock-posture and lossy-representation grounds, and a public surface held to
   two small modules. The design's complexity is mostly researched
   constraints made explicit, not speculative machinery.

## Findings

- [minor][new] §1 Modules and public surface — "Every `pub` item above is
  exercised by a test in *Test strategy*" — `SteamLinuxPlugin::from_host()` is
  in "above" but no Test-strategy item exercises it: #9 and #10 explicitly
  keep every test off the host and the developer's home directory, and no
  other item calls it. The sentence is the stated justification for the public
  surface, and it is false as written for the one item that exists only to be
  wired into the CLI. Fix: qualify it — "Every `pub` item above except
  `from_host()` is exercised by a test; `from_host()` is pure composition
  (§2) whose end-to-end behavior is Rollout #5's manual check" — replacing the
  false claim rather than adding a host-probing test.

## FYI (no severity)

- Test strategy #9's "both existing tests" checks out: `apps/discovery-cli/tests/cli_tests.rs`
  holds exactly two tests; they currently shell out via `cargo run -p
  discovery-cli`, which the `CARGO_BIN_EXE_discovery-cli` switch replaces
  (A19 verified in the Cargo reference claim).
- Items #1–#7 are labeled "Unit" but land in `crates/discovery/tests/*_tests.rs`,
  i.e. cargo integration tests — consistent with the existing
  `slug_tests.rs`/`plugin_tests.rs` layout, so a terminology quirk only;
  terminology and Done-when mapping belong to another seat.
- The four expected slugs in #8 cross-check cleanly against
  `crates/discovery/src/slug.rs` (`unknown/example-game`,
  `unknown/mixed-case-game`, `unknown/nameless-install`,
  `unknown/secondary-title`); the parsing that produces those display names is
  for the correctness seat to verify.
- `docs/adrs/022-game-discovery.md` anchors cited by the proposal (:21 v1
  scope/Deck exclusion, :49 no-AppID, :55 slug derivation) all read as quoted.
