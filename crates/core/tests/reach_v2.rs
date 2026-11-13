//! The `reach-classes/v2` entry set, the `keys` pair encoding, and the shipped
//! taxonomy in `crates/core`: what the loader accepts for a v2 table, what it
//! refuses, and what the two shipped hand tables say.
//!
//! proves: Test strategy #2 — the v2 entry set and the pair encoding: a document
//! with exactly the five defined entries parses; `keys` on a v1 document is
//! refused naming the entry; a v2 document without `keys` is `MissingEntry`; a
//! pair that is not exactly two space-separated fields, a non-string element, a
//! class reference outside the document's `classes`, a repeated input id, and a
//! `classes` array outside the closed taxonomy — wrong order, a duplicated id, an
//! id the taxonomy does not carry — are refused, naming the entry or id; and a
//! `keys` pair naming `mouse:left` is refused, because grades are `keyboard:*`
//! only.
//!
//! proves: Test strategy #3 — the shipped tables' invariants: `load_reach_tables`
//! on `schemas/reach-classes/v2/` returns two distinct hand tables; both hands
//! grade exactly the same 48 input ids, each once; both `classes` arrays are
//! exactly `home`, `thumb`, `stretch`, `far`, `off-reach` in that order;
//! `keyboard:;` grades `home` right / `off-reach` left and `keyboard:'` grades
//! `stretch` right / `off-reach` left; `keyboard:space` grades `thumb` on both;
//! and a document whose declared version differs from its loading directory's
//! segment is refused, v1 directories included.
//!
//! The severity order is the shared `classes` array order, worst last, and it is
//! *shared*: every v2 document lists exactly the five taxonomy ids, once each, in
//! that order, "so two hands cannot define two scales" (Options considered). That
//! makes the canonical list a `parse`-level rule, which is why these cases go
//! through `parse` rather than through a loader comparing files. v1 stays closed
//! forever (A1), so `reach-classes/v1/fixtures/invalid/keys-entry.toml` is the
//! corpus pin for that refusal, and
//! `reach-classes/v2/fixtures/invalid/non-ascii-member.toml` is the corpus pin for
//! the closed member set being written in ASCII.
//!
//! Unit-level: `parse` over document text, plus scratch directories for the load
//! rules — no network (ADR-004, repo practice). The corpus walk that runs these
//! fixtures for every format is `conformance.rs` (Test strategy #6).

mod common;

use std::collections::BTreeSet;

use bindforge_core::{CoreError, Hand, InputId, InputNamespace, ReachTable, ReachTables};
use common::{ScratchDir, fixture_text, quoted_array, schemas_root, version_dir};

/// The taxonomy's five ids in the severity order this design pins — worst last —
/// which every v2 document must repeat exactly (A9).
const TAXONOMY: [&str; 5] = ["home", "thumb", "stretch", "far", "off-reach"];

/// The 48 graded inputs of the closed vocabulary (A10): 26 letters, 10 digits,
/// the 11 ANSI-US punctuation glyphs, and the reserved word `space`.
const GRADED_IDS: [&str; 48] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "`", "-",
    "=", "[", "]", "\\", ";", "'", ",", ".", "/", "space",
];

/// The `classes` line every valid v2 document repeats.
const CANONICAL_CLASSES: &str =
    "classes = [\"home\", \"thumb\", \"stretch\", \"far\", \"off-reach\"]";

/// A v2 table for the given hand whose `keys` array is exactly these pairs.
fn v2_table(hand: &str, keys: &[&str]) -> String {
    format!(
        "format = \"reach-classes\"\nversion = \"v2\"\nhand = \"{hand}\"\n{CANONICAL_CLASSES}\nkeys \
         = [{}]\n",
        quoted_join(keys)
    )
}

/// A v1 table — the shape ADR-008 shipped and ADR-012 leaves untouched — with any
/// extra entry lines appended, so the `keys`-in-a-v1-document case spells out the
/// entry it adds.
fn v1_table(hand: &str, extra_entries: &[&str]) -> String {
    let mut text = format!(
        "format = \"reach-classes\"\nversion = \"v1\"\nhand = \"{hand}\"\nclasses = [\"near\"]\n"
    );
    for line in extra_entries {
        text.push_str(line);
        text.push('\n');
    }
    text
}

