# correctness review r2 (pass c0, full panel, whole document)

verdict: request-changes

Scope: whole of `docs/designs/tasks/001-steam-linux-discovery/02-proposal.md`.
Inputs used: the proposal, `docs/tasks/T-001-steam-linux-discovery.md`, the proposal's
Revision log, and read-only inspection of the code, workflow and lock files the proposal
cites. `01-research.md` was not in this pass's allowed inputs and was not opened.
No earlier review report or transcript was opened.

## Findings

- [blocker][new] Assumption register A9 + Chosen design §2 Root selection — "`home` comes
  from `std::env::home_dir()` (A9)" / A9 row: "`std::env::home_dir()` yields the home
  directory … | verified | https://doc.rust-lang.org/std/env/fn.home_dir.html".
  That call is `#[deprecated]` on the toolchain this repo pins, so the design as written
  cannot pass the gate it cites for itself. `rust-toolchain.toml:7` pins
  `channel = "1.85.0"` and `Cargo.toml:17` sets `rust-version = "1.85"`; the versioned std
  docs for that release (https://doc.rust-lang.org/1.85.1/std/env/fn.home_dir.html) carry
  the banner "👎 Deprecated since 1.29.0: This function's behavior may be unexpected on
  Windows. Consider using a crate from crates.io instead.", and Rust 1.85.0's own
  compatibility note is "`std::env::home_dir` is now deprecated … Consider using the
  `home_dir` function from crates.io/crates/`dirs` instead" (it was undeprecated only in
  1.87.0, https://github.com/rust-lang/libs-team/issues/372). The `deprecated` lint is
  warn-by-default — it is absent from the rustc deny-by-default listing — so every call
  site warns, and `.github/workflows/check.yml:31` runs
  `cargo clippy --workspace --all-targets -- -D warnings`, the exact command Test strategy
  #10 asserts will pass; there is no `[lints]` table in the workspace `Cargo.toml` to
  soften it. The 1.85 note's recommended replacement is precisely what §10 forbids
  ("`crates/discovery/Cargo.toml` and `apps/discovery-cli/Cargo.toml` gain nothing, so
  `Cargo.lock` stays byte-identical"), so an implementer following §2 hits a hard gate
  failure with no in-design exit. A9 is marked `verified` on a live-version docs page
  (std 1.100) that no longer shows the banner — the cited basis does not support the row
  for the pinned floor. Fix: in §2 name the source of `home` as
  `std::env::var_os("HOME")` with an empty value treated as absent, replacing
  `std::env::home_dir()` (the plugin is Linux-only, so the Windows caveat that caused the
  deprecation does not apply), and re-cite A9 to the 1.85.1 docs page instead of the
  live-version URL.

- [major][new] §3 Library selection within a root + §2 Canonical path rule — "`<root>/steamapps`
  itself, always" against "dropping entries that fail to canonicalize or that lack the
  required `steamapps` directory". Everywhere else the design means by *library* a directory
  that *contains* `steamapps/`: §3 reads manifests "only as entries directly inside a
  selected library's `steamapps/`", §5 composes
  `install_path = l.join("steamapps").join("common").join(m.install_dir)` — A2's
  `<library>/steamapps/common/<installdir>` — and Risks row 3 says "Library paths are
  canonicalized and must contain a `steamapps` directory". The cited basis agrees: the
  A2/A3 source (steamlocate-rs `src/library.rs`) asserts `path.join("steamapps").is_dir()`
  for a library and reads `path/steamapps/appmanifest_<id>.acf`. So `<root>/steamapps` can
  never satisfy the §2 gate — its `steamapps` child would be `<root>/steamapps/steamapps` —
  and the always-included entry is dropped: a root whose `libraryfolders.vdf` parses to zero
  libraries yields zero games, the silent-zero bug §3 says this inclusion exists to prevent,
  while Test strategy #3 ("a root whose `libraryfolders.vdf` is absent still yields its own
  `steamapps`") and #8's four-game fixture expectation both fail against §2 as literally
  written. Under the other reading — taking `<root>/steamapps` as the library — §5 emits
  `<root>/steamapps/steamapps/common/…`, contradicting A2 and the fixture tree, and the
  fixture yields one game, not four. Fix: state once that a library is a directory
  containing a `steamapps/`, and make §3's always-included entry `<root>` itself, naming
  `<root>/steamapps` as the directory scanned inside it; keep the §2 gate as the library
  check and reword #3 to "still yields the root".

- [major][new] §9 Fixtures + §2 Canonical path rule — "the `\"0\" → \".\"` entry exercises
  root/library de-duping" against §2's "the `\"0\" → root` duplicate that
  libraryfolders.vdf normally contains". §3 joins a relative `path` "to the directory
  holding the VDF", which is `<root>/steamapps`, so `"."` resolves to `<root>/steamapps`
  and not to `<root>`: the fixture's `"0"` entry cannot collide with, and so cannot
  exercise de-duping against, the root candidate — and per the finding above it is dropped
  instead. Nothing in Test strategy #1–#10 then exercises the de-dup step §2 leans on for
  the `~/.steam/root` symlink alias. Fix: set the fixture VDF's `"0"` value to `".."` so it
  canonicalizes to `<root>` and de-dupes against the always-included root, leaving §9's
  comment as a claim the fixture actually tests.

- [major][new] §7 Failure policy + Test strategy #7 — "at least one root was selected but
  **no** library under any selected root could be scanned" against "a root that has a
  `steamapps` directory whose entries are all unparseable returns exactly one
  `Err(DiscoveryError::PluginError(_))`". In #7's scenario the library *was* scanned: its
  `steamapps/` was enumerated and each manifest was skipped under §7's own skip list ("a
  manifest that fails to parse … skipped and the scan continues"). The stated trigger
  therefore never fires and `discover()` returns `Ok(vec![])` where #7 demands `Err`, so
  the "degraded scan prints nothing" case that Risks row 4 says "#7 pins both sides of" is
  left unguarded. Fix: define the single `Err` trigger as "at least one root was selected,
  no game was emitted, and at least one candidate manifest was skipped as unreadable or
  malformed" — that fires for #7 and still leaves a genuinely empty install (no manifests,
  no skips) at `Ok(vec![])` per §7's first paragraph and the Compatibility matrix's "Steam
  not installed" row.

- [minor][new] §2 Root selection — "`from_roots()` skips `host_candidates` and the existence
  probe, taking caller-supplied roots through the canonical path rule". The canonical path
  rule is itself an existence probe (it drops entries that fail to canonicalize and entries
  "that lack the required `steamapps` directory"), so the sentence contradicts itself about
  whether an explicit `--steam-root` that exists but has no `steamapps` child — what a user
  typing `--steam-root ~/.steam` supplies — is scanned or silently dropped. Fix: say
  `from_roots()` skips only `host_candidates`, still applies the full canonical path rule
  including the `steamapps` gate, and state what `discovery list` prints for a `--steam-root`
  the gate drops.

- [minor][new] §1 Modules and public surface — "Every `pub` item above is exercised by a
  test in *Test strategy*". `SteamLinuxPlugin::from_host()` appears in no test: #5
  deliberately tests the pure `host_candidates` instead, and Risks row 5 states the
  opposite of §1 — "`from_host()` cannot be unit-tested". Fix: qualify the sentence —
  "every `pub` item except `from_host()`, whose only coverage is the manual Done-when check
  in *Rollout* #5" — so the coverage claim matches §Test strategy and Risks.

- [minor][new] §2 candidate 1 + A9 — "default `$HOME/.local/share/Steam`" with A9's "(on
  Unix `$HOME`, else the passwd entry)". On the pinned 1.85.0 the versioned docs state the
  function "Returns the value of the 'HOME' environment variable if it is set (including to
  an empty string)" — the empty-value fallback is a later release — so `HOME=""` yields
  `Some("")`, every candidate becomes a relative path, and §2's canonicalization resolves
  them against the process working directory: a stray `steamapps/` in the current directory
  would be selected as a Steam root. Fix: state in §2 that an empty `$HOME` is treated as
  absent (same root cause, and same fix, as the A9 finding above).

