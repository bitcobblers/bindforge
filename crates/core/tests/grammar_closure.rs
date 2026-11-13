//! The closed entry set and the documented grammar subset of the `schemas/`
//! formats contract.
//!
//! proves: Chosen design → Formats contract + `schemas/profile/v1/spec.md`,
//! `schemas/reach-classes/v1/spec.md`, `schemas/reach-classes/v2/spec.md`,
//! `schemas/keyboard-geometry/v1/spec.md` and `schemas/manifests/v1/spec.md` —
//! each format's entry set is closed, so
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

/// A `manifests/v1` document using every entry `schemas/manifests/v1/spec.md`
/// defines: the identity group (`id`, `revision`), the codec family id, the
/// optional fingerprint, the action table, the three constraint entries, and the
/// preset ids — the eleven the entry table lists, no more.
const MANIFEST_USING_DEFINED_ENTRIES: &str = r#"format = "manifests"
version = "v1"
id = "remnant-2"
revision = 3
codec = "unreal-engine-5"
fingerprint = "sha256-2Z7Gx9m0YU3bJ8kT4sQw1n7dC6vH0pL2xR5yB8aN3eM="
actions = ["move.steer-left Turn left", "aim.camera Mouse camera"]
exclusive-groups = ["keyboard:w keyboard:s"]
dead-keys = ["keyboard:x"]
docked-actions = ["ui.menu keyboard:y"]
presets = ["default", "southpaw"]
"#;

/// A `manifests/v1` document carrying the entry the design considered and
/// refused: an `axis-kinds` array restating, per row, the axis kind
/// `schemas/action-ids/v1/registry.toml` already owns per id — the second answer
/// to one fact, and the drift channel the closed set exists to refuse (A7).
const MANIFEST_WITH_AN_UNDEFINED_ENTRY: &str = r#"format = "manifests"
version = "v1"
id = "remnant-2"
revision = 3
codec = "unreal-engine-5"
actions = ["move.steer-left Turn left"]
axis-kinds = ["move.steer-left none"]
"#;

/// A `profile/v2` document using every entry `schemas/profile/v2/spec.md`
/// defines: v1's four unchanged in meaning plus `kind`, the `for` pin, the
/// three delta lists, and the two provenance labels — the eleven the entry
/// table lists, no more (design 014, A1).
const PROFILE_V2_USING_DEFINED_ENTRIES: &str = r#"format = "profile"
version = "v2"
id = "defined-entries-v2"
derives = ["game-base"]
kind = "patch"
for = "remnant-2@3"
set = ["move.steer-left keyboard:d"]
clear = ["aim.camera"]
swap = ["move.steer-left combat.fire"]
seeded-from = "default"
generated-by = "human"
"#;

/// A `profile/v2` document that aliases a known entry: a reader that ignores
/// unknown entries reads it as a profile with no ancestors at all, under the
/// very version whose chain machinery the entry would steer (design 014's Test
/// strategy #8).
const PROFILE_V2_WITH_AN_ALIASED_ENTRY: &str = r#"format = "profile"
version = "v2"
id = "aliased-chain-v2"
derives = ["game-base"]
kind = "patch"
for = "remnant-2@3"
derives_ = ["ancestor-1"]
"#;

/// A `keyboard-geometry/v1` document whose `geometry` name breaks the slug
/// grammar `[a-z][a-z0-9-]*` — the same grammar a manifest `id`, `codec` and
/// preset id ride, defined once and renamed `is_slug_name` when the third entry
/// kind arrived (A18).
const GEOMETRY_WITH_AN_UPPERCASE_NAME: &str = r#"format = "keyboard-geometry"
version = "v1"
geometry = "ANSI-US"
keys = ["keyboard:a 0 2"]
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
/// strategy #7), and `manifests/v1` joins it the same way again (ADR-013, design
/// 013's Test strategy #7).
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
        ("manifests", "v1"),
        ("profile", "v2"),
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

