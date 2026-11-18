//! The slug-bound slice of the `schemas/` conformance corpora: the four fixtures that
//! pin the shared `MAX_SLUG_OCTETS` bound, which ADR-024 raises from 63 to 128 octets
//! and gives one stated home in `schemas/README.md`.
//!
//! proves: design 024's Test strategy #10 — the re-pinned `id-past-cap.toml`
//! (`policy/v1`, `profile/v2`, and new in `manifests/v1`) and `segment-past-cap.toml`
//! (`action-ids/v1`) are refused at 129 octets, and every valid fixture in all four
//! trees still parses under the 128 bound.
//!
//! `conformance.rs` walks every `schemas/*/v*/fixtures/{valid,invalid}` directory and
//! already refuses every invalid fixture with a single-line, bounded message; what it
//! cannot say is *which* bound each of these fixtures is one octet past, or that the
//! manifest format gained the fixture its Rust-only pin used to be alone in (A11: the
//! manifest `id` is the exact id a discovered slug must be adoptable byte-for-byte
//! as). This suite pins the four fixtures by name and length, so a re-pin that leaves
//! a 64-octet literal or the superseded `segment-past-63-octets.toml` name in the
//! tree goes red here rather than silently passing the walk.
//!
//! The fixture files themselves are the contract tree's, per Rollout 2 of the design;
//! this file reads them, it never writes them.

mod common;

use common::{fixture_text, fixtures, string_array, string_entry, version_dir};

/// The four format/major pairs whose id (or id segment) rides the shared slug bound.
const SLUG_BEARING_TREES: [(&str, &str); 4] = [
    ("manifests", "v1"),
    ("profile", "v2"),
    ("policy", "v1"),
    ("action-ids", "v1"),
];

/// The three axis-kind arrays an `action-ids` document may list ids in.
const AXIS_ARRAYS: [&str; 3] = ["discrete-lateral", "continuous-lateral", "none"];

/// The longest id, or id segment, one of these fixtures' documents declares.
fn longest_declared_id(format: &str, text: &str) -> Option<String> {
    if format == "action-ids" {
        let mut longest: Option<String> = None;
        for array in AXIS_ARRAYS {
            for id in string_array(text, array).unwrap_or_default() {
                let segment = id.rsplit('.').next().unwrap_or(id.as_str());
                if longest
                    .as_ref()
                    .is_none_or(|held| segment.len() > held.len())
                {
                    longest = Some(segment.to_string());
                }
            }
        }
        return longest;
    }
    string_entry(text, "id")
}

/// proves: design 024's Test strategy #10 + Specs, fixtures, ledger — each of the
/// four slug-bound fixtures ships, is named after the bound rather than a literal
/// number where ADR-024 renamed it, and carries its fault at exactly one octet past
/// the exported constant.
#[test]
fn the_slug_bound_fixtures_sit_one_octet_past_the_exported_constant() {
    let past_cap = bindforge_core::MAX_SLUG_OCTETS + 1;

    for (format, major) in SLUG_BEARING_TREES {
        let dir = version_dir(format, major);
        let name = if format == "action-ids" {
            "segment-past-cap.toml"
        } else {
            "id-past-cap.toml"
        };
        let path = dir.path.join("fixtures").join("invalid").join(name);
        assert!(
            path.is_file(),
            "`{format}/{major}` must ship `fixtures/invalid/{name}`: the invalid corpus is the \
             contract for rejection behavior — ADR-024 re-pins it (and adds it to \
             `manifests/v1`, whose `id` is what a discovered slug must be adoptable as); {}",
            dir.path.display()
        );

        let text = fixture_text(&path);
        let longest = longest_declared_id(format, &text).unwrap_or_else(|| {
            panic!(
                "`{name}` must declare the id (or id segment) it carries as a one-line entry, so \
                 its length is checkable: {}",
                path.display()
            )
        });
        assert_eq!(
            longest.len(),
            past_cap,
            "`{format}/{major}/{name}` must carry its fault at exactly one octet past \
             MAX_SLUG_OCTETS ({bound}), not the superseded 64",
            bound = bindforge_core::MAX_SLUG_OCTETS
        );
    }
}

