//! The closed static registry and its two claim passes (`017-C5`, §The registry).
//!
//! proves: Test strategy #9.

mod common;

use bindforge_codecs::ExtractionRefusal;
use bindforge_codecs::codec::{CodecFamily, Detection, Evidence, UserCandidates};
use bindforge_codecs::context::GameContext;
use bindforge_codecs::registry::{REGISTRY, claim_with, family_ids, is_known_family};
use bindforge_codecs::snapshot::BindingSnapshot;
use common::context;
use std::path::PathBuf;

/// A family that probe-claims anything (stands in for a grammar match).
struct ProbeEverything {
    id: &'static str,
}

/// A family that claims nothing.
struct ProbeNothing {
    id: &'static str,
}

fn synthetic_claim(family: &'static str) -> Detection {
    Detection {
        family,
        evidence: Evidence::InstallTreeProbe,
        per_user: vec![UserCandidates {
            user: "76561197960287930".to_string(),
            files: Vec::new(),
            tree_root: PathBuf::from("/synthetic/tree"),
        }],
    }
}

impl CodecFamily for ProbeEverything {
    fn id(&self) -> &'static str {
        self.id
    }
    fn appids(&self) -> &'static [&'static str] {
        &[]
    }
    fn detect(&self, _ctx: &GameContext) -> Option<Detection> {
        Some(synthetic_claim(self.id))
    }
    fn extract(&self, _claim: &Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal> {
        Ok(Vec::new())
    }
}

impl CodecFamily for ProbeNothing {
    fn id(&self) -> &'static str {
        self.id
    }
    fn appids(&self) -> &'static [&'static str] {
        &[]
    }
    fn detect(&self, _ctx: &GameContext) -> Option<Detection> {
        None
    }
    fn extract(&self, _claim: &Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal> {
        Ok(Vec::new())
    }
}

/// A family owning appid `900100` by table, claiming nothing on probe.
struct AppidOwner;

impl CodecFamily for AppidOwner {
    fn id(&self) -> &'static str {
        "synthetic-appid-owner"
    }
    fn appids(&self) -> &'static [&'static str] {
        &["900100"]
    }
    fn detect(&self, ctx: &GameContext) -> Option<Detection> {
        if ctx.appid == "900100" {
            Some(synthetic_claim(self.id()))
        } else {
            None
        }
    }
    fn extract(&self, _claim: &Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal> {
        Ok(Vec::new())
    }
}

