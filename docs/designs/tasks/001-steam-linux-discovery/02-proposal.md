# Design: Linux Steam discovery plugin (T-001)

## Context

`crates/discovery` ships the engine, the two-method plugin trait and the data model, but no real probe:
`apps/discovery-cli` registers a `DummyPlugin` returning `Ok(vec![])`
(apps/discovery-cli/src/main.rs:25-41). T-001 (docs/tasks/T-001-steam-linux-discovery.md) adds the first
implementation — `SteamLinuxPlugin`, which reads library folders from `libraryfolders.vdf`, enumerates
`appmanifest_*.acf` under each library's `steamapps/`, extracts `name` and `installdir`, and emits
`DiscoveredGame` values with `platform = SteamLinux` and `codec_id = None`
(docs/tasks/T-001-steam-linux-discovery.md:15-19). Research:
docs/designs/tasks/001-steam-linux-discovery/01-research.md.

The task is additive: no change to `DiscoveredGame`, `Platform`, the `DiscoveryPlugin` trait,
`normalize_slug`, `DiscoveryEngine` or any `schemas/` spec. The open decisions research left (19 open
questions) are all about selection policy and failure policy, not about the data model, and this document
settles each one that traces to T-001's Decision clauses; the rest are listed in
*Deferred / out of scope*.

## Goals / Non-goals

- Goals:
  - `SteamLinuxPlugin` in `crates/discovery` implementing the existing `DiscoveryPlugin` trait unchanged.
  - Library folders from `libraryfolders.vdf`, plus a guaranteed scan of each root's own `steamapps/`.
  - Manifest parsing that yields `display_name` + `install_path` for every well-formed
    `appmanifest_*.acf`, with the `name`-absent fallback §5 defines.
  - `DiscoveredGame` construction through `normalize_slug`, `platform = SteamLinux`, `codec_id = None`,
    in deterministic order.
  - `discovery list` runs the plugin, with a `--steam-root` seam that makes both CLI tests hermetic.
  - Unit tests in `crates/discovery` for manifest parsing and library-folder detection, against
    checked-in fixtures, with no new dependency and no new filesystem writes in tests.
  - Degraded scans: one unreadable file never costs the whole scan.
