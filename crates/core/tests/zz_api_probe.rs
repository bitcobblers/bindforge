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
//!
//! proves: design 013's Test strategy #7 + its Delta inventory (`lib.rs` "re-exports
//! `Manifest`, `ManifestAction`, `ExclusivityGroup`, `DockedAction`, `load_manifest`,
//! and the four budget constants") — the same probe for ADR-013's manifest names,
//! whose behavior `manifest.rs` pins (#1–#5).
//!
//! proves: design 014's Test strategy #8 + its Delta inventory (`lib.rs` gains
//! the `chain_ops` module and its re-exports — `Chain`, `load_chain`,
//! `resolve`, `ResolvedBinding`, `fork`, `diff`, `merge`, `ProfileKind` — and
//! `error.rs` the one `ChainLinkMissing` variant) — the same probe for the
//! chain names, whose behavior `chain_ops.rs` pins (#1–#6).
//!
//! proves: design 015's Test strategy #10 + its Delta inventory (`document.rs`
//! gains the `Policy`/`Transform`/`TagPattern` model, the `Document::Policy` arm,
//! and the two budget constants; `policy.rs` adds `load_policy`; `lib.rs`
//! re-exports them and says so in its public-surface doc) — the same probe for
//! the policy names, whose behavior `policy.rs` pins (#1–#3). `error.rs` gains no
//! variant, so nothing here asks for one (A9).

mod common;

use std::path::PathBuf;

use bindforge_core::{
    ActionId, ActionRegistry, Chain, CoreError, DockedAction, Document, ExclusivityGroup,
    GradeBound, Hand, InputGrade, InputId, InputNamespace, KeyboardGeometry, Manifest,
    ManifestAction, PlacementCheck, Policy, Profile, ProfileKind, ReachClass, ReachTable,
    ReachTables, ResolvedBinding, TagPattern, Transform,
};
use common::{ScratchDir, schemas_root};

/// A well-formed `policy` document: exactly the entries
/// `schemas/policy/v1/spec.md` defines, in the shape its valid corpus fixtures
/// ship — the document every policy name below is resolved through.
const POLICY_TEXT: &str = r#"format = "policy"
version = "v1"
id = "probe-policy"
transforms = ["relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k", "swap-mouse-buttons"]
"#;

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

/// The four manifest budgets, taken as arguments so the exported names are read in
/// value position — the shape `manifest.rs` asserts them through.
fn manifest_budgets(rows: usize, groups: usize, members: usize, octets: usize) -> [usize; 4] {
    [rows, groups, members, octets]
}