/// proves: design 013's Test strategy #7 + ADR-013 (The contract, closed entry
/// set) — closing the set is not shrinking the grammar for the manifest format
/// either: a document using exactly the eleven entries its entry table defines
/// parses, `actions` rows that split at their first space and the optional
/// fingerprint included.
#[test]
fn accepts_every_entry_the_manifest_grammar_defines() {
    if let Err(err) = bindforge_core::parse(MANIFEST_USING_DEFINED_ENTRIES) {
        panic!(
            "a manifest using only the entries its entry table defines must parse; it was \
             refused: {err}"
        );
    }
}

/// proves: design 013's Test strategy #7 + Chosen design (Options considered: the
/// action row) — the manifest entry set is closed the same way, and the case this
/// suite pins is the dangerous one: an `axis-kinds` entry would restate per row a
/// fact `schemas/action-ids/v1/registry.toml` already owns per id, giving one fact
/// two answers and a drift channel (A7). It is a refusal naming the entry, never
/// an ignored key.
#[test]
fn rejects_a_manifest_document_carrying_an_entry_outside_the_grammar() {
    let err = bindforge_core::parse(MANIFEST_WITH_AN_UNDEFINED_ENTRY)
        .expect_err("a manifest carrying `axis-kinds` must be refused, not ignored");

    let detail = err.to_string();
    assert!(
        detail.contains("axis-kinds"),
        "the refusal must name the entry it refused, got `{detail}`"
    );
}

/// proves: design 013's Test strategy #7 + A18 — the slug-name predicate is one
/// definition now that three entry kinds share its grammar (a geometry name, a
/// manifest `id`, `codec`, and a preset id), and renaming it
/// `is_geometry_name` → `is_slug_name` may not move the grammar it defines: a
/// geometry name outside `[a-z][a-z0-9-]*` is refused exactly as it was before the
/// rename, in the one-line message that names what it refused.
#[test]
fn rejects_a_keyboard_geometry_name_outside_the_slug_grammar() {
    let err = bindforge_core::parse(GEOMETRY_WITH_AN_UPPERCASE_NAME).expect_err(
        "an uppercase geometry name is outside the slug grammar, before the rename and after it",
    );

    let detail = err.to_string();
    assert!(
        detail.contains("ANSI-US") || detail.contains("geometry"),
        "the refusal must name the name or the entry it refused, got `{detail}`"
    );
    assert!(
        !detail.contains('\n') && !detail.contains('\r'),
        "the refusal stays one bounded line: {detail:?}"
    );
}

/// proves: design 014's Test strategy #8 + The contract (closed entry set) —
/// closing the v2 set is not shrinking the grammar: a profile using exactly
/// the eleven entries the v2 entry table defines parses, the v1 four
/// unchanged in meaning (A1, A15).
#[test]
fn accepts_every_entry_the_profile_v2_grammar_defines() {
    if let Err(err) = bindforge_core::parse(PROFILE_V2_USING_DEFINED_ENTRIES) {
        panic!(
            "a profile using only the entries the v2 entry table defines must parse; it was \
             refused: {err}"
        );
    }
}

/// proves: design 014's Test strategy #8 — the closed entry set is per-
/// version, and v2 is closed the same way v1 is: a `profile/v2` document
/// aliasing `derives_` is refused naming the entry and the version whose entry
/// set was checked — an ignored alias would read as *no ancestors at all*
/// under the very version whose whole contract is the chain (A4: the entry-set
/// arm keyed on the declared version is the shipped `reach-classes` mechanism).
#[test]
fn rejects_a_profile_v2_document_carrying_an_aliased_entry() {
    let err = bindforge_core::parse(PROFILE_V2_WITH_AN_ALIASED_ENTRY).expect_err(
        "a v2 profile carrying `derives_` must be refused, not read as a chain-shaping entry \
         outside the grammar",
    );

    let detail = err.to_string();
    assert!(
        detail.contains("derives_"),
        "the refusal must name the entry it refused, got `{detail}`"
    );
    assert!(
        detail.contains("v2"),
        "the refusal names the version whose entry set it checked, not a hard-coded `v1`, got \
         `{detail}`"
    );
}
