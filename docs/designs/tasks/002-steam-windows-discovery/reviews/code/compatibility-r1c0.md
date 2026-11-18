# Code review — compatibility seat, round 1 pass c0

- Change under review: `git diff d4242107d82c9cf2ef3750615158165b4260f397...HEAD -- ':(exclude)docs/designs'`
- Design: `docs/designs/tasks/002-steam-windows-discovery/02-proposal.md`; task: `docs/tasks/T-002-steam-windows-discovery.md`
- Inputs: the diff, the proposal, the task, the runner's gate record (green, 2026-11-18T06:26:23Z), and my own read-only re-runs. No other review reports consulted.

## Scope inspected

All 11 paths in the diff: `.github/workflows/check.yml`, `Cargo.lock`, `Cargo.toml`,
`apps/discovery-cli/src/main.rs`, `apps/discovery-cli/tests/cli_dual_plugin_tests.rs`,
`crates/discovery/Cargo.toml`, `crates/discovery/src/{lib,steam_common,steam_linux,steam_windows}.rs`,
`crates/discovery/tests/steam_windows_tests.rs`. No fixture trees are touched (absent
from the stat), which Test strategy #5 requires.

## Checklist walk-through

1. **Platform code matches the Compatibility matrix.** PASS. Every "Yes" row has a
   code path: registry candidates HKCU→HKLM-32→HKLM-64 (`steam_windows.rs:151-159`,
   view flags per variant in `registry_string_value`, `steam_windows.rs:205-224`);
   `%ProgramFiles(x86)%`/`%ProgramFiles%` defaults appended with `absolute_env`
   (`steam_windows.rs:165-169`, `steam_common.rs:332-336`); non-`REG_SZ` refusals
   (`steam_windows.rs:229-232`); failed reads are "no candidate" (every `?`/`.ok()?`
   path returns `None`, `steam_windows.rs:197-249`); not-installed is
   `Ok(vec![])` (empty-roots rule, `steam_common.rs:85,101`); Linux hosts get a
   zero-root Windows plugin via the `#[cfg(not(windows))] from_host()` arm
   (`steam_windows.rs:96-99`); the `\` refusal is target-parameterized so the Linux
   rule set is unchanged (`steam_common.rs:266-274`, `steam_linux.rs` `LinuxTarget`
   answering `false`). The `#[cfg(windows)]` surface is exactly the proposal's §3/§8
   shape: only `registry_string_value`, `from_host()` and the CLI roots branch are
   gated; plugin, `WindowsTarget`, `host_candidates`, `RegistryKey` and the rule
   wrappers compile and run on every host — verified by my own Linux test run below.
2. **Node floor.** N/A — no Node/TypeScript path in the diff.
3. **Paths built with joins; case assumptions.** PASS. `dir.join("Steam")`,
   `root.join("steamapps").join("libraryfolders.vdf")`,
   `library.join("steamapps").join("common").join(&install_dir)` — no string
   concatenation constructs a path. Registry subkey spellings are fixed constants
   (`Software\Valve\Steam` / `SOFTWARE\Valve\Steam`), which is safe because registry
   keys are case-insensitive; the two pinned env spellings are queried exactly once
   each in order, asserted by `host_candidates_orders_registry_candidates_then_env_defaults`
   (A21 pinned). Windows case-insensitivity of registry values is handled by the
   canonical-path rule (`steam_common.rs:316-328`), per §5's stated no-normalization-here rule.
4. **Guarded platform imports.** PASS. `winreg` appears in exactly one source file,
   inside `#[cfg(windows)] fn registry_string_value` (grep over `crates/`: only
   `steam_windows.rs:198-199`), and is declared only under
   `[target.'cfg(windows)'.dependencies]` (`crates/discovery/Cargo.toml:8-13`). The
   green Linux `cargo test --workspace --locked` proves winreg is not even resolved
   into the Linux build.
