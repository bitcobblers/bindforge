//! The transform stack of ADR-015: `crates/transforms/src/apply.rs` — the
//! sequential stack of named, preconditioned transforms over core's public
//! surface, the per-row verdict reports, the untouchable guard, and the produced
//! base document.
//!
//! proves: design 015's Test strategy #4 (`relocate-cluster` and every named
//! outcome), #5 (`swap-polarity`), #6 (`swap-mouse-buttons`), #7 (stack
//! semantics: sequential application, the all-inapplicable policy, and
//! `report.index`), and #8 (the produced document as a chain citizen, and
//! determinism).
//!
//! The canonical statement of every rule here is the proposal (Chosen design →
//! The stack, The report, The output). Four of its decisions bind the whole
//! suite:
//!
//! - **The channels are the shipped ones.** A row that does not fit is a
//!   *verdict*: a `TransformReport` naming a closed reason, never a
//!   `CoreError`, because a refusal means "could not evaluate at all" (A9). The
//!   only `CoreError`s these cases expect are the entry refusals — a patch as
//!   the base argument, a pin that does not match the manifest, a policy naming
//!   its own base, a base that binds one action twice — which reuse existing
//!   variants (A1, A7, A9).
//! - **"Never mangles" is structural.** A row is atomic: its writes land only if
//!   every check passes, so a failing row leaves the working layout byte-equal
//!   (A1). The suite pins that on every failure shape, not only once.
//! - **The match rule is stated once** (Chosen design → The stack): the match set
//!   is the registry ids the pattern matches, sorted byte-order; the k-th matched
//!   id pairs with the k-th target in row order; the produced `set` preserves the
//!   source base's element order in place. Sorted containers are also what makes
//!   which offender a report names deterministic, so the offender assertions are
//!   part of the determinism surface, not incidental.
//! - **One binding per action, one answer per key.** The stack reads every
//!   precondition off the one layout and writes into that same layout, so a base
//!   that binds one action twice is an entry refusal, and an occupancy question
//!   is answered by every action holding the key rather than the first one
//!   (`schemas/policy/v1/spec.md` → Applying the stack, as amended by the
//!   accepted round-1 security finding, `reviews/code/security-r1c0.md` major).
//!   Two bases in the shape that finding reported are pinned below: they must
//!   never read as `Applied` over a change no player can feel.
//! - **The data is loaded, never guessed.** Hands are the row's declared inputs
//!   to the predicate, not facts derived from the base (A4, A21); grades and grid
//!   coordinates come only from the shipped contract tree (A20, A24); the
//!   untouchable guard reads only the loaded manifest's constraints (A6).
//!
//! Unit-level: loaded contract data and scratch documents in a temp directory,
//! no network and no live server (repo practice, ADR-004).
//!
//! # Workspace state at this commit
//!
//! The test tree was written ahead of the code it pins (Rollout 4). The
//! implementation joined `crates/transforms` to the workspace's `crates/*`
//! glob and shipped the surface this suite and its sibling import:
//! `bindforge_transforms::{apply, Applied, Outcome, TransformReport,
//! NotApplicableReason}` in this directory, and
//! `bindforge_core::{Policy, Transform, TagPattern, load_policy,
//! MAX_POLICY_TRANSFORMS, MAX_RELOCATE_TARGETS}` in
//! `crates/core/tests/policy.rs`. Those imports are the design's contract
//! stated ahead of the code — that posture is unchanged: when a case here
//! fails, the fix is in the surface or behavior that answers it, never in
//! these imports or the assertions below.

mod common;

use bindforge_core::{Document, ProfileKind};
use bindforge_transforms::{Outcome, apply};
use common::{
    BASE_ID, DEFAULT_SET, PIN, POLICY_ID, ScratchDir, assert_entry_refusal, assert_offender_names,
    assert_row_applied, assert_row_not_applicable, base_text, base_text_pinned, layout,
    load_default_registry, load_registry, manifest_of, manifest_text, patch_text, policy_of,
    policy_text, profile_of, report_at, shipped_inputs,
};

/// The relocate row this suite reuses: `move.steer-*` is the registry's two-id
/// family, the hands are the row's declared predicate inputs, bound `0` is the
/// tightest point of the v2 scale, and the targets `keyboard:s keyboard:d` put
/// the cluster one column right — the translation the shipped placement suite
/// already pins legal left-to-left at bound 0.
const RELOCATE_STEER: &str = "relocate-cluster move.steer-* left left 0 keyboard:s keyboard:d";

/// The layout every "the row no-ops" assertion compares against.
fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(action, input)| (action.to_string(), input.to_string()))
        .collect()
}

