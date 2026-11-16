# Design: Game discovery via pluggable probes

## Context
ADR-022 (docs/adrs/022-game-discovery.md) identifies the need for a reliable way to enumerate installed games and expose a stable BindForge identifier for each. Discovery must be platform-aware and extensible to support different stores (Steam, Epic) and manual installs. The first consumer is a bootstrap CLI that probes the system before the Tauri UI is available.

## Goals / Non-goals

- Goals:
  - A `crates/discovery` library with a plugin trait for extensibility.
  - A bootstrap CLI (`apps/discovery-cli`) for immediate feedback.
  - Steam discovery for Linux and Windows in v1.
  - A provisional slugging mechanism (`unknown/<kebab(display_name)>`) to decouple discovery from third-party IDs.
  - A platform-agnostic plugin contract.
- Non-goals:
  - Epic Games Store or manual discovery (deferred).
  - Steam Deck support (deferred).
  - Persistence of discovered games (deferred).
  - Matching provisional slugs to real `publisher/name` manifests (deferred to a resolver step).
  - Codec selection during discovery (deferred).

## Assumption register

| # | Assumption | Status | Basis |
|---|---|---|---|
| A1 | Steam on Linux uses `libraryfolders.vdf` and `appmanifest_*.acf` | verified | standard Steam behavior |
| A2 | Steam on Windows uses Registry for installation location | verified | standard Steam behavior |
| A3 | The bootstrap CLI must run without the Tauri UI | verified | ADR-022 requirement |
| A4 | Slugs should be stable and internal to BindForge | verified | ADR-022 decision |

## Options considered

**Use Steam AppID as the game id.** Rejected: BindForge IDs should be stable and under our control. Syncing with third-party library IDs introduces unnecessary coupling and dependency on their ID stability.

**Discover via a central database / remote service.** Rejected for v1: adds network dependency and complexity. Local probing is sufficient for the bootstrap requirement.

**Hard-code a single Steam implementation without a plugin trait.** Rejected: The plugin model is essential for supporting Epic, manual installs, and platform differences in a clean, extensible way.

**Derive slug from install path.** Rejected: Paths are unstable, vary by user/OS, and are not human-readable.

## Chosen design

The design implements a pluggable discovery system in Rust, providing a library for core logic and a CLI for bootstrapping.

### 1. Components

- **`crates/discovery` Library**: The core engine.
  - Defines the `DiscoveryPlugin` trait.
  - Manages plugin execution and result aggregation.
  - Implements the slug normalization logic.
- **`apps/discovery-cli` Binary**: A lightweight CLI.
  - Provides a `discovery list` command.
  - Outputs a human-readable table or JSON for programmatic use.
- **Steam Discovery Plugins**:
  - **Linux implementation**: Parses `libraryfolders.vdf` to find library paths, then scans `steamapps/appmanifest_*.acf` for game metadata.
  - **Windows implementation**: Uses the Windows Registry to find the Steam installation, enumerates library folders, and parses `appmanifest_*.acf`.

### 2. Data Model

The primary unit of discovery is the `DiscoveredGame` struct:

```rust
pub struct DiscoveredGame {
    /// Provisional identifier: `unknown/<kebab(display_name)>` for v1.
    pub slug: String,
    /// Human-readable name of the game.
    pub display_name: String,
    /// Filesystem path to the game installation.
    pub install_path: PathBuf,
    /// Platform tag (e.g., `steam-linux`, `steam-windows`).
    pub platform: Platform,
    /// The ID of the codec required (None for v1).
    pub codec_id: Option<String>,
}
```

### 3. Plugin Contract

Plugins are decoupled from the core engine via the `DiscoveryPlugin` trait:

```rust
pub trait DiscoveryPlugin {
    /// Returns the unique name of the plugin.
    fn name(&self) -> &'static str;
    /// Executes the discovery process.
    fn discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError>;
}
```

### 4. Slug Derivation

To maintain BindForge's independence, slugs are derived from the `display_name` using a strict normalization process:
1. Convert to lower-case.
2. Replace all non-alphanumeric characters with `-`.
3. Collapse consecutive hyphens (e.g., `--` becomes `-`).
4. Prefix with the provisional namespace `unknown/`.

*Example: "Super Mario Odyssey!" → `unknown/super-mario-odyssey`*

### 5. CLI Specification

The CLI will support the following command structure:

- `discovery list`: Runs all enabled plugins and prints a table of discovered games.
- `discovery list --json`: Same as above, but outputs a JSON array of `DiscoveredGame` objects.

**Example Output (`list`):**

| Slug | Display Name | Install Path | Platform |
|---|---|---|---|
| `unknown/elden-ring` | Elden Ring | `/home/user/.steam/steam/steamapps/common/ELDEN RING` | `steam-linux` |
| `unknown/counter-strike-2` | Counter-Strike 2 | `C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike 2` | `steam-windows` |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Steam metadata changes (e.g., `.acf` format) | Low | Medium | The plugin model isolates Steam-specific parsing; updates only affect the Steam plugin. |
| Slug collisions in the `unknown/` namespace | Medium | Low | Provisional slugs are intended for v1; a resolver step will map them to real `publisher/name` slugs in the future. |
| Performance impact of large library scans | Low | Low | Plugins are run independently; scans are local and filesystem-bound. |

## Compatibility matrix

| Platform / Environment | Supported? | Notes |
|---|---|---|
| Linux | Yes | Supported via Steam/Linux plugin. |
| Windows | Yes | Supported via Steam/Windows plugin. |
| macOS | No | No plugin implemented for macOS. |
| Steam Deck | No | Out of scope for v1. |
| Epic Games Store | No | Deferred to future plugins. |

## Test strategy

1. **Unit Tests (`crates/discovery`)**:
   - Test slug normalization with various edge cases (special chars, casing, repeats).
   - Test `DiscoveredGame` construction.
   - Test error handling in the plugin registry.
2. **Plugin Integration Tests**:
   - Mock filesystem/registry to test Steam Linux/Windows parsing logic without requiring actual Steam installs.
   - Verify that `.acf` parsing correctly extracts `name` and `installdir`.
3. **CLI Tests**:
   - Verify `discovery list` output format (table and JSON).
   - Ensure the CLI correctly aggregates results from multiple mock plugins.

## Rollout

1. **Phase 1: Core Library**: Implement `crates/discovery` with the trait, slugging logic, and the base `DiscoveredGame` model.
2. **Phase 2: Steam Linux Plugin**: Implement the Linux-specific backend using `libraryfolders.vdf` and `.acf` parsing.
3. **Phase 3: Bootstrap CLI**: Implement `apps/discovery-cli` to provide immediate visibility.
4. **Phase 4: Steam Windows Plugin**: Implement the Windows-specific backend using registry and `.acf` parsing.
5. **Phase 5: Verification**: Run the full suite of integration and CLI tests.

## Deferred / out of scope

- Epic Games Store discovery.
- Manual/custom directory discovery.
- Steam Deck (SteamOS) specifics.
- Persistence/caching of discovered games.
- Mapping `unknown/` slugs to real `publisher/name` slugs.
- Codec taxonomy and per-game codec selection.