// proves: Test strategy #9 (§The registry: "declared in family-id byte order,
// with a unit test asserting the declaration *is* byte-sorted so the ordering
// law cannot rot").
#[test]
fn the_registry_declaration_is_byte_sorted() {
    let ids: Vec<&str> = REGISTRY.iter().map(|family| family.id()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(
        ids, sorted,
        "REGISTRY must be declared in family-id byte order: {ids:?}"
    );
    assert_eq!(
        family_ids(),
        sorted,
        "family_ids must publish the same order the slice declares"
    );
}

// proves: Test strategy #9 (§The registry, pass 1) — a synthetic family owning
// appid `900100` wins the appid pass with `Evidence::AppidTable` even when
// another family could probe-claim it: the appid pass runs before any tree
// probing, whatever the slice order.
#[test]
fn the_appid_pass_wins_before_any_tree_probe() {
    let probe_first = ProbeEverything {
        id: "aaa-probe-first",
    };
    let appid_owner = AppidOwner;
    let ctx = context("900100", vec![PathBuf::from("/r")], PathBuf::from("/l"));

    // Byte-sorted declaration order (probe family sorts first)…
    let claim = claim_with(&[&probe_first, &appid_owner], &ctx).expect("the appid pass claims");
    // …must still hand the claim to the appid-table family, not the probe.
    assert_eq!(
        claim.family, "synthetic-appid-owner",
        "pass 1 claims before pass 2"
    );
    assert!(
        matches!(claim.evidence, Evidence::AppidTable),
        "an appid-table claim carries AppidTable evidence, got {:?}",
        claim.evidence
    );

    // And it is not an accident of slice order: reversed input, same outcome.
    let claim = claim_with(&[&appid_owner, &probe_first], &ctx).expect("the appid pass claims");
    assert_eq!(claim.family, "synthetic-appid-owner");
    assert!(matches!(claim.evidence, Evidence::AppidTable));
}

// proves: Test strategy #9 (§The registry, pass 2) — with an unlisted appid the
// probe pass claims in family-id order, and the claim carries
// `Evidence::InstallTreeProbe`.
#[test]
fn the_probe_pass_claims_in_id_order() {
    let second = ProbeEverything {
        id: "zzz-probe-second",
    };
    let first = ProbeEverything {
        id: "mmm-probe-first",
    };
    let ctx = context("900999", vec![PathBuf::from("/r")], PathBuf::from("/l"));
    let claim = claim_with(&[&second, &first], &ctx).expect("pass 2 claims");
    assert_eq!(
        claim.family, "mmm-probe-first",
        "the probe pass iterates in id order"
    );
    assert!(matches!(claim.evidence, Evidence::InstallTreeProbe));
}

// proves: Test strategy #9 (§The registry: "A tree probe claims only when it
// located at least one file in its own grammar") — a probe that locates
// nothing does not claim.
#[test]
fn a_probe_that_locates_nothing_does_not_claim() {
    let ctx = context("900998", vec![PathBuf::from("/r")], PathBuf::from("/l"));
    assert!(
        claim_with(&[&ProbeNothing { id: "solo-nothing" }], &ctx).is_none(),
        "locating nothing is not a claim"
    );
    // And an appid-table family whose own detect declines also does not claim
    // through pass 1 — the table selects whose detect runs, it does not rubber-stamp.
    let claim = claim_with(
        &[&AppidOwner],
        &context("900998", vec![], PathBuf::from("/l")),
    );
    assert!(
        claim.is_none(),
        "an unlisted appid skips the table family entirely"
    );
}

// proves: Test strategy #9 (§The registry: "`family_ids()` and
// `is_known_family(id)` publish the closed set for consumers") — the published
// set agrees with the declared slice, and the shipped set is exactly the
// `unreal-ini` family this run lands.
#[test]
fn the_published_family_set_agrees_with_the_registry_slice() {
    let published: Vec<String> = family_ids().into_iter().map(|id| id.to_string()).collect();
    let declared: Vec<String> = REGISTRY
        .iter()
        .map(|family| family.id().to_string())
        .collect();
    assert_eq!(published, declared, "family_ids is the slice, published");
    for id in &published {
        assert!(is_known_family(id), "family_ids members must be known");
    }
    assert!(
        is_known_family("unreal-ini"),
        "`unreal-ini` is this run's first family"
    );
    assert!(
        !is_known_family("source-bind"),
        "an unshipped family is not in the closed set"
    );
    assert!(
        !is_known_family("Unreal-Ini"),
        "family ids are case-sensitively their slug grammar"
    );
    assert_eq!(
        published,
        vec!["unreal-ini".to_string()],
        "the shipped implementation set is exactly `unreal-ini`, `017-R3` deferring everything else"
    );
}

/// A family whose `appids()` name the context's appid and whose `detect`
/// counts its invocations.
struct CountingDecliner;

static COUNTING_DETECTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl CodecFamily for CountingDecliner {
    fn id(&self) -> &'static str {
        "synthetic-counting-decliner"
    }
    fn appids(&self) -> &'static [&'static str] {
        &["900101"]
    }
    fn detect(&self, _ctx: &GameContext) -> Option<Detection> {
        COUNTING_DETECTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        None // the table selects whose detect runs; it declines.
    }
    fn extract(&self, _claim: &Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal> {
        Ok(Vec::new())
    }
}

// proves: Test strategy #9 (§The registry: pass 2 "runs only if pass 1
// claimed nothing: every family in id order") — pinned on the work done, per
// the accepted performance finding 5 (round 1): pass 2 does not re-run the
// `detect` of a family the appid pass already selected, so one `claim` runs
// each family's `detect` at most once. Test-update item "registry pass-2
// skip" in `03-build-notes.md`.
#[test]
fn pass_two_does_not_re_run_a_family_the_appid_pass_already_ran() {
    let ctx = context("900101", vec![PathBuf::from("/r")], PathBuf::from("/l"));
    assert!(
        claim_with(&[&CountingDecliner], &ctx).is_none(),
        "the family declines on the table pass and claims nothing on probe"
    );
    assert_eq!(
        COUNTING_DETECTS.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "pass 1 ran this family's detect once; pass 2 must skip the ids pass 1 already ran"
    );
}

// proves: Test strategy #9 and #15 (§Options: discharging `013-R3`) — the
// registry-check half this run makes available is the published set: a codec id
// outside it (`unreal-engine-5`, the id the core suites carry) is unknown here,
// while `load_manifest`'s load-time behavior is untouched (`crates/core`'s
// `load_does_not_registry_check_codec_or_preset_ids` stays green — a manifest
// naming a not-yet-implemented family stays loadable).
#[test]
fn the_published_set_is_a_seam_not_a_load_time_guard() {
    assert!(
        !is_known_family("unreal-engine-5"),
        "the registry names implementation families only; manifests naming future ones stay loadable (`013-R3` narrowed, `017-R4` carries the check)"
    );
}
