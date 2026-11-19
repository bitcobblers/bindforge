//! Unit and fixture-integration tests for the Linux Steam discovery plugin
//! (`crates/discovery/src/steam_linux.rs`).
//!
//! Every test is hermetic: the plugin is driven through `from_roots` at the
//! read-only fixture trees under `tests/fixtures/steam/`, so no test probes the
//! host, writes to the filesystem, or needs Steam installed (Chosen design §9).

use discovery::game::{DiscoveredGame, Platform};
use discovery::plugin::DiscoveryPlugin;
use discovery::registry::DiscoveryError;
use discovery::steam_linux::{
    SteamLinuxPlugin, host_candidates, libraries_in_root, parse_manifest,
};
use std::path::{Path, PathBuf};

/// A checked-in fixture root under `tests/fixtures/steam/`.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/steam")
        .join(name)
}

/// The plugin's canonical path rule emits canonical paths; tests compare against
/// the same canonicalization.
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).expect("fixture path must exist")
}

/// Games discovered from the `minimal` fixture root.
fn minimal_games() -> Vec<DiscoveredGame> {
    SteamLinuxPlugin::from_roots([fixture("minimal")])
        .discover()
        .expect("the minimal fixture root must discover without error")
}

/// KeyValues text for an `AppState` section carrying the given `name` / `installdir` pair.
fn manifest_text(name: Option<&str>, installdir: Option<&str>) -> String {
    let mut text = String::from("\"AppState\"\n{\n\t\"appid\"\t\t\"900001\"\n");
    if let Some(n) = name {
        text.push_str(&format!("\t\"name\"\t\t\"{n}\"\n"));
    }
    if let Some(i) = installdir {
        text.push_str(&format!("\t\"installdir\"\t\t\"{i}\"\n"));
    }
    text.push_str("}\n");
    text
}

fn no_env(_key: &str) -> Option<String> {
    None
}

fn env_with<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |key| {
        let hit = vars.iter().find(|(k, _)| *k == key);
        hit.map(|(_, v)| (*v).to_string())
    }
}

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

// ---------------------------------------------------------------------------
// parse_manifest (Chosen design §5)
// ---------------------------------------------------------------------------

// proves: Test strategy #2 (Chosen design §5) — name + installdir yields both fields.
// proves: design 025's Test strategy #3 (Chosen design: Emission — `AppManifest`
// gains `appid`) — the reduction step now reads the ACF's `appid` beside them.
#[test]
fn parse_manifest_extracts_name_and_installdir() {
    let m = parse_manifest(&manifest_text(Some("Example Game"), Some("Example Game")))
        .expect("a manifest with name and installdir must parse");
    assert_eq!(m.display_name, "Example Game");
    assert_eq!(m.install_dir, "Example Game");
    assert_eq!(m.appid, "900001", "ADR-025 reads the appid the ACF names");
}

// proves: design 025's Test strategy #3 (Chosen design: Emission — the appid read
// "already a case-insensitive lookup") — the key spellings `AppId` and `APPID`
// resolve to the same value the lowercase spelling does.
#[test]
fn parse_manifest_reads_the_appid_key_case_insensitively() {
    for key in ["appid", "AppId", "APPID"] {
        let text = format!(
            "\"AppState\"\n{{\n\t\"{key}\"\t\t\"900001\"\n\t\"name\"\t\t\"Example Game\"\n\t\
             \"installdir\"\t\t\"Example Game\"\n}}\n"
        );
        assert!(
            parse_manifest(&text).is_some_and(|m| m.appid == "900001"),
            "the `{key}` spelling must resolve to the appid"
        );
    }
}

// proves: design 025's Test strategy #3 (Chosen design: Emission — "requires 1–20
// ASCII digits; anything else is the same skip-and-count outcome") — the value
// refusals: absent, empty, whitespace-only after trim, non-numeric, an embedded `/`,
// and 21 digits all skip the manifest; 20 digits is the accepted extreme; a padded
// value is trimmed, not refused.
#[test]
fn parse_manifest_requires_a_numeric_appid() {
    let text =
        "\"AppState\"\n{\n\t\"name\"\t\t\"Some Game\"\n\t\"installdir\"\t\t\"Some Game\"\n}\n";
    assert!(
        parse_manifest(text).is_none(),
        "an absent appid key must skip the manifest"
    );

    let base = |value: &str| {
        format!(
            "\"AppState\"\n{{\n\t\"appid\"\t\t\"{value}\"\n\t\"name\"\t\t\"Some Game\"\n\t\
             \"installdir\"\t\t\"Some Game\"\n}}\n"
        )
    };
    for bad in [
        "",
        "   ",
        "900o1",
        "900/1",
        "123456789012345678901", // 21 digits
        "-1",
    ] {
        assert!(
            parse_manifest(&base(bad)).is_none(),
            "appid `{bad}` is outside the 1–20 ASCII-digit set and must skip"
        );
    }

    let twenty = "1".repeat(20);
    let m = parse_manifest(&base(&twenty)).expect("20 digits is inside the accepted set");
    assert_eq!(m.appid, twenty);

    let m = parse_manifest(&base(" 900001 ")).expect("the design trims the value");
    assert_eq!(
        m.appid, "900001",
        "the appid is read trimmed, not refused for padding"
    );
}