- Non-goals:
  - Windows discovery, Steam Deck, Epic/manual discovery (ADR-022 scopes; see *Deferred*).
  - Any change to `DiscoveredGame`, the trait, the engine's error handling, or `DiscoveryError`.
  - Non-game (Proton / runtime / redistributable) filtering and `StateFlags` readiness filtering.
  - Slug migration away from `unknown/`, and any AppID use.
  - Persistence, caching, or a performance-budget harness.

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | An `appmanifest_<appid>.acf` is text KeyValues whose root section is `AppState`, carrying `installdir` and an optional `name`. | verified | https://github.com/leovp/steamfiles/blob/master/docs/acf_overview.rst, https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/app.rs |
| A2 | `installdir` is a directory *name*; the game directory is `<library>/steamapps/common/<installdir>`. | verified | https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs |
| A3 | A root's library list is `<root>/steamapps/libraryfolders.vdf`; the modern shape keys libraries by decimal index and keeps only blocks whose key parses as a number, each carrying `path`. | verified | https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/library.rs |
| A4 | Look-alike `libraryfolders.vdf` copies live under `<library>/steamapps/compatdata/<appid>/pfx/…` in another format. | verified | https://github.com/ValveSoftware/steam-for-linux/issues/8653 |
| A5 | Key case varies across real files (`INSTALLDIR`, `stateflags`), so key lookup must be case-insensitive. | verified | https://docs.rs/steamie-core/latest/steamie_core/vdf/index.html |
| A6 | The only escapes Steam writes inside quoted values are `\\` and `\"`. | unverified | https://github.com/frankea/Whisky/blob/e51bd9f6/WhiskyKit/Sources/WhiskyKit/Steam/VDFParser.swift (no Valve grammar exists) |
| A7 | These two file kinds contain no `#base` directive, `[$WIN32]`-style conditional or other construct the reader is designed to reject. | unverified | no statement of what the Linux client writes was found (01-research.md:352-357) |
| A8 | Steam roots are found under the home directory (native plus `~/.steam/*`), `$XDG_DATA_HOME/Steam`, the Flatpak `~/.var/app/com.valvesoftware.Steam` tree and the Snap `~/snap/steam/common` tree; a directory is a root when it has a `steamapps` directory. | verified | https://raw.githubusercontent.com/WilliamVenner/steamlocate-rs/master/src/locate/linux.rs |
| A9 | `std::env::home_dir()` yields the home directory (on Unix `$HOME`, else the passwd entry), and a relative `$XDG_DATA_HOME` is invalid and must be ignored. | verified | https://doc.rust-lang.org/std/env/fn.home_dir.html, https://specifications.freedesktop.org/basedir-spec/latest/ |
| A10 | A Flatpak install can name library paths that exist only inside the sandbox, or on mounts the discovery user cannot read. | verified | https://docs.flatpak.org/en/latest/sandbox-permissions.html |
| A11 | `ReadDir` yields entries in unspecified order, so any stable output needs an explicit sort. | verified | https://doc.rust-lang.org/std/fs/struct.ReadDir.html |
| A12 | `libraryfolders.vdf` also exists in a pre-2021 shape that a modern-shape-only reader parses to zero libraries. | verified | https://docs.rs/aurelia-steam/latest/aurelia/library/fn.library_folder_paths.html |
| A13 | Proton builds, the Steam Linux Runtime and redistributables own appmanifests inside ordinary libraries. | verified | https://docs.rs/steamlocate/latest/steamlocate/, https://github.com/Eloreden/protonaut |
| A14 | `DiscoveryEngine::discover_all` swallows a plugin's `Err` into one `eprintln!` on stderr and still returns `Ok`. | verified | crates/discovery/src/engine.rs:20-33 |
| A15 | Today's `Cargo.lock` is a 32-package closure containing no KeyValues parser, `dirs`, `walkdir` or `tempfile`. | verified | Cargo.lock (32 `[[package]]` entries), crates/discovery/Cargo.toml:6-7 |
| A16 | A real `appmanifest_*.acf` / `libraryfolders.vdf` is far smaller than a 1 MiB read cap. | unverified | no size figure is published; the captured samples described in 01-research.md:82-88 run to tens of lines |
| A17 | The seam is unchanged: `discover(&self)` takes no arguments, `Platform::SteamLinux` and `normalize_slug` exist, `DiscoveredGame` has five fields, and `DummyPlugin` has no user outside `apps/discovery-cli/src/main.rs`. | verified | crates/discovery/src/{plugin.rs:4-7, game.rs:5-27, slug.rs:1-35, lib.rs:1-5}, apps/discovery-cli/src/main.rs:25-41 |
| A18 | The CI `rust` job runs fmt, clippy `-D warnings` and `cargo test --workspace --locked` on `ubuntu-latest` only. | verified | .github/workflows/check.yml:16-33 |
| A19 | `env!("CARGO_MANIFEST_DIR")` gives a test its own package root at compile time, and `env!("CARGO_BIN_EXE_<name>")` gives an integration test the absolute path of a binary of its own package. | verified | https://doc.rust-lang.org/cargo/reference/environment-variables.html |
| A20 | Current Linux installs spell the directory `steamapps`, never the legacy `SteamApps`. | unverified | 01-research.md:416-420 found no primary statement either way |

## Options considered

**Depend on `keyvalues-parser` (+ `keyvalues-serde`).** It is what `steamlocate` uses and is current.
Rejected: it drags `pest` and its support crates into a committed `Cargo.lock` on a pinned 1.85 floor
(A15, A18), and its own docs say the representation is lossy for this job — a `BTreeMap` loses declaration
order, an empty value sequence renders as a missing pair, and `#base` needs a separate type
(01-research.md:120-123). The two files need a small subset of the grammar, and the subset is exactly what
this design must constrain anyway.

**Depend on `steamlocate`.** It is the same pipeline, already tested. Rejected: it models games by AppID
(ADR-022 forbids AppID use, docs/adrs/022-game-discovery.md:49), it fixes the root/library policy to its
own, and its failure policy aborts a whole scan where one library block lacks a usable `path` and aborts a
file read on non-UTF-8 bytes (01-research.md:169-174) — the precise blast radius this design avoids.

**Model both files with `serde`.** Cheap to write. Rejected on precedent: a maintained project abandoned
serde for `libraryfolders.vdf` because an untagged enum spanning the modern and pre-2021 shapes matched
neither and silently yielded zero libraries (A12).

