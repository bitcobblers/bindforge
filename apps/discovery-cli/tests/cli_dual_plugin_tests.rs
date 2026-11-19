//! Integration tests for `discovery list` with both Steam plugins registered
//! (T-002 Chosen design §8, re-pinned by design 025's Test strategy #14: the
//! aggregate carries `steam/<appid>` lookup keys and friendly names, and no
//! namespace rides on any row).
//!
//! The subprocess test points the host probe at a nonexistent home so it does
//! not depend on the build host having Steam installed, and — per the Rollout 3
//! migration — behind the loopback catalog fake, since `list` refuses anything
//! before the gate passes. The engine test composes the two plugins in-process,
//! where no gate exists. The three pre-existing tests in `cli_tests.rs` are the
//! Linux-regression gate and are not duplicated here.

mod common;

use common::FakeCatalog;
use discovery::engine::DiscoveryEngine;
use discovery::game::Platform;
use discovery::steam_linux::SteamLinuxPlugin;
use discovery::steam_windows::SteamWindowsPlugin;
use std::path::{Path, PathBuf};

/// The crate's checked-in fixture tree, shared with `crates/discovery` tests.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/discovery/tests/fixtures/steam/minimal")
}

/// proves: Test strategy #10 (Chosen design §8, §3) with the ADR-025 gate — the
/// host-probe branch registers both Steam plugins: on a host with no Steam
/// install in the candidate list, and with the satisfied gate pointing at the
/// loopback fake, the command still exits 0, prints `[]`, and writes nothing to
/// stderr — an empty Windows root set is `Ok(vec![])`, not an error the engine
/// would print, and the absent store sandbox is the empty store. (The
/// registration order itself is the §8 shape, type-checked by Test strategy #12
/// and observed on a real host by #13.)
#[test]
fn test_cli_list_host_probe_is_quiet_and_empty_without_steam() {
    let fake = FakeCatalog::start("dual-fake-empty", |_| common::ok("[]"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_discovery-cli"))
        .args(["list", "--json", "--catalog-url", fake.url().as_str()])
        .env_clear()
        .env("HOME", "/bindforge-no-such-home")
        .output()
        .expect("failed to execute discovery-cli");
    assert!(
        output.status.success(),
        "discovery-cli must exit successfully, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "[]",
        "a host with no Steam install must yield the empty JSON array"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "the empty Windows root set must print nothing on stderr, got: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// proves: Test strategy #10 (Chosen design §8) re-pinned to the ADR-025 row —
/// with the populated Linux plugin registered first and
/// `SteamWindowsPlugin::from_roots([])` beside it, the engine's aggregate is only
/// the Linux rows: exactly the four fixture games, all tagged `SteamLinux`, each
/// carrying its `steam/<appid>` lookup key and its own friendly name; the empty
/// Windows plugin contributes nothing and turns the run into no error.
#[test]
fn test_empty_windows_plugin_adds_nothing_beside_a_populated_linux_plugin() {
    let mut engine = DiscoveryEngine::new();
    engine.register_plugin(Box::new(SteamLinuxPlugin::from_roots([fixture_root()])));
    engine.register_plugin(Box::new(SteamWindowsPlugin::from_roots(Vec::new())));
    let games = engine
        .discover_all()
        .expect("the engine must aggregate even with an empty Windows root set");

    assert_eq!(
        games.len(),
        4,
        "only the four Linux fixture rows may appear"
    );
    for game in &games {
        assert_eq!(
            game.platform,
            Platform::SteamLinux,
            "the empty Windows plugin must contribute no row: {game:?}"
        );
    }
    let mut rows: Vec<(&str, &str)> = games
        .iter()
        .map(|game| (game.slug.as_str(), game.friendly_name.as_str()))
        .collect();
    rows.sort_unstable();
    assert_eq!(
        rows,
        [
            ("steam/900001", "Example Game"),
            ("steam/900002", "Nameless Install"),
            ("steam/900003", "Mixed Case Game"),
            ("steam/900100", "Secondary Title"),
        ],
        "the aggregate must be exactly the fixture's four Linux rows, each with \
         the `steam/<appid>` lookup key and the friendly name (including the \
         install-dir fallback for the nameless manifest)"
    );
}
