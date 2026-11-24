//! The corpus boundary of design 025 / ADR-025 (`025-L5`): the source key and the
//! friendly name are the identifiers that must never reach a manifest, a profile, or
//! a path component — only the curated `game_id` may.
//!
//! proves: design 025's Test strategy #16 (Chosen design: Corpus boundary and ledger
//! — "`crates/discovery/tests/boundary_tests.rs` (new): for every slug the Steam
//! fixture trees emit, `!is_slug_name(slug)` and the slug is inside `025-B1` — the
//! structural proof, for the source key only, that it cannot become a product id,
//! because the grammar refuses it"). The friendly-name half of `025-L5` rests on the
//! match-store rule that only `game_id` is ever an id or path component — no grammar
//! assertion is made about friendly names here.
//!
//! It also pins the two corpus halves the same section names: the shipped corpus
//! carries no source key (`data/` and the valid schema fixtures contain no `steam/`,
//! `epic/`, or `local/` spelling — Rollout 4 against A20's observation), and
//! `schemas/manifests/v1/fixtures/invalid/id-source-key.toml` stands in the invalid
//! corpus for the conformance walk (`crates/core/tests/conformance.rs`) to refuse.
//!
//! Unit-level: reads the checked-in fixture trees and corpus, writes nothing, no
//! network.

use bindforge_core::{MAX_DISCOVERY_SLUG_OCTETS, is_slug_name};
use discovery::game::DiscoveredGame;
use discovery::plugin::DiscoveryPlugin;
use discovery::steam_linux::SteamLinuxPlugin;
use discovery::steam_windows::SteamWindowsPlugin;
use std::path::{Path, PathBuf};

/// A checked-in fixture root under `tests/fixtures/steam/`.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/steam")
        .join(name)
}

/// The workspace root: `crates/discovery` sits two levels below it.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/discovery sits directly under the workspace root")
        .to_path_buf()
}

/// Every slug the Steam fixture trees emit, through both shipped plugins.
fn fixture_slugs() -> Vec<String> {
    let mut slugs: Vec<String> = SteamLinuxPlugin::from_roots([fixture("minimal")])
        .discover()
        .expect("the minimal fixture root must discover")
        .iter()
        .map(|game: &DiscoveredGame| game.slug.clone())
        .collect();
    slugs.extend(
        SteamWindowsPlugin::from_roots([fixture("minimal")])
            .discover()
            .expect("the Windows scan of the shared fixture must succeed")
            .iter()
            .map(|game| game.slug.clone()),
    );
    assert!(
        !slugs.is_empty(),
        "the boundary proof is about the fixture slugs; the trees must emit rows"
    );
    slugs
}

/// proves: design 025's Test strategy #16 (Chosen design: Corpus boundary — the
/// structural proof) — for every slug the Steam fixture trees emit,
/// `!is_slug_name(slug)`: the shared product-slug grammar (A5's `[a-z][a-z0-9-]*`,
/// which refuses any `/`) rejects the source key, so it can never be adopted as a
/// manifest `id` or a path component, exactly as `025-L5` requires.
#[test]
fn every_fixture_emitted_slug_fails_the_product_slug_grammar() {
    for slug in fixture_slugs() {
        assert!(
            !is_slug_name(&slug),
            "`{slug}` — a source key that passed the product-slug grammar could be \
             adopted as a product id, which `025-L5` forbids: the grammar must refuse \
             every emitted lookup key"
        );
        assert!(
            slug.starts_with("steam/"),
            "`{slug}` must carry its source token: the `steam` token is the only one \
             with a shipped emitter this run (`025-C1`, `025-R2`)"
        );
    }
}

/// proves: design 025's Test strategy #16 + `025-B1` — every slug the Steam fixture
/// trees emit sits inside `MAX_DISCOVERY_SLUG_OCTETS`: the lookup-key bound is the
/// one thing standing between an emitted key and an unbounded string, and it is the
/// only check the emit boundary makes (`025-L3`).
#[test]
fn every_fixture_emitted_slug_is_inside_the_discovery_bound() {
    for slug in fixture_slugs() {
        assert!(
            slug.len() <= MAX_DISCOVERY_SLUG_OCTETS,
            "`{slug}` is {} octets, past `025-B1`'s cap of {MAX_DISCOVERY_SLUG_OCTETS}",
            slug.len()
        );
    }
}

