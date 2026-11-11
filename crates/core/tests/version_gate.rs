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

mod common;

use std::collections::BTreeSet;

use bindforge_core::CoreError;
use common::{
    declares_version, fixture_text, fixtures, format_dirs, profile_doc_declaring, version_dir,
};

/// The proposal pins the rejection's *shape* — "a structured error naming the
/// supported majors" — not its spelling; `CoreError::UnsupportedVersion` is
/// the name this suite pins for that variant of core's API.
fn assert_version_rejection(err: CoreError, document: &str) {
    assert!(
        matches!(err, CoreError::UnsupportedVersion { .. }),
        "expected the version gate's structured error for {document}, got {err:?}"
    );

    let message = err.to_string();
    for major in bindforge_core::supported_majors() {
        assert!(
            message.contains(major.as_str()),
            "the rejection for {document} must name supported major `{major}`: `{message}`"
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
    assert_version_rejection(err, "the v99 profile document");
}

/// proves: Test strategy #2 — a document that declares no version at all is
/// rejected with the same structured error, never defaulted to the format its
/// fixture happens to sit under.
#[test]
fn rejects_a_document_that_declares_no_version_naming_the_supported_majors() {
    let document = profile_doc_declaring("missing-version", None, &[]);

    let err = bindforge_core::parse(&document)
        .expect_err("a profile with no `version` entry must be rejected rather than read as v1");
    assert_version_rejection(err, "the versionless profile document");
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
