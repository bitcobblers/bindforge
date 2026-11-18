# Code review — security seat — round 1, pass c0

- Change under review: `git diff d4242107d82c9cf2ef3750615158165b4260f397...HEAD -- ':(exclude)docs/designs'`
  (11 files: `steam_common.rs` + `steam_windows.rs` + `steam_linux.rs` rewiring, CLI wiring + dual-plugin
  test, `check.yml` cross-`cargo check` step, `Cargo.toml`/`Cargo.lock` `winreg` addition, Windows test suite).
- Inputs used: that diff, `docs/designs/tasks/002-steam-windows-discovery/02-proposal.md`,
  `docs/tasks/T-002-steam-windows-discovery.md`, the runner's gate record (2026-11-18T06:26:23Z, all four
  legs green), and read-only inspection of the changed files for line locations. No prior findings (first pass).
- Checklist applied: `references/security.md` (code security), security seat only.

## Checklist walk

1. **External input validated at the boundary — PASS.** Untrusted ACF/VDF input goes through the unchanged
   `keyvalues` parser with the 1 MiB `MAX_DOCUMENT_BYTES` cap enforced on metadata *before* allocation
   (`steam_common.rs:283-296`), non-UTF-8 files are skipped, and `parse_manifest` reduces each manifest to
   two validated fields. Registry input is accepted only as a plain `REG_SZ`
   (`steam_windows.rs:230`), decoded UTF-16LE with a validity check (`steam_windows.rs:237-248`); every
   other value type is `None` (proposal §4, A9/A18). No code evaluation anywhere in the scan.
2. **Path handling — PASS with one minor note.** `usable_install_dir` (`steam_common.rs:266-273`) refuses
   empty-after-trim, interior NUL, `.`, `..`, absolute, `/`-containing, and — Windows only
   (`install_dir_rejects_backslash` = `true`, `steam_windows.rs:63-69`) — `\`-containing `installdir`, so
   the joined `install_path` cannot gain a traversal or nested segment; the Windows clause subsumes
   `C:\games` and `\\server\share` even on a Linux host where `is_absolute` would miss them (proposal §6).
   The `libraryfolders.vdf` VDF is read at the one fixed path and never searched (Proton look-alikes stay
   unopened). The Linux rule set is bit-for-bit the pre-refactor one (verified move, not a rewrite).
   Minor asymmetry for registry-sourced candidates: finding S-1 below. No shell commands exist at all
   (the `reg.exe` option was rejected; the crate is `unsafe`-free).
3. **Injection — PASS.** No shell, no regex, no `format!`-interpolated commands. The only dynamic path
   joins are the validated `installdir` (item 2) and registry/env candidate roots, and every candidate must
   pass `std::fs::canonicalize` plus an adjacent-`steamapps`-directory gate before anything under it is
   read (`steam_common.rs:305-318`). Registry keys and value names are hardcoded constants; the
   `RegistryKey` enum is a closed three-variant set fixing one (key, view) pair each, so no call site can
   invent a fourth hive/path.
4. **Secrets and PII — PASS.** No secrets in source, fixtures, or tests. The single error path emits only
   the selected root paths and a skipped-file count — no file contents — and the new suite pins that
   (`steam_windows_tests.rs`: `all_broken_root_yields_plugin_error_prefixed_with_steam_windows` asserts no
   `AppState`/`appid`/`installdir`/marker strings leak). No exports added.
5. **New dependencies — PASS.** One addition: `winreg 0.56.0` (workspace-pinned `winreg = "0.56"`, lock
   entry with checksum, `[target.'cfg(windows)'.dependencies]` so it never compiles on Linux), transitive
   `cfg-if 1.0.5`, reusing the already-locked `windows-sys` — the lock diff is exactly `winreg` + `cfg-if`,
   as proposal A10/#11 promises. Justified per proposal §9 and *Options considered* (chosen over
   `windows`-crate and hand-rolled FFI, each with stated cost). Checked against RustSec: no advisory
   directory exists for `winreg`, `cfg-if`, or `windows-sys` in rustsec/advisory-db `crates/` (404s;
   control path `crates/lz4-sys` resolves), i.e. no known advisories. The single-maintainer posture is
   accepted in the design with a one-file migration path (Risks), and the reader is confined to one
   private `#[cfg(windows)]` function.