5. **Locale/layout reality.** PASS. `keyvalues` and `normalize_slug` are untouched,
   so non-ASCII names and slugs behave as they already do on Linux. The Windows-only
   addition reads `REG_SZ` as UTF-16LE and decodes with `String::from_utf16`
   (`steam_windows.rs:236-248`), so a non-ASCII install path from the registry
   survives to `PathBuf::from`; odd length, unpaired surrogates and every non-`REG_SZ`
   type map to `None` as §4 prescribes. No new ID or format spelling is introduced.
6. **Native dependencies.** PASS. One new dependency, `winreg 0.56` (MIT, pure-Rust
   FFI through `windows-sys`), target-gated; the `Cargo.lock` diff is exactly the
   `winreg` + `cfg-if` additions A10 predicted (`windows-sys 0.61.2`, inside winreg's
   `>=0.59,<=0.61` window, and `windows-link` were already locked and are not
   duplicated). Packaging: no Windows binaries ship from this repo (explicit
   non-goal), so there is no packaging story to break; the CI matrix gains the
   cross-`cargo check` step (§9), now the first step of the `rust` job
   (`check.yml:29-32`).
7. **Tests run on the platforms they claim.** PASS (one evidence caveat, below).
   `crates/discovery/tests/steam_windows_tests.rs` (26 tests) and
   `apps/discovery-cli/tests/cli_dual_plugin_tests.rs` (2 tests) are plain
   host-platform tests — none is `#[cfg(windows)]`-skipped — and all ran green on
   this Linux host. The Windows-only reader is claimed only to be *compile*-verified
   (#12), which is the CI step's job, not claimed to run anywhere in this diff.
   Regression fixed point: `steam_linux_tests.rs` 23 passed, `cli_tests.rs` 3
   passed, both files untouched by the diff.

## Evidence

- Runner gate record (green): `npm run check`, `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace --locked` — all exit 0.
- My own read-only re-run of `cargo test --workspace --locked` (Linux, this
  worktree): all legs ok, including
  `steam_windows_tests ... test result: ok. 26 passed; 0 failed`,
  `cli_dual_plugin_tests ... ok. 2 passed`,
  `steam_linux_tests ... ok. 23 passed`, `cli_tests ... ok. 3 passed`.
- winreg 0.56.0 API audit against docs.rs (`RegKey::predef`,
  `open_subkey`/`open_subkey_with_flags(_, REG_SAM_FLAGS)` with
  `KEY_READ|KEY_WOW64_32KEY`/`KEY_WOW64_64KEY` constants,
  `get_raw_value -> RegValue { vtype, bytes }`, `RegType::REG_SZ`,
  `HKEY_CURRENT_USER`/`HKEY_LOCAL_MACHINE` in `winreg::enums`): every call in
  `registry_string_value` matches the documented 0.56.0 surface.
- **Missing evidence (recorded, see nit):** the Windows-target
  `cargo check --target x86_64-pc-windows-msvc` leg could not be established from
  the review environment — the target's `std` is not installed here and `rustup`
  is outside this seat's shell allow — and the runner's gate record contains no
  such leg. The property is enforced pre-merge by the very CI step the diff adds
  (`check.yml:29-32`), and my API audit above found no plausible type error, so
  this does not block; it is recorded so the run attaches the leg's tail
  (e.g. in `03-build-notes.md`) once CI produces it.

## Findings

- **nit** — `.github/workflows/check.yml:29-32`: the Windows cross-`cargo check`
  that compile-verifies the `#[cfg(windows)]` code (proposal Test strategy #12,
  Done-when clause) exists in CI but has no captured green output in this run's
  records yet, and no local leg ran it. Fix: attach the step's output (or the
  CI run link) to the run record when it lands, as Test strategy #12's evidence.
  No code change requested.

## FYI (no severity, out of specialty / proposal-accepted)

- §8's host-probe branch registers `SteamLinuxPlugin` on every host, so a Windows
  host that defines `HOME` pointing at a canonicalizable tree with `steamapps/`
  could emit `steam-linux`-tagged rows there. This is the accepted §8 design and
  degrades to zero roots on ordinary Windows hosts (`HOME` unset →
  `steam_linux.rs:61-64` yields no candidates); noted for the record only.

## Verdict

approve — zero blocker and zero major findings; one nit (evidence attachment).