**Hand-rolled reader (chosen).** Several current Rust scanners that read exactly these two files document
the same choice — "the format is small and stable and not worth a dependency" (01-research.md:180-192).
It keeps `Cargo.lock` byte-identical and puts the grammar subset, the error positions and the caps under
this repo's control.

**Scan only the first valid Steam root.** One install per run is simpler. Rejected: A8's candidate list
exists because native, Flatpak and Snap layouts coexist, and `steamlocate` exposes `locate_all()` for
exactly this; scanning the first match silently loses a second installation's libraries.

**Take the installed-game list from `libraryfolders.vdf`'s `apps` map.** Fewer files read. Rejected: the
reference implementation deliberately ignores that map and enumerates `steamapps/` instead, "since the
values in `libraryfolders.vdf` may be stale" (01-research.md:99-102).

**Filter non-games by an appid/name denylist.** Proton and runtime manifests are product-visible noise
(A13). Rejected for this task: it needs either AppID (ADR-022 forbids it) or an unowned name heuristic
that a real tool already maintains "heuristically" (01-research.md:136-140); designing one here commits
BindForge to a list nobody owns. Deferred with its owner in *Deferred / out of scope*.

**Fail the plugin on any unreadable file.** Loud and simple. Rejected: with A14 the only visible effect is
one debug line and an empty list, so one stale entry would turn into zero games — the whole-scan failure
mode research flags (01-research.md:281-286). Skip-and-continue with one narrow error path is chosen
instead.

**Build a writable Steam tree in a temp directory for tests.** Realistic. Rejected as unnecessary: it
needs either a `tempfile` dev-dependency (A15) or hand-rolled temp-dir code plus a generated
`libraryfolders.vdf`; a checked-in read-only fixture tree plus the relative-`path` rule below covers the
same cases without a write or a dependency.

## Chosen design

### 1. Modules and public surface

Two new modules in `crates/discovery`, exported from `crates/discovery/src/lib.rs` alongside the existing
five (crates/discovery/src/lib.rs:1-5):

```rust
// crates/discovery/src/keyvalues.rs — platform-neutral, no Steam knowledge.
pub struct KeyValueError { pub line: usize, pub message: &'static str }
pub enum Value { Str(String), Section(Section) }
pub struct Section { pub children: Vec<(String, Value)> }   // declaration order preserved
impl Section {
    pub fn get_str(&self, key: &str) -> Option<&str>;       // case-insensitive, first match wins
    pub fn get_section(&self, key: &str) -> Option<&Section>;
}
pub fn parse(text: &str) -> Result<Section, KeyValueError>; // top-level pseudo-section

// crates/discovery/src/steam_linux.rs — the plugin and its Linux policy.
pub struct SteamLinuxPlugin { roots: Vec<PathBuf> }
impl SteamLinuxPlugin {
    pub fn from_host() -> Self;
    pub fn from_roots(roots: impl IntoIterator<Item = PathBuf>) -> Self;
}
impl DiscoveryPlugin for SteamLinuxPlugin { /* name() -> "steam-linux" */ }

pub fn host_candidates(home: &Path, env: &dyn Fn(&str) -> Option<String>) -> Vec<PathBuf>; // pure
pub fn libraries_in_root(root: &Path) -> Vec<PathBuf>;
pub struct AppManifest { pub display_name: String, pub install_dir: String }
pub fn parse_manifest(text: &str) -> Option<AppManifest>;   // None = skip this manifest
```

No `new()` / `Default` impl: a bare `pub fn new()` would trip clippy's `new_without_default` under
`-D warnings` (A18), and both constructors say more than `new()` does. Every `pub` item above is exercised
by a test in *Test strategy*; the rest (scan loop, composition, comparison key) stays private. The plugin
holds its roots because `discover(&self)` takes no arguments (A17).

### 2. Root selection

`from_host()` builds its candidate list with the pure `host_candidates(home, env)` — so the list is unit
testable without touching the environment — and probes it. `home` comes from `std::env::home_dir()` (A9);
when it returns `None`, the candidate list is empty and the plugin discovers nothing. Candidates, in this
order, each kept only if `<candidate>/steamapps` is a directory (A8):

