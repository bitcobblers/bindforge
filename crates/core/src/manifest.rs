//! The game manifest of ADR-013: `schemas/manifests/v1/` — the record of what
//! a game *is*: its slug id and integer data revision, its codec family id, the
//! action table joining the game's own names to taxonomy ids, the constraints
//! a transform must respect, and the ids of the presets the game ships.
//!
//! Every fact here is game data reviewed per PR; the taxonomy facts it joins
//! to stay where they already live. The action row is two fields, not three:
//! `"<action-id> <native-name>"`. A third column for axis kind would restate,
//! per row, a fact `schemas/action-ids/v1/registry.toml` already owns per id —
//! two answers to one fact and the drift channel the closed entry sets exist
//! to refuse. A consumer that wants the axis kind asks `ActionRegistry::axis_kind`.
//!
//! The row splits at its *first* space: an action id never carries one (the id
//! grammar pins it), while a game-native name routinely does, so the split
//! point is unique and the whole remainder is the name. That name is the
//! format's only free-text column, so it carries its own grammar —
//! `MAX_NATIVE_NAME_OCTETS`, no ASCII control character, no leading or
//! trailing space — which is what makes the encoding round-trip byte-exactly,
//! `#` and every glyph the closed escape table decodes included.
//!
//! The split between parse level and load level is deliberate. Every rule the
//! conformance corpus can express refuses at `parse` alone, which keeps
//! `schemas/manifests/v1/fixtures/invalid/` registry-free; the two cross-file
//! rules — the registry join and the `id`-equals-directory-name rule — wait
//! for `load_manifest`, where the registry and the game directory are in hand.
//! No new `CoreError` variant enters with this format: every refusal is
//! `InvalidValue`, `MissingEntry`, or the version gate, in the single-line,
//! escaped, truncating, path-free message `schemas/README.md` promises.

use std::collections::BTreeMap;
use std::path::Path;

use crate::action::{ActionId, ActionRegistry};
use crate::document::{Document, parse_file, string_array, string_entry, string_value};
use crate::error::CoreError;
use crate::input::InputId;
use crate::placement::is_slug_name;
use crate::toml::Value;

/// Parse budget: the largest `actions`, `dead-keys`, `docked-actions`, or
/// `presets` array the grammar admits. Each cap sits well above the baselines
/// it was surveyed for (the 40-id registry; TF2-scale bind lists), and it is
/// what stops a hostile manifest driving the load-time join over an unbounded
/// table. The shared 1 MiB / 16-deep budgets (`toml`) still bind first.
pub const MAX_MANIFEST_ROWS: usize = 256;

/// Parse budget: the largest `exclusive-groups` array the grammar admits.
pub const MAX_CONSTRAINT_GROUPS: usize = 64;

/// Parse budget: the most members one exclusivity group may name. The bound
/// is what stops a hostile group building an unbounded membership set.
pub const MAX_GROUP_MEMBERS: usize = 16;

/// Parse budget: the longest game-native action name, in octets. The cap is
/// what bounds a refusal that has to echo the name, so one hostile row cannot
/// inflate log volume past the message bound.
pub const MAX_NATIVE_NAME_OCTETS: usize = 128;

/// The length bound an `id`, `codec`, or preset id shares with the slug
/// grammar: the DNS-label bound RFC 1035 §2.3.1 publishes, the same one the
/// action-id segment grammar already borrows, so no unbounded string rides a
/// displayed refusal.
const MAX_SLUG_OCTETS: usize = 63;

/// The closed fingerprint algorithm set, in its one member: `sha256-` then
/// standard base64 decoding to exactly 32 octets. Algorithm agility is a
/// future format major, and the value is provenance data — which bytes it
/// covers and what a mismatch means are ADR-017's.
const FINGERPRINT_ALGORITHM: &str = "sha256-";

/// The octets one `sha256` fingerprint digest must decode to.
const FINGERPRINT_DIGEST_OCTETS: usize = 32;

