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
//! `action-ids`, `keyboard-geometry` or `manifests` document declaring `v2` names a
//! major that format never shipped and is refused. A gate reading one crate-wide
//! list accepts those documents the moment any one format gains a major — the same
//! fail-open, arriving from the other side. `bindforge-core`'s `supported_majors`
//! stays the union across formats, which is what the majors-equality case below
//! compares against the tree: adding `schemas/manifests/v1/` (ADR-013, design 013's
//! Test strategy #3 and #7) grows only the *per-format* list, because the union
//! already carries `v1` — that proposal's A2, and the reason the invariant the last
//! case pins needs no change.
//!
//! Design 015 adds `schemas/policy/v1/` under exactly the same rule: `policy`'s
//! majors are its own (`v1`, and nothing else until a `policy/v2` directory
//! exists — Options considered: the closed kind set), a `policy` document
//! declaring `v2` is refused naming *policy's* majors, and the crate-wide union is
//! unchanged by the new directory, which is why the invariant case at the end of
//! this file still needs no edit of its own (design 015's Test strategy #10, A11).

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

/// A well-formed `manifests` document: exactly the required entries
/// `schemas/manifests/v1/spec.md` defines, in the shape its valid corpus fixtures
/// ship, so the version line is the document's only fault.
const MANIFEST_DOC: &str = r#"format = "manifests"
version = "v1"
id = "remnant-2"
revision = 1
codec = "unreal-engine-5"
actions = ["move.steer-left Turn left"]
"#;

/// A well-formed `keyboard-geometry` document: exactly the entries
/// `schemas/keyboard-geometry/v1/spec.md` defines.
const GEOMETRY_DOC: &str = r#"format = "keyboard-geometry"
version = "v1"
geometry = "dvorak-us"
keys = ["keyboard:a 0 2"]
"#;

