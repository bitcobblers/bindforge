# Design: Extract installed games' key bindings through codec families (ADR-017)

## Context

Source: `docs/adrs/017-keybinding-extraction-codecs.md`, built on
`docs/designs/017-keybinding-extraction-codecs/01-research.md`.

Four things land: a `crates/codecs` crate owning the codec-family trait, the
claim payload and the closed registry; the launcher-context plumbing that lets
a codec be told *which* Steam root and library own a discovered game (today's
scan flattens them — A2); the `bindings/v1` snapshot contract under `schemas/`,
as JSON; and a read-only `discovery bindings <slug>` face on
`apps/discovery-cli`. The first family is `unreal-ini`. The run writes nothing —
to no game, no launcher, no framework tree — and `discovery list` is untouched,
so its `codec_id` keeps spelling `None` (`017-L5`).

Nine clauses of the source are read differently than worded, each on an
empirical basis the research surfaced and recorded in the Revision log:
`017-B2`'s and `017-B3`'s units of count, `017-C1`'s codec-id bound and its
`extract` return shape, `017-C2`'s root set, `017-C3`'s `inputs` spelling and
document identity, `017-C4`'s "same scan", and `017-S1`'s native launcher
context — the one list §Rollout item 8 hands to the operator.

## Goals / Non-goals

- Goals:
  - `crates/codecs` (package `bindforge-codecs`) lands with the
    `CodecFamily` trait (`id`/`appids`/`detect`/`extract`), the `Detection`
    claim payload with its match evidence, and a closed static registry
    iterated in family-id order with the appid pass before any tree probing
    (`017-C1`, `017-C5`).
  - `GameContext` carries the lookup key, the digit appid, the install path,
    and the *owning* Steam root and library — reached by a new additive
    `scan_records()` seam in `crates/discovery`; the `DiscoveryPlugin` trait is
    not touched (§The codec face, §Discovery plumbing).
  - `schemas/bindings/v1/` lands: `spec.md`, valid/invalid JSON fixture
    corpora, and a loader in `crates/codecs` that refuses by name;
    `schemas/README.md` names it as the contract tree's second JSON format and
    states its budgets (§`bindings/v1`).
  - A native token that the ADR-012 closed vocabulary cannot spell is recorded
    **as the file spells it**: a snapshot carries native action names and
    native input tokens verbatim, and every vocabulary join stays the
    manifest's (`013-C4`, `017-L3`; canonical: §`bindings/v1`).
  - `unreal-ini` ships with fixture trees for the two launcher contexts whose
    roots a record can justify, the read-only verification pass, and the
    never-write discipline (`017-S1`, `017-L1`).
  - `discovery bindings <slug>` with `--probe`, the aligned table and `--json`,
    and four named statuses on four distinct exit codes — claim-refused,
    family-confirmed-files-absent, extraction-refused, ok (`017-C4`, `017-L2`;
    canonical: §The CLI face).
  - Determinism: one ordering law for entries, inputs, users, provenance and
    serialization, pinned by a byte-identity test over the fixtures
    (`017-L4`; canonical: §Determinism).
  - `discovery list`'s rows, columns, JSON keys, exit code and plugin loop are
    unchanged and stay pinned by the existing suites (`017-L5`).
- Non-goals:
  - No apply face, no backup/restore, no authoring CLI, no snapshot file
    written anywhere (`017-R1`).
  - No Windows launcher trees (`017-R2`), no Source or later families
    (`017-R3`), no appid→family routing *data* (`017-R4`).
  - No widening of the ADR-012 input vocabulary — `tab`, `esc`, arrows,
    analog axes stay outside it; that is ADR-012's vocabulary major, not this
    run's.
  - No catalog probe, no match-store load, no candidate query, no confirmation
    prompt on the bindings face (§The CLI face), and no change to `list`'s gate.
  - No manifest creation, no fingerprint computation, no snapshot caching or
    persistence, no shadowing/order model in the format, no non-Steam sources.
  - No new `apps/cli` crate and no `toml_edit` dependency: ADR-008's planned
    names for this work are superseded by the shipped layout (§Deferred).

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | `DiscoveryPlugin` is exactly `{name, discover}`, `discover()` takes no arguments, and `DiscoveredGame` is `{slug, friendly_name, display_name, install_path, platform, codec_id: Option<String>}` with `codec_id` never written | verified | crates/discovery/src/plugin.rs:4-7; crates/discovery/src/game.rs:115-126 |
| A2 | `SteamCore` unions the libraries of all selected roots, canonicalizes and dedupes them, scans each, and records neither which root owned which library nor which root was "main"; the `(slug, install_path)` sort is the only ordering, and there is deliberately no collision pass | verified | crates/discovery/src/steam_common.rs:90-131 |
| A3 | Steam root selection is an ordered candidate list (XDG, three native suffixes, three Flatpak, two Snap) and every existing candidate is kept, so more than one full install can be in play | verified | crates/discovery/src/steam_linux.rs:92-128 |
| A4 | The read posture this design reuses is a metadata length check against `keyvalues::MAX_DOCUMENT_BYTES` (1 MiB) applied *before* allocation, then a strict-UTF-8 `String::from_utf8` that skips a non-UTF-8 file | verified | crates/discovery/src/keyvalues.rs:21-24; crates/discovery/src/steam_common.rs:329-359; crates/core/src/toml.rs:36-58 |
| A5 | The input vocabulary is closed at 48 graded keyboard members (glyphs + `space`), 7 mouse members and 18 gamepad members, and the code says outright that "modifiers and named keys are outside the closed set"; there is no chord or modifier syntax in the id grammar | verified | crates/core/src/input.rs:70-121 |
| A6 | The manifest `codec` entry is "slug grammar, at most the shared `MAX_SLUG_OCTETS`", and that bound's stated home is `024-B1`; `013-B4` is `MAX_NATIVE_NAME_OCTETS = 128` — a different constant for a different fact | verified | schemas/manifests/v1/spec.md:56,121; schemas/README.md:145-160 |
| A7 | `load_manifest` deliberately does *not* registry-check `codec` or preset ids, "until ADR-017", and `013-R3` names ADR-017 as the owner of that check | verified | schemas/manifests/v1/spec.md:157-160; docs/adrs/013-game-manifests.md:157-159 |
| A8 | The manifest fingerprint is grammar-only (`sha256-<base64>`), which bytes are hashed and what a mismatch mean are "ADR-017's", and `013-L7` fixes it as provenance, never a load-time guard | verified | schemas/manifests/v1/spec.md:123-129; docs/adrs/013-game-manifests.md:122-123,155-156 |
| A9 | The corpus-boundary law is `025-L5`, and its gate is a sweep over `data/` and every `schemas/*/v*/fixtures/valid/` for the `steam/`/`epic/`/`local/` spellings with exactly one skipped format tree named by `SOURCE_KEY_TREE = "catalog"` | verified | docs/adrs/025-discovery-identity-and-catalog-matching.md:134-138; crates/discovery/tests/boundary_tests.rs:104-130 |
| A10 | Formats are TOML except `catalog`, named in the README, and the harness's one carve-out const `JSON_FIXTURE_FORMAT = "catalog"` is the single name every skip site reads (`is_core_toml_format`, `is_fixture_name`) | verified | schemas/README.md:61-88; crates/core/tests/common/mod.rs:420-444; crates/core/tests/conformance.rs:47-54 |
| A11 | The CLI has one subcommand (`List`), global `--json`/`--catalog-url`, one failure voice (`error: ` line + `ExitCode::FAILURE`), compact `serde_json` output, and a shared table renderer keyed to `COLUMNS.len()` | verified | apps/discovery-cli/src/main.rs:34-83,253-267,377-454 |
| A12 | `apps/discovery-cli` already depends on `discovery`, `bindforge-core`, `clap`, `serde`, `serde_json`; `serde_json` is already resolved in `Cargo.lock`, and workspace members are the `crates/*` + `apps/*` globs, so a new `crates/codecs` needs no glob change and no new external dependency | verified | apps/discovery-cli/Cargo.toml:6-22; Cargo.toml:12; crates/discovery/Cargo.toml:15-16 |
| A13 | `list` refuses with a named error *before constructing any plugin* when no catalog endpoint is configured, then probes it and queries every unresolved source key; the `bindings` face therefore could never run where no service is reachable | verified | apps/discovery-cli/src/main.rs:88-109,194-199 |
| A14 | `bindforge-core` depends on nothing and `bindforge-transforms` depends on core and nothing else — the crate-split precedent ADR-017 cites as "clause 3" is clause 2's workspace list plus ADR-009's "`crates/transforms` owns the policy machinery" | verified | crates/core/src/lib.rs:1-5; crates/transforms/Cargo.toml:9-13; docs/adrs/008-rust-for-the-product.md:36-57; docs/adrs/009-canonical-profile-and-manifest-model.md:142-148 |
| A15 | UE ships user config under `Saved/Config/<Platform>/`, its `Input`/`PlayerInput` category carries `Bindings=(Name=…,Command=…)` lines with documented `+`/`-`/`.`/`!` merge operators and bottom-most-wins for one key, its per-category hierarchy runs base → platform → user, and the platform directory name is not one constant across editor and packaged builds | unverified | https://dev.epicgames.com/documentation/unreal-engine/configuration-files?application_version=4.27; https://hyaniner.com/en/blog/config-ini-of-ue-5.5-20250218 |
| A16 | UE 5.3+ `UEnhancedInputUserSettings` persists user remaps as a saved-game object under `Saved/SaveGames/`, invisible to a codec whose grammar is the `Saved/Config` INI layout | unverified | https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Plugins/EnhancedInput/UEnhancedInputUserSettings |
| A17 | A Proton prefix's user directory is always `steamuser` (never the host username), prefixes live per library at `<library>/steamapps/compatdata/<appid>/`, and prefix trees are held together by symlinks that break on NTFS/exFAT libraries | unverified | https://checkpoint64.com/blog/steam-deck-save-file-location; https://www.gamingonlinux.com/guides/view/how-to-find-where-steam-stores-proton-game-files; https://forum.endeavouros.com/t/multiple-steam-proton-games-no-longer-launching/16692 |
| A18 | Steam's per-user game data is keyed `<main root>/userdata/<steamid>/<appid>/` — under the main root, not the library folder the game sits in — and no scan output marks which resolved root is the main one | unverified | https://partner.steamgames.com/doc/features/steam_controller/iga_file; https://www.pcgamingwiki.com/wiki/Glossary:Game_data |
| A19 | UE may write its INI files as UTF-16 (BOM in either endianness, BOM-absent behaviour platform-dependent), while today's shared reader only yields strict UTF-8 | unverified (UE) / verified (reader) | https://dev.epicgames.com/documentation/unreal-engine/character-encoding-in-unreal-engine; crates/discovery/src/steam_common.rs:329-359 |
| A20 | Real binding stores name inputs the closed set refuses — `MouseX`, `MouseY`, `Gamepad_RightX/Y`, `TAB`, `ALT`, `CAPSLOCK`, `PGUP`, `DOWNARROW`, F-keys | unverified | https://github.com/martiancraft/Unreal-Engine-4-Plugins/blob/master/Plugins%20Example%20Project/Config/DefaultInput.ini; https://github.com/acx0/cs2-cfg/blob/master/csgo-autoexec.cfg |
| A21 | No appid→family table exists anywhere in the tree: `data/` holds two READMEs, a catalog seed and its README, and nothing else | verified | data/ tree (`data/profiles/README.md`, `data/manifests/README.md`, `data/catalog/README.md`, `data/catalog/v1/catalog.json`) |
| A22 | CI runs the cargo trio plus a Windows *compile* check (`x86_64-pc-windows-msvc`, excluding `bindforge-registry`) on `ubuntu-latest`, and the Node check; no leg runs a Windows or macOS test binary | verified | .github/workflows/check.yml:25-59,61-81 |
| A23 | `017-S1`'s "two locally installed Unreal-lineage titles" are unnamed in the ADR, and a pipeline run cannot install games | verified | docs/adrs/017-keybinding-extraction-codecs.md:166-170 |

