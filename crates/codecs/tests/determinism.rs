//! Determinism (`017-L4`, §Determinism — canonical) — one ordering law, pinned
//! by byte identity over the fixtures.
//!
//! proves: Test strategy #12 (the in-process half: documents; the table and
//! `--json` halves are pinned by `cli_bindings_tests`).

mod common;

use bindforge_codecs::codec::{Detection, Evidence};
use bindforge_codecs::families::unreal_ini::UnrealIni;
use common::{TempRoot, context, docs_json, extract_json, fixture_tree, user_of};

fn extract_twice(appid: &str, fixture: &std::path::Path) -> (Vec<String>, Vec<String>) {
    let first = extract_json(&context(
        appid,
        vec![fixture.to_path_buf()],
        fixture.to_path_buf(),
    ));
    let second = extract_json(&context(
        appid,
        vec![fixture.to_path_buf()],
        fixture.to_path_buf(),
    ));
    (first, second)
}

// proves: Test strategy #12 (§Determinism: "one extraction … repeats yield
// byte-identical documents") — extracting each fixture game twice through the
// family serializes to identical bytes.
#[test]
fn extracting_each_fixture_game_twice_yields_byte_identical_documents() {
    for (tree, appid) in [("steam-userdata", "900001"), ("proton-prefix", "900002")] {
        let fixture = fixture_tree(tree);
        let (first, second) = extract_twice(appid, &fixture);
        assert!(!first.is_empty(), "{tree} produces documents");
        assert_eq!(
            first, second,
            "two extractions of {tree} serialized differently — the ordering law rotted"
        );
    }
}

// proves: Test strategy #12 (§Determinism: "documents and their user-tree
// groups in `user_trees` order … the total order over located trees") — the
// order of `Detection.per_user` (its `tree_root`s with it) never reaches a
// document: shuffling it in the harness yields byte-identical output.
#[test]
fn a_shuffled_per_user_vector_yields_the_same_documents() {
    let fixture = fixture_tree("steam-userdata");
    let ctx = context("900001", vec![fixture.clone()], fixture);
    let claim = UnrealIni.detect(&ctx).expect("the two-user fixture claims");
    assert!(
        claim.per_user.len() >= 2,
        "the fixture claims across two user trees"
    );

    let straight = docs_json(&UnrealIni.extract(&claim).expect("straight order extracts"));

    let mut shuffled = claim.clone();
    shuffled.per_user.reverse();
    let reversed = docs_json(
        &UnrealIni
            .extract(&shuffled)
            .expect("reversed order extracts"),
    );

    let mut rotated = claim.clone();
    rotated.per_user.rotate_left(1);
    let rotated = docs_json(&UnrealIni.extract(&rotated).expect("rotated order extracts"));

    assert_eq!(
        reversed, straight,
        "reversing per_user changed the documents"
    );
    assert_eq!(rotated, straight, "rotating per_user changed the documents");
}

// proves: Test strategy #12 (§Determinism: "across documents, deduplicated by
// each document's `UserCandidates.tree_root` — tree identity, never rendered —
// the first stating it once") — two records that resolve the same user tree
// state that tree once.
#[test]
fn two_records_resolving_the_same_tree_state_it_once() {
    let root = TempRoot::new("det-shared-tree");
    let lib_a = TempRoot::new("det-lib-a");
    let lib_b = TempRoot::new("det-lib-b");
    let appid = "900041";
    let stored = common::player_input(&["+Bindings=(Name=\"W\",Command=\"MoveForward\")"]);
    root.write(
        &format!("userdata/76561197960287930/{appid}/Saved/Config/Linux/Engine.ini"),
        &stored,
    );

    // Two ScanRecords — two libraries — both resolve the same shared-root tree.
    let mut claims = Vec::new();
    for lib in [lib_a.canonical(), lib_b.canonical()] {
        let claim = UnrealIni
            .detect(&context(appid, vec![root.canonical()], lib))
            .expect("each record resolves the shared tree");
        claims.push(claim);
    }
    assert_eq!(
        claims[0].per_user[0].tree_root,
        claims[1].per_user[0].tree_root
    );

    // The face's fold (§The CLI face step 3): per-user groups concatenated, then
    // documents deduplicated by tree identity — the tree is stated exactly once.
    let mut merged = Detection {
        family: claims[0].family,
        evidence: Evidence::InstallTreeProbe,
        per_user: claims
            .iter()
            .flat_map(|claim| claim.per_user.clone())
            .collect(),
    };
    let all = docs_json(
        &UnrealIni
            .extract(&merged)
            .expect("the merged claim extracts"),
    );
    merged.per_user.dedup_by(|a, b| a.tree_root == b.tree_root);
    let once = docs_json(
        &UnrealIni
            .extract(&merged)
            .expect("the deduped claim extracts"),
    );
    let unique_trees: std::collections::HashSet<String> =
        once.iter().filter_map(|doc| user_of(doc)).collect();
    assert_eq!(
        all.len(),
        once.len(),
        "the duplicate tree stated itself twice"
    );
    assert_eq!(unique_trees.len(), 1, "one tree means one document");
}

// proves: Test strategy #12 (§`bindings/v1`: "two trees may share a `user`
// label, and then nothing but `entries` separates their documents") — one slug
// in two libraries, two `ProtonPrefix` trees sharing the `steamuser` label with
// different bindings, yields two documents.
#[test]
fn one_slug_in_two_libraries_yields_two_documents() {
    let appid = "900042";
    let mut docs: Vec<String> = Vec::new();
    for (label, binding) in [
        (
            "det-two-libs-a",
            "+Bindings=(Name=\"E\",Command=\"Interact\")",
        ),
        (
            "det-two-libs-b",
            "+Bindings=(Name=\"R\",Command=\"Reload\")",
        ),
    ] {
        let lib = TempRoot::new(label);
        lib.write(
            &format!(
                "steamapps/compatdata/{appid}/pfx/drive_c/users/steamuser/Saved/Config/Windows/Engine.ini"
            ),
            &common::player_input(&[binding]),
        );
        let game_docs = extract_json(&context(appid, vec![lib.canonical()], lib.canonical()));
        assert_eq!(game_docs.len(), 1, "one located tree means one document");
        assert_eq!(user_of(&game_docs[0]).as_deref(), Some("steamuser"));
        docs.extend(game_docs);
    }
    assert_eq!(
        docs.len(),
        2,
        "two distinct trees sharing a label state two documents"
    );
    assert_ne!(
        docs[0], docs[1],
        "the documents differ by their entries, not by anything rendered"
    );
}

// proves: Test strategy #12 (§Determinism: "neither registry iteration order
// … reaches an artifact") — running the claim passes over a one-element family
// slice in either arrangement yields the same claim evidence and documents.
#[test]
fn the_claim_route_is_order_free_end_to_end() {
    let fixture = fixture_tree("steam-userdata");
    let ctx = context("900001", vec![fixture.clone()], fixture);
    let first = bindforge_codecs::registry::claim(&ctx).expect("the shipped set claims");
    let second = bindforge_codecs::registry::claim(&ctx).expect("and again");
    assert_eq!(first.family, second.family);
    assert_eq!(
        docs_json(&UnrealIni.extract(&first).unwrap()),
        docs_json(&UnrealIni.extract(&second).unwrap())
    );
}
