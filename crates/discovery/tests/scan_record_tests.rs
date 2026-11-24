//! `ScanRecord` and `SteamCore::scan()` — the additive discovery seam that
//! keeps the owning root and library beside each row (§Discovery plumbing).
//!
//! proves: Test strategy #1.

mod common;

use common::{SteamRoot, letters};
use discovery::game::{DiscoveredGame, ScanRecord};
use discovery::plugin::DiscoveryPlugin;
use discovery::steam_linux::SteamLinuxPlugin;
use std::fs;
use std::path::{Path, PathBuf};

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|err| panic!("canonicalizing {}: {err}", path.display()))
}

fn records(roots: impl IntoIterator<Item = PathBuf>) -> Vec<ScanRecord> {
    SteamLinuxPlugin::from_roots(roots)
        .scan_records()
        .expect("a fixture scan must succeed")
}

fn by_slug(mut games: Vec<DiscoveredGame>) -> Vec<DiscoveredGame> {
    games.sort_by(|a, b| {
        a.slug
            .cmp(&b.slug)
            .then_with(|| a.install_path.cmp(&b.install_path))
    });
    games
}

// proves: Test strategy #1 (§Discovery plumbing: "the row plus the Steam root
// that *owns* the library it was found in") — each row's `library` is the one
// scanned and its `root` the root that named it: the root itself for its own
// library, the naming root for the `libraryfolders.vdf` secondary.
#[test]
fn each_record_names_the_library_scanned_and_the_root_that_named_it() {
    let root = SteamRoot::new("scan-own", true);
    root.acf("910001", "", Some("Root Game"), "Root Game");
    root.acf(
        "910002",
        "secondary",
        Some("Secondary Game"),
        "Secondary Game",
    );
    let root_path = canonical(root.path());

    let mut recs = records(vec![root_path.clone()]);
    recs.sort_by(|a, b| a.game.slug.cmp(&b.game.slug));
    assert_eq!(recs.len(), 2, "one row per discovered game");

    let (own, secondary) = (&recs[0], &recs[1]);
    assert_eq!(own.game.slug, "steam/910001");
    assert_eq!(
        own.root, root_path,
        "the root's own library is owned by the root itself"
    );
    assert_eq!(own.library, root_path);
    assert_eq!(secondary.game.slug, "steam/910002");
    assert_eq!(
        secondary.library,
        root_path.join("secondary"),
        "the secondary library row carries the library that was scanned"
    );
    assert_eq!(
        secondary.root, root_path,
        "…and the root whose libraryfolders.vdf named it"
    );
}

// proves: Test strategy #1 (§Discovery plumbing: "a library two roots share is
// scanned once, owned by the first root that named it — the same dedupe A2
// describes, now with its owner kept") — both roots' `libraryfolders.vdf` name
// one library; its game appears exactly once, rooted at the first in
// `roots()` (canonical byte) order.
#[test]
fn a_library_two_roots_share_is_scanned_once_owned_by_the_first() {
    let root_a = SteamRoot::new("scan-share-a", false);
    let root_b = SteamRoot::new("scan-share-b", false);
    let shared = root_a.path().join("sharedlib");
    fs::create_dir_all(shared.join("steamapps")).expect("shared library");
    fs::write(
        shared.join("steamapps").join("appmanifest_910003.acf"),
        "\"AppState\"\n{\n\t\"appid\"\t\t\"910003\"\n\t\"Universe\"\t\t\"1\"\n\t\
         \"name\"\t\t\"Shared Game\"\n\t\"StateFlags\"\t\t\"4\"\n\t\
         \"installdir\"\t\t\"Shared Game\"\n\t\"SizeOnDisk\"\t\t\"1000000\"\n}\n",
    )
    .expect("shared acf");
    for root in [&root_a, &root_b] {
        fs::write(
            root.path().join("steamapps").join("libraryfolders.vdf"),
            common_library_folders(&[root.path(), &shared]),
        )
        .expect("libraryfolders.vdf");
    }

    let (can_a, can_b) = (canonical(root_a.path()), canonical(root_b.path()));
    let mut ordered = [can_a.clone(), can_b.clone()];
    ordered.sort();
    let first = ordered[0].clone();

    let recs = records(vec![can_a, can_b]);
    let shared_recs: Vec<&ScanRecord> = recs
        .iter()
        .filter(|rec| rec.game.slug == "steam/910003")
        .collect();
    assert_eq!(
        shared_recs.len(),
        1,
        "the shared library is scanned once, not once per naming root"
    );
    assert_eq!(
        shared_recs[0].root, first,
        "the shared library's row is owned by the first root (roots() byte order) that named it"
    );
    assert_eq!(shared_recs[0].library, canonical(&shared));
}