## Options considered

**The snapshot's format: JSON, TOML, or an internal CLI shape.** An internal
shape is the ADR's own rejected option — a thing only a developer's terminal
sees never has to age, and this document is the input to manifest authoring,
the apply tool and a future UI import. TOML keeps "Formats are TOML" true and
needs zero harness change (the `schemas/*/v*/` walk picks a new directory up by
itself), but `017-C4` says `--json` prints *the snapshot document verbatim*, so
TOML would leave the contract with two serializations of one fact — the drift
channel the closed-set posture exists to refuse — and `017-L4`'s
"byte-identical" claim would have to name a form. It also cannot express the
shape at all: the documented TOML subset refuses table headers and dotted keys,
so nested entries and provenance records would need a second space-joined row
encoding whose free-text column is a game's own action name *and* a file path.
Chosen: **JSON**, the second named JSON format, amending `schemas/README.md`'s
"One format is JSON…" passage exactly as ADR-026's `catalog` amendment did
(A10), with `catalog`'s posture kept: the loader that owns the document states
its refusals, and the harness's single JSON-const carve-out grows by one name.

**How a snapshot spells an input.** (a) As written in `017-C3` — an ADR-012
input id: rejected on A5 + A20. A first extraction of a real Unreal title hits
`MouseX`, `Left`/`Right` arrows and `Gamepad_RightX` on its first file, so the
closed set forces a choice between dropping those bindings (a false statement
of the game's surface, against `017-L3`) and refusing the snapshot (the death of
this run's purpose, which is first manifests). (b) A per-entry named refusal
list: rejected — it invents a second outcome class the ADR's three
(claim / claim+refusal / no claim) do not carry, and it still loses the fact.
(c) Widen the ADR-012 member sets: rejected — the sets' home is
`schemas/reach-classes/v2/spec.md`, widening "widens what every validator
accepts" and rides a vocabulary major ADR-012 owns, and reach grades would have
to be authored for every new member. (d) Chosen: **native tokens verbatim**, so
both halves of a binding row are the file's own words and the format performs no
translation at all. This is `017-L3` applied uniformly instead of to the action
half only, and it is the smaller design: no refusal class, no lossy path, no
ADR-012 edit. Cost, recorded: a snapshot carries no input ids, so the
native-token→input-id join is a consumer's job (as the action-id join already
is, `013-C4`), and the vocabulary gap is *named* rather than papered over.

**What the codec is handed, given `discover()` takes no arguments.** (a) Change
the `DiscoveryPlugin` trait: rejected — the no-argument signature is a
documented constraint (T-001 A17, A1), it would re-pin every plugin suite, and
it puts roots in front of third-party plugins that cannot use them. (b) Have the
CLI re-probe Steam roots itself for the bindings face: rejected — it duplicates
`host_candidates` and the library enumeration a second time and lets two trees
of code disagree about which roots exist. (c) Chosen: an **additive
`scan_records()` inherent seam** on the Steam plugins that returns the row
beside its owning root and library, with `discover()` becoming a projection over
it. `DiscoveredGame` and the trait are untouched, so `list`'s byte-pinned
renderings cannot change (§Discovery plumbing), and `roots()` hands the same
selected set to `GameContext.roots`.

**Does the bindings face run `list`'s whole scan?** (a) Yes, gate included, as
`017-C4`'s "the same scan" reads literally: rejected on A13 — it makes a
local-extraction command refuse whenever no catalog is configured, sends the
queried AppID to whatever endpoint is set, and contradicts the ADR's own
"privacy stays local by construction". (b) Add an `--offline` flag to `list` and
reuse it: rejected — it changes `list`'s pinned gate for no `list` benefit.
(c) Chosen: the bindings face runs the same **enumeration** — same plugin
construction, same roots, same emit boundary — and no network leg at all,
because extraction consumes only an install path and the roots, which are local
facts. The divergence from `017-C4`'s wording is recorded in the Revision log.

**Discharging `013-R3` (registry-check `codec` ids).** (a) Have `load_manifest`
check `codec` against the new registry: rejected twice over — `bindforge-core`
depends on nothing (A14), so a check there inverts the graph, and with a closed
implementation set of size one every manifest naming a future family becomes
unloadable, which is a corpus-wide breakage with no benefit to any consumer.
(b) Chosen: this run supplies the *set* — `crates/codecs::registry::family_ids`
and `is_known_family` — and the *construction-time* guarantee that a snapshot's
`codec` value is a registry id (it is produced by the claiming family, never
typed), while a load-time check on curated manifests rides the routing-data
family set (`017-R4`). `013-R3` is therefore narrowed, not silently dropped.