- [nit][new] §9 Fixtures — "the CLI's through `env!(\"CARGO_MANIFEST_DIR\")` (A19)". From
  `apps/discovery-cli` that variable expands to the CLI package root, so reaching the crate's
  tree needs an explicit `../../crates/discovery/tests/fixtures/steam/minimal` hop; naming
  it would pin the cross-package path Test strategy #9 depends on. Fix: state the relative
  hop in §9.

## Verification performed

- Seam assumptions re-checked first-hand and accurate: `plugin.rs:4-7` (two-method trait,
  `discover(&self)` takes no arguments), `game.rs:20-27` (five fields), `Platform::SteamLinux`
  at `game.rs:7`, `slug.rs:1-35` (`unknown/` + kebab), `lib.rs:1-5` (exactly five modules),
  `engine.rs:20-33` (an `Err` becomes one `eprintln!` and `discover_all` still returns `Ok`)
  for A14, `registry.rs:1-5` (`DiscoveryError::PluginError(String)`, no `Display`), and
  `DummyPlugin` exists only at `apps/discovery-cli/src/main.rs:25-33` with its single use at
  `:41` — A17 holds, though the cited range `25-41` covers definition plus use rather than
  proving uniqueness, which the repo-wide search supplies.
- A15 holds: `Cargo.lock` has exactly 32 `[[package]]` entries and none is a KeyValues
  parser, `dirs`, `walkdir` or `tempfile`; `crates/discovery/Cargo.toml:6-7` carries only
  `serde`. `apps/discovery-cli` has no `[[bin]]` section, so `CARGO_BIN_EXE_discovery-cli`
  (A19, Test #9) names the right bin target. A18 holds: `check.yml:16-33`, `ubuntu-latest`,
  fmt, clippy `-D warnings`, `cargo test --workspace --locked`.
- External bases spot-checked (five): `leovp/steamfiles` `acf_overview.rst` supports A1
  (`AppState` root section carrying `name` and `installdir`); steamlocate-rs
  `src/library.rs` supports A2 and A3 (numeric-key filter, `path` per block,
  `path/steamapps/appmanifest_*.acf`, `path/steamapps/common/<installdir>`) and is what
  contradicts §3's library wording as described above; `src/locate/linux.rs` supports A8
  (same Flatpak/xdg/`.steam`/Snap candidates, same `steamapps`.is_dir gate);
  `steamie-core::vdf` supports A5 ("Key lookups are case-insensitive (Steam mixes `installdir`
  and `StateFlags`)"). A9's basis fails as described.
- ADR-022 citations check out: `:21` (no Steam Deck), `:49` (no AppID usage), `:55` (slug
  derived from display name), `:76` (v1 slugs provisional) match how the design and its
  Deferred section use them. `docs/tasks/T-001-steam-linux-discovery.md:15-25` is decided by
  §1–§8 and Test strategy #1–#10; the Deferred items are consistent with that Decision, so
  none is re-opened.
- Fixture arithmetic, once the library level in finding 2 is resolved, is consistent:
  `900001`/`900002`/`900003` plus `secondary` give the four slugs Test #8 lists, in the
  `(slug, install_path)` order §5 sorts to, and `900004`/`900005` and the `compatdata`
  decoys are excluded by §4 and §3.
- Missing evidence, recorded: the `01-research.md` line references (82-88, 99-106, 120-123,
  136-140, 169-192, 281-324, 352-357, 416-420) were not opened, so A7, A16 and A20's
  reliance on them is unverified here — all three are honestly marked `unverified`, which is
  why this does not block. Nothing else required by the checklist is unestablished.

## FYI (no severity)

- The Revision log carries only `r0` entries although this is round 2's c0 pass. If round 1
  produced edits, no entry records them, and from this pass's allowed inputs I could not
  check whether any earlier round's findings were closed rather than reworded (checklist
  item 7); my own last verdict carried no findings, so nothing here is labelled
  `re-raised` or `regression` on that account.
- §4 fixes the two caps as numbers while Risks row 7 calls them "single named constants";
  naming them in §4 is the complexity/performance seat's call, not a correctness finding.
