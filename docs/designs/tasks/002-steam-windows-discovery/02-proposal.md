# Design: Windows Steam discovery plugin (T-002)

## Context

ADR-022 scopes Steam discovery v1 to Linux and Windows, and fixes the Windows
backend as "read Steam install via registry, enumerate library folders, parse
`appmanifest_*.acf`" (docs/adrs/022-game-discovery.md:47). `crates/discovery`
ships the Linux half of that — `SteamLinuxPlugin`
(crates/discovery/src/steam_linux.rs) — and `Platform::SteamWindows` already
exists and renders as `steam-windows` (crates/discovery/src/game.rs:5-18), but
nothing emits it. T-002 (docs/tasks/T-002-steam-windows-discovery.md) adds
`SteamWindowsPlugin` with the same row shape (`platform = SteamWindows`,
`codec_id = None`), and makes reuse a hard requirement: everything after root
selection moves into a shared module that both plugins call, with host probing
as the only Windows-specific code. Research:
docs/designs/tasks/002-steam-windows-discovery/01-research.md.

The change is additive for Linux users: `SteamLinuxPlugin`'s behavior, public
surface and existing tests are the refactoring's fixed point (its task text at
docs/tasks/T-002-steam-windows-discovery.md:34-35), the data model, plugin
trait, engine and `schemas/` are untouched, and the new registry dependency is
compiled only for Windows targets. Everything research left open (its eight
open questions) is decided here or listed in *Deferred / out of scope*.

## Goals / Non-goals