/// The `keys` array's elements as quoted one-line entries.
fn quoted_join(pairs: &[&str]) -> String {
    pairs
        .iter()
        .map(|pair| format!("\"{pair}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The `keys` entry of a shipped table document, read textually the way `common`
/// does for its other contract helpers: this names what the contract file lists,
/// it is not a second parser. The `\\` a TOML string carries for the backslash
/// glyph is turned back into the one glyph the grammar means.
fn shipped_keys(text: &str, label: &str) -> Vec<(String, String)> {
    let pairs = quoted_array(text, "keys").unwrap_or_else(|| {
        panic!(
            "{label}: the `keys` entry must be one one-line string array — the only shape the \
             documented grammar subset admits, a multi-line array being itself a refusal"
        )
    });
    pairs
        .iter()
        .map(|pair| {
            let unescaped = pair.replace("\\\\", "\\");
            let mut fields = unescaped.split(' ');
            let input = fields
                .next()
                .unwrap_or_else(|| panic!("{label}: a `keys` pair names an input id"));
            let class = fields
                .next()
                .unwrap_or_else(|| panic!("{label}: a `keys` pair names a class id"));
            (input.to_string(), class.to_string())
        })
        .collect()
}

/// The shipped v2 tables, loaded from the contract tree.
fn load_shipped_v2() -> ReachTables {
    let dir = schemas_root().join("reach-classes").join("v2");
    bindforge_core::load_reach_tables(&dir).unwrap_or_else(|err| {
        panic!(
            "the shipped v2 tables under {} must load: {err}",
            dir.display()
        )
    })
}

/// A loaded table's class ids, in table order.
fn class_ids(table: &ReachTable) -> Vec<&str> {
    table
        .classes()
        .iter()
        .map(|class| class.id.as_str())
        .collect()
}

/// A graded input's class id, when the table grades it.
fn grade_id(table: &ReachTable, id: &InputId) -> Option<String> {
    table.grade(id).map(|class| class.id.clone())
}

/// The shape a version/segment refusal keeps: it names a version, escaped and
/// truncated like an echoed id (A11), and it stays one line.
fn assert_version_refusal(err: &CoreError) {
    let detail = err.to_string();
    assert!(
        !detail.is_empty(),
        "a version/segment refusal must say what is wrong"
    );
    assert!(
        detail.chars().all(|ch| !ch.is_control()) && !detail.contains('\n'),
        "the echoed version takes the same printable-escape path as an echoed id (A11), got \
         {detail:?}"
    );
    assert!(
        detail.contains("v1") || detail.contains("v2"),
        "the refusal must name the version it is about, got `{detail}`"
    );
}

/// proves: Test strategy #2 — a v2 table carrying exactly the five defined
/// entries (`format`, `version`, `hand`, `classes`, `keys`) parses: closing the
/// entry set is not the same as shrinking the grammar.
#[test]
fn parses_a_v2_table_with_exactly_the_five_defined_entries() {
    let document = v2_table(
        "left",
        &["keyboard:a home", "keyboard:d home", "keyboard:space thumb"],
    );
    if let Err(err) = bindforge_core::parse(&document) {
        panic!(
            "a v2 table using only the entries the entry table defines must parse; it was refused: \
             {err}"
        );
    }
}

/// proves: Test strategy #2 + A1 — `keys` is not part of the v1 entry set, which
/// stays closed forever, so a v1 document carrying it is refused naming the entry
/// rather than read as a widened v1.
#[test]
fn refuses_a_keys_entry_on_a_v1_document_naming_the_entry() {
    let document = v1_table(
        "left",
        &[&format!("keys = [{}]", quoted_join(&["keyboard:a home"]))],
    );
    let Err(err) = bindforge_core::parse(&document) else {
        panic!("a v1 reach-classes table carrying `keys` must be refused, not read as a v1 table");
    };

    let detail = err.to_string();
    assert!(
        detail.contains("keys"),
        "the refusal must name the entry it refused, got `{detail}`"
    );
}

/// proves: Test strategy #2 + #6 — that refusal is part of the language-neutral
/// contract, not only of `bindforge-core`: `schemas/reach-classes/v1` ships the
/// fixture that pins it, and the harness runs it for every implementer.
#[test]
fn the_v1_corpus_ships_the_fixture_that_refuses_a_keys_entry() {
    let path = version_dir("reach-classes", "v1")
        .path
        .join("fixtures")
        .join("invalid")
        .join("keys-entry.toml");

    assert!(
        path.is_file(),
        "reach-classes/v1 must ship `fixtures/invalid/keys-entry.toml`: v1's entry set is closed \
         forever and the invalid-fixture corpus is the contract for that refusal (missing at {})",
        path.display()
    );
    let text = fixture_text(&path);
    let Err(err) = bindforge_core::parse(&text) else {
        panic!(
            "{} is an invalid fixture and must be refused",
            path.display()
        );
    };
    let detail = err.to_string();
    assert!(
        detail.contains("keys"),
        "{}`s refusal must name the entry it refused, got `{detail}`",
        path.display()
    );
}

/// proves: Test strategy #2 — the corpus carries the ASCII half of the member
/// grammar too: `reach-classes/v2/fixtures/invalid/non-ascii-member.toml` grades
/// `keyboard:é`, a member that reads as lowercase only to a Unicode-aware check,
/// so every implementer refuses it naming the member rather than folding it onto a
/// glyph the set does carry.
#[test]
fn the_v2_corpus_ships_the_fixture_that_refuses_a_non_ascii_member() {
    let path = version_dir("reach-classes", "v2")
        .path
        .join("fixtures")
        .join("invalid")
        .join("non-ascii-member.toml");

    assert!(
        path.is_file(),
        "reach-classes/v2 must ship `fixtures/invalid/non-ascii-member.toml`: the closed member \
         set is written `{{a-z}}` and `{{0-9}}`, and the invalid-fixture corpus is the contract \
         for that refusal (missing at {})",
        path.display()
    );
    let Err(err) = bindforge_core::parse(&fixture_text(&path)) else {
        panic!(
            "{} is an invalid fixture and must be refused",
            path.display()
        );
    };
    let detail = err.to_string();
    assert!(
        detail.contains("é"),
        "the refusal must name the member it refused, got `{detail}`"
    );
}

/// proves: Test strategy #2 — `keys` is required in v2: the mapping is the whole
/// reason the major exists, so a v2 table without it is `MissingEntry`, never a
/// table with an empty mapping.
#[test]
fn refuses_a_v2_document_without_keys_as_a_missing_entry() {
    let document = format!(
        "format = \"reach-classes\"\nversion = \"v2\"\nhand = \"left\"\n{CANONICAL_CLASSES}\n"
    );
    let err = bindforge_core::parse(&document).expect_err(
        "a v2 table with no `keys` entry must be refused, not read as an empty mapping",
    );

    assert!(
        matches!(err, CoreError::MissingEntry { .. }),
        "an absent required entry is the contract's `MissingEntry` refusal, got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains("keys"),
        "the refusal must name the entry that is missing, got `{detail}`"
    );
}

/// proves: Test strategy #2 — a `keys` element is exactly one
/// `"<input-id> <class-id>"` pair: zero, one, or three space-separated fields are
/// refused. This is the encoding the closed member grammars exist to make
/// unambiguous (Options considered: "ids forbid spaces"), and
/// `pair-three-fields.toml` is the corpus pin.
#[test]
fn refuses_a_keys_pair_that_is_not_exactly_two_space_separated_fields() {
    for pair in [
        "",
        "keyboard:a",
        "keyboard:a home extra",
        "home keyboard:a",
        "keyboard:a home keyboard:d",
    ] {
        let document = v2_table("left", &[pair]);
        let Err(err) = bindforge_core::parse(&document) else {
            panic!("the `keys` pair `{pair}` is not exactly two space-separated fields");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a malformed pair is refused as `InvalidValue`, got {err:?}"
        );
    }
}

/// proves: Test strategy #2 — every `keys` element is a string: a number, a
/// boolean, or a nested array inside the array is refused, because the documented
/// grammar subset admits one-line string arrays only (`non-string-element.toml`
/// is the corpus pin).
#[test]
fn refuses_a_non_string_element_in_the_keys_array() {
    for keys in [
        "keys = [\"keyboard:a home\", 7]",
        "keys = [\"keyboard:a home\", true]",
        "keys = [[]]",
    ] {
        let document = format!(
            "format = \"reach-classes\"\nversion = \"v2\"\nhand = \"left\"\n{CANONICAL_CLASSES}\n\
             {keys}\n"
        );
        if bindforge_core::parse(&document).is_ok() {
            panic!("a `keys` array holding a non-string element must be refused: {keys}");
        }
    }
}

/// proves: Test strategy #2 — a pair's class id must be one the same document
/// lists in its own `classes`: `near` is a v1 placeholder the v2 taxonomy does not
/// carry, so naming it is a refusal that names the id, never a class that grades
/// nothing (`unknown-class-reference.toml` is the corpus pin).
#[test]
fn refuses_a_keys_pair_naming_a_class_the_document_does_not_list() {
    let document = v2_table("left", &["keyboard:a home", "keyboard:d near"]);
    let Err(err) = bindforge_core::parse(&document) else {
        panic!(
            "a `keys` pair naming the class `near`, which the document's `classes` does not list, \
             must be refused"
        );
    };

    let detail = err.to_string();
    assert!(
        detail.contains("near"),
        "the refusal must name the class id it refused, got `{detail}`"
    );
}

/// proves: Test strategy #2 — each input id appears once per table: one key cannot
/// carry two grades for the same hand, so a repeated id is refused
/// (`duplicate-input.toml` is the corpus pin).
#[test]
fn refuses_an_input_id_repeated_in_the_keys_array() {
    let document = v2_table("left", &["keyboard:a home", "keyboard:a stretch"]);
    if bindforge_core::parse(&document).is_ok() {
        panic!(
            "`keyboard:a` appears twice in one hand's `keys`; a repeated input id must be refused"
        );
    }
}

/// proves: Test strategy #2 — `classes` is exactly the five taxonomy ids, once
/// each, in the severity order this design pins: a reordered list, a duplicated
/// id, an id the taxonomy does not carry, or a short list is refused. Two hands
/// defining two scales is the failure the shared list exists to prevent, and rank
/// arithmetic over that order is what the placement bound reads
/// (`class-order-mismatch.toml` and `unknown-class-id.toml` are the corpus pins).
#[test]
fn refuses_a_classes_array_outside_the_closed_taxonomy() {
    for (classes, fault) in [
        (
            "classes = [\"thumb\", \"home\", \"stretch\", \"far\", \"off-reach\"]",
            "the five ids out of the pinned severity order",
        ),
        (
            "classes = [\"home\", \"home\", \"thumb\", \"stretch\", \"far\", \"off-reach\"]",
            "a duplicated taxonomy id",
        ),
        (
            "classes = [\"home\", \"thumb\", \"stretch\", \"far\", \"unreachable\"]",
            "an id the taxonomy does not carry",
        ),
        (
            "classes = [\"home\", \"thumb\", \"stretch\"]",
            "fewer than the five taxonomy ids",
        ),
    ] {
        let document = format!(
            "format = \"reach-classes\"\nversion = \"v2\"\nhand = \"left\"\n{classes}\nkeys = \
             [\"keyboard:a home\"]\n"
        );
        let Err(err) = bindforge_core::parse(&document) else {
            panic!("`classes` holding {fault} must be refused");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a `classes` array outside the closed taxonomy ({fault}) is refused as `InvalidValue`, \
             got {err:?}"
        );
    }
}

/// proves: Test strategy #2 + Decision Three — reach grades attach to `keyboard:*`
/// only, so a pair naming a `mouse:*` or `gamepad:*` id is refused naming the id:
/// the vocabulary names those inputs without grading them
/// (`mouse-input.toml` is the corpus pin).
#[test]
fn refuses_a_keys_pair_grading_a_non_keyboard_input() {
    for pair in [
        "mouse:left home",
        "gamepad:a home",
        "mouse:wheel-up stretch",
    ] {
        let document = v2_table("left", &["keyboard:a home", pair]);
        let Err(err) = bindforge_core::parse(&document) else {
            panic!("`{pair}` grades a non-keyboard input, which Decision Three refuses");
        };
        let detail = err.to_string();
        let id = pair.split(' ').next().unwrap_or(pair);
        assert!(
            detail.contains(id),
            "the refusal must name the id it refused (`{id}`), got `{detail}`"
        );
    }
}

/// proves: Test strategy #3 — the shipped v2 tables load as two hand tables that
/// stay distinct: each reports its own hand, and the same key grades differently
/// for the two, so the loader cannot answer both hands out of one table.
#[test]
fn the_shipped_v2_tables_load_as_two_distinct_hand_tables() {
    let dir = schemas_root().join("reach-classes").join("v2");
    let tables = bindforge_core::load_reach_tables(&dir).unwrap_or_else(|err| {
        panic!(
            "the shipped v2 tables under {} must load: {err}",
            dir.display()
        )
    });

    let left = tables
        .hand(Hand::Left)
        .unwrap_or_else(|| panic!("the v2 data declares a left-hand table"));
    let right = tables
        .hand(Hand::Right)
        .unwrap_or_else(|| panic!("the v2 data declares a right-hand table"));

    assert_eq!(
        left.hand(),
        Hand::Left,
        "the left table reports its own hand"
    );
    assert_eq!(
        right.hand(),
        Hand::Right,
        "the right table reports its own hand"
    );

    let anchor = InputId::parse("keyboard:a").expect("`keyboard:a` is a glyph member");
    assert_eq!(
        (grade_id(left, &anchor), grade_id(right, &anchor)),
        (Some("home".to_string()), Some("off-reach".to_string())),
        "the same key must grade differently per hand — the two tables are not one answer"
    );
}

/// proves: Test strategy #3 — both shipped tables list exactly the five taxonomy
/// ids, once each, in the severity order this design pins: rank is the index in
/// that shared order, so a table that reorders it would redefine the bound.
#[test]
fn both_shipped_tables_list_the_five_classes_in_severity_order() {
    let tables = load_shipped_v2();
    for (hand, table) in [
        (
            "left",
            tables
                .hand(Hand::Left)
                .expect("the v2 data declares a left-hand table"),
        ),
        (
            "right",
            tables
                .hand(Hand::Right)
                .expect("the v2 data declares a right-hand table"),
        ),
    ] {
        assert_eq!(
            class_ids(table),
            TAXONOMY,
            "the shipped {hand} table's classes must be the five taxonomy ids in severity order"
        );
    }
}

/// proves: Test strategy #3 — both hands grade exactly the same 48 input ids,
/// each exactly once, and the loader files each pair's grade under the hand its
/// document declares.
#[test]
fn both_shipped_hands_grade_the_same_48_inputs_each_once() {
    let dir = schemas_root().join("reach-classes").join("v2");
    let tables = load_shipped_v2();
    let vocabulary: BTreeSet<String> = GRADED_IDS
        .iter()
        .map(|member| format!("keyboard:{member}"))
        .collect();

    for (hand, file) in [(Hand::Left, "left.toml"), (Hand::Right, "right.toml")] {
        let label = format!("reach-classes/v2/{file}");
        let text = fixture_text(&dir.join(file));
        let pairs = shipped_keys(&text, &label);
        let declared: BTreeSet<String> = pairs.iter().map(|(input, _)| input.clone()).collect();

        assert_eq!(
            pairs.len(),
            declared.len(),
            "{label}: an input id appears once per hand, so {} pairs must name {} distinct ids",
            pairs.len(),
            declared.len()
        );
        assert_eq!(
            declared.len(),
            GRADED_IDS.len(),
            "{label}: the graded set is the closed keyboard set — {} inputs",
            GRADED_IDS.len()
        );
        assert_eq!(
            declared, vocabulary,
            "{label}: both hands grade exactly the closed vocabulary's keyboard members"
        );

        let table = tables
            .hand(hand)
            .unwrap_or_else(|| panic!("{label} must be filed under its own hand"));
        for (input, class) in &pairs {
            let id = InputId::parse(input)
                .unwrap_or_else(|err| panic!("{label}: `{input}` is a graded id: {err}"));
            assert_eq!(
                id.namespace(),
                InputNamespace::Keyboard,
                "{label}: a graded input is a `keyboard:*` id"
            );
            let graded = table
                .grade(&id)
                .unwrap_or_else(|| panic!("{label}: `{input}` is graded, so `grade` answers"));
            assert_eq!(
                graded.id, *class,
                "{label}: `grade` must answer with the class the `keys` pair names for `{input}`"
            );
        }
    }
}

/// proves: Test strategy #3 — the asymmetry the ADR records is data, not a special
/// case: the right pinky's `;` and `'` are `off-reach` for the left hand, and each
/// hand grades them from its own anchor (A6).
#[test]
fn the_two_hands_grade_the_right_hand_punctuation_differently() {
    let tables = load_shipped_v2();
    let left = tables
        .hand(Hand::Left)
        .expect("the v2 data declares a left-hand table");
    let right = tables
        .hand(Hand::Right)
        .expect("the v2 data declares a right-hand table");

    for (member, right_grade, left_grade) in
        [(";", "home", "off-reach"), ("'", "stretch", "off-reach")]
    {
        let id = InputId::parse(&format!("keyboard:{member}"))
            .unwrap_or_else(|err| panic!("`keyboard:{member}` is a glyph member: {err}"));
        assert_eq!(
            grade_id(right, &id).as_deref(),
            Some(right_grade),
            "`keyboard:{member}` grades `{right_grade}` for the right hand"
        );
        assert_eq!(
            grade_id(left, &id).as_deref(),
            Some(left_grade),
            "`keyboard:{member}` grades `{left_grade}` for the left hand"
        );
    }
}

/// proves: Test strategy #3 — `keyboard:space` is `thumb` for both hands: the thumb
/// is the one designated finger for the one key it reaches from a resting hand,
/// which is why `thumb` is a class of its own (A9, A10).
#[test]
fn space_grades_thumb_for_both_hands() {
    let tables = load_shipped_v2();
    let id = InputId::parse("keyboard:space").expect("the reserved word `space` is a member");

    for (hand, table) in [
        (
            "left",
            tables
                .hand(Hand::Left)
                .expect("the v2 data declares a left-hand table"),
        ),
        (
            "right",
            tables
                .hand(Hand::Right)
                .expect("the v2 data declares a right-hand table"),
        ),
    ] {
        assert_eq!(
            grade_id(table, &id).as_deref(),
            Some("thumb"),
            "`keyboard:space` is the thumb's designated key for the {hand} hand too"
        );
    }
}

/// proves: Test strategy #3 + The model — a reach table whose declared `version`
/// differs from the version segment of the directory it loads from is refused, v1
/// directories included: a v2 document must not answer from a v1 directory and a
/// v1 document must not answer from a v2 one. v1-on-v1 passes trivially, which is
/// why the shipped v1 corpus and `reach_tables.rs` keep their behavior.
#[test]
fn a_reach_table_declaring_a_version_that_disagrees_with_its_directory_is_refused() {
    let pairs = ["keyboard:a home", "keyboard:d home"];

    let agreed = ScratchDir::new("reach-version-v1-on-v1");
    agreed.write("v1/left.toml", &v1_table("left", &[]));
    agreed.write("v1/right.toml", &v1_table("right", &[]));
    bindforge_core::load_reach_tables(&agreed.path().join("v1")).unwrap_or_else(|err| {
        panic!(
            "a v1 document loading from a v1 directory passes trivially — every shipped v1 table \
             declares v1: {err}"
        )
    });

    let v2_in_v1 = ScratchDir::new("reach-version-v2-document-in-v1-directory");
    v2_in_v1.write("v1/left.toml", &v2_table("left", &pairs));
    v2_in_v1.write("v1/right.toml", &v2_table("right", &pairs));
    let Err(err) = bindforge_core::load_reach_tables(&v2_in_v1.path().join("v1")) else {
        panic!(
            "a table declaring `v2` loaded from a `v1` directory must be refused: the declared \
             version must equal the directory's version segment"
        );
    };
    assert_version_refusal(&err);

    let v1_in_v2 = ScratchDir::new("reach-version-v1-document-in-v2-directory");
    v1_in_v2.write("v2/left.toml", &v1_table("left", &[]));
    v1_in_v2.write("v2/right.toml", &v1_table("right", &[]));
    let Err(err) = bindforge_core::load_reach_tables(&v1_in_v2.path().join("v2")) else {
        panic!(
            "a table declaring `v1` loaded from a `v2` directory must be refused: the declared \
             version must equal the directory's version segment"
        );
    };
    assert_version_refusal(&err);
}

/// proves: A2 + Test strategy #6 — `v2` is a supported major beside the untouched
/// `v1`, so a v1-only reader meets the version gate naming the supported majors
/// rather than misreading a v2 document; and because the `v2` directory exists, the
/// majors-present equality in `version_gate.rs` now covers both majors.
#[test]
fn v2_is_a_supported_major_beside_the_untouched_v1() {
    let majors: BTreeSet<String> = bindforge_core::supported_majors().into_iter().collect();

    for major in ["v1", "v2"] {
        assert!(
            majors.contains(major),
            "bindforge-core must declare `{major}` as a supported major, got {majors:?}"
        );
    }
}
