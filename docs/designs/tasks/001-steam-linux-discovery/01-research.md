# Research: Linux Steam discovery plugin (T-001)

Source: `docs/tasks/T-001-steam-linux-discovery.md` (task, `associated_adr: ADR-022`, `depends_on: []`,
no `flow` key → full flow). Lenses applied: **Prior art**, **Platform notes**, and **Security landscape**
(the last applied under *Risks & unknowns*, because the artifact template has no security heading).
This is a fresh pass: every cited URL below was fetched and re-read during this run, and repo statements
cite files read here.

## Summary

T-001 asks for a `SteamLinuxPlugin` in `crates/discovery` that reads library folders from
`libraryfolders.vdf`, parses `appmanifest_*.acf` under each library's `steamapps/`, and emits
`DiscoveredGame` values tagged `platform = SteamLinux` with `codec_id = None`
(source: docs/tasks/T-001-steam-linux-discovery.md:15-19). Nothing in the existing contract has to change:
the two-method plugin trait, the five-field `DiscoveredGame` with a `SteamLinux` variant, the
`normalize_slug` helper and the error-swallowing engine all already exist, and the CLI registers only a
`DummyPlugin` today (source: crates/discovery/src/plugin.rs:4-7, crates/discovery/src/game.rs:5-27,
crates/discovery/src/slug.rs:1-35, crates/discovery/src/engine.rs:20-33,
apps/discovery-cli/src/main.rs:25-41). External evidence converges on the format: an ACF is text KeyValues
whose first pair is `"AppState"`, carrying `appid`, `name`, `StateFlags` and `installdir`, and the game
directory is `<library>/steamapps/common/<installdir>`
(source: https://github.com/leovp/steamfiles/blob/master/docs/acf_overview.rst,
https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs). The open design
decisions are not about parsing but about selection policy and edges: which Steam roots count, whether
Proton and runtime manifests belong in the game list, what to do when `name` is absent, and how to test a
filesystem plugin hermetically when the existing CLI test asserts an empty result
(source: https://docs.rs/steamlocate/latest/steamlocate/,
https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs,
apps/discovery-cli/tests/cli_tests.rs:14-23). The dependency question has a visible precedent: several
current Rust Steam scanners hand-roll a small KeyValues reader instead of taking `keyvalues-parser` +
`pest` (source: https://docs.rs/fragcap-steam/latest/fragcap_steam/vdf/index.html,
https://docs.rs/steamie-core/latest/steamie_core/vdf/index.html).

## Findings

### What already exists in the repo (the plugin is additive)

- The plugin contract is exactly `name(&self) -> &'static str` and
  `discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError>`; a `SteamLinuxPlugin` implements it with
  no trait change (source: crates/discovery/src/plugin.rs:4-7).
- `Platform::SteamLinux` already exists and renders as `steam-linux`; `DiscoveredGame` has precisely the
  five fields ADR-022 specifies: `slug`, `display_name`, `install_path`, `platform`, `codec_id`
  (source: crates/discovery/src/game.rs:5-27, docs/adrs/022-game-discovery.md:29-36).
- `normalize_slug` already implements ADR-022's provisional rule — lowercase, every non-`is_alphanumeric`
  char → `-`, collapse repeats, trim `-`, prefix `unknown/`. Because the test is `char::is_alphanumeric`,
  non-ASCII titles survive into the slug (source: crates/discovery/src/slug.rs:1-35,
  docs/adrs/022-game-discovery.md:54-55).
- `DiscoveryEngine::discover_all` deliberately swallows a plugin's `Err`, prints
  `eprintln!("Error from plugin {}: {:?}", …)` and still returns `Ok` — so "no Steam here" must be a
  non-fatal outcome, and a plugin's error payloads are only ever visible as debug text (source:
  crates/discovery/src/engine.rs:20-33). Results from plugins are only `append`ed: no dedupe, no sort
  (source: crates/discovery/src/engine.rs:21-32).
- `DiscoveryError` is `PluginError(String)` / `RegistryError(String)` with `Debug`/`Clone`/`PartialEq` only
  — no `Display`, no `std::error::Error`, no taxonomy distinguishing "not installed" from "unreadable"
  from "malformed" (source: crates/discovery/src/registry.rs:1-5).
