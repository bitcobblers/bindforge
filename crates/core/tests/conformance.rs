//! The conformance harness for the `schemas/` formats contract (ADR-008
//! clause 3): walk every `schemas/*/v*/fixtures/{valid,invalid}` directory
//! and assert what every implementer of the formats must agree on.
//!
//! proves: Test strategy #1 — for every `fixtures/valid/` fixture,
//! `bindforge-core` parses the fixture and re-serializes it to a semantically
//! equal value; for every `fixtures/invalid/` fixture it rejects with a
//! structured error; and a `schemas/*/v*/` directory yielding zero valid or
//! zero invalid fixtures fails the run.
//!
//! Reach-class corpus fixtures are asserted by parse *and* by load — the load
//! half files the table's classes under the hand the document names, and files a
//! v2 fixture's `keys` grades under the hand too — and they are not
//! re-serialized, per `schemas/reach-classes/v1/spec.md` and
//! `schemas/reach-classes/v2/spec.md`: the per-hand distinctness of the version
//! directory's own `left.toml` / `right.toml` table data is `reach_tables.rs`
//! (design 008's Test strategy #4) and `reach_v2.rs` (design 012's Test strategy
//! #3).
//!
//! proves: design 012's Test strategy #6 — the same `schemas/*/v*/` walk picks up
//! `reach-classes/v2` and `keyboard-geometry/v1` with no harness change: every
//! valid fixture parses and, for reach tables, loads with its grades filed under
//! the declared hand, the completing table taken from the fixture's **own major**
//! rather than `v1`; every invalid fixture — including
//! `reach-classes/v1/fixtures/invalid/keys-entry.toml` — is refused with a
//! single-line bounded message; and the zero-fixture rule and the case-unique
//! fixture-name rule cover both new directories.
//!
//! proves: design 013's Test strategy #6 — the same walk picks up
//! `manifests/v1` again with no harness change: every valid manifest fixture
//! parses and re-serializes to a semantically equal value (manifests take the
//! re-serialization path, not the `reach-classes` exemption), every invalid one is
//! refused with a single-line, control-free, truncating message, and the
//! zero-fixture and case-unique-name rules cover the new directory too. What the
//! walk cannot do is the manifest's two cross-file rules — the registry join and
//! the game-directory location rule — which refuse at `load_manifest` instead, so
//! the manifest corpus stays registry-free (`manifest.rs` pins those).
//!
//! Test strategy #5 (fresh-clone one-gate promise) and #6 (CI job
//! independence) have a code-level counterpart in `gate_contract.rs`, which
//! pins what the workflows promise; their fresh-clone and Windows legs are
//! rollout checks (Rollout 4, 7), as no test file can run a GitHub Actions job
//! on another machine. What this file pins for #5 is the harness precondition —
//! the contract tree is reached through `env!("CARGO_MANIFEST_DIR")`, so the
//! cargo gate needs no cwd convention, no network and no database (A8, A26).
//!
//! # Corpus obligations Rollout 3 must satisfy
//!
//! Every suite in this directory reads the contract tree, so the scaffold
//! ships: `profile/v1` and `reach-classes/v1`, each with at least one fixture
//! in `fixtures/valid/` and one in `fixtures/invalid/`; inside
//! `profile/v1/fixtures/invalid/`, a document that declares no version
//! (`version_gate.rs`) and chain cases that reject as `Cycle` and
//! `ChainTooDeep` (`chain_guard.rs`).

mod common;

use std::fs;

use bindforge_core::Hand;
use common::{
    ScratchDir, fixture_text, fixtures, format_dirs, format_dirs_under, is_fixture_name,
    quoted_array, rejection, schemas_root, string_array, string_entry, version_dir,
    zero_fixture_violation,
};