- Goals:
  - `SteamWindowsPlugin` in `crates/discovery`, emitting `DiscoveredGame` rows
    tagged `Platform::SteamWindows` with `codec_id = None`.
  - One Steam scan core, shared by both plugins: `AppManifest`/`parse_manifest`,
    `libraryfolders.vdf` resolution, `appmanifest_*.acf` enumeration, the
    size-capped UTF-8 reader, the canonical-path rule, skip counting, the single
    error path and the output sort — no duplicated scan code.
  - A Windows host probe behind an injectable registry seam, pure and unit-testable
    on Linux with a fake registry plus a fake environment lookup.
  - The mandated extra `installdir` refusal (values containing `\`) as a
    target-specific rule, leaving the Linux rule set as it is.
  - `steam-windows` wired into `apps/discovery-cli`'s `discovery list`, with
    `--steam-root` still working.
  - Linux gates green (`fmt`, `clippy -D warnings`, `test --workspace --locked`),
    plus compile verification of the `#[cfg(windows)]` code from Linux via
    `cargo check --target x86_64-pc-windows-msvc`.
- Non-goals:
  - Any change to `SteamLinuxPlugin`'s observable behavior, its `pub` items, or
    its test files; any change to `keyvalues` or `normalize_slug`.
  - Any change to `DiscoveredGame`, `Platform`, `DiscoveryPlugin`,
    `DiscoveryEngine`, `DiscoveryError`, or the CLI's output shape.
  - Non-game filtering, `StateFlags` readiness filtering, AppID use, slug
    migration, persistence, caching (all still ADR-022 territory).
  - A Windows CI runner, a Windows build/release pipeline, or self-publishing
    Windows binaries.
  - Epic/manual discovery, Steam Deck, macOS.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | Everything after root selection is the same on Windows as on Linux: the same `libraryfolders.vdf` shape (numeric-keyed blocks carrying `path`), the same ACF `AppState` field set. | verified | https://stackoverflow.com/questions/78531838/deserialize-steams-libraryfolders-vdf-into-an-object-c |
| A1b | Windows `libraryfolders.vdf` `path` values are written with doubled backslashes. | unverified | its only supporting citation was dead; the design assumes no spelling — the `\\`/`\"` unescape (A2, crates/discovery/src/keyvalues.rs:186-207) makes either form arrive usable |
| A2 | The existing reader already unescapes `\\` and `\"` inside quoted tokens, so a Windows `path` value arrives usable. | verified | crates/discovery/src/keyvalues.rs:186-207 |
| A3 | The user-level Steam key is `HKCU\Software\Valve\Steam`, value `SteamPath`, type `REG_SZ`, stored lowercase with forward slashes. | verified | https://superuser.com/questions/1507846/how-do-i-properly-use-tokens-and-skip-with-reg-query, https://noverse.dev/docs/app-guides/steam/ |
| A4 | The machine-level key is `HKLM\SOFTWARE\Valve\Steam`, value `InstallPath`; on 64-bit Windows it physically lives under `WOW6432Node` and must be reached with the `KEY_WOW64_32KEY`/`KEY_WOW64_64KEY` view flags rather than a hardcoded `Wow6432Node` path. | verified | https://learn.microsoft.com/en-us/windows/win32/winprog64/registry-redirector, https://learn.microsoft.com/en-us/windows/win32/winprog64/accessing-an-alternate-registry-view |
| A5 | `HKCU\Software` is not WOW64-redirected, so `HKCU\Software\Valve\Steam` has one physical copy that any bitness reads directly. | verified | https://learn.microsoft.com/en-us/windows/win32/winprog64/shared-registry-keys |
| A6 | `%ProgramFiles(x86)%` names the 64-bit-systems-only folder and is absent on 32-bit Windows; these values are absolute when set; `C:\Program Files (x86)\Steam` is Steam's default install location. | verified | https://learn.microsoft.com/en-us/windows/deployment/usmt/usmt-recognized-environment-variables, https://stackoverflow.com/questions/39557722/where-does-steam-store-library-directories |
| A7 | On Windows, `\` is the path separator and a reserved character in file names; on Linux it is an ordinary filename byte. | verified | https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file |
| A8 | `std::fs::canonicalize` on Windows returns extended-length (`\\?\`) paths; joining is backslash-delimited under that namespace. | verified | https://doc.rust-lang.org/std/fs/fn.canonicalize.html |
| A9 | `winreg` 0.56 (MIT, MSRV 1.60) exposes `HKCU`/`HKLM`, `open_subkey_with_flags(path, REG_SAM_FLAGS)` accepting the view flags, `get_value::<String>` (which decodes `REG_SZ`, `REG_EXPAND_SZ` **or** `REG_MULTI_SZ`), and `get_raw_value` returning a `RegValue { vtype, bytes }` that makes the value type observable. Its own dependencies are `cfg-if` and `windows-sys >=0.59,<=0.61`. | verified | https://docs.rs/winreg/latest/winreg/reg_key/struct.RegKey.html, https://crates.io/crates/winreg |
| A10 | A `Cargo.lock` records the dependency graph for all targets, so a Windows-only dependency adds lock entries to this Linux-managed lock; `windows-sys 0.61.2` (inside winreg's accepted window) and `windows-link 0.2.1` are already in it, and `cfg-if` is not. | verified | Cargo.lock:36-53, Cargo.lock:249-262, https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html |
| A11 | `[target.'cfg(windows)'.dependencies]` keeps a crate out of non-Windows compiles, and `cargo check` performs no code generation or linking, so it type-checks `#[cfg(windows)]` code from Linux with only `rustup target add`. | verified | https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html, https://doc.rust-lang.org/cargo/commands/cargo-check.html |
| A12 | `x86_64-pc-windows-msvc` and `x86_64-pc-windows-gnu` are Tier 1 targets shipping a precompiled `std`; `windows-gnu` additionally supports full cross-builds from Linux with mingw-w64. | verified | https://doc.rust-lang.org/rustc/platform-support.html, https://doc.rust-lang.org/rustc/platform-support/windows-gnu.html |
| A13 | The workspace pins the 1.85.0 channel and edition 2024, above `winreg`'s 1.60 MSRV; `crates/discovery` depends on `serde` alone today. | verified | rust-toolchain.toml:7, Cargo.toml:14-18, crates/discovery/Cargo.toml:6-7 |
| A14 | The seam Linux already demonstrates: `SteamLinuxPlugin::from_roots` for caller-supplied roots, `host_candidates(home, &dyn Fn(&str) -> Option<String>)` as a pure probe, `canonical_paths` as the root/library gate, one three-clause `Err` path, and `(slug, install_path)` sorting. | verified | crates/discovery/src/steam_linux.rs:46-50, :58-93, :101, :322-334 |
| A15 | The public surface `crates/discovery`'s own tests import from `steam_linux` is `SteamLinuxPlugin`, `host_candidates`, `libraries_in_root`, `parse_manifest`; the CLI registers exactly one plugin and asserts exactly four fixture rows and the serde spelling `"SteamLinux"`. | verified | crates/discovery/tests/steam_linux_tests.rs:12-14, apps/discovery-cli/src/main.rs:33-38, apps/discovery-cli/tests/cli_tests.rs:68-72, :124 |
| A16 | `Platform::SteamWindows` renders as `steam-windows` through `Display` and as `"SteamWindows"` through serde's default variant naming. | verified | crates/discovery/src/game.rs:5-18 |
| A17 | Both Valve keys are readable by an ordinary user process with `KEY_READ`. | unverified | community usage only (https://stackoverflow.com/questions/58388230/how-to-read-the-steam-install-path-from-the-windows-registry); no Valve or Microsoft guarantee, and no Windows runner to check on |
| A18 | In practice `SteamPath`/`InstallPath` are written as `REG_SZ`; precedents *also* accept `REG_EXPAND_SZ`. | unverified | registry exports in 01-research.md (§"Host probing"), https://github.com/osztenkurden/steam-game-path/blob/master/docs/windows-registry.md |
| A19 | Current Windows Steam keeps `libraryfolders.vdf` where the shared core looks (`<root>/steamapps/`); community tooling reports a second location, `<root>/config/`, depending on client version. | unverified | https://github.com/lutris/lutris/blob/master/lutris/util/steam/config.py |
| A20 | On a 32-bit Windows host, requesting the 64-bit registry view fails rather than returning another key's data. | unverified | the view-flag docs describe redirection on 64-bit hosts only (https://learn.microsoft.com/en-us/windows/win32/winprog64/accessing-an-alternate-registry-view); no statement found for the 32-bit case |
| A21 | Windows environment-variable names are case-insensitive, so one pinned spelling per variable is enough. | unverified | no primary statement found in research; the seam takes the name as a string constant, so the spelling is at least pinned in one place (docs/tasks/T-002-steam-windows-discovery.md:44) |

## Options considered

**Copy `steam_linux.rs` into `steam_windows.rs` and edit the host probe.** Fastest,
and the two files would stay trivially readable. Rejected outright: the task makes
no-duplication a Done-when clause (docs/tasks/T-002-steam-windows-discovery.md:22-35,
:56-58), and a forked reader, caps, skip counter, error path and sort are five places
where the two platforms would silently drift.

**A platform-parameterized core (`SteamCore<T>`) over a crate-private `SteamTarget`
parameter trait (chosen).** Keeps one copy of every scan rule, gives the shared code
the two things it genuinely needs per platform (the `Platform` tag / message prefix,
and whether `installdir` may contain `\`), and lets `steam_linux.rs` keep its `pub`
wrappers so its tests compile unchanged. Chosen; its cost is one trait and one
generic struct where today there are plain functions — see *Risks*.

**A `steam_common` module of plain functions with a `platform: Platform` argument
and a `reject_backslash: bool` flag.** Also duplicating nothing. Rejected: a bare
`bool` argument at each call site is exactly the "which argument means what" drift the
refactor exists to remove, and it cannot carry the tag *and* the path rule together.
The parameter trait is the same size and says which platform owns which rule.

**`winreg` behind the registry seam (chosen).** Small (≈2.0K SLoC), MIT, MSRV 1.60,
reuses the `windows-sys` already in the lock (A9, A10), and covers exactly the two
operations needed: open a subkey with view flags, read one string value. Chosen; its
single-maintainer "work in progress" posture is accepted *because* the seam makes the
swap a one-file change (see *Risks*).

**`windows` / `windows-core` calling `RegOpenKeyEx`/`RegGetValueW` directly.**
Vendor-supported and the home of the view flags, and `windows-core` alone does not
carry the registry feature set, so this means the `windows` crate: a much larger
metadata-driven dependency tree pulled in for two calls. Rejected for v1; recorded as
the migration target if `winreg` goes unmaintained.

**Hand-rolled `extern "system"` declarations for `RegOpenKeyExW`/`RegQueryValueExW`.**
Zero dependencies and precisely the needed surface. Rejected: it puts `unsafe` FFI and
hand-written signatures into a crate that is otherwise `unsafe`-free, for the sake of
avoiding one small target-gated dependency.

**Spawn `reg.exe` and parse stdout.** No dependency, and it works from a shell.
Rejected: an already-documented encoding hazard for non-ASCII paths, plus a spawned
process per key. (source: https://github.com/osztenkurden/steam-game-path/blob/master/docs/windows-registry.md)

**A trait (`trait Registry { fn string_value(...) }`) with a named fake in the crate.**
Reused-testability across packages is nicer, but nothing outside `crates/discovery`
constructs a registry — `apps/discovery-cli` only ever calls `from_host()`. Rejected in
favor of the `&dyn Fn` shape the Linux probe already uses for the environment
(A14): the fake becomes a closure in the test file and the crate gains no test-only
public type.

**Hardcode `SOFTWARE\Wow6432Node\Valve\Steam` as a candidate key.** What some community
tools do. Rejected: Microsoft's guidance is to open the logical key with the view flag,
and the hardcoded spelling is wrong on 32-bit Windows (A4).

**Query HKLM before HKCU (steam-game-path's order).** Rejected: the task prescribes
HKCU-`SteamPath`-first with the machine key as fallback
(docs/tasks/T-002-steam-windows-discovery.md:41-43), and the per-user key is the one
that tracks the Steam client the user actually runs (A5).

**Expand `REG_EXPAND_SZ` registry values before use.** Matches the widest precedent.
Rejected for v1: it buys one more FFI call (`ExpandEnvironmentStringsW`) for a value
type no observed install uses (A18), and an *unexpanded* value would otherwise be a
candidate path containing literal `%VAR%`. §4 refuses the type instead.

**Strip the `\\?\` prefix from canonical paths on Windows.** Nicer table output.
Rejected: the extended-length namespace is what makes long library paths openable
(A8), and no consumer of `install_path` exists yet to argue for the trade — §7 keeps
the canonical form and *Deferred* owns the normalization question.

**Gate the whole `steam_windows` module behind `#[cfg(windows)]`.** Keeps non-Windows
builds minimal. Rejected: the task requires roots-injection and root-selection tests to
run on Linux (docs/tasks/T-002-steam-windows-discovery.md:59-61), which a fully gated
type cannot do; only the registry reads, the `from_host()` probe and the CLI roots branch
are gated (§3, §8).

**Register `SteamWindowsPlugin::from_roots(--steam-root)` on every host too.** Would emit
`steam-windows` rows from a Linux runner. Rejected: `crates/discovery/tests/fixtures/steam/minimal`
would be scanned by both plugins, doubling every CLI assertion ("exactly the four fixture
rows", A15) to protect rows no supported host can produce; §8 instead feeds the flag to the
plugin of the compiled-for host, where the task's ":65-67" clause is met.

## Chosen design

### 1. Modules and public surface

Four moves in `crates/discovery`, alongside the existing seven modules
(crates/discovery/src/lib.rs:1-7): a private `steam_common` and a public
`steam_windows` module, the `steam_linux.rs` rewiring described below, and
`lib.rs` gaining the two module declarations.

```rust
// lib.rs gains:
mod steam_common;                 // crate-private: nothing outside the crate reaches it
pub mod steam_windows;            // the Windows plugin and its host policy

// crates/discovery/src/steam_common.rs — the scan core both plugins call.
pub struct AppManifest { pub display_name: String, pub install_dir: String }
pub trait SteamTarget: Copy {                 // sealed: `Sealed` supertrait, no downcast
    const NAME: &'static str;                 // "steam-linux" | "steam-windows"
    fn platform(&self) -> Platform;
    fn install_dir_rejects_backslash(&self) -> bool;
}
pub struct SteamCore<T: SteamTarget> { roots: Vec<PathBuf>, target: T }
impl<T: SteamTarget> SteamCore<T> { pub(crate) fn with_roots(roots: impl IntoIterator<Item = PathBuf>, target: T) -> Self; }
impl<T: SteamTarget> DiscoveryPlugin for SteamCore<T> { /* name() -> T::NAME, discover() as today */ }
// moved verbatim from steam_linux.rs: library_paths, libraries_in_root_with_skips,
// scan_library, read_text_file, canonical_paths, discovered_game, absolute_env, and
// parse_manifest/usable_install_dir generalized over `T: SteamTarget`.

// crates/discovery/src/steam_windows.rs — the plugin and its Windows policy.
pub struct SteamWindowsPlugin { core: SteamCore<WindowsTarget> }   // private field
impl SteamWindowsPlugin {
    pub fn from_host() -> Self;
    pub fn from_roots(roots: impl IntoIterator<Item = PathBuf>) -> Self;
    pub fn from_registry_and_roots(
        reg: &dyn Fn(RegistryKey, &str) -> Option<String>,   // the registry seam (§4)
        env: &dyn Fn(&str) -> Option<String>,                 // the environment seam
    ) -> Self;
}
impl DiscoveryPlugin for SteamWindowsPlugin { /* delegates to `core`: name() is self.core.name() */ }
pub struct WindowsTarget;                    // implements SteamTarget
pub fn host_candidates(
    reg: &dyn Fn(RegistryKey, &str) -> Option<String>,   // pure: no registry, no filesystem
    env: &dyn Fn(&str) -> Option<String>,                 // the shape Linux's probe uses
) -> Vec<PathBuf>;
pub fn parse_manifest(text: &str) -> Option<AppManifest>;          // Windows rule set
pub fn usable_install_dir(install_dir: &str) -> bool;              // Windows rule set
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryKey { CurrentUserValveSteam, LocalMachineValveSteam32, LocalMachineValveSteam64 }   // the (key, view) pairs §5 reads
```

Items in the crate-private `steam_common` are declared `pub` so `steam_linux` and
`steam_windows` can use and re-export them; the module itself stays private, so nothing
becomes public by accident. `steam_linux.rs` keeps every `pub` item it has today
(A15): `SteamLinuxPlugin` holds a `SteamCore<LinuxTarget>` instead of a `Vec<PathBuf>`,
`host_candidates` stays a Linux-only function in `steam_linux.rs`, and
`parse_manifest` / `libraries_in_root` / `AppManifest` become thin wrappers and a
`pub use` re-export over `steam_common`. `crates/discovery/tests/steam_linux_tests.rs`
and `crates/discovery/tests/plugin_tests.rs` therefore compile and pass without a single
edited line, and that is the refactor's acceptance test.

The plugin struct, not the core, is what `discovery-cli` boxes; `SteamCore<T>`'s
blanket `DiscoveryPlugin` impl is crate-internal.

### 2. The shared scan core

`SteamCore<T>::discover()` is today's `SteamLinuxPlugin::discover()` body
(crates/discovery/src/steam_linux.rs:58-93) with three parameterizations and no other
change:

- the row's `platform` comes from `self.target.platform()` instead of the literal
  `Platform::SteamLinux` (this is the only field in `discovered_game`
  — crates/discovery/src/steam_linux.rs:234-247 — that differs by platform);
- the error message's `steam-linux:` prefix becomes `T::NAME`, which is the same
  string the plugin's `name()` returns, so the prefix can no longer drift from the
  registered plugin name;
- `usable_install_dir` consults `self.target.install_dir_rejects_backslash()` (§6).

Unchanged, because shared: `libraryfolders.vdf` read at the one fixed path
`<root>/steamapps/libraryfolders.vdf` and never searched for (so the Proton-prefix
look-alikes stay unopened), the numeric-keyed-block walk, the relative-`path`-joins-the-
VDF-directory rule, the always-included root-as-library, the `appmanifest_*.acf`
enumeration directly inside each library's `steamapps/`, the 1 MiB / valid-UTF-8 read
gate, the canonical-path rule (canonicalize → require an adjacent `steamapps/` → sort →
de-dup), the three-clause single error path, the `(slug, install_path)` sort, and the
`Ok(vec![])`-when-no-roots rule. `keyvalues` and `normalize_slug` are used as they are
(crates/discovery/src/keyvalues.rs:1-6 states the platform-neutrality as a property of
the code; A2 confirms the unescape behavior Windows needs).

Keeping the VDF location at that one path is a decision, not an oversight: the second
location community tooling reports (A19) would be a shared-core change and therefore
Linux-visible, which the task forbids. It is *Deferred* with its evidence.

### 3. Windows host probing, and where `cfg(windows)` sits

`SteamWindowsPlugin::from_host()` is the only function that touches a real registry,
and it is target-gated:

```rust
#[cfg(windows)]
pub fn from_host() -> Self {
    Self::from_registry_and_roots(&registry_string_value, &|key| std::env::var(key).ok())
}
#[cfg(not(windows))]
pub fn from_host() -> Self {
    Self::from_roots(Vec::new())     // there is no Windows Steam install on this host
}
```

`from_registry_and_roots` runs `host_candidates` and pipes the result through the shared
canonical-path rule, exactly as `SteamLinuxPlugin::from_host()` does with
`host_candidates` (crates/discovery/src/steam_linux.rs:38-43). `registry_string_value`,
the concrete Windows reader, is a private `#[cfg(windows)]` function and the crate's only
`winreg` consumer, so `winreg` never appears in a non-Windows compile (A11). The plugin
type, `WindowsTarget`, `host_candidates`, `RegistryKey` and both rule functions compile
and are tested on every host — that is what lets *Test strategy* #3 exercise root
selection on Linux.

`registry_string_value` takes one `RegistryKey` variant at a time, and each variant fixes one
(key, view) pair: handle `HKCU` (`winreg::enums` predefined handles) with subkey
`Software\Valve\Steam`, opened plainly — `HKCU\Software` is a shared, non-redirected key (A5) —
or handle `HKLM` with subkey `SOFTWARE\Valve\Steam` opened with
`KEY_READ | KEY_WOW64_32KEY` (`LocalMachineValveSteam32`) or `KEY_READ | KEY_WOW64_64KEY`
(`LocalMachineValveSteam64`) (A4). It performs one `get_raw_value(name)` in that call, returns
the decoded string **only when `vtype == REG_SZ`** (A9), and maps every failure — absent
key, absent value, access denied, other `vtype`, non-UTF-16 data — to `None`. Reading is
the only thing that can fail, and a failure is "no candidate", never an error. Which HKLM
views to ask, and in what order, is `host_candidates`' policy, not the reader's (§5).

### 4. The registry seam and value typing

The seam is a function parameter, not a trait:
`reg: &dyn Fn(RegistryKey, &str) -> Option<String>`, the sibling of the
`env: &dyn Fn(&str) -> Option<String>` parameter the Linux probe already takes (A14).
`RegistryKey` is a three-variant enum listing exactly the (key, view) pairs §5 reads, so no
call site can invent a fourth. Windows passes the private `registry_string_value` (§3); every
test passes a closure over a `HashMap<(RegistryKey, &str), String>`. That is the whole seam:
the crate gains no trait, no fake type and no test-only public API.

The seam reports a value *only* when the registry stored a plain `REG_SZ` path: a
`REG_EXPAND_SZ` (which would carry raw `%VAR%` text), a `REG_MULTI_SZ`, a `REG_DWORD`, or
a non-UTF-16 value is `None` (A9, A18). Consequences, stated once here and referenced
elsewhere: no `%VAR%` expansion, no multi-string guessing, and no environment-variable
syntax in an emitted path. A host whose Steam records its path in another value type
simply loses that candidate and is covered by the `%ProgramFiles%` default or by
`--steam-root` (§8).

### 5. Root candidates

`host_candidates` is pure — registry and environment are lookups, no filesystem access —
and returns candidates in this order:

1. `HKCU\Software\Valve\Steam` → `SteamPath` (A3, A5).
2. `HKLM\SOFTWARE\Valve\Steam` → `InstallPath` as two ordered candidates:
   `LocalMachineValveSteam32` then `LocalMachineValveSteam64` (A4); values are appended in
   that order, and the canonical-path rule collapses both views reporting one path.
3. `%ProgramFiles(x86)%\Steam`, then `%ProgramFiles%\Steam`, each only when the variable is
   set and absolute — an unset variable is "no candidate", the whole story on 32-bit hosts
   where `(x86)` does not exist (A6); a relative value is ignored by `absolute_env` (§1).

The env defaults are appended even when a registry candidate was found: no candidate
needs a separate existence gate (§2's canonical-path rule drops what is not there), and
`canonical_paths` collapses the common case where the registry points at the default
folder, so a double-listed root costs nothing. Registry values arrive in mixed case and either
slash form (A3, A4); no normalization happens here — the canonical-path rule is what turns both
into one comparable spelling (§7).

`from_registry_and_roots` feeds this list to `SteamCore::with_roots`; there is no
separate "is Steam installed here" test, because a root that cannot be canonicalized or
lacks `steamapps/` is already dropped by the rule the Linux candidate list depends on.

### 6. `installdir` validation on Windows

`WindowsTarget::install_dir_rejects_backslash()` is `true`, so
`steam_common::usable_install_dir` refuses, in addition to the existing set (trim-empty,
interior NUL, `.`, `..`, absolute, contains `/` —
crates/discovery/src/steam_linux.rs:275-282), any value containing `\`.
`LinuxTarget`'s is `false`, so the Linux rule set is bit-for-bit what it is today. This
is the task's mandated addition (docs/tasks/T-002-steam-windows-discovery.md:52-53), and
the reason it is not a formality: on Linux `\` is an ordinary filename byte (A7), while
on Windows it is the separator, so `foo\bar` would silently become a nested path inside
the emitted `install_path`. The added clause also subsumes the Windows-shaped values a
corrupt or hostile manifest could carry — `C:\games` and `\\server\share` both contain
`\` — so the rule is checkable on Linux, where the `\` clause decides the case rather
than `Path::is_absolute` (which reads a Windows path as relative on a Linux host).

Nothing wider is added here: reserved device names, trailing dots and spaces, and
alternate-data-stream colons are *Deferred*. A manifest is untrusted input in the same
way that the Linux design already treats it, and the composed path is emitted without being
opened.

### 7. Emitted rows and their path namespace

Rows are built exactly as on Linux: `slug = normalize_slug(display_name)`,
`display_name`, `install_path = <canonical library>/steamapps/common/<installdir>`,
`codec_id = None`, `platform = Platform::SteamWindows` (A16 governs both spellings that
reach `discovery list`). AppIDs stay unread and unstored (docs/adrs/022-game-discovery.md:49).

Because Windows `canonicalize` yields extended-length paths (A8), an emitted
`install_path` on Windows starts with `\\?\`, e.g.
`\\?\C:\Program Files (x86)\Steam\steamapps\common\<installdir>`, and so does the JSON.
This design keeps that form: the prefix is the namespace in which long library paths are
openable at all, `install_path` is the field a later consumer opens, and stripping it
would trade that guarantee for a prettier table with no consumer yet to weigh it. The
table, `--json`, and the `DiscoveredGame` shape are otherwise unchanged; Linux output is
byte-identical to today's. Recorded in *Risks* and *Deferred*.

### 8. CLI wiring (`apps/discovery-cli`)

`Commands::List` keeps its arguments as they are; only the plugin selection at
apps/discovery-cli/src/main.rs:33-38 changes, host-aware on the roots branch only:

```rust
if steam_root.is_empty() {
    // Host probing: each plugin probes its own platform (§3).
    engine.register_plugin(Box::new(SteamLinuxPlugin::from_host()));
    engine.register_plugin(Box::new(SteamWindowsPlugin::from_host()));
} else {
    // `--steam-root` feeds the plugin whose host can produce tagged rows; the
    // row's platform tag is the plugin's, as on T-001's seam.
    #[cfg(not(windows))]
    engine.register_plugin(Box::new(SteamLinuxPlugin::from_roots(steam_root.iter().cloned())));
    #[cfg(windows)]
    engine.register_plugin(Box::new(SteamWindowsPlugin::from_roots(steam_root.iter().cloned())));
}
```

So `steam-windows` is wired into `discovery list` and the flag "keeps working for it" (Done-when,
docs/tasks/T-002-steam-windows-discovery.md:65-67) on the host that can produce its rows. The
Linux binary's roots branch keeps T-001's one-plugin shape, so the three existing CLI tests
(apps/discovery-cli/tests/cli_tests.rs) pass untouched — *Test strategy* #9's gate; the Windows
branch is type-checked by #12 and run only by #13's `--steam-root` check, named as its lone gate
in *Rollout* #5. Printing, JSON and exit status are untouched.

### 9. Dependencies, gating, and gates

`crates/discovery/Cargo.toml` gains one table, nothing else:

```toml
[target.'cfg(windows)'.dependencies]
winreg = "0.56"
```

and the workspace's `[workspace.dependencies]` (Cargo.toml:20-22) gains the pinned entry
so the crate inherits it with `workspace = true`, matching how `serde` is handled. The
lock file therefore gains `winreg` and its `cfg-if` dependency; `windows-sys 0.61.2` is
already locked and sits inside winreg's accepted window, so it is reused, not duplicated
(A10). `Cargo.lock` is committed once with that diff, and every later gate runs under
`--locked` as before. Nothing else in the workspace changes: no new `apps/` member, no
change to `bindforge-core`, no schema change.

Gate posture: the three existing CI commands (.github/workflows/check.yml:28-33) stay
as they are on Linux; one step is added before them —
`rustup target add x86_64-pc-windows-msvc` then
`cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` — which is what
compile-verifies §3's gated code and `registry_string_value` from a Linux runner
(A11, A12). The `x86_64-pc-windows-gnu` target is equally supported and is the local
alternative; CI does not run it.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| The `SteamCore<T>`/`SteamTarget` refactor regresses Linux behavior. | Low | High | The refactor is a move, not a rewrite: `steam_linux_tests.rs`, `plugin_tests.rs` and the three CLI tests are the gate and are not allowed to change (§1); *Test strategy* #1 and #9 pin the two rule sets apart. |
| Windows Steam keeps `libraryfolders.vdf` only under `config/`, so secondary libraries go missing on Windows (A19). | Medium | Medium | The root's own `steamapps/` is still scanned, so the default install still yields games; the manual Windows gate (*Rollout* #5) checks secondary libraries explicitly, and *Deferred* names the follow-up if it reports a miss. |
| No Windows CI runner: registry policy is compile-verified but never run against a real install (A17, A18 unverified). | Medium | Medium | `host_candidates` is pure and its full ordering matrix is unit-tested with a fake (§5, *Test strategy* #3); `--steam-root` gives any user an override; the manual gate records a real `discovery list` run in `03-build-notes.md`. |
| `winreg` goes unmaintained (single maintainer, self-described work in progress). | Low | Medium | It is target-gated, reached through one private function, and its only non-trivial need (view flags, raw-type read) is two calls; the migration to `windows-core` is one file (§*Options considered*). |
| A corrupt or hostile registry value makes the plugin scan somewhere unexpected, and its path becomes an emitted prefix. | Medium | Medium | A candidate must canonicalize and contain a `steamapps/` directory before anything under it is read (§2); scanning is read-only with per-file caps; AppIDs stay unused and nothing persists; write-side validation stays the codec's job (*Deferred*). |
| `install_path` values carry a `\\?\` prefix that surprises a later consumer (§7). | High | Low–Medium | Documented in the *Compatibility matrix*; the extended-length namespace is the safe-to-open form; normalization is *Deferred* until a consumer exists. |
| A reserved-name or trailing-dot `installdir` yields a path Windows cannot open. | Low | Medium | The existing reject set plus the `\` clause covers traversal; the wider naming rules are *Deferred* with their evidence, and discovery never opens the path. |
| Windows-only code rots between runs because Linux clippy never lints it. | Medium | Low | The new CI step `cargo check`s it for the Windows target on every PR; clippy-for-cross-target is noted as an optional strengthening in *Rollout*. |
| `ProgramFiles(x86)` / `ProgramFiles` spelling behaves differently from the pinned constants on some host (A21). | Low | Low | One named constant per variable in `host_candidates`, unit-pinned in *Test strategy* #3; a wrong name simply drops that fallback candidate, never a wrong path. |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Windows 10/11 x64, default Steam install under `%ProgramFiles(x86)%\Steam` | Yes | Registry candidate and env default both resolve; verified only through *Rollout* #5 (no Windows runner). |
| Windows, Steam on a non-default drive, or a second library on another volume | Yes | `SteamPath` plus `libraryfolders.vdf` `path` values, through the canonical-path rule (§5). |
| Windows x64 whose Valve key exists only under `WOW6432Node` | Yes | HKLM is queried with the 32-bit view first, the 64-bit view second (§3, §5). |
| Windows, per-user install with no HKLM Valve key, or a machine key with no HKCU key | Yes | Both keys are optional candidates; whatever resolves is used, and neither is required (§5). |
| Windows, a Valve key or value the user cannot read | Yes | A failed read is "no candidate" (A17, §3); the scan continues with the remaining candidates. |
| Windows, Steam not installed (keys absent, env vars unset) | Yes | `Ok(vec![])`, no stderr, exit 0 — the same "not installed here" rule Linux has. |
| Windows, a `REG_EXPAND_SZ` / `REG_MULTI_SZ` / non-`REG_SZ` path value | No | That candidate is refused (§4); coverage falls back to the env default or `--steam-root`. Unverified how common this is (A18). |
| Windows, library declared unmounted (external drive offline) | Partial | Dropped by the shared canonicalize-and-require-`steamapps` rule — identical to Linux. |
| Windows, secondary library whose `libraryfolders.vdf` lives only in `<root>/config/` | Partial | The core reads `<root>/steamapps/libraryfolders.vdf` only (§2, A19); the root's own `steamapps/` is still scanned. |
| Windows, 32-bit (x86) host | Partial | HKCU candidate and the scan work (A5); the HKLM 64-bit-view query (A20) and the absent `%ProgramFiles(x86)%` fallback are unexercised (caveat in *Risks*). |
| Windows, ARM64 host | Partial | Same code path as x64 (A4's model); not exercised in this run. |
| Linux build host / `cargo test --workspace --locked` | Yes | `winreg` is not compiled for a Linux target (A11); all tests here are hermetic through `from_roots` and the fake registry. |
| Linux host, `discovery list` | Yes | `steam-windows` registers with zero roots → `Ok(vec![])`, no rows, no stderr (§3, §8). |
| CI `rust` job (`ubuntu-latest`, no Steam, no network) | Yes | Unchanged three commands plus one cross-`cargo check` step (§9); no test reads a home directory. |
| Cross-compile to `x86_64-pc-windows-gnu` | Partial (local only) | Same `cfg(windows)` code path; supported locally (A12), and *Rollout* #4 runs the `cargo check` for it once — CI checks only the `msvc` target. |
| macOS | No | Outside ADR-022's v1 scope (docs/adrs/022-game-discovery.md:21). |
| Steam Deck | No | Excluded by ADR-022 (docs/adrs/022-game-discovery.md:21). |

## Test strategy

1. Unit (shared core, both platforms): for every case in the current Linux
   `parse_manifest` suite (name+installdir, name-absent and empty-name fallbacks, no
   `AppState`, unparseable text, mixed key case, and the `""`/`.`/`..`/`/abs/path`/
   `sub/dir`/NUL reject set), `steam_windows::parse_manifest` agrees with
   `steam_linux::parse_manifest` — the sharing claim in §2 is a test, not a comment.
2. Unit (the one rule that differs): an `installdir` containing a backslash —
   `foo\bar`, the doubled form `foo\\bar` as a real Windows ACF writes it (A2),
   `C:\games`, `\\server\share` — yields `None` from `steam_windows::parse_manifest`
   and still yields `Some` from `steam_linux::parse_manifest`; #1's both-sides suite
   is what pins this as the only difference, without a public `usable_install_dir`
   on the Linux side.
3. Unit (`steam_windows::host_candidates`, pure, fake registry + fake env): the exact ordered
   candidate list for each of — both keys present (HKCU `SteamPath`, then both HKLM views'
   `InstallPath` in §5's order, then the two env candidates); HKCU only; HKLM-32-bit-view only;
   HKLM-64-bit-view only; both views answering (one path from both → one root after the
   canonical-path rule, distinct paths → two roots and rows from two installs); neither key
   (exactly the two env candidates in order); `%ProgramFiles(x86)%` unset (only
   `%ProgramFiles%\Steam`); both unset and no key (empty list); a set-but-relative env value
   ignored; a key the fake reports as `None` (the §4 refusals) contributing no candidate; and a
   registry value equal to an env candidate not de-duped by `host_candidates` (§5).
4. Unit (`SteamWindowsPlugin::from_registry_and_roots` on the `minimal` fixture): the
   fake registry points at the fixture root and the four games come back tagged
   `Platform::SteamWindows` with `codec_id == None`,
   `slug == normalize_slug(display_name)`, `install_path` under the canonicalized
   library's `steamapps/common/`, ordered by `(slug, install_path)` — proving a
   registry-sourced root reaches the shared scan end to end.
5. Unit (fixture sharing, §2): `SteamWindowsPlugin::from_roots([steam/minimal])` returns
   the same four rows as `SteamLinuxPlugin::from_roots([steam/minimal])` except for the
   `platform` field, and no row ever carries a `compatdata/` path or the prefix-decoy
   slug — no new fixture tree is introduced.
6. Unit (failure policy for the new plugin, §2): `from_roots` with a nonexistent root
   returns `Ok(vec![])`; `from_roots([steam/all-broken])` returns exactly one
   `Err(DiscoveryError::PluginError(msg))` whose message starts `steam-windows:`, names
   the selected root, and leaks no file contents; the prefix equals the plugin's
   `name()` for both plugins.
7. Unit (plugin identity): `SteamWindowsPlugin::from_roots([]).name() == "steam-windows"`
   and `SteamLinuxPlugin::from_roots([]).name() == "steam-linux"`.
8. Unit (registry seam shape): a recording fake shows `host_candidates` querying exactly
   the three `RegistryKey` variants §5 names, in that order, so a fourth pair cannot be
   added silently; `registry_string_value` stays the crate's only `winreg` consumer and stays
   `#[cfg(windows)]`, which *Test strategy* #12 compiles.
9. Integration (Linux regression, the refactor's gate): `crates/discovery/tests/*` and
   `apps/discovery-cli/tests/cli_tests.rs` compile and pass unmodified — including the
   `"SteamLinux"` serde spelling and the "exactly the four fixture rows" count — which
   is what proves §1's public-surface preservation and §8's no-doubling choice.
10. Integration (`apps/discovery-cli`): the host-probe branch registers both plugins in
    order, and a `SteamWindowsPlugin::from_roots([])` registered beside a populated Linux
    plugin yields only the Linux rows with nothing on stderr — an empty Windows root set
    is `Ok(vec![])`, not an error the engine would print.
11. Platform (Linux gates): `cargo fmt --all -- --check`,
    `cargo clippy --workspace --all-targets -- -D warnings` and
    `cargo test --workspace --locked` pass on the pinned 1.85.0 toolchain (A13), with the
    committed `Cargo.lock` diff limited to `winreg` and `cfg-if` (A10).
12. Platform (Windows compile gate): `rustup target add x86_64-pc-windows-msvc` then
    `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` passes on
    Linux, type-checking §3's `from_host()` and `registry_string_value` (A11, A12); the same with
    `x86_64-pc-windows-gnu` is run once locally per *Rollout* #4.
13. Platform (manual Windows gate, no runner yet): on a Windows machine with Steam installed,
    `discovery list`, `discovery list --json` and `discovery list --steam-root <non-default
    install>` are pasted into `03-build-notes.md` with the machine's real `libraryfolders.vdf`
    location noted; the check covers the default install, any secondary library, the `\\?\`
    prefix in `install_path`, a non-default install path from the registry, and that the roots
    run yields that install's `steam-windows` rows — the only gate for §8's roots branch.

## Rollout

1. Land the refactor alone first: `steam_common.rs` plus the `steam_linux.rs` rewiring to
   `SteamCore<LinuxTarget>`, with `lib.rs` gaining the private module. No behavior and no
   public surface change; *Test strategy* #9 is this commit's gate.
2. Land `steam_windows.rs` (plugin, `WindowsTarget`, `host_candidates`, rule wrappers),
   the `[target.'cfg(windows)'.dependencies]` table, and the new tests; commit the
   `Cargo.lock` diff (`winreg`, `cfg-if`) with it. No CLI change yet, so the CLI tests are
   untouched by this commit too.
3. Land the `discovery-cli` wiring (§8) and *Test strategy* #10.
4. Add the cross-`cargo check` step to the `rust` job (.github/workflows/check.yml), run
   all gates locally, and run `cargo check --target x86_64-pc-windows-gnu` once locally (the
   lone gate for that *Compatibility matrix* row). Strengthening the CI step to
   `cargo clippy --target x86_64-pc-windows-msvc` is a one-line follow-up, deliberately not
   taken in the same change as the dependency addition.
5. Manual gate for the Done-when clauses no Linux runner can answer — the real-install check
   (:72-74) and §8's `#[cfg(windows)]` roots branch (:65-67): *Test strategy* #13, recorded in
   `03-build-notes.md` with any A17/A18/A19 surprise (an unreadable HKLM key, a
   `REG_EXPAND_SZ` value, or a `config/libraryfolders.vdf`-only install), each of which
   opens the matching *Deferred* item as its own task.
6. Defaults on merge: `discovery list` probes both platforms; there is no new flag,
   environment variable, feature flag, config file or migration, and Linux output is
   unchanged. Rollback is reverting the commits.

## Deferred / out of scope

- A second `libraryfolders.vdf` probe at `<root>/config/` (A19): no follow-up yet; it is a
  shared-core behavior change, so it needs its own task that decides the Linux side too.
- `REG_EXPAND_SZ` expansion (§4): no follow-up yet; reopens only if *Rollout* #5 observes
  the type on a real install.
- Registry-key coverage beyond the two keys — `InstallPath`/`SteamPath` aliases,
  `BaseInstallFolder_N` legacy entries, per-user `HKU\<SID>` hives: no follow-up yet.
- Path-namespace normalization for `install_path` (§7's `\\?\` prefix): no follow-up yet;
  belongs to the first consumer that opens the path.
- `installdir` hardening beyond the mandated `\` refusal — reserved device names, trailing
  dots/spaces, ADS colons (https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file):
  no follow-up yet.
- Validating `install_path` before a codec writes to it: the codec's own design (carried
  from docs/designs/tasks/001-steam-linux-discovery/02-proposal.md).
- Non-game filtering, `StateFlags` readiness, slug migration, AppID use, persistence,
  `DiscoveryError` taxonomy and engine-side error collection: unchanged from
  docs/designs/tasks/001-steam-linux-discovery/02-proposal.md → *Deferred / out of scope*.
- A Windows CI runner, `x86_64-pc-windows-gnu` CI coverage, and cross-target clippy:
  no follow-up yet.
- Steam Deck / SteamOS and macOS layouts: ADR-022 excludes them.
- `crates/discovery/Cargo.toml` → `version.workspace = true`: chore; no follow-up yet.

## Revision log

- r0: q1 registry crate and seam — chose `winreg` target-gated behind a
  `&dyn Fn(RegistryKey, &str)` seam beside the `&dyn Fn` env lookup; fake in the test file; *§1, §3, §4*.
- r1: view ordering in the seam (supersedes r0: q2) — order kept (HKCU first, HKLM before the
  always-appended env defaults) as `…ValveSteam32/64` variants `host_candidates` orders in pure
  tested code, one (key, view) per reader call, both views answering pinned; *§1, §3–§5, #3*.
- r0: q3 decomposition and the error-message tag — `SteamCore<T>` over a sealed
  `SteamTarget`; `steam_linux`'s `pub` surface kept by wrappers; prefix is `T::NAME` = `name()`; *§1, §2*.
- r0: q4 `libraryfolders.vdf` location — kept `steamapps/`-only, since a second probe is
  shared-core behavior and Linux must not change; *§2, Risks, Deferred*.
- r0: q5 emitted-path namespace — `\\?\` canonical paths shipped as-is for the long-path
  guarantee; no normalizer; *§7, Risks, Compatibility matrix, Deferred*.
- r0: q6 `installdir` hardening — the `\` refusal is the whole Windows addition,
  target-parameterized so Linux keeps its rules; device names and dots deferred; *§6, Deferred*.
- r0: q7 `REG_EXPAND_SZ` — the seam surfaces `REG_SZ` only, type read via
  `get_raw_value().vtype`; no expansion call; *§3, §4, Deferred*.
- r1: `--steam-root` roots branch (supersedes r0: q8) — `#[cfg(windows)]` feeds the flag to the
  plugin of the compiled-for host (task :65-67), Linux roots branch and CLI tests unchanged; #12
  only type-checks it, so #13's `--steam-root` run (*Rollout* #5) is its named gate; *§8, #13*.
- r1: A1's backslash spelling (split out as A1b) — the doubled-backslash claim rested on a dead
  citation; no spelling is assumed and A2's unescape covers either; *Assumption register*.
- r0: gating shape — only the registry reads, `from_host()` and the CLI roots branch (§8) are
  `#[cfg(windows)]`, so the plugin and its policy stay testable on Linux as the task requires;
  *§1, §3, §8, Test strategy #12*.

## As-built (2026-11-18)

Recorded against the change set `d424210…5a08426` (implementation `cd8ecf6e`,
tests `3ea3c08c`). The design above is unchanged; this section records only where
what was built differs from or goes beyond it. Gate detail is in
`03-build-notes.md`.

1. **The Rollout increments landed as one commit** — drift: *Rollout* #1–#4
   prescribed four sequential landings (refactor alone, then the plugin plus
   dependency and lock, then the CLI wiring, then the CI step). Reason: the
   pipeline's implement stage is one increment and the runner owns commits; the
   per-increment gate (*Test strategy* #9, the untouched suites) was still run
   against the whole change, and rollback stays a revert. Affected: *Rollout*
   #1–#4. The tests shipped in a preceding `test:` commit, before the
   implementation commit.
2. **Additive public-surface growth beyond §1's sketches** — drift: `steam_linux`
   gained `pub struct LinuxTarget` (the mirror of §1's `pub struct WindowsTarget`,
   needed because the core is parameterized); `steam_windows` carries
   `pub use crate::steam_common::AppManifest;` although §1 names the re-export only
   for `steam_linux`; and `SteamCore`, `SteamLinuxPlugin` and `SteamWindowsPlugin`
   derive `Debug, Clone` (the Linux plugin's pre-existing derives, mirrored).
   Reason: symmetry and not removing capability Linux already had. Nothing was
   removed or re-signatured, so A15's fixed point holds — `steam_linux_tests.rs`,
   `plugin_tests.rs` and `cli_tests.rs` compile and pass with no edited line.
   Affected: §1.
3. **`registry_string_value` decodes `REG_SZ` by hand** — drift: after the
   `vtype == REG_SZ` gate it does its own UTF-16LE decode (odd byte length →
   `None`, trailing NUL units trimmed, unpaired surrogates → `None`) rather than
   delegating the decode. Reason: §4's type gate rules out winreg's
   `get_value::<String>`, which also accepts `REG_EXPAND_SZ` and `REG_MULTI_SZ`
   (A9, A18); the design fixed the refusal set, not the decoder. Behavior is
   within §4's stated rule set. Affected: §3, §4.
4. **Registry candidates are not absolute-path-gated** — no drift from the text
   (§5 gates only the two environment candidates through `absolute_env` and appends
   registry values verbatim), recorded because code review S-1 (minor) flagged the
   asymmetry against the *Risks* row "a corrupt or hostile registry value makes the
   plugin scan somewhere unexpected". Unchanged: §2/§5's
   canonicalize-and-require-`steamapps` rule is the design's accepted mitigation.
   Affected: §5, *Risks*.
5. **Test strategy #1's NUL spelling is unpinned** — drift: the parity loop and the
   Windows rule-set list carry `""`/`.`/`..`/`/abs/path`/`sub/dir` but no interior
   NUL, mirroring the Linux suite at `crates/discovery/tests/steam_linux_tests.rs:113`
   that #1 describes and that likewise has no NUL case. Reason: #1's parenthetical
   overstates the existing Linux suite; the parity claim itself is intact.
   `steam_common::usable_install_dir` does refuse `\0`, but the clause is untested
   on either platform (code review correctness F1 / testing #1, both minor, open).
   Affected: *Test strategy* #1–#2.
6. **#11's toolchain leg ran on a different compiler** — drift: the Linux gate trio
   was executed with the host's cargo 1.100.0, not the pinned 1.85.0 channel,
   because the stage shell has no `rustup`. Reason: environment limitation, not a
   design change; CI still runs the three commands on the pinned toolchain.
   Affected: *Test strategy* #11, A13.
7. **#12's cross-target compile leg produced no evidence** — drift: neither
   `cargo check --target x86_64-pc-windows-msvc` (Test strategy #12) nor the
   local `x86_64-pc-windows-gnu` run (*Rollout* #4) could execute in the stage
   environment (target `std` absent, `rustup target add` outside the stage
   allowlist). Reason: same environment limitation; the §9 CI step is now the gate,
   and `registry_string_value`'s winreg calls were instead checked against the
   winreg 0.56.0 sources. Consequence: §3's gated code, §8's
   `#[cfg(windows)]` roots branch and the *Compatibility matrix*'s `msvc`/`gnu`
   rows are compile-unverified until that CI leg runs green. Affected: §9,
   *Test strategy* #12, *Rollout* #4, *Compatibility matrix*.
8. **#13's manual Windows gate was not run** — drift: no Windows machine was
   available, so the real registry probe, the `\\?\`-prefixed `install_path` form
   (§7) and §8's roots branch remain unverified at runtime; A17, A18, A19 and A20
   stay unverified. The gate stays open in `03-build-notes.md` rather than being
   closed by inspection. Affected: §7, §8, *Test strategy* #13, *Rollout* #5.
9. **Test strategy #10 landed as a new `apps/discovery-cli/tests/cli_dual_plugin_tests.rs`**
   — drift in form only: a subprocess `discovery list --json` run with `env_clear()`
   and a nonexistent `HOME` (exits 0, prints `[]`, nothing on stderr) plus an
   in-process engine test with the populated Linux plugin beside
   `SteamWindowsPlugin::from_roots([])`. The registration *order* is not asserted —
   it is unobservable while the Windows plugin yields no rows — so it rests on #12's
   compile check and #13. Reason: keeping both plugins' probe hermetic on a Linux
   runner without touching `cli_tests.rs` (Test strategy #9). Correctness review F3
   (nit) notes the subprocess test would read a real registry if it were ever run on
   a Windows host. Affected: §8, *Test strategy* #10.