/// proves: design 015's Test strategy #4 + Chosen design → The stack (the match
/// rule) — a contiguous, bound-respecting move applies. The match set
/// `[move.steer-left, move.steer-right]` (byte order) pairs with the targets in
/// row order `[keyboard:s, keyboard:d]`, so `move.steer-left` walks
/// `keyboard:a → keyboard:s` and `move.steer-right` walks `keyboard:s →
/// keyboard:d`: one translation of `(+1, 0)`, whose landing keys are `home` for
/// the left hand, so bound 0 is met. Walking onto the cluster's own vacated keys
/// is legal, and the produced `set` keeps the source base's element order — the
/// base spells `move.steer-right` first, so the output does too.
#[test]
fn relocate_cluster_applies_a_contiguous_legal_move_pairing_by_the_match_rule() {
    let scratch = ScratchDir::new("relocate-applies");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a row that fits this base must apply, not refuse");

    assert_row_applied(&applied.reports, 0, "relocate-cluster");
    assert_eq!(
        layout(&applied.base),
        pairs(&[
            ("move.steer-right", "keyboard:d"),
            ("move.steer-left", "keyboard:s"),
            ("combat.fire", "mouse:left"),
            ("ui.menu", "mouse:right"),
        ]),
        "the matched ids carry their paired targets and the untouched rows stay in place, in the \
         source base's element order"
    );
}

/// proves: design 015's Test strategy #4 + Options considered (Tags) — a tag
/// that matches no registry id is an inapplicable report naming the emptiness,
/// never a silent no-op transform: `move.jump-*` is grammar-valid and matches
/// nothing in the loaded registry, so the row reports rather than doing nothing
/// quietly (A1, and the research's tag-matching risk).
#[test]
fn relocate_cluster_with_a_tag_matching_no_registry_id_reports_unmatched_tags() {
    let scratch = ScratchDir::new("relocate-unmatched");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &["relocate-cluster move.jump-* left right 2 keyboard:i keyboard:k"],
    ));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("an inapplicable row is a verdict, not a refusal (A9)");

    assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "unmatched-tags");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "an inapplicable row no-ops: the working layout is byte-equal with the source"
    );
}

/// proves: design 015's Test strategy #4 — a matched action that is unbound in
/// the working layout reports `unbound-action` naming it: the cluster the row
/// describes is not the cluster this base has, and moving an action that has no
/// key is exactly the mangling the ADR refuses.
#[test]
fn relocate_cluster_with_an_unbound_matched_action_reports_it_by_name() {
    let scratch = ScratchDir::new("relocate-unbound");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &["move.steer-left keyboard:a", "combat.fire mouse:left"],
    ));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a row that does not fit answers with a verdict");

    let offender =
        assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "unbound-action");
    assert_offender_names(offender, &["move.steer-right"], "unbound-action");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "the row never partially applies its one bound pair"
    );
}

/// proves: design 015's Test strategy #4 + Chosen design → The stack (the match
/// rule) — a match count that is not the target count reports `match-count`: the
/// k-th-with-k-th pairing has no answer for the leftover, and dropping or
/// inventing a target would be the transform deciding the base's shape.
#[test]
fn relocate_cluster_with_a_match_count_that_is_not_the_target_count_reports_match_count() {
    let scratch = ScratchDir::new("relocate-match-count");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &["relocate-cluster move.steer-* left left 0 keyboard:s"],
    ));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a count mismatch is a verdict, not a refusal");

    assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "match-count");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "no part of a mispaired cluster moves"
    );
}

/// proves: design 015's Test strategy #4 + A5 — a source bound to a `gamepad:`
/// input reports `not-keyboard` naming it. The reach grades and the grid cover
/// `keyboard:*` only, so this is the transform's own check, not the predicate's
/// definedness refusal, which stays reserved for data gaps.
#[test]
fn relocate_cluster_with_a_gamepad_bound_source_reports_not_keyboard() {
    let scratch = ScratchDir::new("relocate-not-keyboard");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &[
            "move.steer-left gamepad:dpad-up",
            "move.steer-right keyboard:s",
            "combat.fire mouse:left",
        ],
    ));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("an inapplicable row is a verdict (A9)");

    let offender =
        assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "not-keyboard");
    assert_offender_names(offender, &["gamepad:dpad-up"], "not-keyboard");
}

/// proves: design 015's Test strategy #4 + A21 — source keys that are not
/// connected by rook-adjacency in the loaded grid report `not-contiguous`. This
/// is the grid computation core does not ship: the cluster is a concept this
/// design builds over base bindings and grid coordinates, so a base whose
/// `move.steer-*` sits on `keyboard:a` and `keyboard:f` — three columns apart —
/// has no cluster to relocate.
#[test]
fn relocate_cluster_with_disconnected_source_keys_reports_not_contiguous() {
    let scratch = ScratchDir::new("relocate-not-contiguous");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &[
            "move.steer-left keyboard:a",
            "move.steer-right keyboard:f",
            "combat.fire mouse:left",
            "ui.menu mouse:right",
        ],
    ));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a base that does not carry the cluster the row names is a verdict");

    assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "not-contiguous");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "a disconnected cluster is reported, not mirrored or smeared across the grid"
    );
}

