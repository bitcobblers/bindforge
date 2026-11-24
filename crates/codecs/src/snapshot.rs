//! The `bindings/v1` snapshot model, its budgets, and its loader
//! (`017-C3`, §`bindings/v1`).
//!
//! One JSON document per game per located user tree. The key set is closed
//! at every level; an unknown key, a missing required key, or a value
//! outside its bound is a refusal naming the key. The loader checks
//! `fs::metadata` length against [`MAX_SNAPSHOT_OCTETS`] **before** the read
//! — the posture §Read discipline and `catalog` keep — so an oversized
//! document is never buffered.
//!
//! The struct's declared field order is the contract's serialization order
//! (`serde_json::to_string`, compact): the loader's re-serialization of a
//! fixture must be byte-stable, which is why the fields are declared in
//! their byte-sorted spelling.

use std::path::Path;

use serde::{Deserialize, Serialize};

use bindforge_core::{
    MAX_DISCOVERY_SLUG_OCTETS, MAX_NATIVE_NAME_OCTETS, MAX_SLUG_OCTETS, escape_and_truncate,
    is_slug_name,
};

use crate::MAX_CANDIDATE_FILES;
use crate::MAX_LOCATE_DEPTH;

/// One snapshot document's distinct (`action`, input-token) pairs as
/// admitted (`017-B2`).
pub const MAX_SNAPSHOT_ENTRIES: usize = 4096;

/// One snapshot document the loader reads (design-time budget).
pub const MAX_SNAPSHOT_OCTETS: usize = 4_194_304;

/// The contract's `format` token.
pub const SNAPSHOT_FORMAT: &str = "bindings";

/// The contract's only `version`.
pub const SNAPSHOT_VERSION: u64 = 1;

/// One snapshot entry: the native action (or command) name verbatim and the
/// located file's own input tokens verbatim (§Input spelling (canonical):
/// `bindings/v1` carries no ADR-012 input ids anywhere).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotEntry {
    /// The `Command` string verbatim.
    pub action: String,
    /// The `Name` tokens verbatim, deduplicated and byte-sorted.
    pub inputs: Vec<String>,
}

/// Where the snapshot's facts came from (`017-C5`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// The claiming pass's evidence token: `appid-table` or
    /// `install-tree-probe`.
    pub evidence: String,
    /// The located files, tree-relative, sorted byte order.
    pub files: Vec<String>,
}

/// One `bindings/v1` document. Field order is serialization order (§module
/// docs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingSnapshot {
    /// A registry family id.
    pub codec: String,
    /// The bindings the located user files state, byte-sorted by action.
    pub entries: Vec<SnapshotEntry>,
    /// The format token, [`SNAPSHOT_FORMAT`].
    pub format: String,
    /// The game's lookup key.
    pub game: String,
    /// Where the entries were read from.
    pub provenance: Provenance,
    /// The context label; absent when the family's storage is not
    /// user-scoped. Launcher-local identity — a sharing surface treats it as
    /// personal data (`schemas/bindings/v1/spec.md`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// The bare major integer, [`SNAPSHOT_VERSION`].
    pub version: u64,
}

/// A loader refusal naming its key (or its bound). Messages are single-line,
/// control-free and truncating: every echoed value passes through
/// `bindforge_core::escape_and_truncate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRefusal {
    /// The message, already escaped and bounded.
    pub message: String,
}

impl SnapshotRefusal {
    fn named(key: &str, detail: impl AsRef<str>) -> Self {
        Self::one_line(format!("{key}: {}", detail.as_ref()))
    }

    fn one_line(text: String) -> Self {
        // Single line, control-free, truncating.
        Self {
            message: escape_and_truncate(&text),
        }
    }
}

impl std::fmt::Display for SnapshotRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SnapshotRefusal {}

/// Reads and validates one `bindings/v1` document from `path`. The length
/// check runs against [`MAX_SNAPSHOT_OCTETS`] before any buffering.
pub fn load_snapshot(path: &Path) -> Result<BindingSnapshot, SnapshotRefusal> {
    let meta = std::fs::metadata(path).map_err(|_| {
        SnapshotRefusal::named(
            "document",
            format!("{} could not be opened for reading", path.display()),
        )
    })?;
    if meta.len() > MAX_SNAPSHOT_OCTETS as u64 {
        return Err(SnapshotRefusal::one_line(format!(
            "document: {} octets, past MAX_SNAPSHOT_OCTETS ({MAX_SNAPSHOT_OCTETS})",
            meta.len()
        )));
    }
    let text = std::fs::read_to_string(path).map_err(|_| {
        SnapshotRefusal::named(
            "document",
            format!("{} is not readable text", path.display()),
        )
    })?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
        SnapshotRefusal::named("document", format!("not a JSON document: {error}"))
    })?;
    validate(&value)
}

