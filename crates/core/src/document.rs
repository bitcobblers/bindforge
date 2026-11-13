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
//! The gate is **per-format**, because majors are per-format: `format_majors`
//! lists what each format ships, and a document declaring a major its own
//! format does not carry is refused naming that format's majors. A crate-wide
//! list alone would fail open the moment any format grows a second major —
//! ADR-012 adds `reach-classes/v2`, and without the per-format half a document
//! declaring `format = "manifests", version = "v2"` would be read with the
//! `manifests/v1` grammar, which is exactly the misreading the gate exists to
//! prevent. The crate-wide list still binds first, so a major no format ships
//! is refused naming every major the crate declares.
//!
//! Each format's entry set is closed, exactly as its `spec.md` states ("Any
//! other entry is rejected"): an entry the grammar does not define is
//! refused, not ignored. Accepting one would admit data the contract has no
//! meaning for into the same trust boundary the chain guards protect — a
//! misspelled or aliased `derives_` reads as *no ancestors at all*, quietly
//! turning a derived profile into a standalone one. A closed set is also
//! version-specific: ADR-012 adds the `keys` entry as `reach-classes/v2`, and
//! `reach-classes/v1` stays closed forever, which is why the arm picks the set
//! from the *declared* version and a refusal names that version.
//!
//! The scaffold ships the `profile/v1` and `reach-classes/v1` documents,
//! ADR-011 adds `action-ids/v1`, ADR-012 adds `reach-classes/v2` — the
//! five-class taxonomy with each hand's grade for every graded input — and
//! `keyboard-geometry/v1` — the key grid the placement predicate in
//! `placement` checks its shape rule against — and ADR-013 adds `manifests/v1`
//! — the game facts of `manifest`, whose two cross-file rules (the registry
//! join and the location rule) live in `manifest::load_manifest`, not here.
//! Design 014 (ADR-014) adds `profile/v2`: the link vocabulary of the chain —
//! `kind`, the `for` pin, the `set`/`clear`/`swap` delta lists, and the two
//! provenance labels — all of it *parse*-level, so the corpus stays
//! single-document. What needs other documents in hand (the resolved-chain
//! guards, the position rules, the manifest joins) lives in `chain_ops`, the
//! same split `manifest` uses; `profile/v1` stays exactly as shipped, entry
//! set, `serialize` bytes, and fixtures included.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use crate::action::{ActionId, ActionRegistry, AxisKind, registry_from_entries};
use crate::error::CoreError;
use crate::input::{InputId, InputNamespace};
use crate::manifest::{Manifest, manifest_from_entries};
use crate::placement::is_slug_name;
use crate::reach::{Hand, InputGrade, ReachClass};
use crate::toml::{self, Value};

/// The major versions of the contract this crate implements. Must stay equal
/// to the majors present under `schemas/*/v*/` (Test strategy #2).
const MAJOR_V1: &str = "v1";
/// `pub(crate)` because `chain_ops` builds chain documents as well as reads
/// them, and every chain link is written in this major.
pub(crate) const MAJOR_V2: &str = "v2";

/// The format names this crate parses.
const SUPPORTED_FORMATS: &[&str] = &[
    "action-ids",
    "keyboard-geometry",
    "manifests",
    "profile",
    "reach-classes",
];

/// Every entry the `profile/v1` grammar defines, and the only ones it admits.
const PROFILE_V1_ENTRIES: &[&str] = &["derives", "format", "id", "version"];

/// Every entry the `profile/v2` grammar defines: v1's four unchanged in meaning
/// (`format`, `version`, `id`, `derives`, A1) plus the link vocabulary the
/// chain is written in — `kind`, the `for` pin, the three delta lists, and the
/// two provenance labels. The set is closed the same way every other format's
/// is: `fixtures/invalid/unknown-entry.toml` pins the refusal, and the retired
/// `based-on` is refused rather than read as `derives` under another name.
const PROFILE_V2_ENTRIES: &[&str] = &[
    "clear",
    "derives",
    "format",
    "for",
    "generated-by",
    "id",
    "kind",
    "seeded-from",
    "set",
    "swap",
    "version",
];

/// Every entry the `reach-classes/v1` grammar defines, and the only ones it
/// admits. This set stays closed forever: `keys` is what `v2` adds, and
/// `reach-classes/v1/fixtures/invalid/keys-entry.toml` pins that refusal.
const REACH_CLASSES_V1_ENTRIES: &[&str] = &["classes", "format", "hand", "version"];

/// Every entry the `reach-classes/v2` grammar defines: v1's four plus `keys`,
/// the key→class mapping ADR-012 owes (Decision One).
const REACH_CLASSES_V2_ENTRIES: &[&str] = &["classes", "format", "hand", "keys", "version"];

/// Every entry the `keyboard-geometry/v1` grammar defines, and the only ones
/// it admits.
const GEOMETRY_ENTRIES: &[&str] = &["format", "geometry", "keys", "version"];

/// Every entry the `action-ids/v1` grammar defines, and the only ones it
/// admits. The three axis arrays are the entry set's load-bearing part: they
/// are the id set, so an entry naming a fourth axis kind, or an `emphasis`,
/// is refused rather than ignored (ADR-011: emphasis is *not* in the
/// vocabulary).
const ACTION_IDS_ENTRIES: &[&str] = &[
    "continuous-lateral",
    "discrete-lateral",
    "format",
    "none",
    "version",
];

/// Every entry the `manifests/v1` grammar defines, and the only ones it
/// admits: the identity group (`id`, `revision`), the codec family id, the
/// optional fingerprint, the action table, the three constraint entries, and
/// the preset ids. The set is closed the same way as every other format's,
/// and the refused alternative it exists to refuse is `axis-kinds`: axis kind
/// is a fact `schemas/action-ids/v1/registry.toml` already owns per id, and a
/// manifest restating it per row is the second answer to one fact (ADR-013).
const MANIFEST_ENTRIES: &[&str] = &[
    "actions",
    "codec",
    "dead-keys",
    "docked-actions",
    "exclusive-groups",
    "fingerprint",
    "format",
    "id",
    "presets",
    "revision",
    "version",
];

