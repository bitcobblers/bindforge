---
doc_type: task
status: completed
date: 2026-11-18
type: feature
functional_area: backend
associated_adr: ADR-022
depends_on: [T-001-steam-linux-discovery]
---

# T-002: Implement Windows Steam discovery plugin

ADR-022 scopes Steam discovery v1 to Linux and Windows. `crates/discovery`
currently ships only `SteamLinuxPlugin`; the Windows backend ("read Steam
install via registry, enumerate library folders, parse `appmanifest_*.acf`",
ADR-022 Decision) is still missing.

Implement a `SteamWindowsPlugin` in `crates/discovery`, emitting
`DiscoveredGame` rows with `platform` set to `Platform::SteamWindows` (already
defined in `game.rs`) and `codec_id = None`.

**Reuse is a hard requirement.** The `libraryfolders.vdf` and
`appmanifest_*.acf` formats, the library-folder model, and the scan policy are
identical across the two platforms, so the Windows plugin must build on the
existing implementation rather than duplicate it:

- Reuse the platform-agnostic `keyvalues` parser and `normalize_slug` as-is.
- Reuse the parts of `steam_linux.rs` that are not host-probing: `AppManifest`
  and `parse_manifest`, `libraries_in_root` (VDF path resolution), the
  `appmanifest_*.acf` enumeration, the size-capped UTF-8 file reader, the
  canonical-path rule, the skip-counting / degraded-scan semantics, the single
  error path, and the output sort. Refactor these into a shared module (e.g. a
  `steam_common`-style internal module) or a platform-parameterized core that
  both plugins call — no copy-pasted scan logic. The Linux plugin's behavior
  and its existing tests must not change.
- The only genuinely Windows-specific code is host probing: locating the Steam
  install and building the root candidate list.

**Host probing (Windows-specific):**

- Read the Steam install path from the registry (candidate keys:
  `HKCU\Software\Valve\Steam` `SteamPath`, falling back to the machine-level
  Valve key), plus conventional default locations such as
  `%ProgramFiles(x86)%\Steam` and `%ProgramFiles%\Steam` as candidates.
- Registry access must sit behind a small injectable seam (trait or lookup
  function, mirroring how `SteamLinuxPlugin` injects roots via `from_roots`
  and the environment via a lookup fn) so that all root-selection logic is
  unit-testable on non-Windows hosts with a fake. The concrete registry
  dependency (`winreg` vs `windows-core`) is a design decision for the
  proposal; whatever is chosen must be gated to Windows targets so the
  workspace still builds and tests cleanly on Linux.
- `installdir` validation must additionally refuse values containing `\` on
  top of the existing `usable_install_dir` rules.

Done when:
- A `SteamWindowsPlugin` exists in `crates/discovery` sharing the Steam scan
  core with `SteamLinuxPlugin`; `crates/discovery` has no duplicated
  manifest/VDF scanning code.
- `SteamWindowsPlugin` exposes a roots-injection constructor (and a
  registry-injection seam or equivalent) so scanning and root selection are
  unit-tested on Linux with fixture trees and a fake registry.
- Unit tests cover: registry-sourced and default-candidate root selection,
  VDF/manifest scanning on fixtures, the `steam-windows` platform tag,
  backslash rejection in `installdir`, and the degraded-scan/error semantics.
- The `steam-windows` plugin is wired into `apps/discovery-cli`
  (`discovery list`) alongside `steam-linux`, and `--steam-root` keeps working
  for it.
- `cargo fmt`, `cargo clippy -D warnings`, and `cargo test --workspace
  --locked` pass on Linux, and the workspace type-checks for a Windows target
  (`cargo check --target x86_64-pc-windows-msvc`, or the `x86_64-pc-windows-gnu`
  target) on Linux so the `#[cfg(windows)]` code is compile-verified.
- On a Windows machine with Steam installed, `discovery list` correctly
  identifies installed games. (Integration gate; if CI has no Windows runner
  yet, note it as a manual verification step in the build notes.)
