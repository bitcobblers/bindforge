//! `bindforge-core` — the reference implementation of the `schemas/`
//! formats contract (ADR-008 clause 3), the root of the workspace
//! dependency graph (it depends on nothing), and the crate that owns the
//! model types, reach-table loader, action registry, and chain resolution
//! this design scaffolds.
//!
//! Model semantics beyond the adversarial guard are the 012-015 series';
//! the guard this crate carries — `resolve_chain` with its `Cycle` /
//! `ChainTooDeep` variants, `MAX_CHAIN_DEPTH`, and the parse budget in
//! `toml` — is decided here (Chosen design -> Adversarial guard) and
//! documented for third-party implementers in `schemas/README.md`. The
//! budget constants are part of the public surface beside `MAX_CHAIN_DEPTH`
//! so a caller (a CLI reading a synced tree, a hub checking an upload) can
//! apply the size limit *before* it allocates a document, and
//! `parse_file`/`parse_reader` apply it for callers that take the crate's
//! word for it instead.
//!
//! The action taxonomy (ADR-011) is typed here and shipped as data:
//! `ActionId`, `AxisKind`, and `ActionRegistry`, loaded from
//! `schemas/action-ids/v1/` by `load_action_registry`. Nothing in this crate
//! repeats the id list.
//!
//! The input vocabulary and the reach taxonomy (ADR-012) are typed here and
//! shipped as data the same way: `InputId`/`InputNamespace` hold the
//! OS-neutral `keyboard:`/`mouse:`/`gamepad:` ids whose grammar
//! `schemas/reach-classes/v2/spec.md` states, `ReachTable` carries each
//! hand's grades over the five-class taxonomy loaded from
//! `reach-classes/v2/`, and `KeyboardGeometry` carries the key grid loaded
//! from `keyboard-geometry/v1/`. `placement_is_legal` is Decision Four's
//! predicate — a pure function over that loaded data, which is what lets
//! ADR-015 precondition its transforms on it. Nothing in this crate repeats
//! the class list, the grade tables, or the grid.
//!
//! The game manifests of ADR-013 are typed here as `Manifest`,
//! `ManifestAction`, `ExclusivityGroup`, and `DockedAction`, loaded by
//! `load_manifest` against a `load_action_registry`ed registry; the row
//! encoding, the one-fact-per-entry rules, the fingerprint grammar, and the
//! four budget constants — `MAX_MANIFEST_ROWS`, `MAX_CONSTRAINT_GROUPS`,
//! `MAX_GROUP_MEMBERS`, `MAX_NATIVE_NAME_OCTETS` — ride `manifest` beside the
//! existing budget family.
//!
//! The chains of ADR-014 (design 014) are read and answered here too:
//! `load_chain` walks a `profile/v2` chain out of the major directory its head
//! sits in — the shipped `Cycle`/`ChainTooDeep` refusals applied to the
//! resolved chain, and one new named refusal, `ChainLinkMissing`, for a link the
//! walk itself constructed from a `derives` element and could not find, while an
//! absent head stays `Io` and carries the path — `resolve` answers what layout
//! that chain produces against one manifest, in the manifest's own row order,
//! and `fork`, `diff`, and `merge` are chain operations over documents. The
//! determinism rules that make `resolve` a compatibility surface, the
//! one-directory rule, and the pin equality the joins check are stated in
//! `schemas/profile/v2/spec.md` for third-party readers and pinned by
//! `tests/chain_ops.rs`. The two new budgets, `MAX_PATCH_DELTA_ELEMENTS` and
//! `MAX_GENERATED_BY_OCTETS`, join the exported family beside `MAX_CHAIN_DEPTH`
//! so a caller caps a patch before it allocates it.
//!
//! The policy documents of design 015 (ADR-015) are typed and parsed here —
//! `Policy`, `Transform`, and `TagPattern`, loaded by `load_policy` with the
//! slug-before-filesystem discipline `load_geometry` pins — while the stack that
//! *applies* them is `crates/transforms`, the second workspace crate, which
//! depends on this one and nothing else. The tag grammar, the closed three
//! transform kinds, and the exact-or-prefix matching rule are contract, so they
//! live here; the verdicts a stack reports (`NotApplicableReason`,
//! `TransformReport`) live there. The two new budgets,
//! `MAX_POLICY_TRANSFORMS = 64` and `MAX_RELOCATE_TARGETS = 16`, join the
//! exported family beside the manifest's, mirroring `MAX_CONSTRAINT_GROUPS` and
//! `MAX_GROUP_MEMBERS` rather than adding an aggregate cap.
//!
//! The public surface exercised by the conformance suites: `parse`,
//! `parse_file`, `parse_reader`, `serialize`, `supported_majors`,
//! `resolve_chain`, `MAX_CHAIN_DEPTH`, `MAX_DOCUMENT_BYTES`,
//! `MAX_NESTING_DEPTH`, `load_reach_tables`, `hand_matches_file_name`,
//! `Hand`, `ReachTables`, `load_action_registry`, `ActionId`,
//! `ActionRegistry`, `AxisKind`, `CoreError` — and, from ADR-012,
//! `InputId`, `InputNamespace`, `InputGrade`, `KeyboardGeometry`,
//! `load_geometry`, `geometry_matches_file_name`, `GradeBound`,
//! `PlacementCheck`, and `placement_is_legal` — and, from ADR-013, `Manifest`,
//! `ManifestAction`, `ExclusivityGroup`, `DockedAction`, `load_manifest`, and
//! the four manifest budget constants named above — and, from ADR-014, `Chain`,
//! `load_chain`, `resolve`, `ResolvedBinding`, `fork`, `diff`, `merge`,
//! `ProfileKind`, and the two chain budgets named above — and, from design 015,
//! `Policy`, `Transform`, `TagPattern`, `load_policy`, and the two policy
//! budgets named above.

mod action;
mod chain;
mod chain_ops;
mod document;
mod error;
mod input;
mod manifest;
mod placement;
mod policy;
mod reach;
mod toml;

pub use action::{ActionId, ActionRegistry, AxisKind, load_action_registry};
pub use chain::{MAX_CHAIN_DEPTH, resolve_chain};
pub use chain_ops::{Chain, ResolvedBinding, diff, fork, load_chain, merge, resolve};
pub use document::{Document, GeometryKey, KeyboardGeometryTable};
pub use document::{MAX_GENERATED_BY_OCTETS, MAX_PATCH_DELTA_ELEMENTS, supported_majors};
pub use document::{MAX_POLICY_TRANSFORMS, MAX_RELOCATE_TARGETS};
pub use document::{Policy, TagPattern, Transform};
pub use document::{Profile, ProfileKind, ReachClassTable};
pub use document::{parse, parse_file, parse_reader, serialize};
pub use error::CoreError;
pub use input::{InputId, InputNamespace};
pub use manifest::{
    DockedAction, ExclusivityGroup, MAX_CONSTRAINT_GROUPS, MAX_GROUP_MEMBERS, MAX_MANIFEST_ROWS,
    MAX_NATIVE_NAME_OCTETS, Manifest, ManifestAction, load_manifest,
};
pub use placement::{GradeBound, KeyboardGeometry, PlacementCheck};
pub use placement::{geometry_matches_file_name, load_geometry, placement_is_legal};
pub use policy::load_policy;
pub use reach::{Hand, InputGrade, ReachClass, ReachTable, ReachTables};
pub use reach::{hand_matches_file_name, load_reach_tables};
pub use toml::{MAX_DOCUMENT_BYTES, MAX_NESTING_DEPTH};
