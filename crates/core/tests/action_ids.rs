//! The `action-ids` contract tree end to end: the shipped registry at
//! `schemas/action-ids/v1/`, the fixture corpus the conformance harness walks,
//! and what `load_action_registry` answers from that directory.
//!
//! proves: Test strategy #5 — the harness is unchanged: `conformance.rs`'s
//! `schemas/*/v*/` walk picks up `action-ids/v1` with no harness change, every
//! valid fixture parses and re-serializes to a semantically equal document,
//! every invalid fixture is refused with a single-line bounded message, and the
//! zero-fixture rule fires if either fixture directory is emptied. The
//! `grammar_closure.rs` leg of #5 is the closed-entry-suite format list itself,
//! which names `action-ids` alongside `profile` and `reach-classes`.
//!
//! proves: Test strategy #6 — the shipped registry loads: 40 ids, each
//! grammar-valid and listed once, over the seven namespaces with the counts
//! Chosen design states; and a version directory with no `registry.toml`, or one
//! holding another format's document, is refused without naming a local path.
//! Plus the #4 half that keeps the widened tree honest: the majors
//! `bindforge-core` declares still equal the majors present under `schemas/*/v*/`.
//!
//! Test strategy #7 has no test-file leg: the three gate commands running green
//! on the Linux CI runner is the merge PR's `rust` job, and the Windows
//! contributor run is a rollout check, exactly as design 008's fresh-clone and CI
//! legs are (`conformance.rs` header). What this suite can pin for the
//! case-insensitive-filesystem half of #7 is that every id and every fixture name
//! the design ships is lowercase ASCII — the corpus-name rule `conformance.rs`
//! asserts for every format, `action-ids` included.
//!
//! The shipped `registry.toml` is the only id list (Options considered): nothing
//! in `src/` repeats it, so the enumeration below is copied from Chosen design
//! deliberately — the registry is a compatibility promise, and an id added,
//! renamed or retargeted is a reviewed PR that moves this list with it. These
//! tests are written from the proposal before the format ships, so they fail
//! until it does; none may be weakened to make anything pass.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bindforge_core::{ActionId, ActionRegistry, AxisKind, CoreError};
use common::{
    ScratchDir, fixture_text, fixtures, format_dirs, format_dirs_under, string_array, version_dir,
    zero_fixture_violation,
};

/// The ids `schemas/action-ids/v1/registry.toml` ships, by axis-kind array, as
/// Chosen design enumerates them: 40 ids over seven namespaces.
const SHIPPED_IDS: &[&str] = &[
    // discrete-lateral
    "move.steer-left",
    "move.steer-right",
    "move.strafe-left",
    "move.strafe-right",
    // continuous-lateral
    "aim.camera",
    // none
    "move.forward",
    "move.backward",
    "move.jump",
    "move.sprint",
    "aim.sights",
    "aim.cycle-target",
    "combat.melee",
    "combat.fire",
    "combat.reload",
    "combat.dodge",
    "combat.block",
    "combat.skill-1",
    "combat.skill-2",
    "combat.skill-3",
    "interact.primary",
    "interact.secondary",
    "interact.pickup",
    "item.slot-1",
    "item.slot-2",
    "item.slot-3",
    "item.slot-4",
    "item.slot-5",
    "item.slot-6",
    "item.heal",
    "item.cycle",
    "ui.map",
    "ui.inventory",
    "ui.menu",
    "ui.journal",
    "ui.chat",
    "ui.scoreboard",
    "system.pause",
    "system.settings",
    "system.respawn",
    "system.screenshot",
];

/// The namespace counts Chosen design states: the namespaces are a naming
/// convention, not a closed grammar set, and the registry is the closed set.
const NAMESPACE_COUNTS: [(&str, usize); 7] = [
    ("move", 8),
    ("aim", 3),
    ("combat", 8),
    ("interact", 3),
    ("item", 8),
    ("ui", 6),
    ("system", 4),
];

/// The corpus Chosen design names. Each invalid fixture carries one fault and is
/// named after it (`toml-test`'s rule), so the corpus itself is checkable.
const VALID_FIXTURES: [&str; 3] = [
    "basic-registry.toml",
    "empty-axis-arrays.toml",
    "numeric-slot-ids.toml",
];
const INVALID_FIXTURES: [&str; 14] = [
    "unknown-entry.toml",
    "missing-axis-arrays.toml",
    "uppercase-id.toml",
    "leading-digit-segment.toml",
    "leading-hyphen-segment.toml",
    "trailing-hyphen-segment.toml",
    "empty-segment.toml",
    "underscore-id.toml",
    "unprefixed-id.toml",
    "third-segment.toml",
    "segment-past-63-octets.toml",
    "duplicate-id.toml",
    "non-string-element.toml",
    "non-ascii-id.toml",
];

