//! The action taxonomy's typed model in `crates/core`: the `ActionId` grammar,
//! the closed `AxisKind` set, the `action-ids` entry set, and the version gate
//! for the new format.
//!
//! proves: Test strategy #1 — the id grammar, one case per rule; #2 — axis kind
//! is total; #3 — the closed entry set for the format; #4 — the version gate for
//! the new format.
//!
//! The canonical statement of every rule here is `schemas/action-ids/v1/spec.md`
//! (Chosen design): the grammar is the DNS-label shape of RFC 1035 §2.3.1
//! restricted to lowercase and **exact** — an uppercase segment is a rejection,
//! never a canonicalization — an id is exactly two segments, an id appears once
//! across the three axis arrays, and any entry outside the entry table is a typed
//! refusal naming the entry.
//!
//! The version gate is per-format: `action-ids` ships one major, so a refusal
//! for an `action-ids` document names the majors the `action-ids` corpus carries
//! rather than `bindforge-core`'s crate-wide union. `reach-classes` gained a
//! `v2` (ADR-012 Decision One); that does not give `action-ids` a `v2` grammar,
//! and reading a `v2`-declaring registry with the v1 entry set is the fail-open
//! the gate exists to prevent.
//!
//! Every refusal is expected to be an existing `CoreError` variant — the model
//! admits no new one (`InvalidValue` for a malformed id, a duplicate, an
//! out-of-set entry or a version that disagrees with its directory; `MissingEntry`
//! for an absent axis array) — in the single-line, truncating, path-free message
//! the contract already promises, with the offending id echoed escaped to
//! printable characters.
//!
//! This suite is unit-level: pure functions and scratch directories, no network
//! (ADR-004, repo practice). The legs that read the contract tree itself — the
//! shipped registry, the fixture corpus, the harness walk — are in
//! `action_ids.rs`. These tests are written from the proposal while
//! `crates/core/src/action.rs` does not exist yet, so they fail until the design
//! is implemented; that failure is the point, and none of them may be weakened to
//! make anything pass.

mod common;

use bindforge_core::{ActionId, ActionRegistry, AxisKind, CoreError};
use common::{ScratchDir, format_majors};

/// The three axis-kind spellings, which are exactly the three array names the
/// entry table defines (Chosen design: the arrays *are* the id set).
const AXIS_SPELLINGS: [(&str, AxisKind); 3] = [
    ("discrete-lateral", AxisKind::DiscreteLateral),
    ("continuous-lateral", AxisKind::ContinuousLateral),
    ("none", AxisKind::None),
];

/// An `action-ids` document using exactly the entries `spec.md` defines:
/// `format`, `version`, and the three axis arrays.
const VALID_DOC: &str = r#"format = "action-ids"
version = "v1"
discrete-lateral = ["move.steer-left", "move.strafe-left"]
continuous-lateral = ["aim.camera", "aim.look"]
none = ["move.forward", "combat.fire", "item.slot-6"]
"#;

/// A small `action-ids` document the axis-kind rules can be read against, two ids
/// per array so a scratch registry is as total as the shipped one.
const SMALL_DOC: &str = r#"format = "action-ids"
version = "v1"
discrete-lateral = ["move.strafe-left", "move.strafe-right"]
continuous-lateral = ["aim.camera", "aim.look"]
none = ["move.forward", "combat.fire"]
"#;

/// One `entry = value` line's worth of an `action-ids` document, so each case
/// below spells out the entries its rule is about. This is not a second TOML
/// reader — the same line `common` draws for its textual helpers.
fn entry<'a>(name: &'a str, value: &str) -> (&'a str, String) {
    (name, format!("{name} = {value}"))
}

/// The `action-ids` document made of these entries, one per line, in the order
/// they are given: the arrays are written in document order, which is the order
/// `serialize` writes them back in.
fn action_ids_doc(entries: &[(&str, String)]) -> String {
    entries
        .iter()
        .map(|(_, line)| format!("{line}\n"))
        .collect()
}