/// One `actions` row: a taxonomy id and the name the game itself uses for it,
/// exactly as written — the name is what ADR-014's `seeded-from` and ADR-015's
/// explanations display, so it is never canonicalized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestAction {
    pub action: ActionId,
    pub native_name: String,
}

/// One exclusivity group: input ids the game will not honour simultaneously.
/// ADR-015 refuses any member of one as a transform target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExclusivityGroup {
    pub inputs: Vec<InputId>,
}

/// One docked row: an action the game hard-docks to an input, so that input
/// is never a transform target, and the action side names *why* a refusal
/// reads the way it does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockedAction {
    pub action: ActionId,
    pub input: InputId,
}

/// One `manifests/v1` document: the game facts, in document order throughout.
/// Row, group, and array order is data — `serialize` writes it back as it was
/// read, so the harness's re-parse comparison is equality, not luck.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    pub version: String,
    pub id: String,
    pub revision: i64,
    pub codec: String,
    pub fingerprint: Option<String>,
    pub actions: Vec<ManifestAction>,
    pub exclusive_groups: Vec<ExclusivityGroup>,
    pub dead_keys: Vec<InputId>,
    pub docked_actions: Vec<DockedAction>,
    pub presets: Vec<String>,
}

/// Builds a manifest from one `manifests` document's entries, after the closed
/// entry set has refused anything outside the entry table: the identity and
/// revision rules, the fingerprint grammar, the row encoding, the
/// one-fact-per-document rules, and the named budgets. Every rule here is a
/// parse-level rule — this is what the conformance corpus holds the reference
/// reader to, registry-free.
pub(crate) fn manifest_from_entries(
    entries: &BTreeMap<String, Value>,
    version: String,
) -> Result<Manifest, CoreError> {
    let id = string_entry(entries, "id")?;
    check_slug("id", &id)?;

    let revision = match entries.get("revision") {
        None => {
            return Err(CoreError::MissingEntry {
                key: "revision".to_string(),
            });
        }
        Some(Value::Integer(value)) => *value,
        Some(_) => {
            return Err(CoreError::InvalidValue {
                key: "revision".to_string(),
                message: "the data revision is a decimal integer".to_string(),
            });
        }
    };
    if revision < 1 {
        let message = format!("`{revision}` is not a data revision: revisions start at 1");
        return Err(CoreError::InvalidValue {
            key: "revision".to_string(),
            message,
        });
    }

    let codec = string_entry(entries, "codec")?;
    check_slug("codec", &codec)?;

    let fingerprint = match entries.get("fingerprint") {
        None => None,
        Some(value) => {
            let text = string_value("fingerprint", value)?;
            check_fingerprint(&text)?;
            Some(text)
        }
    };

    let actions = parse_actions(entries)?;
    let exclusive_groups = parse_groups(entries)?;
    let dead_keys = parse_dead_keys(entries)?;
    let docked_actions = parse_docked(entries)?;

    // `dead-keys` and `docked-actions` are disjoint: a key the game both will
    // not honour at all and hard-docks to an action is two answers to one
    // fact about one key.
    for docked in &docked_actions {
        if dead_keys.contains(&docked.input) {
            let text = docked.input.as_str();
            let message = format!("`{text}` is both a dead key and a docked input");
            return Err(CoreError::InvalidValue {
                key: "docked-actions".to_string(),
                message,
            });
        }
    }

    let presets = parse_presets(entries)?;

    Ok(Manifest {
        version,
        id,
        revision,
        codec,
        fingerprint,
        actions,
        exclusive_groups,
        dead_keys,
        docked_actions,
        presets,
    })
}