/// The version directory whose `registry.toml` is the registry.
fn registry_dir() -> PathBuf {
    version_dir("action-ids", "v1").path
}

/// The shipped registry, loaded through the same path every reader takes.
fn load_shipped() -> ActionRegistry {
    let dir = registry_dir();
    bindforge_core::load_action_registry(&dir).unwrap_or_else(|err| {
        panic!(
            "the shipped registry under {} must load: {err}",
            dir.display()
        )
    })
}

/// The text of the shipped `registry.toml`, so the arrays the loader files ids
/// under can be compared with the file that lists them.
fn shipped_registry_text() -> String {
    fixture_text(&registry_dir().join("registry.toml"))
}

/// The grammar-valid ids of the shipped registry, as strings.
fn shipped_id_strings(registry: &ActionRegistry) -> Vec<String> {
    registry
        .ids()
        .iter()
        .map(|id| id.as_str().to_string())
        .collect()
}

/// proves: Test strategy #5 — the harness needs no change: the `schemas/*/v*/`
/// walk already exercises `action-ids/v1`, and the corpus Chosen design names is
/// present for it, every fixture named after the one fault it carries.
#[test]
fn the_conformance_walk_exercises_the_action_ids_version_directory() {
    let found: Vec<String> = format_dirs()
        .iter()
        .map(|dir| format!("{}/{}", dir.format, dir.major))
        .collect();
    assert!(
        found.iter().any(|dir| dir == "action-ids/v1"),
        "the harness walk must exercise `action-ids/v1` with no harness change; the walk \
         found {found:?}"
    );

    let dir = registry_dir();
    for fixture in VALID_FIXTURES {
        assert!(
            dir.join("fixtures").join("valid").join(fixture).is_file(),
            "`action-ids/v1` must ship `fixtures/valid/{fixture}`; a file's presence there is \
             the definition of \"supported\": {}",
            dir.display()
        );
    }
    for fixture in INVALID_FIXTURES {
        assert!(
            dir.join("fixtures").join("invalid").join(fixture).is_file(),
            "`action-ids/v1` must ship `fixtures/invalid/{fixture}`; the invalid corpus is the \
             contract for rejection behavior: {}",
            dir.display()
        );
    }
}

/// proves: Test strategy #5 — every valid `action-ids` fixture parses, passes the
/// guards, and re-serializes to a semantically equal document: `serialize` writes
/// the three arrays back in document order, so the corpus round-trips the way
/// `profile` does and the harness needs no special case for the format.
#[test]
fn every_action_ids_valid_fixture_re_serializes_to_a_semantically_equal_value() {
    let dir = version_dir("action-ids", "v1");
    let corpus = fixtures(&dir, "valid");
    assert!(
        !corpus.is_empty(),
        "{} must hold valid fixtures: presence in `fixtures/valid/` is what makes the format \
         supported",
        dir.path.display()
    );

    for path in &corpus {
        let label = path.display().to_string();
        let text =
            std::fs::read_to_string(path).unwrap_or_else(|err| panic!("reading {label}: {err}"));

        let document = bindforge_core::parse(&text)
            .unwrap_or_else(|err| panic!("{label} is a valid fixture and must parse: {err}"));
        let round_tripped = bindforge_core::parse(&bindforge_core::serialize(&document))
            .unwrap_or_else(|err| panic!("{label} does not re-parse after serializing: {err}"));
        assert_eq!(
            round_tripped, document,
            "{label} must re-serialize to a semantically equal document"
        );
    }
}

/// proves: Test strategy #5 — every invalid `action-ids` fixture is refused with
/// a structured error in one bounded line with no raw control character, so a
/// hostile registry cannot forge a log line or leak a path.
#[test]
fn every_action_ids_invalid_fixture_is_refused_with_a_single_line_message() {
    let dir = version_dir("action-ids", "v1");
    let corpus = fixtures(&dir, "invalid");
    assert!(
        !corpus.is_empty(),
        "{} must hold invalid fixtures: the invalid corpus is the contract for which condition \
         produces which rejection",
        dir.path.display()
    );

    for path in &corpus {
        let label = path.display().to_string();
        let text =
            std::fs::read_to_string(path).unwrap_or_else(|err| panic!("reading {label}: {err}"));

        let detail = match bindforge_core::parse(&text) {
            Err(err) => err.to_string(),
            Ok(_) => panic!("{label} is an invalid fixture and must be rejected"),
        };
        assert!(
            !detail.is_empty(),
            "{label} must be rejected with an error that says what is wrong"
        );
        assert!(
            !detail.contains('\n') && !detail.contains('\r'),
            "{label} must be rejected in one line; the rejection rendered {detail:?}"
        );
        assert!(
            detail.chars().all(|ch| !ch.is_control()),
            "{label}'s rejection must escape control characters rather than emit them: {detail:?}"
        );
    }
}

