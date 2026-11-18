//! Integration tests for `discovery list` with both Steam plugins registered
//! (T-002 Chosen design §8, Test strategy #10).
//!
//! The subprocess test points the host probe at a nonexistent home so it does
//! not depend on the build host having Steam installed; the engine test
//! composes the two plugins in-process. The three pre-existing tests in
//! `cli_tests.rs` are the Linux-regression gate (Test strategy #9) and are not
//! duplicated here; ADR-024 re-pinned their rendering there, and the rows below
//! carry the same prefix-free slugs.

use discovery::engine::DiscoveryEngine;
use discovery::game::{Namespace, Platform};
use discovery::steam_linux::SteamLinuxPlugin;
use discovery::steam_windows::SteamWindowsPlugin;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The crate's checked-in fixture tree, shared with `crates/discovery` tests.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/discovery/tests/fixtures/steam/minimal")
}

// proves: Test strategy #10 (Chosen design §8, §3) — the host-probe branch
// registers both `SteamLinuxPlugin::from_host()` and
// `SteamWindowsPlugin::from_host()`: on a host with no Steam install in the
// candidate list the command still exits 0, prints `[]`, and writes nothing to
// stderr — an empty Windows root set is `Ok(vec![])`, not an error the engine
// would print. (The registration order itself is the §8 shape, type-checked by
// Test strategy #12 and observed on a real host by #13.)
#[test]
fn test_cli_list_host_probe_is_quiet_and_empty_without_steam() {
    let output = Command::new(env!("CARGO_BIN_EXE_discovery-cli"))
        .args(["list", "--json"])
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

// proves: Test strategy #10 (Chosen design §8) — with the populated Linux
// plugin registered first and `SteamWindowsPlugin::from_roots([])` beside it,
// the engine's aggregate is only the Linux rows: exactly the four fixture
// games, all tagged `SteamLinux`; the empty Windows plugin contributes nothing
// and turns the run into no error.
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
    let mut slugs: Vec<&str> = games.iter().map(|g| g.slug.as_str()).collect();
    slugs.sort_unstable();
    assert_eq!(
        slugs,
        [
            "example-game",
            "mixed-case-game",
            "nameless-install",
            "secondary-title",
        ],
        "the aggregate must be exactly the fixture's four Linux rows, with ADR-024's \
         prefix-free slugs"
    );
    assert!(
        games.iter().all(|g| g.namespace == Namespace::Unknown),
        "the engine's aggregate carries the v1 namespace on every row, kept out of the slug"
    );
}
