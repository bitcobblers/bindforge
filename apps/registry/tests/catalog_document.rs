//! The `026-C3` catalog-document loader: every valid document loads, every
//! out-of-shape document is a named refusal, and nothing on disk is rewritten by
//! a refusal.
//!
//! proves: design 026's Test strategy #1 — "Catalog-document loader (`026-C3`):
//! every `fixtures/valid/*.json` loads; each of the following — wrong `version`,
//! negative `revision`, grammar-invalid `game_id` (A4), repeated `game_id`,
//! `name` over 128 octets (A5), a `source_key` failing §The serving face step 1's
//! rule (A3, A16), a `source_key` repeated within one record, malformed record,
//! document over `MAX_CATALOG_DOCUMENT_OCTETS` — ships in `fixtures/invalid/
//! *.json` as a named refusal; nothing on disk is rewritten".
//!
//! The loader is pinned through the only surface the design fixes for it:
//! `registry serve --root --corpus <path>` must refuse an unloadable corpus
//! before it answers anything (`026-C1`, A22's metadata-gated load discipline).
//! Every corpus lives in a temp directory that is removed on drop, and each
//! refusal re-reads the file to prove it came back byte-for-byte.

mod common;

use common::{
    TempDir, corpus_doc, expect_refused_startup, registry_document, repo_root, slug_query,
    status_code,
};
use std::path::Path;

/// The `<data>/bindforge/registry/` document is irrelevant here; root mode reads
/// only `--corpus`.
fn refuse(data: &TempDir, label: &str, document: &str, named: &[&str]) {
    let file = format!("{label}.json");
    let path = data.write(&file, document.as_bytes());
    let before = std::fs::read(&path).expect("the corpus fixture is readable");

    let run = expect_refused_startup(
        &[
            "serve",
            "--root",
            "--corpus",
            path.to_str().expect("a utf-8 temp corpus path"),
        ],
        &[],
    );

    let lines: Vec<&str> = run.stderr.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "the `{label}` refusal is exactly one named stderr line: {:?}",
        run.stderr
    );
    for needle in named.iter().chain(std::iter::once(&file.as_str())) {
        assert!(
            lines[0].contains(needle),
            "the `{label}` refusal must be named — it has to mention {needle:?} (and the \
             file it refused): {:?}",
            lines[0]
        );
    }
    assert!(
        !run.stderr.contains("panicked at"),
        "the `{label}` refusal is a clean error path: {}",
        run.stderr
    );

    let after = std::fs::read(&path).expect("the corpus fixture is still there");
    assert_eq!(
        after, before,
        "a refused corpus is never rewritten on disk (`026-C3`, A7's load discipline)"
    );
    assert_eq!(
        data.files_below(".").len(),
        1,
        "a refused corpus writes nothing else — no tmp, no side file: {:?}",
        data.files_below(".")
    );
}

fn accepts(label: &str, path: &Path) {
    let server = common::Registry::root(label, path, &[]);
    assert!(
        server.port() != 0,
        "`{label}` loads and serves on the port it printed ({})",
        server.port()
    );
}

/// proves: design 026's Test strategy #1, first clause — `schemas/catalog/v1/`
/// ships a JSON format whose valid fixtures all load and whose invalid fixtures
/// are all refused by name: the fixture corpora *are* the refusal contract
/// (`schemas/README.md` discipline), and the zero-fixture rule requires at least
/// one of each.
#[test]
fn every_catalog_fixture_loads_or_is_the_named_refusal() {
    let fixtures = repo_root().join("schemas").join("catalog").join("v1");
    let valid = fixtures.join("fixtures").join("valid");
    let invalid = fixtures.join("fixtures").join("invalid");
    for dir in [&valid, &invalid] {
        assert!(
            dir.is_dir(),
            "`026-C3` fixes one spec at `schemas/catalog/v1/`, shipping \
             fixtures/valid and fixtures/invalid: missing {}",
            dir.display()
        );
    }

    let valid_files = json_files(&valid);
    let invalid_files = json_files(&invalid);
    assert!(
        !valid_files.is_empty(),
        "`schemas/catalog/v1/fixtures/valid/` must ship at least one `.json` fixture"
    );
    assert!(
        !invalid_files.is_empty(),
        "`schemas/catalog/v1/fixtures/invalid/` must ship at least one `.json` fixture"
    );

    for path in &valid_files {
        accepts(
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("valid"),
            path,
        );
    }
    for path in &invalid_files {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("invalid")
            .to_string();
        let document = std::fs::read_to_string(path).expect("the fixture is readable");
        let data = TempDir::new("catalog-fixture");
        let copied = data.write(&format!("copied-{name}"), document.as_bytes());
        let run = expect_refused_startup(
            &[
                "serve",
                "--root",
                "--corpus",
                copied.to_str().expect("a utf-8 temp path"),
            ],
            &[],
        );
        let lines: Vec<&str> = run.stderr.lines().collect();
        assert_eq!(
            lines.len(),
            1,
            "the invalid fixture `{name}` is refused with exactly one named line: {:?}",
            run.stderr
        );
    }
}

