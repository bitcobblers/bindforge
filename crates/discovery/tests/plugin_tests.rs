//! The plugin contract as a third party sees it (`crates/discovery/src/plugin.rs`,
//! `registry.rs`): a plugin is named, and it answers with rows.
//!
//! proves: design 025's Test strategy #1 + #14 (the re-pin of the mock plugin's row;
//! Chosen design: Row identity) — a hand-written plugin outside the shipped pair
//! emits the ADR-025 row shape: the composed `<source>/<external-id>` lookup key in
//! `slug`, the normalized `friendly_name` beside it, and no namespace field. The
//! row's identity is the source key: it rides `025-B1`'s length bound and is exempt
//! from the product slug grammar by design (`025-L3`), which is exactly what this
//! contract row pins.

use bindforge_core::{MAX_DISCOVERY_SLUG_OCTETS, is_slug_name};
use discovery::game::{DiscoveredGame, Platform, Source, friendly_name};
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
            slug: Source::Steam.lookup_key("4242"),
            friendly_name: friendly_name("Mock Game"),
            display_name: "Mock Game".to_string(),
            install_path: PathBuf::from("/mock/path"),
            platform: Platform::SteamLinux,
            codec_id: None,
        }])
    }
}

// proves: design 025's Test strategy #1 (Chosen design: Row identity) — the row a
// plugin returns carries the composed lookup key built through the one constructor,
// and the friendly name as its own field.
#[test]
fn test_mock_plugin_discovery() {
    let plugin = MockPlugin;
    assert_eq!(plugin.name(), "mock-plugin");
    let games = plugin.discover().unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].slug, "steam/4242");
    assert_eq!(games[0].friendly_name, "Mock Game");
}

// proves: design 025's Test strategy #4 + #14 (Chosen design: Emission — the identity
// gate becomes bound-only) — the row a plugin hands over is checked against
// `025-B1`'s length bound at the seams, and the shared grammar is explicitly NOT part
// of the output contract anymore: a composed source key fails `is_slug_name` by
// design, survives the bound-only checks, and can never be adopted as a product id
// (`025-L5`) — the inversion the retired `024-L1` grammar-before-emit check enforced.
#[test]
fn a_plugin_rows_slug_is_the_lookup_key_inside_the_discovery_bound() {
    let games = MockPlugin.discover().expect("the mock plugin answers");
    let slug = games[0].slug.as_str();

    assert_eq!(
        slug, "steam/4242",
        "the slug is the `<source>/<external-id>` lookup key"
    );
    assert!(
        slug.len() <= MAX_DISCOVERY_SLUG_OCTETS,
        "`{slug}` is {} octets, past `025-B1`'s lookup-key bound of \
         {MAX_DISCOVERY_SLUG_OCTETS} — the only thing the emit boundary checks",
        slug.len()
    );
    assert!(
        !is_slug_name(slug),
        "`{slug}` fails the shared product-slug grammar because the key carries its \
         source token and a `/` — by design, so it can never join a manifest, profile, \
         or path (`025-L3`, `025-L5`)"
    );
    assert!(
        !slug.contains('\\') && !slug.contains(':') && slug.matches('/').count() == 1,
        "`{slug}` is exactly one `/`-joined source/external-id pair: it carries no \
         path-traversal or list separator of its own"
    );
}