/// Loads the manifest of one game from `game_dir/manifest.toml`, applying the
/// two cross-file rules the corpus cannot carry because they need the
/// directory and the registry in hand:
///
/// 1. the file is read under the shared size budget, before allocation, and
///    every parse-level rule of the format applies (`document::parse_file`);
/// 2. the directory's file-name segment must equal the `id` entry
///    byte-for-byte — the name↔location mismatch a validator exists to catch.
///    A malformed id is already refused at parse, and the refusal below
///    echoes only the last path segment, never a local path: `source_path` is
///    the error contract's channel for paths;
/// 3. every taxonomy id the document names — each `actions` row and each
///    `docked-actions` row — must be in the loaded registry. A manifest
///    naming an action the taxonomy does not own joins nothing on the profile
///    side, and ADR-011 grows the taxonomy only through manifest review: a
///    manifest PR that names a new id must add its `registry.toml` row in the
///    same PR, which is exactly the review this refusal enforces.
///
/// `codec` and the preset ids are *not* registry-checked: no implementation
/// set exists to check them against until ADR-017, and refusing an unknown
/// codec would be applying a rule the contract does not have. Axis kind is
/// never read from the document either: a consumer that wants it asks
/// `ActionRegistry::axis_kind`.
pub fn load_manifest(game_dir: &Path, registry: &ActionRegistry) -> Result<Manifest, CoreError> {
    let document = parse_file(&game_dir.join("manifest.toml"))?;
    let Document::Manifest(manifest) = document else {
        return Err(CoreError::InvalidValue {
            key: "format".to_string(),
            message: "`manifest.toml` must hold a manifests document".to_string(),
        });
    };

    let Some(segment) = game_dir.file_name().and_then(|name| name.to_str()) else {
        return Err(CoreError::InvalidValue {
            key: "id".to_string(),
            message: "a game directory is named for the manifest's id".to_string(),
        });
    };
    if segment != manifest.id {
        let message = format!(
            "`{segment}` is not the manifest's id `{}`: the game directory's name must equal \
             the `id` entry byte-for-byte",
            manifest.id
        );
        return Err(CoreError::InvalidValue {
            key: "id".to_string(),
            message,
        });
    }

    for row in &manifest.actions {
        if !registry.contains(&row.action) {
            let message = format!(
                "`{}` is not an action id the loaded registry carries",
                row.action.as_str()
            );
            return Err(CoreError::InvalidValue {
                key: "actions".to_string(),
                message,
            });
        }
    }
    for row in &manifest.docked_actions {
        if !registry.contains(&row.action) {
            let message = format!(
                "`{}` is not an action id the loaded registry carries",
                row.action.as_str()
            );
            return Err(CoreError::InvalidValue {
                key: "docked-actions".to_string(),
                message,
            });
        }
    }
    Ok(manifest)
}

/// The `actions` entry: a bounded array of `"<action-id> <native-name>"`
/// rows, each action id once and each native name once.
fn parse_actions(entries: &BTreeMap<String, Value>) -> Result<Vec<ManifestAction>, CoreError> {
    let Some(value) = entries.get("actions") else {
        return Err(CoreError::MissingEntry {
            key: "actions".to_string(),
        });
    };
    let raw_rows = string_array("actions", value)?;
    check_row_count("actions", raw_rows.len())?;
    let mut actions: Vec<ManifestAction> = Vec::new();
    for raw in raw_rows {
        let row = parse_row(&raw)?;
        let repeated = actions.iter().any(|other| other.action == row.action);
        if repeated {
            let message = format!("`{}` is listed twice by the `actions` entry", raw);
            return Err(CoreError::InvalidValue {
                key: "actions".to_string(),
                message,
            });
        }
        let repeated = actions
            .iter()
            .any(|other| other.native_name == row.native_name);
        if repeated {
            let message = format!(
                "`{}` names two actions in the `actions` entry",
                row.native_name
            );
            return Err(CoreError::InvalidValue {
                key: "actions".to_string(),
                message,
            });
        }
        actions.push(row);
    }
    Ok(actions)
}

/// One `actions` element, split at its first space: the head is a
/// grammar-valid action id, the whole remainder is the native name. A row
/// with no space is a row with no name; an action with no native name answers
/// "what does this game call it" with nothing.
fn parse_row(raw: &str) -> Result<ManifestAction, CoreError> {
    let Some((head, name)) = raw.split_once(' ') else {
        let message = format!("`{raw}` is not an `<action-id> <native-name>` row: no name");
        return Err(CoreError::InvalidValue {
            key: "actions".to_string(),
            message,
        });
    };
    let parsed = ActionId::parse(head);
    let action = parsed.map_err(|err| name_the_entry(err, "actions"))?;
    check_native_name(name)?;
    Ok(ManifestAction {
        action,
        native_name: name.to_string(),
    })
}

