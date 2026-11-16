use discovery::game::{DiscoveredGame, Platform};
use std::path::PathBuf;

#[test]
fn test_discovered_game_construction() {
    let game = DiscoveredGame {
        slug: "unknown/test-game".to_string(),
        display_name: "Test Game".to_string(),
        install_path: PathBuf::from("/path/to/game"),
        platform: Platform::SteamLinux,
        codec_id: None,
    };

    assert_eq!(game.slug, "unknown/test-game");
    assert_eq!(game.display_name, "Test Game");
    assert_eq!(game.install_path, PathBuf::from("/path/to/game"));
    assert_eq!(game.platform, Platform::SteamLinux);
    assert_eq!(game.codec_id, None);
}
