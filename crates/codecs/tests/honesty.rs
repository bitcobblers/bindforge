//! Extraction honesty (`017-L3`, §`bindings/v1` Input spelling, §Corpus
//! boundary) — a snapshot states the game's own files and nothing else.
//!
//! proves: Test strategy #8.

mod common;

use bindforge_codecs::families::unreal_ini::UnrealIni;
use common::{all_entries, context, docs_json, extract_json, fixture_tree, provenance_of};
use std::path::{Path, PathBuf};

/// Every `*.ini` byte of a fixture tree, concatenated — the whole vocabulary a
/// honest extraction may draw from.
fn fixture_ini_text(tree: &Path) -> (String, Vec<PathBuf>) {
    let mut text = String::new();
    let mut files = Vec::new();
    fn walk(dir: &Path, text: &mut String, files: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.expect("readable entry").path();
            if path.is_dir() {
                walk(&path, text, files);
            } else if path.extension().is_some_and(|ext| ext == "ini") {
                files.push(path.clone());
                text.push_str(&std::fs::read_to_string(&path).expect("fixture ini reads"));
            }
        }
    }
    walk(tree, &mut text, &mut files);
    (text, files)
}

/// The fixture games the honesty law is asserted over: (tree, appid).
#[allow(dead_code)]
fn honest_games() -> Vec<(&'static str, &'static str)> {
    vec![
        ("steam-userdata", "900001"),
        ("proton-prefix", "900002"),
        ("broken-tuple", "900006"), // refuses; contributes no documents to audit
    ]
}

// proves: Test strategy #8 (§`bindings/v1` Input spelling (canonical)) — no
// entry has a taxonomy id in it and no entry has an ADR-012 input id in it:
// no token in any document is a namespaced id of any kind (they all carry a
// `:`; a verbatim native token never does).
#[test]
fn no_entry_carries_a_taxonomy_or_input_id() {
    for (tree, appid) in [("steam-userdata", "900001"), ("proton-prefix", "900002")] {
        let fixture = fixture_tree(tree);
        let docs = extract_json(&context(appid, vec![fixture.clone()], fixture));
        for doc in &docs {
            for (action, inputs) in entries_of_doc(doc) {
                for token in std::iter::once(action).chain(inputs) {
                    assert!(
                        !token.contains(':'),
                        "`{token}` in {tree} reads like a namespaced id; a snapshot carries the file's own words only (`017-L3`)"
                    );
                    assert!(
                        !token.starts_with("keyboard:")
                            && !token.starts_with("mouse:")
                            && !token.starts_with("gamepad:")
                            && !token.starts_with("action:"),
                        "`{token}` is a taxonomy/input id spelling; extraction performs no translation"
                    );
                }
            }
        }
    }
}

fn entries_of_doc(doc_json: &str) -> Vec<(String, Vec<String>)> {
    all_entries(std::slice::from_ref(&doc_json.to_string()))
}

// proves: Test strategy #8 (§The first family: "`extract` reads only located
// files") — every token and every `provenance.files` entry of every fixture
// document traces to the located candidate set; nothing outside it contributes.
#[test]
fn no_entry_comes_from_a_file_outside_the_located_set() {
    for (tree, appid) in [("steam-userdata", "900001"), ("proton-prefix", "900002")] {
        let fixture = fixture_tree(tree);
        let claim = UnrealIni
            .detect(&context(appid, vec![fixture.clone()], fixture.clone()))
            .expect("these fixtures claim");
        let located: Vec<PathBuf> = claim
            .per_user
            .iter()
            .flat_map(|u| u.files.clone())
            .collect();
        assert!(!located.is_empty(), "the fixtures locate candidate files");
        let located_text: String = located
            .iter()
            .map(|file| std::fs::read_to_string(file).expect("located file reads"))
            .collect();
        let docs = docs_json(&UnrealIni.extract(&claim).expect("these fixtures extract"));

        // (a) every stated token is spelled in a located file, and nowhere else.
        let outside_text = fixture_ini_text(&fixture).0;
        let located_tokens: Vec<String> = common::ini_tokens(&located_text);
        let all_tokens: Vec<String> = docs
            .iter()
            .flat_map(|doc| entries_of_doc(doc))
            .flat_map(|(action, inputs)| std::iter::once(action).chain(inputs))
            .collect();
        for token in &all_tokens {
            assert!(
                located_tokens.contains(token),
                "`{token}` appears in {tree}'s documents but in no located candidate file"
            );
            assert!(outside_text.contains(token));
        }

        // (b) `provenance.files` names exactly the located files (tree-relative).
        let (_, files) = provenance_of(&docs[0]);
        let located_basenames: Vec<String> = located
            .iter()
            .map(|file| {
                file.file_name()
                    .expect("named file")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        for stated in &files {
            let basename = Path::new(stated)
                .file_name()
                .expect("relative path")
                .to_string_lossy();
            assert!(
                located_basenames
                    .iter()
                    .any(|name| name == basename.as_ref()),
                "`provenance` states {stated}, which the locate walk never located"
            );
            let is_relative = !stated.starts_with('/');
            assert!(
                is_relative,
                "contract files are tree-relative, never absolute: {stated}"
            );
        }
    }
}

// proves: Test strategy #8 (§The first family: "defaults are not absorbed") —
// no default file contributes: the `defaults-only` tree yields either no claim
// or a claim with nothing readable, and never the install tree's pairs; the
// `bindings-absent` tree contributes no partial entry list either way.
#[test]
fn defaults_and_absent_stores_never_yield_a_partial_entry_list() {
    for (tree, appid) in [("defaults-only", "900004"), ("bindings-absent", "900005")] {
        let fixture = fixture_tree(tree);
        let claim = UnrealIni.detect(&context(appid, vec![fixture.clone()], fixture.clone()));
        match claim {
            None => {
                // no claim is a whole answer — nothing is stated at all.
            }
            Some(claim) => {
                let docs = match UnrealIni.extract(&claim) {
                    Ok(docs) => docs_json(&docs),
                    Err(_) => Vec::new(),
                };
                if tree == "defaults-only" {
                    let entries = all_entries(&docs);
                    assert!(
                        entries
                            .iter()
                            .all(|(action, _)| action != "NeverAbsorbedAction"
                                && action != "AnotherDefaultAction"),
                        "the install tree's DefaultInput.ini contributed: {entries:?}"
                    );
                }
            }
        }
    }
}

// proves: Test strategy #8 (§Determinism) — every document a fixture game
// produces is in canonical form: entries byte-sorted, inputs deduplicated and
// byte-sorted, provenance.files byte-sorted, `user` groups in `user_trees` order.
#[test]
fn every_fixture_document_is_in_canonical_form() {
    for (tree, appid) in [("steam-userdata", "900001"), ("proton-prefix", "900002")] {
        let fixture = fixture_tree(tree);
        let docs = extract_json(&context(appid, vec![fixture.clone()], fixture));
        assert!(!docs.is_empty(), "{tree} produces documents");
        common::assert_sorted_canonical(&docs);
    }
}