/// The native name's own grammar — the only free-text column in the format,
/// and the reason the name carries its own rules: 1 to `MAX_NATIVE_NAME_OCTETS`
/// octets, no ASCII control character (even one the escape table decodes), no
/// leading space (an empty remainder), no trailing space (the name must
/// round-trip byte-exactly). Spaces, `#`, and every glyph the closed escape
/// table decodes are data.
fn check_native_name(name: &str) -> Result<(), CoreError> {
    let fault = |reason: &str| CoreError::InvalidValue {
        key: "actions".to_string(),
        message: format!("`{name}` is not a native name: {reason}"),
    };
    if name.is_empty() {
        return Err(fault("the name is empty"));
    }
    if name.len() > MAX_NATIVE_NAME_OCTETS {
        return Err(fault(&format!(
            "longer than the {MAX_NATIVE_NAME_OCTETS} octet name budget"
        )));
    }
    if name.starts_with(' ') {
        return Err(fault("it starts with a space"));
    }
    if name.ends_with(' ') {
        return Err(fault("it ends with a space"));
    }
    if name.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(fault("it carries a control character"));
    }
    Ok(())
}

/// The `exclusive-groups` entry: at most `MAX_CONSTRAINT_GROUPS` groups, each
/// naming `2..=MAX_GROUP_MEMBERS` distinct input ids from the closed
/// vocabulary, and an input id appears in at most one group — two groups
/// sharing a key would make "is this key exclusive" answer two ways.
fn parse_groups(entries: &BTreeMap<String, Value>) -> Result<Vec<ExclusivityGroup>, CoreError> {
    let Some(value) = entries.get("exclusive-groups") else {
        return Ok(Vec::new());
    };
    let raw_groups = string_array("exclusive-groups", value)?;
    if raw_groups.len() > MAX_CONSTRAINT_GROUPS {
        return Err(CoreError::InvalidValue {
            key: "exclusive-groups".to_string(),
            message: format!(
                "more than {MAX_CONSTRAINT_GROUPS} groups: the `exclusive-groups` budget"
            ),
        });
    }
    let mut groups: Vec<ExclusivityGroup> = Vec::new();
    for raw in raw_groups {
        let fields: Vec<&str> = raw.split(' ').collect();
        if fields.len() < 2 {
            let message =
                format!("`{raw}` is not an exclusivity group: a group relates at least two inputs");
            return Err(CoreError::InvalidValue {
                key: "exclusive-groups".to_string(),
                message,
            });
        }
        if fields.len() > MAX_GROUP_MEMBERS {
            let message =
                format!("`{raw}` is wider than the {MAX_GROUP_MEMBERS} members one group may name");
            return Err(CoreError::InvalidValue {
                key: "exclusive-groups".to_string(),
                message,
            });
        }
        let mut inputs: Vec<InputId> = Vec::new();
        for field in fields {
            let parsed = InputId::parse(field);
            let input = parsed.map_err(|err| name_the_entry(err, "exclusive-groups"))?;
            if inputs.contains(&input) {
                let message = format!("`{raw}` names `{field}` twice in one group");
                return Err(CoreError::InvalidValue {
                    key: "exclusive-groups".to_string(),
                    message,
                });
            }
            let shared = groups.iter().any(|group| group.inputs.contains(&input));
            if shared {
                let message = format!("`{field}` appears in two `exclusive-groups` entries");
                return Err(CoreError::InvalidValue {
                    key: "exclusive-groups".to_string(),
                    message,
                });
            }
            inputs.push(input);
        }
        groups.push(ExclusivityGroup { inputs });
    }
    Ok(groups)
}