/// proves: design 015's Test strategy #4 + Chosen design → The stack — a target
/// already bound to an action *outside* the match set reports `occupied-target`
/// naming it, while the cluster's own source keys stay legal to walk onto: the
/// distinction is exactly what a naive "is the key free" test would get wrong.
#[test]
fn relocate_cluster_with_a_foreign_occupied_target_reports_it_by_name() {
    let scratch = ScratchDir::new("relocate-occupied");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &[
            "move.steer-left keyboard:a",
            "move.steer-right keyboard:s",
            "combat.fire keyboard:d",
        ],
    ));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("an occupied target is a verdict, not a refusal");

    let offender =
        assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "occupied-target");
    assert_offender_names(offender, &["keyboard:d"], "occupied-target");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "the cluster does not displace an action the row never named"
    );
}

/// proves: design 015's Test strategy #4 + Chosen design → The stack (the match
/// rule's occupancy clause, as amended by the accepted round-1 security finding
/// `reviews/code/security-r1c0.md` major) — occupancy asks *every* action holding
/// the target key, not the first answer in `set` order. This is the finding's
/// bypass shape: `keyboard:s` is the cluster's own source key, so walking onto it
/// is the legal overlap the design allows, and the first holder in array order is
/// `move.steer-right`, inside the match set — but `combat.fire` holds that same
/// key too, and a first-match reading never sees it. `profile/v2` accepts a base
/// binding one input to two actions (that is why this base is no entry refusal),
/// so the row must report rather than land a matched action on a key a
/// never-named action still holds. Every other precondition passes here — the
/// move is the same one-column translation the suite's first case pins legal —
/// so `occupied-target` is the only reason this row can report.
#[test]
fn relocate_cluster_occupancy_asks_every_action_holding_the_target_key() {
    let scratch = ScratchDir::new("relocate-occupied-second-holder");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &[
            "move.steer-left keyboard:a",
            "move.steer-right keyboard:s",
            "combat.fire keyboard:s",
            "ui.menu mouse:right",
        ],
    ));
    // Byte order pairs `move.steer-left` with the row's first target and
    // `move.steer-right` with its second, so `keyboard:s` is a landing key held
    // both by a matched action (that overlap is legal) and by `combat.fire`.
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a foreign second holder on a landing key is a verdict, not a refusal (A9)");

    let offender =
        assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "occupied-target");
    assert_offender_names(offender, &["keyboard:s"], "occupied-target");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "the row that cannot land writes nothing: `combat.fire` keeps `keyboard:s` and the \
         cluster keeps its keys"
    );
}

/// proves: design 015's Test strategy #4 + A4 — a move that violates the declared
/// bound reports `placement-illegal` carrying the `PlacementCheck` itself as the
/// evidence. `keyboard:a` is `home` and `keyboard:;` is `off-reach` for the left
/// hand — the four-rank degradation the shipped placement suite pins — so bound
/// `0` refuses it, and the verdict can say *why*: `shape_ok` true (it is one
/// translation) with a degradation of `4` past the bound, which is exactly the
/// answer a bool cannot give.
#[test]
fn relocate_cluster_with_a_bound_violating_move_reports_placement_illegal_with_the_check() {
    let scratch = ScratchDir::new("relocate-placement-illegal");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &["move.steer-left keyboard:a", "move.steer-right keyboard:s"],
    ));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &["relocate-cluster move.steer-left left left 0 keyboard:;"],
    ));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("an illegal placement is a verdict on the `PlacementCheck` channel (A9)");

    let report = report_at(&applied.reports, 0, "relocate-cluster");
    match &report.outcome {
        Outcome::NotApplicable(verdict) => {
            assert_eq!(
                verdict.reason.name(),
                "placement-illegal",
                "the bound is what stopped this row"
            );
            let check = verdict.check.as_ref().expect(
                "a `placement-illegal` verdict carries the `PlacementCheck` as its evidence — the \
                 answer a bool cannot give (A4)",
            );
            assert!(!check.legal, "the move is illegal at `GradeBound(0)`");
            assert!(
                check.shape_ok,
                "it fails the bound, not the shape: a single move is one translation"
            );
            assert_eq!(
                check.degradations,
                vec![4],
                "`keyboard:a` is `home` (rank 0) and `keyboard:;` is `off-reach` (rank 4) for the \
                 left hand"
            );
            assert_eq!(
                check.worst_degradation(),
                Some(4),
                "the report answers the evidence the proposal names: `shape_ok` plus the worst \
                 degradation, the shipped `PlacementCheck` surface this design leaves untouched"
            );
        }
        Outcome::Applied => panic!(
            "a four-rank degradation past a declared bound of 0 must report `placement-illegal`, \
             not `Applied`"
        ),
    }
    assert_eq!(
        layout(&applied.base),
        pairs(&[
            ("move.steer-left", "keyboard:a"),
            ("move.steer-right", "keyboard:s"),
        ]),
        "the illegal move lands nowhere"
    );
}

