//! The version gate of the `schemas/` formats contract.
//!
//! proves: Test strategy #2 — a document declaring an unsupported major
//! version, or declaring no version at all, is rejected with an error naming
//! the supported majors; and the majors `bindforge-core` declares equal the
//! majors present under `schemas/*/v*/`.
//!
//! The rule is canonical in the proposal (Chosen design → Formats contract)
//! and mirrored in `schemas/README.md`. A document defaulted to v1 when it
//! declares no version fails open — a document written against a future major
//! parsed with v1 semantics — which is exactly what versioning exists to
//! prevent.
//!
//! The gate is per-format, because a major exists per format: `reach-classes`
//! carries `v1` and `v2` (Decision One: the key→class mapping rides a new major)
//! and no other shipped format carries anything else, so a `profile`,
//! `action-ids` or `keyboard-geometry` document declaring `v2` names a major that
//! format never shipped and is refused. A gate reading one crate-wide list
//! accepts those documents the moment any one format gains a major — the same
//! fail-open, arriving from the other side. `bindforge-core`'s `supported_majors`
//! stays the union across formats, which is what the majors-equality case below
//! compares against the tree.

mod common;

use std::collections::BTreeSet;

use bindforge_core::CoreError;
use common::{
    declares_version, fixture_text, fixtures, format_dirs, format_majors, profile_doc_declaring,
    version_dir,
};

/// A well-formed `action-ids` document: exactly the entries
/// `schemas/action-ids/v1/spec.md` defines, in the shape its valid corpus
/// fixtures ship, so the version line is the document's only fault.
const ACTION_IDS_DOC: &str = r#"format = "action-ids"
version = "v1"
discrete-lateral = ["move.steer-left", "move.steer-right"]
continuous-lateral = ["aim.camera"]
none = ["move.forward", "combat.fire"]
"#;

/// A well-formed `keyboard-geometry` document: exactly the entries
/// `schemas/keyboard-geometry/v1/spec.md` defines.
const GEOMETRY_DOC: &str = r#"format = "keyboard-geometry"
version = "v1"
geometry = "dvorak-us"
keys = ["keyboard:a 0 2"]
"#;

/// The same document with a different `version` entry, so a case is about the
/// declared major and nothing else.
fn declaring(document: &str, version: &str) -> String {
    document.replace("version = \"v1\"", &format!("version = \"{version}\""))
}

/// The proposal pins the rejection's *shape* — "a structured error naming the
/// supported majors" — not its spelling; `CoreError::UnsupportedVersion` is
/// the name this suite pins for that variant of core's API.
///
/// `supported` is that *format's* majors, taken from the tree: naming
/// `bindforge-core`'s union instead would make a refusal for a `profile` document
/// advertise a `v2` no `profile` grammar exists for.
fn assert_version_rejection(err: CoreError, subject: &str, supported: &[String]) {
    assert!(
        matches!(err, CoreError::UnsupportedVersion { .. }),
        "expected the version gate's structured error for {subject}, got {err:?}"
    );

    let message = err.to_string();
    for major in supported {
        assert!(
            message.contains(major.as_str()),
            "the rejection for {subject} must name supported major `{major}`: `{message}`"
        );
    }
}

/// proves: Test strategy #2 — a document declaring a major `bindforge-core`
/// does not support is rejected, and the error names the supported majors.
#[test]
fn rejects_a_document_declaring_an_unsupported_major_naming_the_supported_majors() {
    let document = profile_doc_declaring("unsupported-major", Some("v99"), &[]);

    let err = bindforge_core::parse(&document)
        .expect_err("a profile declaring `version = \"v99\"` must be rejected, not parsed");
    assert_version_rejection(err, "the v99 profile document", &format_majors("profile"));
}

/// proves: Test strategy #2 — a document that declares no version at all is
/// rejected with the same structured error, never defaulted to the format its
/// fixture happens to sit under.
#[test]
fn rejects_a_document_that_declares_no_version_naming_the_supported_majors() {
    let document = profile_doc_declaring("missing-version", None, &[]);

    let err = bindforge_core::parse(&document)
        .expect_err("a profile with no `version` entry must be rejected rather than read as v1");
    assert_version_rejection(
        err,
        "the versionless profile document",
        &format_majors("profile"),
    );
}