// proves: Test strategy #2 (Chosen design §5, A1) — a manifest with no `name`
// yields `display_name == installdir`.
#[test]
fn parse_manifest_falls_back_to_installdir_when_name_is_absent() {
    let m = parse_manifest(&manifest_text(None, Some("Nameless Install")))
        .expect("a manifest without a name must fall back to installdir");
    assert_eq!(m.display_name, "Nameless Install");
    assert_eq!(m.install_dir, "Nameless Install");
}

// proves: Test strategy #2 (Chosen design §5) — an empty `name` falls back to
// `installdir` just like an absent one.
#[test]
fn parse_manifest_falls_back_to_installdir_when_name_is_empty() {
    let m = parse_manifest(&manifest_text(Some(""), Some("Nameless Install")))
        .expect("an empty name must fall back to installdir");
    assert_eq!(m.display_name, "Nameless Install");
}

// proves: Test strategy #2 (Chosen design §5) — a manifest with neither `name`
// nor `installdir` yields None.
#[test]
fn parse_manifest_rejects_manifest_with_neither_name_nor_installdir() {
    assert!(
        parse_manifest(&manifest_text(None, None)).is_none(),
        "a manifest with neither name nor installdir must be skipped"
    );
}

// proves: Test strategy #2 (Chosen design §5) — installdir is rejected when it
// is absent, empty, `.`, `..`, absolute, or contains a path separator.
#[test]
fn parse_manifest_rejects_unusable_installdir() {
    let absent = parse_manifest(&manifest_text(Some("Some Game"), None));
    assert!(absent.is_none(), "an absent installdir must be skipped");
    for bad in ["", ".", "..", "/abs/path", "sub/dir"] {
        assert!(
            parse_manifest(&manifest_text(Some("Some Game"), Some(bad))).is_none(),
            "installdir `{bad}` must be rejected: it is appended unvalidated to the emitted path"
        );
    }
}

// proves: Test strategy #2 (Chosen design §5, §7) — a manifest whose text fails
// to parse yields None (skip-and-continue).
#[test]
fn parse_manifest_rejects_unparseable_text() {
    let text = "\"AppState\"\n{\n\t\"name\" \"Example Game\"\n"; // unbalanced braces
    assert!(
        parse_manifest(text).is_none(),
        "a manifest that fails to parse must be skipped"
    );
}

// proves: Test strategy #2 (Chosen design §5) — a KeyValues document without an
// `AppState` section yields None.
#[test]
fn parse_manifest_rejects_document_without_appstate_section() {
    let text =
        "\"NotAppState\"\n{\n\t\"name\" \"Example Game\"\n\t\"installdir\" \"Example Game\"\n}\n";
    assert!(
        parse_manifest(text).is_none(),
        "a document without an AppState section must be skipped"
    );
}

// proves: Test strategy #2 (Chosen design §5, A5) — the `AppState` root section
// and the task keys are taken case-insensitively. Re-pinned by ADR-025 (Test
// strategy #3): the section now carries the `appid` the row identity is built
// from — without it the manifest skips, so the mixed-case text rides one too.
#[test]
fn parse_manifest_reads_appstate_and_keys_case_insensitively() {
    let text = "\"appstate\"\n{\n\t\"Appid\"\t\t\"900001\"\n\t\"NAME\"\t\t\"Mixed Case Game\"\n\t\"stateflags\"\t\t\"4\"\n\t\"INSTALLDIR\"\t\t\"Mixed Case Game\"\n}\n";
    let m = parse_manifest(text).expect("mixed-case keys must resolve");
    assert_eq!(m.display_name, "Mixed Case Game");
    assert_eq!(m.install_dir, "Mixed Case Game");
    assert_eq!(
        m.appid, "900001",
        "the mixed-case `Appid` spelling resolves too"
    );
}

// ---------------------------------------------------------------------------
// libraries_in_root (Chosen design §3, §2 canonical path rule)
// ---------------------------------------------------------------------------