/// proves: design 015's Test strategy #4 + Chosen design → The stack (the
/// untouchable guard) — the ADR's "hard-docked or exclusive-group key is never a
/// target" made load-bearing, and unconditional rather than author-declared: a
/// row that touches an action a `docked-actions` row names, or reads or writes an
/// input appearing in `docked-actions`, `dead-keys`, or an `exclusive-groups`
/// member, reports `reserved-input` naming the input. Every mature system carries
/// such a set; this one is loaded data (A6, A23).
#[test]
fn relocate_cluster_touching_reserved_manifest_data_reports_reserved_input() {
    let (tables, geometry) = shipped_inputs();
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    for (label, constraints, needles) in [
        (
            "an exclusive-group member as a source key",
            ["exclusive-groups = [\"keyboard:a keyboard:z\"]".to_string()],
            &["keyboard:a"][..],
        ),
        (
            "a dead key as a target",
            ["dead-keys = [\"keyboard:d\"]".to_string()],
            &["keyboard:d"][..],
        ),
        (
            "a docked input as a target",
            ["docked-actions = [\"combat.fire keyboard:d\"]".to_string()],
            &["keyboard:d"][..],
        ),
        (
            "a docked action inside the match set",
            ["docked-actions = [\"move.steer-right keyboard:m\"]".to_string()],
            &["keyboard:m", "move.steer-right"][..],
        ),
    ] {
        let scratch = ScratchDir::new("relocate-reserved-input");
        let registry = load_default_registry(&scratch);
        let manifest = manifest_of(&manifest_text(
            &constraints.iter().map(String::as_str).collect::<Vec<_>>(),
        ));

        let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
            .expect("a reserved input is a verdict — the guard reports, it does not refuse (A9)");

        let offender =
            assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "reserved-input");
        assert_offender_names(offender, needles, &format!("reserved-input ({label})"));
        assert_eq!(
            layout(&applied.base),
            layout(&base),
            "{label}: the reserved data keeps its binding and the cluster keeps its keys"
        );
    }
}

/// proves: design 015's Test strategy #4 (atomicity) + A1 — a row that does not
/// fit leaves the working layout byte-equal: the row above it lands in full, the
/// failing row lands in none of its parts, so "never mangles" is structural.
#[test]
fn a_row_that_does_not_fit_leaves_the_working_layout_byte_equal() {
    let scratch = ScratchDir::new("apply-atomicity");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));

    let one_row = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));
    let after = apply(&one_row, &base, &manifest, &registry, &tables, &geometry)
        .expect("the first row fits");

    let two_rows = policy_of(&policy_text(
        POLICY_ID,
        &[
            RELOCATE_STEER,
            "relocate-cluster move.steer-* left left 0 keyboard:a keyboard:space",
        ],
    ));
    let through = apply(&two_rows, &base, &manifest, &registry, &tables, &geometry)
        .expect("a stack whose second row does not fit still answers with a base and reports");

    assert_row_applied(&through.reports, 0, "relocate-cluster");
    assert_row_not_applicable(&through.reports, 1, "relocate-cluster", "placement-illegal");
    assert_eq!(
        layout(&through.base),
        layout(&after.base),
        "the failing row's pairs are absent from a layout that is otherwise exactly the first \
         row's: a row never partially applies"
    );

    let through_bytes = bindforge_core::serialize(&Document::Profile(through.base));
    let after_bytes = bindforge_core::serialize(&Document::Profile(after.base));
    assert_eq!(
        through_bytes, after_bytes,
        "byte-equal, not merely equal in the fields this case happens to read"
    );
}

/// proves: design 015's Test strategy #5 + A2 — a bound `discrete-lateral` pair
/// exchanges: `swap-polarity` is the consumer 011's enum comment pre-announced,
/// and its output is the two actions' bindings swapped, in place.
#[test]
fn swap_polarity_exchanges_a_bound_discrete_lateral_pair() {
    let scratch = ScratchDir::new("polarity-applies");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &[
            "move.steer-left keyboard:a",
            "move.steer-right keyboard:d",
            "combat.fire mouse:left",
        ],
    ));
    let policy = policy_of(&policy_text(POLICY_ID, &["swap-polarity move.steer-*"]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a bound discrete-lateral pair is exactly what this row is for");

    assert_row_applied(&applied.reports, 0, "swap-polarity");
    assert_eq!(
        layout(&applied.base),
        pairs(&[
            ("move.steer-left", "keyboard:d"),
            ("move.steer-right", "keyboard:a"),
            ("combat.fire", "mouse:left"),
        ]),
        "the two actions exchange their bindings and nothing else moves"
    );
}

/// proves: design 015's Test strategy #5 + A2, A3 — the camera rule has teeth: a
/// match that includes a `continuous-lateral` id reports `axis-kind` naming that
/// id. `DiscreteLateral` is "the only kind a polarity flip may touch", and
/// `ContinuousLateral` is camera look, "whose inversion is a game option, never a
/// key swap" — the enum comment made load-bearing.
#[test]
fn swap_polarity_including_a_continuous_lateral_id_reports_axis_kind_naming_it() {
    let scratch = ScratchDir::new("polarity-axis-kind");
    let registry = load_registry(
        &scratch,
        &["move.steer-left"],
        &["move.steer-right"],
        &["combat.fire", "ui.menu"],
    );
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &["swap-polarity move.steer-*"]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("an axis-kind mismatch is a verdict (A9)");

    let offender = assert_row_not_applicable(&applied.reports, 0, "swap-polarity", "axis-kind");
    assert_offender_names(offender, &["move.steer-right"], "axis-kind");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "an inversion that belongs to the game's options is never written into a base"
    );
}