/// proves: Test strategy #1 — every valid fixture parses, is accepted by the
/// guards, and re-serializes to a semantically equal value.
#[test]
fn every_valid_fixture_re_serializes_to_a_semantically_equal_value() {
    for dir in &format_dirs() {
        for path in fixtures(dir, "valid") {
            let label = path.display().to_string();
            let text =
                fs::read_to_string(&path).unwrap_or_else(|err| panic!("reading {label}: {err}"));

            let document = bindforge_core::parse(&text)
                .unwrap_or_else(|err| panic!("{label} is a valid fixture and must parse: {err}"));
            if let Err(error) = rejection(&text) {
                panic!(
                    "{label} is a valid fixture and must pass the guards: {}",
                    error.detail()
                );
            }

            if dir.format == "reach-classes" {
                // Reach-class tables are asserted by load, not by
                // re-serialization (`schemas/reach-classes/v1/spec.md`,
                // `schemas/reach-classes/v2/spec.md`).
                load_corpus_table(&label, &text, &dir.major);
                continue;
            }

            let round_tripped = bindforge_core::parse(&bindforge_core::serialize(&document))
                .unwrap_or_else(|err| panic!("{label} does not re-parse after serializing: {err}"));
            assert_eq!(
                round_tripped, document,
                "{label} must re-serialize to a semantically equal value"
            );
        }
    }
}

/// proves: design 008's Test strategy #4 + design 012's Test strategy #6 +
/// Formats contract (Harness) — the load half of the reach-class corpus: a valid
/// corpus table loads into the model, its classes
/// are filed under the hand its document names in table order, and a v2
/// fixture's `keys` grades are filed under that hand as well.
///
/// A stand-alone corpus fixture carries no file name of its own, so it stands in
/// for its declared hand inside a scratch directory, with the contract tree's own
/// table for the other hand completing the directory
/// (`schemas/reach-classes/v1/spec.md`, `schemas/reach-classes/v2/spec.md`). The
/// completing table is taken from the fixture's **own major**, not from `v1`: the
/// loader refuses a document whose declared version differs from the version
/// segment of the directory it loads from (design 012's Test strategy #3), so a
/// v2 fixture is completed from `reach-classes/v2/` and a v1 fixture from
/// `reach-classes/v1/`, and the scratch directory the pair sits in carries that
/// same segment.
fn load_corpus_table(label: &str, text: &str, major: &str) {
    let declared = string_entry(text, "hand")
        .unwrap_or_else(|| panic!("{label} is a table document: it declares a `hand` entry"));
    let classes = string_array(text, "classes")
        .unwrap_or_else(|| panic!("{label} is a table document: it declares a `classes` array"));
    let hand = match declared.as_str() {
        "left" => Hand::Left,
        "right" => Hand::Right,
        other => panic!("{label} declares `hand = \"{other}\"`, which is not a hand"),
    };
    let other = if declared == "left" { "right" } else { "left" };

    let scratch = ScratchDir::new("reach-classes-corpus-load");
    let shipped = schemas_root()
        .join("reach-classes")
        .join(major)
        .join(format!("{other}.toml"));
    scratch.write(&format!("{major}/{other}.toml"), &fixture_text(&shipped));
    scratch.write(&format!("{major}/{declared}.toml"), text);
    let directory = scratch.path().join(major);

    let tables = bindforge_core::load_reach_tables(&directory)
        .unwrap_or_else(|err| panic!("the valid corpus table {label} must load: {err}"));
    let table = tables.hand(hand).unwrap_or_else(|| {
        panic!("{label} declares hand `{declared}`; the loader must file its classes there")
    });

    assert_eq!(
        table.hand(),
        hand,
        "the table loaded from {label} must report the hand its document declares"
    );
    // The contract's `classes` entry lists reach-class ids in table order
    // (`spec.md`), so the comparison is on ids: nothing in this design makes an
    // id-shaped string part of `ReachClass`'s public surface.
    let loaded: Vec<String> = table
        .classes()
        .iter()
        .map(|class| class.id.clone())
        .collect();
    assert_eq!(
        loaded, classes,
        "the loader must file {label}'s classes, in table order, under the hand it declares"
    );

    // A v2 document's grades ride the same load: every
    // `"<input-id> <class-id>"` pair must answer from `grade` under the hand the
    // fixture declares (`schemas/reach-classes/v2/spec.md`). A v1 document
    // carries no `keys` entry at all, so the load half stops at its classes.
    let Some(pairs) = quoted_array(text, "keys") else {
        return;
    };
    for pair in &pairs {
        let pair = pair.replace("\\\\", "\\");
        let mut fields = pair.split(' ');
        let (Some(input), Some(class)) = (fields.next(), fields.next()) else {
            panic!(
                "{label}: a `keys` element is one `\"<input-id> <class-id>\"` pair, got `{pair}`"
            );
        };
        let id = bindforge_core::InputId::parse(input).unwrap_or_else(|err| {
            panic!("{label}: `{input}` must be an id the grammar admits: {err}")
        });
        let graded = table.grade(&id).unwrap_or_else(|| {
            panic!("{label}: the loader must file the grade `{input}` carries under `{declared}`")
        });
        assert_eq!(
            graded.id, class,
            "{label}: `grade({input})` must answer `{class}`, the class its own pair names"
        );
    }
}