fn json_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

/// proves: Test strategy #1 (wrong `version`) — the bare integer `1` is the only
/// version `026-C3` spells, and the refusal is in `corpus.rs`, named (Chosen
/// design: "the version-refusal lives in `corpus.rs`, named, not in core's
/// `UnsupportedVersion` path").
#[test]
fn a_catalog_document_of_the_wrong_version_is_refused_by_name() {
    for version in ["0", "2", "\"v1\"", "null"] {
        let data = TempDir::new("catalog-version");
        let document = format!(
            "{{\"version\": {version}, \"revision\": 0, \"games\": \
             [{{\"game_id\": \"example-game\", \"name\": \"Example\", \
             \"source_keys\": [\"steam/900001\"]}}]}}"
        );
        refuse(&data, "wrong-version", &document, &["version"]);
    }
}

/// proves: Test strategy #1 (negative `revision`) — `026-C3` fixes a non-negative
/// integer bumped by every mutation.
#[test]
fn a_negative_revision_is_refused_by_name() {
    let data = TempDir::new("catalog-revision");
    let document = "{\"version\": 1, \"revision\": -1, \"games\": []}".to_string();
    refuse(&data, "negative-revision", &document, &["revision"]);
}

/// proves: Test strategy #1 (grammar-invalid `game_id`, A4) — `game_id` rides the
/// shared slug grammar `[a-z][a-z0-9-]*` with `MAX_SLUG_OCTETS = 128`
/// (`013-L5`, `024-B1`), checked at load.
#[test]
fn a_grammar_invalid_game_id_is_refused_by_name() {
    let over_bound = "a".repeat(129);
    let out_of_grammar = [
        ("uppercase", "Example-Game"),
        ("underscore", "example_game"),
        ("leading-digit", "1example"),
        ("dot", "example.game"),
        ("slash", "a/b"),
        ("empty", ""),
        ("over-bound", over_bound.as_str()),
    ];
    for (label, game_id) in out_of_grammar {
        let data = TempDir::new("catalog-game-id");
        let keys = ["steam/900001"];
        let document = corpus_doc(0, &[(game_id, "Example Game", &keys[..])]);
        refuse(&data, label, &document, &["game_id"]);
    }
}

/// proves: Test strategy #1 (repeated `game_id`) — records are keyed by
/// `game_id`, so a duplicate "loads ambiguously" (Chosen design: Catalog document
/// and its spec) and is refused rather than resolved.
#[test]
fn a_repeated_game_id_is_refused_by_name() {
    let data = TempDir::new("catalog-duplicate-id");
    let document = corpus_doc(
        0,
        &[
            ("example-game", "First", &["steam/900001"][..]),
            ("example-game", "Second", &["steam/900002"][..]),
        ],
    );
    refuse(
        &data,
        "repeated-game-id",
        &document,
        &["game_id", "example-game"],
    );
}

/// proves: Test strategy #1 (`name` over 128 octets, A5 / `013-B4`
/// `MAX_NATIVE_NAME_OCTETS`) — one octet over the bound is a named refusal.
#[test]
fn a_name_over_the_native_name_bound_is_refused_by_name() {
    let data = TempDir::new("catalog-name");
    let long_name = "n".repeat(129);
    let keys = ["steam/900001"];
    let document = corpus_doc(0, &[("example-game", long_name.as_str(), &keys[..])]);
    refuse(&data, "name-over-bound", &document, &["name"]);
}

