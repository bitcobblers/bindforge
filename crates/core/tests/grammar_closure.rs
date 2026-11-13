//! The closed entry set and the documented grammar subset of the `schemas/`
//! formats contract.
//!
//! proves: Chosen design → Formats contract + `schemas/profile/v1/spec.md`,
//! `schemas/reach-classes/v1/spec.md`, `schemas/reach-classes/v2/spec.md` and
//! `schemas/keyboard-geometry/v1/spec.md` — each format's entry set is closed, so
//! an entry the grammar does not define is a typed refusal naming the entry, never
//! an ignored key: an alias of a known entry (`derives_`) read as *that entry
//! being absent* would quietly change what a profile derives from, and an
//! undefined entry admits data no implementer has agreed a meaning for. ADR-012
//! makes that rule version-aware for `reach-classes`: v1's set stays closed forever
//! and v2's set is v1's plus `keys`, so the refusal names the version it checked
//! (The model → `document.rs`).
//!
//! The same contract states the grammar as a documented subset of TOML v1.1.0
//! (`schemas/README.md`): table headers, dotted keys, multi-line arrays, floats
//! and hex integers are legal TOML and outside the v1 grammar, so a reader built
//! on a standard TOML parser still has to refuse them — and each one has a
//! fixture under `schemas/profile/v1/fixtures/invalid/` pinning it for every
//! implementer.

mod common;

use common::{fixture_text, version_dir};

/// A profile that aliases a known entry: a reader that ignores unknown entries
/// reads it as a profile with no ancestors at all.
const PROFILE_WITH_AN_ALIASED_ENTRY: &str = r#"format = "profile"
version = "v1"
id = "aliased-chain"
derives = []
derives_ = ["ancestor-1"]
"#;

/// A reach-classes table carrying an entry the grammar never defined.
const TABLE_WITH_AN_UNDEFINED_ENTRY: &str = r#"format = "reach-classes"
version = "v1"
hand = "left"
classes = ["near"]
scope = "global"
"#;

/// A profile using every entry `profile/v1/spec.md` defines.
const PROFILE_USING_DEFINED_ENTRIES: &str = r#"format = "profile"
version = "v1"
id = "defined-entries"
derives = ["ancestor-1"]
"#;

/// A reach-classes table using every entry `reach-classes/v1/spec.md` defines.
const TABLE_USING_DEFINED_ENTRIES: &str = r#"format = "reach-classes"
version = "v1"
hand = "right"
classes = ["near", "mid"]
"#;

/// A `reach-classes/v2` table using every entry `reach-classes/v2/spec.md`
/// defines: v1's four plus `keys`, the mapping the new major exists to carry.
const TABLE_V2_USING_DEFINED_ENTRIES: &str = r#"format = "reach-classes"
version = "v2"
hand = "left"
classes = ["home", "thumb", "stretch", "far", "off-reach"]
keys = ["keyboard:a home", "keyboard:space thumb", "keyboard:\\ off-reach"]
"#;

/// A `reach-classes/v2` table carrying an entry the v2 entry table never defines.
/// The v1 placeholder `classes` list is not enough here: the v2 entry set is chosen
/// from the declared version, so the refusal has to be about the entry.
const TABLE_V2_WITH_AN_UNDEFINED_ENTRY: &str = r#"format = "reach-classes"
version = "v2"
hand = "left"
classes = ["home", "thumb", "stretch", "far", "off-reach"]
keys = ["keyboard:a home"]
scope = "global"
"#;

/// A `keyboard-geometry/v1` document using every entry its spec defines:
/// `format`, `version`, `geometry`, and the `keys` triples.
const GEOMETRY_USING_DEFINED_ENTRIES: &str = r#"format = "keyboard-geometry"
version = "v1"
geometry = "ansi-us"
keys = ["keyboard:a 0 2", "keyboard:space 3 4"]
"#;