/// proves: design 015's Test strategy #5 + Chosen design → The stack — a match
/// that is not exactly two ids reports `match-count`: `swap-polarity` is a
/// polarity, so one id or three has no answer, and guessing one would be the
/// transform deciding the base's shape.
#[test]
fn swap_polarity_with_a_match_that_is_not_two_ids_reports_match_count() {
    let scratch = ScratchDir::new("polarity-match-count");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &["swap-polarity move.steer-left"]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a one-id match is a verdict");

    assert_row_not_applicable(&applied.reports, 0, "swap-polarity", "match-count");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "the tag matches one id, so nothing is flipped"
    );
}

/// proves: design 015's Test strategy #5 + A7, A10 — an unbound side stays
/// unbound: `swap-polarity` carries 014's `swap` semantics, which the shipped
/// `profile/v2` corpus already pins, so a row over a half-bound pair cannot
/// invent a binding for the half that has none.
#[test]
fn swap_polarity_leaves_an_unbound_side_unbound() {
    let scratch = ScratchDir::new("polarity-unbound");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &["move.steer-left keyboard:a", "combat.fire mouse:left"],
    ));
    let policy = policy_of(&policy_text(POLICY_ID, &["swap-polarity move.steer-*"]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a half-bound pair is the `swap` algebra the patch level already ships");

    assert_row_applied(&applied.reports, 0, "swap-polarity");
    assert!(
        !layout(&applied.base)
            .iter()
            .any(|(action, _)| action == "move.steer-right"),
        "`move.steer-right` was unbound in the source base, and an unbound side stays unbound \
         (A10): the produced base must not name a binding for it, got {:?}",
        layout(&applied.base)
    );
}

/// proves: design 015's Test strategy #5 + A10 — the same row applied twice is
/// the identity, mirroring the shipped `swap-pair-applied-twice` algebra: rows
/// compose the way delta elements already do, so an author can stack a polarity
/// flip twice without mangling the base.
#[test]
fn swap_polarity_applied_twice_is_the_identity() {
    let scratch = ScratchDir::new("polarity-twice");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &["move.steer-left keyboard:a", "move.steer-right keyboard:d"],
    ));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &["swap-polarity move.steer-*", "swap-polarity move.steer-*"],
    ));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("both rows fit: the second sees the first's output and flips it back");

    assert_row_applied(&applied.reports, 0, "swap-polarity");
    assert_row_applied(&applied.reports, 1, "swap-polarity");
    assert_eq!(
        layout(&applied.base),
        pairs(&[
            ("move.steer-left", "keyboard:a"),
            ("move.steer-right", "keyboard:d"),
        ]),
        "two polarity flips are no flip at all — the algebra the patch level already pins (A10)"
    );
}

/// proves: design 015's Test strategy #6 + A5, A19 — the actions bound to
/// `mouse:left` and `mouse:right` exchange, and the exchange *lands as `set`
/// elements* on the produced base. The row is input-wise at read time and
/// action-wise where it materializes, which is the answer to the research's
/// "`swap` deltas are action-pair-wise, the mouse swap is input-wise" — and no
/// geometry or grade is consulted, because nothing here moves on a grid.
#[test]
fn swap_mouse_buttons_exchanges_the_bound_actions_and_lands_as_set_elements() {
    let scratch = ScratchDir::new("mouse-applies");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &["swap-mouse-buttons"]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("both mouse members are bound, so this row applies");

    assert_row_applied(&applied.reports, 0, "swap-mouse-buttons");
    assert_eq!(
        layout(&applied.base),
        pairs(&[
            ("move.steer-right", "keyboard:s"),
            ("move.steer-left", "keyboard:a"),
            ("combat.fire", "mouse:right"),
            ("ui.menu", "mouse:left"),
        ]),
        "the two actions trade inputs in place, and the produced base expresses that as `set`"
    );
    assert!(
        applied.base.swap.is_empty(),
        "a produced base carries no `swap` element at all: its layout *is* its `set` list (A7), \
         so the input-wise swap materializes action-wise"
    );
}

/// proves: design 015's Test strategy #6 — one unbound mouse member reports
/// `unbound-input` naming the mouse id: the pair the row is defined over is not
/// the pair this base has, and swapping one button into nothing is the mangling
/// the ADR refuses.
#[test]
fn swap_mouse_buttons_with_an_unbound_mouse_input_reports_it_by_name() {
    let scratch = ScratchDir::new("mouse-unbound");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &["ui.menu mouse:right", "move.steer-left keyboard:a"],
    ));
    let policy = policy_of(&policy_text(POLICY_ID, &["swap-mouse-buttons"]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("an unbound mouse member is a verdict (A9)");

    let offender =
        assert_row_not_applicable(&applied.reports, 0, "swap-mouse-buttons", "unbound-input");
    assert_offender_names(offender, &["mouse:left"], "unbound-input");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "`ui.menu` keeps its `mouse:right`: a row that does not fit writes nothing"
    );
}