- `discovery-cli` registers only `DummyPlugin` (which returns `Ok(vec![])`) and prints a pipe-delimited
  table, or a `serde_json` array under `--json`; enabling a real plugin is one added
  `engine.register_plugin(..)` line in the `Commands::List` arm (source: apps/discovery-cli/src/main.rs:25-58).
- The crate currently exports five modules (`engine`, `game`, `plugin`, `registry`, `slug`) and depends on
  `serde` alone; the CLI adds `clap 4` and `serde_json 1.0` (source: crates/discovery/src/lib.rs:1-5,
  crates/discovery/Cargo.toml:6-7, apps/discovery-cli/Cargo.toml:6-10).
- `Cargo.lock` contains 32 packages, all of them this workspace's two library crates, `discovery`,
  `discovery-cli`, and the `clap`/`serde`/`serde_json` closures. There is no VDF/KeyValues parser, no `pest`,
  no `dirs`, no `walkdir` and no `tempfile` anywhere in the lock file (source: Cargo.lock).
- The two discovery packages hard-code `version = "0.1.0"` / `edition = "2024"`, while
  `crates/transforms` (and per the workspace comment `crates/core`) use `version.workspace = true` etc.;
  the workspace comment states third-party deps "enter when their consumer lands" (source:
  crates/discovery/Cargo.toml:1-4, apps/discovery-cli/Cargo.toml:1-4, crates/transforms/Cargo.toml:4-7,
  Cargo.toml:1-3, Cargo.toml:20-23).
- Toolchain floor is `rust-version = "1.85"` / `edition = "2024"` with the toolchain pinned to `1.85.0`, and
  the workspace globs are `["crates/*", "apps/*"]` (source: Cargo.toml:12-18, rust-toolchain.toml:6-7).
- Test layout in the crate is one `<module>_tests.rs` per module under `crates/discovery/tests/`
  (`game_tests.rs`, `plugin_tests.rs`, `registry_tests.rs`, `slug_tests.rs`) — no fixtures directory, no
  dev-dependencies, so a steam-linux test file needs a fixture decision (source: crates/discovery/tests/,
  crates/discovery/Cargo.toml).
- The existing CLI tests shell out to `cargo run -p discovery-cli -- list` and assert the table header, and
  that `list --json` prints exactly `[]`; both become machine-dependent the moment a real plugin is
  registered (source: apps/discovery-cli/tests/cli_tests.rs:3-23).

### The two files the plugin parses