// proves: Test strategy #3 (Chosen design §3) — the root itself and the
// relative-path secondary library are both returned, canonicalized and sorted,
// and `"0" → ".."` adds no duplicate against the always-included root.
#[test]
fn libraries_in_root_returns_root_and_secondary_canonicalized_sorted_and_deduped() {
    let root = fixture("minimal");
    let libs = libraries_in_root(&root);
    let mut expected = vec![canonical(&root), canonical(&root.join("secondary"))];
    expected.sort();
    assert_eq!(
        libs, expected,
        "the root and `../secondary` must be the two libraries, canonicalized and sorted"
    );
}

// proves: Test strategy #3 (Chosen design §3) — a root with no `steamapps`
// directory yields an empty list.
#[test]
fn libraries_in_root_of_directory_without_steamapps_is_empty() {
    let not_a_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/steam");
    assert_eq!(
        libraries_in_root(&not_a_root),
        Vec::<PathBuf>::new(),
        "a directory without a `steamapps` directory is not a root"
    );
}

// proves: Test strategy #3 (Chosen design §3) — a root whose
// `libraryfolders.vdf` is absent still yields the root itself.
#[test]
fn libraries_in_root_without_libraryfolders_vdf_still_yields_the_root() {
    let root = fixture("all-broken"); // has `steamapps/`, no `libraryfolders.vdf`
    assert_eq!(
        libraries_in_root(&root),
        vec![canonical(&root)],
        "a missing libraryfolders.vdf degrades to the root library only, never to zero"
    );
}