/// proves: Test strategy #1 — every invalid fixture is rejected with a
/// structured error. A typed `CoreError` is the structured error: a document
/// that parses cleanly, or a parser that panics instead of returning one,
/// fails this test.
#[test]
fn every_invalid_fixture_is_rejected_with_a_structured_error() {
    for dir in &format_dirs() {
        for path in fixtures(dir, "invalid") {
            let label = path.display().to_string();
            let text =
                fs::read_to_string(&path).unwrap_or_else(|err| panic!("reading {label}: {err}"));

            match rejection(&text) {
                Err(error) => assert!(
                    !error.detail().is_empty(),
                    "{label} must be rejected with an error that says what is wrong"
                ),
                Ok(()) => panic!("{label} is an invalid fixture and must be rejected"),
            }
        }
    }
}

/// proves: Chosen design → Formats contract (Harness) + Test strategy #1 — a
/// `schemas/*/v*/` directory yielding zero valid or zero invalid fixtures
/// fails the run, so an empty corpus can never make a format look supported.
#[test]
fn every_format_version_directory_ships_valid_and_invalid_fixtures() {
    let dirs = format_dirs();
    assert!(
        !dirs.is_empty(),
        "the walk found no `schemas/*/v*/` directory under {}",
        schemas_root().display()
    );

    if let Some(violation) = zero_fixture_violation(&dirs) {
        panic!(
            "{violation}: presence in `fixtures/valid/` is the definition of \"supported\", and \
             the invalid corpus is the contract for rejection behavior, so an empty fixture \
             directory must fail the run"
        );
    }
}

/// proves: Chosen design → Formats contract (Harness) + Test strategy #1 — the
/// zero-fixture rule's own trigger: pointed at a scratch tree whose format
/// directory has an empty `fixtures/valid/` or an empty `fixtures/invalid/`,
/// the walk reports the violation; a scratch tree carrying one fixture of each
/// kind reports none. Without this, the rule above could never be shown to
/// fire, because the checked-in tree is only ever non-empty.
#[test]
fn the_zero_fixture_rule_fires_on_an_empty_fixture_directory() {
    let placeholder = "# placeholder so the directory is not empty\n";

    let scratch = ScratchDir::new("zero-fixture");
    scratch.make_dir("empty-valid/v1/fixtures/valid");
    scratch.write("empty-valid/v1/fixtures/invalid/document.toml", placeholder);
    scratch.write("empty-invalid/v1/fixtures/valid/document.toml", placeholder);
    scratch.make_dir("empty-invalid/v1/fixtures/invalid");

    let dirs = format_dirs_under(scratch.path());
    assert_eq!(
        dirs.len(),
        2,
        "the walk must see both scratch format directories under {}",
        scratch.path().display()
    );
    for dir in &dirs {
        let version = format!("{}/{}", dir.format, dir.major);
        assert!(
            zero_fixture_violation(std::slice::from_ref(dir)).is_some(),
            "{version} has an empty fixture directory, so the zero-fixture rule must fire"
        );
    }

    let complete = ScratchDir::new("complete-corpus");
    complete.write("profile/v1/fixtures/valid/document.toml", placeholder);
    complete.write("profile/v1/fixtures/invalid/document.toml", placeholder);
    assert!(
        zero_fixture_violation(&format_dirs_under(complete.path())).is_none(),
        "a version directory with at least one fixture of each kind must satisfy the rule"
    );
}

