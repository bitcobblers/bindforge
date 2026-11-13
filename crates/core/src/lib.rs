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
//! the four manifest budget constants named above.

mod action;
mod chain;
mod document;
mod error;
mod input;
mod manifest;
mod placement;
mod reach;
mod toml;

pub use action::{ActionId, ActionRegistry, AxisKind, load_action_registry};
pub use chain::{MAX_CHAIN_DEPTH, resolve_chain};
pub use document::{Document, GeometryKey, KeyboardGeometryTable, Profile, ReachClassTable};
pub use document::{parse, parse_file, parse_reader, serialize, supported_majors};
pub use error::CoreError;
pub use input::{InputId, InputNamespace};
pub use manifest::{
    DockedAction, ExclusivityGroup, MAX_CONSTRAINT_GROUPS, MAX_GROUP_MEMBERS, MAX_MANIFEST_ROWS,
    MAX_NATIVE_NAME_OCTETS, Manifest, ManifestAction, load_manifest,
};
pub use placement::{GradeBound, KeyboardGeometry, PlacementCheck};
pub use placement::{geometry_matches_file_name, load_geometry, placement_is_legal};
pub use reach::{Hand, InputGrade, ReachClass, ReachTable, ReachTables};
pub use reach::{hand_matches_file_name, load_reach_tables};
pub use toml::{MAX_DOCUMENT_BYTES, MAX_NESTING_DEPTH};