/// proves: Test strategy #5 — the zero-fixture rule fires on the new corpus: an
/// `action-ids/v1` with an empty `fixtures/valid/` or an empty `fixtures/invalid/`
/// fails the run, so an empty corpus can never make the format look supported.
#[test]
fn the_zero_fixture_rule_fires_if_the_action_ids_fixture_directories_are_emptied() {
    let shipped = registry_dir();
    let one_invalid = fixture_text(
        &shipped
            .join("fixtures")
            .join("invalid")
            .join("unknown-entry.toml"),
    );
    let one_valid = fixture_text(
        &shipped
            .join("fixtures")
            .join("valid")
            .join("basic-registry.toml"),
    );

    let scratch = ScratchDir::new("action-ids-zero-fixture");
    scratch.make_dir("action-ids/v1/fixtures/valid");
    scratch.write(
        "action-ids/v1/fixtures/invalid/unknown-entry.toml",
        &one_invalid,
    );

    let dirs = format_dirs_under(scratch.path());
    assert_eq!(
        dirs.len(),
        1,
        "the walk must see the scratch `action-ids/v1` under {}",
        scratch.path().display()
    );
    assert!(
        zero_fixture_violation(&dirs).is_some(),
        "an `action-ids/v1` with an empty `fixtures/valid/` must fail the run"
    );

    let scratch = ScratchDir::new("action-ids-zero-invalid");
    scratch.write(
        "action-ids/v1/fixtures/valid/basic-registry.toml",
        &one_valid,
    );
    scratch.make_dir("action-ids/v1/fixtures/invalid");

    let dirs = format_dirs_under(scratch.path());
    assert_eq!(
        dirs.len(),
        1,
        "the walk must see the scratch `action-ids/v1` under {}",
        scratch.path().display()
    );
    assert!(
        zero_fixture_violation(&dirs).is_some(),
        "an `action-ids/v1` with an empty `fixtures/invalid/` must fail the run"
    );
}

/// proves: Test strategy #4 — adding a format does not move the version gate:
/// the majors `bindforge-core` declares still equal the majors present under
/// `schemas/*/v*/`, and the new format's own majors are among them. The whole
/// tree already satisfies this rule with `profile` and `reach-classes`
/// (`version_gate.rs`); what this suite adds is that it still holds once
/// `action-ids` joins the walk.
#[test]
fn core_still_declares_exactly_the_major_versions_present_under_schemas() {
    let declared: BTreeSet<String> = bindforge_core::supported_majors().into_iter().collect();
    let dirs = format_dirs();
    let present: BTreeSet<String> = dirs.iter().map(|dir| dir.major.clone()).collect();
    let action_ids_majors: BTreeSet<String> = dirs
        .iter()
        .filter(|dir| dir.format == "action-ids")
        .map(|dir| dir.major.clone())
        .collect();

    assert!(
        !action_ids_majors.is_empty(),
        "the walk must find at least one `schemas/action-ids/v*/` directory"
    );
    assert_eq!(
        declared, present,
        "bindforge-core's declared majors must equal the majors present under `schemas/*/v*/`"
    );
    for major in &action_ids_majors {
        assert!(
            declared.contains(major),
            "`action-ids/{major}` is walked but not declared supported"
        );
    }
}

