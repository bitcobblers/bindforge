//! The `bindings/v1` snapshot loader — closed key sets, per-key bounds, and
//! refusals that name the key (§`bindings/v1`, §Read discipline).
//!
//! proves: Test strategy #10 and #11.

mod common;

use bindforge_codecs::families::unreal_ini::UnrealIni;
use bindforge_codecs::{MAX_SNAPSHOT_ENTRIES, MAX_SNAPSHOT_OCTETS};
use bindforge_core::MAX_NATIVE_NAME_OCTETS;
use common::{TempRoot, context, schema_fixture_dir, unreal_claim};

// proves: Test strategy #10 (§`bindings/v1`: "`fixtures/valid/` and
// `fixtures/invalid/` are the conformance currency") — the contract tree
// carries both corpora for `bindings/v1`.
#[test]
fn the_schema_tree_carries_the_bindings_v1_fixture_corpora() {
    for kind in ["valid", "invalid"] {
        let dir = schema_fixture_dir("bindings", "v1", kind);
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("schemas/bindings/v1/fixtures/{kind}/ must exist: {err}"));
        let count = entries.count();
        assert!(
            count > 0,
            "schemas/bindings/v1/fixtures/{kind}/ must carry fixtures"
        );
    }
    assert!(
        std::fs::File::open(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(|c| c.parent())
                .expect("workspace root")
                .join("schemas/bindings/v1/spec.md")
        )
        .is_ok(),
        "`schemas/bindings/v1/spec.md` is the contract's own statement"
    );
}

// proves: Test strategy #10 (§`bindings/v1`) — every `fixtures/valid/*.json`
// loads, and its re-serialization is byte-stable through the model.
#[test]
fn every_valid_schema_fixture_loads() {
    for path in fixture_paths("valid") {
        let loaded = common::load(&path)
            .unwrap_or_else(|err| panic!("{} must load: {err:?}", path.display()));
        let reserialized = serde_json::to_vec(&loaded).expect("documents re-serialize");
        let original = std::fs::read(&path).expect("fixture reads");
        let original_compact = serde_json::to_vec(
            &serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&original))
                .unwrap(),
        )
        .expect("fixture re-serializes");
        assert_eq!(
            reserialized,
            original_compact,
            "{} changed shape through the loader (§Determinism: `serde_json::to_string`, struct-declared key order)",
            path.display()
        );
    }
}

// proves: Test strategy #10 (§`bindings/v1`: "Unknown key, missing required
// key, or a value outside its bound is a refusal naming the key") — every
// `fixtures/invalid/*.json` is refused, and the message names its key.
#[test]
fn every_invalid_schema_fixture_is_refused_naming_its_key() {
    for path in fixture_paths("invalid") {
        let key = expected_key(&path);
        let outcome = common::load(&path);
        match outcome {
            Err(refusal) => {
                let message = format!("{refusal}");
                assert!(
                    message.contains(key.as_str()),
                    "{} must be refused naming `{key}`, got: {message}",
                    path.display()
                );
            }
            Ok(_) => panic!("{} must be refused", path.display()),
        }
    }
}

// proves: Test strategy #10 (§`bindings/v1`: "messages are single-line,
// control-free and truncating") — every refusal message over the invalid
// corpus obeys the message discipline.
#[test]
fn refusal_messages_are_single_line_and_control_free() {
    for path in fixture_paths("invalid") {
        let refusal = common::load(&path).expect_err("invalid fixtures refuse");
        let message = format!("{refusal}");
        assert_eq!(
            message.lines().count(),
            1,
            "refusal must be one line: {message}"
        );
        assert!(
            !message.chars().any(|c| c.is_control()),
            "refusal must be control-free: {message:?}"
        );
        assert!(
            message.len() < 1024,
            "refusal must be truncating, got {} octets",
            message.len()
        );
    }
}

