//! The document model and the version gate of the `schemas/` formats
//! contract (ADR-008 clause 3).
//!
//! A contract document declares its `format` and its `version`. A document
//! that declares no version, or a major this crate does not support, is
//! rejected with a structured error naming the supported majors — never
//! defaulted to the format its fixture happens to sit under (versioning rule,
//! canonical in the proposal and mirrored in `schemas/README.md`). The
//! majors this crate declares equal the majors present under
//! `schemas/*/v*/` (Test strategy #2), pinned by `tests/version_gate.rs`.
//!
//! Each format's entry set is closed, exactly as its `spec.md` states ("Any
//! other entry is rejected"): an entry the grammar does not define is
//! refused, not ignored. Accepting one would admit data the contract has no
//! meaning for into the same trust boundary the chain guards protect — a
//! misspelled or aliased `derives_` reads as *no ancestors at all*, quietly
//! turning a derived profile into a standalone one.
//!
//! The scaffold ships the `profile/v1` and `reach-classes/v1` documents, and
//! ADR-011 adds `action-ids/v1`; the manifest and profile-chain formats arrive
//! with the 012-015 series and the same harness walk picks them up with no
//! harness change.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use crate::action::{ActionRegistry, AxisKind, registry_from_entries};
use crate::error::CoreError;
use crate::reach::{Hand, ReachClass};
use crate::toml::{self, Value};

/// The one major version of the contract this crate implements. Must stay
/// equal to the majors present under `schemas/*/v*/` (Test strategy #2).
const MAJOR_V1: &str = "v1";

/// The format names this crate parses.
const SUPPORTED_FORMATS: &[&str] = &["action-ids", "profile", "reach-classes"];

/// Every entry the `profile/v1` grammar defines, and the only ones it admits.
const PROFILE_ENTRIES: &[&str] = &["derives", "format", "id", "version"];

/// Every entry the `reach-classes/v1` grammar defines, and the only ones it
/// admits.
const REACH_CLASSES_ENTRIES: &[&str] = &["classes", "format", "hand", "version"];

/// Every entry the `action-ids/v1` grammar defines, and the only ones it
/// admits. The three axis arrays are the entry set's load-bearing part: they
/// are the id set, so an entry naming a fourth axis kind, or an `emphasis`,
/// is refused rather than ignored (ADR-011: emphasis is *not* in the
/// taxonomy).
const ACTION_IDS_ENTRIES: &[&str] = &[
    "continuous-lateral",
    "discrete-lateral",
    "format",
    "none",
    "version",
];

/// The majors `bindforge-core` declares supported.
pub fn supported_majors() -> Vec<String> {
    vec![MAJOR_V1.to_string()]
}

/// A `profile/v1` document. This design carries the identity and the
/// inline ancestor chain the guards read (`id`, `derives`); ADR-014 owns
/// profile content beyond that.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub version: String,
    pub id: String,
    pub derives: Vec<String>,
}

/// A `reach-classes/v1` table document: one hand's class list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachClassTable {
    pub version: String,
    pub hand: Hand,
    pub classes: Vec<ReachClass>,
}

/// One contract document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Profile(Profile),
    ReachClasses(ReachClassTable),
    ActionIds(ActionRegistry),
}

impl Document {
    /// The profile half of the document, when it is a profile — the handle
    /// `resolve_chain` guards (Chosen design -> Adversarial guard).
    pub fn profile(&self) -> Option<&Profile> {
        match self {
            Document::Profile(profile) => Some(profile),
            Document::ReachClasses(_) | Document::ActionIds(_) => None,
        }
    }
}

/// Parse one contract document from text, applying the parse budget (`toml`
/// module), the version gate, and the closed per-format grammar from its
/// `spec.md`.
///
/// Every refusal is a structured [`CoreError`]; this function never panics
/// on input, because profile, patch, policy, and manifest documents arriving
/// from disk, from another user, or from the network are untrusted at every
/// entry point.
pub fn parse(text: &str) -> Result<Document, CoreError> {
    let entries = toml::read_document(text)?;

    let format = string_entry(&entries, "format")?;

    let version = match entries.get("version") {
        None => {
            return Err(CoreError::UnsupportedVersion {
                declared: None,
                supported: supported_majors(),
            });
        }
        Some(value) => string_value("version", value)?,
    };
    if !supported_majors().contains(&version) {
        return Err(CoreError::UnsupportedVersion {
            declared: Some(version.clone()),
            supported: supported_majors(),
        });
    }

    match format.as_str() {
        "profile" => {
            reject_unknown_entry(&entries, "profile", PROFILE_ENTRIES)?;
            Ok(Document::Profile(parse_profile(&entries, version)?))
        }
        "reach-classes" => {
            reject_unknown_entry(&entries, "reach-classes", REACH_CLASSES_ENTRIES)?;
            let table = parse_reach_classes(&entries, version)?;
            Ok(Document::ReachClasses(table))
        }
        "action-ids" => {
            reject_unknown_entry(&entries, "action-ids", ACTION_IDS_ENTRIES)?;
            let registry = registry_from_entries(&entries, version)?;
            Ok(Document::ActionIds(registry))
        }
        other => Err(CoreError::UnknownFormat {
            declared: other.to_string(),
            supported: SUPPORTED_FORMATS
                .iter()
                .map(|name| name.to_string())
                .collect(),
        }),
    }
}

/// Parse one contract file, applying the size budget to the file's length
/// *before* reading it, so a caller handling an untrusted tree never
/// allocates past the budget (`schemas/README.md`, Input budget).
pub fn parse_file(path: &Path) -> Result<Document, CoreError> {
    let text = toml::read_file(path)?;
    parse(&text)
}

