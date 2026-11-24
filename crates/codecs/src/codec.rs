//! The codec face (`017-C1`, §The codec face).
//!
//! A claim is never a boolean: [`Detection`] carries the claiming family's
//! id, the candidate files grouped per located user tree, and the claiming
//! pass's [`Evidence`]. No claim, a claim, and a claim whose parse refused
//! stay three distinct outcomes end to end.

use std::fmt;
use std::path::PathBuf;

use crate::MAX_CANDIDATE_FILES;
use crate::context::GameContext;
use crate::snapshot::BindingSnapshot;

/// One codec family: a **storage grammar**, not a lineage (§The registry).
///
/// `Sync` so the closed static [`REGISTRY`](crate::registry::REGISTRY) can
/// hold `&dyn CodecFamily` values: the registry is shared static data.
pub trait CodecFamily: Sync {
    /// The family id: `[a-z][a-z0-9-]*` of at most
    /// `bindforge_core::MAX_SLUG_OCTETS` octets — the value a snapshot's
    /// `codec` records and the key the registry iterates by (§Codec family
    /// id).
    fn id(&self) -> &'static str;
    /// Appids this family claims by table (pass 1). May be empty — and for
    /// `unreal-ini` it is, this run (A21: no appid knowledge is surveyed into
    /// the tree).
    fn appids(&self) -> &'static [&'static str];
    /// Locates the family's grammar for `ctx`; `None` is "no claim".
    fn detect(&self, ctx: &GameContext) -> Option<Detection>;
    /// Reads only the located files and produces one snapshot document per
    /// located user tree, in the order §Determinism's law fixes.
    fn extract(&self, claim: &Detection) -> Result<Vec<BindingSnapshot>, ExtractionRefusal>;
}

/// Which claim pass produced a [`Detection`] (`017-C5`): the appid table or
/// the tree probe. The token is what a snapshot's `provenance.evidence`
/// spells, so an artifact says which family claimed a game and on what basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evidence {
    /// Pass 1 — the family's `appids()` contained the context's appid.
    AppidTable,
    /// Pass 2 — the family located at least one file in its own grammar.
    InstallTreeProbe,
}

impl Evidence {
    /// The contract token (`schemas/bindings/v1/spec.md`).
    #[must_use]
    pub fn token(&self) -> &'static str {
        match self {
            Evidence::AppidTable => "appid-table",
            Evidence::InstallTreeProbe => "install-tree-probe",
        }
    }
}

/// One located user tree: the context label, the candidate files the locate
/// walk admitted, and the tree's own root — §Determinism's dedup and ordering
/// key, never rendered into a document.
#[derive(Debug, Clone)]
pub struct UserCandidates {
    /// The context label: the local SteamID directory name, or the constant
    /// `steamuser` for a Proton prefix (§Config-path resolvers).
    pub user: String,
    /// The located candidate files, absolute, in the walk's byte order.
    pub files: Vec<PathBuf>,
    /// That tree's `UserTree::root`: tree identity, never rendered.
    pub tree_root: PathBuf,
}

impl UserCandidates {
    /// The `017-B3` type invariant behind the walk's skip-and-count: a
    /// `MAX_CANDIDATE_FILES`-breaching push is refused, and the set is left
    /// as it was.
    pub fn push(&mut self, file: PathBuf) -> Result<(), ExtractionRefusal> {
        if self.files.len() >= MAX_CANDIDATE_FILES {
            return Err(ExtractionRefusal::CandidateFilesBudget {
                limit: MAX_CANDIDATE_FILES,
            });
        }
        self.files.push(file);
        Ok(())
    }
}

/// A family's claim on one game: the claiming id, the claiming pass's
/// evidence, and the candidate files grouped per located user tree.
/// `per_user` is in `user_trees` order and `extract`'s `Vec` is positionally
/// parallel to it — one document per located tree (`017-C3`) as a type-level
/// fact (§Determinism fixes the order `extract` renders in).
#[derive(Debug, Clone)]
pub struct Detection {
    /// The claiming family's id.
    pub family: &'static str,
    /// Which pass claimed.
    pub evidence: Evidence,
    /// One entry per located tree.
    pub per_user: Vec<UserCandidates>,
}