6. **Filesystem writes — N/A.** Discovery is read-only; nothing is written, so atomicity does not arise.
   CLI `--steam-root` feeds the host-compiled plugin only; no new surface.
7. **Clean rejection — PASS.** Parse failure, oversize, non-UTF-8, unreadable → per-file skip counters,
   scan continues; the three-clause aggregate `Err` names only roots and counts. Registry reader maps
   every failure (absent key/value, other type, odd-length or unpaired-surrogate UTF-16) to `None`, and
   the decode indexes only after the `% 2` guard — no panic path. The `cli_dual_plugin_tests` subprocess
   test pins exit-0/`[]`/empty-stderr on a host with no install. `from_roots([])` is `Ok(vec![])`.

## Structural notes

- The sealed `SteamTarget` (crate-private `sealed::Sealed`, `steam_common.rs:31-34`) keeps the platform
  parameterization closed to `LinuxTarget`/`WindowsTarget`, so no external impl can relax the `\` clause
  or the rules the core enforces. `SteamCore`'s `with_roots` is `pub(crate)`.
- `steam_linux.rs` is a behavior-preserving delegation (`from_roots` → `SteamCore::with_roots`,
  `discover`/`name` forwarded); the Linux rule set, caps, error text and sort are the unchanged fixed
  point, matching the task's "must not change" clause.

## Evidence record / limitations

- Gate record (green, citable): `npm run check`, `cargo fmt --check`, `cargo clippy -D warnings`, and
  `cargo test --workspace --locked` all exit 0; the quoted `test result: ok.` line confirms the suite ran.
- Not established by execution anywhere: the `#[cfg(windows)]` `registry_string_value` body is never run
  (no Windows runner); the new CI step compile-verifies it for `x86_64-pc-windows-msvc` and proposal
  *Rollout* #5 / *Test strategy* #13 own the manual real-registry check. I record this gap explicitly —
  the design scopes it out knowingly (*Risks*: "No Windows CI runner"), its failure modes are `None`-or-
  panic-free `Result`/predicate maps, and no security property I require is left unestablished in a way
  that blocks the merge; not blocking.
- Deferred by the design (not re-raised here): reserved device names / trailing dots-spaces / ADS colons
  in `installdir`, `REG_EXPAND_SZ` expansion, `<root>/config/libraryfolders.vdf`, `\\?\` normalization,
  and consumer-side `install_path` validation. `install_path` is emitted without being opened, so the
  deferred naming rules cannot cause an open-anything failure in this change.
- The metadata-then-read size check has a TOCTOU window (file grows after `metadata`); this is verbatim
  pre-existing Linux code the task requires to stay unchanged, and the read stays capped in practice at
  whatever the file holds — noted for the record, not a finding against this diff.

## Findings

### S-1 (minor) — registry-sourced root candidates are not gated to absolute paths

- Location: `crates/discovery/src/steam_windows.rs:153-158` (`host_candidates`, the
  `candidates.push(PathBuf::from(value))` of the three registry values).
- Problem: the two env candidates pass through `steam_common::absolute_env` (set-but-relative values are
  ignored), but a registry value is appended verbatim with no absolute check. A corrupt or hostile
  `SteamPath`/`InstallPath` value containing a relative path (e.g. `..\..\some\dir`) resolves against the
  process working directory in `canonical_paths`, steering the scan there — the asymmetry is the one
  loose end of the Risks row "a corrupt or hostile registry value makes the plugin scan somewhere
  unexpected". Impact is bounded: the candidate still needs to canonicalize and contain a `steamapps/`
  directory, scanning is read-only and size-capped, and writing HKCU already implies code execution as
  the user — hence minor, and the design's stated mitigation covers it. Real-world Valve values are
  absolute (A3/A6), so a gate costs nothing.
- Fix: apply an absolute-path gate to registry values before pushing, mirroring `absolute_env` — e.g.
  `let p = PathBuf::from(value); if p.is_absolute() { candidates.push(p); }` — optionally via a shared
  helper, and pin "a relative registry value contributes no candidate" beside the existing
  `host_candidates_ignores_a_relative_env_value` test.

## Verdict

No blocker or major findings; one minor hardening asymmetry that does not violate a stated requirement
and whose threat is already carried, mitigated, and risk-accepted in the design. **approve**.
