//! The public surface ADR-012 promises `crates/core` re-exports: the input
//! vocabulary names, the graded reach table, and the placement names ADR-015
//! preconditions its transforms on.
//!
//! proves: Chosen design → The model + Placement legality (`lib.rs` "re-exports
//! `InputId`, `InputNamespace`, `InputGrade`, and the placement names below"),
//! read together with A5 — 015 preconditions its transforms on this design's
//! placement-legality predicate, so the *names* are part of what another component
//! compiles against. A rename here is a break for a consumer that has not been
//! written yet, which is exactly the moment a suite should catch it: this target
//! resolves every promised name in type or call position, at the crate root, with
//! nothing but `use bindforge_core::…`.
//!
//! The behavior behind each name is pinned by the module-mirrored suites —
//! `input.rs` (#1), `reach_v2.rs` (#2, #3), `placement.rs` (#4, #5) — so this file
//! adds no rule of its own; it keeps the surface the rule-bearing suites are
//! written through honest.

mod common;

use std::path::PathBuf;

use bindforge_core::{
    CoreError, GradeBound, Hand, InputGrade, InputId, InputNamespace, KeyboardGeometry,
    PlacementCheck, ReachClass, ReachTable, ReachTables,
};
use common::schemas_root;

/// `InputGrade` is the parsed `keys` pair — input id plus class id — that a v2
/// document's `keys` array carries (A11, r2: `InputGrade.class` typed). It reaches
/// this suite only through this signature, which is what pins the re-export.
fn pinned_pair_count(pairs: Vec<InputGrade>) -> usize {
    pairs.len()
}

/// proves: Chosen design → The model + A5 — every name ADR-012 promises is
/// reachable at the crate root, constructible or callable, and connects to the
/// next one: id → grade → geometry → check.
#[test]
fn the_adr_012_names_resolve_at_the_crate_root() {
    // `InputId` and `InputNamespace`: the vocabulary's typed id and its namespace.
    let id: InputId = InputId::parse("keyboard:a").expect("`keyboard:a` is a member");
    assert_eq!(id.namespace(), InputNamespace::Keyboard);

    // `ReachTables` → `ReachTable` → `ReachClass`, and the `InputGrade` the parsed
    // `keys` pair is (A11's typed `InputGrade.class`, r2).
    let reach_dir: PathBuf = schemas_root().join("reach-classes").join("v2");
    let tables: ReachTables = bindforge_core::load_reach_tables(&reach_dir)
        .unwrap_or_else(|err| panic!("the shipped v2 tables must load: {err}"));
    let table: &ReachTable = tables
        .hand(Hand::Left)
        .expect("the v2 data declares a left-hand table");
    let grade_pairs: usize = pinned_pair_count(Vec::new());
    assert_eq!(
        grade_pairs, 0,
        "`InputGrade` names the parsed pair a `keys` element carries"
    );
    let class: &ReachClass = table
        .grade(&id)
        .expect("the left table grades `keyboard:a`");
    assert!(
        !class.id.is_empty(),
        "a class carries the id the table lists"
    );

    // `KeyboardGeometry`, `GradeBound`, `PlacementCheck`, `placement_is_legal`: the
    // placement half ADR-015 gates on.
    let geometry_dir: PathBuf = schemas_root().join("keyboard-geometry").join("v1");
    let geometry: KeyboardGeometry = bindforge_core::load_geometry(&geometry_dir, "ansi-us")
        .expect("the shipped geometry loads");
    let moves = [(id, InputId::parse("keyboard:d").expect("a glyph member"))];
    let check: PlacementCheck = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &moves,
        GradeBound(4),
        &tables,
        &geometry,
    )
    .expect("both ids are graded and placed, so this is a verdict");
    assert_eq!(check.degradations.len(), 1, "one degradation per move");

    // The pure name rules the loaders use beside `hand_matches_file_name`.
    assert!(
        bindforge_core::geometry_matches_file_name("ansi-us", "ansi-us"),
        "`geometry_matches_file_name` is the post-load name rule"
    );

    // `CoreError` is the refusal type the whole surface answers with: definedness
    // refuses rather than returning a verdict, and no new variant is promised
    // (A11).
    let refused: Result<PlacementCheck, CoreError> = bindforge_core::placement_is_legal(
        Hand::Left,
        Hand::Left,
        &[(
            InputId::parse("mouse:left").expect("a closed `mouse:` member"),
            InputId::parse("keyboard:a").expect("a glyph member"),
        )],
        GradeBound(0),
        &tables,
        &geometry,
    );
    assert!(
        refused.is_err(),
        "an ungraded input is a structured refusal, not a verdict"
    );
}
