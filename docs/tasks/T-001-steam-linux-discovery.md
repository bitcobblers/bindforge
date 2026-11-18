---
doc_type: task
status: pending
date: 2026-11-17
type: feature
functional_area: backend
associated_adr: ADR-022
depends_on: []
---

# T-001: Implement Linux Steam discovery plugin

Currently, the `discovery` crate provides the engine, plugin trait, and data models, but lacks any actual discovery implementations. The `discovery-cli` is limited to a `DummyPlugin` that returns an empty list.

This task involves implementing a `SteamLinuxPlugin` within the `discovery` crate. The plugin will:
- Identify Steam library folders by parsing `libraryfolders.vdf`.
- Locate and parse `appmanifest_*.acf` files in the `steamapps` directory of each identified library folder.
- Extract the game `name` and `installdir` from the manifests.
- Construct `DiscoveredGame` objects using the `normalize_slug` utility, setting `platform` to `SteamLinux` and `codec_id` to `None`.

Done when:
- A `SteamLinuxPlugin` is implemented in `crates/discovery`.
- The `discovery-cli` can successfully run the `SteamLinuxPlugin`.
- Running `discovery list` on a Linux machine with Steam correctly identifies installed games.
- Unit tests in `crates/discovery` cover manifest parsing and library folder detection.