/// proves: Chosen design → Adversarial guard (rejection messages) —
/// `schemas/README.md` fixes a rejection as "a single bounded line" quoting
/// escaped parts of the document, so a rejection for any corpus case cannot
/// forge an extra log line or carry a raw control character.
/// `profile/v1/fixtures/invalid/forged-log-line.toml` is the corpus case.
#[test]
fn every_invalid_fixture_is_rejected_with_a_single_line_message() {
    for dir in &format_dirs() {
        for path in fixtures(dir, "invalid") {
            let label = path.display().to_string();
            let text =
                fs::read_to_string(&path).unwrap_or_else(|err| panic!("reading {label}: {err}"));

            let detail = match rejection(&text) {
                Err(error) => error.detail(),
                Ok(()) => panic!("{label} is an invalid fixture and must be rejected"),
            };
            assert!(
                !detail.contains('\n') && !detail.contains('\r'),
                "{label} must be rejected in one line; the rejection rendered {detail:?}"
            );
            assert!(
                detail.chars().all(|ch| !ch.is_control()),
                "{label}'s rejection must escape control characters rather than emit them; \
                 the rejection rendered {detail:?}"
            );
        }
    }
}

/// proves: Chosen design → Adversarial guard (rejection messages) — the bound
/// half of the same rule: a rejection quoting an untrusted value truncates it,
/// so one hostile document cannot inflate log volume by echoing a value close to
/// the whole `MAX_DOCUMENT_BYTES` budget.
#[test]
fn a_rejection_truncates_an_untrusted_value_rather_than_echoing_it() {
    let tail = "tail-of-the-echoed-value";
    let document = format!(
        "format = \"profile\"\nversion = \"v99\\n{} {tail}\"\nid = \"echo\"\nderives = []\n",
        "x".repeat(4096)
    );

    let rejected = bindforge_core::parse(&document)
        .expect_err("a version string carrying a forged newline must be rejected");
    let detail = rejected.to_string();
    assert!(
        !detail.contains(tail),
        "the rejection must truncate the untrusted value, not echo all 4 KiB of it: {detail}"
    );
    assert!(
        detail.chars().all(|ch| !ch.is_control()),
        "the rejection must stay one line for a value carrying a decoded newline: {detail:?}"
    );
}

/// proves: Formats contract → versioning rule — fixture file names are
/// lowercase ASCII, `-`-separated, and case-unique within a fixture
/// directory, which is what keeps the same corpus green on a contributor's
/// case-insensitive Windows filesystem as on the Linux CI runner.
#[test]
fn fixture_file_names_are_lowercase_ascii_dash_separated_and_case_unique() {
    for dir in &format_dirs() {
        for kind in ["valid", "invalid"] {
            let files = fixtures(dir, kind);

            for path in &files {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_else(|| panic!("{} has no usable file name", path.display()));
                assert!(
                    is_fixture_name(name),
                    "{} is not a valid fixture name: use lowercase ASCII words separated by `-`",
                    path.display()
                );
            }

            let mut lowered: Vec<String> = files
                .iter()
                .map(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default()
                        .to_lowercase()
                })
                .collect();
            let unique = {
                lowered.sort();
                lowered.dedup();
                lowered.len()
            };
            assert_eq!(
                unique,
                files.len(),
                "fixtures/{kind} of {} holds names that differ only by case",
                dir.path.display()
            );
        }
    }
}