/// A one-line string array: the shape the v1 grammar admits, which is also the
/// shape `registry.toml` ships every axis array in.
fn array(ids: &[&str]) -> String {
    let items = ids
        .iter()
        .map(|id| format!("\"{id}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}

/// The five entries the entry table defines, each with a one-id array, so a case
/// edits exactly one of them.
fn complete_entries() -> Vec<(&'static str, String)> {
    vec![
        entry("format", "\"action-ids\""),
        entry("version", "\"v1\""),
        entry("discrete-lateral", &array(&["move.steer-left"])),
        entry("continuous-lateral", &array(&["aim.camera"])),
        entry("none", &array(&["move.forward"])),
    ]
}

/// An `action-ids` document carrying the defined five entries, with `name` set to
/// `value`: an entry the grammar defines is replaced, and one it does not define
/// is added — which keeps the unknown-entry cases one-fault-each.
fn doc_with_entry(name: &str, value: &str) -> String {
    let mut entries: Vec<(&str, String)> = complete_entries();
    let mut defined = false;
    for (entry_name, line) in entries.iter_mut() {
        if *entry_name == name {
            *line = format!("{name} = {value}");
            defined = true;
        }
    }
    if !defined {
        entries.push((name, format!("{name} = {value}")));
    }
    action_ids_doc(&entries)
}

/// An `action-ids` document carrying the three axis arrays except `missing`.
fn doc_missing_axis_array(missing: &str) -> String {
    let mut entries = vec![
        entry("format", "\"action-ids\""),
        entry("version", "\"v1\""),
    ];
    for (name, ids) in [
        ("discrete-lateral", vec!["move.steer-left"]),
        ("continuous-lateral", vec!["aim.camera"]),
        ("none", vec!["move.forward"]),
    ] {
        if name != missing {
            entries.push(entry(name, &array(&ids)));
        }
    }
    action_ids_doc(&entries)
}

/// Loads `doc` through `load_action_registry` from a scratch directory named
/// `v1` — the directory shape the loader's version check compares against.
fn registry_from(doc: &str, label: &str) -> ActionRegistry {
    let scratch = ScratchDir::new(label);
    scratch.write("v1/registry.toml", doc);
    bindforge_core::load_action_registry(&scratch.path().join("v1"))
        .unwrap_or_else(|err| panic!("a document the grammar defines must load: {err}"))
}

/// The version gate's shape, as `version_gate.rs` pins it: a structured error
/// naming the majors *this format* supports. `action-ids` ships `v1` only, so a
/// refusal for an `action-ids` document names `v1`; naming the crate-wide union
/// instead would advertise a `v2` no `action-ids` grammar exists for.
fn assert_names_the_supported_majors(err: &CoreError, document: &str, supported: &[String]) {
    assert!(
        matches!(err, CoreError::UnsupportedVersion { .. }),
        "expected the version gate's structured error for {document}, got {err:?}"
    );

    let message = err.to_string();
    for major in supported {
        assert!(
            message.contains(major.as_str()),
            "the rejection for {document} must name supported major `{major}`: `{message}`"
        );
    }
}

/// proves: Test strategy #1 — the grammar accepts the ids the design names as
/// well-formed, up to the shared `MAX_SLUG_OCTETS` segment bound that design 024
/// folds into the one exported constant (128 octets, up from the 63 the RFC 1035
/// §2.3.1 sentence used to state),
/// and returns each one byte-exact: comparison is exact, so there is no case
/// folding and no canonicalization to undo.
#[test]
fn accepts_a_well_formed_two_segment_id() {
    let long_id = format!("move.{}", "a".repeat(bindforge_core::MAX_SLUG_OCTETS));

    for id in [
        "move.strafe-left",
        "item.slot-6",
        "aim.cycle-target",
        long_id.as_str(),
    ] {
        let parsed =
            ActionId::parse(id).unwrap_or_else(|err| panic!("`{id}` is well-formed: {err}"));
        assert_eq!(
            parsed.as_str(),
            id,
            "`as_str()` must return the id exactly as written — a canonicalized id would let two \
             spellings name one action, which is the churn ADR-011 forbids"
        );
    }
}

/// proves: Test strategy #1 — one case per rejection rule: an uppercase segment,
/// a non-ASCII letter, a leading digit, a leading or trailing hyphen, an
/// underscore, an empty segment, a single-segment id, a three-segment id, a
/// segment one past the shared bound, and the empty string. Each is a rejection,
/// never a
/// canonicalization, and the refusal is the model's `InvalidValue`.
#[test]
fn rejects_an_id_that_breaks_the_segment_grammar() {
    let long_id = format!("move.{}", "a".repeat(bindforge_core::MAX_SLUG_OCTETS + 1));

    for (id, fault) in [
        ("Move.Forward", "an uppercase first segment"),
        ("move.Steer-left", "an uppercase second segment"),
        ("möve.forward", "a non-ASCII letter in the first segment"),
        ("move.steér", "a non-ASCII letter in the second segment"),
        ("move.1step", "a segment opening with a digit"),
        ("1move.forward", "a segment opening with a digit"),
        ("move.-left", "a segment opening with a hyphen"),
        ("move.left-", "a segment closing with a hyphen"),
        ("move.strafe_left", "an underscore"),
        ("move..steer", "an empty segment"),
        ("move.", "an empty second segment"),
        (".forward", "an empty first segment"),
        (
            "steer-left",
            "a single-segment id: the grammar asks for exactly two segments, so an unprefixed \
             name is malformed rather than an unregistered one",
        ),
        ("move.steer.left", "a three-segment id"),
        ("", "the empty string"),
        (
            long_id.as_str(),
            "a segment one past the shared `MAX_SLUG_OCTETS` bound",
        ),
    ] {
        let err = ActionId::parse(id).expect_err(&format!("`{id}` — {fault} — must be refused"));
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "The model refuses a malformed id as `InvalidValue`; no new error variant enters the \
             contract, got {err:?}"
        );
    }
}

/// proves: design 024's Test strategy #6 + Chosen design (Shared grammar and bound)
/// — a segment-shape refusal that states the bound states it from the exported
/// constant: the number in the message is `MAX_SLUG_OCTETS` (128 from ADR-024), and
/// the pre-amendment `63 octets` sentence is gone, so a future amendment has one
/// source rather than a stale literal in a refusal.
#[test]
fn a_segment_bound_refusal_names_the_bound_it_interpolates_from_the_constant() {
    let long_id = format!("move.{}", "a".repeat(bindforge_core::MAX_SLUG_OCTETS + 1));
    let err = ActionId::parse(&long_id)
        .expect_err("a segment one past MAX_SLUG_OCTETS must be refused at the bound");
    let message = err.to_string();

    assert!(
        message.contains(&bindforge_core::MAX_SLUG_OCTETS.to_string()),
        "the refusal must name the shared bound ({}) it enforces, got `{message}`",
        bindforge_core::MAX_SLUG_OCTETS
    );
    assert!(
        !message.contains("63 octets"),
        "the refusal must not carry the superseded 63-octet literal, got `{message}`"
    );
}

/// proves: Test strategy #2 — axis kind is total: `name()` spells the three
/// registry arrays and `from_name()` accepts exactly those three spellings,
/// mirroring the closed-set `from_name` of `Hand`. A fourth spelling would be a
/// fourth axis kind the taxonomy does not have.
#[test]
fn the_axis_kind_spellings_are_exactly_the_three_registry_arrays() {
    for (spelling, kind) in AXIS_SPELLINGS {
        assert_eq!(
            kind.name(),
            spelling,
            "an axis kind must name the registry array its ids come from"
        );
        assert_eq!(
            AxisKind::from_name(spelling),
            Some(kind),
            "`{spelling}` is an array name the entry table defines, so it must map to its kind"
        );
    }

    for spelling in [
        "",
        "lateral",
        "discrete",
        "DiscreteLateral",
        "emphasis",
        "none-elsewhere",
    ] {
        assert!(
            AxisKind::from_name(spelling).is_none(),
            "`{spelling}` is not one of the three spellings, so the closed set must refuse it"
        );
    }
}

/// proves: Test strategy #2 — an id's axis kind is the array it sits in, and the
/// registry's closed set is what answers: a grammar-valid id the registry does not
/// list is simply not present, so `axis_kind` is `None` rather than a default. A
/// typo'd id passing through as a valid binding that matches nothing is the
/// failure mode Options considered refuses.
#[test]
fn axis_kind_answers_from_the_array_an_id_sits_in() {
    let registry = registry_from(SMALL_DOC, "action-axis-kind");

    for (spelling, kind, ids) in [
        (
            "discrete-lateral",
            AxisKind::DiscreteLateral,
            ["move.strafe-left", "move.strafe-right"],
        ),
        (
            "continuous-lateral",
            AxisKind::ContinuousLateral,
            ["aim.camera", "aim.look"],
        ),
        ("none", AxisKind::None, ["move.forward", "combat.fire"]),
    ] {
        for raw in ids {
            let id = ActionId::parse(raw)
                .unwrap_or_else(|err| panic!("`{raw}` is well-formed and must parse: {err}"));
            assert!(
                registry.contains(&id),
                "the registry must contain `{raw}`, which its document lists"
            );
            assert_eq!(
                registry.axis_kind(&id),
                Some(kind),
                "an id in the `{spelling}` array must report `{spelling}`: membership and axis \
                 kind are one fact"
            );
        }
    }

    let unregistered = ActionId::parse("move.steer-up").expect("`move.steer-up` is grammar-valid");
    assert!(
        !registry.contains(&unregistered),
        "an id outside the registry's closed set is not present"
    );
    assert_eq!(
        registry.axis_kind(&unregistered),
        None,
        "an unregistered id has no axis kind — no default, no guess"
    );
}

/// proves: Test strategy #3 + Formats contract (closed entry set) — closing the
/// set is not shrinking the grammar: a document carrying exactly the entries the
/// entry table defines parses.
#[test]
fn accepts_every_entry_the_action_ids_grammar_defines() {
    if let Err(err) = bindforge_core::parse(VALID_DOC) {
        panic!(
            "an `action-ids` document using only the entries `spec.md` defines must parse; it \
             was refused: {err}"
        );
    }
}

/// proves: Test strategy #3 + Chosen design (entry table) — an entry outside the
/// entry table is refused naming the entry, never ignored: neither an `emphasis`
/// — ADR-011 says emphasis is *not* in the taxonomy — nor a fourth axis array,
/// which would invent an axis kind the taxonomy does not have.
#[test]
fn refuses_an_action_ids_document_carrying_an_entry_outside_the_grammar() {
    let emphasis = doc_with_entry("emphasis", &array(&["combat.melee"]));
    let fourth_axis = doc_with_entry("vertical", &array(&["move.up"]));

    for (document, name) in [
        (emphasis.as_str(), "emphasis"),
        (fourth_axis.as_str(), "vertical"),
    ] {
        let err = bindforge_core::parse(document).expect_err(&format!(
            "`{name}` is not an entry the `action-ids` grammar defines"
        ));
        let detail = err.to_string();
        assert!(
            detail.contains(name),
            "the refusal must name the entry it refused, got `{detail}`"
        );
    }
}

/// proves: Test strategy #3 — the three axis arrays are all required: an absent
/// one is `MissingEntry` naming the entry, not an empty array read in its place.
/// Defaulting an array open would let a document drop half the id set unnoticed.
#[test]
fn refuses_an_action_ids_document_missing_an_axis_array_naming_the_entry() {
    for missing in ["discrete-lateral", "continuous-lateral", "none"] {
        let document = doc_missing_axis_array(missing);

        let err = bindforge_core::parse(&document).expect_err(&format!(
            "a document with no `{missing}` array must be refused"
        ));
        assert!(
            matches!(err, CoreError::MissingEntry { .. }),
            "an absent axis array is `MissingEntry` (The model, error rule), got {err:?}"
        );
        let detail = err.to_string();
        assert!(
            detail.contains(missing),
            "the refusal must name the entry it found missing, got `{detail}`"
        );
    }
}

/// proves: Test strategy #3 — an axis array is an array of strings: a non-string
/// element is refused, exactly as `non-string-element.toml` is an invalid fixture
/// for the format.
#[test]
fn refuses_a_non_string_element_in_an_axis_array() {
    let document = doc_with_entry("none", r#"["move.forward", 7]"#);

    bindforge_core::parse(&document)
        .expect_err("an axis array holds ids; a number is not an id and must be refused");
}

/// proves: Test strategy #3 + Chosen design (Uniqueness) — an id appears once
/// across the three arrays: a repeat inside one array is a refusal naming the id.
#[test]
fn refuses_a_duplicate_id_within_one_axis_array_naming_the_id() {
    let document = doc_with_entry(
        "none",
        &array(&["move.forward", "aim.sights", "move.forward"]),
    );

    let err = bindforge_core::parse(&document)
        .expect_err("`move.forward` is listed twice; an id appears once across the three arrays");
    let detail = err.to_string();
    assert!(
        detail.contains("move.forward"),
        "the refusal must name the duplicated id, got `{detail}`"
    );
}

/// proves: Test strategy #3 + Chosen design (Uniqueness) — the rule spans the
/// arrays: one id graded two ways is refused naming the id, so an id cannot sit in
/// two axis kinds and quietly widen what ADR-015 may flip.
#[test]
fn refuses_a_duplicate_id_across_two_axis_arrays_naming_the_id() {
    let document = action_ids_doc(&[
        entry("format", "\"action-ids\""),
        entry("version", "\"v1\""),
        entry(
            "discrete-lateral",
            &array(&["move.steer-left", "aim.camera"]),
        ),
        entry("continuous-lateral", &array(&["aim.camera"])),
        entry("none", &array(&[])),
    ]);

    let err = bindforge_core::parse(&document)
        .expect_err("`aim.camera` is listed in two arrays; the three arrays are one id set");
    let detail = err.to_string();
    assert!(
        detail.contains("aim.camera"),
        "the refusal must name the duplicated id, got `{detail}`"
    );
}

/// proves: Test strategy #3 + The model (error rule) — the entry-set check also
/// grammar-checks the ids it lists: a malformed id inside an axis array is
/// `InvalidValue`, echoed as written rather than repaired.
#[test]
fn refuses_a_registry_document_holding_a_malformed_id_naming_the_id() {
    let document = doc_with_entry("none", &array(&["Move.Forward"]));

    let err = bindforge_core::parse(&document)
        .expect_err("`Move.Forward` is not grammar-valid, so the document is invalid");
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "a malformed id is `InvalidValue`, got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains("Move.Forward"),
        "the refusal must echo the offending id, got `{detail}`"
    );
}

/// proves: The model (error rule) + Adversarial guard — the bound half of the
/// rejection promise: the echoed id is truncated, so one hostile registry cannot
/// inflate log volume by pasting 4 KiB of id into it.
#[test]
fn a_refusal_for_a_malformed_id_truncates_the_id_it_echoes() {
    let tail = "Ztail-of-the-echoed-id";
    let malformed = format!("move.{}{tail}", "x".repeat(4096));
    let document = doc_with_entry("none", &array(&[malformed.as_str()]));

    let err = bindforge_core::parse(&document)
        .expect_err("an id carrying an uppercase letter is malformed at any length");
    let detail = err.to_string();
    assert!(
        !detail.contains(tail),
        "the refusal must truncate the id it echoes rather than paste 4 KiB of it into the log: \
         {detail}"
    );
    assert!(
        !detail.contains('\n') && !detail.contains('\r'),
        "the rejection must stay one line: {detail:?}"
    );
}

/// proves: The model (error rule) + Adversarial guard — the escaped half of the
/// same promise: an offending id is echoed escaped to printable characters, so a
/// control character inside an id cannot forge a log line.
#[test]
fn a_refusal_for_an_id_escapes_control_characters_rather_than_emitting_them() {
    let document = doc_with_entry("none", &array(&["move.for\\u0007ward"]));

    let err = bindforge_core::parse(&document)
        .expect_err("an id carrying a control character is not grammar-valid");
    let detail = err.to_string();
    assert!(
        !detail.contains('\u{7}') && detail.chars().all(|ch| !ch.is_control()),
        "the rejection must escape control characters rather than emit them: {detail:?}"
    );
    assert!(
        !detail.contains('\n'),
        "the rejection must stay one line: {detail:?}"
    );
}

/// proves: Test strategy #4 — the version gate covers the new format: an
/// `action-ids` document declaring no version is refused with the error naming the
/// supported majors, never defaulted to the `v1` its fixture sits under.
#[test]
fn refuses_an_action_ids_document_declaring_no_version_naming_the_supported_majors() {
    let document = action_ids_doc(&[
        entry("format", "\"action-ids\""),
        entry("discrete-lateral", &array(&[])),
        entry("continuous-lateral", &array(&[])),
        entry("none", &array(&[])),
    ]);

    let err = bindforge_core::parse(&document).expect_err(
        "an `action-ids` document with no `version` entry must be rejected, never defaulted to v1",
    );
    let supported = format_majors("action-ids");
    assert_names_the_supported_majors(&err, "the versionless action-ids document", &supported);
}

/// proves: Test strategy #4 — an `action-ids` document declaring a major
/// `bindforge-core` does not support is refused with the same structured error.
#[test]
fn refuses_an_action_ids_document_declaring_an_unsupported_major() {
    let document = action_ids_doc(&[
        entry("format", "\"action-ids\""),
        entry("version", "\"v99\""),
        entry("discrete-lateral", &array(&[])),
        entry("continuous-lateral", &array(&[])),
        entry("none", &array(&[])),
    ]);

    let err = bindforge_core::parse(&document)
        .expect_err("an `action-ids` document declaring `v99` must be rejected, not parsed");
    let supported = format_majors("action-ids");
    assert_names_the_supported_majors(&err, "the v99 action-ids document", &supported);
}

/// proves: Test strategy #4 + Revision log — the declared `version` must equal
/// the version segment of the directory it is loaded from: that gate is what makes
/// a widened grammar ride `action-ids/v2` instead of breaking every v1 reader. The
/// refusal is `InvalidValue`, one line, and it names no local path.
#[test]
fn refuses_a_registry_whose_declared_version_disagrees_with_its_directory() {
    let scratch = ScratchDir::new("action-ids-version-mismatch");
    scratch.write("v2/registry.toml", VALID_DOC);

    let err = bindforge_core::load_action_registry(&scratch.path().join("v2"))
        .expect_err("a `registry.toml` declaring `v1` cannot be the registry of a `v2` directory");
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "a version that disagrees with its directory is an `InvalidValue` refusal, got {err:?}"
    );

    let detail = err.to_string();
    assert!(
        !detail.contains('\n') && (detail.contains("v1") || detail.contains("v2")),
        "the refusal must stay one line and name the version it refused: {detail}"
    );
    let path_text = scratch.path().display().to_string();
    assert!(
        !detail.contains(path_text.as_str()),
        "a rejection must not disclose a local filesystem path: {detail}"
    );
}