/// proves: design 013's Test strategy #7 + its Delta inventory (`lib.rs`
/// re-exports) — every name ADR-013 promises resolves at the crate root and
/// connects to the next one: budgets → registry → load → model. The behavior is
/// `manifest.rs`'s (#1–#5); this target only keeps the names honest, which is what
/// a consumer that has not been written yet compiles against.
#[test]
fn the_adr_013_names_resolve_at_the_crate_root() {
    // The budget family, exported beside `MAX_DOCUMENT_BYTES` and `MAX_NESTING_DEPTH`.
    let budgets = manifest_budgets(
        bindforge_core::MAX_MANIFEST_ROWS,
        bindforge_core::MAX_CONSTRAINT_GROUPS,
        bindforge_core::MAX_GROUP_MEMBERS,
        bindforge_core::MAX_NATIVE_NAME_OCTETS,
    );
    assert!(
        budgets.iter().all(|limit| *limit > 0),
        "the four manifest budgets are the public names for the caps: {budgets:?}"
    );

    // `load_manifest` over a `load_action_registry`ed registry: the join in call
    // position, answering with the typed model.
    let scratch = ScratchDir::new("api-probe-manifest");
    scratch.write(
        "v1/registry.toml",
        r#"format = "action-ids"
version = "v1"
discrete-lateral = ["move.steer-left"]
continuous-lateral = ["aim.camera"]
none = ["combat.fire", "ui.menu"]
"#,
    );
    let registry: ActionRegistry = bindforge_core::load_action_registry(&scratch.path().join("v1"))
        .expect("the scratch registry loads");

    let manifest_text = r#"format = "manifests"
version = "v1"
id = "probe-game"
revision = 1
codec = "unreal-engine-5"
actions = ["move.steer-left Turn left", "aim.camera Mouse camera"]
exclusive-groups = ["keyboard:w keyboard:s"]
dead-keys = ["keyboard:x"]
docked-actions = ["ui.menu keyboard:y"]
presets = ["default"]
"#;
    scratch.write("probe-game/manifest.toml", manifest_text);
    let manifest: Manifest =
        bindforge_core::load_manifest(&scratch.path().join("probe-game"), &registry)
            .expect("a manifest the registry backs must load");
    // `Manifest`, `ManifestAction`, `ExclusivityGroup`, `DockedAction`: the model a
    // consumer reads the game facts through.
    let first_row: &ManifestAction = manifest
        .actions
        .first()
        .expect("the probe manifest names one row");
    assert_eq!(first_row.action.as_str(), "move.steer-left");
    assert_eq!(first_row.native_name, "Turn left");
    let group: &ExclusivityGroup = manifest
        .exclusive_groups
        .first()
        .expect("the probe manifest names one group");
    assert_eq!(group.inputs.len(), 2, "a group names its members");
    let docked: &DockedAction = manifest
        .docked_actions
        .first()
        .expect("the probe manifest names one docked row");
    assert_eq!(docked.input.as_str(), "keyboard:y");
    assert_eq!(
        manifest.dead_keys.first().map(|id| id.as_str()),
        Some("keyboard:x"),
        "`dead_keys` is the typed input-id list ADR-015 looks up"
    );
    assert_eq!(manifest.presets, vec!["default".to_string()]);

    // `CoreError` is still the only refusal type: the location rule refuses the same
    // document sitting in a directory whose name contradicts its `id`, and no new
    // variant is promised.
    scratch.write("not-the-id/manifest.toml", manifest_text);
    let refused: Result<Manifest, CoreError> =
        bindforge_core::load_manifest(&scratch.path().join("not-the-id"), &registry);
    assert!(
        refused.is_err(),
        "a game directory whose name is not the manifest's id is a structured refusal"
    );
}

