# compatibility review r1

verdict: request-changes

Scope: whole document (c0, round 1). Inputs: 02-proposal.md,
docs/tasks/T-002-steam-windows-discovery.md, the Revision log. Claims the
proposal makes about existing code were verified read-only against
`crates/discovery/src/{lib,game,steam_linux,keyvalues}.rs`,
`apps/discovery-cli/src/main.rs`, `apps/discovery-cli/tests/cli_tests.rs`,
`crates/discovery/tests/steam_linux_tests.rs`, `Cargo.toml`, `Cargo.lock`,
`rust-toolchain.toml`, `.github/workflows/check.yml`, and the winreg 0.56.0
dependency list on crates.io. A2 (unescape of `\\` and `\"` at
keyvalues.rs:186-207), A13 (1.85.0, edition 2024), A15 (CLI test imports, the
"exactly the four fixture rows" count at cli_tests.rs:68-72 and the
`"SteamLinux"` assertion at :124), A16 (game.rs:5-18), and A10/§9 (lock has
`windows-sys 0.61.2` + `windows-link 0.2.1`, no `cfg-if`, no `winreg`; winreg
0.56.0 depends on `cfg-if ^1` and `windows-sys >=0.59, <=0.61`, so 0.61.2 is
reused and the lock diff is exactly `winreg` + `cfg-if`) all check out.

## Findings

- [major][new] Compatibility matrix — "Windows, 32-bit (x86) host | unknown"
  and "Windows, ARM64 host | unknown" — two matrix rows carry `unknown`
  status. Checklist rule 1 admits `unknown` only as a finding: the matrix must
  carry a decision per row, and the uncertainties behind these two cells
  (A20's 32-bit view-flag behavior) are already carried in *Risks*, so nothing
  is lost by committing to a status. Both hosts plausibly work through the
  non-redirected HKCU read (A5) and degrade safely otherwise, so the honest
  answer is a status with a caveat, not a shrug. Fix: replace both `unknown`
  cells with `Partial` — 32-bit: "HKCU candidate and scan work (A5); the
  HKLM 64-bit-view query (A20) and the absent `%ProgramFiles(x86)%` fallback
  are unexercised" — ARM64: "same code path as x64 (A4's model); not
  exercised in this run" — and leave the verification caveat in *Risks*
  rather than in the matrix status.

- [major][new] §8 CLI wiring — "engine.register_plugin(Box::new(SteamLinuxPlugin::from_roots(steam_root.iter().cloned())));"
  (with "`--steam-root` keeps its existing meaning for both plugins' scanning
  code") — on a Windows host, `discovery list --steam-root <root>` registers
  only `SteamLinuxPlugin`: the Windows plugin is not registered at all on that
  branch, so explicit roots can never emit `steam-windows`-tagged rows from
  any CLI path, and real Windows Steam roots get rows tagged `SteamLinux`.
  This contradicts the task's Done-when clause "The `steam-windows` plugin is
  wired into `apps/discovery-cli` (`discovery list`) alongside `steam-linux`,
  and `--steam-root` keeps working for it"
  (docs/tasks/T-002-steam-windows-discovery.md:65-67), and the *Deferred*
  entry ("An explicit 'these roots are Windows Steam roots' CLI seam ... no
  follow-up yet") hands off a requirement the source document states outright,
  so it is not a permissible deferral. The Options-considered rejection is
  sound only against the reading that both plugins must share `--steam-root`
  on every host (which would double cli_tests.rs:68-72's four-row assertion);
  it does not prove the Windows-host case impossible. Fix: make the
  `--steam-root` branch target-aware — one `#[cfg(windows)]` that passes
  `steam_root` to `SteamWindowsPlugin::from_roots` on Windows and keeps
  `SteamLinuxPlugin::from_roots` on Linux — which leaves the Linux binary's
  behavior and all three CLI tests byte-identical while making the flag work
  for the Windows plugin on the host that can produce Windows rows; delete §8's
  "it stays host-agnostic — no `#[cfg]` in `main.rs`" sentence accordingly.

- [nit][new] Compatibility matrix / Test strategy #12 — "Cross-compile to
  `x86_64-pc-windows-gnu` | Yes" vs "the same with `x86_64-pc-windows-gnu` is
  a local alternative" — the row claims support but the test strategy makes
  the check optional and CI never runs it, so no named gate answers the row.
  Fix: mark that row `Partial (local only)` or fold running the `gnu` check
  once into *Rollout* #4's local gate list.

## Checklist dispositions

1. Matrix statuses: two `unknown` cells → finding 1; no empty cells.
2. OS input APIs (X11/Wayland/Raw Input/HID): not applicable — discovery
   reads registry/filesystem, no input path involved.
3. Node version floor: not applicable — no `tools/orchestration` change.
4. Native modules: satisfied in the Rust sense — `winreg` is target-gated,
   lock-file diff bounded and verified, CI compile gate named (§9, #12).
5. Keyboard reality (layouts/dead keys/IME): not applicable; non-ASCII
   registry/game names pass through UTF-16→String and the existing UTF-8 ACF
   gate with no lossy step.
6. File paths: satisfied — `PathBuf::join`, `canonicalize` and the shared
   `absolute_env` helper, no string concatenation; §5's lowercase/`\\?\`
   collapse argument holds because Windows `canonicalize` returns true case,
   so the `BTreeSet` dedupe in `canonical_paths` compares equal spellings.
7. Platform test per supported row: satisfied for Windows rows via #12/#13
   and Linux rows via #11; the one gap is the `gnu` row → nit finding 3.

## FYI (no severity)

- §1 freezes `steam_linux.rs`'s *current* pub surface, but *Test strategy* #2
  calls `steam_linux::usable_install_dir`, which is private today
  (steam_linux.rs:275); satisfying #2 from an integration test adds a new pub
  item to the Linux module. Additive, so not a compatibility break — flag it
  to the complexity seat.
- The crate already has a `registry` module (`DiscoveryError`); the new
  `steam_windows::RegistryKey` names a different thing — naming-confusion
  fodder for the complexity seat.