/// Validates one parsed document against the closed key sets and every
/// bound, refusing by name.
fn validate(value: &serde_json::Value) -> Result<BindingSnapshot, SnapshotRefusal> {
    let object = value
        .as_object()
        .ok_or_else(|| SnapshotRefusal::named("document", "the top level must be an object"))?;
    const KEYS: [&str; 7] = [
        "codec",
        "entries",
        "format",
        "game",
        "provenance",
        "user",
        "version",
    ];
    for key in object.keys() {
        if !KEYS.contains(&key.as_str()) {
            return Err(SnapshotRefusal::named(
                key.as_str(),
                "unknown key in the closed top-level set",
            ));
        }
    }

    let format = string_field(object.get("format"), "format")?;
    if format != SNAPSHOT_FORMAT {
        return Err(SnapshotRefusal::named(
            "format",
            format!("must be `{SNAPSHOT_FORMAT}`, got `{}`", display(format)),
        ));
    }
    let version_value = object
        .get("version")
        .ok_or_else(|| SnapshotRefusal::named("version", "required key missing"))?;
    let version = version_value.as_u64().ok_or_else(|| {
        SnapshotRefusal::named(
            "version",
            "must be the bare integer 1, not a string or a fraction",
        )
    })?;
    if version != SNAPSHOT_VERSION {
        return Err(SnapshotRefusal::named(
            "version",
            format!("must be {SNAPSHOT_VERSION}, got {version}"),
        ));
    }
    let codec = string_field(object.get("codec"), "codec")?;
    if !is_slug_name(codec) || codec.len() > MAX_SLUG_OCTETS {
        return Err(SnapshotRefusal::named(
            "codec",
            format!(
                "`{}` is outside the slug grammar at MAX_SLUG_OCTETS ({MAX_SLUG_OCTETS})",
                display(codec)
            ),
        ));
    }
    let game = string_field(object.get("game"), "game")?;
    if game.is_empty() || game.len() > MAX_DISCOVERY_SLUG_OCTETS {
        return Err(SnapshotRefusal::named(
            "game",
            format!(
                "must be a non-empty lookup key at most MAX_DISCOVERY_SLUG_OCTETS ({MAX_DISCOVERY_SLUG_OCTETS}) octets, got {} octets",
                game.len()
            ),
        ));
    }
    let user = match object.get("user") {
        Some(user) => {
            let user = string_field(Some(user), "user")?;
            check_token(user, "user")?;
            Some(user.to_string())
        }
        None => None,
    };

    let entries_value = object
        .get("entries")
        .ok_or_else(|| SnapshotRefusal::named("entries", "required key missing"))?;
    let entries_raw = entries_value
        .as_array()
        .ok_or_else(|| SnapshotRefusal::named("entries", "must be an array"))?;
    let mut entries: Vec<SnapshotEntry> = Vec::new();
    let mut pair_total = 0usize;
    for entry_value in entries_raw {
        let entry_object = entry_value
            .as_object()
            .ok_or_else(|| SnapshotRefusal::named("entries", "each entry must be an object"))?;
        for key in entry_object.keys() {
            if key != "action" && key != "inputs" {
                return Err(SnapshotRefusal::named(
                    key.as_str(),
                    "unknown key in the closed entry set",
                ));
            }
        }
        let action = string_field(entry_object.get("action"), "action")?;
        check_token(action, "action")?;
        let inputs_value = entry_object
            .get("inputs")
            .ok_or_else(|| SnapshotRefusal::named("inputs", "required key missing"))?;
        let inputs_raw = inputs_value
            .as_array()
            .ok_or_else(|| SnapshotRefusal::named("inputs", "must be an array"))?;
        let mut inputs: Vec<String> = Vec::new();
        for input_value in inputs_raw {
            let input = string_field(Some(input_value), "inputs")?;
            check_token(input, "inputs")?;
            inputs.push(input.to_string());
        }
        pair_total += inputs.len();
        entries.push(SnapshotEntry {
            action: action.to_string(),
            inputs,
        });
    }
    if pair_total > MAX_SNAPSHOT_ENTRIES {
        return Err(SnapshotRefusal::one_line(format!(
            "entries: {pair_total} (action, input) pairs, past MAX_SNAPSHOT_ENTRIES ({MAX_SNAPSHOT_ENTRIES})"
        )));
    }

    let provenance_value = object
        .get("provenance")
        .ok_or_else(|| SnapshotRefusal::named("provenance", "required key missing"))?;
    let provenance_object = provenance_value.as_object().ok_or_else(|| {
        SnapshotRefusal::named("provenance", "must be a {{evidence, files}} object")
    })?;
    for key in provenance_object.keys() {
        if key != "evidence" && key != "files" {
            return Err(SnapshotRefusal::named(
                key.as_str(),
                "unknown key in the closed provenance set",
            ));
        }
    }
    let evidence = string_field(provenance_object.get("evidence"), "evidence")?;
    if evidence != "appid-table" && evidence != "install-tree-probe" {
        return Err(SnapshotRefusal::named(
            "evidence",
            format!("`{}` is outside the closed evidence set", display(evidence)),
        ));
    }
    let files_value = provenance_object
        .get("files")
        .ok_or_else(|| SnapshotRefusal::named("files", "required key missing"))?;
    let files_raw = files_value
        .as_array()
        .ok_or_else(|| SnapshotRefusal::named("files", "must be an array"))?;
    if files_raw.len() > MAX_CANDIDATE_FILES {
        return Err(SnapshotRefusal::one_line(format!(
            "files: {} entries, past MAX_CANDIDATE_FILES ({MAX_CANDIDATE_FILES})",
            files_raw.len()
        )));
    }
    let mut files: Vec<String> = Vec::new();
    for file_value in files_raw {
        let file = string_field(Some(file_value), "files")?;
        if file.is_empty() || file.starts_with('/') {
            return Err(SnapshotRefusal::named(
                "files",
                format!("`{}` is not a tree-relative path", display(file)),
            ));
        }
        // A tree-relative path is a plain `/`-separated sequence of ordinary
        // components, and nothing else resolves outside the tree it names:
        // `ParentDir`/`CurDir` spell out of the tree, and the Windows
        // spellings of an absolute path (`C:`, UNC `\\server\share`, a
        // `\`-joined drive path) must not reach a consumer's path resolver
        // dressed as relative (`schemas/bindings/v1/spec.md`, Bounds).
        let foreign_spelling = file.contains('\\') || file.contains(':');
        let only_ordinary = std::path::Path::new(file)
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)));
        if foreign_spelling || !only_ordinary {
            return Err(SnapshotRefusal::named(
                "files",
                format!(
                    "`{}` is not a plain `/`-separated tree-relative path",
                    display(file)
                ),
            ));
        }
        let components = std::path::Path::new(file)
            .components()
            .count()
            .max(file.matches('/').count() + 1);
        if components > MAX_LOCATE_DEPTH {
            return Err(SnapshotRefusal::named(
                "files",
                format!(
                    "`{}` has {components} tree-relative components, past MAX_LOCATE_DEPTH ({MAX_LOCATE_DEPTH})",
                    display(file)
                ),
            ));
        }
        files.push(file.to_string());
    }

    Ok(BindingSnapshot {
        codec: codec.to_string(),
        entries,
        format: format.to_string(),
        game: game.to_string(),
        provenance: Provenance {
            evidence: evidence.to_string(),
            files,
        },
        user,
        version,
    })
}

