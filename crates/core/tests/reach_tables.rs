//! The reach-table loader: `crates/core`'s reach-class tables come from the
//! `schemas/` contract tree, not from Rust literals (ADR-008 clause 3; the
//! member table's "reach-table loader" row for `bindforge-core`).
//!
//! proves: Test strategy #4 — tables load from `schemas/reach-classes/v1/`
//! fixture data into the in-memory model with per-hand (left/right) tables kept
//! distinct.
//!
//! The table *content* is ADR-009's (`spec.md`: the shipped class ids are
//! placeholders pending that taxonomy), so these tests pin the load contract:
//! each hand's table comes from its own file, filed under the hand its file name
//! and its `hand` entry agree on, and the two hands are never one table. The
//! exact ids below are the shipped v1 table data — if ADR-009 retunes them, the
//! expectation moves with the files; what must never move is that the two hands
//! hold different classes.
//!
//! The loader is pointed at the version directory, whose table data sits
//! directly in it; that directory's `fixtures/valid` + `fixtures/invalid` pair
//! is the conformance harness's corpus (`conformance.rs`, Test strategy #1),
//! asserted there by parse and by load — a stand-alone corpus fixture carries no
//! file name of its own, so the file-name/hand agreement rule is not part of the
//! fixture assertion but part of the loader contract asserted here
//! (`schemas/reach-classes/v1/spec.md`). Reaching the tree through
//! `env!("CARGO_MANIFEST_DIR")` is what makes it load identically for a
//! contributor and in the gate (A8, A26).

mod common;

use std::path::PathBuf;

use bindforge_core::{CoreError, Hand, ReachTables};
use common::{ScratchDir, fixture_text, schemas_root};

fn version_tree() -> PathBuf {
    schemas_root().join("reach-classes").join("v1")
}

fn load() -> ReachTables {
    let dir = version_tree();
    bindforge_core::load_reach_tables(&dir).unwrap_or_else(|err| {
        panic!("the reach-class tables under {} must load: {err}", dir.display())
    })
}

/// A `hand = "right"` table document: a mismatched `left.toml` in the conflict
/// case below, and a document body to pad in the budget case.
const TABLE_DECLARING_RIGHT: &str = r#"format = "reach-classes"
version = "v1"
hand = "right"
classes = ["near"]
"#;

/// A `hand = "left"` table document: a mismatched `right.toml` in the conflict
/// case, and a correctly named `left.toml` in the budget case.
const TABLE_DECLARING_LEFT: &str = r#"format = "reach-classes"
version = "v1"
hand = "left"
classes = ["far"]
"#;

/// proves: Test strategy #4 — the v1 reach data loads from the contract tree
/// into the in-memory model, and both hands of the reach model are present in
/// it.
#[test]
fn reach_class_tables_load_from_the_contract_tree_into_the_model() {
    let tables = load();

    let left = tables
        .hand(Hand::Left)
        .unwrap_or_else(|| panic!("the v1 reach data declares a left-hand table"));
    let right = tables
        .hand(Hand::Right)
        .unwrap_or_else(|| panic!("the v1 reach data declares a right-hand table"));

    assert!(
        !left.classes().is_empty(),
        "the left-hand table must carry its classes, not an empty placeholder"
    );
    assert!(
        !right.classes().is_empty(),
        "the right-hand table must carry its classes, not an empty placeholder"
    );
}

/// proves: Test strategy #4 — the per-hand tables stay distinct in the model:
/// each table knows the hand it belongs to, and each hand holds exactly the
/// classes of its own file, in table order, so the loader cannot answer both
/// hands out of one table. A pointer check would prove nothing here — two
/// entries of the same `Vec` always have distinct addresses — so the assertion
/// is on the loaded class ids.
#[test]
fn the_loader_keeps_the_left_and_right_hand_tables_distinct() {
    let tables = load();
    let left = tables
        .hand(Hand::Left)
        .expect("the v1 reach data declares a left-hand table");
    let right = tables
        .hand(Hand::Right)
        .expect("the v1 reach data declares a right-hand table");

    assert_eq!(
        left.hand(),
        Hand::Left,
        "the table filed under the left hand must report that hand"
    );
    assert_eq!(
        right.hand(),
        Hand::Right,
        "the table filed under the right hand must report that hand"
    );

    let left_ids: Vec<&str> = left.classes().iter().map(|class| class.id.as_str()).collect();
    let right_ids: Vec<&str> = right.classes().iter().map(|class| class.id.as_str()).collect();
    assert_eq!(
        left_ids,
        ["near", "mid", "extended"],
        "the left-hand table must hold exactly the classes `left.toml` lists, in table order"
    );
    assert_eq!(
        right_ids,
        ["near", "mid", "cross"],
        "the right-hand table must hold exactly the classes `right.toml` lists, in table order"
    );
    assert_ne!(
        left_ids, right_ids,
        "the loader must keep a separate table per hand; {} holds one table for both hands",
        version_tree().display()
    );
}

