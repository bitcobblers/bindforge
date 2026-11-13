//! The closed entry set and the documented grammar subset of the `schemas/`
//! formats contract.
//!
//! proves: Chosen design → Formats contract + `schemas/profile/v1/spec.md` and
//! `schemas/reach-classes/v1/spec.md` — each format's entry set is closed, so an
//! entry the grammar does not define is a typed refusal naming the entry, never
//! an ignored key: an alias of a known entry (`derives_`) read as *that entry
//! being absent* would quietly change what a profile derives from, and an
//! undefined entry admits data no implementer has agreed a meaning for.
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
#[test]
fn the_invalid_corpus_ships_an_unknown_entry_fixture_for_every_format() {
    for format in ["profile", "reach-classes", "action-ids"] {
        let dir = version_dir(format, "v1");
        let path = dir
            .path
            .join("fixtures")
            .join("invalid")
            .join("unknown-entry.toml");

        assert!(
            path.is_file(),
            "{format}/v1 must ship `fixtures/invalid/unknown-entry.toml`; the invalid-fixture \
             corpus is the contract for rejection behavior"
        );
        assert!(
            bindforge_core::parse(&fixture_text(&path)).is_err(),
            "{format}/v1's unknown-entry fixture must be refused: {}",
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