/// `catalog` and `bindings` are the trees in the contract corpus where a
/// `<source>/<external-id>` lookup key is legal content (design 026: Catalog
/// document and its spec, §The seed corpus — `026-C3` documents are built out
/// of source keys; design 017 §Corpus boundary (canonical) — a `bindings/v1`
/// document's `game` key *is* the lookup key, which is why the only carve-out
/// is the format's own fixture tree, no `data/bindings/` ships). Every other
/// tree stays under the hygiene gate.
const SOURCE_KEY_TREES: [&str; 2] = ["catalog", "bindings"];

/// proves: design 025's Test strategy #16 (Chosen design: Corpus boundary — the
/// corpus sweep) — no file under `data/` or `schemas/*/v*/fixtures/valid/` contains
/// the `steam/`, `epic/`, or `local/` spelling: Rollout 4's "no source key or
/// friendly name reaches a manifest, a profile, or a path" pinned literally against
/// the shipped corpus (A20 re-asserted as a gate, not an observation). Both roots
/// stay in the sweep (design 026: the rejected "just don't put it in `schemas/`"
/// dodge); only files under `data/catalog/`, `schemas/catalog/` and
/// `schemas/bindings/` are skipped — `026-S1` mandates source keys as content
/// there and design 017's `game` key spells one (`SOURCE_KEY_TREES` names the
/// trees) — and the `checked > 0` guard is unchanged.
#[test]
fn the_shipped_corpus_carries_no_source_key_spelling() {
    let root = repo_root();
    let mut checked = 0usize;
    let mut violations: Vec<String> = Vec::new();

    let skipped: Vec<PathBuf> = SOURCE_KEY_TREES
        .iter()
        .flat_map(|tree| {
            [
                root.join("data").join(tree),
                root.join("schemas").join(tree),
            ]
        })
        .collect();

    let mut dirs: Vec<PathBuf> = vec![root.join("data")];
    let schemas = root.join("schemas");
    for format in read_dir_sorted(&schemas) {
        if !format.is_dir() {
            continue;
        }
        for major in read_dir_sorted(&format) {
            if !major.is_dir() {
                continue;
            }
            let valid = major.join("fixtures").join("valid");
            if valid.is_dir() {
                dirs.push(valid);
            }
        }
    }

    for dir in dirs {
        for path in walk_files(&dir) {
            if skipped.iter().any(|skip| path.starts_with(skip)) {
                continue;
            }
            checked += 1;
            let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("reading {}: {err}", path.display());
            });
            for token in ["steam/", "epic/", "local/"] {
                if text.contains(token) {
                    violations.push(format!("{} contains `{token}`", path.display()));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "the shipped corpus must not carry a discovery source key as an id:\n{}",
        violations.join("\n")
    );
    assert!(
        checked > 0,
        "the sweep must actually read corpus files under `data/` and the valid \
         schema fixtures; it read {checked}"
    );
}

/// proves: design 025's Test strategy #16 (Chosen design: Corpus boundary — the new
/// invalid manifest fixture) — `schemas/manifests/v1/fixtures/invalid/id-source-key.toml`
/// ships with a `steam/900001` `id`; the existing conformance walk
/// (`crates/core/tests/conformance.rs`, A12) refuses every invalid fixture with a
/// structured single-line message, so the fixture needs no test of its own beyond
/// its presence and its spelling — the grammar refusal is enforced by that walk.
#[test]
fn the_invalid_manifest_corpus_carries_the_source_key_id_fixture() {
    let path = repo_root()
        .join("schemas")
        .join("manifests")
        .join("v1")
        .join("fixtures")
        .join("invalid")
        .join("id-source-key.toml");
    assert!(
        path.is_file(),
        "`{}` must ship: ADR-025 states a source key can never be a manifest `id`, \
         and the invalid corpus is where that refusal is pinned",
        path.display()
    );
    let text = std::fs::read_to_string(&path).expect("the fixture is readable");
    assert!(
        text.contains("id = \"steam/900001\""),
        "the fixture's `id` must be the `steam/900001` lookup key — the exact spelling a \
         discovered Steam row carries:\n{text}"
    );
}

fn read_dir_sorted(path: &Path) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(path).unwrap_or_else(|err| panic!("reading {path:?}: {err}"));
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| entry.expect("readable entry").path())
        .collect();
    paths.sort();
    paths
}

/// Every regular file under `dir`, recursively, sorted. The corpus sweep walks
/// `data/` (which holds READMEs today) and each `fixtures/valid` directory.
fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return files;
    }
    for entry in read_dir_sorted(dir) {
        if entry.is_dir() {
            files.extend(walk_files(&entry));
        } else if entry.is_file() {
            files.push(entry);
        }
    }
    files
}
