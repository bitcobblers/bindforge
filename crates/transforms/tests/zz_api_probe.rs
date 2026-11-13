//! The public surface ADR-015 promises `crates/transforms` re-exports: the
//! application entry point, the produced-base answer, and the report vocabulary
//! ADR-017's CLI and the hub display.
//!
//! proves: design 015's Test strategy #10 + its Delta inventory
//! (`crates/transforms/`'s `src/lib.rs` — "module doc: what the crate is, what
//! surface it re-exports") — this target resolves `apply`, `Applied`,
//! `TransformReport`, and `NotApplicableReason` at the crate root, in call, type,
//! and value position, with nothing but `use bindforge_transforms::…`, and
//! connects one to the next: loaded inputs → `apply` → `Applied` → reports → the
//! produced base.
//!
//! The behavior behind each name is pinned by the module-mirrored `apply.rs`
//! (Test strategy #4–#8); this file adds no rule of its own — it keeps the
//! surface the rule-bearing suite is written through honest, which is exactly
//! what a consumer that has not been written yet compiles against (the same
//! posture `crates/core/tests/zz_api_probe.rs` ships).
//!
//! It also pins the closed reason vocabulary itself: `NotApplicableReason::name()`
//! answers the ten contract spellings `schemas/policy/v1/spec.md` states, so a
//! non-Rust implementer produces the same verdicts, and the vocabulary cannot
//! drift or grow without a `policy/v2` (A11).

mod common;

use bindforge_core::{
    ActionRegistry, CoreError, KeyboardGeometry, Manifest, Policy, Profile, ReachTables,
};
use bindforge_transforms::{Applied, NotApplicableReason, TransformReport, apply};
use common::{
    BASE_ID, DEFAULT_SET, POLICY_ID, ScratchDir, base_text, load_default_registry, manifest_of,
    manifest_text, patch_text, policy_of, policy_text, profile_of, shipped_inputs,
};

/// The row this probe applies: it fits the default base, so the probe is about
/// the names connecting, not about a verdict (`apply.rs` pins those).
const PROBE_ROW: &str = "relocate-cluster move.steer-* left left 0 keyboard:s keyboard:d";

/// proves: design 015's Test strategy #10 — every name the crate promises
/// resolves at `bindforge-transforms`'s root and connects to the next one.
#[test]
fn the_adr_015_names_resolve_at_the_crate_root() {
    let scratch = ScratchDir::new("api-probe-transforms");
    let registry: ActionRegistry = load_default_registry(&scratch);
    let (tables, geometry): (ReachTables, KeyboardGeometry) = shipped_inputs();
    let manifest: Manifest = manifest_of(&manifest_text(&[]));
    let base: Profile = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy: Policy = policy_of(&policy_text(POLICY_ID, &[PROBE_ROW]));

    let applied: Applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("the probe row fits the probe base, so `apply` answers with an `Applied`");

    let reports: Vec<TransformReport> = applied.reports;
    assert_eq!(reports.len(), 1, "one report per row of the stack");
    assert_eq!(reports[0].index, 0, "a report carries its row position");
    assert_eq!(
        reports[0].kind, "relocate-cluster",
        "and the contract spelling of the row's kind"
    );

    let produced: Profile = applied.base;
    assert_eq!(
        produced.id, POLICY_ID,
        "`Applied::base` is the produced base document, named by the policy"
    );

    // `CoreError` is still the only refusal type on the entry channel: a patch as
    // the base argument refuses, and no new variant is promised (A9).
    let patched: Profile = profile_of(&patch_text("probe-patch", &DEFAULT_SET));
    let refused: Result<Applied, CoreError> =
        apply(&policy, &patched, &manifest, &registry, &tables, &geometry);
    assert!(
        refused.is_err(),
        "`apply` over a patch is a structured refusal, not a produced base and not a report"
    );
}

/// proves: design 015's Test strategy #10 + Chosen design → The report — the
/// reason vocabulary is closed and its spellings are contract: `name()` answers
/// the same ten strings `spec.md` states, the way `ProfileKind::name()` answers
/// its spellings today. Every one of them is exercised in a fitting case by
/// `apply.rs`; this pins the vocabulary as a set.
#[test]
fn the_not_applicable_reason_vocabulary_answers_the_contract_spellings() {
    for (reason, spelling) in [
        (NotApplicableReason::UnmatchedTags, "unmatched-tags"),
        (NotApplicableReason::UnboundAction, "unbound-action"),
        (NotApplicableReason::MatchCount, "match-count"),
        (NotApplicableReason::NotKeyboard, "not-keyboard"),
        (NotApplicableReason::NotContiguous, "not-contiguous"),
        (NotApplicableReason::OccupiedTarget, "occupied-target"),
        (NotApplicableReason::PlacementIllegal, "placement-illegal"),
        (NotApplicableReason::ReservedInput, "reserved-input"),
        (NotApplicableReason::AxisKind, "axis-kind"),
        (NotApplicableReason::UnboundInput, "unbound-input"),
    ] {
        assert_eq!(
            reason.name(),
            spelling,
            "`NotApplicableReason::name()` must answer the contract's spelling, the verdict a \
             third-party implementer of `policy/v1` produces for the same fault"
        );
    }
}