1. `$XDG_DATA_HOME/Steam`, default `$HOME/.local/share/Steam`; a set-but-relative `XDG_DATA_HOME` is
   ignored per the base-directory spec (A9).
2. `$HOME/.steam/steam`, `$HOME/.steam/root`, `$HOME/.steam/debian-installation`.
3. `$HOME/.var/app/com.valvesoftware.Steam/{.local/share/Steam,.steam/steam,.steam/root}` (Flatpak).
4. `$HOME/snap/steam/common/{.local/share/Steam,.steam/steam,.steam/root}`, or the same three under
   `$SNAP_USER_DATA/steam/common` when that variable is set (Snap).

**Canonical path rule (canonical statement, referenced below):** every set of directories the plugin
scans — roots from the candidate list, and library paths within a root — is built by canonicalizing each
entry, dropping entries that fail to canonicalize or that lack the required `steamapps` directory, then
sorting, then de-duplicating. `std::fs::canonicalize` resolves the symlink aliases (`~/.steam/root` →
the real root) and the `"0" → root` duplicate that libraryfolders.vdf normally contains, so no library is
ever scanned twice. The emitted `install_path` keeps the canonical library prefix; nothing the plugin
emits carries an unresolved `..`.

`from_roots()` skips `host_candidates` and the existence probe, taking caller-supplied roots through the
canonical path rule. It is the seam tests and `--steam-root` use; `from_host()` is a thin wrapper over it.

### 3. Library selection within a root

`libraries_in_root(root)` returns, through the canonical path rule:

- `<root>/steamapps` itself, always. This is the root-as-library inclusion the second scanner in
  01-research.md:103-106 uses, and it is what makes a `libraryfolders.vdf` that parses to zero libraries —
  the legacy shape (A12) or a hand-edited file — a degraded result instead of the silent-zero bug.
- every `path` from `<root>/steamapps/libraryfolders.vdf`: the `libraryfolders` section (case-insensitive)
  is walked for numeric-keyed blocks (A3); a `path` is used as-is when absolute and otherwise joined to
  the directory holding the VDF; empty values are dropped. The relative case is not hypothetical in this
  repo — a checked-in fixture cannot contain this machine's absolute paths (see *Fixtures*).

The VDF is read at that one fixed path per root and never searched for, and nothing below `steamapps/` is
walked recursively, so the Proton-prefix look-alikes (A4) are never opened: `appmanifest_*.acf` files are
read only as entries directly inside a selected library's `steamapps/`.

### 4. KeyValues reader

`keyvalues::parse` accepts, for both file kinds: whitespace-separated tokens; quoted and bare keys and
values; `{}`-nested sections; `//` line comments; a leading UTF-8 BOM; LF and CRLF. It unescapes `\\` and
`\"` inside quoted values — assumed sufficient for what Steam emits (A6) and the behaviour the two
surveyed readers share. `children` keeps declaration order and `get_str` is case-insensitive first-match
(A5), so a duplicate key resolves to its first occurrence. It rejects, as a positioned
`KeyValueError { line, message }`: unbalanced braces, an unterminated string, a `[` conditional or `#base`
line — constructs assumed absent from these files (A7) — and any section nested deeper than 16 levels.
Input is `&str`; the caller reads bytes and skips a file that is not valid UTF-8 rather than aborting a
scan. Files larger than 1 MiB are skipped as malformed (A16): the reader's work is bounded per file rather
than by a wall-clock target, since no scan-latency baseline exists to budget against
(01-research.md:319-324).

### 5. Manifest to `DiscoveredGame`

`parse_manifest(text)`:

- parses with `keyvalues::parse` and takes the `AppState` section case-insensitively;
- `display_name` = `name`, falling back to `installdir` when `name` is absent or empty (A1) — a
  `DiscoveredGame` needs a non-empty `display_name` because the slug derives from it
  (docs/adrs/022-game-discovery.md:55), and the fallback keeps `unknown/` out of otherwise good rows;
- `install_dir` = `installdir`; the manifest is rejected (`None`) when it is absent, empty, `.`, `..`,
  absolute, or contains `/`, because it is appended unvalidated to the emitted path — unlike library
  paths, the composed path need not exist and so cannot be canonicalized (see *Root selection*);