- `appmanifest_<app_id>.acf` is plain-text KeyValues; "the first key-value pair of an ACF file is
  `AppState`, which has the root section as a value", sections are brace-delimited, and string values follow
  their key separated by tabs; a real CS:GO manifest carries `appid`, `Universe`, `name`, `StateFlags`,
  `installdir`, `LastUpdated`, `UpdateResult`, `SizeOnDisk`, `buildid`, `LastOwner`, `AutoUpdateBehavior`,
  plus nested `UserConfig` and `MountedDepots` (source:
  https://github.com/leovp/steamfiles/blob/master/docs/acf_overview.rst).
- `name` is optional in practice: the mature Rust implementation types it `name: Option<String>` and its
  `minimal` parse fixture is a manifest carrying only `appid` and `installdir` (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs).
- `installdir` is a directory *name*, not a path: `Library::resolve_app_dir` composes exactly
  `library.join("steamapps").join("common").join(install_dir)`, and its doc example asserts
  `steamapps/common/Graveyard Keeper` (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs).
- `libraryfolders.vdf` is KeyValues rooted at `"libraryfolders"`, with libraries keyed by decimal index
  (`"0"`, `"1"`, …) whose block carries `path`, `label`, and an `apps` map of appid → size; the reference
  implementation keeps only blocks whose key parses as `u32` and reads `path` from each (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs).
- That `apps` map is deliberately not trusted: the reference `Library::from_dir` instead enumerates
  `<library>/steamapps/` for `appmanifest_<id>.acf` filenames, with the comment "since the values in
  `libraryfolders.vdf` may be stale" (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs).
- A second scanner uses the opposite inclusion rule and is explicit about determinism: it treats the Steam
  root itself as a library *in addition to* every `path` in `steamapps/libraryfolders.vdf`, then
  `paths.sort(); paths.dedup();` (source:
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html).
- `StateFlags` is a bit field documented by Lutris: `StateUninstalled 1`, `StateUpdateRequired 2`,
  `StateFullyInstalled 4`, `StateFilesMissing 32`, `StateValidating 131072`, `StateDownloading 1048576`, etc.
  A captured manifest with `"StateFlags" "6"` parses to `UpdateRequired | FullyInstalled`, i.e. a bit-test
  on 4 admits some not-ready installs while `StateFlags == 4` does not (source:
  https://raw.githubusercontent.com/lutris/lutris/master/docs/steam.rst,
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs).
- Key case is not stable in Steam's own files, so every hand-rolled reader examined looks keys up
  case-insensitively: one reader documents "Key lookups are case-insensitive (Steam mixes `installdir` and
  `StateFlags`)", another's fixture asserts `INSTALLDIR` and `stateflags` resolve, and a third notes
  `"AppState"`, `"appid"`, `"StateFlags"` are case-inconsistent (source:
  https://docs.rs/steamie-core/latest/steamie_core/vdf/index.html,
  https://docs.rs/yandere-core/latest/src/yandere_core/vdf.rs.html,
  https://github.com/frankea/Whisky/blob/e51bd9f6/WhiskyKit/Sources/WhiskyKit/Steam/VDFParser.swift).
- VDF text has no authoritative grammar the discovery path can lean on: the maintained parser states "VDF
  text is drastically underspecified", stores pairs in a `BTreeMap` so declaration order is lost, renders an
  empty value sequence as a missing pair, and models top-level `#base` directives in a separate
  `PartialVdf` type (source: https://docs.rs/keyvalues-parser/latest/keyvalues_parser/).
- Lenient readers in this space also tolerate `//` line comments, bare (unquoted) tokens, a leading UTF-8
  BOM, `[$WIN32]`-style conditionals, and depth limits — features a hand-rolled reader must decide about
  explicitly rather than by accident (source:
  https://docs.rs/fragcap-steam/latest/src/fragcap_steam/vdf.rs.html,
  https://pkg.go.dev/github.com/ApolloF/gamekit/vdf).

### Behaviour the "Done when" list implies but does not settle

- Non-game apps own appmanifests too: the reference crate's own sample Linux output lists, under two
  libraries, `App 1628350 - Steam Linux Runtime 3.0 (sniper)`, `App 1493710 - Proton Experimental` and
  `App 2348590 - Proton 8.0` alongside real games — so a literal "parse every `appmanifest_*.acf`" plugin
  puts Proton builds into the BindForge game list (source: https://docs.rs/steamlocate/latest/steamlocate/).
- Existing tools handle that with heuristics rather than facts: one scanner "filters out non-game entries
  (Proton runtimes, Steam Linux Runtime, redistributables, SDKs, dedicated servers, uploaders) …
  heuristically", and a `libraryfolders.vdf` fixture in another project shows the redistributable appid
  `228980` sitting in a normal library's `apps` map (source: https://github.com/Eloreden/protonaut,
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html).
- `libraryfolders.vdf` is not written identically everywhere: a maintainer abandoned serde for it because
  "the file has two historical shapes — modern (`"0" { "path" "…" "apps" { … } }`) and legacy pre-2021
  (`"1" "D:\\SteamLibrary"`) — and a `#[serde(untagged)]` enum spanning both matched neither on real files,
  silently yielding zero libraries", which caused games on secondary drives to never be found (source:
  https://docs.rs/aurelia-steam/latest/aurelia/library/fn.library_folder_paths.html).
- Copies of `libraryfolders.vdf` exist outside the Steam root. A user report lists many under per-game
  Proton prefixes, e.g. `<library>/steamapps/compatdata/208140/pfx/drive_c/Program Files (x86)/Steam/steamapps/libraryfolders.vdf`,
  noting they "have a different format and seem to be for library paths relative to `Z:\`" — a recursive
  search, or a prefix directory treated as a library root, is a live mis-discovery path (source:
  https://github.com/ValveSoftware/steam-for-linux/issues/8653).
- Which copy of the file is authoritative is unresolved in the wild: the same report shows two separate
  files, `<steam>/config/libraryfolders.vdf` and `<steam>/steamapps/libraryfolders.vdf`, "two separate files
  with different inode numbers, not hard links … I have no idea which one is actually used", both rewritten
  at Steam start; the reference Rust implementation reads only the `steamapps/` one, and the
  `debian-installation` client root appears in the same thread and in the reference locator's candidate list
  (source: https://github.com/ValveSoftware/steam-for-linux/issues/8653,
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/locate/linux.rs).
- Rust's `ReadDir` does not guarantee entry order, so a directory-scan plugin needs an explicit sort if the
  CLI table and the tests are to be stable (source: https://doc.rust-lang.org/std/fs/struct.ReadDir.html).

## Prior art

- **`steamlocate-rs` 2.1.1 (MIT, 16 Oct 2026)** is the closest comparable and confirms the exact pipeline
  T-001 describes: locate Steam dir → `libraryfolders.vdf` via `keyvalues-parser`/`keyvalues-serde` →
  enumerate each `steamapps/` for `appmanifest_*.acf` → per-manifest struct → `resolve_app_dir`. Its
  dependencies are `crc`, `keyvalues-parser ^0.2`, `keyvalues-serde ^0.2`, `serde`, `winreg` (Windows), with
  `insta` for tests (source: https://docs.rs/steamlocate/latest/steamlocate/,
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs).
  Two of its weaknesses are instructive for a design that must not lose a whole scan: `parse_library_paths`
  returns `Err` for the entire call if any single indexed block lacks a usable `path`, and
  `Library::from_dir` errors if `read_dir(<library>/steamapps)` fails; both file kinds are read with
  `fs::read_to_string`, so non-UTF-8 bytes abort them (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs,
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs).
- **Two independent Linux scanners that include the Steam root as a library** (`steamie_core::steam_apps`)
  also carry `STATE_FULLY_INSTALLED: u32 = 4` and an `is_fully_installed()` bit test on every
  `InstalledApp`, i.e. a published precedent for both the root-inclusion rule and the state filter that
  BindForge's task text leaves open (source:
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html).
- **Won by convention (parsing).** For the grammar question, several current Rust projects that read exactly
  these two files hand-roll a small reader instead of taking `keyvalues-parser` + `pest`:
  `fragcap_steam::vdf` states "Steam records library locations in `libraryfolders.vdf` and per-title
  metadata in `appmanifest_<app_id>.acf`, both in this format … the format is small and stable and not worth
  a dependency", covering quoted/bare keys, quoted values, `{}` nesting, `//` comments and `\\`/`\"`
  escapes, with a positioned error "so a discovery caller can report the offending file and skip it while
  the well-formed files survive"; `steamie_core::vdf` is "intentionally tiny: enough to read install
  metadata, no more … Pure → unit-tested against real captured files"; `yandere_core::vdf` is the same
  pattern with line-numbered errors (source:
  https://docs.rs/fragcap-steam/latest/fragcap_steam/vdf/index.html,
  https://docs.rs/fragcap-steam/latest/src/fragcap_steam/vdf.rs.html,
  https://docs.rs/steamie-core/latest/steamie_core/vdf/index.html,
  https://docs.rs/yandere-core/latest/src/yandere_core/vdf.rs.html).
- **The dependency-side alternative is real and current.** `keyvalues-parser 0.2.4` (27 Sep 2026,
  MIT OR Apache-2.0, `pest ^2.7` as its only non-optional dependency, developed at
  codeberg.org/CosmicHarper/vdf-rs) is the parser `steamlocate` uses; its own docs list the lossy
  representation caveats quoted above (source: https://docs.rs/keyvalues-parser/latest/keyvalues_parser/).
- **Escape handling diverges in practice.** One hand-rolled reader unescapes `\\` and `\"` and asserts
  `"a\\b\"c"` → `a\b"c`; a second asserts that the Windows paths in a real `libraryfolders.vdf` come out as
  `C:\Program Files (x86)\Steam` from the doubled-backslash source; a third (Whisky's Swift parser) calls
  `\\` and `\"` "the two escapes Valve emits"; and Node's long-used `vdf` package has an open issue titled
  "Backslash escaping is not handled" (source:
  https://docs.rs/fragcap-steam/latest/src/fragcap_steam/vdf.rs.html,
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html,
  https://github.com/frankea/Whisky/blob/e51bd9f6/WhiskyKit/Sources/WhiskyKit/Steam/VDFParser.swift,
  https://github.com/node-steam/vdf/issues/14).
- **Legacy-shape handling is a design axis of its own**: `aurelia::library::library_folder_paths` is
  hand-rolled precisely to span the modern and pre-2021 `libraryfolders.vdf` shapes after a serde attempt
  silently returned zero libraries (source:
  https://docs.rs/aurelia-steam/latest/aurelia/library/fn.library_folder_paths.html).
- **Non-game filtering has published precedent, all heuristic**: `Eloreden/protonaut` parses
  `libraryfolders.vdf` then each `appmanifest_*.acf` and filters non-game entries heuristically (source:
  https://github.com/Eloreden/protonaut), as does the reference crate's sample output, which simply shows
  the noise a filter must remove (source: https://docs.rs/steamlocate/latest/steamlocate/).
- **BindForge's own prior art is ADR-022**: provisional `unknown/<kebab(display_name)>` slug, the two-method
  plugin trait, "No AppID usage, no third-party sync. No persistence", v1 scoped to Steam on Linux and
  Windows with no Steam Deck, and a bootstrap CLI that prints slug/display_name/install_path/platform
  (source: docs/adrs/022-game-discovery.md:21-23, 37-55).

## Platform notes

- **Linux root is a candidate list, not a path.** The reference Linux locator probes, in order:
  `~/.var/app/com.valvesoftware.Steam/{.local/share/Steam,.steam/steam,.steam/root}` (Flatpak),
  `$XDG_DATA_HOME/Steam` (defaulting to `~/.local/share/Steam`), `~/.steam/steam`, `~/.steam/root`,
  `~/.steam/debian-installation`, and `$SNAP_USER_DATA/steam/common/{.local/share/Steam,.steam/steam,.steam/root}`
  (default `~/snap/…`), keeping a candidate only when `<candidate>/steamapps` is a directory and de-duping
  by `read_link()` so symlink aliases (`~/.steam/root` → the real root) collapse. `XDG_DATA_HOME` and
  `SNAP_USER_DATA` are read from the environment rather than hard-coded, and the crate exposes both
  `locate()` (one installation) and `locate_all()` (all installations) (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/locate/linux.rs,
  https://docs.rs/steamlocate/latest/steamlocate/).
- **Flatpak changes what a host-side CLI can read.** By default a Flatpak app has "No access to any host
  files except the runtime, the app, `~/.var/app/$FLATPAK_ID`, and `$XDG_RUNTIME_DIR/app/$FLATPAK_ID`", and
  external drive mounts need explicit `--filesystem=/media`, `--filesystem=/run/media` or `--filesystem=/mnt`
  grants. Consequence for BindForge: a Flatpak `libraryfolders.vdf` may name library paths that exist only
  inside the sandbox, or paths the discovery user cannot stat, so `install_path` can be unresolvable on the
  host (source: https://docs.flatpak.org/en/latest/sandbox-permissions.html).
- **Libraries legitimately live on arbitrary mounts.** The fixtures other projects test against include
  `/run/media/ray/Games/SteamLibrary` and `/home/ray/Gamemount/SteamLibrary` next to
  `/home/ray/.local/share/Steam`, and a multi-library reporter keeps libraries under `/var/games/steam*` on
  ZFS datasets — i.e. no "under $HOME" assumption is safe (source:
  https://docs.rs/yandere-core/latest/src/yandere_core/vdf.rs.html,
  https://github.com/ValveSoftware/steam-for-linux/issues/8653).
- **Case sensitivity.** Every example manifest spells the root `"AppState"` and the task keys `appid` /
  `installdir` / `name` / `StateFlags`, but the case of the same keys varies across files
  (`INSTALLDIR`, `stateflags`, `universe` vs `Universe`), so lookups must be case-insensitive even though
  the Linux *filesystem* is case-sensitive; whether a legacy `SteamApps` directory spelling must be
  accepted is unresolved — see Open questions (source:
  https://docs.rs/yandere-core/latest/src/yandere_core/vdf.rs.html,
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs,
  https://docs.rs/steamie-core/latest/steamie_core/vdf/index.html).
- **Toolchain constraints on the dependency choice.** The workspace floor is Rust 1.85 / edition 2024 with
  the toolchain pinned at 1.85.0 and `Cargo.lock` committed (`cargo test --workspace --locked` is a gate),
  so a new parser crate must build on that floor and its full transitive tree (`pest` and its support
  crates) lands in the lock file, on top of today's 32-package closure (source: Cargo.toml:14-18,
  rust-toolchain.toml:6-7, Cargo.lock, AGENTS.md).
- **Consumer surface.** The CLI prints `| Slug | Display Name | Install Path | Platform |` rows and a JSON
  array under `--json`, and `DiscoveredGame` already derives `Serialize`, so added fields or changed slug
  rules are immediately visible to whatever consumes that output (source: apps/discovery-cli/src/main.rs:45-58,
  crates/discovery/src/game.rs:20-27).
- **Scopes that stay out**: ADR-022 limits v1 to Steam on Linux and Windows and excludes Steam Deck, and
  forbids AppID in the data model — which is what makes a non-game filter and a per-appid denylist awkward
  (source: docs/adrs/022-game-discovery.md:21-23, 49).

## Risks & unknowns

Security landscape applied in this section (the artifact template has no security heading).

- **The trust boundary is the filesystem, and the emitted value is a hand-off.** `libraryfolders.vdf`'s
  `path` and a manifest's `installdir` are composed into `install_path` verbatim by every implementation
  surveyed (`library.join("steamapps").join("common").join(install_dir)`, and the same composition in the
  second scanner), with no validation of either. Discovery is read-only, but a later codec that writes to
  `install_path` inherits `..`/absolute-path values from a file anyone writable can edit (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs,
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html).
- **Escaping is a correctness trap, not a nicety**: the same field is unescaped two different ways by two
  current readers, and a widely used parser has an open "Backslash escaping is not handled" issue; on Linux
  this bites titles containing `\` and Windows-authored values inside shared libraries (source:
  https://docs.rs/fragcap-steam/latest/src/fragcap_steam/vdf.rs.html,
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html,
  https://github.com/node-steam/vdf/issues/14).
- **Whole-scan blast radius.** With the reference implementation, one indexed library block lacking a usable
  `path` fails `parse_library_paths` entirely, and one unreadable `steamapps` directory fails that library;
  combined with `discover_all` swallowing the plugin's `Err` into one `eprintln!`, a single stale or
  hand-edited file yields zero games rather than a degraded list (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs,
  crates/discovery/src/engine.rs:20-33).
- **Non-UTF-8 and I/O errors abort per-file reads** where `fs::read_to_string` is used; per-file
  skip-and-continue is the pattern the hand-rolled readers adopt ("report the offending file and skip it
  while the well-formed files survive") (source:
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs,
  https://docs.rs/fragcap-steam/latest/src/fragcap_steam/vdf.rs.html).
- **Duplicate and repeated entries are unhandled in this repo.** `discover_all` only `append`s, ADR-022
  forbids AppID in the model, and the two published inclusion rules differ (file-listed paths vs
  root-plus-paths with `sort()`/`dedup()`), so "root also appears as library `0`" can double-scan a library
  and surface identical `unknown/<slug>` rows for the same path (source: crates/discovery/src/engine.rs:21-32,
  docs/adrs/022-game-discovery.md:49,
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html).
- **Non-game manifests are a product-visible defect, not cosmetic**: `codec_id` is `None` for everything and
  `DiscoveredGame` has no kind field, so Proton builds and the Steam Linux Runtime appear as bindable games
  (source: https://docs.rs/steamlocate/latest/steamlocate/, crates/discovery/src/game.rs:20-27).
- **Missing `name` breaks the ADR's slug rule.** ADR-022 derives the slug from the display name and the task
  says only "extract the game `name` and `installdir`", yet `name` is `Option` in the reference model and a
  real manifest can carry just `appid` + `installdir`; one current scanner falls back to an empty name,
  which would feed `unknown/` (source: docs/adrs/022-game-discovery.md:23,
  docs/tasks/T-001-steam-linux-discovery.md:17-19,
  https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs,
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html).
- **The Done-when list collides with the existing test.** "The `discovery-cli` can successfully run the
  `SteamLinuxPlugin`" cannot hold together with `list --json` == `[]` on a machine that has Steam, and both
  CLI tests launch through `cargo run`, inheriting the developer's machine as test input; an override seam
  (explicit root argument or env var) or a test change is required (source:
  apps/discovery-cli/tests/cli_tests.rs:3-23, docs/tasks/T-001-steam-linux-discovery.md:22-24).
- **Diagnostic leakage.** Anything the plugin embeds in `DiscoveryError::PluginError(String)` is echoed to
  stderr via `{:?}` by the engine, mixing host paths into the same stream a `--json` consumer sees (source:
  crates/discovery/src/engine.rs:25-29, apps/discovery-cli/src/main.rs:45-46).
- **Proton prefix trees are look-alike data**: per-game prefixes contain `…/Steam/steamapps/libraryfolders.vdf`
  copies in a different format; a plugin that globs recursively, or that treats a `compatdata/<appid>/pfx`
  path as a root, will mis-discover (source: https://github.com/ValveSoftware/steam-for-linux/issues/8653).
- **No performance baseline exists for this scan shape.** Published facts stop at "read these small text
  files directly — no Steam process, no network" and ADR-022's accepted cost "repeated probes re-scan the
  filesystem"; no measured latency or throughput figure for a Steam library scan was found, so any budget
  in the design will be an internal one (source:
  https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html,
  docs/adrs/022-game-discovery.md:75-78).
- **Unverifiable-in-run claims**: the Valve KeyValues wiki page
  (https://developer.valvesoftware.com/wiki/KeyValues), long cited as the format's informal definition,
  sits behind a bot-check and could not be read in this pass, so this document states the grammar only from
  parsers and captured fixtures above.

## Open questions

1. **Root policy.** Which candidate set does `SteamLinuxPlugin` probe — native only, or native + Flatpak +
   Snap + `$XDG_DATA_HOME` — and are all valid roots scanned (the reference crate's `locate_all()`) or only
   the first? If Flatpak roots are scanned, do sandbox-only library paths get reported, skipped, or
   reported-and-flagged? (source: https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/locate/linux.rs,
   https://docs.flatpak.org/en/latest/sandbox-permissions.html)
2. **Testability seam.** Should the plugin take an explicit Steam root (constructor argument, plus a
   `--steam-root` CLI flag or env var) so crate tests and the CLI tests run against a fixture tree with no
   Steam installed? Without one, `apps/discovery-cli/tests/cli_tests.rs` must change (source:
   apps/discovery-cli/tests/cli_tests.rs:3-23).
3. **Fixture strategy.** Checked-in `.acf` / `libraryfolders.vdf` samples under `crates/discovery/tests/`
   (the pattern other projects use: "unit-tested against real captured files") versus building a temp tree
   at test time, which needs a `tempfile` dev-dependency the workspace does not carry today (source:
   https://docs.rs/steamie-core/latest/steamie_core/vdf/index.html, Cargo.lock,
   crates/discovery/Cargo.toml).
4. **Dependency posture.** Hand-rolled depth-aware KeyValues reader with no new dependency (the choice
   several current Rust scanners documented, and the one that keeps the 32-package `Cargo.lock` unchanged)
   versus `keyvalues-parser` (+`keyvalues-serde`) with `pest` added — and if a crate is added, does it enter
   `[workspace.dependencies]` per the existing "enter when their consumer lands" convention? (source:
   https://docs.rs/fragcap-steam/latest/fragcap_steam/vdf/index.html,
   https://docs.rs/keyvalues-parser/latest/keyvalues_parser/, Cargo.toml:20-23)
5. **Grammar subset to commit to.** Which of `//` comments, bare/unquoted tokens, duplicate keys, `#base`,
   BOM, CRLF, and `[$WIN32]`-style conditionals must the reader accept or reject for these two file kinds?
   No statement of exactly what the Linux client writes was found for these (source:
   https://docs.rs/fragcap-steam/latest/src/fragcap_steam/vdf.rs.html,
   https://docs.rs/keyvalues-parser/latest/keyvalues_parser/,
   https://pkg.go.dev/github.com/ApolloF/gamekit/vdf).
6. **Escaping.** Unescape `\\` and `\"` (as `fragcap`/Whisky do) or preserve the literal bytes (as the
   Windows-path fixture test suggests), and where does that rule live so a Windows plugin later can share
   it? (source: https://docs.rs/fragcap-steam/latest/src/fragcap_steam/vdf.rs.html,
   https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html)
7. **Key-case rule.** Adopt case-insensitive key lookup (all surveyed readers do) and does that apply to the
   root key `AppState`/`libraryfolders` as well as to inner keys? (source:
   https://docs.rs/yandere-core/latest/src/yandere_core/vdf.rs.html)
8. **Legacy `libraryfolders.vdf` shape.** Support the pre-2021 `"1" "D:\SteamLibrary"` form, or fail those
   files — and what does the plugin do when the modern parser yields zero libraries, given a real tool has
   already shipped that silent-zero bug (source:
   https://docs.rs/aurelia-steam/latest/aurelia/library/fn.library_folder_paths.html)?
9. **Non-game filtering.** Include every manifest, filter by a name/appid denylist, or filter by
   `StateFlags`/`installdir` heuristics — and where does that policy live given ADR-022's "No AppID usage"
   and the absence of a kind field on `DiscoveredGame`? (source: https://docs.rs/steamlocate/latest/steamlocate/,
   https://github.com/Eloreden/protonaut, docs/adrs/022-game-discovery.md:49)
10. **`StateFlags` semantics.** Emit everything, only installs with the `4` bit set, or only
    `StateFlags == 4`? The two differ for value 6 (`UpdateRequired | FullyInstalled`) — which side of that
    line does the design want, and should it be stated per library or per app? (source:
    https://raw.githubusercontent.com/lutris/lutris/master/docs/steam.rst,
    https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html)
11. **Absent `name`.** Skip the manifest, or fall back to `installdir` as `display_name` (which then also
    feeds the provisional slug), and what is the minimum viable entry (one scanner drops entries with an
    empty `installdir`)? (source: https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs,
    https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html)
12. **Path existence and composition.** Is `install_path` emitted as the composed
    `steamapps/common/<installdir>` regardless of whether it exists, or dropped when the directory is
    missing — and should `installdir` be rejected when it contains a path separator or `..`? Note the
    reference `resolve_app_dir` documents itself as "an unvalidated path" (source:
    https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs).
13. **Failure policy per library and per file.** On a missing `path`, an unreadable `steamapps`, or a
    malformed manifest: skip-and-continue or fail the plugin, and how is a degraded result surfaced given
    the engine prints one debug line? (source:
    https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs,
    crates/discovery/src/engine.rs:20-33)
14. **Which `libraryfolders.vdf` copy, and library de-duping.** Read `steamapps/` only, or fall back to
    `config/`, and is the Steam root added as a library the way the second scanner does, with
    `sort()`/`dedup()` (or `read_link()` comparison) to stop double-scanning? (source:
    https://github.com/ValveSoftware/steam-for-linux/issues/8653,
    https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html)
15. **Ordering, duplicates and `DummyPlugin`'s fate.** Should the plugin sort its own output (slug then
    install path) for a deterministic table, should identical slugs collapse across libraries, and does
    `DummyPlugin` leave `discovery-cli` once a real plugin exists? (source:
    https://doc.rust-lang.org/std/fs/struct.ReadDir.html, crates/discovery/src/engine.rs:21-32,
    apps/discovery-cli/src/main.rs:25-41)
16. **Error surface.** Does `DiscoveryError` grow `Display` + `std::error::Error` and variants for
    `NotInstalled` / `Io` / `Parse`, does `SteamLinuxPlugin` return `Ok(vec![])` when no Steam root exists,
    and does the engine keep swallowing errors — plus whether this crate's manifest switches to
    `version.workspace = true` while its dependencies are being edited (source:
    crates/discovery/src/registry.rs:1-5, crates/discovery/Cargo.toml:1-4,
    crates/transforms/Cargo.toml:4-7).
17. **Performance budget.** No external scan-latency numbers exist; what wall-clock and files-read budget
    should `discover()` hold for a multi-library install (including slow mounts), and should the plugin cap
    files read per library? (source:
    https://docs.rs/steamie-core/latest/src/steamie_core/steam_apps.rs.html,
    docs/adrs/022-game-discovery.md:75-78)
18. **Forward compatibility with the Windows backend.** Should the KeyValues reader and manifest model be
    written now as reusable `crates/discovery` internals so `SteamWindowsPlugin` needs no public-surface
    change? (source: docs/adrs/022-game-discovery.md:45-48)
19. **Legacy `SteamApps` directory spelling.** Older installs reportedly used `SteamApps`; is that spelling
    in scope for a Linux v1, given Linux is case-sensitive and no primary statement of what current Linux
    installs contain was read in this pass (open question; see
    https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs for the lowercase
    composition only).
