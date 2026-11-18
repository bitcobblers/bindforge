# Build notes: Windows Steam discovery plugin (T-002)

## Progress

- Completed (all increments, gates run after the final edit):
  - `crates/discovery/src/steam_common.rs` — the shared scan core: `AppManifest`,
    sealed `SteamTarget` (`sealed::Sealed` supertrait), `SteamCore<T>` with
    `pub(crate) with_roots` (canonical-path rule applied there), the blanket
    `DiscoveryPlugin` impl (`name()` is `T::NAME`, error prefix is `T::NAME`),
    and the verbatim-moved helpers: `library_paths`,
    `libraries_in_root_with_skips`, `scan_library`, `read_text_file`,
    `canonical_paths`, `discovered_game`, `absolute_env`, plus
    `parse_manifest(text, target)` / `usable_install_dir(install_dir,
    reject_backslash)` generalized over `T` (§1, §2, §6).
  - `crates/discovery/src/steam_linux.rs` — rewired: `SteamLinuxPlugin` holds a
    `SteamCore<LinuxTarget>`; `pub` surface preserved exactly
    (`SteamLinuxPlugin`, `from_host`, `from_roots`, Linux `host_candidates`,
    `libraries_in_root`, `parse_manifest`, `AppManifest` re-export); added
    `pub struct LinuxTarget` (`install_dir_rejects_backslash() == false`).
  - `crates/discovery/src/steam_windows.rs` — `SteamWindowsPlugin`
    (`from_host` target-gated both ways per §3, `from_roots`,
    `from_registry_and_roots`), `WindowsTarget`, pure `host_candidates(reg, env)`
    with the §5 order (HKCU `SteamPath`, HKLM-32 `InstallPath`, HKLM-64
    `InstallPath`, then `ProgramFiles(x86)\Steam`, `ProgramFiles\Steam`),
    `RegistryKey` (three variants), Windows `parse_manifest` /
    `usable_install_dir`, and the private `#[cfg(windows)]
    registry_string_value` — the crate's only `winreg` consumer, `REG_SZ`-only
    via `get_raw_value().vtype`, every failure → `None` (§3–§6).
  - `crates/discovery/src/lib.rs` — `mod steam_common;` (private) +
    `pub mod steam_windows;`.
  - `apps/discovery-cli/src/main.rs` — §8 wiring: host-probe branch registers
    both plugins (Linux then Windows); the roots branch stays single-plugin via
    `#[cfg(not(windows))]` / `#[cfg(windows)]`.
  - `Cargo.toml` — `[workspace.dependencies] winreg = "0.56"`;
    `crates/discovery/Cargo.toml` gains
    `[target.'cfg(windows)'.dependencies] winreg = { workspace = true }`.
    `Cargo.lock` diff is exactly `winreg 0.56.0` + `cfg-if 1.0.5` (windows-sys
    0.61.2 reused, per A10).
  - `.github/workflows/check.yml` — added the §9 step
    (`rustup target add x86_64-pc-windows-msvc` +
    `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc`)
    before the three existing commands.
- Remaining work: none for the Linux gates. Three items stay open, none of them
  answerable from this runner:
  - Test strategy #13 (manual Windows run on a real install, *Rollout* #5) —
    needs a Windows machine; it is the only gate for §8's `#[cfg(windows)]`
    roots branch and for the `\\?\` prefix in `install_path` (§7).
  - Test strategy #12 / *Rollout* #4's cross-target `cargo check` — the new CI
    step (`.github/workflows/check.yml`) is the gate; a green leg has to be
    cited before the `#[cfg(windows)]` code is called compile-verified.
  - Two open code-review minors: the interior-NUL `installdir` spelling of
    Test strategy #1 is pinned by no test (correctness F1 / testing #1), and the
    relative-registry-value asymmetry (security S-1, design-conformant and
    risk-accepted in §5).
- Validation (last run, all post-change): see *Gate posture* below.
- Next concrete action: hand off to the runner; the CI cross-`cargo check`
  step is now the compile gate for the `#[cfg(windows)]` code.

## Gate posture