/// The named extraction refusals (§Read discipline): each is printed by name
/// and none of them is an empty snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractionRefusal {
    /// A file a codec opened was over `MAX_BINDING_FILE_OCTETS` (`017-B1`) —
    /// refused on the metadata length check, before allocation.
    FileOverBudget {
        /// The offending file (tree-relative when the fold knows a tree).
        path: String,
        /// The measured length.
        octets: u64,
        /// The bound.
        limit: usize,
    },
    /// Bytes that no BOM sniffing plus strict UTF-8 / UTF-16 decoding yields
    /// as text (A19).
    UndecodableText {
        /// The offending file, empty when the refusal is at the pure
        /// `decode(bytes)` seam.
        path: String,
    },
    /// A `Bindings=(…)` tuple that lacks a quoted `Name=` or `Command=`,
    /// carries an unexpected key, is unbalanced, or states a verbatim token
    /// outside the grammar — drift becomes loud rather than a silently
    /// shorter snapshot. The message names the file's tree-relative path and
    /// its 1-based line, and the bound's name when the verdict is a token
    /// over `MAX_NATIVE_NAME_OCTETS`.
    UnparsableEntry {
        /// The file's tree-relative path (or the reader's path at the seam).
        path: String,
        /// The offending line, 1-based.
        line: usize,
        /// What about the line refused it.
        detail: String,
    },
    /// The distinct (`action`, input-token) pairs a tree's fold has ever
    /// admitted passed `MAX_SNAPSHOT_ENTRIES` (`017-B2`, read as *as
    /// admitted*: a removal frees no room).
    SnapshotEntryBudget {
        /// The bound.
        limit: usize,
    },
    /// The `UserCandidates::push` invariant: one tree's located set past
    /// `MAX_CANDIDATE_FILES` (`017-B3`).
    CandidateFilesBudget {
        /// The bound.
        limit: usize,
    },
    /// A located file the OS refused to open at all. Named, so the face
    /// prints a status rather than inventing an empty answer.
    UnreadableFile {
        /// The offending file.
        path: String,
    },
}

/// Every `path` field a refusal echoes is assembled from directory and file
/// names a located, untrusted tree chose; the `Display` renders each of them
/// through the shared `escape_and_truncate` renderer, so a control byte in a
/// file name can never put a cursor move, a title change, or a second line
/// on the terminal that prints the `error: ` line
/// (`schemas/bindings/v1/spec.md`: messages are "single-line, control-free
/// and truncating; echoed values pass through the shared
/// `escape_and_truncate` renderer"). Clean paths render unchanged.
impl fmt::Display for ExtractionRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let echo = |value: &str| bindforge_core::escape_and_truncate(value);
        match self {
            ExtractionRefusal::FileOverBudget {
                path,
                octets,
                limit,
            } => write!(
                f,
                "FileOverBudget: {} is {octets} octets, past MAX_BINDING_FILE_OCTETS ({limit})",
                echo(path)
            ),
            ExtractionRefusal::UndecodableText { path } if path.is_empty() => {
                write!(f, "UndecodableText: the bytes do not decode as text")
            }
            ExtractionRefusal::UndecodableText { path } => {
                write!(f, "UndecodableText: {} does not decode as text", echo(path))
            }
            ExtractionRefusal::UnparsableEntry { path, line, detail } => {
                write!(f, "UnparsableEntry: {}:{line}: {detail}", echo(path))
            }
            ExtractionRefusal::SnapshotEntryBudget { limit } => write!(
                f,
                "SnapshotEntryBudget: more than MAX_SNAPSHOT_ENTRIES ({limit}) distinct pairs admitted"
            ),
            ExtractionRefusal::CandidateFilesBudget { limit } => write!(
                f,
                "CandidateFilesBudget: more than MAX_CANDIDATE_FILES ({limit}) files in one tree"
            ),
            ExtractionRefusal::UnreadableFile { path } => {
                write!(
                    f,
                    "UnreadableFile: {} could not be opened for reading",
                    echo(path)
                )
            }
        }
    }
}

impl std::error::Error for ExtractionRefusal {}