// proves: Test strategy #1 (§Discovery plumbing: "`discover()` becomes
// `Ok(self.scan()?.into_iter().map(|r| r.game).collect())`") — `discover()`
// returns exactly the rows `scan_records()` returns, in exactly the same
// `(slug, install_path)` order as before the refactor.
#[test]
fn discover_returns_exactly_the_projected_rows_in_the_same_order() {
    let root = SteamRoot::new("scan-order", true);
    root.acf("910011", "", Some(&letters(6, 4)), "Install K"); // steam/910011
    root.acf("910010", "", Some(&letters(5, 4)), "Install J"); // steam/910010
    root.acf("910012", "secondary", Some(&letters(7, 4)), "Install L"); // steam/910012

    let plugin = SteamLinuxPlugin::from_roots(vec![canonical(root.path())]);
    let recs = plugin.scan_records().expect("scan");
    let discovered = plugin.discover().expect("discover");

    let projected: Vec<DiscoveredGame> = recs.iter().map(|rec| rec.game.clone()).collect();
    assert_eq!(
        discovered, projected,
        "discover() is exactly the row projection of scan(), order included"
    );

    // The pre-refactor law: the `(slug, install_path)` sort is the only ordering.
    let sorted = by_slug(discovered.clone());
    let expected_slugs: Vec<&str> = sorted.iter().map(|game| game.slug.as_str()).collect();
    let actual_slugs: Vec<&str> = discovered.iter().map(|game| game.slug.as_str()).collect();
    assert_eq!(
        actual_slugs, expected_slugs,
        "rows come back in (slug, install_path) byte order — unchanged by the refactor"
    );
}

// proves: Test strategy #1 (§Discovery plumbing: "`SteamLinuxPlugin`/
// `SteamWindowsPlugin` expose `scan_records()` and `roots()` as thin inherent
// wrappers; `roots()` is the selected set as the core holds it — canonicalized,
// existence-filtered, byte order") — `roots()` hands the same canonicalized
// byte-ordered set to a `GameContext`.
#[test]
fn roots_exposes_the_selected_set_in_canonical_byte_order() {
    let root_a = SteamRoot::new("scan-roots-a", false);
    let root_b = SteamRoot::new("scan-roots-b", false);
    let plugin = SteamLinuxPlugin::from_roots(vec![
        root_b.path().to_path_buf(),
        PathBuf::from("/definitely/not/a/steam/root"),
        root_a.path().to_path_buf(),
    ]);
    let mut expected = vec![canonical(root_a.path()), canonical(root_b.path())];
    expected.sort();
    assert_eq!(
        plugin.roots(),
        expected,
        "roots() is the resolved set: canonicalized, existence-filtered, byte order"
    );
}

/// `libraryfolders.vdf` naming the given libraries by absolute path.
fn common_library_folders(paths: &[&Path]) -> String {
    let mut text = String::from("\"libraryfolders\"\n{\n");
    for (index, path) in paths.iter().enumerate() {
        text.push_str(&format!(
            "\t\"{index}\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n",
            path.display()
        ));
    }
    text.push_str("}\n");
    text
}
