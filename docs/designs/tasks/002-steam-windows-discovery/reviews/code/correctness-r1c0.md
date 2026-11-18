# Code review — correctness seat — round 1, pass c0

- Diff reviewed: `git diff d4242107d82c9cf2ef3750615158165b4260f397...HEAD -- ':(exclude)docs/designs'` (11 files, +1636/−246).
- Design: `02-proposal.md`; task: `docs/tasks/T-002-steam-windows-discovery.md`; runner gate record (green, 2026-11-18T06:26:23Z).
- Inputs honored: diff, proposal, task, gate record only. No other `reviews/` file opened. `03-build-notes.md` is not among this seat's permitted inputs (it lives under the excluded `docs/designs/` scope), so checklist items 3 and 5 are assessed against the proposal and code only — see *Evidence limits*.

## Checklist walk

1. **Components and interfaces vs proposal §1/§3/§8 — pass.**
   - `lib.rs` gains private `mod steam_common;` and `pub mod steam_windows;` (§1) ✓.
   - `steam_common`: `AppManifest` (pub fields), sealed `SteamTarget: Copy + sealed::Sealed` with `const NAME` / `platform()` / `install_dir_rejects_backslash()`, `SteamCore<T> { roots, target }` with `pub(crate) with_roots` and a crate-internal blanket `DiscoveryPlugin` impl (`name()` → `T::NAME`) ✓. All items §1 names as moved are present with the same names: `library_paths`, `libraries_in_root_with_skips`, `scan_library`, `read_text_file`, `canonical_paths`, `discovered_game`, `absolute_env`, `parse_manifest`/`usable_install_dir` generalized over `T` (helpers used only inside the module keep private visibility, which the crate-private module makes behaviorally equivalent to §1's `pub`-for-use rationale).
   - `steam_windows`: `SteamWindowsPlugin` (private `core` field), `from_host()` gated per §3 (real probe on Windows, `from_roots(Vec::new())` elsewhere), `from_roots`, `from_registry_and_roots(reg: &dyn Fn(RegistryKey, &str) -> Option<String>, env: &dyn Fn(&str) -> Option<String>)`, `WindowsTarget`, pure `pub fn host_candidates`, `pub fn parse_manifest`, `pub fn usable_install_dir`, `RegistryKey` with exactly the three (key, view) variants and `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]` — all signatures match §1/§4/§5 ✓.
   - `steam_linux`: keeps every public item of A15 (`SteamLinuxPlugin` (now holding `SteamCore<LinuxTarget>`), `host_candidates`, `libraries_in_root`, `parse_manifest` as thin wrappers, `pub use ... AppManifest`); `LinuxTarget::install_dir_rejects_backslash()` is `false` so the Linux rule set is bit-for-bit the old one ✓.
   - CLI (§8): host-probe branch registers Linux-then-Windows; roots branch is `#[cfg(not(windows))]` Linux / `#[cfg(windows)]` Windows; printing, JSON, exit status untouched ✓.
2. **Assumption register — honored.** A3/A4/A5: HKCU `Software\Valve\Steam` `SteamPath` opened plainly; HKLM `SOFTWARE\Valve\Steam` opened with `KEY_READ | KEY_WOW64_32KEY` then `KEY_READ | KEY_WOW64_64KEY`, logical keys not hardcoded `Wow6432Node` ✓. A6: `%ProgramFiles(x86)%` then `%ProgramFiles%`, appended unconditionally, absolute-only via `absolute_env` ✓. A9: `winreg 0.56` in the lock with `cfg-if` + reused `windows-sys` only (lock diff adds exactly `cfg-if` and `winreg`, per A10) ✓; `get_raw_value` + `vtype == REG_SZ`-only + manual UTF-16LE decode with trailing-NUL trim and fail-to-`None` on every irregularity (§3/§4) ✓. A7: `\` refusal parameterized through the target, exercised on Linux where the `\` clause decides (§6) ✓. A14 seam shapes reused verbatim ✓.
3. **Deviations — none visible in the code.** The moved scan core was diffed clause-by-clause against its `steam_linux.rs` original: `discover()`, the three-clause error path (message byte-identical for Linux, prefix now `T::NAME`), the `(slug, install_path)` sort, `Ok(vec![])`-when-no-roots, the 1 MiB/UTF-8 read gate, the canonical-path rule, and the VDF walk are all identical modulo the target parameter (§2's "three parameterizations and no other change") ✓. Whether any deviation the authors *did* declare is recorded in build-notes cannot be checked here (see *Evidence limits*) — nothing in the diff looks undeclared.
4. **Scope creep — none.** CI step (proposal §9: one cross-`cargo check` step before the trio), workspace + target-gated dependency (§9), CLI wiring (§8), new tests only under the two new test files. No fixture tree added or touched; `steam_linux_tests.rs`, `plugin_tests.rs`, `cli_tests.rs` absent from the diff name-list — the A15/#9 fixed point holds ✓.
5. **Build-notes claims — not assessable** (input restriction; see *Evidence limits*).
6. **Error paths fail closed.** Single `Err` path preserved once in the core; prefix equals registered `name()` and is tested for both plugins; registry reads map every failure to "no candidate", never an error; malformed-registry/path candidates are dropped by the canonical-and-require-`steamapps` rule before anything is opened ✓.
7. **Previous rounds — n/a** (first pass; no prior findings).

## Independent evidence gathered

- `cargo test --workspace --locked` (my run): every suite green — `steam_windows_tests` 26 passed, `steam_linux_tests` 23 passed (unmodified), `cli_tests` 3 passed (unmodified), `cli_dual_plugin_tests` 2 passed. The gate record's lone `0 passed` tail is the renderer's single-line capture of a doc-test/empty unit-test summary, not the new tests; the suites did run and pass.
- `cargo fmt --all -- --check`: clean. `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `winreg` appears in exactly one non-test source (`steam_windows.rs`, inside `#[cfg(windows)] fn registry_string_value`) — §3's "only consumer" claim holds.
- Linux `host_candidates` candidates are all `HOME`/`XDG_DATA_HOME`/`SNAP_USER_DATA`-derived, so the `env_clear`+fake-`HOME` CLI subprocess test is hermetic on Linux gate hosts.

## Findings

### F1 (minor) — Test strategy #1's enumerated NUL spelling is absent from both Windows rule-set loops
- Location: `crates/discovery/tests/steam_windows_tests.rs:215` (parity reject loop) and `:290` (`usable_install_dir` reject list).
- Problem: the proposal's Test strategy #1 enumerates the reject set as `""`/`.`/`..`/`/abs/path`/`sub/dir`/NUL. The ported parity loop and the direct rule test each carry five of the six spellings; no new test exercises the interior-NUL refusal (implemented in shared code at `steam_common.rs` `usable_install_dir`, `!install_dir.contains('\0')`). Note the *current* Linux suite (`steam_linux_tests.rs:113`) also lacks a NUL case, so the proposal's parenthetical overstates the existing suite and the parity/sharing claim of #1 is not broken — this is the one enumerated case family neither side pins.
- Fix: add a `"\0Hidden"`-style spelling to the `for bad in [...]` lists at both locations and to the Linux-side suite's expectation if parity requires it; the value survives the quoted-token reader, so the assertion is expressible.

### F2 (minor) — The Windows compile-verification leg is not established by any evidence available at review
- Location: `crates/discovery/src/steam_windows.rs:196-249` (`#[cfg(windows)] registry_string_value` / `from_host`); gate record (four legs, none the cross-target check).
- Problem: the task's Done-when requires the workspace to type-check for a Windows target so the `#[cfg(windows)]` code is compile-verified. The runner's gate record captured no `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` leg, and I could not run it here (the target's std is not installed on the review host: `E0463 can't find crate for core` for every crate, `rustup target add` outside my allowlist). A type error in the winreg usage would therefore pass every captured leg. Mitigating facts recorded rather than assumed away: this very diff adds the CI step that runs the leg (§9), and a static reading of the reader against winreg 0.56's API (`RegKey::predef`, `open_subkey`/`open_subkey_with_flags(path, REGSAM)`, `get_raw_value → RegValue { vtype, bytes }`, `RegType: PartialEq`) is consistent. Not blocking: the property is establishable by the CI leg this change ships, so nothing about proceeding is prevented; recorded so the ratification pass can confirm a green leg.
- Fix: have the runner capture (or CI confirm) a green `rustup target add x86_64-pc-windows-msvc && cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` leg and cite it in the gate record before ratification.

### F3 (nit) — The subprocess CLI test's host-independence is Linux-only
- Location: `apps/discovery-cli/tests/cli_dual_plugin_tests.rs:31-47`.
- Problem: the comment says the test "does not depend on the build host having Steam installed", but on a Windows host (this file is compiled there by `cargo check --all-targets`, and `cargo test` on a developer's Steam-installed Windows machine would run it) `SteamWindowsPlugin::from_host()` reads the real registry, which `env_clear` + fake `HOME` cannot control, so the `[]` assertion fails spuriously. All gate-mandated runs are on Linux, where the test is hermetic; manual gate #13 owns real-Windows verification.
- Fix: annotate the test `#[cfg(not(windows))]` (with a pointer to Test strategy #13) so the Windows build compiles without the un-controllable probe.

## Evidence limits

- `03-build-notes.md` was not among the permitted inputs (and `docs/designs/` is excluded from the reviewed diff), so checklist item 5 (build-notes claims) and the "declared" side of item 3 could not be read; no code-visible deviation exists that would need declaring, and item 3's undeclared-deviation test was applied directly to the diff.
- The Windows-target compile (F2) could not be executed on the review host for environment reasons; recorded as an evidence gap, not an observed defect.

## Verdict

approve — 0 blocker, 0 major, 2 minor, 1 nit. The refactor is a faithful move, the Windows plugin matches the proposal's shape and policy, the Linux fixed point is untouched and green, and the shipped tests cover Test strategies #1–#10 as enumerated except the single NUL spelling of F1.