/// proves: Test strategy #6 — the shipped registry loads: exactly the 40 ids
/// Chosen design enumerates, each listed once, in the loaded model. This is the
/// enumeration the ADR leaves unwritten; the TOML is the only id list, so this
/// list moving means the registry moved, which is a reviewed PR on a
/// compatibility promise.
#[test]
fn the_shipped_registry_loads_with_the_enumerated_id_set() {
    let registry = load_shipped();

    let mut found = shipped_id_strings(&registry);
    assert_eq!(
        found.len(),
        SHIPPED_IDS.len(),
        "the shipped registry must load exactly {} ids; it loaded {}",
        SHIPPED_IDS.len(),
        found.len()
    );

    let with_duplicates = found.len();
    found.sort();
    found.dedup();
    assert_eq!(
        found.len(),
        with_duplicates,
        "an id appears once across the three arrays; the loaded registry repeats one"
    );

    let mut expected: Vec<String> = SHIPPED_IDS.iter().map(|id| (*id).to_string()).collect();
    expected.sort();
    assert_eq!(
        found, expected,
        "the shipped registry must hold exactly the ids Chosen design enumerates"
    );
}

/// proves: Test strategy #6 + #2 — every shipped id is grammar-valid and answers
/// from the registry that lists it: `contains` is true and `axis_kind` is
/// present, because the array an id sits in *is* its axis kind.
#[test]
fn every_shipped_id_is_grammar_valid_and_graded_by_the_registry() {
    let registry = load_shipped();

    for id in SHIPPED_IDS {
        let parsed = ActionId::parse(id)
            .unwrap_or_else(|err| panic!("the shipped id `{id}` must be grammar-valid: {err}"));
        assert!(
            registry.contains(&parsed),
            "the loaded registry must contain `{id}`"
        );
        assert!(
            registry.axis_kind(&parsed).is_some(),
            "`{id}` sits in an axis array, so it must report an axis kind"
        );
    }
}

/// proves: Test strategy #6 — the 40 ids sit over the seven namespaces with the
/// counts Chosen design states, and no namespace outside them. Namespaces are a
/// naming convention, not a closed set, so a new one would be a deliberate
/// change to the enumeration rather than a grammar change.
#[test]
fn the_shipped_registry_covers_the_seven_namespaces_with_their_counts() {
    let registry = load_shipped();

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for id in shipped_id_strings(&registry) {
        let namespace = id.split('.').next().unwrap_or_default().to_string();
        *counts.entry(namespace).or_insert(0) += 1;
    }

    let expected: BTreeMap<String, usize> = NAMESPACE_COUNTS
        .iter()
        .map(|(namespace, count)| ((*namespace).to_string(), *count))
        .collect();
    assert_eq!(
        counts, expected,
        "the shipped ids must be the seven namespaces with the counts Chosen design states"
    );
}

/// proves: Test strategy #2 — over the shipped registry every id sits in exactly
/// one axis array, and `axis_kind` returns the kind of the array that lists it:
/// the three arrays are one id set, so a re-grade or a doubled array fails here.
#[test]
fn the_shipped_registry_grades_every_id_in_exactly_one_axis_array() {
    let text = shipped_registry_text();
    let registry = load_shipped();

    let mut graded: BTreeSet<String> = BTreeSet::new();
    for (spelling, kind) in [
        ("discrete-lateral", AxisKind::DiscreteLateral),
        ("continuous-lateral", AxisKind::ContinuousLateral),
        ("none", AxisKind::None),
    ] {
        let ids = string_array(&text, spelling).unwrap_or_else(|| {
            panic!(
                "the shipped registry must declare the `{spelling}` array; membership and axis \
                 kind are one fact"
            )
        });

        for id in &ids {
            assert!(
                graded.insert(id.clone()),
                "`{id}` is listed in more than one axis array; an id appears once across the \
                 three arrays"
            );
            let parsed = ActionId::parse(id)
                .unwrap_or_else(|err| panic!("the shipped id `{id}` must be grammar-valid: {err}"));
            assert_eq!(
                registry.axis_kind(&parsed),
                Some(kind),
                "an id in the shipped `{spelling}` array must report `{spelling}`"
            );
        }
    }

    let loaded: BTreeSet<String> = shipped_id_strings(&registry).into_iter().collect();
    assert_eq!(
        graded, loaded,
        "the three arrays are the id set: an id in the loaded registry is missing from them, or \
         an id in them is missing from the loaded registry"
    );
}

/// proves: Test strategy #2 — the `discrete-lateral` set is pinned to exactly
/// the four left/right endpoints, so a re-grade that widens or narrows what
/// ADR-015 may flip fails the gate.
#[test]
fn the_shipped_discrete_lateral_set_is_exactly_the_four_lateral_endpoints() {
    let registry = load_shipped();

    let discrete: BTreeSet<String> = registry
        .ids()
        .iter()
        .filter(|id| registry.axis_kind(id) == Some(AxisKind::DiscreteLateral))
        .map(|id| id.as_str().to_string())
        .collect();
    let expected: BTreeSet<String> = [
        "move.steer-left",
        "move.steer-right",
        "move.strafe-left",
        "move.strafe-right",
    ]
    .into_iter()
    .map(|id| id.to_string())
    .collect();

    assert_eq!(
        discrete, expected,
        "the registry pins `discrete-lateral` to the four separately-bound endpoints: 015 flips \
         polarity inside this set and nowhere else"
    );
}