/// One required (or present-and-checked) string field, refused by name when
/// absent or not a string.
fn string_field<'a>(
    value: Option<&'a serde_json::Value>,
    key: &str,
) -> Result<&'a str, SnapshotRefusal> {
    let value = value.ok_or_else(|| SnapshotRefusal::named(key, "required key missing"))?;
    value
        .as_str()
        .ok_or_else(|| SnapshotRefusal::named(key, "must be a string"))
}

/// The shared token grammar `action`, every input token and `user` ride:
/// 1–`MAX_NATIVE_NAME_OCTETS` octets, no ASCII control character, and no
/// leading or trailing space (the manifest rule, kept so the table renderer
/// needs no second escape path).
fn check_token(token: &str, key: &str) -> Result<(), SnapshotRefusal> {
    if token.is_empty() {
        return Err(SnapshotRefusal::named(key, "must not be empty"));
    }
    if token.len() > MAX_NATIVE_NAME_OCTETS {
        return Err(SnapshotRefusal::one_line(format!(
            "{key}: `{}` is {} octets, past MAX_NATIVE_NAME_OCTETS ({MAX_NATIVE_NAME_OCTETS})",
            display(token),
            token.len()
        )));
    }
    if token.chars().any(char::is_control) {
        return Err(SnapshotRefusal::named(
            key,
            format!("`{}` carries a control character", display(token)),
        ));
    }
    if token.starts_with(' ') || token.ends_with(' ') {
        return Err(SnapshotRefusal::named(
            key,
            format!("`{}` carries a leading or trailing space", display(token)),
        ));
    }
    Ok(())
}

/// A value echo ready for a refusal message.
fn display(text: &str) -> String {
    escape_and_truncate(text)
}