- returns `None` on any parse error, which is how "this file is broken, the rest survive" is expressed.

The plugin then builds, for manifest `m` in library `l`:
`install_path = l.join("steamapps").join("common").join(m.install_dir)` (A2), emitted whether or not that
directory exists; `slug = normalize_slug(&m.display_name)`; `platform = Platform::SteamLinux`;
`codec_id = None`. `appid` is never read and never stored (docs/adrs/022-game-discovery.md:49). Results
are sorted by `(slug, install_path)` — `read_dir` order is unspecified (A11), and the CLI table and the
tests both need a stable order. Identical slugs from two libraries stay two rows: which install a slug
means is the resolver's problem (*Deferred*).

### 6. Selection policy

Every manifest that survives §5 is emitted. No `StateFlags` bit test and no non-game filter, so Proton
builds and the Steam Linux Runtime appear in `discovery list` as rows (A13) — an accepted, visible cost of
a plugin that only reports what Steam reports as installed. Filtering is one policy function in one file,
deferred with its constraints in *Deferred / out of scope*.

### 7. Failure policy and the one error path

Skip-and-continue everywhere: an unreadable or non-UTF-8 manifest, a manifest that fails to parse, a
missing or malformed `libraryfolders.vdf`, an unreadable library and an unresolvable root are each skipped
and the scan continues. With no roots (no Steam, or no home directory) `discover()` returns
`Ok(vec![])` — "not installed here" is not an error, because A14 turns any error into one `{:?}` line on
stderr that a `--json` consumer must wade through.

The single `Err` case: at least one root was selected but **no** library under any selected root could be
scanned. Then `discover()` returns `DiscoveryError::PluginError(msg)`, where `msg` names the selected roots
and the count of files skipped as unreadable or malformed and nothing else — no file contents. That is
enough for the engine's debug line to tell "Steam present, everything under it unreadable" apart from
"nothing installed", and it stays on stderr, out of the stdout stream a `--json` consumer parses.

### 8. CLI wiring (`apps/discovery-cli`)

`Commands::List` gains one argument:

```rust
/// Steam root(s) to scan instead of probing this host. Repeatable.
#[arg(long = "steam-root")] steam_root: Vec<PathBuf>,
```

In the `List` arm, `steam_root.is_empty()` selects `SteamLinuxPlugin::from_host()`, otherwise
`SteamLinuxPlugin::from_roots(steam_root.iter().cloned())`, registered in place of the deleted
`DummyPlugin` — its only user is this arm (A17), and it asserts nothing a fixture run does not. The
printing code is untouched, so the table header and JSON shape are unchanged.

### 9. Fixtures

Read-only, checked-in, one tree used by both packages — the crate's tests directly, the CLI's through
`env!("CARGO_MANIFEST_DIR")` (A19):

```
crates/discovery/tests/fixtures/steam/minimal/
├── steamapps/
│   ├── libraryfolders.vdf                  # modern shape: "0" → ".", "1" → "../secondary"
│   ├── appmanifest_900001.acf              # AppState with name + installdir "Example Game"
│   ├── appmanifest_900002.acf              # no name; installdir "Nameless Install"
│   ├── appmanifest_900003.acf              # "NAME" "Mixed Case Game", "INSTALLDIR", "stateflags"
│   ├── appmanifest_900004.acf              # truncated mid-section → skipped
│   ├── appmanifest_900005.acf              # installdir "../outside" → skipped
│   └── compatdata/900001/pfx/drive_c/Program Files (x86)/Steam/steamapps/
│       ├── libraryfolders.vdf              # prefix decoy (A4) — must never be opened
│       └── appmanifest_900099.acf          # prefix decoy — must never be discovered
└── secondary/steamapps/
    └── appmanifest_900100.acf              # the secondary library's one game ("Secondary Title")
```

Four games survive from this tree; the `"0" → "."` entry exercises root/library de-duping and the
relative `"1"` entry exercises the join rule portably. AppIDs are synthetic: the number appears only in
the filename, which is the filename Steam uses, and the design never reads it. Existing crate tests keep
the `crates/discovery/tests/<module>_tests.rs` layout, adding `keyvalues_tests.rs` and
`steam_linux_tests.rs`. No test writes to the filesystem; a later round that needs a writable tree has
`env!("CARGO_TARGET_TMPDIR")` (A19) before reaching for a dev-dependency.