/// proves: Test strategy #2 + Axis kind (classification rule) — the ids the
/// classification rule calls out keep the kind the rule gives them: steering is
/// graded by the shape of the binding, not the device, camera look is one graded
/// lateral control, and forward/back is not lateral. This is the grading the
/// research flags as silently consequential.
#[test]
fn the_consequential_ids_keep_the_axis_kind_the_classification_rule_gives_them() {
    let registry = load_shipped();

    for (id, kind) in [
        ("move.steer-left", AxisKind::DiscreteLateral),
        ("move.steer-right", AxisKind::DiscreteLateral),
        ("aim.camera", AxisKind::ContinuousLateral),
        ("move.forward", AxisKind::None),
        ("move.backward", AxisKind::None),
    ] {
        let parsed = ActionId::parse(id)
            .unwrap_or_else(|err| panic!("the shipped id `{id}` must be grammar-valid: {err}"));
        assert_eq!(
            registry.axis_kind(&parsed),
            Some(kind),
            "`{id}` must report the axis kind the classification rule states for it"
        );
    }
}

/// proves: Test strategy #6 — the registry is the copy shipped in the versioned
/// directory: a `v1` directory with no `registry.toml` is refused, and the
/// refusal keeps the local filesystem path out of the message a hub or CLI would
/// show whoever supplied the document.
#[test]
fn the_loader_refuses_a_version_directory_without_a_registry_file() {
    let scratch = ScratchDir::new("action-ids-missing-registry");
    scratch.make_dir("v1");

    let err = bindforge_core::load_action_registry(&scratch.path().join("v1"))
        .expect_err("a version directory with no `registry.toml` is not a registry");
    let detail = err.to_string();
    let path_text = scratch.path().display().to_string();

    assert!(
        !detail.is_empty() && !detail.contains('\n'),
        "the refusal must be a single line saying what is wrong: {detail:?}"
    );
    assert!(
        !detail.contains(path_text.as_str()),
        "a refusal must not disclose a local filesystem path: {detail}"
    );
}

/// proves: Test strategy #6 + Stability — only the `registry.toml` shipped in the
/// versioned directory is the registry: a file of that name holding another
/// format's document is refused, not read as data for `axis_kind`, and the
/// refusal names no local path.
#[test]
fn the_loader_refuses_a_registry_file_holding_another_formats_document() {
    let scratch = ScratchDir::new("action-ids-wrong-format");
    scratch.write(
        "v1/registry.toml",
        "format = \"profile\"\nversion = \"v1\"\nid = \"not-a-registry\"\nderives = []\n",
    );

    let err = bindforge_core::load_action_registry(&scratch.path().join("v1"))
        .expect_err("`registry.toml` holding a `profile` document is not the action registry");
    let detail = err.to_string();
    let path_text = scratch.path().display().to_string();

    assert!(
        !detail.contains('\n'),
        "the refusal must stay one line: {detail:?}"
    );
    assert!(
        !detail.contains(path_text.as_str()),
        "a refusal must not disclose a local filesystem path: {detail}"
    );
}

/// proves: Chosen design (The model, loader) + Adversarial guard — the size
/// budget binds before the registry is read: an oversized `registry.toml` is
/// refused as `DocumentTooLarge` rather than loaded into memory, the same guard
/// `reach_tables.rs` pins for the reach loader.
#[test]
fn the_loader_refuses_a_registry_larger_than_the_parse_budget() {
    let budget = bindforge_core::MAX_DOCUMENT_BYTES;
    let mut oversized = shipped_registry_text();
    oversized.push_str(&"#".repeat(budget + 1 - oversized.len()));

    let scratch = ScratchDir::new("action-ids-over-budget");
    scratch.write("v1/registry.toml", &oversized);

    let err = bindforge_core::load_action_registry(&scratch.path().join("v1"))
        .expect_err("a `registry.toml` larger than MAX_DOCUMENT_BYTES must be refused, not loaded");
    assert!(
        matches!(err, CoreError::DocumentTooLarge { .. }),
        "expected the `DocumentTooLarge` refusal for an oversized registry file, got {err:?}"
    );
}