/// The `dead-keys` entry: bounded, each member a closed-vocabulary input id,
/// each key once — a repeated member states one game fact twice, and the
/// set-membership lookups downstream would read it as two facts.
fn parse_dead_keys(entries: &BTreeMap<String, Value>) -> Result<Vec<InputId>, CoreError> {
    let Some(value) = entries.get("dead-keys") else {
        return Ok(Vec::new());
    };
    let raw_keys = string_array("dead-keys", value)?;
    check_row_count("dead-keys", raw_keys.len())?;
    let mut dead_keys: Vec<InputId> = Vec::new();
    for raw in raw_keys {
        let parsed = InputId::parse(&raw);
        let input = parsed.map_err(|err| name_the_entry(err, "dead-keys"))?;
        if dead_keys.contains(&input) {
            let message = format!("`{raw}` is listed twice by the `dead-keys` entry");
            return Err(CoreError::InvalidValue {
                key: "dead-keys".to_string(),
                message,
            });
        }
        dead_keys.push(input);
    }
    Ok(dead_keys)
}

/// The `docked-actions` entry: bounded, exactly
/// `"<action-id> <input-id>"` per element — both fields carry no space, which
/// is what makes this split unambiguous — and each action id once, because
/// the action side explains a refused transform target and two docked rows
/// for one action are two answers to one question.
fn parse_docked(entries: &BTreeMap<String, Value>) -> Result<Vec<DockedAction>, CoreError> {
    let Some(value) = entries.get("docked-actions") else {
        return Ok(Vec::new());
    };
    let raw_rows = string_array("docked-actions", value)?;
    check_row_count("docked-actions", raw_rows.len())?;
    let mut docked: Vec<DockedAction> = Vec::new();
    for raw in raw_rows {
        let fault = || CoreError::InvalidValue {
            key: "docked-actions".to_string(),
            message: format!("`{raw}` is not an `<action-id> <input-id>` row"),
        };
        let Some((head, tail)) = raw.split_once(' ') else {
            return Err(fault());
        };
        if tail.contains(' ') {
            return Err(fault());
        }
        let parsed_action =
            ActionId::parse(head).map_err(|err| name_the_entry(err, "docked-actions"))?;
        let parsed_input =
            InputId::parse(tail).map_err(|err| name_the_entry(err, "docked-actions"))?;
        let repeated = docked.iter().any(|row| row.action == parsed_action);
        if repeated {
            let message = format!("`{head}` is docked twice by the `docked-actions` entry");
            return Err(CoreError::InvalidValue {
                key: "docked-actions".to_string(),
                message,
            });
        }
        docked.push(DockedAction {
            action: parsed_action,
            input: parsed_input,
        });
    }
    Ok(docked)
}

/// The `presets` entry: bounded, each element a slug-grammar preset id, each
/// once — a repeated id lists one built-in twice, which ADR-014's
/// `seeded-from` would join twice. Preset *contents* are ADR-016's and
/// ADR-017's; this entry records only that the game ships the preset.
fn parse_presets(entries: &BTreeMap<String, Value>) -> Result<Vec<String>, CoreError> {
    let Some(value) = entries.get("presets") else {
        return Ok(Vec::new());
    };
    let raw_presets = string_array("presets", value)?;
    check_row_count("presets", raw_presets.len())?;
    let mut presets: Vec<String> = Vec::new();
    for raw in raw_presets {
        check_slug("presets", &raw)?;
        if presets.contains(&raw) {
            let message = format!("`{raw}` is listed twice by the `presets` entry");
            return Err(CoreError::InvalidValue {
                key: "presets".to_string(),
                message,
            });
        }
        presets.push(raw);
    }
    Ok(presets)
}

/// The shared array cap: `MAX_MANIFEST_ROWS` bounds every array-shaped entry,
/// which is why the design has no fifth constant.
fn check_row_count(key: &str, length: usize) -> Result<(), CoreError> {
    if length > MAX_MANIFEST_ROWS {
        return Err(CoreError::InvalidValue {
            key: key.to_string(),
            message: format!("more than {MAX_MANIFEST_ROWS} elements: the manifest array budget"),
        });
    }
    Ok(())
}