/// proves: Test strategy #4 + Formats contract → reach-classes "Table data"
/// (loader contract) — which file a hand's table may come from, decided without
/// a filesystem: the hand matches its own `<hand>.toml` stem and nothing else.
/// The conflict case above can only be a real refusal if the matching itself is
/// exact: a match that also accepted the other hand's stem, or a stem that
/// differs only in case, would let one hand be filed from two files — and on a
/// case-insensitive filesystem the two names are the same file.
#[test]
fn a_hand_matches_only_its_own_lowercase_table_file_stem() {
    for (hand, stem, expected) in [
        (Hand::Left, "left", true),
        (Hand::Left, "right", false),
        (Hand::Left, "Left", false),
        (Hand::Right, "right", true),
        (Hand::Right, "left", false),
        (Hand::Right, "Right", false),
    ] {
        let matched = bindforge_core::hand_matches_file_name(hand, stem);
        let want = if expected { "" } else { "not " };
        assert_eq!(
            matched, expected,
            "`{stem}.toml` must {want}be a table file for one hand only"
        );
    }
}

/// proves: Formats contract → reach-classes table data (loader contract) — a
/// `<hand>.toml` whose `hand` entry names the other hand is refused: the table
/// is filed under the hand its file name and its entry agree on, so a table
/// cannot migrate between hands by editing one line. The refusal names the file
/// and keeps the local filesystem path out of the message, since rejection text
/// is what a hub or CLI shows to whoever supplied the document
/// (`schemas/README.md`).
#[test]
fn the_loader_refuses_a_table_file_whose_hand_entry_contradicts_its_file_name() {
    let scratch = ScratchDir::new("reach-hand-conflict");
    scratch.write("left.toml", TABLE_DECLARING_RIGHT);
    scratch.write("right.toml", TABLE_DECLARING_LEFT);

    let err = bindforge_core::load_reach_tables(scratch.path())
        .err()
        .expect("a table whose `hand` entry contradicts its file name must be refused");
    let detail = err.to_string();
    let path_text = scratch.path().display().to_string();

    assert!(
        detail.contains("left.toml") || detail.contains("right.toml"),
        "the refusal must name the table file it refused, got `{detail}`"
    );
    assert!(
        !detail.contains(path_text.as_str()),
        "a rejection shown to whoever sent the file must not disclose a local path: {detail}"
    );
}

/// proves: Formats contract → reach-classes "Table data" — reading a table file
/// is subject to the same input budget as every other document, so an oversized
/// file in a served or synced tree is refused rather than loaded into memory.
#[test]
fn the_loader_refuses_a_table_file_larger_than_the_parse_budget() {
    let budget = usize::try_from(bindforge_core::MAX_DOCUMENT_BYTES)
        .expect("MAX_DOCUMENT_BYTES is a byte count, so it fits in a usize");
    let mut oversized = String::from(TABLE_DECLARING_LEFT);
    oversized.push_str(&"#".repeat(budget + 1 - oversized.len()));

    let scratch = ScratchDir::new("reach-table-over-budget");
    let shipped_right = version_tree().join("right.toml");
    scratch.write("left.toml", &oversized);
    scratch.write("right.toml", &fixture_text(&shipped_right));

    let err = bindforge_core::load_reach_tables(scratch.path())
        .err()
        .expect("a table file larger than MAX_DOCUMENT_BYTES must be refused, not loaded");
    assert!(
        matches!(err, CoreError::DocumentTooLarge { .. }),
        "expected the `DocumentTooLarge` refusal for an oversized table file, got {err:?}"
    );
}