/// The v2 taxonomy: the five class ids, once each, in **severity order**, worst
/// last. The array order *is* the severity scale, so two hands cannot define
/// two scales, and the rule is one rule per document — which is what lets a
/// loader compare no files. Changing the scale changes what a bound means, so
/// it rides a new major.
const REACH_CLASSES_V2_TAXONOMY: [&str; 5] = ["home", "thumb", "stretch", "far", "off-reach"];

/// The geometry-name grammar `keyboard-geometry/v1/spec.md` states, echoed in
/// the refusal so a document says which grammar it broke.
const GEOMETRY_NAME_RULE: &str = "expected `[a-z][a-z0-9-]*`";

/// The largest grid coordinate the grammar admits. Coordinates are small
/// unsigned integers read through checked conversion, so an oversized literal
/// is a grammar refusal rather than a coordinate that wraps into a legal one.
const MAX_GEOMETRY_COORDINATE: u32 = 999;

/// Parse budget: the most elements one `profile/v2` `set`, `clear`, or `swap`
/// list may hold. The cap mirrors `MAX_MANIFEST_ROWS`, which bounds one game's
/// whole action table, so one patch can retouch every row of the manifest it
/// is pinned to and no more (design 014, A17).
pub const MAX_PATCH_DELTA_ELEMENTS: usize = 256;

/// Parse budget: the longest `generated-by` label, in octets. The cap is what
/// bounds a refusal that has to name the label (A13); the label is provenance,
/// so it is short and displayed, never a payload.
pub const MAX_GENERATED_BY_OCTETS: usize = 128;

/// The octet bound a v2 profile `id`, each `derives` element, and each pin's
/// manifest half share with the slug grammar — the DNS-label bound RFC 1035
/// §2.3.1 publishes, the same one the action-id and manifest grammars borrow,
/// so no unbounded string rides a displayed refusal.
const MAX_SLUG_OCTETS: usize = 63;

/// The slug grammar those ids ride, named in the refusal so a document says
/// which grammar it broke — the same spelling `placement::is_slug_name` checks.
const SLUG_RULE: &str = "expected `[a-z][a-z0-9-]*`";

/// The majors `bindforge-core` declares supported: the union over the formats,
/// and exactly the majors present under `schemas/*/v*/`. A major outside this
/// list is refused naming the whole union; a major inside it is still refused
/// when the declaring document's own format does not ship it — see
/// [`format_majors`].
pub fn supported_majors() -> Vec<String> {
    vec![MAJOR_V1.to_string(), MAJOR_V2.to_string()]
}

/// The majors one format ships, which is what the version gate actually checks.
/// `reach-classes` grew a second major in ADR-012 and `profile` grew one in
/// design 014; the list is where a future `manifest`/`patch`/`policy` major
/// gets declared when its directory is added: a format's majors widen only with
/// its own directory under `schemas/`, never because another format grew one.
/// `None` says the format is not one this crate parses — that is the
/// `UnknownFormat` refusal in `parse`, whatever its version line says.
pub(crate) fn format_majors(format: &str) -> Option<&'static [&'static str]> {
    match format {
        "action-ids" | "keyboard-geometry" | "manifests" => Some(&[MAJOR_V1]),
        "profile" | "reach-classes" => Some(&[MAJOR_V1, MAJOR_V2]),
        _ => None,
    }
}

/// The per-format half of the version gate: a document declaring a major its
/// own format does not ship is refused naming that format's majors. Reading
/// such a document with another major's grammar is the fail-open case the
/// contract forbids (`schemas/README.md`, versioning rule).
fn check_format_major(format: &str, version: &str) -> Result<(), CoreError> {
    let Some(majors) = format_majors(format) else {
        return Ok(());
    };
    if majors.contains(&version) {
        return Ok(());
    }
    Err(CoreError::UnsupportedVersion {
        declared: Some(version.to_string()),
        supported: majors.iter().map(|major| major.to_string()).collect(),
    })
}

/// What one `profile/v2` link is: a base stands alone at the bottom of a
/// chain, a patch modifies the one link below it. The label is required at v2
/// — never inferred from position — because it is the label the contract names,
/// and it is checked against `derives` at parse and against chain position at
/// load (design 014, Options: what `kind` means).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileKind {
    Base,
    Patch,
}

impl ProfileKind {
    /// The contract spelling of this kind: the value of a link's `kind` entry.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ProfileKind::Base => "base",
            ProfileKind::Patch => "patch",
        }
    }

    /// The kind one of those spellings names, or `None` when it names neither —
    /// the closed pair `spec.md` states. A third label is a refusal, never a
    /// kind whose position gets guessed.
    #[must_use]
    pub fn from_name(name: &str) -> Option<ProfileKind> {
        match name {
            "base" => Some(ProfileKind::Base),
            "patch" => Some(ProfileKind::Patch),
            _ => None,
        }
    }
}

/// A profile document. A `v1` document carries the identity and the inline
/// ancestor chain the shipped guards read (`id`, `derives`) and nothing else —
/// its v2 fields sit at their defaults and its `serialize` output stays
/// byte-exact (A15). A `v2` document is one link of a chain: the `kind` label,
/// the `for` pin naming the manifest the link applies to, the delta lists that
/// write the layout, and the two provenance labels nothing consumes as trust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub version: String,
    pub id: String,
    pub derives: Vec<String>,
    /// `v2`: required, so always `Some`; `v1`: `None`.
    pub kind: Option<ProfileKind>,
    /// `v2`: `"<manifest-id>@<revision>"`, grammar-checked at parse and joined
    /// to the manifest at load; `v1`: empty.
    pub for_manifest: String,
    /// Each element binds that action to that input, replacing any binding.
    pub set: Vec<(ActionId, InputId)>,
    /// Each action's binding is dropped.
    pub clear: Vec<ActionId>,
    /// Each pair of actions exchanges its current binding.
    pub swap: Vec<(ActionId, ActionId)>,
    /// Provenance: the preset the chain's base was seeded from. Validated
    /// referentially at load, consumed by nothing.
    pub seeded_from: Option<String>,
    /// Provenance: what produced this document. Never consumed as trust.
    pub generated_by: Option<String>,
}

