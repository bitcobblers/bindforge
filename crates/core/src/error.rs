//! Structured rejections for untrusted documents.
//!
//! Profile, patch, policy, and manifest documents arriving from disk, from
//! another user, or from the network are untrusted input at every entry
//! point (Chosen design -> Adversarial guard). Every refusal `bindforge-core`
//! makes is one of these typed variants — never a panic, never a bare string
//! — so the CLI, the desktop app, and the hub report the same reason for the
//! same document, and the invalid-fixture corpus under `schemas/` is the
//! contract for which variant a given document must produce.
//!
//! A rejection message is also a log line, and every string it can carry
//! comes from the document being rejected: a declared `format`, a declared
//! `version`, an entry key, a hand value. Those go through `one_line` before
//! they are written, so one hostile upload cannot forge extra log lines or
//! inflate log volume. A local filesystem path stays out of the message
//! entirely — `source_path` is how a server-side log asks for it.
//!
//! A wording one of these arms ships is part of the contract the fixtures and
//! the loader tests pin: a variant's message is not re-worded to satisfy a new
//! assertion, and a variant with no payload has no offender to name — the test
//! asserts its variant instead.

use std::fmt;

use crate::chain::MAX_CHAIN_DEPTH;

/// How many characters of an untrusted string one rejection message may
/// carry; `schemas/README.md` promises bounded rejection messages.
const MESSAGE_VALUE_CHARS: usize = 80;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreError {
    /// The document is larger than the parse budget in `schemas/README.md`.
    DocumentTooLarge { limit: usize },
    /// Tables or arrays nest deeper than the parse budget allows.
    NestingTooDeep { limit: usize },
    /// The text is not valid TOML under the grammar the format's `spec.md` allows.
    Malformed { line: usize, message: String },
    /// The bytes handed to `parse_reader` are not UTF-8 text.
    NotUtf8,
    /// A required entry of the format's grammar is absent.
    MissingEntry { key: String },
    /// The document declares no version at all (`declared: None`), or a major
    /// this crate does not support. Either way the message names the
    /// supported majors (Test strategy #2); the document is never defaulted.
    UnsupportedVersion {
        declared: Option<String>,
        supported: Vec<String>,
    },
    /// The document declares a `format` this crate does not parse.
    UnknownFormat {
        declared: String,
        supported: Vec<String>,
    },
    /// An entry is present but its value violates the format's grammar —
    /// including an entry the grammar does not define at all, which the
    /// closed entry sets in `document` refuse rather than ignore.
    InvalidValue { key: String, message: String },
    /// A chain derives from itself (Chosen design -> Adversarial guard).
    Cycle,
    /// A chain is longer than `MAX_CHAIN_DEPTH` links (Chosen design ->
    /// Adversarial guard).
    ChainTooDeep,
    /// A chain link names an ancestor document that is not there. A chain is a
    /// contract object, so a missing link is a completeness failure about a
    /// named link, not an I/O detail a reader could work around (design 014,
    /// A18): the `id` says which link, and the path it was looked for at stays
    /// out of the message like every other path. Only a link a walk constructed
    /// from a `derives` element gets this refusal — the `id` it carries is
    /// parse-validated, and the directory rule fixes where the file belongs. A
    /// head the caller named is an `Io` refusal, whose variant carries the path.
    ChainLinkMissing { id: String },
    /// A contract file could not be read. `path` says where it was read from
    /// (a file path, or a label when the source has none) and stays out of
    /// the message: `source_path` is the way to log it.
    Io { path: String, message: String },
}

impl CoreError {
    /// Where a read refusal happened, for a server-side log. It never appears
    /// in the `Display` text, which is what a CLI or a hub shows or logs about
    /// a document uploaded by someone else — a local path is not the
    /// uploader's business.
    #[must_use]
    pub fn source_path(&self) -> Option<&str> {
        match self {
            CoreError::Io { path, .. } => Some(path),
            _ => None,
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::DocumentTooLarge { limit } => {
                write!(f, "document is larger than the {limit} byte parse budget")
            }
            CoreError::NestingTooDeep { limit } => {
                write!(
                    f,
                    "document nests tables or arrays deeper than {limit} levels"
                )
            }
            CoreError::Malformed { line, message } => {
                write!(
                    f,
                    "malformed document on line {line}: {}",
                    one_line(message)
                )
            }
            CoreError::NotUtf8 => write!(f, "document is not valid UTF-8"),
            CoreError::MissingEntry { key } => {
                write!(
                    f,
                    "document is missing the required `{}` entry",
                    one_line(key)
                )
            }
            CoreError::UnsupportedVersion {
                declared: None,
                supported,
            } => write!(
                f,
                "document declares no format version; supported versions: {}",
                supported.join(", ")
            ),
            CoreError::UnsupportedVersion {
                declared: Some(declared),
                supported,
            } => write!(
                f,
                "document declares unsupported version `{}`; supported versions: {}",
                one_line(declared),
                supported.join(", ")
            ),
            CoreError::UnknownFormat {
                declared,
                supported,
            } => write!(
                f,
                "document declares unknown format `{}`; supported formats: {}",
                one_line(declared),
                supported.join(", ")
            ),
            CoreError::InvalidValue { key, message } => {
                write!(f, "`{}` is invalid: {}", one_line(key), one_line(message))
            }
            CoreError::Cycle => write!(f, "chain derives from itself"),
            CoreError::ChainTooDeep => {
                write!(
                    f,
                    "chain is longer than the maximum of {MAX_CHAIN_DEPTH} links"
                )
            }
            CoreError::ChainLinkMissing { id } => {
                write!(f, "chain link `{}` is missing", one_line(id))
            }
            CoreError::Io { path: _, message } => {
                write!(f, "could not read a contract file: {}", one_line(message))
            }
        }
    }
}

/// Renders one untrusted string as a bounded, single-line fragment:
/// backslashes and control characters are escaped, and more than
/// `MESSAGE_VALUE_CHARS` characters are cut with a `…`. The grammar decodes
/// `\n` inside a quoted string, so an unescaped value could otherwise put a
/// second, timestamp-shaped line into the hub's log and echo a whole
/// parse-budget-sized value back at whoever reads the rejection.
fn one_line(raw: &str) -> String {
    let mut out = String::new();
    for (index, c) in raw.chars().enumerate() {
        if index >= MESSAGE_VALUE_CHARS {
            out.push('…');
            break;
        }
        match c {
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            other => out.push(other),
        }
    }
    out
}

impl std::error::Error for CoreError {}