// proves: Test strategy #4 (A4) — no path under `compatdata/` is ever returned
// by `libraries_in_root`.
#[test]
fn libraries_in_root_never_returns_compatdata_paths() {
    for lib in libraries_in_root(&fixture("minimal")) {
        assert!(
            !lib.to_string_lossy().contains("compatdata"),
            "a Proton prefix path must never be treated as a library: {lib:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// host_candidates (Chosen design §2, A8/A9)
// ---------------------------------------------------------------------------

// proves: Test strategy #5 (Chosen design §2) — with `home = /home/t` and an
// empty environment the list is exactly the ordered $HOME candidates.
#[test]
fn host_candidates_lists_ordered_home_candidates_for_empty_environment() {
    assert_eq!(
        host_candidates(Path::new("/home/t"), &no_env),
        paths(&[
            "/home/t/.local/share/Steam",
            "/home/t/.steam/steam",
            "/home/t/.steam/root",
            "/home/t/.steam/debian-installation",
            "/home/t/.var/app/com.valvesoftware.Steam/.local/share/Steam",
            "/home/t/.var/app/com.valvesoftware.Steam/.steam/steam",
            "/home/t/.var/app/com.valvesoftware.Steam/.steam/root",
            "/home/t/snap/steam/common/.local/share/Steam",
            "/home/t/snap/steam/common/.steam/steam",
            "/home/t/snap/steam/common/.steam/root",
        ])
    );
}

// proves: Test strategy #5 (Chosen design §2) — an absolute `XDG_DATA_HOME`
// replaces candidate 1.
#[test]
fn host_candidates_xdg_data_home_replaces_first_candidate() {
    let vars = [("XDG_DATA_HOME", "/x/data")];
    let env = env_with(&vars);
    assert_eq!(
        host_candidates(Path::new("/home/t"), &env),
        paths(&[
            "/x/data/Steam",
            "/home/t/.steam/steam",
            "/home/t/.steam/root",
            "/home/t/.steam/debian-installation",
            "/home/t/.var/app/com.valvesoftware.Steam/.local/share/Steam",
            "/home/t/.var/app/com.valvesoftware.Steam/.steam/steam",
            "/home/t/.var/app/com.valvesoftware.Steam/.steam/root",
            "/home/t/snap/steam/common/.local/share/Steam",
            "/home/t/snap/steam/common/.steam/steam",
            "/home/t/snap/steam/common/.steam/root",
        ])
    );
}

// proves: Test strategy #5 (A9) — a set-but-relative `XDG_DATA_HOME` is ignored
// and the default `$HOME/.local/share/Steam` candidate stands.
#[test]
fn host_candidates_ignores_relative_xdg_data_home() {
    let vars = [("XDG_DATA_HOME", "relative/steam-data")];
    let env = env_with(&vars);
    assert_eq!(
        host_candidates(Path::new("/home/t"), &env),
        paths(&[
            "/home/t/.local/share/Steam",
            "/home/t/.steam/steam",
            "/home/t/.steam/root",
            "/home/t/.steam/debian-installation",
            "/home/t/.var/app/com.valvesoftware.Steam/.local/share/Steam",
            "/home/t/.var/app/com.valvesoftware.Steam/.steam/steam",
            "/home/t/.var/app/com.valvesoftware.Steam/.steam/root",
            "/home/t/snap/steam/common/.local/share/Steam",
            "/home/t/snap/steam/common/.steam/steam",
            "/home/t/snap/steam/common/.steam/root",
        ])
    );
}

// proves: Test strategy #5 (Chosen design §2) — `SNAP_USER_DATA` replaces the
// base of the Snap trio.
#[test]
fn host_candidates_snap_user_data_replaces_snap_trio_base() {
    let vars = [("SNAP_USER_DATA", "/snap/user")];
    let env = env_with(&vars);
    assert_eq!(
        host_candidates(Path::new("/home/t"), &env),
        paths(&[
            "/home/t/.local/share/Steam",
            "/home/t/.steam/steam",
            "/home/t/.steam/root",
            "/home/t/.steam/debian-installation",
            "/home/t/.var/app/com.valvesoftware.Steam/.local/share/Steam",
            "/home/t/.var/app/com.valvesoftware.Steam/.steam/steam",
            "/home/t/.var/app/com.valvesoftware.Steam/.steam/root",
            "/snap/user/steam/common/.local/share/Steam",
            "/snap/user/steam/common/.steam/steam",
            "/snap/user/steam/common/.steam/root",
        ])
    );
}

// proves: Test strategy #5 (Chosen design §2, A9) — an empty or relative home
// yields the empty list.
#[test]
fn host_candidates_of_empty_or_relative_home_is_empty() {
    assert_eq!(
        host_candidates(Path::new(""), &no_env),
        Vec::<PathBuf>::new(),
        "an empty home must yield no candidates"
    );
    assert_eq!(
        host_candidates(Path::new("relative/home"), &no_env),
        Vec::<PathBuf>::new(),
        "a relative home must yield no candidates"
    );
}

// ---------------------------------------------------------------------------
// SteamLinuxPlugin (Chosen design §1, §5, §6, §7)
// ---------------------------------------------------------------------------

// proves: Chosen design §1 — the plugin registers under the name "steam-linux".
#[test]
fn plugin_name_is_steam_linux() {
    assert_eq!(
        SteamLinuxPlugin::from_roots(Vec::new()).name(),
        "steam-linux"
    );
}

// proves: Test strategy #8 — the minimal fixture root yields exactly four games
// in slug order: the two libraries, the name-absent fallback, mixed key case,
// the skipped malformed and escaping manifests, and the ignored decoys. Re-pinned
// by ADR-025: the slug is the `steam/<appid>` lookup key read from each ACF, so the
// row order is the byte order of the appids and the display names travel in their
// own column.
#[test]
fn discover_fixture_root_yields_the_four_expected_games() {
    let games = minimal_games();
    let slugs: Vec<&str> = games.iter().map(|g| g.slug.as_str()).collect();
    assert_eq!(
        slugs,
        [
            "steam/900001",
            "steam/900002",
            "steam/900003",
            "steam/900100",
        ]
    );
    let names: Vec<&str> = games.iter().map(|g| g.display_name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Example Game",
            "Nameless Install",
            "Mixed Case Game",
            "Secondary Title",
        ]
    );
    let friendly: Vec<&str> = games.iter().map(|g| g.friendly_name.as_str()).collect();
    assert_eq!(
        friendly,
        [
            "Example Game",
            "Nameless Install",
            "Mixed Case Game",
            "Secondary Title",
        ],
        "each row carries the normalized display name as its friendly name"
    );
}

// proves: Test strategy #6 (Chosen design §5, A2) — the output contract: every
// game carries platform SteamLinux, codec_id None and the `steam/<appid>` lookup key
// read from its manifest; install_path is `<canonical library>/steamapps/common/\
// <installdir>`; the vector is ordered by (slug, install_path). Re-pinned by ADR-025:
// the slug is the composed lookup key — grammar-exempt, inside `025-B1` — and the
// row carries no namespace at all.
#[test]
fn discover_output_honours_the_game_output_contract() {
    let root = canonical(&fixture("minimal"));
    let games = minimal_games();

    for game in &games {
        assert_eq!(game.platform, Platform::SteamLinux);
        assert!(game.codec_id.is_none(), "codec_id must stay None");
        let appid = game
            .slug
            .strip_prefix("steam/")
            .unwrap_or_else(|| panic!("`{}` must be a `steam/<appid>` lookup key", game.slug));
        assert!(
            !appid.is_empty() && appid.bytes().all(|byte| byte.is_ascii_digit()),
            "`{}` must carry the numeric appid the manifest names: identity comes from \
             the ACF, never from the display name",
            game.slug
        );
        assert!(
            game.slug.len() <= bindforge_core::MAX_DISCOVERY_SLUG_OCTETS,
            "`{}` must sit inside `025-B1`'s lookup-key bound",
            game.slug
        );
        assert!(
            !bindforge_core::is_slug_name(&game.slug),
            "`{}` is grammar-invalid by design — the lookup key never becomes a product id",
            game.slug
        );
        assert_eq!(
            game.friendly_name, game.display_name,
            "the fixture names need no normalization, so friendly and display agree"
        );
        assert!(
            game.install_path.starts_with(&root),
            "{:?} must start with the canonicalized fixture root {root:?}",
            game.install_path
        );
    }

    let by_slug: std::collections::BTreeMap<&str, &Path> = games
        .iter()
        .map(|g| (g.slug.as_str(), g.install_path.as_path()))
        .collect();
    // §5/A2: `install_path` composes from the library that holds the manifest,
    // so the secondary library's game carries the `<root>/secondary` prefix.
    for (slug, library, install_dir) in [
        ("steam/900001", "", "Example Game"),
        ("steam/900002", "", "Nameless Install"),
        ("steam/900003", "", "Mixed Case Game"),
        ("steam/900100", "secondary", "Secondary Title"),
    ] {
        let library_root = if library.is_empty() {
            root.clone()
        } else {
            canonical(&root.join(library))
        };
        let expected = library_root
            .join("steamapps")
            .join("common")
            .join(install_dir);
        assert_eq!(
            by_slug.get(slug).copied(),
            Some(expected.as_path()),
            "install_path for {slug} must be the canonical library's steamapps/common path (emitted even though it does not exist)"
        );
    }

    let keys: Vec<(&str, &Path)> = games
        .iter()
        .map(|g| (g.slug.as_str(), g.install_path.as_path()))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(
        keys, sorted,
        "output must be ordered by (slug, install_path)"
    );
}

// proves: Test strategy #4 (A4) — nothing under `compatdata/` ever appears in
// plugin output: the prefix `libraryfolders.vdf` decoy is never opened and its
// `appmanifest_900099.acf` decoy is never discovered.
#[test]
fn discover_never_emits_compatdata_paths_or_decoy_games() {
    let games = minimal_games();
    assert_eq!(
        games.len(),
        4,
        "only the four fixture games may be discovered"
    );
    for game in &games {
        assert!(
            !game.install_path.to_string_lossy().contains("compatdata"),
            "a Proton prefix path must never appear in output: {:?}",
            game.install_path
        );
        assert!(
            !game.slug.contains("900099"),
            "the compatdata decoy game must never be discovered: {:?}",
            game.slug
        );
    }
}

// ---------------------------------------------------------------------------
// Failure policy (Chosen design §7)
// ---------------------------------------------------------------------------

// proves: Test strategy #7 (Chosen design §7) — a nonexistent root is "not
// installed here", not an error: `Ok(vec![])`.
#[test]
fn discover_with_nonexistent_root_returns_empty_ok() {
    let missing = fixture("minimal").join("no-such-steam-root");
    let games = SteamLinuxPlugin::from_roots([missing])
        .discover()
        .expect("an unresolvable root must yield Ok, not Err");
    assert!(games.is_empty(), "no roots must yield zero games");
}

// proves: Test strategy #7 (Chosen design §7) — a selected root whose every
// manifest is malformed returns one `DiscoveryError::PluginError` naming the
// selected root and carrying no file contents.
#[test]
fn discover_of_all_broken_root_returns_plugin_error_naming_root_without_contents() {
    let result = SteamLinuxPlugin::from_roots([fixture("all-broken")]).discover();
    match result {
        Ok(games) => panic!(
            "a root that emitted zero games with files skipped must be Err, got {} games",
            games.len()
        ),
        Err(DiscoveryError::PluginError(msg)) => {
            assert!(
                msg.contains("all-broken"),
                "the message must name the selected root, got: {msg}"
            );
            for leak in [
                "AppState",
                "appid",
                "installdir",
                "BrokenMarker900201",
                "BrokenMarker900202",
                "Broken Install",
            ] {
                assert!(
                    !msg.contains(leak),
                    "the message must not leak file contents (`{leak}` found): {msg}"
                );
            }
        }
        Err(other) => panic!("expected DiscoveryError::PluginError, got {other:?}"),
    }
}