/// proves: design 014's Test strategy #8 + its Delta inventory (`lib.rs` adds
/// the `chain_ops` module, its re-exports, and the public-surface doc;
/// `error.rs` adds `ChainLinkMissing`) — every name ADR-014 promises resolves
/// at the crate root and connects to the next one: budgets → load → resolve →
/// fork/diff/merge → the new refusal variant. The behavior is
/// `chain_ops.rs`'s (Test strategy #1–#6); this target only keeps the names
/// honest, which is what a consumer that has not been written yet compiles
/// against.
#[test]
fn the_adr_014_names_resolve_at_the_crate_root() {
    // The two new budgets, exported beside the existing family (Rollout 2).
    let budgets = [
        bindforge_core::MAX_PATCH_DELTA_ELEMENTS,
        bindforge_core::MAX_GENERATED_BY_OCTETS,
    ];
    assert!(
        budgets.iter().all(|limit| *limit > 0),
        "the v2 delta budgets are the public names for the caps: {budgets:?}"
    );

    // `load_chain` over a scratch chain directory: the cross-file loader in
    // call position, answering with the `Chain` only it builds.
    let scratch = ScratchDir::new("api-probe-chain");
    scratch.write(
        "v2/probe-base.toml",
        r#"format = "profile"
version = "v2"
id = "probe-base"
kind = "base"
for = "probe-game@1"
set = ["move.steer-left keyboard:w"]
"#,
    );
    scratch.write(
        "v2/probe-patch.toml",
        r#"format = "profile"
version = "v2"
id = "probe-patch"
derives = ["probe-base"]
kind = "patch"
for = "probe-game@1"
set = ["move.steer-left keyboard:e"]
"#,
    );
    let chain: Chain = bindforge_core::load_chain(&scratch.path().join("v2/probe-patch.toml"))
        .expect("a two-link v2 chain in its major directory must load");

    let manifest: Manifest = match bindforge_core::parse(
        r#"format = "manifests"
version = "v1"
id = "probe-game"
revision = 1
codec = "unreal-engine-5"
actions = ["move.steer-left Turn left"]
"#,
    )
    .expect("the probe manifest is a valid document")
    {
        Document::Manifest(manifest) => manifest,
        _ => panic!("a `manifests` document must parse into the manifest arm"),
    };

    // `resolve` → `ResolvedBinding`: the deterministic layout in call
    // position, typed ids on both sides.
    let layout: Vec<ResolvedBinding> =
        bindforge_core::resolve(&chain, &manifest).expect("the probe chain joins its manifest");
    assert_eq!(
        layout.first().map(|binding| binding.action.as_str()),
        Some("move.steer-left"),
        "the layout answers with the manifest's one row"
    );
    assert_eq!(
        layout.first().map(|binding| binding.input.as_str()),
        Some("keyboard:e"),
        "the patch's `set` is the last writer over the base"
    );

    // `fork` → `Profile` + `ProfileKind`: the appended patch, deriving the
    // head.
    let shell: Profile = bindforge_core::fork(&chain, "probe-fork")
        .expect("forking under a fresh slug id must work");
    assert!(
        matches!(shell.kind, Some(ProfileKind::Patch)),
        "`fork` answers with a patch, got {:?}",
        shell.kind
    );
    assert_eq!(shell.derives, vec!["probe-patch".to_string()]);

    // `diff` between two chains over the same base document.
    scratch.write(
        "v2/probe-other.toml",
        r#"format = "profile"
version = "v2"
id = "probe-other"
derives = ["probe-base"]
kind = "patch"
for = "probe-game@1"
clear = ["move.steer-left"]
"#,
    );
    let other: Chain = bindforge_core::load_chain(&scratch.path().join("v2/probe-other.toml"))
        .expect("the second probe chain must load");
    let patch: Profile = bindforge_core::diff(&chain, &other, &manifest, "probe-diff")
        .expect("two chains over one base must diff");
    assert_eq!(patch.derives, vec!["probe-patch".to_string()]);

    // `merge` over two parsed patches: `Profile` in argument position.
    let lower = bindforge_core::parse(
        r#"format = "profile"
version = "v2"
id = "probe-lower"
derives = ["probe-base"]
kind = "patch"
for = "probe-game@1"
set = ["move.steer-left keyboard:w"]
"#,
    )
    .expect("the lower probe patch is valid");
    let upper = bindforge_core::parse(
        r#"format = "profile"
version = "v2"
id = "probe-upper"
derives = ["probe-base"]
kind = "patch"
for = "probe-game@1"
clear = ["move.steer-left"]
"#,
    )
    .expect("the upper probe patch is valid");
    let merged: Profile = bindforge_core::merge(
        lower
            .profile()
            .expect("a profile document parses into the profile arm"),
        upper
            .profile()
            .expect("a profile document parses into the profile arm"),
        "probe-merge",
    )
    .expect("a set under a clear composes to the lone clear");
    assert_eq!(
        merged.clear.first().map(|action| action.as_str()),
        Some("move.steer-left"),
        "`merge` composes the two delta lists into one patch"
    );

    // `CoreError::ChainLinkMissing`: the one new variant, in pattern
    // position — a missing ancestor is this refusal, not an I/O detail (A18).
    scratch.write(
        "v2/probe-orphan.toml",
        r#"format = "profile"
version = "v2"
id = "probe-orphan"
derives = ["probe-absent"]
kind = "patch"
for = "probe-game@1"
set = ["move.steer-left keyboard:w"]
"#,
    );
    let missing: Result<Chain, CoreError> =
        bindforge_core::load_chain(&scratch.path().join("v2/probe-orphan.toml"));
    assert!(
        matches!(missing, Err(CoreError::ChainLinkMissing { .. })),
        "a missing link is the named completeness refusal, got {missing:?}"
    );
}