/// proves: design 024's Test strategy #10 + Risks (third-party readers) — the
/// superseded fixture name is gone: a 64-octet fault left in the tree under a name
/// that states a number the contracts no longer publish would pin the old bound as
/// conformance.
#[test]
fn the_superseded_segment_bound_fixture_is_out_of_the_corpus() {
    let dir = version_dir("action-ids", "v1");
    let stale = dir
        .path
        .join("fixtures")
        .join("invalid")
        .join("segment-past-63-octets.toml");
    assert!(
        !stale.exists(),
        "`segment-past-63-octets.toml` is renamed to `segment-past-cap.toml` by ADR-024: the \
         old name states a bound the four published contracts no longer carry, and the file it \
         pins is no longer one past it: {}",
        stale.display()
    );
}

/// proves: design 024's Test strategy #10 — each slug-bound fixture refuses at
/// `parse` alone, with the refusal naming the entry it refused and never the length
/// the fixture's prose used to spell out.
#[test]
fn each_slug_bound_fixture_refuses_one_octet_past_the_constant() {
    for (format, major) in SLUG_BEARING_TREES {
        let dir = version_dir(format, major);
        let name = if format == "action-ids" {
            "segment-past-cap.toml"
        } else {
            "id-past-cap.toml"
        };
        let path = dir.path.join("fixtures").join("invalid").join(name);
        let text = fixture_text(&path);

        let err = bindforge_core::parse(&text).err().unwrap_or_else(|| {
            panic!(
                "`{format}/{major}/fixtures/invalid/{name}` is one octet past the shared bound \
                 ({bound}) and must be refused",
                bound = bindforge_core::MAX_SLUG_OCTETS
            )
        });
        assert!(
            err.to_string()
                .contains(&bindforge_core::MAX_SLUG_OCTETS.to_string())
                || err
                    .to_string()
                    .contains(&(bindforge_core::MAX_SLUG_OCTETS + 1).to_string()),
            "the refusal for `{format}/{major}/{name}` should name the shared bound it enforced \
             ({}, built from the constant); got `{err}`",
            bindforge_core::MAX_SLUG_OCTETS
        );
    }
}

/// proves: design 024's Test strategy #10 + Compatibility matrix (third-party schema
/// readers) — the amendment is relaxation-only: every valid fixture in all four
/// slug-bearing trees still parses under the 128 bound and re-serializes to a
/// semantically equal document, so a document valid under the 63-octet text stays
/// valid under the 128-octet one (`024-L3`).
#[test]
fn every_valid_fixture_in_the_four_slug_bearing_trees_parses_under_the_bound() {
    for (format, major) in SLUG_BEARING_TREES {
        let dir = version_dir(format, major);
        let valid = fixtures(&dir, "valid");
        assert!(
            !valid.is_empty(),
            "`{format}/{major}` must keep a non-empty `fixtures/valid/`: presence there is the \
             definition of \"supported\"; {}",
            dir.path.display()
        );

        for path in valid {
            let label = path.display().to_string();
            let text = fixture_text(&path);
            let document = bindforge_core::parse(&text)
                .unwrap_or_else(|err| panic!("{label} is a valid fixture and must parse: {err}"));
            let round_tripped = bindforge_core::parse(&bindforge_core::serialize(&document))
                .unwrap_or_else(|err| panic!("{label} does not re-parse after serializing: {err}"));
            assert_eq!(
                round_tripped, document,
                "{label} must re-serialize to a semantically equal value under the 128 bound"
            );
        }
    }
}

/// proves: design 024's Test strategy #10 + Chosen design (Specs, fixtures, ledger) —
/// the manifest format, whose `id` is what a discovered game's slug must be adoptable
/// byte-for-byte as, is no longer pinned by a Rust test alone: its bound fixture
/// rides the same corpus walk as every other format's, and the walk's own roster of
/// version directories still covers all four trees.
#[test]
fn the_manifest_format_ships_the_slug_bound_fixture_its_corpus_lacked() {
    let manifests = version_dir("manifests", "v1");
    let invalid_names: Vec<String> = fixtures(&manifests, "invalid")
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .collect();

    assert!(
        invalid_names.iter().any(|name| name == "id-past-cap.toml"),
        "`manifests/v1` must ship `fixtures/invalid/id-past-cap.toml` (A11: the manifest `id` \
         bound was pinned only by a Rust test); the invalid corpus now holds {invalid_names:?}"
    );

    let found: Vec<String> = common::format_dirs()
        .iter()
        .map(|dir| format!("{}/{}", dir.format, dir.major))
        .collect();
    for tree in SLUG_BEARING_TREES {
        let label = format!("{}/{}", tree.0, tree.1);
        assert!(
            found.contains(&label),
            "the conformance walk must keep exercising `{label}` with no harness change; it \
             found {found:?}"
        );
    }
}