/// proves: Test strategy #1 (a `source_key` failing the shipped lookup-key rule,
/// A3 + A16) — `spells_lookup_key` over the closed `steam` · `epic` · `local`
/// token set, inside `025-B1`'s 128-octet cap.
#[test]
fn a_source_key_outside_the_lookup_key_rule_is_refused_by_name() {
    let over_cap = format!("steam/{}", "9".repeat(130));
    let bad_keys = [
        ("unknown-source", "gog/900001"),
        ("missing-separator", "steam-900001"),
        ("empty-external-id", "steam/"),
        ("empty-source", "/900001"),
        ("second-separator", "steam/900001/eu"),
        ("over-cap", over_cap.as_str()),
    ];
    for (label, source_key) in bad_keys {
        let data = TempDir::new("catalog-source-key");
        let keys = [source_key];
        let document = corpus_doc(0, &[("example-game", "Example Game", &keys[..])]);
        refuse(&data, label, &document, &["source_key"]);
    }
}

/// proves: Test strategy #1 (a `source_key` repeated within one record) — the
/// slug→candidates index would post the same game twice and crowd the
/// `MAX_CANDIDATES` window (Chosen design: Catalog document and its spec).
#[test]
fn a_source_key_repeated_within_one_record_is_refused_by_name() {
    let data = TempDir::new("catalog-duplicate-key");
    let document = corpus_doc(
        0,
        &[(
            "example-game",
            "Example Game",
            &["steam/900001", "steam/900001"][..],
        )],
    );
    refuse(
        &data,
        "repeated-source-key",
        &document,
        &["source_key", "steam/900001"],
    );
}

/// proves: Test strategy #1 (malformed record) — a non-object/missing field is a
/// named refusal, and so is a document that is not an object at all.
#[test]
fn a_malformed_record_is_refused_by_name() {
    let malformed = [
        (
            "missing-name",
            "{\"version\": 1, \"revision\": 0, \"games\": [{\"game_id\": \"example-game\", \
             \"source_keys\": [\"steam/900001\"]}]}",
            "name",
        ),
        (
            "missing-source-keys",
            "{\"version\": 1, \"revision\": 0, \"games\": [{\"game_id\": \"example-game\", \
             \"name\": \"Example\"}]}",
            "source_key",
        ),
        (
            "missing-games",
            "{\"version\": 1, \"revision\": 0}",
            "games",
        ),
        (
            "non-object-record",
            "{\"version\": 1, \"revision\": 0, \"games\": [\"example-game\"]}",
            "games",
        ),
        (
            "source-keys-not-an-array",
            "{\"version\": 1, \"revision\": 0, \"games\": [{\"game_id\": \"example-game\", \
             \"name\": \"Example\", \"source_keys\": \"steam/900001\"}]}",
            "source_key",
        ),
        ("not-json", "this is not a document", "version"),
    ];
    for (label, document, named) in malformed {
        let data = TempDir::new("catalog-malformed");
        refuse(&data, label, document, &[named]);
    }
}

/// proves: Test strategy #1 (document over `MAX_CATALOG_DOCUMENT_OCTETS`) plus
/// the load discipline behind it (A22): the cap is checked on the file's
/// `fs::metadata` *before* the read, and refused by name.
#[test]
fn a_document_over_the_catalog_octet_cap_is_refused_by_name() {
    const CAP: usize = 16_777_216;
    let data = TempDir::new("catalog-cap");
    // A well-shaped document padded past the cap with legal records: the refusal
    // is the size, not the shape. The running length counts the exact octets the
    // compact final document writes — wrapper, records, and the `,` between
    // records — so the loop stops at the first record that pushes the real
    // document over the cap.
    let mut games: Vec<(String, String, Vec<String>)> = Vec::new();
    let mut bytes = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "revision": 0,
        "games": [],
    }))
    .expect("the empty corpus serialises")
    .len();
    while bytes <= CAP {
        let index = games.len();
        let game_id = format!("game-{index}");
        let source_key = format!("steam/{}000000", index);
        let record = serde_json::to_string(&serde_json::json!({
            "game_id": game_id,
            "name": "A padded game",
            "source_keys": [source_key],
        }))
        .expect("a padded record serialises");
        bytes += record.len() + usize::from(index > 0);
        games.push((game_id, "A padded game".to_string(), vec![source_key]));
        assert!(
            games.len() < 400_000,
            "the fixture generator must terminate"
        );
    }
    let records: Vec<serde_json::Value> = games
        .iter()
        .map(|(game_id, name, keys)| {
            serde_json::json!({"game_id": game_id, "name": name, "source_keys": keys})
        })
        .collect();
    let document = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "revision": 0,
        "games": records,
    }))
    .expect("the padded corpus serialises");
    assert!(
        document.len() > CAP,
        "the fixture is over the cap by construction"
    );
    refuse(&data, "over-cap-document", &document, &[]);
}

