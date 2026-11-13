//! The action taxonomy as a typed model: ids, axis kinds, and the registry
//! that ships them (ADR-011, decision two).
//!
//! The id set lives in `schemas/action-ids/v1/registry.toml`, not in Rust
//! literals — the same rule the reach tables already follow
//! (`crates/core/src/reach.rs`): this module loads the contract's data and
//! never restates it, so the crate and the contract cannot drift into two
//! different id sets. What this module owns is the grammar `spec.md` states:
//! the two-segment id rule, the closed three-value axis-kind set, the closed
//! entry set of an `action-ids` document, and the rule that the `registry.toml`
//! shipped inside the versioned directory is the registry — an `action-ids`
//! document from elsewhere is data, never a source of `axis_kind`.
//!
//! Ids are a compatibility promise (ADR-011, Consequences), so comparison is
//! byte-exact: no case folding, no canonicalization, and no defaulting. An id
//! the registry does not list is simply absent — `axis_kind` answers `None`,
//! because a typo that passes through as a valid binding matching nothing is
//! the failure the closed entry sets already refuse.
//!
//! Every refusal reuses an existing [`CoreError`] variant, so no new variant
//! enters the error contract: `InvalidValue` for a malformed id, a repeat of
//! an id, an entry outside the entry set, or a `version` that disagrees with
//! the directory holding it; `MissingEntry` for an absent axis array. The load
//! goes through `document::parse_file`, so the size budget binds before the
//! registry is read, and a path stays out of every message (`CoreError`'s
//! `source_path` is the channel for it).

use std::collections::BTreeMap;
use std::path::Path;

use crate::document::{Document, parse_file, string_array};
use crate::error::CoreError;
use crate::toml::Value;

/// How long one id segment may be, in octets: the DNS-label bound RFC 1035
/// §2.3.1 publishes, borrowed rather than guessed, so the number a rejection
/// names is a published one.
const MAX_SEGMENT_OCTETS: usize = 63;

/// The rule an id breaks when it does not hold exactly two segments.
const SEGMENT_COUNT_RULE: &str = "an action id is exactly two `.`-separated segments";

/// The rule an id breaks when one of its segments is malformed.
const SEGMENT_SHAPE_RULE: &str = "expected 1-63 octets of `a-z`, `0-9`, and `-`";

/// The rule's edge condition, stated apart so each refusal names one rule.
const SEGMENT_EDGE_RULE: &str = "letter first, letter or digit last";

/// Which axis an action sits on: the array of the registry that lists it. The
/// three values are exactly the three array names the entry table defines, so
/// there is no id without an axis kind and no axis kind to forget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisKind {
    /// One half of a left/right pair bound as two separately-bound endpoints,
    /// so mirroring swaps two bindings. The only kind a polarity flip may
    /// touch (ADR-015).
    DiscreteLateral,
    /// One graded lateral control whose mirror is a sign flip on that control
    /// — camera look, whose inversion is a game option, never a key swap.
    ContinuousLateral,
    /// Neither of the two: everything that is not a lateral control, including
    /// forward/back, which is not lateral.
    None,
}

impl AxisKind {
    /// The contract-tree spelling of this axis kind: the name of the registry
    /// array its ids come from.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            AxisKind::DiscreteLateral => "discrete-lateral",
            AxisKind::ContinuousLateral => "continuous-lateral",
            AxisKind::None => "none",
        }
    }

    /// The axis kind one of those spellings names, or `None` when it names
    /// neither — the closed axis-kind set `spec.md` states. A fourth spelling
    /// would be a fourth axis kind the taxonomy does not have.
    #[must_use]
    pub fn from_name(name: &str) -> Option<AxisKind> {
        match name {
            "discrete-lateral" => Some(AxisKind::DiscreteLateral),
            "continuous-lateral" => Some(AxisKind::ContinuousLateral),
            "none" => Some(AxisKind::None),
            _ => None,
        }
    }

    /// The three kinds, in the order an `action-ids` document writes their
    /// arrays and the order `serialize` writes them back.
    const ALL: [AxisKind; 3] = [
        AxisKind::DiscreteLateral,
        AxisKind::ContinuousLateral,
        AxisKind::None,
    ];
}

/// One action id, validated against the grammar `spec.md` states: exactly two
/// `.`-separated segments, each a lowercase label. The text is kept exactly as
/// written — comparison is byte-exact, so two spellings never name one action,
/// which is the churn ADR-011 forbids.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActionId {
    text: String,
}

impl ActionId {
    /// Validates one id. A wrong segment count or a malformed segment is an
    /// `InvalidValue` echoing the id as written; `CoreError`'s rendering is
    /// what escapes and truncates what it echoes.
    pub fn parse(text: &str) -> Result<ActionId, CoreError> {
        let Some((first, second)) = text.split_once('.') else {
            return Err(invalid_id(text, SEGMENT_COUNT_RULE));
        };
        if second.contains('.') {
            return Err(invalid_id(text, SEGMENT_COUNT_RULE));
        }
        for segment in [first, second] {
            if !is_well_formed_segment(segment) {
                let fault = format!(
                    "`{segment}` is not an id segment: {SEGMENT_SHAPE_RULE}, {SEGMENT_EDGE_RULE}"
                );
                return Err(invalid_id(text, &fault));
            }
        }
        Ok(ActionId {
            text: text.to_string(),
        })
    }

