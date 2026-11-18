# compatibility review r2

verdict: request-changes

Scope: whole document (full panel c0, round 2). Inputs: 02-proposal.md,
docs/tasks/T-001-steam-linux-discovery.md, the Revision log, read-only code
inspection (crates/discovery/src/*, apps/discovery-cli/*, Cargo.lock,
rust-toolchain.toml, .github/workflows/check.yml, docs/adrs/022-game-discovery.md)
and the versioned std docs for the pinned toolchain.

## Findings

- [major][new] Assumption register A9 / Chosen design §2 "Root selection" —
  "`home` comes from `std::env::home_dir()` (A9)" — on this repo's pinned
  toolchain (`rust-toolchain.toml`: `channel = "1.85.0"`),
  `std::env::home_dir()` carries `#[deprecated(since = "1.29.0", note = "This
  function's behavior may be unexpected on Windows. Consider using a crate from
  crates.io instead.")]`. Confirmed on the versioned docs page for exactly the
  pinned floor (doc.rust-lang.org/1.85.0/std/env/fn.home_dir.html shows the
  "Deprecated since 1.29.0" badge) and on the current latest-stable page, where
  the badge is *absent* — the undeprecation shipped only after 1.85.0
  (rust-lang/rust#132515 "Fix and undeprecate home_dir()"; the 1.85.0 release
  notes state "a subsequent release will remove the deprecation for this
  function"; the attribute removal, rust-lang/rust#137327, merged 2025-03-07,
  after 1.85.0's 2025-02-20 release). A call site therefore emits a warn-level
  `deprecated` lint, and `cargo clippy --workspace --all-targets -- -D warnings`
  (.github/workflows/check.yml:31 — the design's own gate, asserted by A18,
  Rollout step 3 and Test strategy #10) turns that warning into an error: as
  written, `from_host()` fails the design's stated CI gate on the pinned floor.
  A9's "verified" basis is additionally anchored to the *unversioned*
  doc.rust-lang.org URL, which tracks latest stable and no longer shows the
  deprecation — the citation supports the current toolchain, not the pinned one.
  Violated requirement: the design's own Test strategy #10 / A18 (gates pass on
  the pinned 1.85 toolchain). Fix: in `from_host()`, read
  `std::env::var_os("HOME")` directly — A9's own Unix basis is "$HOME, else the
  passwd entry", and the only behavior lost (the getpwuid fallback when HOME is
  unset) already lands in §2's existing `None`-handling: empty candidate list →
  `Ok(vec![])`, which the Compatibility matrix row "Steam not installed / no
  home directory | Yes" already sanctions. This also honors A15's no-new-deps
  posture, since the deprecation note's own advice ("use a crate from
  crates.io") is barred there (`dirs` is inside A15's absent-deps claim).
  Re-anchor A9 to the versioned URL (doc.rust-lang.org/1.85.0/...) or re-status
  it. Alternative one-line fix: keep the call and add `#[allow(deprecated)]` at
  the single call site with a comment citing the post-1.85 undeprecation —
  smaller diff, but pins a deprecated API until the toolchain bump.

- [minor][new] Compatibility matrix vs §3 vs Test strategy — matrix row
  "Linux, libraries on arbitrary mounts (`/run/media/…`, `/mnt/…`, ZFS) | Yes |
  Absolute `path` values are accepted as-is" — the absolute branch of §3's join
  rule ("a `path` is used as-is when absolute and otherwise joined to the
  directory holding the VDF") is exercised by no named test: §9's fixture
  deliberately carries only `"0" → "."` and `"1" → "../secondary"` ("a
  checked-in fixture cannot contain this machine's absolute paths"), and Test
  strategy #3/#6 assert only the relative-join outcomes, so the sole basis for
  a Yes-claimed environment ships untested. Checklist item 7 (a platform test
  for every supported row) is not met by this row. Fix: add a pure-text unit
  test for a pre-canonicalization "VDF section → library paths" step of
  `libraries_in_root` (text in, `Vec<PathBuf>` out) asserting an absolute value
  passes through unchanged and a relative value is joined to the VDF's
  directory; the text-in seam needs no filesystem write, so §9's no-write rule
  and #3 are untouched.

## Checklist coverage

1. Matrix status per row — pass: every row carries Yes/Partial/No; no
   `unknown`, no empty cell.
2. OS input APIs (X11/Wayland/Raw Input/HID) — not applicable: no input
   surface; the design's OS interaction is filesystem and environment only.
3. Node version floor — not applicable: no Node code touched
   (`tools/orchestration` untouched).
4. Native modules / dependency posture — pass: no new crates; verified the
   A15 claim against the tree (Cargo.lock holds 32 `[[package]]` entries;
   crates/discovery/Cargo.toml deps are `serde` only). The one std-API
   floor-compatibility failure is Finding 1.
5. Keyboard reality (layouts, dead keys, IME, non-ASCII names) — no keyboard
   surface; non-ASCII display names survive `normalize_slug` (slug.rs:6-12 uses
   Unicode-aware `is_alphanumeric`), and the empty-name case is guarded by §5's
   `installdir` fallback. No finding.
6. File paths — pass: `PathBuf::join` and `canonicalize`, not string
   concatenation; XDG / Flatpak / Snap per-OS conventions per basedir-spec
   (relative `XDG_DATA_HOME` ignored, A9); `installdir` rejected on
   `.`/`..`/absolute/`/`; A17 seam claims all verified against
   crates/discovery/src/{engine,plugin,game,slug,lib}.rs.
7. Test strategy names a platform test per supported matrix row — violated for
   the arbitrary-mounts row (Finding 2); the other Yes rows are covered
   (#5 candidates/XDG/Snap, #3/#8/#9 fixture scans, #7 no-Steam, #10 CI).

## FYI (no severity)

- apps/discovery-cli/tests/cli_tests.rs currently shells out via
  `Command::new("cargo").args(["run", "-p", "discovery-cli", ...])`, so the
  CLI tests reenter cargo and resolve the toolchain at test runtime; Test
  strategy #9's switch to `env!("CARGO_BIN_EXE_discovery-cli")` fixes that too,
  which the proposal does not claim as a benefit — belongs to another seat's
  account, noting only.
- The pure `host_candidates(home, env: &dyn Fn(&str) -> Option<String>)` seam
  also keeps tests off `std::env::set_var`, which is `unsafe` on this crate's
  edition 2024 (crates/discovery/Cargo.toml:4) — worth knowing when the
  correctness seat audits the test list.

## Revision log check

All fifteen entries are r0 decisions; none rejected or closed a
`home_dir`-deprecation or untested-absolute-path concern, so neither finding
above re-opens a rebutted decision. A20's rejection (legacy `SteamApps`
spelling, kept as an assumption) is respected: no finding re-opens it, and the
Deferred list was checked against T-001's Decision and Done-when clauses —
nothing deferred there contradicts the task (filtering, `StateFlags`, slug
migration, Windows, Deck all trace to ADR-022 scope lines the proposal cites
accurately: 022-game-discovery.md:21, :49, :55, :76 all verified against the
file).