// proves: Test strategy #4 (§`bindings/v1`: the loader "checks `fs::metadata`
// length against `MAX_SNAPSHOT_OCTETS` before the read") — a
// `MAX_SNAPSHOT_OCTETS + 1` document is refused on its length check.
#[test]
fn the_loader_refuses_an_over_budget_document_on_the_length_check() {
    let root = TempRoot::new("snap-over-octets");
    let mut text = String::from(
        "{\"format\":\"bindings\",\"version\":1,\"codec\":\"unreal-ini\",\"game\":\"steam/900001\",\"entries\":[],\"provenance\":{\"evidence\":\"install-tree-probe\",\"files\":[]},\"pad\":\"",
    );
    text.push_str(&"x".repeat(MAX_SNAPSHOT_OCTETS));
    text.push_str("\"}");
    assert!(text.len() > MAX_SNAPSHOT_OCTETS);
    let path = root.write("over-budget.json", &text);
    match common::load(&path) {
        Err(refusal) => {
            let message = format!("{refusal:?} {refusal}");
            assert!(
                message.contains("MAX_SNAPSHOT_OCTETS"),
                "the refusal names its bound: {message}"
            );
        }
        Ok(_) => panic!("a document over MAX_SNAPSHOT_OCTETS must be refused, never buffered"),
    }
}

// proves: Test strategy #10 (§`bindings/v1`: "the (`action`, input-token) pair
// total ≤ `MAX_SNAPSHOT_ENTRIES`, which bounds `entries` and every `inputs`
// array") — the loader refuses a document whose pair total breaches: 4097
// entries and one entry with 4097 inputs breach it as surely as each other.
#[test]
fn the_loader_refuses_a_pair_total_over_the_entry_budget() {
    let root = TempRoot::new("snap-entry-budget");

    let entries: Vec<String> = (0..MAX_SNAPSHOT_ENTRIES + 1)
        .map(|i| format!("{{\"action\":\"A{i}\",\"inputs\":[\"K\"]}}"))
        .collect();
    let many_entries = format!(
        "{{\"format\":\"bindings\",\"version\":1,\"codec\":\"unreal-ini\",\"game\":\"steam/900001\",\"entries\":[{}],\"provenance\":{{\"evidence\":\"install-tree-probe\",\"files\":[]}}}}",
        entries.join(",")
    );
    let path = root.write("many-entries.json", &many_entries);
    assert!(
        common::load(&path).is_err(),
        "MAX_SNAPSHOT_ENTRIES + 1 entries must be refused"
    );

    let inputs: Vec<String> = (0..MAX_SNAPSHOT_ENTRIES + 1)
        .map(|i| format!("\"K{i}\""))
        .collect();
    let wide_entry = format!(
        "{{\"format\":\"bindings\",\"version\":1,\"codec\":\"unreal-ini\",\"game\":\"steam/900001\",\"entries\":[{{\"action\":\"One\",\"inputs\":[{}]}}],\"provenance\":{{\"evidence\":\"install-tree-probe\",\"files\":[]}}}}",
        inputs.join(",")
    );
    let path = root.write("wide-entry.json", &wide_entry);
    match common::load(&path) {
        Err(refusal) => {
            let message = format!("{refusal:?} {refusal}");
            assert!(
                message.contains("MAX_SNAPSHOT_ENTRIES") || message.contains("entries"),
                "the breach must name its key or bound: {message}"
            );
        }
        Ok(_) => panic!("one entry with MAX_SNAPSHOT_ENTRIES + 1 inputs must be refused"),
    }
}

// proves: Test strategy #10 (§`bindings/v1`: `files` are tree-relative paths;
// §Contract-tree amendments' closed key set) — every escaping `files`
// spelling is refused naming `files`, not only the shipped `..` fixture:
// Windows drive, UNC, backslash, parent- and current-directory, and absolute
// spellings, per the accepted security finding 1 (round 1: the loader admits
// only a plain `/`-separated sequence of `Normal` components). Test-update
// item "Traversal refusals beyond the shipped fixture" in
// `03-build-notes.md`.
#[test]
fn the_loader_refuses_escaping_files_spellings_naming_files() {
    let root = TempRoot::new("snap-files-spellings");
    let spellings = [
        "C:/Windows/win.ini",    // drive prefix and a raw `:`
        r"\\server\share\x.ini", // UNC spelling, raw backslashes
        r"..\..\x.ini",          // parent directories, backslash-joined
        "../../x.ini",           // parent directories above the tree
        "./x.ini",               // a current-directory component
        "/etc/passwd",           // an absolute path component
    ];
    for (index, spelling) in spellings.iter().enumerate() {
        let doc = serde_json::json!({
            "format": "bindings",
            "version": 1,
            "codec": "unreal-ini",
            "game": "steam/900001",
            "user": "76561197960287930",
            "entries": [{ "action": "MoveForward", "inputs": ["W"] }],
            "provenance": {
                "evidence": "install-tree-probe",
                "files": [spelling],
            },
        });
        let path = root.write(&format!("escaping-{index}.json"), &doc.to_string());
        match common::load(&path) {
            Err(refusal) => {
                let message = format!("{refusal}");
                assert!(
                    message.contains("files"),
                    "`{spelling}` must be refused naming `files`, got: {message}"
                );
            }
            Ok(_) => panic!(
                "`{spelling}` is not a plain tree-relative spelling and must be refused, never loaded"
            ),
        }
    }
}