/// The slug grammar an `id`, a `codec` id, and each preset id ride —
/// `is_slug_name` plus the DNS-label length bound the action-id grammar
/// already borrows. The refusal echoes the offender; `CoreError`'s rendering
/// escapes and truncates what it echoes.
fn check_slug(key: &str, value: &str) -> Result<(), CoreError> {
    if !is_slug_name(value) {
        let message = format!("`{value}` is not a slug: expected `[a-z][a-z0-9-]*`");
        return Err(CoreError::InvalidValue {
            key: key.to_string(),
            message,
        });
    }
    if value.len() > MAX_SLUG_OCTETS {
        let message = format!("`{value}` is longer than the {MAX_SLUG_OCTETS} octet label bound");
        return Err(CoreError::InvalidValue {
            key: key.to_string(),
            message,
        });
    }
    Ok(())
}

/// The fingerprint grammar, and nothing more: the closed one-algorithm set
/// (`sha256-`) followed by standard base64 that decodes to exactly 32 octets.
/// A digest of another length is a different provenance claim wearing the same
/// spelling. What bytes are hashed and what a mismatch mean are ADR-017's.
fn check_fingerprint(text: &str) -> Result<(), CoreError> {
    let fault = |reason: String| CoreError::InvalidValue {
        key: "fingerprint".to_string(),
        message: format!("`{text}` is not a fingerprint: {reason}"),
    };
    let Some(body) = text.strip_prefix(FINGERPRINT_ALGORITHM) else {
        return Err(fault(
            "the algorithm set is `sha256-` only: algorithm agility rides a new format major"
                .to_string(),
        ));
    };
    let Some(digest) = decode_base64(body) else {
        return Err(fault("the body is not standard base64".to_string()));
    };
    if digest.len() != FINGERPRINT_DIGEST_OCTETS {
        return Err(fault(format!(
            "the body decodes to {} octets, not {FINGERPRINT_DIGEST_OCTETS}",
            digest.len()
        )));
    }
    Ok(())
}

/// Strict standard-base64 decoding: the alphabet is `A-Za-z0-9+/`, `=` padding
/// only at the end of the final quartet, the length a multiple of four. A body
/// outside this never decodes through a permissive fallback, which would let
/// two spellings claim one digest.
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return None;
    }
    let quartets = bytes.len() / 4;
    let mut out: Vec<u8> = Vec::with_capacity(quartets * 3);
    for (index, quartet) in bytes.chunks(4).enumerate() {
        let last = index + 1 == quartets;
        let mut significant = 4;
        if last {
            while significant > 0 && quartet[significant - 1] == b'=' {
                significant -= 1;
            }
            if significant < 2 {
                return None;
            }
        }
        let mut sextets = [0u32; 4];
        for (position, byte) in quartet.iter().enumerate() {
            if !last && *byte == b'=' {
                return None;
            }
            if position >= significant {
                if *byte != b'=' {
                    return None;
                }
                continue;
            }
            sextets[position] = sextet(*byte)?;
        }
        let bits = (sextets[0] << 18) | (sextets[1] << 12) | (sextets[2] << 6) | sextets[3];
        out.push(((bits >> 16) & 0xff) as u8);
        if significant >= 3 {
            out.push(((bits >> 8) & 0xff) as u8);
        }
        if significant >= 4 {
            out.push((bits & 0xff) as u8);
        }
    }
    Some(out)
}

/// The value of one base64 alphabet character, or `None` outside the
/// alphabet.
fn sextet(byte: u8) -> Option<u32> {
    match byte {
        b'A'..=b'Z' => Some(u32::from(byte - b'A')),
        b'a'..=b'z' => Some(u32::from(byte - b'a') + 26),
        b'0'..=b'9' => Some(u32::from(byte - b'0') + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Re-keys a refusal raised inside an id grammar so it also names the
/// manifest entry the offending id sat in.
fn name_the_entry(err: CoreError, entry: &str) -> CoreError {
    match err {
        CoreError::InvalidValue { message, .. } => CoreError::InvalidValue {
            key: entry.to_string(),
            message,
        },
        other => other,
    }
}