Commands run in the stage environment (system cargo 1.100.0; `rustup` is not
available to stage shells and the Windows targets' `std` is not installed):

- `cargo fmt --all -- --check` — exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo test --workspace --locked` — exit 0; every suite green, including the
  untouched regression gate: `steam_windows_tests` 26 passed, `steam_linux_tests`
  23 passed, `plugin_tests` 1 passed, `cli_tests` 3 passed,
  `cli_dual_plugin_tests` 2 passed, plus all `bindforge-core`/
  `bindforge-transforms` suites.
- `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` —
  exit non-zero; environment limitation, not a code failure. Tail:

      error[E0463]: can't find crate for `core`
        |
        = note: the `x86_64-pc-windows-msvc` target may not be installed
        = help: consider downloading the target with `rustup target add x86_64-pc-windows-msvc`

  `x86_64-pc-windows-gnu` fails identically. `rustup target add` is outside the
  stage shell's allowed commands, so the target cannot be installed here. As a
  substitute for the missing type-check, the `registry_string_value` calls were
  verified against the downloaded winreg-0.56.0 sources: `RegKey::predef(HKEY)`
  (reg_key.rs:56), `open_subkey` / `open_subkey_with_flags(_, REG_SAM_FLAGS)`
  (:146, :164), `get_raw_value` → `RegValue { bytes: Cow<[u8]>, vtype: RegType }`
  (:690, reg_value.rs:14-15), and the `enums` re-exports of the hive/flag
  constants and `RegType::REG_SZ` (enums.rs:9-17, :30-44). The new CI step is
  the full gate for that code.

## Deviations from the design

1. **Cross-target compile check not run locally** (Test strategy #12,
   *Rollout* #4): not possible in this environment (see *Gate posture*). The
   CI step from §9 was still added; *Rollout* #4's local
   `x86_64-pc-windows-gnu` check was likewise not run. No code change.
2. **`pub use crate::steam_common::AppManifest;` in `steam_windows` as well as
   `steam_linux`**: §1 names the re-export only for `steam_linux`, but the
   Windows `pub fn parse_manifest` returns the same type, so the re-export
   keeps it nameable on that module's surface too — the same wrapper/re-export
   pattern, additive only.
3. **`SteamWindowsPlugin` and `SteamLinuxPlugin` derive `Debug, Clone`** (and
   `SteamCore` derives `Debug, Clone`): `SteamLinuxPlugin` already derived them
   before the refactor; the derives were carried over and mirrored so no
   existing or new consumer loses capability.
4. **Windows registry value / environment-variable names are module constants**
   (`STEAM_PATH_VALUE`, `INSTALL_PATH_VALUE`, `PROGRAM_FILES_X86`,
   `PROGRAM_FILES`) per the *Risks* mitigation ("one named constant per
   variable in `host_candidates`"); the `SteamTarget` trait is `Copy +
   sealed::Sealed` exactly as §1 sketches (the sketch listed `: Copy` and the
   sealed note).
5. **`pub struct LinuxTarget` in `steam_linux`**: §1 sketches only
   `pub struct WindowsTarget`, but the core is parameterized in both modules, so
   the Linux side needs the same parameter type. Additive only — every `pub` item
   `steam_linux` had before the refactor (`SteamLinuxPlugin` with `from_host` and
   `from_roots`, `host_candidates`, `libraries_in_root`, `parse_manifest`,
   `AppManifest`) is still there with the same signature, which is what Test
   strategy #9 checks.
6. **`registry_string_value` decodes `REG_SZ` by hand** (odd byte length →
   `None`, trailing NUL units trimmed, unpaired surrogates → `None`): §4's
   `vtype == REG_SZ` gate rules out winreg's `get_value::<String>`, which also
   accepts `REG_EXPAND_SZ`/`REG_MULTI_SZ`. The refusal set stays §4's; only the
   decoder is spelled out here.
7. **Registry candidates are not absolute-path-gated** (`steam_windows.rs`,
   `host_candidates`): §5 passes only the two env candidates through
   `absolute_env`. Code review security S-1 (minor) flagged the asymmetry;
   unchanged, because §2/§5's canonicalize-and-require-`steamapps` rule is the
   design's accepted mitigation for that *Risks* row.
8. **Test strategy #10 is a new file** (`apps/discovery-cli/tests/
   cli_dual_plugin_tests.rs`) rather than an edit to `cli_tests.rs`, to keep #9's
   untouched-suite gate intact; its subprocess leg uses `env_clear()` plus a
   nonexistent `HOME` for hermeticity, and it does not assert the two plugins'
   registration order (unobservable while the Windows plugin yields no rows —
   correctness F3, nit).
9. **The pinned 1.85.0 toolchain was not used for the local legs** (see *Gate
    posture*): Test strategy #11's trio ran on the host's cargo 1.100.0 because
    the stage shell has no `rustup`; CI runs them on the pinned channel (A13).
10. **Rollout #1–#4 landed as one commit** (`cd8ecf6e`, with the new tests in the
    preceding `3ea3c08c`): the refactor was not landed alone before the plugin,
    the CLI wiring and the CI step, because the pipeline's implement stage is a
    single increment and the runner owns commits. Test strategy #9's untouched
    suites were still run against the whole change, and rollback stays a revert.

No test file was edited. No changes to `keyvalues`, `normalize_slug`,
`DiscoveredGame`, `Platform`, `DiscoveryPlugin`, `DiscoveryEngine`,
`DiscoveryError`, the CLI output shape, or any fixture.

## Code review outcome

Full-panel pass r1c0 over `git diff d4242107…HEAD` (reports under
`reviews/code/`): **five approvals, zero blocker, zero major**.

- correctness: approve — the moved core is a faithful, clause-by-clause move
  (Linux error message byte-identical, prefix now `T::NAME`); F1 (minor, the
  NUL spelling of Test strategy #1, above), F2 (minor, #12's missing evidence,
  above), F3 (nit, the subprocess test's host-independence is Linux-only).
- testing: approve — 26 + 2 new tests, all of Test strategies #1–#10 accounted
  for; Finding 1 = the NUL gap, Finding 2 = #12's uncaptured leg.
- security: approve — S-1 (minor), the relative-registry-value asymmetry
  (deviation 7), explicitly carried and mitigated by the design.
- compatibility: approve — one nit on evidence attachment; the Linux surface and
  output are unchanged.
- performance: approve — no findings against the shared core's cost.

The three distinct minors (the unpinned NUL spelling, #12's uncaptured leg, the
relative-registry-value asymmetry) and the F3 nit are recorded above as open
items; no post-review code change was made.