/// proves: design 015's Test strategy #6 + A6 — the untouchable guard covers the
/// mouse too: a docked mouse input is `reserved-input`, and this row's whole
/// precondition set is vocabulary plus manifest constraints, since mouse ids
/// carry no coordinates and no grades to check (A5).
#[test]
fn swap_mouse_buttons_on_a_docked_mouse_input_reports_reserved_input() {
    let scratch = ScratchDir::new("mouse-reserved");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[
        "docked-actions = [\"ui.menu mouse:right\"]",
    ]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &["swap-mouse-buttons"]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a reserved input is a verdict, not a refusal (A9)");

    let offender =
        assert_row_not_applicable(&applied.reports, 0, "swap-mouse-buttons", "reserved-input");
    assert_offender_names(offender, &["mouse:right"], "reserved-input");
    assert_eq!(
        layout(&applied.base),
        layout(&base),
        "the game hard-docks that button; the row reports and no-ops"
    );
}

/// proves: design 015's Test strategy #7 + Options considered (Stack order) —
/// rows apply sequentially and each row's preconditions see the output of the
/// rows above it. `keyboard:q` is occupied by `combat.fire` in the untouched
/// base, so a base-relative precondition would report `occupied-target` for the
/// second row; it is free because the first row moved that action off it, and the
/// second row applies.
#[test]
fn a_second_row_sees_the_layout_the_row_above_it_produced() {
    let scratch = ScratchDir::new("stack-sequential");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &[
            "move.steer-left keyboard:a",
            "move.steer-right keyboard:s",
            "combat.fire keyboard:q",
        ],
    ));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &[
            "relocate-cluster combat.fire left left 4 keyboard:e",
            "relocate-cluster move.steer-* left left 4 keyboard:q keyboard:w",
        ],
    ));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("the second row fits the layout the first row produced");

    assert_row_applied(&applied.reports, 0, "relocate-cluster");
    assert_row_applied(&applied.reports, 1, "relocate-cluster");
    assert_eq!(
        layout(&applied.base),
        pairs(&[
            ("move.steer-left", "keyboard:q"),
            ("move.steer-right", "keyboard:w"),
            ("combat.fire", "keyboard:e"),
        ]),
        "array order is application order: `keyboard:q` and `keyboard:w` are reachable only \
         because the row above moved `combat.fire` off `keyboard:q`"
    );
}

/// proves: design 015's Test strategy #7 + Options considered (Stack order) —
/// the other half of the same rule: a row can make the row below it inapplicable.
/// `keyboard:e` is free in the untouched base and occupied by `combat.fire` after
/// the first row moved it there, so the second row reports `occupied-target`
/// naming it — and reports rather than displacing the action that got in the way.
#[test]
fn a_row_can_break_the_preconditions_of_the_row_below_it() {
    let scratch = ScratchDir::new("stack-breaks");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &[
            "move.steer-left keyboard:a",
            "move.steer-right keyboard:s",
            "combat.fire keyboard:q",
        ],
    ));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &[
            "relocate-cluster combat.fire left left 4 keyboard:e",
            "relocate-cluster move.steer-* left left 0 keyboard:d keyboard:e",
        ],
    ));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("an inapplicable row in a stack is a verdict, not a refusal");

    assert_row_applied(&applied.reports, 0, "relocate-cluster");
    let offender =
        assert_row_not_applicable(&applied.reports, 1, "relocate-cluster", "occupied-target");
    assert_offender_names(offender, &["keyboard:e"], "occupied-target");
    assert_eq!(
        layout(&applied.base),
        pairs(&[
            ("move.steer-left", "keyboard:a"),
            ("move.steer-right", "keyboard:s"),
            ("combat.fire", "keyboard:e"),
        ]),
        "the first row's output stands and the second row wrote nothing"
    );
}

/// proves: design 015's Test strategy #7 + A1 — a policy in which no row fits
/// reports every row and produces a base equal to the source except `id` and
/// `generated-by`: the three reasons below are three different named verdicts, and
/// the player's layout comes out exactly as it went in.
#[test]
fn an_all_inapplicable_policy_reports_every_row_and_produces_the_source_layout() {
    let scratch = ScratchDir::new("all-inapplicable");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &["move.steer-left keyboard:a", "move.steer-right keyboard:f"],
    ));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &[
            "relocate-cluster move.jump-* left right 2 keyboard:i keyboard:k",
            "swap-mouse-buttons",
            "relocate-cluster move.steer-* left left 0 keyboard:c keyboard:d",
        ],
    ));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a policy whose every row fails still answers with a base and reports (A1)");

    assert_row_not_applicable(&applied.reports, 0, "relocate-cluster", "unmatched-tags");
    let offender =
        assert_row_not_applicable(&applied.reports, 1, "swap-mouse-buttons", "unbound-input");
    assert_offender_names(offender, &["mouse:left"], "unbound-input");
    assert_row_not_applicable(&applied.reports, 2, "relocate-cluster", "not-contiguous");

    assert_eq!(
        layout(&applied.base),
        pairs(&[
            ("move.steer-left", "keyboard:a"),
            ("move.steer-right", "keyboard:f"),
        ]),
        "nothing applied, so the produced base's layout is the source's, element for element"
    );
    assert_eq!(
        applied.base.id, POLICY_ID,
        "the produced base is named by the policy"
    );
    assert_eq!(
        applied.base.generated_by.as_deref(),
        Some("policy:probe-policy"),
        "and carries the policy's provenance label, which is never consumed as trust"
    );
}