**Order-dependent and duplicate bindings.** (a) Fold to the engine's effective
answer (UE's documented bottom-most-wins): rejected — the ADR defines a snapshot
as "the bindings a game's own files state", precedence is engine behaviour
outside the located file, and modelling it means the codec interprets rather
than reports (`017-L3`). (b) Chosen: record every distinct (native action,
native input) pair the located **user** files state, grouped by action; dedupe
byte-identical pairs only; represent UE's documented `-Bindings` removals as
removals from the working set, which is the file's own merge operator rather
than an interpretation. Shadowing is then not modelled in `v1` (§Deferred).

**Where the appid table lives.** (a) A `data/` file: rejected — that is
`017-R4`'s routing *data*, whose owner is unassigned. (b) A registry-side table:
rejected — the registry must stay family-agnostic to keep its id-order iteration
and first-claim rule generic. (c) Chosen: `appids()` on the family, empty for
`unreal-ini` this run (A21: no appid knowledge is surveyed into the tree, and
inventing two AppIDs would be fabricated content). The appid pass, its
precedence over tree probing, and its evidence spelling are built and pinned now
against a synthetic test family, so `017-R4` lands data into a seam instead of
prompting a redesign.

## Chosen design

### `crates/codecs` — layout and dependencies

New package `bindforge-codecs` at `crates/codecs`, picked up by the existing
`crates/*` glob, with the third-party-crate posture of
`crates/transforms/Cargo.toml` (a `description`, workspace-inherited version and
edition).

- `src/lib.rs` — module surface and the crate's public-surface paragraph.
- `src/codec.rs` — `CodecFamily`, `Detection`, `UserCandidates`, `Evidence`,
  `ExtractionRefusal`.
- `src/context.rs` — `GameContext` and its `from_record` constructor.
- `src/registry.rs` — the closed static set, `claim`, `family_ids`,
  `is_known_family`.
- `src/paths.rs` — `TreeKind`, `UserTree`, `user_trees` (the resolvers).
- `src/read.rs` — the size-capped, BOM-sniffing reader.
- `src/snapshot.rs` — the `bindings/v1` model, its budgets, and its loader.
- `src/families/unreal_ini.rs` — the first family.
- `tests/common/mod.rs` (fixture-tree and synthetic-family helpers) and
  `tests/{registry,context,paths,read,snapshot,unreal_ini,determinism,honesty,zz_api_probe}.rs`
  mirroring `src/` (ADR-004).
- Dependencies: `bindforge-core` (workspace), `discovery` (path), `serde`
  (workspace, derive); dev-dependency `serde_json` (A12) — **no new external
  package and no resolved version moves: the only `Cargo.lock` delta is the
  `bindforge-codecs` stanza and the `discovery-cli` dependency edge**.
  `crates/codecs` depends on
  `discovery` because ADR-017's own Alternatives sentence names that data flow
  ("Discovery feeds the `GameContext`; codecs read files"); nothing in it
  constructs a plugin or scans an install.

### The codec face — `src/codec.rs` (`017-C1`)

```rust
pub trait CodecFamily {
    /// The family id: slug grammar, `MAX_SLUG_OCTETS` — the value a snapshot's
    /// `codec` records and the key the registry iterates by.
    fn id(&self) -> &'static str;
    /// Appids this family claims by table (pass 1). May be empty.
    fn appids(&self) -> &'static [&'static str];
    fn detect(&self, ctx: &GameContext) -> Option<Detection>;
    fn extract(&self, claim: &Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal>;
}

pub struct Detection {
    pub family: &'static str,
    pub evidence: Evidence,                  // AppidTable | InstallTreeProbe
    pub per_user: Vec<UserCandidates>,       // one per located tree
}
pub struct UserCandidates { pub user: String, pub files: Vec<PathBuf>,
    pub tree_root: PathBuf } // that tree's `UserTree.root`: §Determinism's dedup
                             // and ordering key, never rendered into a document
```

A claim is never a boolean: `Detection` carries the id, the candidate files
grouped per located user tree, and the evidence. No claim, a claim, and a claim
whose parse refused stay three distinct outcomes end to end (`017-C1`);
`per_user` is in `user_trees` order and `extract`'s `Vec` is positionally
parallel to it — one document per located tree (`017-C3`) as a type-level fact.

**Codec family id (canonical).** A family id is `[a-z][a-z0-9-]*` of at most
`bindforge_core::MAX_SLUG_OCTETS` octets — the bound the manifest `codec` entry
already rides (A6), which is what makes `unreal-ini` legal in a snapshot and in
`data/manifests/…/manifest.toml` at once. The ADR's "013-B4 bounds" is a slip
(A6): `013-B4` is the native-name bound. Every other section cites this one.

### Discovery plumbing — `crates/discovery`

- `src/game.rs` gains `pub struct ScanRecord { pub game: DiscoveredGame, pub
  root: PathBuf, pub library: PathBuf }` — the row plus the Steam root that
  *owns* the library it was found in (the root itself, or the root whose
  `libraryfolders.vdf` named it).
- `src/steam_common.rs`: `SteamCore::scan()` returns `Vec<ScanRecord>`;
  `discover()` becomes `Ok(self.scan()?.into_iter().map(|r| r.game).collect())`.
  Inside `scan`, libraries are enumerated per root in `roots()` order and a
  library two roots share is scanned once, owned by the first root that named it
  — the same dedupe A2 describes, now with its owner kept. The row sort stays the
  `(slug, install_path)` sort, unchanged, and the §"one error path" clause stays
  in `scan()`.
- `SteamLinuxPlugin`/`SteamWindowsPlugin` expose `scan_records()` and `roots()`
  as thin inherent wrappers; `roots()` is the selected set as the core holds it
  — canonicalized, existence-filtered, byte order, not the candidate list.
  `plugin.rs`'s `DiscoveryPlugin` trait is not modified: third-party plugins keep
  `discover()` and simply have no bindings face this run, which is consistent
  with Steam being the only source with an emitter.
- `DiscoveredGame`, its serde shape, `COLUMNS`, `json_row` and the emit boundary
  are untouched (`017-L5`).

### `GameContext` — `src/context.rs` (`017-C2`)

```rust
pub struct GameContext {
    pub lookup_key: String,   // "<source>/<external-id>", byte-for-byte the row's
    pub appid: String,        // the external id: 1..=20 ASCII digits
    pub install_path: PathBuf,
    pub platform: discovery::game::Platform,
    pub roots: Vec<PathBuf>,  // every root the scan resolved, core byte order
    pub library: PathBuf,     // the owning library: `steamapps/compatdata/` here
}
impl GameContext {
    /// `None` unless the slug is a `steam` lookup key with a digit appid.
    pub fn from_record(record: &ScanRecord, roots: &[PathBuf]) -> Option<GameContext>;
}
```

`roots` is `017-C2`'s whole resolved set, read differently than its "main root
and library roots separately" wording: A18 puts `userdata/` under a root no scan
output labels, so every root is searched and a tree is used only if it exists,
in the core's canonicalized byte order (§Discovery plumbing).

`from_record` is the whole gate: `Source::spells_lookup_key` with the `steam`
token and a 1–20-digit external id (mirroring the appid rule
`steam_common.rs:26` already enforces at the scan seam). A row that is not a
Steam install yields no context, and the face answers `claim-refused` — no
codec sees a half-parsed identity. A codec reads nothing outside the candidate
set `detect` located and locates nothing outside the trees `user_trees` returns.

### Config-path resolvers — `src/paths.rs`

```rust
pub enum TreeKind { SteamUserdata, ProtonPrefix }
pub struct UserTree { pub kind: TreeKind, pub user: String, pub root: PathBuf }
pub fn user_trees(ctx: &GameContext) -> Vec<UserTree>;
```

Two template families, each returned only when it exists, in `TreeKind`
declaration order then byte order of the tree's `root` — a total order over the
trees, which §Determinism relies on — and each built from a `GameContext` anchor:

- `SteamUserdata`: `<root>/userdata/<dir>/<ctx.appid>` for every `root` in
  `ctx.roots` in `roots()` order, one per digit-named `<dir>` in byte order,
  keeping at most `MAX_USER_DIRS_PER_GAME` such trees in that enumeration order
  across all roots; `user` is the directory name (the local SteamID).
- `ProtonPrefix`: `<ctx.library>/steamapps/compatdata/<ctx.appid>/pfx/drive_c/users/steamuser`,
  with `user` the constant `steamuser` — A17's finding that Proton never uses
  the host username, which is why this is a template and not a glob of
  `drive_c/users/*`.

No host-config tree ships this run: `$HOME/.config` is shared by every Unreal
title and no `ScanRecord` field names the project directory inside it
(`017-L3`); §Deferred carries that context and the owner that would justify it.

A `userdata` directory name outside 1–20 digits is not a SteamID and the tree is
skipped, and so is a tree whose own final component is a link
(`DirEntry::file_type()` on an enumerated `<dir>`, `symlink_metadata` on a
template root), counted as §Read discipline counts a link in the walk. Candidate
paths are never canonicalized: the anchors (`roots`, `library`) are already
canonical from the scan and no link is followed at an anchor or below one, so a
walk never leaves the tree it started in.

### Read discipline and budgets — `src/read.rs` (`017-L1`, `017-B1`–`017-B3`)

Budgets, exported by `bindforge-codecs` (not core — the README's "exports all
thirteen constants" sentence stays core's):

| Constant | Bounds | Source |
|---|---|---|
| `MAX_BINDING_FILE_OCTETS = 1_048_576` | one file a codec opens | `017-B1` |
| `MAX_SNAPSHOT_ENTRIES = 4096` | one snapshot's distinct (`action`, input-token) pairs as admitted | `017-B2` |
| `MAX_CANDIDATE_FILES = 32` | one user tree's located candidate set | `017-B3` |
| `MAX_USER_DIRS_PER_GAME = 16` | `SteamUserdata` trees kept per game, across all of `ctx.roots` | design-time |
| `MAX_LOCATE_DEPTH = 8` | components in a located file's tree-relative path — its last four are `Saved/Config/<dir>/<name>.ini` | design-time |
| `MAX_LOCATE_ENTRIES_PER_TREE = 16_384` | entries the locate walk examines below one `UserTree` root | design-time |
| `MAX_SNAPSHOT_OCTETS = 4_194_304` | one snapshot document the loader reads | design-time |

No aggregate constant, one aggregate sentence in `schemas/README.md`'s budget
family, in the shape `MAX_CHAIN_DEPTH × MAX_DOCUMENT_BYTES` already uses: one
extraction opens at most `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES`
files — the `+1` is the `ProtonPrefix` tree, and since §The first family caps a
tree's reads at `MAX_CANDIDATE_FILES` the product also caps the *opens*, so
the worst case is `17 × 32 × MAX_BINDING_FILE_OCTETS` (544 MiB) per game,
offline, repeating per matching record the face walks; located sets in the
fixtures are 1–3 files. `MAX_USER_DIRS_PER_GAME` is the one silent bound — past
the 16th tree a game's remaining local users are simply absent — and `spec.md`
and the README budget bullet state it as such.

`read::open_capped(path)` applies the A4 posture (metadata length against
`MAX_BINDING_FILE_OCTETS` **before** allocation, then a bounded read), and
`read::decode(bytes)` does BOM sniffing: a UTF-8 BOM is stripped, a UTF-16LE/BE
BOM decodes to UTF-8 with an unpaired surrogate refused, and no BOM means strict
UTF-8 (A19). Named `ExtractionRefusal` variants, each printed by name and none
of them an empty snapshot: `FileOverBudget`, `UndecodableText`,
`UnparsableEntry` (naming the file's tree-relative path and its 1-based line —
and the bound's name when the verdict is a verbatim `Command=` or `Name=` string
over `MAX_NATIVE_NAME_OCTETS` — escaped and truncated through
`bindforge_core::escape_and_truncate`), and `SnapshotEntryBudget`, checked against
the distinct pairs a tree's fold has ever admitted — a removal frees no room —
so the fold stops at `MAX_SNAPSHOT_ENTRIES + 1` admitted pairs and names itself.

Never-write (`017-L1`): every open is `std::fs::OpenOptions::new().read(true)`
and nothing else; no code path in the crate takes a `&mut` to a path, creates a
directory, or writes a byte. Locate walks take `DirEntry::file_type()` — which
does not follow symlinks — so a link is never descended, never read as a file,
and never taken as a tree anchor (§Config-path resolvers); a link, an
over-deep directory or an entry past `MAX_LOCATE_ENTRIES_PER_TREE` is skipped and
counted.

### `bindings/v1` — the snapshot contract (`017-C3`)

One JSON document per game per located user tree; two trees may share a `user`
label, and then nothing but `entries` separates their documents, so a consumer
unions same-`user` documents' entries. Top-level keys, closed:
`format` (`"bindings"`), `version` (bare integer `1`, the `catalog` spelling),
`codec` (a registry family id), `game` (the lookup key), `user` (the context
label, absent when the family's storage is not user-scoped), `entries`
(`[{action, inputs}]`), `provenance` (`{evidence, files}`). Entry keys are
closed to `{action, inputs}`; `files` are tree-relative paths, sorted byte order
(the portability rule: the same layout states the same document on any machine;
`--probe` prints absolute paths instead, §The CLI face). Unknown key, missing
required key, or a value outside its bound is a refusal naming the key, and the
`snapshot.rs` loader checks `fs::metadata` length against `MAX_SNAPSHOT_OCTETS`
**before** the read — the posture §Read discipline and `catalog` keep — so an
oversized document is never buffered.

Bounds, stated once here and cited elsewhere: `codec` per §Codec family id;
`game` at most `MAX_DISCOVERY_SLUG_OCTETS`; `action` and every input token
1–`MAX_NATIVE_NAME_OCTETS` — deliberately the *manifest's* native-name bound, so
an unjoinable action never appears; no ASCII control character and no leading or
trailing space in any string (the manifest rule, kept so the table renderer needs
no second escape path); `user` rides that same token grammar rather than a closed
list of the labels §Config-path resolvers builds, so another launcher context's
name costs no format major; the (`action`, input-token) pair total ≤
`MAX_SNAPSHOT_ENTRIES`, which bounds `entries` and every `inputs` array; `files`
≤ `MAX_CANDIDATE_FILES`, each entry at most `MAX_LOCATE_DEPTH` tree-relative
components (§Read discipline's table); the document ≤ `MAX_SNAPSHOT_OCTETS`.

**Input spelling (canonical).** `inputs` are the located file's own input tokens
verbatim, and `bindings/v1` carries no ADR-012 input ids anywhere. A snapshot is
pre-taxonomy by definition — `017-L3` makes the taxonomy join the manifest's —
so both halves of a row are reported in the game's words and nothing in this
format validates against the closed member sets (A5, A20; Options).

`provenance.evidence` is the claiming pass's evidence token (`appid-table` or
`install-tree-probe`), which is what lets an artifact say which family claimed a
game and on what basis (`017-C5`). `schemas/bindings/v1/spec.md` states the key
set, the bounds, the ordering law (cited from §Determinism, never restated), the
refusal names, and that `user` is launcher-local identity a sharing surface
treats as personal data; `fixtures/valid/` and `fixtures/invalid/` are the
conformance currency.

Contract-tree amendments, all additive in the "no new refusal on an existing
document" sense:

1. `schemas/README.md` — the "One format is JSON" passage becomes the two-format
   passage naming `bindings` with the same loader-owns-its-refusals posture, and
   the source-key-content sentence names `schemas/bindings/` beside
   `schemas/catalog/` (and *not* `data/bindings/`: this run ships no snapshot
   under `data/`). A budget bullet states the budget table above, names
   `bindforge-codecs` as their exporter, and states the aggregate product.
2. `crates/core/tests/common/mod.rs` — `JSON_FIXTURE_FORMAT` becomes
   `JSON_FIXTURE_FORMATS: [&str; 2] = ["catalog", "bindings"]`, with
   `is_core_toml_format` and `is_fixture_name` reading it, so the TOML parse and
   rejection loops skip bindings' `.json` fixtures while the zero-fixture and
   case-unique-name rules keep covering them (A10's carve-out shape, unchanged).
3. `schemas/manifests/v1/spec.md` — the Fingerprint grammar paragraph, which
   today punts "which bytes are hashed and what a mismatch mean" to ADR-017
   (A8), gets the answer: the hashed bytes are the located files' octets
   concatenated in the snapshot's sorted `provenance.files` order, with no
   separator and no path or length octets; a mismatch means the config behind
   the manifest is not the config it was read from — a re-curate signal, never a
   load-time rule (`013-L7` unchanged). This discharges `013-R2`'s question;
   computing a digest is the apply-and-authoring face's (`017-R1`). No fixture
   and no existing refusal changes. The `codec` row's "implementations are
   ADR-017" note gains the registry's path.
4. `crates/discovery/tests/boundary_tests.rs` — `SOURCE_KEY_TREE` becomes
   `SOURCE_KEY_TREES: [&str; 2] = ["catalog", "bindings"]`, with the same reason
   `catalog` has (the format's content *is* the key) and the sweep otherwise
   unchanged (A9).

**Corpus boundary (canonical).** A `bindings/v1` document is machine-local
output that this run only prints: it writes no file, so `025-L5`'s boundary — no
source key in a shipped artifact or path component — is preserved by there being
nothing to preserve, and the only relaxation anywhere is fixture content.

### The registry — `src/registry.rs` (`017-C5`)

`pub static REGISTRY: &[&dyn CodecFamily] = &[&unreal_ini::UnrealIni];`, declared
in family-id byte order, with a unit test asserting the declaration *is*
byte-sorted so the ordering law cannot rot.

`claim(ctx)` runs two passes over that slice and the first claim wins:

1. **Appid pass** — families whose `appids()` contain `ctx.appid`, in id order;
   `detect` runs only for those, and a claim carries `Evidence::AppidTable`.
2. **Probe pass** — runs only if pass 1 claimed nothing: every family in id
   order; a claim carries `Evidence::InstallTreeProbe`.

The two passes answer the "what is a family" question once, for every future
family: a family id names a **storage grammar**, not a lineage. An
appid-table hit claims regardless of what the grammar's files turn out to
contain — that is exactly `017-L2`'s family-confirmed-but-files-not-found case,
and it is the visible one. A tree probe claims only when it located at least one
file in its own grammar. So an Enhanced-Input-era Unreal title (A16) is a
claim-with-nothing-to-read, printed as its own status and made actionable by
`--probe`, and the next family (`017-R3`) is additive rather than a split of the
first.

`family_ids()` and `is_known_family(id)` publish the closed set for consumers —
the half of `013-R3` (A7) this run makes available without putting a dependency
edge in core (Options).

### The first family — `src/families/unreal_ini.rs` (`017-S1`)

- `id()` → `"unreal-ini"` (slug grammar, 10 octets). `appids()` → `&[]`, empty
  this run on A21; the pass that reads it is pinned by the synthetic family in
  `tests/common/mod.rs`.
- `detect` walks each `UserTree` depth-first in byte order, pruning a directory
  whose tree-relative path already has `MAX_LOCATE_DEPTH` components, and
  examining at most `MAX_LOCATE_ENTRIES_PER_TREE` entries per tree. A candidate
  is an `*.ini` whose `Path::components()` end component-wise in
  `Saved/Config/<one directory>/<name>.ini` and whose contents hold a
  `[/Script/Engine.PlayerInput]` section with a `Bindings=` line. At most
  `MAX_CANDIDATE_FILES` path-grammar matches per tree, in the walk's byte order,
  are opened through `read::open_capped` and `read::decode` for that content
  test; a further match is skipped and counted the way an entry past
  `MAX_LOCATE_ENTRIES_PER_TREE` is. A match the reader refuses cannot pass the
  content test either, so it is located on path grammar alone and `extract`
  refuses it by name; a decodable file without the section is not located. No
  platform-directory constant is assumed (A15), no UE project name is guessed,
  and the walk prunes on nothing but depth and entry count. Claims iff located.
- `extract` reads only located files, through `read.rs`. Accepted lines:
  `[Section]` headers and `Key=Value` under UE's four documented merge operators —
  `+` adds, `.` adds and may duplicate, `-` removes on an exact match, `!` removes
  by name — which build the working set; a bare `!Key=` (no value) clears every
  pair accumulated for that key, `!Key=<tuple>` removes only the pairs it names,
  and a bare `+`, `-` or `.` operand is out-of-grammar and skipped; a `Command`
  whose pairs the fold has all removed leaves no entry. Blank lines and
  out-of-grammar lines in a section are skipped, and a
  `Bindings=(…)` tuple that lacks a quoted `Name=` or `Command=`, carries an
  unexpected key, or is unbalanced is `UnparsableEntry` — drift becomes loud
  rather than a silently shorter snapshot. Entries are keyed by the `Command`
  string verbatim (the native action or command name, `017-C3`) and carry the
  distinct `Name` tokens verbatim as `inputs`.
- The install tree is never a candidate tree, so a `DefaultInput.ini` there can
  contribute no entry: "defaults are not absorbed" holds structurally.
- Fixture trees under `crates/codecs/tests/fixtures/unreal/` encode the *layout
  claim*, not one machine (the research's Proton-drift finding): `proton-prefix`,
  `steam-userdata`, `bom-utf16`, `defaults-only`, `bindings-absent`,
  `broken-tuple`, `symlinked-prefix` — whose links, including a symlinked tree
  root, are made at test time behind `#[cfg(unix)]` with a `#[cfg(not(unix))]`
  early return, so the Windows compile leg stays green — each with a small INI.
- `017-S1`'s read-only verification pass against two locally installed titles is
  an operator-run step recorded in `03-build-notes.md` (A23): the run cannot
  install games, so the fixtures are the gate and the pass is a manual
  confirmation with its output pasted in the notes, never a test assertion.

### Determinism (`017-L4`) — canonical

One law, stated once in `schemas/bindings/v1/spec.md` and implemented once:
documents and their user-tree groups in `user_trees` order — the total order over
located trees of §Config-path resolvers; within a document, `entries` sorted by
`action` byte order, each entry's `inputs` deduplicated and sorted byte order,
`provenance.files` sorted byte order; across documents, deduplicated by each
document's `UserCandidates.tree_root` — tree identity, never rendered — the
first stating it once; each file's entries folded in byte order of its relative
path. JSON is `serde_json::to_string` (compact, struct-declared key order); the
table goes through the shared width algorithm. Nothing reads wall-clock time or
the process
environment — `user_trees` takes the context alone — and neither registry
iteration order nor directory-return order reaches an artifact: `claim` returns
the first claim in id order, and every collection that reaches a document is
sorted on the way in.

### The CLI face — `apps/discovery-cli/src/bindings.rs` (`017-C4`, `017-L2`, `017-L5`)

`Commands::Bindings { slug: String, #[arg(long)] probe: bool,
#[arg(long = "steam-root")] steam_root: Vec<PathBuf> }`.

1. Build the Steam plugins exactly as `list` does (same host probe, same
   `--steam-root` override and its `cfg(windows)` selection) — a concretely typed
   copy of that selection, since `scan_records()` and `roots()` are inherent
   methods and not on the trait — and keep `roots()` beside the records.
   **No catalog client, no probe, no match store, no prompt, no candidate
   query** — the divergence from `017-C4`'s "the same scan" is Options' and
   A13's, and it is what makes the ADR's privacy consequence true of this face.
2. Select records whose `game.slug` equals `slug` (compared as given; the slug is
   the lookup key the operator read off `list`); no match is the ordinary failure
   voice with `EXIT_FAILURE`, echoing the escaped slug and no path.
3. Per matching record, in `(slug, install_path)` order:
   `GameContext::from_record(record, roots)` → `registry::claim` → `extract`
   (skipped under `--probe`). More than one record matching (one game in two
   libraries) is not an error: each is processed, and §Determinism's
   dedup by tree identity keeps one tree stated once.
4. Render: the shared table renderer, generalized from `COLUMNS.len()` to a
   `&[String]` header so `list`'s five columns and their pinned widths are
   unchanged. `bindings` columns are `Codec | User | Action | Inputs`;
   `--probe` columns are `Family | Evidence | User | File` with absolute paths,
   which is the local-operator-tool scoping decision below. `--json` prints one
   compact JSON array of snapshot documents; `--probe --json` prints
   `{"status":…,"claims":[…]}`.
5. Statuses and exit codes, `017-L2`'s two named statuses among them, each above
   the code clap's parser returns for a usage error before `main` runs, so no
   status can be confused with one: `ok` 0 · `EXIT_FAILURE` 1 (undiscovered slug,
   scan failure — today's one voice) · clap's usage error 2 · `claim-refused` 3 ·
   `family-confirmed-files-absent` 4 · `extraction-refused` 5 (its `error: ` line
   carries the `ExtractionRefusal` name). Each non-ok status prints exactly one
   `status: <name>` line on stderr
   (stdout stays the table or the document) and exits its own code; a run over
   several records exits the highest-ranked status it reported
   (`extraction-refused` > `family-confirmed-files-absent` > `claim-refused` >
   `ok`), so the code is a function of what stdout showed.
6. `discovery list` is untouched: its plugin loop, gate, store, prompt, columns,
   JSON keys and exit code stay as shipped, and `codec_id` keeps spelling
   `null` because nothing ever assigns it. The ADR's "`codec_id` leaves `None`
   only on the face this ADR adds" is satisfied on that face's own rendering,
   which carries a `Codec` column sourced from the snapshot — the row type stays
   out of it, which is what keeps `017-L5` provable rather than probable.

**Local paths (scoping).** `013-L5`'s "the refusal never echoes a local path"
and `schemas/README.md`'s no-path-in-a-message rule govern messages about
documents that arrived from elsewhere. This face is a local operator tool whose
`--probe` mode exists to name files, so it prints absolute paths in probe output
and tree-relative ones in the contract document; nothing it prints is a refusal
*about* an outside document, and every value from a file still goes through
`escape_and_truncate`. Probe lines also carry the OS username and the SteamID:
that output is for whoever is sitting at the machine, not for a public report.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| A snapshot carries no input ids, so every consumer does the native-token→input-id join and the ADR-012 gap stays open | Certain (chosen) | Medium | Options' reasoning: the closed set cannot spell what the files contain (A5, A20); the join is already the manifest's for actions (`013-C4`); widening ADR-012 is Deferred to its own vocabulary major, and a later `bindings` major can add canonical spellings beside the verbatim ones |
| The first family covers only the `PlayerInput Bindings` era, so modern UE titles answer `family-confirmed-files-absent` | High (A16) | Medium | That status is printed as itself, `--probe` shows the located files, and the fix is an additive family (`017-R3`); the operator verification pass reports how often it bites |
| Proton layout drift (Valve moves the layout; symlinked prefixes break on NTFS/exFAT) | Medium | Medium | Fixtures encode the layout claim per context; links are never descended and never taken as a tree anchor, so a broken prefix degrades to files-absent instead of hanging or escaping; the resolvers' two templates are named constants in one module |
| The appid pass is dead in shipped configuration (no family carries appids) | Certain (A21) | Low | The mechanism, ordering and evidence spelling are pinned by a synthetic family with an appid; `017-R4` supplies data into a built seam |
| Untrusted config files: enormous, deeply nested, UTF-16, symlinked out of the tree, over budget | Medium | Medium | Pre-read length cap before allocation, the named budgets of §Read discipline, depth-, entry- and per-tree-capped walks, no link descent and no link tree anchors, closed key sets, bounded escaped echoes, and `017-L1`'s never-write with a bytes-unchanged test |
| The `steam_common` refactor changes what `list` prints | Low | High | `discover()` stays a projection over `scan()`, the row sort and error clause are untouched, `DiscoveredGame` does not change, and the pre-existing `list` suites run unmodified as a gate item |
| A second JSON format lets the harness carve-out drift | Low | Medium | One `JSON_FIXTURE_FORMATS` const is the only name skip sites read (A10); `schemas/README.md` names both formats in the passage the rule lives in |
| Nine clauses read differently than worded, with `docs/adrs/` operator-owned | Medium | Low | Each divergence is a Revision-log entry with its basis, the unit ids are listed for the operator's ledger annotation (§Rollout item 8), and `03-build-notes.md` repeats them |
| Probe output leaks paths, the OS username and the SteamID where a reviewer expects the no-path discipline | Low | Low | §Local paths states which rule governs which surface and that probe output is operator-local, and Test strategy #13 pins both |

## Compatibility matrix

| Platform / environment | Supported? | Notes |
|---|---|---|
| Linux (ext4), Steam install | Yes | The full leg: the whole suite runs here (A22); the `SteamUserdata` and `ProtonPrefix` resolvers are exercised against fixture trees. A title whose bindings live only in the host config home is `claim-refused` (§Deferred) |
| Steam library on NTFS/exFAT with symlinked prefixes | Yes, with a named degradation | Links are never descended and never anchors, so a broken prefix reads as `family-confirmed-files-absent` (exit 4), never a hang or an out-of-tree read (A17) |
| Proton-launched game (Steam Play) | Yes | `steamuser` is a template constant, prefix under the owning library (A17); fixture tree `proton-prefix` pins it |
| Flatpak / Snap Steam | Yes, for the root-relative trees | The face inherits `host_candidates` roots (A3) unchanged and both templates hang off a resolved root, not `$HOME`; the sandboxed launcher's own config home is not searched, so a host-config title answers `claim-refused` (§Deferred) |
| Steam Deck | Yes, by the Linux row | A Linux Steam install with the root-relative `userdata/` and Proton prefix layout the two rows above claim, the host config home deferred as they state; unexercised here — no Deck leg (A22) and no Deck fixture |
| Windows host | Compiles, claims nothing | In the `x86_64-pc-windows-msvc` compile check (A22); the root-relative trees resolve and are stat'd, but no template ships for the Windows host config home, so a real title locates no grammar and answers `claim-refused` — `017-R2` owns `%LOCALAPPDATA%` and `Documents\My Games` |
| macOS | No | `discovery::game::Platform` is exactly `SteamLinux`/`SteamWindows` and the CLI builds only those two plugins, so no macOS row ever reaches `GameContext::from_record`; the macOS Steam root is `017-R2`'s |
| Non-Steam sources (`epic`, `local`) | No | `GameContext::from_record` requires a `steam` key with a digit appid; any other row is `claim-refused`. The discovery sources are `022`'s and `025`'s |
| Unreal Enhanced-Input / `ActionMappings`-era titles | No (named status) | Grammar not located, so `017-L2`; the family for them is `017-R3` (A16) |
| Source-family titles (`bind`/`alias` under `userdata`) | No | Non-claim this run; `017-R3`. The `{action, inputs}` shape keeps key→command expressible because both halves are verbatim strings |
| `discovery list` | Not affected | Rows, columns, JSON keys, gate, store and exit code unchanged; `codec_id` stays `null` (`017-L5`) |
| Third-party schema readers | Additive | One new format directory with its own `spec.md` and `.json` fixtures; no existing grammar, bound or fixture changes. The manifests Fingerprint paragraph gains the semantics it already deferred to ADR-017, with no new refusal |
| `tools/orchestration` (Node) | Not affected | No pipeline stage reads discovery output; `npm ci && npm run check` is unchanged (A22) |
| Rust toolchain and dependencies | Yes | No new external dependency and no `Cargo.lock` version change (A12); `crates/codecs` is in the existing `crates/*` glob |

## Test strategy

1. Unit: `ScanRecord` and `scan()` — each row's `library` is the one scanned and
   its `root` the root that named it, a library two roots share is scanned once
   and owned by the first, and `discover()` returns exactly the same rows in
   exactly the same `(slug, install_path)` order as before the refactor.
2. Unit: `GameContext::from_record` — `steam/900001` yields the appid, every
   resolved root and the owning library; a `local/x` key, a `steam/` empty id, a non-digit appid and a
   21-digit appid each yield `None`.
3. Unit: `user_trees` — both templates resolve from a fixture machine;
   `SteamUserdata` enumerates `ctx.roots` in `roots()` order and `<dir>` in byte
   order, and a root with no `<dir>/<appid>` under it contributes no tree; absent
   trees are not returned; a non-digit directory name and a digit-named symlink
   are skipped and counted; two fixture roots holding 9- and 8-digit directories
   keep the first 16 in that enumeration order, the 17th being past the cap.
4. Unit: budgets at the seams — `UserCandidates::push` refuses a 33rd file
   (`017-B3`'s type invariant, behind the walk's skip-and-count);
   `read::open_capped` refuses a `MAX_BINDING_FILE_OCTETS + 1` file as
   `FileOverBudget` without returning contents; the distinct-pair total breaching
   mid-fold names itself as `SnapshotEntryBudget`, one entry with
   `MAX_SNAPSHOT_ENTRIES + 1` inputs breaching it as surely as 4097 entries do;
   the loader refuses a `MAX_SNAPSHOT_OCTETS + 1` document on its length check.
5. Unit: `read::decode` — UTF-8 passes; a UTF-8 BOM is stripped; UTF-16LE and
   UTF-16BE BOMs decode; an unpaired surrogate and a BOM-less invalid-UTF-8 body
   are `UndecodableText`; the `bom-utf16` fixture extracts the same entries as
   its UTF-8 twin.
6. Unit: locate walk — the `Saved/Config/<dir>/` grammar is located at
   ≤ `MAX_LOCATE_DEPTH` tree-relative components and not beyond; a platform
   directory name other than those the fixtures use
   is still found (no constant assumed); out-of-grammar `.ini` files, non-`.ini`
   files, and `defaults-only` install trees contribute nothing; a symlinked directory, a
   symlinked `.ini` and a symlinked *tree root* in `symlinked-prefix` are neither
   descended, read nor anchored; a walk that reaches `MAX_LOCATE_ENTRIES_PER_TREE`
   stops and counts the rest, so every run ends in a status rather than a loop.
7. Unit: `unreal-ini` extraction — `+Bindings` pairs group by command verbatim,
   `Name` tokens stay verbatim (the `MouseX`/`Left`/`Gamepad_RightX` cases from
   A20 are in the fixture and are *not* refused), `-Bindings` removes an exact
   pair, `!Bindings=(…)` removes just the pairs it names, a bare `!Bindings=`
   clears the key for it, a bare `+`/`-`/`.` operand is skipped, and a `Command`
   whose pairs are all removed leaves no entry; a `.` add may duplicate a pair
   and collapses with the one it repeats, duplicate identical pairs collapse,
   one key bound to two commands keeps both entries; `broken-tuple`, and a
   verbatim `Command=` or `Name=` over `MAX_NATIVE_NAME_OCTETS`, are
   `UnparsableEntry` naming the relative path and line.
8. Unit: extraction honesty (`017-L3`) — no entry comes from any file outside
   the located candidate set, no entry has a taxonomy id in it, no entry has an
   input id in it, and no default file contributes: asserted over every fixture
   document, with the `defaults-only` tree yielding a claim-and-nothing-to-read
   or no claim, never a partial entry list.
9. Unit: registry — the declaration order is byte-sorted; a synthetic family
   owning appid `900100` wins the appid pass with `Evidence::AppidTable` even
   when another family could probe-claim it; with an unlisted appid the probe
   pass claims in id order; a probe that locates nothing does not claim;
   `family_ids`/`is_known_family` agree with the slice.
10. Unit: snapshot loader — every `schemas/bindings/v1/fixtures/valid/*.json`
    loads; every `fixtures/invalid/` one is refused naming its key (an unknown
    key, a missing key, a `codec` outside slug grammar, an over-bound `action`, a
    control character, a leading space, a pair total over `MAX_SNAPSHOT_ENTRIES`,
    a document over `MAX_SNAPSHOT_OCTETS`, a `user` over `MAX_NATIVE_NAME_OCTETS`,
    a `files` entry over `MAX_LOCATE_DEPTH` components, a `version` of `"1"` or `2`);
    messages are single-line, control-free and truncating.
11. Unit: `bindings/v1` documents produced by the family all pass the loader —
    the extraction and contract halves of the run cannot disagree.
12. Integration: determinism (`017-L4`) — extracting each fixture game twice,
    and with the registry and each `Detection.per_user` vector (its
    `tree_root`s with it) shuffled in the test harness, yields byte-identical
    documents, byte-identical tables and byte-identical `--json`; the ordering
    law is asserted on entries, inputs, users and provenance together; two
    records that resolve the same user tree yield one document, while one slug
    in two libraries — two `ProtonPrefix` trees sharing the `steamuser` label
    with different bindings — yield two documents and two `--probe` groups.
13. Integration (CLI, subprocess): slug resolution and rendering — one table and
    one JSON array for a fixture game, a `Codec` column populated from the
    snapshot, four statuses each producing one named stderr `status:` line and
    its own exit code (0/3/4/5) with `--json` printing the status document, and a
    `--probe` run printing families, evidence and absolute candidate paths and
    extracting nothing, and every status exit code reached by a real fixture
    route rather than a stub. Also pinned: no network leg exists on this face
    (the test runs with no `--catalog-url`, no `BINDFORGE_CATALOG_URL` and no
    listener, and still exits 0), and the fixture tree's file bytes are
    byte-identical after the run (`017-L1`).
14. Integration: `list` is untouched — `cli_output_polish_tests`,
    `cli_json_mode_tests`, `cli_catalog_gate_tests` and `cli_slug_refusal_tests`
    pass unmodified, and `list --json` still carries `codec_id: null` for every
    row of a tree the bindings face claims a codec for (`017-L5`).
15. Contract-tree pins — the conformance walk's fixture-name and zero-fixture
    rules cover `schemas/bindings/v1` while its TOML parse/rejection loops skip
    it via `JSON_FIXTURE_FORMATS`; `schemas/README.md` states the two JSON
    formats, the budget table with its aggregate product, and the
    `schemas/bindings/` hygiene carve-out; the boundary sweep skips exactly
    `data/catalog/`, `schemas/catalog/` and `schemas/bindings/`, and still fails
    on any `steam/` spelling under `data/`;
    `load_manifest` still refuses nothing new (a manifest naming a
    not-yet-implemented family stays loadable).
16. Unit: `zz_api_probe` for `bindforge-codecs` resolves the published surface
    (`CodecFamily`, `Detection`, `Evidence`, `ExtractionRefusal`, `GameContext`,
    `ScanRecord`, `user_trees`, `claim`, `family_ids`, the budget constants) in
    value or call position, and pins that the only `Cargo.lock` delta is the
    `bindforge-codecs` stanza and the `discovery-cli` dependency edge.
17. Platform: the suite runs on the Linux runner only (A22). The `#[cfg(unix)]`
    link fixtures are the one platform-scoped assertion; the Windows-visible half
    of the face is the compile-check leg's, where the `cfg(windows)`
    plugin-selection line and the link tests' compile-only arm are covered.
    Nothing performs a network request, installs a game, or reads a path outside
    its own fixture tree and `target/`.

## Rollout

One branch; the runner owns the commits. Items 1–7 are one commit each, follow
the ADR's Rollout order, and carry its unit handles; items 8 and 9 are the
ledger hand-off and the gate list.

1. Discovery plumbing: `ScanRecord` in `crates/discovery/src/game.rs`,
   `SteamCore::scan()` with `discover()` as its projection, the per-root
   library ownership with the shared dedupe kept, and the two plugin wrappers'
   `scan_records()` and `roots()`; the existing suites re-pin only where a helper
   builds a core. `017-C2`
2. The `codecs` crate lands: `Cargo.toml`, `lib.rs`, `codec.rs`, `context.rs`,
   `registry.rs` with the two claim passes, the budget constants, and
   `zz_api_probe`; the synthetic family pins the pass ordering without inventing
   appids. `017-C1 017-C5`
3. Resolvers and reader: `paths.rs`, `read.rs` with the BOM and cap posture, the
   walk that follows no link at a tree anchor or below one, and their suites
   against the fixture layouts. `017-C2 017-B1 017-B3`
4. The contract lands: `schemas/bindings/v1/spec.md` with its key set, bounds
   and ordering law; `fixtures/valid/` and `fixtures/invalid/`; the loader in
   `snapshot.rs`; the `schemas/README.md` two-JSON-formats and budget
   amendments; `JSON_FIXTURE_FORMATS` in the harness. `017-C3 017-B2`
5. `unreal-ini` lands with its fixture trees and the merge-operator fold, under
   the never-write discipline, with the read-only verification pass recorded in
   `03-build-notes.md` as an operator step. `017-S1 017-L1 017-L3`
6. The CLI face lands: the `Bindings` subcommand, `bindings.rs` with its four
   statuses and exit codes, the shared table renderer's generalization, probe
   mode, the no-catalog-leg decision, and `cli_bindings_tests` +
   `cli_bindings_status_tests`; `list` untouched. `017-C4 017-L2 017-L5`
7. The pins and the amendments close the run: the determinism and honesty
   suites, the boundary-sweep carve-out, the manifests Fingerprint-semantics
   paragraph (`013-R2`), the `03-build-notes.md` deferral record for
   `017-R1`–`017-R4`, and the ledger annotation request. `017-L4 017-R1 017-R2 017-R3 017-R4`
8. **Ledger.** `docs/adrs/` files are runner/operator-owned, so this run claims
   no edit to ADR-017: it hands the operator the units whose shipped reading
   differs from their wording (`017-B2`'s and `017-B3`'s units of count,
   `017-C1`'s bound and its `extract` shape, `017-C2`'s root set, `017-C3`'s
   `inputs` and document identity, `017-C4`'s "same scan", `017-S1`'s launcher
   contexts) for `amended-by` lines per `021-L4`, and repeats them in
   `03-build-notes.md`; nothing renumbers a unit.
9. Gates: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
   -- -D warnings`, `cargo test --workspace --locked`, `cargo deny check
   advisories`, and `npm ci && npm run check` in `tools/orchestration`, per
   `.github/workflows/check.yml`. The Windows compile-check leg must stay green
   with `crates/codecs` added; any change to an external package or to a resolved
   version is a finding.

## Deferred / out of scope

- The apply-and-authoring half of this number's reservation: writing bindings
  back with backup/restore, the authoring CLI, snapshot persistence, and
  computing the fingerprint this run only defined — `017-R1` (unassigned).
- Launcher contexts the record cannot scope: the host config home
  (`$XDG_CONFIG_HOME`/`$HOME/.config`, a Flatpak/Snap Steam's own config home,
  `%LOCALAPPDATA%` and `Documents\My Games`, the macOS Steam root). Naming a UE
  project directory has to come before that tree can be a candidate, so the
  routing is `017-R4`'s and the path resolution is `017-R2`'s.
- Further families: the Source `bind`/`alias` lineage, UE
  `ActionMappings`/`AxisMappings`, and Enhanced-Input saved-game settings —
  `017-R3` (owner: ADR-017 follow-on runs).
- Machine-readable codec routing: appid→family as `data/` content, and with it
  the load-time implementation-set check `013-R3` reserves — `017-R4`
  (unassigned). This run ships `family_ids`/`is_known_family` as its seam.
- Widening the ADR-012 input vocabulary (named keys, modifiers, analog axes) and
  any canonical-spelling form of a snapshot — ADR-012 owns that vocabulary major.
- Shadowing and order-dependence in the contract (which of two bindings on one
  key the engine will honour) — no follow-up yet; `017-R3`'s families will say
  whether they need it.
- Chords, axis scales and modifier state in a snapshot entry — no follow-up yet;
  `v1` reports the file's tokens verbatim and drops nothing.
- SteamID-to-player identity, any export or upload of a snapshot, and the
  DB-backed registry — untouched; ADR-027 and ADR-026 own those surfaces.
- Non-Steam discovery sources — ADR-022 and `025-R2`, unchanged.
- A BindForge process sandboxed so it cannot read another app's tree at all
  (today untested anywhere) — no follow-up yet.
- Any UI or caching surface over snapshots — the UI's import is `017-R1`'s
  consumer, downstream of this contract.
- ADR-008's planned names for this work: `crates/adapters` and `crates/scanner`
  are superseded by the shipped `crates/discovery` plus this run's
  `crates/codecs`, and ADR-008's "clause 3" for the crate-split precedent is
  clause 2's workspace list plus ADR-009's transforms clause (A14). `Cargo.toml`'s
  comments about `apps/cli` and `toml_edit` as ADR-017 arrivals stay as history;
  ADR files and comments are operator-owned, so no edit is claimed here.

## Revision log

- Initial: `bindings/v1` is JSON, not TOML — `--json` prints the document
  verbatim, and nested entries are inexpressible in the documented TOML subset.
- Initial: §`bindings/v1` — `inputs` are native tokens verbatim, not ADR-012 ids
  (`017-C3` read differently): the closed set cannot spell them (A5, A20).
- Initial: §The CLI face — the face runs the enumeration, not the catalog gate
  (`017-C4` read differently), so extraction works with no service.
- Initial: §Codec family id — bound read as `024-B1`'s `MAX_SLUG_OCTETS`; the
  ADR's `013-B4` citation is a slip (A6).
- Initial: §Discovery plumbing — additive `scan_records()` rather than a trait
  change, so `list`'s pinned shapes and `017-L5`'s `None` stay provable.
- Initial: §The registry — a family id names a storage grammar: an appid-table
  hit claims, a probe claims only on a located file.
- Initial: §The first family — `appids()` empty rather than invented appids
  (A21); the appid pass is pinned by a synthetic family.
- Initial: §`bindings/v1` — no engine-precedence fold and no shadowing model:
  entries are the (action, input) pairs the user files state.
- R1: §GameContext — `roots` is `017-C2`'s whole resolved set, read differently
  than its "main root" wording, in the core's canonicalized root byte order.
- R1: §Config-path resolvers — no host-config tree ships: `$HOME/.config` is
  shared by every Unreal title and no record names the project inside it, so
  `017-S1`'s native context is Deferred and every shipped template hangs off a
  resolved root (Linux and Flatpak/Snap rows).
- R1: §Config-path resolvers and §Read discipline — a link is never a `UserTree`
  anchor either, which is what lets candidate paths stay uncanonicalized; locate
  reads candidates through the capped reader; a match the reader refuses is
  located on path grammar alone and `extract` refuses it by name, never unseen.
- R1: §Read discipline — `017-B2` counts distinct (`action`, input-token) pairs
  *as admitted*, so removals free no room; `017-B3`'s `MAX_CANDIDATE_FILES` caps
  a *tree's* opens, `MAX_USER_DIRS_PER_GAME` the trees kept over all `ctx.roots`
  in `roots()`-then-`<dir>` order, `MAX_SNAPSHOT_OCTETS` the loaded document;
  nothing prunes but depth and entry count, and the aggregate counts every tree.
- R1: §Read discipline — an over-bound verbatim `Command=`/`Name=` is the
  existing `UnparsableEntry`, named once here, so no new refusal name enters.
- R1: §`bindings/v1` — `user` rides the token grammar, not a closed list of the
  labels `v1` emits; a `files` entry rides `MAX_LOCATE_DEPTH` tree-relative
  components, the unit §Read discipline's table defines.
- R1: §The first family — the fold keeps UE's four merge operators (`+`, `.`,
  `-`, `!`), not three; a bare `!` clears the key, `!Key=(…)` names pairs, a bare
  `+`/`-`/`.` is skipped, and a fully emptied `Command` leaves no entry.
- R1: §The codec face and §`bindings/v1` — one document per located user tree,
  not `017-C1`'s single one: `017-C3`'s per-local-SteamID wording reads as
  per-tree, `UserCandidates.tree_root` carries the identity §Determinism dedups
  by, and consumers union same-`user` documents' entries.
- R1: §The CLI face — the status codes sit above clap's own 2, which this face
  neither owns nor redefines; documents deduplicate by tree identity —
  the `UserCandidates.tree_root` alone, so §Determinism's order over documents
  is total.
- Initial: `013-R2` discharged by definition in the manifests spec (byte recipe +
  mismatch meaning); `013-R3` narrowed to the published family set, its
  load-time check riding `017-R4`.
- Initial: §Corpus boundary — snapshots are printed, never written, so the only
  source-key carve-out this run needs is `schemas/bindings/` in the hygiene sweep.

## As-built (2026-11-24) — code-vs-proposal drift

The run shipped every design decision above unchanged. The entries below record
where the committed code diverged from the *wording* of this proposal during
implementation and the two code-review fix rounds; none reverses a decision, and
each is cross-referenced to `03-build-notes.md`. They are recorded here so a
reader of the design is not misled by a section that the shipped code reads
differently than it states.

1. **`ExtractionRefusal` ships six variants, not the four §Read discipline names.**
   Drift: §Read discipline enumerates `FileOverBudget`, `UndecodableText`,
   `UnparsableEntry` and `SnapshotEntryBudget`; `crates/codecs/src/codec.rs`
   declares two more — `CandidateFilesBudget` (the `UserCandidates::push`
   `MAX_CANDIDATE_FILES` invariant §Test strategy #4 pins) and `UnreadableFile`
   (a located path the OS will not stat/open). Reason: both seams must name a
   refusal rather than invent an empty answer; `UnreadableFile` also carries the
   compat-A rule that a path the OS will not stat is skip-and-count and never
   becomes a candidate. Affected: §Read discipline, §The codec face. (See
   `03-build-notes.md` "Round-1 review fixes" compat-A.)

2. **`CodecFamily` carries a `Sync` supertrait.** Drift: the §The codec face
   trait snippet is `pub trait CodecFamily { … }`; the shipped trait is
   `pub trait CodecFamily: Sync`. Reason: §The registry holds `pub static
   REGISTRY: &[&dyn CodecFamily]`, and shared static `&dyn` data requires
   `Sync`; the snippet omitted the bound the static it names already implies.
   Affected: §The codec face, §The registry.

3. **The registry publishes two seams beyond §The registry's named set.**
   Drift: §crates/codecs layout and §The registry name `claim`, `family_ids`,
   `is_known_family`; `crates/codecs/src/registry.rs` also exports
   `claim_with(families, ctx)` (re-exported at the crate root) and `family(id)`.
   Reason: `claim_with` is the two-pass body `claim` instantiates over the
   shipped `REGISTRY`, and is the seam the §The first family synthetic family
   pins pass ordering through without inventing appids; `family(id)` is how a
   face holding a `Detection` runs the claiming family's own `extract`. Both
   iterate in family-id byte order whatever the slice's arrangement. Affected:
   §crates/codecs layout, §The registry.

4. **`read::decode_owned` added; `serde_json` ships as a runtime dependency.**
   Drift: §Read discipline names `read::open_capped` and `read::decode` only;
   `read.rs` adds `decode_owned(Vec<u8>)` (re-exported at the crate root) which
   the no-BOM path uses to move the buffer into the decoded `String` instead of
   copying it, and `decode(&[u8])` now delegates to it. Separately, §crates/codecs
   layout lists `serde_json` as a *dev-dependency*; `crates/codecs/Cargo.toml`
   makes it a load dependency. Reason: `decode_owned` is the perf-7 fix; the
   `bindings/v1` loader owns its own JSON grammar, so following the `catalog`
   posture makes `serde_json` a load edge, not a dev one. Affected: §Read
   discipline, §crates/codecs layout. The `Cargo.lock`-delta claim still holds:
   the lock gains exactly the `bindforge-codecs` stanza and the `discovery-cli`
   edge (11 insertions, 0 deletions, no version moves), because `serde_json` was
   already resolved (A12). (See `03-build-notes.md` Ledger and perf-7.)

5. **Worst-case read budget is two passes, not one.** Drift: §Read discipline's
   aggregate sentence gives `(MAX_USER_DIRS_PER_GAME + 1) × MAX_CANDIDATE_FILES
   × MAX_BINDING_FILE_OCTETS` (544 MiB); `schemas/README.md` and
   `schemas/bindings/v1/spec.md` state `2 ×` that product (1,088 MiB). Reason:
   the `unreal-ini` family reads each located file twice — once for `detect`'s
   content test, once for `extract`'s fold — so the docs must describe the run
   the code performs. Carrying decoded text on `UserCandidates` (the reviewer's
   other option) would change the §The codec face type shape; not done. Affected:
   §Read discipline. (See `03-build-notes.md` deviation 5 / perf-2.)

6. **`MAX_LOCATE_ENTRIES_PER_TREE` bounds examined entries, not the listing.**
   Drift: §Read discipline's table reads as a plain cap on walk entries;
   `lib.rs` and `unreal_ini::locate` document it as bounding *examined* entries,
   with each directory's `read_dir` listing materialized and byte-sorted before
   the budget trims what the loop examines (an over-budget walk enters no
   further directory). Reason: determinism of the examined set needs the sorted
   listing; a pre-sort `take` would make the set readdir-order-dependent (the
   reviewer's explicitly offered alternative). Affected: §Read discipline. (See
   `03-build-notes.md` deviation 7 / perf-4.)

7. **The NTFS/exFAT symlinked-prefix degradation answers `claim-refused` (3),
   not `family-confirmed-files-absent` (4).** Drift: the Compatibility-matrix row
   "Steam library on NTFS/exFAT with symlinked prefixes" names that degradation
   as exit 4; `paths.rs::proton_trees` and `bindings.rs` make a broken or
   symlinked prefix locate no grammar, claim nothing, and answer exit 3. Reason:
   with `appids()` empty no shipped route produces exit 4 for it —
   `family-confirmed-files-absent` belongs to appid-table claims, which
   `017-R4`'s routing data would be the first to create. Affected: Compatibility
   matrix, §The registry. Ledger hand-off (Rollout 8): that row's named status is
   exit 3 until `017-R4` lands. (See `03-build-notes.md` deviation 6 / compat-C.)

8. **The `bindings/v1` loader refuses escaping `provenance.files` spellings.**
   Drift: §`bindings/v1` Bounds describes `files` by count
   (`MAX_CANDIDATE_FILES`) and component depth (`MAX_LOCATE_DEPTH`); the loader
   additionally refuses any `files` entry that is not a plain `/`-separated
   sequence of `Normal` path components — `ParentDir`/`CurDir`/`RootDir`/`Prefix`
   and raw `\`/`:` spellings (Windows drive, UNC, backslash-`..`, `./`) — naming
   `files`. Reason: the security-1 fix — an absolute or traversal spelling must
   not reach a consumer's path resolver dressed as relative. Affected:
   §`bindings/v1` Bounds. Stated in `schemas/bindings/v1/spec.md` Bounds and
   pinned by `schemas/bindings/v1/fixtures/invalid/files-traversal-files.json`.
   (See `03-build-notes.md` "Round-1 review fixes" security-1.)
