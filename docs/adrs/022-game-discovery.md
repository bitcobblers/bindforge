---
status: accepted
date: 2026-11-15
depends-on: [008]
---

# ADR 022: Game discovery via pluggable probes

We need a reliable way to enumerate installed games on a host and expose a stable BindForge identifier for each instance. Discovery must be platform-aware and extensible, because different stores and manual installs surface games differently. The first consumer is a bootstrap CLI that probes the system before the Tauri UI exists.

## Context

The product needs three pieces to operate on a game:
* a BindForge-known game id,
* an installation path,
* a codec that can read/write that game's native binding storage.

Currently `crates/core` and `crates/transforms` define the profile/manifest/policy formats, but there is no mechanism to find games on a host. The UI design calls for a list of discovered games with a profile drop-down and an apply button. Discovery must precede codec selection and UI rendering.

We want a plugin model so Steam, Epic, and manual/custom discovery can be added independently. For v1 we only need Steam on Linux and Windows, no Steam Deck.

Slug format is `publisher/name` lower-kebab. For v1 we cannot reliably derive publisher from Steam metadata without third-party IDs, so discovery will emit a provisional slug `unknown/<kebab(display_name)>` and leave codec blank. Matching to a real manifest slug can be added later.

## Decision

Define a `crates/discovery` library with a plugin trait and a bootstrap CLI app.

**Data model**
`DiscoveredGame`:
* `slug: String` – provisional `unknown/<kebab(name)>` for v1
* `display_name: String`
* `install_path: PathBuf`
* `platform: Platform` – e.g. `steam-linux`, `steam-windows`
* `codec_id: Option<String>` – blank for v1

**Plugin contract**
```
DiscoveryPlugin
  name() -> &'static str
  discover() -> Result<Vec<DiscoveredGame>, Error>
```
Plugins are loaded by the CLI and run independently. The contract is platform-agnostic; implementations handle OS specifics.

**Steam discovery v1**
* Linux backend: read library folders from `libraryfolders.vdf` and parse `appmanifest_*.acf` under each library's `steamapps` directory.
* Windows backend: read Steam install via registry, enumerate library folders, parse `appmanifest_*.acf`.
* For each manifest, extract `name` and `installdir`, build `DiscoveredGame` with slug `unknown/<kebab(name)>`, platform tag, and `codec_id = None`.
* No AppID usage, no third-party sync. No persistence.

**CLI bootstrap**
`apps/discovery-cli` binary with `discovery list` command that runs enabled plugins and prints a table with slug, display_name, install_path, platform. Output is human readable and JSON optional.

**Slug derivation**
Slug is derived from discovered display name via simple normalization: lower-case, replace non-alphanumeric with `-`, collapse repeats. Provisional namespace is `unknown/`. Real `publisher/name` slugs can be introduced later via a resolver step.

## Alternatives considered

**Use Steam AppID as the game id.** Rejected: we do not want to be in the business of syncing with third-party library IDs. BindForge ids should be stable and under our control.

**Discover via a central database / remote service.** Rejected for v1: requires network and adds coupling. Local probing is sufficient for bootstrap.

**Hard-code a single Steam implementation with no plugin trait.** Rejected: Epic and manual discovery are planned, and platform differences are best isolated behind a trait.

**Derive slug from install path.** Rejected: paths are unstable and not human readable.

## Consequences

Positive:
* Gives a concrete inventory of installed games with a stable provisional id.
* Plugin trait isolates platform-specific probing and enables future Epic/manual plugins.
* CLI provides immediate feedback without waiting for the Tauri UI.
* Slug derivation is decoupled from third-party IDs.

Negative / costs:
* v1 slugs are provisional and will need migration when real manifests are matched.
* No persistence yet; repeated probes re-scan the filesystem.
* No codec selection yet; that requires a common codec taxonomy first.

Not decided here:
* Manifest matching and slug migration to real `publisher/name` ids.
* Codec taxonomy and per-game codec selection.
* Persistence to SQLite/Postgres and remote discovery sync.
* UI authoring of profiles.

