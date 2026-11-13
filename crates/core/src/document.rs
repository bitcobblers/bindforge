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
//! declaring `format = "profile", version = "v2"` would be read with the
//! `profile/v1` grammar, which is exactly the misreading the gate exists to
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
//! ADR-011 adds `action-ids/v1`, and ADR-012 adds `reach-classes/v2` — the
//! five-class taxonomy with each hand's grade for every graded input — and
//! `keyboard-geometry/v1` — the key grid the placement predicate in
//! `placement` checks its shape rule against. The manifest and profile-chain
//! formats arrive with the rest of the 012-015 series and the same harness
//! walk picks them up with no harness change.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use crate::action::{ActionRegistry, AxisKind, registry_from_entries};
use crate::error::CoreError;
use crate::input::{InputId, InputNamespace};
use crate::placement::is_geometry_name;
use crate::reach::{Hand, InputGrade, ReachClass};
use crate::toml::{self, Value};

/// The major versions of the contract this crate implements. Must stay equal
/// to the majors present under `schemas/*/v*/` (Test strategy #2).
const MAJOR_V1: &str = "v1";
const MAJOR_V2: &str = "v2";

/// The format names this crate parses.
const SUPPORTED_FORMATS: &[&str] = &[
    "action-ids",
    "keyboard-geometry",
    "profile",
    "reach-classes",
];

/// Every entry the `profile/v1` grammar defines, and the only ones it admits.
const PROFILE_ENTRIES: &[&str] = &["derives", "format", "id", "version"];

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

/// The majors `bindforge-core` declares supported: the union over the formats,
/// and exactly the majors present under `schemas/*/v*/`. A major outside this
/// list is refused naming the whole union; a major inside it is still refused
/// when the declaring document's own format does not ship it — see
/// [`format_majors`].
pub fn supported_majors() -> Vec<String> {
    vec![MAJOR_V1.to_string(), MAJOR_V2.to_string()]
}

/// The majors one format ships, which is what the version gate actually checks.
/// `reach-classes` is the only format with a second major so far, and the list
/// is where a future `manifest`/`patch`/`policy` major gets declared when its
/// directory is added: a format's majors widen only with its own directory
/// under `schemas/`, never because another format grew one. `None` says the
/// format is not one this crate parses — that is the `UnknownFormat` refusal in
/// `parse`, whatever its version line says.
fn format_majors(format: &str) -> Option<&'static [&'static str]> {
    match format {
        "action-ids" | "keyboard-geometry" | "profile" => Some(&[MAJOR_V1]),
        "reach-classes" => Some(&[MAJOR_V1, MAJOR_V2]),
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

/// A `profile/v1` document. This design carries the identity and the
/// inline ancestor chain the guards read (`id`, `derives`); ADR-014 owns
/// profile content beyond that.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub version: String,
    pub id: String,
    pub derives: Vec<String>,
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
            reject_unknown_entry(&entries, "profile", &version, PROFILE_ENTRIES)?;
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
    }
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
    if !is_geometry_name(&geometry) {
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