/// A `keyboard-geometry/v1` document carrying an entry the grammar never defined.
const GEOMETRY_WITH_AN_UNDEFINED_ENTRY: &str = r#"format = "keyboard-geometry"
version = "v1"
geometry = "ansi-us"
keys = ["keyboard:a 0 2"]
row-height = "1u"
"#;

/// The TOML constructs the contract's grammar excludes, each with its corpus
/// fixture named after it (`spec.md`: "each is rejected, and each has a fixture
/// under `fixtures/invalid/` pinning it").
const EXCLUDED_TOML_CONSTRUCTS: [&str; 5] = [
    "table-header",
    "dotted-key",
    "multi-line-array",
    "float-value",
    "hex-integer",
];

/// proves: Formats contract (closed entry set) — a profile carrying an entry
/// outside the grammar is refused rather than parsed with the entry dropped, and
/// the refusal names the entry. The alias case is the dangerous one: a reader
/// that ignores unknown entries reads `derives_` as *no ancestors at all*, so a
/// derived profile is silently reinterpreted as a standalone one and the chain
/// guards have nothing to reject.
#[test]
fn rejects_a_profile_carrying_an_entry_outside_the_grammar() {
    let err = bindforge_core::parse(PROFILE_WITH_AN_ALIASED_ENTRY).expect_err(
        "a profile carrying `derives_` must be refused, not read as having no ancestors",
    );

    let detail = err.to_string();
    assert!(
        detail.contains("derives_"),
        "the refusal must name the entry it refused, got `{detail}`"
    );
}

/// proves: Formats contract (closed entry set) — the same rule holds for the
/// reach-classes format: an undefined entry is refused, so no later component
/// decides what it means.
#[test]
fn rejects_a_reach_classes_table_carrying_an_entry_outside_the_grammar() {
    let err = bindforge_core::parse(TABLE_WITH_AN_UNDEFINED_ENTRY)
        .expect_err("a reach-classes table carrying `scope` must be refused, not ignored");

    let detail = err.to_string();
    assert!(
        detail.contains("scope"),
        "the refusal must name the entry it refused, got `{detail}`"
    );
}

/// proves: Formats contract (closed entry set) — the corpus carries the case for
/// every shipped format, which is what pins the closed set for third-party
/// readers instead of only for `bindforge-core`. `action-ids` joins the list with
/// its own `emphasis` fixture (Test strategy #5): ADR-011 says emphasis is not in
/// the taxonomy, so an `emphasis` entry is a refusal naming the entry.
/// `keyboard-geometry/v1` joins it the same way (ADR-012, design 012's Test
/// strategy #7).
/// `reach-classes/v2` is the one directory whose closed set this suite pins
/// directly rather than through a fixture: the approved v2 corpus names its
/// invalid fixtures and no `unknown-entry.toml` among them, and
/// `reach-classes/v1/fixtures/invalid/keys-entry.toml` is the fixture that pins
/// the version-aware half of the rule.
#[test]
fn the_invalid_corpus_ships_an_unknown_entry_fixture_for_every_format() {
    for (format, major) in [
        ("profile", "v1"),
        ("reach-classes", "v1"),
        ("action-ids", "v1"),
        ("keyboard-geometry", "v1"),
    ] {
        let dir = version_dir(format, major);
        let path = dir
            .path
            .join("fixtures")
            .join("invalid")
            .join("unknown-entry.toml");

        assert!(
            path.is_file(),
            "{format}/{major} must ship `fixtures/invalid/unknown-entry.toml`; the invalid-fixture \
             corpus is the contract for rejection behavior"
        );
        assert!(
            bindforge_core::parse(&fixture_text(&path)).is_err(),
            "{format}/{major}'s unknown-entry fixture must be refused: {}",
            path.display()
        );
    }
}

