//! The plugin contract as a third party sees it (`crates/discovery/src/plugin.rs`,
//! `registry.rs`): a plugin is named, and it answers with rows.
//!
//! proves: design 024's Test strategy #8 (the re-pin of the mock plugin's row) —
//! a hand-written plugin outside the shipped pair emits the ADR-024 row shape: a
//! prefix-free slug in the shared grammar, beside its own `namespace` field. The
//! slug is what the CLI renders and what a manifest `id` must be able to become,
//! so the contract's example row carries it that way.

use discovery::game::{DiscoveredGame, Namespace, Platform};
use discovery::plugin::DiscoveryPlugin;
use discovery::registry::DiscoveryError;
use std::path::PathBuf;

struct MockPlugin;

impl DiscoveryPlugin for MockPlugin {
    fn name(&self) -> &'static str {
        "mock-plugin"
    }

    fn discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
        Ok(vec![DiscoveredGame {
            slug: "mock-game".to_string(),
            namespace: Namespace::Unknown,
            display_name: "Mock Game".to_string(),
            install_path: PathBuf::from("/mock/path"),
            platform: Platform::SteamLinux,
            codec_id: None,
        }])
    }
}

// proves: design 024's Test strategy #8 (Chosen design: Identity shape) — the row a
// plugin returns carries the namespace separately from the slug, so a plugin that
// knows a publisher can say so without touching the slug's grammar.
#[test]
fn test_mock_plugin_discovery() {
    let plugin = MockPlugin;
    assert_eq!(plugin.name(), "mock-plugin");
    let games = plugin.discover().unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].slug, "mock-game");
    assert_eq!(games[0].namespace, Namespace::Unknown);
}

// proves: design 024's Test strategy #2, #8 (Goals: one grammar for every slug) —
// the row a third-party plugin hands over is checked with the same predicate the
// shipped plugins and core's parsers use, which is the shape the CLI's emit-boundary
// check expects to find.
#[test]
fn a_plugin_rows_slug_is_a_slug_name_and_carries_no_namespace() {
    let games = MockPlugin.discover().expect("the mock plugin answers");
    let slug = games[0].slug.as_str();

    assert!(
        bindforge_core::is_slug_name(slug),
        "`{slug}` — the slug a plugin emits must satisfy the shared grammar"
    );
    assert!(
        slug.len() <= bindforge_core::MAX_SLUG_OCTETS,
        "`{slug}` is {} octets, past the shared bound",
        slug.len()
    );
    assert!(
        !slug.contains('/'),
        "`{slug}` must not carry the `unknown/` prefix the `namespace` field replaced"
    );
    assert_eq!(games[0].namespace.to_string(), "unknown");
}