/// proves: design 015's Test strategy #7 + The report — `report.index` is the
/// row's position in `transforms`, and one report answers each row: the shape
/// ADR-017's CLI and the hub display is keyed to the row an author wrote, so an
/// index that drifted from the document would point a player at the wrong row.
#[test]
fn every_report_names_the_row_position_it_came_from() {
    let scratch = ScratchDir::new("report-index");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &[
            RELOCATE_STEER,
            "swap-polarity move.jump-*",
            "swap-mouse-buttons",
        ],
    ));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("a mixed stack answers with one report per row");

    assert_eq!(
        applied
            .reports
            .iter()
            .map(|report| (report.index, report.kind))
            .collect::<Vec<_>>(),
        vec![
            (0usize, "relocate-cluster"),
            (1usize, "swap-polarity"),
            (2usize, "swap-mouse-buttons"),
        ],
        "one report per row, carrying the row's own position and the contract's spelling of its \
         kind"
    );
    assert_row_applied(&applied.reports, 0, "relocate-cluster");
    assert_row_not_applicable(&applied.reports, 1, "swap-polarity", "unmatched-tags");
    assert_row_applied(&applied.reports, 2, "swap-mouse-buttons");
}

/// proves: design 015's Test strategy #8 + Chosen design → The output — the
/// produced document is an ordinary `profile/v2` base: `kind = "base"`, `derives`
/// empty, no `clear`/`swap` element, `for` copied from the source pin, and
/// `generated-by = "policy:<policy-id>"` — and it re-parses as one, so it is a
/// chain citizen gated by validation rather than trusted by provenance (A7, A18).
#[test]
fn the_produced_base_is_an_ordinary_profile_v2_base_document() {
    let scratch = ScratchDir::new("produced-base");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let applied = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("the row applies and produces a base");

    assert!(
        matches!(applied.base.kind, Some(ProfileKind::Base)),
        "the output is `kind = \"base\"`, got {:?}",
        applied.base.kind
    );
    assert_eq!(
        applied.base.id, POLICY_ID,
        "the produced base carries the policy's id"
    );
    assert!(
        applied.base.derives.is_empty(),
        "a base derives nothing: it is the bottom of the chain (A7)"
    );
    assert!(
        applied.base.clear.is_empty() && applied.base.swap.is_empty(),
        "a base has no `clear` or `swap` element — its layout *is* its `set` list (A7)"
    );
    assert_eq!(
        applied.base.for_manifest, PIN,
        "`for` is copied from the source base, whose pin equality already joined it to the manifest"
    );
    assert_eq!(
        applied.base.generated_by.as_deref(),
        Some("policy:probe-policy"),
        "the provenance label is `policy:<id>`, grammar-legal inside 128 octets and never a \
         selection key (A18)"
    );

    let written = bindforge_core::serialize(&Document::Profile(applied.base));
    let reloaded = match bindforge_core::parse(&written) {
        Ok(Document::Profile(profile)) => profile,
        _ => panic!("the produced base must parse as a profile document, under `profile/v2` rules"),
    };
    assert!(
        matches!(reloaded.kind, Some(ProfileKind::Base)),
        "and it re-parses as a base, so a consumer can put it in a chain and gate it by validation"
    );
    assert!(
        reloaded.derives.is_empty() && reloaded.clear.is_empty() && reloaded.swap.is_empty(),
        "the base shape survives the round trip, not only the in-memory value"
    );
    assert_eq!(
        reloaded.generated_by.as_deref(),
        Some("policy:probe-policy"),
        "the label survives too"
    );
}

/// proves: design 015's Test strategy #8 + Chosen design → The output
/// (Regeneration) — same inputs, byte-identical output: `apply` is a pure
/// function over parsed values with sorted containers, with no clock and no
/// randomness. This is the rule the research's chain question turns on — a
/// personal patch survives base regeneration only if regeneration is
/// deterministic.
#[test]
fn applying_the_same_policy_twice_answers_serialize_identical_bytes() {
    let scratch = ScratchDir::new("determinism");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(
        POLICY_ID,
        &[
            RELOCATE_STEER,
            "swap-polarity move.steer-*",
            "swap-mouse-buttons",
        ],
    ));

    let first =
        apply(&policy, &base, &manifest, &registry, &tables, &geometry).expect("the stack answers");
    let second = apply(&policy, &base, &manifest, &registry, &tables, &geometry)
        .expect("and answers again, identically");

    let first_bytes = bindforge_core::serialize(&Document::Profile(first.base));
    let second_bytes = bindforge_core::serialize(&Document::Profile(second.base));
    assert_eq!(
        first_bytes, second_bytes,
        "two applies of one policy over one base must be byte-identical — regeneration is a write \
         a consumer can rely on, not a re-draw"
    );
}