/// A `reach-classes` table document: one hand's class list, and the grade that
/// hand gives each graded input. `keys` is the `v2` entry; a `v1` document
/// parses with it empty and its class ids stay what ADR-012 calls placeholders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReachClassTable {
    pub version: String,
    pub hand: Hand,
    pub classes: Vec<ReachClass>,
    pub keys: Vec<InputGrade>,
}

/// One `keyboard-geometry/v1` grid entry: an input id and the column and row
/// it sits at, counted from the row's left and from the digit row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometryKey {
    pub input: InputId,
    pub column: u32,
    pub row: u32,
}

/// A `keyboard-geometry/v1` document: one keyboard layout's key grid, as table
/// data, so another geometry is another file in the same directory under the
/// same entry set — new table files, not new logic (ADR-012, Consequences).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardGeometryTable {
    pub version: String,
    pub geometry: String,
    pub keys: Vec<GeometryKey>,
}

/// One contract document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    Profile(Profile),
    ReachClasses(ReachClassTable),
    ActionIds(ActionRegistry),
    KeyboardGeometry(KeyboardGeometryTable),
    Manifest(Manifest),
}

impl Document {
    /// The profile half of the document, when it is a profile — the handle
    /// `resolve_chain` guards (Chosen design -> Adversarial guard).
    pub fn profile(&self) -> Option<&Profile> {
        let Document::Profile(profile) = self else {
            return None;
        };
        Some(profile)
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
    // A major the crate declares is not automatically a major this document's
    // format ships: the gate is per-format, so a `profile` or `action-ids`
    // document declaring `v2` is refused rather than read with its `v1` grammar.
    check_format_major(format.as_str(), &version)?;

    match format.as_str() {
        "profile" => {
            let defined = profile_entries(&version);
            reject_unknown_entry(&entries, "profile", &version, defined)?;
            Ok(Document::Profile(parse_profile(&entries, version)?))
        }
        "reach-classes" => {
            let defined = reach_classes_entries(&version);
            reject_unknown_entry(&entries, "reach-classes", &version, defined)?;
            let table = parse_reach_classes(&entries, version)?;
            Ok(Document::ReachClasses(table))
        }
        "action-ids" => {
            reject_unknown_entry(&entries, "action-ids", &version, ACTION_IDS_ENTRIES)?;
            let registry = registry_from_entries(&entries, version)?;
            Ok(Document::ActionIds(registry))
        }
        "keyboard-geometry" => {
            reject_unknown_entry(&entries, "keyboard-geometry", &version, GEOMETRY_ENTRIES)?;
            let geometry = parse_keyboard_geometry(&entries, version)?;
            Ok(Document::KeyboardGeometry(geometry))
        }
        "manifests" => {
            reject_unknown_entry(&entries, "manifests", &version, MANIFEST_ENTRIES)?;
            let manifest = manifest_from_entries(&entries, version)?;
            Ok(Document::Manifest(manifest))
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

/// Which closed entry set a `profile` document is held to: `v2` admits the link
/// vocabulary, `v1` never does, and the declared version is what picks — the
/// same version-keyed mechanism `reach_classes_entries` ships (A4). The
/// per-format version gate has already refused anything outside `profile`'s two
/// majors, so one of these two sets always applies.
fn profile_entries(version: &str) -> &'static [&'static str] {
    if version == MAJOR_V2 {
        return PROFILE_V2_ENTRIES;
    }
    PROFILE_V1_ENTRIES
}

/// Which closed entry set a `reach-classes` document is held to: `v2` admits
/// `keys`, `v1` never does, and the declared version is what picks. The
/// per-format version gate has already refused anything outside this format's
/// two majors, so one of these two sets always applies.
fn reach_classes_entries(version: &str) -> &'static [&'static str] {
    if version == MAJOR_V2 {
        return REACH_CLASSES_V2_ENTRIES;
    }
    REACH_CLASSES_V1_ENTRIES
}

/// Refuses an entry outside the format's entry set. The check runs after the
/// required entries are read, so a document missing `version` still gets the
/// version gate's error, and a document carrying an entry the grammar does not
/// define never reaches the model with it. The refusal names the *declared*
/// version rather than a hard-coded `v1`, because the same format now has more
/// than one closed set — and the echoed version takes the same
/// escape-and-truncate path as an echoed id does.
fn reject_unknown_entry(
    entries: &BTreeMap<String, Value>,
    format: &str,
    version: &str,
    defined: &[&str],
) -> Result<(), CoreError> {
    for key in entries.keys() {
        if !defined.contains(&key.as_str()) {
            return Err(CoreError::InvalidValue {
                key: key.clone(),
                message: format!("not part of the {format}/{version} grammar"),
            });
        }
    }
    Ok(())
}

/// Write a document back out as contract text. Re-parsing the output yields
/// a semantically equal document — the round-trip half of Test strategy #1 —
/// for any model this crate produced by parsing one; `serialize_profile_link`
/// states the one precondition a hand-built `profile/v2` model has to meet.
pub fn serialize(document: &Document) -> String {
    match document {
        Document::Profile(profile) => {
            // The writer follows the grammar, not the model: the declared
            // version alone decides which entries exist. A v1 document keeps
            // its shipped bytes (A15), and a hand-built v1 model carrying v2
            // fields in it never prints a document the v1 reader refuses — the
            // same shape `reach-classes` grades decide on.
            if profile.version == MAJOR_V1 {
                let derives = profile
                    .derives
                    .iter()
                    .map(|ancestor| quoted(ancestor))
                    .collect::<Vec<_>>()
                    .join(", ");
                return format!(
                    "format = \"profile\"\nversion = {}\nid = {}\nderives = [{derives}]\n",
                    quoted(&profile.version),
                    quoted(&profile.id)
                );
            }
            serialize_profile_link(profile)
        }
        Document::ReachClasses(table) => {
            let classes = table
                .classes
                .iter()
                .map(|class| quoted(&class.id))
                .collect::<Vec<_>>()
                .join(", ");
            let keys = table
                .keys
                .iter()
                .map(|entry| quoted(&input_grade_pair(entry)))
                .collect::<Vec<_>>()
                .join(", ");
            let head = format!(
                "format = \"reach-classes\"\nversion = {}\nhand = {}\nclasses = [{classes}]\n",
                quoted(&table.version),
                quoted(table.hand.name())
            );
            // The writer follows the grammar, not the model: only a `v2`
            // document has a `keys` entry, so the version alone decides. A v1
            // model that a caller hand-built with grades in it must not print a
            // document the v1 reader refuses (`keys-entry.toml` pins that
            // refusal), and a v2 document always carries its grades.
            if table.version == MAJOR_V1 {
                head
            } else {
                format!("{head}keys = [{keys}]\n")
            }
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
        Document::KeyboardGeometry(table) => {
            let keys = table
                .keys
                .iter()
                .map(|key| quoted(&geometry_pair(key)))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "format = \"keyboard-geometry\"\nversion = {}\ngeometry = {}\nkeys = [{keys}]\n",
                quoted(&table.version),
                quoted(&table.geometry)
            )
        }
        Document::Manifest(manifest) => {
            // The entries go out in the fixed order the entry table defines
            // them, and every array keeps its document order: row, group, and
            // member order is data, so the harness's re-parse comparison is
            // equality, not luck. `quoted` writes the native names back
            // through the same closed escape table the reader decodes.
            let actions = manifest
                .actions
                .iter()
                .map(|row| quoted(&manifest_row(row)))
                .collect::<Vec<_>>()
                .join(", ");
            let groups = manifest
                .exclusive_groups
                .iter()
                .map(|group| quoted(&group_members(group)))
                .collect::<Vec<_>>()
                .join(", ");
            let dead_keys = manifest
                .dead_keys
                .iter()
                .map(|id| quoted(id.as_str()))
                .collect::<Vec<_>>()
                .join(", ");
            let docked = manifest
                .docked_actions
                .iter()
                .map(|row| quoted(&docked_row(row)))
                .collect::<Vec<_>>()
                .join(", ");
            let presets = manifest
                .presets
                .iter()
                .map(|id| quoted(id))
                .collect::<Vec<_>>()
                .join(", ");
            let mut lines = vec![
                "format = \"manifests\"".to_string(),
                format!("version = {}", quoted(&manifest.version)),
                format!("id = {}", quoted(&manifest.id)),
                format!("revision = {}", manifest.revision),
                format!("codec = {}", quoted(&manifest.codec)),
            ];
            // An absent fingerprint stays absent: it means never read against
            // a config, and writing one would invent a provenance claim.
            if let Some(fingerprint) = &manifest.fingerprint {
                lines.push(format!("fingerprint = {}", quoted(fingerprint)));
            }
            lines.extend([
                format!("actions = [{actions}]"),
                format!("exclusive-groups = [{groups}]"),
                format!("dead-keys = [{dead_keys}]"),
                format!("docked-actions = [{docked}]"),
                format!("presets = [{presets}]"),
            ]);
            let mut text = String::new();
            for line in lines {
                text.push_str(&line);
                text.push('\n');
            }
            text
        }
    }
}

/// One `profile/v2` link back out as contract text, in the fixed entry order
/// the entry table defines: `format, version, id, derives, kind, for, set,
/// clear, swap` then the two optional provenance labels. Every delta array is
/// written for every v2 document — an absent one round-trips as `[]`, the 013
/// precedent — and each array keeps its **document order**, because delta order
/// is data and the harness's re-parse comparison is equality (A3).
///
/// `kind` and `for` are written for every v2 document, unconditionally: this
/// writer branches on the declared `version` alone, never on which fields happen
/// to be filled in. So a v2 model is required to carry a `kind`; one hand-built
/// without it writes `kind = ""`, which the reader refuses naming the entry —
/// never a document that quietly lacks a required entry and re-parses as a
/// different shape. Every model this crate produces (a parse, a `fork`, `diff`,
/// or `merge` output) carries one.
fn serialize_profile_link(profile: &Profile) -> String {
    let derives = profile
        .derives
        .iter()
        .map(|ancestor| quoted(ancestor))
        .collect::<Vec<_>>()
        .join(", ");
    let kind = profile.kind.map_or("", ProfileKind::name);
    let mut lines = vec![
        "format = \"profile\"".to_string(),
        format!("version = {}", quoted(&profile.version)),
        format!("id = {}", quoted(&profile.id)),
        format!("derives = [{derives}]"),
        format!("kind = {}", quoted(kind)),
        format!("for = {}", quoted(&profile.for_manifest)),
    ];
    let set = profile
        .set
        .iter()
        .map(|(action, input)| quoted(&set_pair(action, input)))
        .collect::<Vec<_>>()
        .join(", ");
    let clear = profile
        .clear
        .iter()
        .map(|action| quoted(action.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    let swap = profile
        .swap
        .iter()
        .map(|(first, second)| quoted(&swap_pair(first, second)))
        .collect::<Vec<_>>()
        .join(", ");
    lines.extend([
        format!("set = [{set}]"),
        format!("clear = [{clear}]"),
        format!("swap = [{swap}]"),
    ]);
    // An absent provenance label stays absent: writing one would invent a claim
    // about what produced the document.
    if let Some(seeded_from) = &profile.seeded_from {
        lines.push(format!("seeded-from = {}", quoted(seeded_from)));
    }
    if let Some(generated_by) = &profile.generated_by {
        lines.push(format!("generated-by = {}", quoted(generated_by)));
    }
    let mut text = String::new();
    for line in lines {
        text.push_str(&line);
        text.push('\n');
    }
    text
}

/// One `set` element back into the grammar's pair encoding: the action id and
/// the input id, one space between. Neither half ever carries a space, which is
/// what makes the reader's one split point the same point the writer joins at.
fn set_pair(action: &ActionId, input: &InputId) -> String {
    format!("{} {}", action.as_str(), input.as_str())
}

/// One `swap` element back into the same pair encoding, both halves action ids.
fn swap_pair(first: &ActionId, second: &ActionId) -> String {
    format!("{} {}", first.as_str(), second.as_str())
}

/// One graded input back into the grammar's pair encoding: the id and the
/// class id, one space between. `quoted` escapes `\`, so the backslash glyph
/// round-trips through the reader's closed escape table.
fn input_grade_pair(entry: &InputGrade) -> String {
    format!("{} {}", entry.input.as_str(), entry.class.id)
}

/// One grid entry back into the grammar's triple encoding: the id, the column,
/// and the row, one space between.
fn geometry_pair(key: &GeometryKey) -> String {
    format!("{} {} {}", key.input.as_str(), key.column, key.row)
}

/// One manifest row back into the grammar's two-field encoding: the action id
/// and the native name, split at the first space going in and out. `quoted`
/// escapes what the reader's closed escape table decodes, so a name carrying
/// `#`, `"`, or `\` round-trips byte-exactly.
fn manifest_row(row: &crate::manifest::ManifestAction) -> String {
    format!("{} {}", row.action.as_str(), row.native_name)
}

/// One exclusivity group back into its element: the members, space-separated,
/// in the order the document wrote them.
fn group_members(group: &crate::manifest::ExclusivityGroup) -> String {
    group
        .inputs
        .iter()
        .map(|id| id.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// One docked row back into its element: the action id and the input id.
fn docked_row(row: &crate::manifest::DockedAction) -> String {
    format!("{} {}", row.action.as_str(), row.input.as_str())
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
    if version == MAJOR_V2 {
        return parse_profile_link(entries, version);
    }
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
    // A v1 document carries nothing beyond the identity and the inline ancestor
    // list: the v2 fields sit at their defaults, and `serialize` never writes a
    // v2 entry into a v1 document (A15).
    Ok(Profile {
        version,
        id,
        derives,
        kind: None,
        for_manifest: String::new(),
        set: Vec::new(),
        clear: Vec::new(),
        swap: Vec::new(),
        seeded_from: None,
        generated_by: None,
    })
}

/// One `profile/v2` link: the `kind` label, the `for` pin, the three delta
/// lists split into typed ids, and the two provenance labels. Every rule here
/// is one a single document can answer on its own — which is what keeps the
/// `profile/v2` corpus single-document (A16) and leaves only the chain-level
/// rules (position, the joins, the resolved-chain guards) to `chain_ops`.
fn parse_profile_link(
    entries: &BTreeMap<String, Value>,
    version: String,
) -> Result<Profile, CoreError> {
    let id = string_entry(entries, "id")?;
    check_profile_slug("id", &id)?;

    let kind = parse_profile_kind(entries)?;

    let for_manifest = string_entry(entries, "for")?;
    check_pin(&for_manifest)?;

    let derives = match entries.get("derives") {
        None => Vec::new(),
        Some(value) => string_array("derives", value)?,
    };
    // Every element is a chain lookup key and a file name, so the slug grammar
    // runs before any path is ever built from it: `..` and `a/b` refuse here,
    // as `placement::is_slug_name` already refuses them for a geometry name.
    for element in &derives {
        check_profile_slug("derives", element)?;
    }

    let set = parse_set(entries)?;
    let clear = parse_clear(entries)?;
    let swap = parse_swap(entries)?;
    check_link_shape(kind, &derives, &set, &clear, &swap)?;

    let seeded_from = match entries.get("seeded-from") {
        None => None,
        Some(value) => {
            let preset = string_value("seeded-from", value)?;
            check_profile_slug("seeded-from", &preset)?;
            Some(preset)
        }
    };
    let generated_by = match entries.get("generated-by") {
        None => None,
        Some(value) => {
            let label = string_value("generated-by", value)?;
            check_generated_by(&label)?;
            Some(label)
        }
    };

    Ok(Profile {
        version,
        id,
        derives,
        kind: Some(kind),
        for_manifest,
        set,
        clear,
        swap,
        seeded_from,
        generated_by,
    })
}

/// The `kind` entry: required, and one of the closed pair. Absent is
/// `MissingEntry` naming the entry — never a kind guessed from `derives`, which
/// is the label the position rules at load are checked against.
fn parse_profile_kind(entries: &BTreeMap<String, Value>) -> Result<ProfileKind, CoreError> {
    let name = string_entry(entries, "kind")?;
    match ProfileKind::from_name(&name) {
        Some(kind) => Ok(kind),
        None => Err(CoreError::InvalidValue {
            key: "kind".to_string(),
            message: format!("`{name}` is not a profile kind: expected `base` or `patch`"),
        }),
    }
}

/// The base/patch shape, checked against `derives` and the delta lists: a base
/// stands alone with nothing below it to drop or exchange, and a patch names
/// exactly one link below it and carries at least one element. This is the half
/// of the `kind`/`derives` contradiction the contract refuses per document; the
/// half that needs the chain (`kind` against chain position) is `chain_ops`'
/// step 4.
fn check_link_shape(
    kind: ProfileKind,
    derives: &[String],
    set: &[(ActionId, InputId)],
    clear: &[ActionId],
    swap: &[(ActionId, ActionId)],
) -> Result<(), CoreError> {
    let shape_fault = |key: &str, message: String| CoreError::InvalidValue {
        key: key.to_string(),
        message,
    };
    if kind == ProfileKind::Base {
        if let Some(first) = derives.first() {
            let message =
                format!("`{first}` names a link below a base: a base derives from nothing");
            return Err(shape_fault("derives", message));
        }
        if let Some(first) = clear.first() {
            let action = first.as_str();
            let message = format!(
                "`{action}` is a `clear` element on a base: a base has nothing below it to drop"
            );
            return Err(shape_fault("clear", message));
        }
        if let Some((first, second)) = swap.first() {
            let pair = format!("{} {}", first.as_str(), second.as_str());
            let message = format!(
                "`{pair}` is a `swap` element on a base: a base has nothing below it to exchange"
            );
            return Err(shape_fault("swap", message));
        }
        return Ok(());
    }
    if derives.is_empty() {
        let message = "a patch names the one link below it in `derives`".to_string();
        return Err(shape_fault("derives", message));
    }
    if let Some(second) = derives.get(1) {
        let message =
            format!("`{second}` is a second ancestor: a v2 chain is one path, not a tree");
        return Err(shape_fault("derives", message));
    }
    if set.is_empty() && clear.is_empty() && swap.is_empty() {
        let message =
            "a patch carries at least one element across `set`, `clear`, `swap`".to_string();
        return Err(shape_fault("set", message));
    }
    Ok(())
}

/// One of the three delta lists: absent means empty, and each list is capped by
/// `MAX_PATCH_DELTA_ELEMENTS` — the budget, not the shape, is what bounds a
/// hostile patch driving the load-time join over an unbounded action set.
fn delta_elements(key: &str, entries: &BTreeMap<String, Value>) -> Result<Vec<String>, CoreError> {
    let Some(value) = entries.get(key) else {
        return Ok(Vec::new());
    };
    let raws = string_array(key, value)?;
    if raws.len() > MAX_PATCH_DELTA_ELEMENTS {
        let message = format!(
            "more than {MAX_PATCH_DELTA_ELEMENTS} elements: the patch delta budget for `{key}`"
        );
        return Err(CoreError::InvalidValue {
            key: key.to_string(),
            message,
        });
    }
    Ok(raws)
}

/// The `set` list: each element is exactly `"<action-id> <input-id>"`, split at
/// the first space, both halves grammar-validated here — an input outside the
/// closed vocabulary refuses at parse, the same 012 gap it already pinned.
fn parse_set(entries: &BTreeMap<String, Value>) -> Result<Vec<(ActionId, InputId)>, CoreError> {
    let mut set = Vec::new();
    for raw in delta_elements("set", entries)? {
        let (action_text, input_text) = pair_fields("set", &raw)?;
        let action =
            ActionId::parse(action_text).map_err(|err| name_the_element(err, "set", &raw))?;
        let input = InputId::parse(input_text).map_err(|err| name_the_element(err, "set", &raw))?;
        set.push((action, input));
    }
    Ok(set)
}

/// The `clear` list: each element is one action id, and a second field is a
/// malformed delta, not a pair with an implied meaning.
fn parse_clear(entries: &BTreeMap<String, Value>) -> Result<Vec<ActionId>, CoreError> {
    let mut clear = Vec::new();
    for raw in delta_elements("clear", entries)? {
        if raw.contains(' ') {
            let reason = "expected one action id".to_string();
            return Err(delta_fault("clear", &raw, reason));
        }
        let action = ActionId::parse(&raw).map_err(|err| name_the_element(err, "clear", &raw))?;
        clear.push(action);
    }
    Ok(clear)
}

/// The `swap` list: each element is two space-joined action ids, and the two
/// are distinct — exchanging an action with itself is a delta that means
/// nothing, not a legal no-op.
fn parse_swap(entries: &BTreeMap<String, Value>) -> Result<Vec<(ActionId, ActionId)>, CoreError> {
    let mut swap = Vec::new();
    for raw in delta_elements("swap", entries)? {
        let (first_text, second_text) = pair_fields("swap", &raw)?;
        let first =
            ActionId::parse(first_text).map_err(|err| name_the_element(err, "swap", &raw))?;
        let second =
            ActionId::parse(second_text).map_err(|err| name_the_element(err, "swap", &raw))?;
        if first == second {
            let reason = "a `swap` pair names two distinct actions".to_string();
            return Err(delta_fault("swap", &raw, reason));
        }
        swap.push((first, second));
    }
    Ok(swap)
}

/// The shipped pair encoding (A8): exactly two fields split at the first space.
/// No input or action id carries a space, which is what makes that one split
/// point unambiguous; one field or three names the element and refuses.
fn pair_fields<'a>(key: &str, raw: &'a str) -> Result<(&'a str, &'a str), CoreError> {
    let fields: Vec<&str> = raw.split(' ').collect();
    if fields.len() != 2 {
        let reason = "expected two space-joined ids".to_string();
        return Err(delta_fault(key, raw, reason));
    }
    Ok((fields[0], fields[1]))
}

/// The refusal for a malformed delta element: `InvalidValue` keyed by the list
/// that holds it and naming the whole element, so the reader sees which line of
/// the document to fix.
fn delta_fault(key: &str, raw: &str, reason: String) -> CoreError {
    CoreError::InvalidValue {
        key: key.to_string(),
        message: format!("`{raw}` is not a well-formed `{key}` element: {reason}"),
    }
}

/// Re-keys a refusal raised inside the id grammars so it names the delta
/// element and its list instead of a bare `id` — the same re-keying
/// `action::registry_from_entries` applies to a registry array.
fn name_the_element(err: CoreError, key: &str, raw: &str) -> CoreError {
    let reason = match err {
        CoreError::InvalidValue { message, .. } => message,
        other => other.to_string(),
    };
    delta_fault(key, raw, reason)
}

/// The `for` pin: `"<manifest-id>@<revision>"`, exactly design 013's canonical
/// identity — one `@`, a slug head, and a decimal revision of at least 1 written
/// the way `serialize` writes it. A pin that parses two ways would answer
/// "which manifest?" twice, and `revision 0` is the shipped refusal shape (A7).
fn check_pin(text: &str) -> Result<(), CoreError> {
    let fault = |reason: String| CoreError::InvalidValue {
        key: "for".to_string(),
        message: format!("`{text}` is not a manifest pin: {reason}"),
    };
    let Some((head, tail)) = text.split_once('@') else {
        return Err(fault(
            "the pin is `\"<manifest-id>@<revision>\"`".to_string(),
        ));
    };
    if tail.contains('@') {
        return Err(fault("the pin splits at its one `@`".to_string()));
    }
    if !is_slug_name(head) || head.len() > MAX_SLUG_OCTETS {
        return Err(fault(format!("`{head}` is not a manifest id: {SLUG_RULE}")));
    }
    let Ok(revision) = tail.parse::<i64>() else {
        let reason = format!("`{tail}` is not a decimal integer at least 1");
        return Err(fault(reason));
    };
    if revision < 1 || tail != revision.to_string() {
        let reason = format!("`{tail}` is not a data revision: revisions start at 1");
        return Err(fault(reason));
    }
    Ok(())
}

/// The id grammar a v2 profile `id`, each `derives` element, and each
/// `seeded-from` ride: the shipped slug grammar plus the DNS-label octet bound.
/// A v2 id is a chain lookup key and a file name, which is why v1's "not empty"
/// is tightened here and stays tightened nowhere else (A14, A15). `chain_ops`
/// holds a caller-supplied fresh id to the same rule, so one definition binds
/// every id a chain can carry.
pub(crate) fn check_profile_slug(key: &str, value: &str) -> Result<(), CoreError> {
    if !is_slug_name(value) {
        let message = format!("`{value}` is not a slug: {SLUG_RULE}");
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

/// The `generated-by` label: 1 to `MAX_GENERATED_BY_OCTETS` octets, and no
/// ASCII control, default-ignorable, or bidi-control character. The refusal
/// names the entry and the rule rather than echoing the label, because the label
/// is exactly the kind of document-controlled text a log line must not carry
/// (A13, Provenance labels).
fn check_generated_by(text: &str) -> Result<(), CoreError> {
    let fault = |reason: String| CoreError::InvalidValue {
        key: "generated-by".to_string(),
        message: format!("the `generated-by` label {reason}"),
    };
    if text.is_empty() {
        let reason = format!("is empty: it holds 1 to {MAX_GENERATED_BY_OCTETS} octets");
        return Err(fault(reason));
    }
    if text.len() > MAX_GENERATED_BY_OCTETS {
        let length = text.len();
        let reason = format!("is {length} octets, past the {MAX_GENERATED_BY_OCTETS} octet bound");
        return Err(fault(reason));
    }
    if !text.chars().all(is_label_char) {
        return Err(fault(
            "carries a control, default-ignorable, or bidi-control character".to_string(),
        ));
    }
    Ok(())
}

/// Which characters a provenance label may carry. The excluded set is the one
/// that can steer what a reader *sees* rather than what it stores: the C1 and
/// C0 controls, the bidi embedding/override/isolate controls, the zero-width
/// and joiner controls, soft hyphen, and the other default-ignorables — none of
/// which a tool, a policy name, or `human` ever needs.
fn is_label_char(ch: char) -> bool {
    !ch.is_control()
        && !matches!(
            ch,
            '\u{00ad}'
                | '\u{034f}'
                | '\u{061c}'
                | '\u{180e}'..='\u{180f}'
                | '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{2065}'
                | '\u{2066}'..='\u{206f}'
                | '\u{feff}'
                | '\u{fff9}'..='\u{fffb}'
        )
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
            .collect::<Vec<ReachClass>>(),
    };
    // A `v1` document parses with no grades: its class ids stay the placeholders
    // `v1` shipped, and its closed entry set never admits `keys`.
    let mut keys = Vec::new();
    if version == MAJOR_V2 {
        check_taxonomy(&classes)?;
        keys = parse_input_grades(entries, &classes)?;
    }
    Ok(ReachClassTable {
        version,
        hand,
        classes,
        keys,
    })
}

/// The canonical `classes` rule of `reach-classes/v2/spec.md`: exactly the five
/// taxonomy ids, once each, in severity order. A per-key grade number would
/// widen the pair encoding, and an order kept in Rust would put the taxonomy
/// partly in logic, so the array order is the scale and this is where it is
/// checked.
fn check_taxonomy(classes: &[ReachClass]) -> Result<(), CoreError> {
    let ids: Vec<&str> = classes.iter().map(|class| class.id.as_str()).collect();
    if ids == REACH_CLASSES_V2_TAXONOMY {
        return Ok(());
    }
    Err(CoreError::InvalidValue {
        key: "classes".to_string(),
        message: "a v2 classes entry is the five taxonomy ids, in order".to_string(),
    })
}

/// The `keys` entry of a v2 table: every element is exactly
/// `"<input-id> <class-id>"`, the id is a grammatically valid `keyboard:*` id,
/// the class is one this document's `classes` entry lists, and each id is
/// graded once. These are parse-level rules — the taxonomy's ordinality is one
/// rule per document — which is what lets a loader file grades without ever
/// comparing one table file against another.
fn parse_input_grades(
    entries: &BTreeMap<String, Value>,
    classes: &[ReachClass],
) -> Result<Vec<InputGrade>, CoreError> {
    let Some(value) = entries.get("keys") else {
        return Err(CoreError::MissingEntry {
            key: "keys".to_string(),
        });
    };
    let mut grades: Vec<InputGrade> = Vec::new();
    for raw in string_array("keys", value)? {
        let Some((id_text, class_id)) = raw.split_once(' ') else {
            return Err(pair_fault(&raw));
        };
        if class_id.contains(' ') {
            return Err(pair_fault(&raw));
        }
        let input = InputId::parse(id_text)?;
        if input.namespace() != InputNamespace::Keyboard {
            let device = input.namespace().name();
            let message = format!("`{raw}` is a {device} input: grades are `keyboard:*` only");
            return Err(CoreError::InvalidValue {
                key: "keys".to_string(),
                message,
            });
        }
        let Some(class) = classes.iter().find(|listed| listed.id == class_id) else {
            let message = format!("`{raw}` names `{class_id}`, a class `classes` does not list");
            return Err(CoreError::InvalidValue {
                key: "keys".to_string(),
                message,
            });
        };
        let repeated = grades.iter().any(|grade| grade.input == input);
        if repeated {
            let message = format!("`{id_text}` is graded twice by the `keys` entry");
            return Err(CoreError::InvalidValue {
                key: "keys".to_string(),
                message,
            });
        }
        grades.push(InputGrade {
            input,
            class: class.clone(),
        });
    }
    Ok(grades)
}

/// The refusal for a `keys` element that is not the two-field pair encoding.
/// Ids carry no space, which is what makes the encoding's one split point
/// unambiguous, and a three-field element is the fixture that pins it.
fn pair_fault(raw: &str) -> CoreError {
    CoreError::InvalidValue {
        key: "keys".to_string(),
        message: format!("`{raw}` is not an `<input-id> <class-id>` pair"),
    }
}

fn parse_keyboard_geometry(
    entries: &BTreeMap<String, Value>,
    version: String,
) -> Result<KeyboardGeometryTable, CoreError> {
    let geometry = string_entry(entries, "geometry")?;
    if !is_slug_name(&geometry) {
        return Err(CoreError::InvalidValue {
            key: "geometry".to_string(),
            message: format!("`{geometry}` is not a geometry name: {GEOMETRY_NAME_RULE}"),
        });
    }
    let Some(value) = entries.get("keys") else {
        return Err(CoreError::MissingEntry {
            key: "keys".to_string(),
        });
    };
    let mut keys: Vec<GeometryKey> = Vec::new();
    for raw in string_array("keys", value)? {
        let key = parse_coordinate(&raw)?;
        let repeated = keys.iter().any(|other| other.input == key.input);
        if repeated {
            let text = key.input.as_str();
            return Err(CoreError::InvalidValue {
                key: "keys".to_string(),
                message: format!("`{text}` is placed twice by the `keys` entry"),
            });
        }
        keys.push(key);
    }
    Ok(KeyboardGeometryTable {
        version,
        geometry,
        keys,
    })
}

/// One `"<input-id> <col> <row>"` triple: exactly three space-separated
/// fields, an id in the closed `keyboard:` set, and two bounded coordinates.
fn parse_coordinate(raw: &str) -> Result<GeometryKey, CoreError> {
    let fields: Vec<&str> = raw.split(' ').collect();
    if fields.len() != 3 {
        return Err(coordinate_fault(raw));
    }
    let input = InputId::parse(fields[0])?;
    if input.namespace() != InputNamespace::Keyboard {
        let device = input.namespace().name();
        let message = format!("`{raw}` is a {device} input: grids place `keyboard:*` only");
        return Err(CoreError::InvalidValue {
            key: "keys".to_string(),
            message,
        });
    }
    let column = coordinate_value(fields[1], raw)?;
    let row = coordinate_value(fields[2], raw)?;
    Ok(GeometryKey { input, column, row })
}

/// One coordinate literal, bounded by checked conversion. The grammar's
/// integers are 64-bit, so an oversized or negative literal is a refusal here
/// rather than a coordinate that wraps into a legal one.
fn coordinate_value(text: &str, raw: &str) -> Result<u32, CoreError> {
    let Ok(value) = text.parse::<i64>() else {
        return Err(coordinate_fault(raw));
    };
    let Ok(coordinate) = u32::try_from(value) else {
        return Err(coordinate_fault(raw));
    };
    if coordinate > MAX_GEOMETRY_COORDINATE {
        return Err(coordinate_fault(raw));
    }
    Ok(coordinate)
}

/// The refusal for a `keys` element that is not a well-formed triple.
fn coordinate_fault(raw: &str) -> CoreError {
    CoreError::InvalidValue {
        key: "keys".to_string(),
        message: format!("`{raw}` is not an `<input-id> <col> <row>` triple"),
    }
}

/// The load-time half of the version gate: a document sitting directly in a
/// `v<N>/` directory must declare that same major, so a `v2` table can
/// never answer from a `v1` directory (`load_action_registry` applies the same
/// rule to the registry, and a `v1` document in a `v1` directory passes it
/// trivially). A directory that is not named for a major — the stand-in
/// directories the conformance harness loads a corpus fixture in — has no
/// segment to compare against, and the declared major there is already what
/// the version gate checked when the document was parsed.
pub(crate) fn check_dir_version(
    dir: &Path,
    declared: &str,
    file_name: &str,
) -> Result<(), CoreError> {
    let Some(segment) = major_dir_segment(dir) else {
        return Ok(());
    };
    if declared == segment {
        return Ok(());
    }
    Err(CoreError::InvalidValue {
        key: "version".to_string(),
        message: format!("`{file_name}` declares `{declared}` in a `{segment}` directory"),
    })
}

/// The major-version segment a directory is named for — `v` then digits, taken
/// from its last path component — or `None` when the directory is not named for
/// a major. One definition of the contract's "a document sits directly in a
/// `vN/` directory" rule, so the reach and registry loaders cannot drift into
/// two spellings of it; each loader decides what `None` means for its own data.
pub(crate) fn major_dir_segment(dir: &Path) -> Option<&str> {
    let segment = dir.file_name().and_then(|name| name.to_str())?;
    let digits = segment.strip_prefix('v')?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some(segment)
}

pub(crate) fn string_entry(
    entries: &BTreeMap<String, Value>,
    key: &str,
) -> Result<String, CoreError> {
    match entries.get(key) {
        None => Err(CoreError::MissingEntry {
            key: key.to_string(),
        }),
        Some(value) => string_value(key, value),
    }
}

pub(crate) fn string_value(key: &str, value: &Value) -> Result<String, CoreError> {
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