/// A well-formed `policy` document: exactly the entries
/// `schemas/policy/v1/spec.md` defines, in the shape its valid corpus fixtures
/// ship, so the version line is the document's only fault (design 015's
/// Test strategy #10).
const POLICY_DOC: &str = r#"format = "policy"
version = "v1"
id = "lefty-stack"
transforms = ["relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k", "swap-mouse-buttons"]
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
/// ships does not widen what another format accepts. `reach-classes` carries a
/// `v2` (design 012), and design 014 grew `profile` to carry one too — so the
/// formats still refusing `v2` here are `action-ids` and `keyboard-geometry`:
/// they name a major their format never shipped, refused as the version gate's
/// structured error naming that format's own supported majors, never read with
/// v1 semantics. The `keyboard-geometry` case is the one the format's own guard
/// already refuses; it stays pinned here because a per-format gate subsumes
/// that guard, and the refusal must survive the subsumption. `profile`'s own
/// unshipped-major case is design 014's `v3`, pinned by the two cases below.
#[test]
fn rejects_a_document_declaring_a_major_that_only_another_format_ships() {
    let documents = [
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

/// proves: Test strategy #2 + design 014's Test strategy #8 — the corpus
/// carries the per-format half of the gate: design 014 grows `profile`'s
/// shipped majors to `v1` and `v2` (A4), so the unshipped-major case now lives
/// in the v2 corpus: `schemas/profile/v2/fixtures/invalid/major-not-shipped.toml`
/// declares a major no `profile` directory ships, and the refusal names the
/// majors *that format* supports — now both of them. The v1 corpus's own
/// `major-not-shipped.toml` stays byte-exact (A15) and stays a refused fixture
/// under the walk; what it pinned for the gate, the v2 fixture pins now.
#[test]
fn the_invalid_corpus_ships_a_fixture_for_a_major_the_format_does_not_ship() {
    let profile_v2 = version_dir("profile", "v2");
    let path = profile_v2
        .path
        .join("fixtures")
        .join("invalid")
        .join("major-not-shipped.toml");

    assert!(
        path.is_file(),
        "profile/v2 must ship `fixtures/invalid/major-not-shipped.toml`: the per-format gate is \
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
    assert!(
        supported.iter().any(|major| major == "v2"),
        "design 014's A4: `profile`'s shipped majors must include `v2` once its directory \
         exists; the tree carries {supported:?}"
    );
    assert_version_rejection(err, "the profile v2 major-not-shipped fixture", &supported);
}

/// proves: design 014's Test strategy #8 + A4 — `profile` now ships two majors,
/// and the gate closes at the top of the series: a `profile` document declaring
/// `v3` is refused as the version gate's structured error naming `profile`'s
/// majors, `v1` and `v2` — and only those. Widening `profile` grows exactly one
/// `format_majors` line; the majors-equality invariant at the end of this file
/// needs no change, which is the case this one keeps green.
#[test]
fn rejects_a_profile_document_declaring_v3_naming_profiles_shipped_majors() {
    let document = profile_doc_declaring("profile-v3", Some("v3"), &[]);
    let supported = format_majors("profile");

    assert!(
        !supported.iter().any(|major| major == "v3"),
        "this case needs `profile` to ship no `v3`; the tree carries {supported:?}"
    );

    let Err(err) = bindforge_core::parse(&document) else {
        panic!("a profile document declaring `version = \"v3\"` must be refused, not parsed");
    };
    assert!(
        matches!(err, CoreError::UnsupportedVersion { .. }),
        "expected the version gate's structured error, got {err:?}"
    );
    let message = err.to_string();
    for major in ["v1", "v2"] {
        assert!(
            message.contains(major),
            "design 014 grows `profile`'s majors to v1 and v2 (A4), and the refusal must name \
             `{major}`: `{message}`"
        );
    }
}

/// proves: design 013's Test strategy #3, #7 + A2 — the gate is per-format for the
/// manifest format too: a `manifests` document declaring `v2` names a major
/// `manifests` never shipped, so it is refused as the version gate's structured
/// error naming *manifests*' majors. `reach-classes` already ships a `v2`, and a
/// gate reading one crate-wide list would read this document with a grammar that
/// does not exist for it; the union `supported_majors()` returns is unchanged by
/// adding `manifests/v1`, which is the majors-equality case at the end of this file
/// keeping its word.
#[test]
fn rejects_a_manifest_document_declaring_a_major_the_manifests_format_does_not_ship() {
    let document = declaring(MANIFEST_DOC, "v2");
    let supported = format_majors("manifests");

    assert!(
        !supported.is_empty(),
        "the walk must find `manifests` in the contract tree to name its majors"
    );
    assert!(
        !supported.iter().any(|major| major == "v2"),
        "this case needs `manifests` to ship no `v2`; the tree carries {supported:?}"
    );

    let Err(err) = bindforge_core::parse(&document) else {
        panic!("a `manifests` document declaring `v2` must be refused, not parsed");
    };
    assert_version_rejection(err, "the manifests v2 document", &supported);
}

/// proves: design 013's Test strategy #7 — the manifest corpus carries its own
/// half of the per-format gate: `schemas/manifests/v1/fixtures/invalid/major-not-shipped.toml`
/// declares a major no `manifests` directory ships, so a third-party reader is
/// held to the same refusal `bindforge-core` gives, naming the majors *that
/// format* supports.
#[test]
fn the_invalid_corpus_ships_a_manifest_fixture_for_a_major_the_format_does_not_ship() {
    let manifests_v1 = version_dir("manifests", "v1");
    let path = manifests_v1
        .path
        .join("fixtures")
        .join("invalid")
        .join("major-not-shipped.toml");

    assert!(
        path.is_file(),
        "manifests/v1 must ship `fixtures/invalid/major-not-shipped.toml`: the per-format gate is \
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
    let supported = format_majors("manifests");
    assert_version_rejection(err, "the manifests major-not-shipped fixture", &supported);
}

/// proves: design 015's Test strategy #10 + A11 — the gate is per-format for the
/// policy format too: a `policy` document declaring `v2` names a major `policy`
/// never shipped, so it is refused as the version gate's structured error naming
/// *policy's* own majors. `profile` already ships a `v2` (design 014), and a gate
/// reading one crate-wide list would read this document with a grammar that does
/// not exist for it — the same fail-open arriving from the other side.
#[test]
fn rejects_a_policy_document_declaring_a_major_the_policy_format_does_not_ship() {
    let document = declaring(POLICY_DOC, "v2");
    let supported = format_majors("policy");

    assert!(
        supported.iter().any(|major| major == "v1"),
        "the walk must find `policy/v1` in the contract tree to name its majors; the walk saw {:?}",
        format_dirs()
            .iter()
            .map(|dir| format!("{}/{}", dir.format, dir.major))
            .collect::<Vec<_>>()
    );
    assert!(
        !supported.iter().any(|major| major == "v2"),
        "this case needs `policy` to ship no `v2`; the tree carries {supported:?}"
    );

    let Err(err) = bindforge_core::parse(&document) else {
        panic!("a `policy` document declaring `v2` must be refused, not parsed");
    };
    assert_version_rejection(err, "the policy v2 document", &supported);
}

/// proves: design 015's Test strategy #10 + A11 — `policy/v1` joins the tree
/// without the crate-wide union moving at all: `policy`'s own majors are exactly
/// `v1`, `supported_majors()` still answers the `v1`-and-`v2` union it answers
/// today, and the majors-equality invariant stays green with no change to it —
/// which is the whole point of the case below needing no edit for this format.
#[test]
fn the_policy_major_joining_the_tree_leaves_the_declared_union_unchanged() {
    let policy_majors = format_majors("policy");
    assert_eq!(
        policy_majors,
        vec!["v1".to_string()],
        "`policy` ships exactly the major its directory carries, once `schemas/policy/v1/` exists \
         (the corpus is what makes a major supported)"
    );

    let declared: BTreeSet<String> = bindforge_core::supported_majors().into_iter().collect();
    let expected: BTreeSet<String> = ["v1".to_string(), "v2".to_string()].into_iter().collect();
    assert_eq!(
        declared, expected,
        "A11: `supported_majors()` already unions to `v1` and `v2`, so adding `policy/v1` must \
         change nothing it answers"
    );
    let present: BTreeSet<String> = format_dirs().into_iter().map(|dir| dir.major).collect();
    assert_eq!(
        declared, present,
        "the majors-equality invariant holds with `policy/v1` in the tree, unchanged"
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