/// proves: Test strategy #1's closing clause and the other side of every bound —
/// a record sitting exactly on `MAX_SLUG_OCTETS` and `MAX_NATIVE_NAME_OCTETS`,
/// with a 128-octet lookup key, loads: the loader refuses over the bound, never
/// at it (`024-B1`, `013-B4`, `025-B1` are all caps, not targets).
#[test]
fn a_record_exactly_on_every_length_bound_loads() {
    let data = TempDir::new("catalog-bounds");
    let long_id = "a".repeat(128);
    let long_name = "n".repeat(128);
    let long_key = format!("steam/{}", "9".repeat(122));
    assert_eq!(long_key.len(), 128, "the fixture is exactly on `025-B1`");
    let keys = [long_key.as_str()];
    let document = corpus_doc(0, &[(long_id.as_str(), long_name.as_str(), &keys[..])]);
    let path = data.write("on-the-bound.json", document.as_bytes());
    accepts("on-the-bound", &path);
}

/// proves: Test strategy #1 read against `026-S1` — the seed corpus at
/// `data/catalog/v1/catalog.json` is a `026-C3` instance the registry's own loader
/// accepts, whose `game_id`s are grammar-valid kebab strings each carrying its
/// `steam/<appid>` source key (partially satisfied as designed: the ids are
/// minted, since `data/manifests/` is empty, A14).
#[test]
fn the_seed_corpus_is_a_valid_catalog_document_carrying_steam_source_keys() {
    let seed = repo_root()
        .join("data")
        .join("catalog")
        .join("v1")
        .join("catalog.json");
    assert!(
        seed.is_file(),
        "`026-S1` fixes the seed at data/catalog/v1/catalog.json"
    );
    let text = std::fs::read_to_string(&seed).expect("the seed corpus is readable");
    let document: serde_json::Value = serde_json::from_str(&text).expect("the seed is JSON");
    assert_eq!(
        document["version"].as_i64(),
        Some(1),
        "`026-C3` spells the version as the bare integer 1"
    );
    assert!(
        document["revision"].as_u64().is_some(),
        "`revision` is a non-negative integer"
    );
    let games = document["games"].as_array().expect("games is an array");
    assert!(!games.is_empty(), "the seed opens with a handful of games");
    for game in games {
        let game_id = game["game_id"].as_str().expect("a game_id");
        assert!(
            is_slug_name(game_id) && game_id.len() <= 128,
            "the seed mints grammar-valid ids (`026-S1`, A4): {game_id:?}"
        );
        let keys = game["source_keys"].as_array().expect("source_keys");
        assert!(
            keys.iter()
                .any(|key| key.as_str().is_some_and(|key| key.starts_with("steam/"))),
            "every seed game carries its `steam/<appid>` source key (`026-S1`): {game:?}"
        );
        let name = game["name"].as_str().expect("a name");
        assert!(
            !name.is_empty() && name.len() <= 128,
            "`name` is bounded by `013-B4`: {name:?}"
        );
    }

    let server = common::Registry::root("seed-corpus", &seed, &[]);
    let first_key = games[0]["source_keys"][0].as_str().expect("a source key");
    let answer = server.get(&slug_query(first_key));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "the seed answers its own key"
    );
    assert!(
        common::game_ids(&answer).contains(&games[0]["game_id"].as_str().unwrap().to_string()),
        "the seed corpus joins its first record's key to its curated id"
    );
}

/// `is_slug_name` as the contracts state it: `[a-z][a-z0-9-]*` (`024-B1`).
fn is_slug_name(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The registry documents are named here so the fixture suite keeps the
/// `026-C7` spelling in one place; see `data_layout.rs` for their placement.
#[test]
fn the_registry_document_spellings_are_the_ones_the_contract_names() {
    let data = std::path::Path::new("/sandbox-data");
    assert_eq!(
        registry_document(data, "cache-v1.json"),
        data.join("bindforge")
            .join("registry")
            .join("cache-v1.json")
    );
    assert_eq!(
        registry_document(data, "catalog-v1.json"),
        data.join("bindforge")
            .join("registry")
            .join("catalog-v1.json")
    );
}
