//! `bindforge-codecs` — the codec-family side of ADR-017 (design 017).
//!
//! This crate owns three things and nothing else: the [`codec::CodecFamily`]
//! trait with its claim payload [`codec::Detection`], the closed static
//! [`registry::REGISTRY`] whose two claim passes run the appid table before
//! any tree probing (`017-C5`); the launcher-context resolvers
//! ([`paths::user_trees`]) that turn a [`context::GameContext`] — built only
//! through [`GameContext::from_record`], so a half-parsed identity can never
//! reach a codec — into the located user trees a family may read; and the
//! `bindings/v1` snapshot contract ([`snapshot`]), whose loader refuses by
//! name exactly as `schemas/bindings/v1/spec.md` states.
//!
//! The first family is [`families::unreal_ini`]: it reports the bindings a
//! game's own user config files state, tokens verbatim, and performs no
//! taxonomy translation (`017-L3`). The whole crate is read-only by
//! construction (`017-L1`): every open is `OpenOptions::new().read(true)`
//! and nothing else, no code path here takes a `&mut` to a path, creates a
//! directory, or writes a byte. Locate walks take `DirEntry::file_type()`,
//! which does not follow symlinks, so a link is never descended, never read
//! as a file, and never taken as a tree anchor.
//!
//! The budget constants of §Read discipline are exported at the crate root,
//! beside nothing from `bindforge-core` — the README's "exports all thirteen
//! constants" sentence stays core's.

pub mod codec;
pub mod context;
pub mod families;
pub mod paths;
pub mod read;
pub mod registry;
pub mod snapshot;

pub use codec::{CodecFamily, Detection, Evidence, ExtractionRefusal, UserCandidates};
pub use context::GameContext;
pub use paths::{TreeKind, UserTree, user_trees};
pub use read::{MAX_BINDING_FILE_OCTETS, decode, decode_owned, open_capped};
pub use registry::{REGISTRY, claim, claim_with, family_ids, is_known_family};
pub use snapshot::{
    BindingSnapshot, MAX_SNAPSHOT_ENTRIES, MAX_SNAPSHOT_OCTETS, SnapshotRefusal, load_snapshot,
};

/// Trees the `SteamUserdata` template keeps per game, across all of
/// `GameContext::roots`, in `roots()`-then-`<dir>` enumeration order. This is
/// the one silent bound: past the 16th tree a game's remaining local users
/// are simply absent (`schemas/bindings/v1/spec.md` states it as such).
pub const MAX_USER_DIRS_PER_GAME: usize = 16;

/// Components in a located file's tree-relative path — its last four are
/// `Saved/Config/<dir>/<name>.ini` (§Read discipline's table).
pub const MAX_LOCATE_DEPTH: usize = 8;

/// Entries the locate walk examines below one `UserTree` root; a walk that
/// reaches the budget stops and counts the rest (§Read discipline). The
/// constant bounds *examined entries*, not the walk's transient per-directory
/// listing: one `read_dir` of one directory is materialized and byte-sorted
/// before the budget trims what the loop examines (determinism of the
/// examined set needs the sorted listing), and an over-budget walk enters no
/// further directory at all.
pub const MAX_LOCATE_ENTRIES_PER_TREE: usize = 16_384;

/// One user tree's located candidate set (`017-B3`, read as a *tree's* opens
/// — the type invariant behind [`UserCandidates::push`]).
pub const MAX_CANDIDATE_FILES: usize = 32;
