# Research: Game Discovery via Pluggable Probes

## Context and Problem Statement

BindForge requires a reliable way to identify installed games on a host to allow users to apply profiles. The discovery process must:
1.  Enumerate installed games across different platforms and stores (Steam, Epic, etc.).
2.  Provide a stable, BindForge-internal identifier (a slug) for each discovered instance.
3.  Work as a bootstrap mechanism (e.g., a CLI) before the full Tauri UI is available.

Current gap: While `crates/core` and `crates/transforms` define the profile/manifest/policy formats, there is no mechanism to find games on a host.

## Proposed Architecture

### Plugin Model
Discovery will be implemented as a pluggable system to ensure extensibility (e.g., adding Epic or manual discovery later).
- **Library**: A new `crates/discovery` library.
- **Trait**: A `DiscoveryPlugin` trait defining:
  - `name() -> &'static str`
  - `discover() -> Result<Vec<DiscoveredGame>, Error>`
- **Execution**: Plugins will be loaded and run by a bootstrap CLI.

### Data Model: `DiscoveredGame`
Each discovered game will be represented by:
- `slug: String`: A provisional identifier. For v1, this follows the `unknown/<kebab(display_name)>` format.
- `display_name: String`: The human-readable name.
- `install_path: PathBuf`: The filesystem location of the game.
- `platform: Platform`: A tag identifying the discovery source/platform (e.g., `steam-linux`, `steam-windows`).
- `codec_id: Option<String>`: The ID of the codec required to read the game's bindings. For v1, this is `None`.

### CLI Bootstrap
An `apps/discovery-cli` binary will provide a `discovery list` command. This command will execute all enabled plugins and output a table (human-readable or JSON) containing the game details.

## v1 Implementation Plan: Steam Discovery

The initial version focuses on Steam on Linux and Windows.

### Linux Backend
- Locate Steam library folders via `libraryfolders.vdf`.
- Parse `appmanifest_*.acf` files within the `steamapps` directory of each library.
- Extract `name` and `installdir` to build the `DiscoveredGame` record.

### Windows Backend
- Use the Windows Registry to locate the Steam installation.
- Enumerate library folders.
- Parse `appmanifest_*.acf` files similarly to the Linux implementation.

## Slug Derivation and Identity

To maintain stability, BindForge will use its own slugs rather than third-party IDs (like Steam AppIDs).
- **Normalization**: Lower-case, replace non-alphanumeric characters with `-`, and collapse consecutive hyphens.
- **Namespace**: v1 uses the `unknown/` namespace.
- **Evolution**: A future resolver step will be introduced to match these provisional slugs to real `publisher/name` slugs defined in `data/manifests/`.

## Relationships to Existing Design

- **ADR-008 (Rust Product)**: The discovery library and CLI will be implemented in Rust.
- **ADR-013 (Game Manifests)**: Discovery provides the link between the filesystem and the `data/manifests/<slug>/manifest.toml` files. The slug is the primary join key.
- **ADR-017 (Codecs)**: The `codec_id` in `DiscoveredGame` will eventually be populated based on the codec taxonomy defined in ADR-017.

## Open Questions and Future Work

- **Slug Migration**: How to seamlessly transition from `unknown/` slugs to real `publisher/name` slugs once manifests are available.
- **Codec Integration**: Defining the taxonomy and mapping games to codecs during discovery.
- **Persistence**: Whether discovered games should be cached in a local database (e.g., SQLite) to avoid repeated filesystem scans.
- **UI Integration**: Designing how the Tauri UI consumes the discovery output and allows users to select games.