/// Parse one stream — an upload body, a pipe — under the same budget: at most
/// one byte more than `MAX_DOCUMENT_BYTES` is taken from it, and a stream with
/// more than that is refused before anything is parsed.
pub fn parse_reader(source: impl Read) -> Result<Document, CoreError> {
    let text = toml::read_reader(source)?;
    parse(&text)
}

/// Refuses an entry outside the format's entry set. The check runs after the
/// required entries are read, so a document missing `version` still gets the
/// version gate's error, and a document carrying an entry the grammar does not
/// define never reaches the model with it.
fn reject_unknown_entry(
    entries: &BTreeMap<String, Value>,
    format: &str,
    defined: &[&str],
) -> Result<(), CoreError> {
    for key in entries.keys() {
        if !defined.contains(&key.as_str()) {
            return Err(CoreError::InvalidValue {
                key: key.clone(),
                message: format!("not part of the {format}/v1 grammar"),
            });
        }
    }
    Ok(())
}

/// Write a document back out as contract text. Re-parsing the output yields
/// a semantically equal document — the round-trip half of Test strategy #1.
pub fn serialize(document: &Document) -> String {
    match document {
        Document::Profile(profile) => {
            let derives = profile
                .derives
                .iter()
                .map(|ancestor| quoted(ancestor))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "format = \"profile\"\nversion = {}\nid = {}\nderives = [{derives}]\n",
                quoted(&profile.version),
                quoted(&profile.id)
            )
        }
        Document::ReachClasses(table) => {
            let classes = table
                .classes
                .iter()
                .map(|class| quoted(&class.id))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "format = \"reach-classes\"\nversion = {}\nhand = {}\nclasses = [{classes}]\n",
                quoted(&table.version),
                quoted(table.hand.name())
            )
        }
        Document::ActionIds(registry) => {
            let discrete = axis_array(registry, AxisKind::DiscreteLateral);
            let continuous = axis_array(registry, AxisKind::ContinuousLateral);
            let none = axis_array(registry, AxisKind::None);
            let lines = [
                "format = \"action-ids\"".to_string(),
                format!("version = {}", quoted(&registry.version)),
                format!("discrete-lateral = [{discrete}]"),
                format!("continuous-lateral = [{continuous}]"),
                format!("none = [{none}]"),
            ];
            let mut text = String::new();
            for line in lines {
                text.push_str(&line);
                text.push('\n');
            }
            text
        }
    }
}

/// The ids one axis array lists, quoted into the grammar's one-line array
/// form, in the registry's id order. `serialize` writes the three arrays in
/// the order the entry table defines them, so re-parsing its output yields the
/// same registry — the round-trip the harness asserts for this format.
fn axis_array(registry: &ActionRegistry, kind: AxisKind) -> String {
    registry
        .ids()
        .iter()
        .filter(|id| registry.axis_kind(id) == Some(kind))
        .map(|id| quoted(id.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_profile(entries: &BTreeMap<String, Value>, version: String) -> Result<Profile, CoreError> {
    let id = string_entry(entries, "id")?;
    if id.is_empty() {
        return Err(CoreError::InvalidValue {
            key: "id".to_string(),
            message: "must not be empty".to_string(),
        });
    }
    let derives = match entries.get("derives") {
        None => Vec::new(),
        Some(value) => string_array("derives", value)?,
    };
    Ok(Profile {
        version,
        id,
        derives,
    })
}

fn parse_reach_classes(
    entries: &BTreeMap<String, Value>,
    version: String,
) -> Result<ReachClassTable, CoreError> {
    let hand = match entries.get("hand") {
        None => {
            return Err(CoreError::MissingEntry {
                key: "hand".to_string(),
            });
        }
        Some(value) => {
            let name = string_value("hand", value)?;
            match Hand::from_name(&name) {
                Some(hand) => hand,
                None => {
                    return Err(CoreError::InvalidValue {
                        key: "hand".to_string(),
                        message: format!("`{name}` is not a hand: expected `left` or `right`"),
                    });
                }
            }
        }
    };
    let classes = match entries.get("classes") {
        None => {
            return Err(CoreError::MissingEntry {
                key: "classes".to_string(),
            });
        }
        Some(value) => string_array("classes", value)?
            .into_iter()
            .map(|id| ReachClass { id })
            .collect(),
    };
    Ok(ReachClassTable {
        version,
        hand,
        classes,
    })
}

fn string_entry(entries: &BTreeMap<String, Value>, key: &str) -> Result<String, CoreError> {
    match entries.get(key) {
        None => Err(CoreError::MissingEntry {
            key: key.to_string(),
        }),
        Some(value) => string_value(key, value),
    }
}

fn string_value(key: &str, value: &Value) -> Result<String, CoreError> {
    match value {
        Value::String(text) => Ok(text.clone()),
        _ => Err(CoreError::InvalidValue {
            key: key.to_string(),
            message: "expected a string".to_string(),
        }),
    }
}

pub(crate) fn string_array(key: &str, value: &Value) -> Result<Vec<String>, CoreError> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::String(text) => Ok(text.clone()),
                _ => Err(CoreError::InvalidValue {
                    key: key.to_string(),
                    message: "every element must be a string".to_string(),
                }),
            })
            .collect(),
        _ => Err(CoreError::InvalidValue {
            key: key.to_string(),
            message: "expected an array of strings".to_string(),
        }),
    }
}

/// Quote a string into the grammar's basic-string syntax, escaping exactly
/// what the reader in `toml` decodes back identically — `\b` and `\f`
/// included, and any other control character as a `\uXXXX` escape, so every
/// model value has a document the same reader accepts.
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c.is_control() || c == '\u{7f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}