// proves: Test strategy #11 (§`bindings/v1` and §The first family cannot
// disagree) — every document the shipped family produces passes the loader.
#[test]
fn every_document_the_family_produces_passes_the_loader() {
    for (tree, appid) in [("steam-userdata", "900001"), ("proton-prefix", "900002")] {
        let fixture = common::fixture_tree(tree);
        let claim = unreal_claim(&context(appid, vec![fixture.clone()], fixture));
        let docs = UnrealIni.extract(&claim).unwrap_or_else(|refusal| {
            panic!("{tree} must extract for the loader round-trip: {refusal:?}")
        });
        for doc in &docs {
            let text = serde_json::to_vec(doc).expect("a document serializes");
            let root = TempRoot::new("snap-family-roundtrip");
            let path = root.write_bytes("family-document.json", &text);
            let reloaded = common::load(&path)
                .unwrap_or_else(|err| panic!("a family document must load: {err:?}"));
            assert_eq!(
                serde_json::to_vec(&reloaded).expect("reloads to the same bytes"),
                text,
                "the loader and the family disagree on document bytes"
            );
        }
    }
}

// proves: Test strategy #10 (§`bindings/v1` bounds) — the native-name bound the
// contract borrows from the manifest is the one the suites exercise.
#[test]
fn the_native_name_bound_is_the_manifests() {
    assert_eq!(MAX_NATIVE_NAME_OCTETS, 128);
    // The fixture named `action-over-bound` is built at that + 1; keep the
    // suites honest about which constant they mean.
    let offender = "a".repeat(MAX_NATIVE_NAME_OCTETS + 1);
    let doc = format!(
        "{{\"format\":\"bindings\",\"version\":1,\"codec\":\"unreal-ini\",\"game\":\"steam/900001\",\"entries\":[{{\"action\":\"{offender}\",\"inputs\":[\"K\"]}}],\"provenance\":{{\"evidence\":\"install-tree-probe\",\"files\":[]}}}}"
    );
    let root = TempRoot::new("snap-action-bound");
    let path = root.write("action-over-bound.gen.json", &doc);
    match common::load(&path) {
        Err(refusal) => assert!(
            format!("{refusal}").contains("action"),
            "an over-bound action must be refused naming `action`: {refusal}"
        ),
        Ok(_) => panic!("an action over MAX_NATIVE_NAME_OCTETS must be refused"),
    }
}

fn fixture_paths(kind: &str) -> Vec<std::path::PathBuf> {
    let dir = schema_fixture_dir("bindings", "v1", kind);
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|err| panic!("{}: {err}", dir.display()))
        .filter_map(|entry| {
            let path = entry.expect("readable entry").path();
            path.extension()
                .is_some_and(|ext| ext == "json")
                .then_some(path)
        })
        .collect();
    paths.sort();
    paths
}

/// The key the fixture's own file name promises the refusal will name
/// (fixtures are named `<shape>__<key>.json` per the corpus convention).
fn expected_key(path: &std::path::Path) -> String {
    let stem = path
        .file_stem()
        .expect("named fixture")
        .to_string_lossy()
        .into_owned();
    match stem.split_once("__") {
        Some((_, key)) => key.to_string(),
        None => stem
            .split('-')
            .next_back()
            .filter(|last| {
                ![
                    "over", "bound", "not", "an", "with", "no", "unknown", "missing",
                ]
                .contains(last)
            })
            .map(str::to_string)
            .unwrap_or_else(|| {
                panic!("fixture {stem} must name its key (name it `<shape>__<key>.json`)")
            }),
    }
}