/// proves: design 015's Test strategy #8 + Chosen design → The stack (entry
/// refusals) — `policy.id == base.id` refuses on the `CoreError` channel naming
/// the id: a produced base cannot name its own input, because a chain directory
/// holds one file per id and the produced document would overwrite the base it
/// was built from.
#[test]
fn a_policy_id_colliding_with_the_base_id_refuses_naming_the_id() {
    let scratch = ScratchDir::new("entry-id-collision");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(BASE_ID, &DEFAULT_SET));
    let policy = policy_of(&policy_text(BASE_ID, &[RELOCATE_STEER]));

    let Err(err) = apply(&policy, &base, &manifest, &registry, &tables, &geometry) else {
        panic!(
            "a policy named `probe-base` applied to the base `probe-base` must refuse: the \
                produced base would name its own input"
        );
    };
    assert_entry_refusal(&err, &["probe-base"], "a policy naming its own base");
}

/// proves: design 015's Test strategy #8 + A1 — a patch as the base argument
/// refuses: transforms apply to bases only, and the personal patch sits above the
/// stack, never opened and never rewritten. The refusal is a `CoreError`, because
/// the whole application could not be evaluated — not a row verdict (A9).
#[test]
fn a_patch_as_the_base_argument_refuses() {
    let scratch = ScratchDir::new("entry-patch-base");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let patched = profile_of(&patch_text("probe-patch", &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let Err(err) = apply(&policy, &patched, &manifest, &registry, &tables, &geometry) else {
        panic!(
            "`apply` over a `kind = \"patch\"` document must refuse: transforming a patch would \
                touch the player's layer, the ADR's boundary"
        );
    };
    assert_entry_refusal(&err, &["patch", "kind"], "a patch as the base argument");
}

/// proves: design 015's Test strategy #8 + A7 — 014's pin-equality rule,
/// cross-referenced rather than restated: a base pinned to a manifest revision
/// other than the one it is applied against refuses. Nothing re-resolves under
/// changed game facts, and the same rule is what makes a consumer regenerate a
/// chain by bumping pins as a review.
#[test]
fn a_base_pinned_to_another_manifest_revision_refuses() {
    let scratch = ScratchDir::new("entry-pin-mismatch");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text_pinned(BASE_ID, "probe-game@2", &DEFAULT_SET));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let Err(err) = apply(&policy, &base, &manifest, &registry, &tables, &geometry) else {
        panic!(
            "a base pinned `probe-game@2` applied against the manifest answering `probe-game@1` \
             must refuse (014's pin equality)"
        );
    };
    assert_entry_refusal(
        &err,
        &["probe-game@2", "probe-game@1"],
        "a base whose pin differs from the manifest",
    );
}

/// proves: design 015's Test strategy #8 (entry refusals), as amended by the
/// accepted round-1 security finding `reviews/code/security-r1c0.md` (major) — a
/// base whose `set` binds one action twice refuses on the `CoreError` channel
/// naming the duplicated action. `profile/v2` calls a repeated action id data and
/// lets chain resolution settle it last-writer-wins, so this document parses and
/// resolves; but a stack reads every precondition off one layout and writes into
/// that same layout. Two copies of one binding are two layouts to disagree about:
/// the preconditions would answer for `keyboard:a`, the write would land on it,
/// and the trailing `keyboard:z` copy is what the player would actually feel — a
/// row reporting `Applied` over a change nobody can feel, which is the shape the
/// finding reported. The same refusal is what makes a duplicated binding unable to
/// turn a `swap-polarity` or `swap-mouse-buttons` row into a no-op reported as
/// applied. It is an entry refusal, not a row verdict — the whole application
/// could not be evaluated — and it reuses a shipped variant (A9).
#[test]
fn a_base_binding_one_action_twice_refuses_naming_the_duplicated_action() {
    let scratch = ScratchDir::new("entry-duplicate-action");
    let registry = load_default_registry(&scratch);
    let (tables, geometry) = shipped_inputs();
    let manifest = manifest_of(&manifest_text(&[]));
    let base = profile_of(&base_text(
        BASE_ID,
        &[
            "move.steer-right keyboard:s",
            "move.steer-left keyboard:a",
            "combat.fire mouse:left",
            "ui.menu mouse:right",
            "move.steer-left keyboard:z",
        ],
    ));
    let policy = policy_of(&policy_text(POLICY_ID, &[RELOCATE_STEER]));

    let Err(err) = apply(&policy, &base, &manifest, &registry, &tables, &geometry) else {
        panic!(
            "a base binding `move.steer-left` to `keyboard:a` and again to `keyboard:z` must \
             refuse: the row's preconditions would answer for one copy while `resolve` keeps the \
             other effective"
        );
    };
    assert_entry_refusal(
        &err,
        &["move.steer-left"],
        "a base binding one action twice",
    );
}