/// proves: Test strategy #2 + A2 — the gate is per-format, so a major one format
/// ships does not widen what another format accepts. `reach-classes` is the only
/// format carrying a `v2`, so a `profile`, `action-ids` or `keyboard-geometry`
/// document declaring `v2` names a major its format never shipped: it is refused
/// as the version gate's structured error naming that format's own supported
/// majors, never read with v1 semantics. The `keyboard-geometry` case is the one
/// the format's own guard already refuses; it stays pinned here because a
/// per-format gate subsumes that guard, and the refusal must survive the
/// subsumption.
#[test]
fn rejects_a_document_declaring_a_major_that_only_another_format_ships() {
    let documents = [
        (
            "profile",
            profile_doc_declaring("future-major", Some("v2"), &[]),
        ),
        ("action-ids", declaring(ACTION_IDS_DOC, "v2")),
        ("keyboard-geometry", declaring(GEOMETRY_DOC, "v2")),
    ];

    for (format, document) in documents {
        let supported = format_majors(format);
        assert!(
            !supported.is_empty(),
            "the walk must find `{format}` in the contract tree to name its majors"
        );
        assert!(
            !supported.iter().any(|major| major == "v2"),
            "this case needs `{format}` to ship no `v2`; the tree carries {supported:?}"
        );

        let Err(err) = bindforge_core::parse(&document) else {
            panic!("a `{format}` document declaring `v2` must be refused, not parsed");
        };
        assert_version_rejection(err, format, &supported);
    }
}

/// proves: Test strategy #2 — the corpus carries that case too: the proposal
/// puts a missing-version fixture under `schemas/profile/v1/fixtures/invalid/`,
/// which is what makes the rule part of the language-neutral contract rather
/// than a Rust-only behavior.
#[test]
fn the_invalid_corpus_ships_a_missing_version_fixture() {
    let profile_v1 = version_dir("profile", "v1");

    let missing_version = fixtures(&profile_v1, "invalid").into_iter().any(|path| {
        let text = fixture_text(&path);
        !declares_version(&text)
            && matches!(
                bindforge_core::parse(&text),
                Err(CoreError::UnsupportedVersion { .. })
            )
    });

    assert!(
        missing_version,
        "{}'s fixtures/invalid/ must contain a document that declares no version and is \
         rejected as a version error",
        profile_v1.path.display()
    );
}

/// proves: Test strategy #2 — the corpus carries the per-format half of the gate
/// as well: `schemas/profile/v1/fixtures/invalid/major-not-shipped.toml` declares
/// a major no `profile` directory ships, so a third-party reader is held to the
/// same refusal `version_gate.rs` pins for `bindforge-core`, and the refusal names
/// the majors *that format* supports.
#[test]
fn the_invalid_corpus_ships_a_fixture_for_a_major_the_format_does_not_ship() {
    let profile_v1 = version_dir("profile", "v1");
    let path = profile_v1
        .path
        .join("fixtures")
        .join("invalid")
        .join("major-not-shipped.toml");

    assert!(
        path.is_file(),
        "profile/v1 must ship `fixtures/invalid/major-not-shipped.toml`: the per-format gate is \
         part of the language-neutral contract, and the invalid-fixture corpus is the contract for \
         it (missing at {})",
        path.display()
    );
    let Err(err) = bindforge_core::parse(&fixture_text(&path)) else {
        panic!(
            "{} is an invalid fixture and must be refused",
            path.display()
        );
    };
    let supported = format_majors("profile");
    assert_version_rejection(err, "the profile major-not-shipped fixture", &supported);
}

/// proves: Test strategy #2 — the majors `bindforge-core` declares equal the
/// majors present under `schemas/*/v*/`, so the compiled list and the fixture
/// tree cannot drift into two different definitions of "supported".
#[test]
fn core_declares_exactly_the_major_versions_present_under_schemas() {
    let declared: BTreeSet<String> = bindforge_core::supported_majors().into_iter().collect();
    let present: BTreeSet<String> = format_dirs().into_iter().map(|dir| dir.major).collect();

    assert!(
        !present.is_empty(),
        "the walk found no `schemas/*/v*/` version directory to compare against"
    );
    assert_eq!(
        declared, present,
        "bindforge-core's declared majors must equal the majors present under `schemas/*/v*/`"
    );
}
