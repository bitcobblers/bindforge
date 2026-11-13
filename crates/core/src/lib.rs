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
//! The public surface exercised by the conformance suites: `parse`,
//! `parse_file`, `parse_reader`, `serialize`, `supported_majors`,
//! `resolve_chain`, `MAX_CHAIN_DEPTH`, `MAX_DOCUMENT_BYTES`,
//! `MAX_NESTING_DEPTH`, `load_reach_tables`, `hand_matches_file_name`,
//! `Hand`, `ReachTables`, `load_action_registry`, `ActionId`,
//! `ActionRegistry`, `AxisKind`, `CoreError`.

mod action;
mod chain;
mod document;
mod error;
mod reach;
mod toml;

pub use action::{ActionId, ActionRegistry, AxisKind, load_action_registry};
pub use chain::{MAX_CHAIN_DEPTH, resolve_chain};
pub use document::{Document, Profile, ReachClassTable};
pub use document::{parse, parse_file, parse_reader, serialize, supported_majors};
pub use error::CoreError;
pub use reach::{Hand, ReachClass, ReachTable, ReachTables};
pub use reach::{hand_matches_file_name, load_reach_tables};
pub use toml::{MAX_DOCUMENT_BYTES, MAX_NESTING_DEPTH};
