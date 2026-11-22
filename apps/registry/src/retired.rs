//! The retired-id document (`027-L2`, §Issued entries' last subsection).
//!
//! "Deleting a record never frees its id" needs memory the `026-C3` shape cannot
//! carry: an id that leaves `games` must still be known, or the mint rule would
//! hand it to the next store record and `027-L2` would be a name without a
//! mechanism. So the instance owns one more small document,
//! `<data>/bindforge/registry/retired-ids-v1.json`, spelled
//! `{version: 1, ids: [<game_id>…]}` in byte order, absent means empty, same
//! owner-only tmp-and-rename discipline as every other file here.
//!
//! Two properties the rest of the design leans on:
//!
//! - **The append is a set insert.** A delete whose id is already present changes
//!   nothing, which is what lets a run that died between the delete's two persists
//!   be retried: the overlap `games ∩ retired ≠ ∅` is a legal, recoverable state —
//!   the record keeps answering and the retried delete completes the removal —
//!   because a repeated insert cannot produce the duplicate that would break byte
//!   order.
//! - **The cap fails closed.** A write whose rename buffer breaches
//!   `MAX_RETIRED_IDS_OCTETS` is the named refusal `retired id list full` and
//!   mutates nothing. Failing open there would re-open a retired id, which is
//!   `027-L2` broken by arithmetic.

use std::path::{Path, PathBuf};

use crate::constants;
use crate::corpus::LoadError;
use crate::persist;

/// The document's three fields, exactly.
const DOCUMENT_ENTRIES: [&str; 2] = ["version", "ids"];

/// The retired ids, in byte order, with the file they persist to.
#[derive(Debug, Clone)]
pub struct RetiredIds {
    path: PathBuf,
    ids: Vec<String>,
}

impl RetiredIds {
    /// The empty set over `path`, for an instance whose file is not there yet.
    #[must_use]
    pub fn empty(path: PathBuf) -> Self {
        Self {
            path,
            ids: Vec::new(),
        }
    }

    /// The file this set persists to.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads and validates the document, with an absent file as the empty set. The
    /// octet cap is paid on `fs::metadata` before the read (A2), like every other
    /// document this instance loads.
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        let refused = |detail: String| LoadError::at(path, detail);
        let Some(bytes) = persist::read_capped(path, constants::MAX_RETIRED_IDS_OCTETS)
            .map_err(|error| LoadError::at(path, error.reason.clone()))?
        else {
            return Ok(Self::empty(path.to_path_buf()));
        };
        let document: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
            refused(format!(
                "the retired-id document is not readable JSON, and `version` must be the bare \
                 integer {}: {error}",
                constants::DOCUMENT_VERSION
            ))
        })?;
        let fields = document
            .as_object()
            .ok_or_else(|| refused("the retired-id document is not a JSON object".to_string()))?;
        if let Some(extra) = fields
            .keys()
            .find(|name| !DOCUMENT_ENTRIES.contains(&name.as_str()))
        {
            return Err(refused(format!(
                "the retired-id document carries the field `{extra}`, which `027-L2` does not \
                 define"
            )));
        }
        let version = fields.get("version").and_then(serde_json::Value::as_u64);
        if version != Some(constants::DOCUMENT_VERSION) {
            return Err(refused("`version` is not the bare integer 1".to_string()));
        }
        let entries = fields
            .get("ids")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| refused("`ids` is not a list".to_string()))?;
        let mut ids: Vec<String> = Vec::with_capacity(entries.len());
        for entry in entries {
            let id = entry
                .as_str()
                .ok_or_else(|| refused("an `ids` entry is not a string".to_string()))?;
            if !crate::corpus::game_id_is_valid(id) {
                return Err(refused(format!(
                    "`{}` is not a `game_id` this registry would mint or curate",
                    bindforge_core::escape_and_truncate(id)
                )));
            }
            if let Some(last) = ids.last() {
                match last.as_str().cmp(id) {
                    std::cmp::Ordering::Less => {}
                    std::cmp::Ordering::Equal => {
                        return Err(refused(format!(
                            "`{id}` appears twice: the retired list is a set, and a duplicate \
                             is the state the insert rule exists to prevent"
                        )));
                    }
                    std::cmp::Ordering::Greater => {
                        return Err(refused(
                            "`ids` is not in byte order, and an out-of-order list is refused \
                             rather than resorted"
                                .to_string(),
                        ));
                    }
                }
            }
            ids.push(id.to_string());
        }
        Ok(Self {
            path: path.to_path_buf(),
            ids,
        })
    }

    /// Whether `game_id` is retired — which never suppresses an answer, and always
    /// refuses a mint (§Issued entries, `027-L5`'s overlap posture).
    #[must_use]
    pub fn contains(&self, game_id: &str) -> bool {
        self.ids
            .binary_search_by(|held| held.as_str().cmp(game_id))
            .is_ok()
    }

    /// Adds one id in order. `false` when it was already there, which is a no-op
    /// rather than the duplicate that would break byte order.
    pub fn insert(&mut self, game_id: &str) -> bool {
        match self.ids.binary_search_by(|held| held.as_str().cmp(game_id)) {
            Ok(_) => false,
            Err(at) => {
                self.ids.insert(at, game_id.to_string());
                true
            }
        }
    }

    /// The whole document as the bytes a save renames into place.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let ids: Vec<&str> = self.ids.iter().map(String::as_str).collect();
        serde_json::to_vec(&serde_json::json!({
            "version": constants::DOCUMENT_VERSION,
            "ids": ids,
        }))
        .expect("a retired-id document of plain strings serializes")
    }
}