/// proves: Formats contract (documented subset of TOML v1.1.0) — every construct
/// the grammar excludes has its own invalid fixture, and the reference reader
/// refuses each one. A standard TOML parser accepts all of them, so the contract
/// — not the parser — is what closes the grammar.
#[test]
fn the_invalid_corpus_pins_every_toml_construct_the_grammar_excludes() {
    let profile_v1 = version_dir("profile", "v1");

    for construct in EXCLUDED_TOML_CONSTRUCTS {
        let path = profile_v1
            .path
            .join("fixtures")
            .join("invalid")
            .join(format!("{construct}.toml"));

        assert!(
            path.is_file(),
            "{construct} is legal TOML that the v1 grammar excludes; `spec.md` says each excluded \
             construct has a fixture, and {} is missing",
            profile_v1.path.display()
        );
        assert!(
            bindforge_core::parse(&fixture_text(&path)).is_err(),
            "{construct} must be refused by the reference reader: {}",
            path.display()
        );
    }
}

/// proves: Formats contract (closed entry set) — closing the set is not the same
/// as shrinking the grammar: a document using exactly the entries each format's
/// table defines is accepted.
#[test]
fn accepts_every_entry_the_v1_grammars_define() {
    for document in [PROFILE_USING_DEFINED_ENTRIES, TABLE_USING_DEFINED_ENTRIES] {
        if let Err(err) = bindforge_core::parse(document) {
            panic!(
                "a document using only the entries the grammar defines must parse; \
                 it was refused: {err}"
            );
        }
    }
}

/// proves: design 012's Test strategy #7 + ADR-012 Decision One — the same rule
/// holds for the
/// two directories ADR-012 adds: `reach-classes/v2`'s entry set is v1's plus
/// `keys`, and `keyboard-geometry/v1`'s is `format`, `version`, `geometry` and
/// `keys`, so a document using exactly those entries parses — including the
/// backslash glyph as `\\`, which the closed escape table round-trips (A3).
#[test]
fn accepts_every_entry_the_new_grammars_define() {
    for document in [
        TABLE_V2_USING_DEFINED_ENTRIES,
        GEOMETRY_USING_DEFINED_ENTRIES,
    ] {
        if let Err(err) = bindforge_core::parse(document) {
            panic!(
                "a document using only the entries its entry table defines must parse; it was \
                 refused: {err}"
            );
        }
    }
}

/// proves: design 012's Test strategy #7 + The model (`document.rs`) — the closed
/// entry set is
/// version-aware, not one list per format: an entry outside the `reach-classes/v2`
/// set is refused naming the entry, while v1's set stays closed forever. A reader
/// that widened v1 to accept `keys` would break every v1 loader of those files
/// (A1), so the entry set is picked from the declared version on both sides.
#[test]
fn rejects_a_reach_classes_v2_table_carrying_an_entry_outside_the_grammar() {
    let err = bindforge_core::parse(TABLE_V2_WITH_AN_UNDEFINED_ENTRY)
        .expect_err("a v2 reach-classes table carrying `scope` must be refused, not ignored");

    let detail = err.to_string();
    assert!(
        detail.contains("scope"),
        "the refusal must name the entry it refused, got `{detail}`"
    );
    assert!(
        detail.contains("v2"),
        "the refusal names the version whose entry set it checked, not a hard-coded `v1`, got \
         `{detail}`"
    );
}

/// proves: Formats contract (closed entry set) — the new `keyboard-geometry`
/// format is closed the same way: an entry its spec does not define is a typed
/// refusal naming the entry, never an ignored key, so no later component decides
/// what it means.
#[test]
fn rejects_a_keyboard_geometry_document_carrying_an_entry_outside_the_grammar() {
    let err = bindforge_core::parse(GEOMETRY_WITH_AN_UNDEFINED_ENTRY).expect_err(
        "a keyboard-geometry document carrying `row-height` must be refused, not ignored",
    );

    let detail = err.to_string();
    assert!(
        detail.contains("row-height"),
        "the refusal must name the entry it refused, got `{detail}`"
    );
}
