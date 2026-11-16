use discovery::game::{DiscoveredGame, Platform};
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
            slug: "unknown/mock-game".to_string(),
            display_name: "Mock Game".to_string(),
            install_path: PathBuf::from("/mock/path"),
            platform: Platform::SteamLinux,
            codec_id: None,
        }])
    }
}

#[test]
fn test_mock_plugin_discovery() {
    let plugin = MockPlugin;
    assert_eq!(plugin.name(), "mock-plugin");
    let games = plugin.discover().unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].slug, "unknown/mock-game");
}