/// proves: design 015's Test strategy #10 + A13 — the two new budgets resolve at
/// the crate root at the values the contract states, and they are the *per-
/// structure* caps the design chose instead of a new aggregate: 64 mirrors
/// `MAX_CONSTRAINT_GROUPS`, 16 mirrors `MAX_GROUP_MEMBERS`, so the composite
/// `spec.md` states multiplies out of bounds that already exist (nine exported
/// constants become eleven, and no aggregate cap joins them).
#[test]
fn the_adr_015_policy_budgets_resolve_at_the_crate_root() {
    let budgets = [
        bindforge_core::MAX_POLICY_TRANSFORMS,
        bindforge_core::MAX_RELOCATE_TARGETS,
    ];
    assert_eq!(
        budgets,
        [64, 16],
        "the policy budgets are the public names for the caps the contract states: {budgets:?}"
    );
    assert_eq!(
        bindforge_core::MAX_POLICY_TRANSFORMS,
        bindforge_core::MAX_CONSTRAINT_GROUPS,
        "`MAX_POLICY_TRANSFORMS` mirrors the manifest group cap it stands beside"
    );
    assert_eq!(
        bindforge_core::MAX_RELOCATE_TARGETS,
        bindforge_core::MAX_GROUP_MEMBERS,
        "`MAX_RELOCATE_TARGETS` mirrors the manifest member cap it stands beside"
    );
}

/// proves: design 015's Test strategy #10 + its Delta inventory — every policy
/// name resolves at the crate root and connects to the next one: model →
/// matching → load. The behavior is `policy.rs`'s (#1–#3); this target only
/// keeps the names honest, which is what `crates/transforms` — a consumer written
/// in the same run — compiles against.
#[test]
fn the_adr_015_policy_names_resolve_at_the_crate_root() {
    // `Policy`, `Transform`, `TagPattern`: the model, in type position, read out of
    // a parsed document. Matching is grammar, and `matches` is the byte-exact
    // prefixing the stack's match rule is built from (Options: Tags).
    let policy: Policy = match bindforge_core::parse(POLICY_TEXT)
        .expect("a policy document the entry table defines must parse")
    {
        Document::Policy(policy) => policy,
        _ => panic!("a `format = \"policy\"` document must parse into the policy arm"),
    };
    assert_eq!(
        policy.transforms.len(),
        2,
        "the model keeps the stack's array order — order is application order"
    );
    let tags: &TagPattern = match &policy.transforms[0] {
        Transform::RelocateCluster { tags, .. } => tags,
        _ => panic!("the probe stack's first row is a `relocate-cluster` row"),
    };
    assert!(
        tags.matches(&ActionId::parse("move.steer-left").expect("a well-formed action id")),
        "`TagPattern::matches` answers for the prefix form at the crate root"
    );
    assert!(
        !tags.matches(&ActionId::parse("move.strafe-left").expect("a well-formed action id")),
        "and it is byte-exact prefixing, not a general wildcard match (011's posture)"
    );

    // `load_policy` in call position, over a scratch directory carrying the file
    // the location rule pins, and the slug-before-filesystem rule refusing a
    // traversal name without opening anything (A15).
    let scratch = ScratchDir::new("api-probe-policy");
    scratch.write("v1/probe-policy.toml", POLICY_TEXT);
    let loaded: Policy = bindforge_core::load_policy(&scratch.path().join("v1"), "probe-policy")
        .expect("a policy document in its own major directory must load");
    assert_eq!(
        loaded.id, "probe-policy",
        "the loaded model carries the document's own id"
    );

    let refused: Result<Policy, CoreError> =
        bindforge_core::load_policy(&scratch.path().join("v1"), "../probe-policy");
    assert!(
        matches!(refused, Err(CoreError::InvalidValue { .. })),
        "a policy id outside the slug grammar is refused before the filesystem is asked, and no \
         new `CoreError` variant joins the refusal family (A9, A15)"
    );
}