    /// The id exactly as written, never canonicalized.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// One action registry: the ids of one `action-ids` document, each carrying
/// the axis kind of the array that lists it. An id appears once across the
/// three arrays, so the registry is one closed set, and a grammar-valid id it
/// does not list is simply not present.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionRegistry {
    /// The major this registry declares, checked against the directory it is
    /// loaded from by `load_action_registry`.
    pub version: String,
    entries: BTreeMap<ActionId, AxisKind>,
}

impl ActionRegistry {
    /// The registry's ids, ordered by id. That order is stable across a
    /// document round-trip: `serialize` writes the three arrays in this order,
    /// so re-parsing its output yields the same registry.
    #[must_use]
    pub fn ids(&self) -> Vec<ActionId> {
        self.entries.keys().cloned().collect()
    }

    /// Whether the registry's closed set lists this id.
    #[must_use]
    pub fn contains(&self, id: &ActionId) -> bool {
        self.entries.contains_key(id)
    }

    /// The axis kind of the array that lists this id, or `None` when the
    /// registry does not list it: no default, no guess.
    #[must_use]
    pub fn axis_kind(&self, id: &ActionId) -> Option<AxisKind> {
        self.entries.get(id).copied()
    }
}

/// Builds a registry from one `action-ids` document's entries: the three
/// required arrays, every id grammar-checked, every id listed once across all
/// three. `document::parse` calls this from the format's arm, after the closed
/// entry set has refused anything outside the entry table.
pub(crate) fn registry_from_entries(
    entries: &BTreeMap<String, Value>,
    version: String,
) -> Result<ActionRegistry, CoreError> {
    let mut registry = ActionRegistry {
        version,
        entries: BTreeMap::new(),
    };
    for kind in AxisKind::ALL {
        let key = kind.name();
        let Some(value) = entries.get(key) else {
            return Err(CoreError::MissingEntry {
                key: key.to_string(),
            });
        };
        for raw in string_array(key, value)? {
            // Two statements rather than one 61-column chain: `chain_width` is
            // 60, so `ActionId::parse(&raw).map_err(…)?` is not a shape the
            // formatter keeps, and the gate runs `cargo fmt --all -- --check`.
            let parsed = ActionId::parse(&raw);
            let id = parsed.map_err(|err| name_the_entry(err, key))?;
            if registry.entries.insert(id, kind).is_some() {
                let message = format!("`{raw}` appears twice across the three axis arrays");
                return Err(CoreError::InvalidValue {
                    key: key.to_string(),
                    message,
                });
            }
        }
    }
    Ok(registry)
}

/// Loads the action registry from the `registry.toml` sitting directly in one
/// `schemas/action-ids/v<N>/` version directory.
///
/// That file, in that directory, is the registry, and this is the only way to
/// get one: an `action-ids` document parsed from anywhere else is data. The
/// declared `version` must equal the version segment of `dir`, which is what
/// makes a widened id grammar ride `action-ids/v2` instead of breaking every
/// v1 reader.
pub fn load_action_registry(dir: &Path) -> Result<ActionRegistry, CoreError> {
    let document = parse_file(&dir.join("registry.toml"))?;
    let Document::ActionIds(registry) = document else {
        return Err(CoreError::InvalidValue {
            key: "format".to_string(),
            message: "`registry.toml` must hold an action-ids document".to_string(),
        });
    };

    let major = dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if !is_major_dir(major) {
        return Err(CoreError::InvalidValue {
            key: "version".to_string(),
            message: "a registry directory is named for a major, like `v1`".to_string(),
        });
    }
    if registry.version != major {
        let declared = registry.version.clone();
        return Err(CoreError::InvalidValue {
            key: "version".to_string(),
            message: format!("`registry.toml` declares `{declared}` in a `{major}` directory"),
        });
    }
    Ok(registry)
}

/// The one refusal shape for a malformed id: `InvalidValue`, echoing the id as
/// written. `CoreError`'s rendering escapes control characters and truncates,
/// so a hostile id can neither forge a log line nor inflate one.
fn invalid_id(id: &str, fault: &str) -> CoreError {
    CoreError::InvalidValue {
        key: "id".to_string(),
        message: format!("`{id}` is not a valid action id: {fault}"),
    }
}

/// Re-keys a refusal raised inside the id grammar so it also names the
/// registry array the offending id was listed in.
fn name_the_entry(err: CoreError, entry: &str) -> CoreError {
    match err {
        CoreError::InvalidValue { message, .. } => CoreError::InvalidValue {
            key: entry.to_string(),
            message,
        },
        other => other,
    }
}

/// The segment rule on its own: an ASCII lowercase letter, then `a-z`, `0-9`,
/// and `-`, closing with a letter or digit, at most `MAX_SEGMENT_OCTETS`
/// octets. A non-ASCII byte fails the last test, which is what keeps the id
/// grammar byte-exact rather than case- or Unicode-folded.
fn is_well_formed_segment(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_SEGMENT_OCTETS {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() || !bytes[bytes.len() - 1].is_ascii_alphanumeric() {
        return false;
    }
    bytes
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

/// Whether a directory name is a major-version segment: `v` then digits.
fn is_major_dir(name: &str) -> bool {
    match name.strip_prefix('v') {
        Some(digits) => !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()),
        None => false,
    }
}