### 10. Dependencies and gates

No new dependency: `crates/discovery/Cargo.toml` and `apps/discovery-cli/Cargo.toml` gain nothing, so
`Cargo.lock` stays byte-identical (A15) and `cargo test --workspace --locked` (A18) needs no re-lock.
`crates/discovery/Cargo.toml` keeps its literal `version`/`edition` fields — switching it to
`version.workspace = true` is unrelated churn (*Deferred*). `#[path]`-free, `unsafe`-free, no `std`
feature flags required on the 1.85 floor (A15).

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Proton / Steam Linux Runtime / redistributable rows appear as bindable games (A13). | High | Medium | Accepted for T-001 and stated in *Deferred*; the row's display name is human-legible, and the fix is one policy function once a kind field or filter policy exists. |
| Undocumented grammar (A6, A7) makes a real file fail to parse. | Medium | Medium | Per-file skip keeps the rest of the scan; the root's own `steamapps` is always scanned, so a lost `libraryfolders.vdf` costs secondary libraries only; the §7 error path surfaces the all-lost case. |
| `install_path` composes from user-editable files and is later handed to a codec that writes there. | Medium | High | Library paths are canonicalized and must contain a `steamapps` directory; `installdir` is rejected on `.`/`..`/absolute/`/`; no recursion, 1 MiB per file, 16 levels; validation at the point of writing stays the codec's job (*Deferred*). |
| A degraded scan prints nothing: skip-and-continue plus A14's swallowed error. | Medium | Medium | §7's single error path; *Test strategy* #7 pins both sides of it, so the design cannot silently regress to zero. |
| Candidate list (A8) misses a real layout or has a typo, and `from_host()` cannot be unit-tested. | Medium | Medium | The list lives in the pure `host_candidates`, asserted path-by-path in *Test strategy* #5; `--steam-root` gives any user a manual override. |
| Two installs of one title produce two rows with the same `unknown/<slug>`. | Medium | Low | Explicitly not collapsed here (§5); belongs to ADR-022's slug migration (*Deferred*). |
| 1 MiB / 16-level caps skip a file that was actually readable (A16). | Low | Low | Both caps are single named constants, and the skipped-file count is inside the §7 error message. |
| Flatpak roots yield nothing useful from the host (A10). | Medium | Low | Library selection's `steamapps`-directory gate drops sandbox-only paths; *Compatibility matrix* records the partial result instead of failing. |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux, native Steam under `$HOME` (incl. `~/.steam/*`) | Yes | The A8 native candidates; primary and secondary libraries. |
| Linux, `$XDG_DATA_HOME/Steam` override | Yes | Honoured when absolute; a relative value is ignored (A9). |
| Linux, libraries on arbitrary mounts (`/run/media/…`, `/mnt/…`, ZFS) | Yes | Absolute `path` values are accepted as-is; an unmounted library is skipped, not fatal. |
| Linux, Flatpak Steam | Partial | Probed; library paths visible only inside the sandbox are dropped by the `steamapps` gate (A10). |
| Linux, Snap Steam | Partial | Probed under `$HOME/snap/steam/common` or `$SNAP_USER_DATA/steam/common`; same gate. |
| Linux, Steam not installed / no home directory | Yes | `Ok(vec![])`, no error, no stderr output (§7). |
| Linux, pre-2021 `libraryfolders.vdf`, or legacy `SteamApps` spelling (A20) | Partial | The root's own `steamapps` is still scanned; secondary libraries and a legacy-spelled directory are not discovered. |
| CI `rust` job (`ubuntu-latest`, no Steam, no network) | Yes | A18; the suite is hermetic through `from_roots` + fixtures, so no test depends on the runner's home directory. |
| Windows | No | `SteamWindowsPlugin` is ADR-022's Windows clause, not T-001; `keyvalues.rs` is written platform-neutral to be reusable. |
| macOS | No | Outside ADR-022's v1 scope. |
| Steam Deck | No | Excluded by ADR-022 (docs/adrs/022-game-discovery.md:21). |

## Test strategy