/// proves: Chosen design → Formats contract + Rollout 3, and design 012's Test
/// strategy #6 —
/// the scaffold ships `profile/v1` and `reach-classes/v1`, ADR-011 added
/// `action-ids/v1`, ADR-012 adds `reach-classes/v2` and
/// `keyboard-geometry/v1`, ADR-013 adds `manifests/v1`, and design 014 adds
/// `profile/v2`; the same `schemas/*/v*/` walk picks up all of them with
/// no harness change, which is also how `patch` and `policy` join when
/// ADR-015 and its successors add them.
///
/// proves: design 014's Test strategy #7 — the unchanged walk picks up
/// `profile/v2`: every valid v2 fixture parses, passes the guards, and re-
/// serializes to a semantically equal value (profiles keep the round-trip
/// path), every invalid one is refused with a single-line, control-free,
/// truncating message, and the zero-fixture rule and the case-unique fixture-
/// name rule cover the new directory — all of which the loops above run over
/// it the moment `schemas/profile/v2/` ships (A3, A16).
#[test]
fn the_walk_picks_up_every_version_directory_the_contract_tree_ships() {
    let found: Vec<String> = format_dirs()
        .iter()
        .map(|dir| format!("{}/{}", dir.format, dir.major))
        .collect();

    for expected in [
        "profile/v1",
        "reach-classes/v1",
        "action-ids/v1",
        "reach-classes/v2",
        "keyboard-geometry/v1",
        "manifests/v1",
        "profile/v2",
    ] {
        assert!(
            found.iter().any(|dir| dir == expected),
            "the harness must exercise {expected}; the walk found {found:?}"
        );
    }
}

/// proves: design 013's Test strategy #6 — the manifest corpus is bound by the
/// same walk with no harness change, and `manifests` is on the re-serialization
/// path, not the `reach-classes` exemption: every `manifests/v1` valid fixture
/// parses and re-serializes to a semantically equal value, and every invalid one
/// is refused at `parse` alone — which is what keeps the manifest corpus
/// registry-free, since the join and the location rule live at load instead
/// (Chosen design → Load-time validation).
#[test]
fn every_manifest_fixture_is_exercised_by_the_unchanged_walk() {
    let manifests_v1 = version_dir("manifests", "v1");

    let valid = fixtures(&manifests_v1, "valid");
    assert!(
        !valid.is_empty(),
        "manifests/v1 must ship fixtures/valid/: presence there is the definition of \"supported\""
    );
    for path in &valid {
        let label = path.display().to_string();
        let text = fixture_text(path);
        let document = bindforge_core::parse(&text).unwrap_or_else(|err| {
            panic!("{label} is a valid manifest fixture and must parse: {err}")
        });
        let round_tripped = bindforge_core::parse(&bindforge_core::serialize(&document))
            .unwrap_or_else(|err| panic!("{label} does not re-parse after serializing: {err}"));
        assert_eq!(
            round_tripped, document,
            "{label} must re-serialize to a semantically equal value: manifests take the same \
             round-trip obligation as every format except `reach-classes`"
        );
    }

    let invalid = fixtures(&manifests_v1, "invalid");
    assert!(
        !invalid.is_empty(),
        "manifests/v1 must ship fixtures/invalid/: the invalid corpus is the contract for \
         rejection behavior"
    );
    for path in &invalid {
        let label = path.display().to_string();
        let text = fixture_text(path);
        assert!(
            bindforge_core::parse(&text).is_err(),
            "{label} is an invalid manifest fixture and must refuse at `parse` alone — the \
             manifest corpus carries no registry"
        );
    }
}