1. Unit (`keyvalues`): quoted and bare keys/values and nested `{}` sections parse in declaration order;
   `get_str` matches `name`, `NAME` and `Name` alike (A5) and returns the first of duplicate keys;
   `//` comments, a BOM, and CRLF parse; `\\` and `\"` unescape to `\` and `"`; an unbalanced brace, an
   unterminated string and a `[` conditional each return a `KeyValueError` carrying the offending line; a
   section nested 17 levels deep is rejected.
2. Unit (`parse_manifest`): name + installdir yields both fields; a manifest with no `name` yields
   `display_name == installdir`; a manifest with neither, one with `installdir` of `..` / `/abs/path` /
   `sub/dir`, and one that fails to parse all yield `None`.
3. Unit (`libraries_in_root`, fixture): the primary library and `../secondary` are both returned,
   canonicalized, sorted, and the `"0" → "."` entry adds no duplicate (§2's canonical path rule); a root
   with no `steamapps` directory yields an empty list; a root whose `libraryfolders.vdf` is absent still
   yields its own `steamapps`.
4. Unit (fixture decoys, `libraries_in_root` plus plugin output): no path under `compatdata/` is ever returned by
   `libraries_in_root`, and nothing under it appears in plugin output (A4).
5. Unit (`host_candidates`, pure): with `home = /home/t` and an empty environment the list is exactly the
   ordered `$HOME` candidates of §2; setting `XDG_DATA_HOME` replaces candidate 1, a relative
   `XDG_DATA_HOME` does not (A9); setting `SNAP_USER_DATA` replaces the Snap trio's base.
6. Unit (output contract, fixture root): every returned game has `platform == Platform::SteamLinux`,
   `codec_id == None` and `slug == normalize_slug(display_name)`; `install_path` ends with
   `steamapps/common/<installdir>` (A2) and starts with the canonicalized fixture root; the vector is
   ordered by `(slug, install_path)`.
7. Unit (failure policy, §7): `from_roots` with a nonexistent root returns `Ok(vec![])`; a root that has
   a `steamapps` directory whose entries are all unparseable returns exactly one
   `Err(DiscoveryError::PluginError(_))` whose message names the root and no file contents.
8. Integration (`crates/discovery`): `SteamLinuxPlugin::from_roots([fixtures/steam/minimal]).discover()`
   returns exactly four games — `unknown/example-game`, `unknown/mixed-case-game`,
   `unknown/nameless-install`, `unknown/secondary-title` — proving the two libraries, the `name`-absent
   fallback, mixed key case, the skipped malformed and escaping manifests, and the ignored decoys.
9. Integration (`apps/discovery-cli/tests/cli_tests.rs`): both existing tests run the binary through
   `env!("CARGO_BIN_EXE_discovery-cli")` (A19) with `--steam-root` at the shared fixture (A19 path), and
   assert the table header plus the four expected rows, and `list --json` as exactly those four
   `DiscoveredGame` objects; a third test asserts `list --json` prints `[]` for a `--steam-root` that does
   not exist. No CLI test probes the host, so `list --json` stops being machine-dependent
   (01-research.md:308-312).
10. Platform: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
    `cargo test --workspace --locked` pass on the pinned 1.85 toolchain with `Cargo.lock` unchanged (A15,
    A18); the A18 CI job runs on `ubuntu-latest` with no Steam installed and no network, and every test
    above passes there — no test reads the developer's home directory.

## Rollout

1. Land `keyvalues.rs` + `steam_linux.rs` with their two test files and the fixture tree; `discovery-cli`
   untouched at this step, so the existing CLI tests still pass unchanged.
2. Wire `--steam-root`, delete `DummyPlugin`, update the two CLI tests and add the `[]` case; no output
   format, schema or data-model change, so nothing downstream migrates.
3. Run the gates (A18). `Cargo.lock` shows no diff — that is the check that the dependency posture held.
4. Default on merge: `discovery list` probes the host, so a Linux user with Steam sees their games
   immediately, and a user without Steam sees an empty list. No config file, no env var, no persistence,
   and `BINDFORGE_*` names are not introduced.
5. Manual Done-when check (docs/tasks/T-001-steam-linux-discovery.md:23): a contributor runs
   `discovery list` and `discovery list --json` on a Linux machine with a real Steam install and pastes
   both into `03-build-notes.md`, together with one real `appmanifest_*.acf` and the machine's
   `libraryfolders.vdf` if they can be shared. If a real file breaks the reader's grammar subset (A6, A7),
   that captured file becomes a fixture and the subset is re-opened in review.
6. No feature flag and no rollback path beyond reverting the commit: the plugin is additive and the CLI's
   `--steam-root` seam is the only user-visible surface.

## Deferred / out of scope

- Non-game filtering (Proton, Steam Linux Runtime, redistributables) and where its policy lives: no
  follow-up yet; constrained by ADR-022's "No AppID usage" and the absence of a kind field on
  `DiscoveredGame`.
- `StateFlags` readiness filtering: no follow-up yet; belongs with the filter above.
- Slug migration from `unknown/<kebab(name)>` and slug collisions across libraries: ADR-022 Consequences
  ("v1 slugs are provisional and will need migration").
- `DiscoveryError` taxonomy (`Display`, `std::error::Error`, `NotInstalled`/`Io`/`Parse`) and engine-side
  error collection instead of the swallowed `eprintln!`: ADR-022's plugin contract; no follow-up yet.
- Richer per-skip diagnostics for degraded scans: no follow-up yet; §7's error path is the v1 floor.
- `SteamWindowsPlugin` and registry-based root discovery: ADR-022's Steam discovery v1 Windows clause; no
  task filed.
- Legacy pre-2021 `libraryfolders.vdf` shape and a legacy `SteamApps` directory spelling (A12, A20): no
  follow-up yet; §3's always-scan of the root's own `steamapps` limits the damage to secondary libraries.
- Flatpak host visibility, and reporting library paths that exist only in the sandbox: no follow-up yet.
- Steam Deck / SteamOS layouts: ADR-022 excludes Deck.
- Scan-latency budgets and a benchmark harness: no follow-up yet; this design bounds work per file only.
- `crates/discovery/Cargo.toml` → `version.workspace = true`: chore; no follow-up yet.
- Validating `install_path` before a codec writes to it: the codec's own design.
- Sorting or de-duplicating results across plugins in `DiscoveryEngine`: ADR-022's plugin contract; the
  plugin sorts its own output only.

## Revision log

- r0: q1 root policy — chose the full A8 candidate list (native + Flatpak + Snap) with all valid roots
  scanned; *Chosen design §2*.
- r0: q2, q3 test seam and fixtures — added `from_roots` / `--steam-root` and a checked-in read-only
  fixture tree; no `tempfile`, CLI tests switched to `CARGO_BIN_EXE_*`; *§8, §9, Test strategy #9*.
- r0: q4, q5, q6, q7 dependency, grammar, escaping, key case — hand-rolled `keyvalues.rs` with an explicit
  accepted/rejected subset, `\\`/`\"` unescaped (A6), case-insensitive first-match lookup; *§1, §4*.
- r0: q8, q14 legacy shape and which VDF copy — one fixed VDF path per root, root's own `steamapps` always
  scanned, sets canonicalized/sorted/de-duped; the legacy shape itself deferred; *§2, §3*.
- r0: q9, q10 non-game filter and `StateFlags` — emitted unfiltered in v1, recorded as an accepted visible
  cost; the policy itself deferred; *§6, Risks, Deferred*.
- r0: q11 absent `name` — falls back to `installdir`; a manifest with neither is skipped; *§5*.
- r0: q12 path composition and validation — composed unconditionally, `installdir` rejected on
  `.`/`..`/absolute/`/`, library paths validated by canonicalization plus the `steamapps` gate; *§2, §5*.
- r0: q13, q16 failure policy and error surface — skip-and-continue with one `Err` case, `Ok(vec![])` when
  nothing is installed, `DiscoveryError` and the engine left unchanged; *§7, Deferred*.
- r0: q15 ordering and `DummyPlugin` — output sorted by `(slug, install_path)`, `DummyPlugin` deleted, no
  cross-plugin de-duping; *§5, §8*.
- r0: q17 performance budget — no external baseline exists, so the budget is per file (1 MiB, 16 levels),
  not per scan; wall-clock tuning deferred; *§4, Deferred*.
- r0: q18 Windows reuse — `keyvalues.rs` carries no Linux or Steam knowledge so a later Windows plugin
  reuses it without a public-surface change; *§1, Compatibility matrix*.
- r0: q19 legacy `SteamApps` spelling — not accepted, kept as assumption A20 and deferred; *A20, Deferred*.
