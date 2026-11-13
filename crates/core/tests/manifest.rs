//! The game manifest format of ADR-013: `schemas/manifests/v1/` read through
//! `bindforge-core`'s `manifests` arm — the row encoding, the one-fact-per-entry
//! rules, the named budgets, the identity and revision rules, the fingerprint
//! grammar, and the two rules the corpus cannot carry (the registry join and the
//! location rule), which `load_manifest` applies.
//!
//! proves: design 013's Test strategy #1 (the row encoding), #2 (uniqueness and
//! disjointness), #3 (identity, revision, fingerprint, and this format's half of
//! the version gate), #4 (load-time validation and the budgets), and #5
//! (round-trip through `serialize`).
//!
//! The canonical statement of every rule here is the proposal (Chosen design →
//! The contract, The model, Load-time validation, and Identity, revision, and the
//! series pins), which `schemas/manifests/v1/spec.md` restates for third-party
//! readers, read against 01-research.md's Findings. Three decisions bound the
//! whole suite:
//!
//! - **The row encoding.** Each `actions` element is `"<action-id> <native-name>"`
//!   split at its *first* space: an action id never carries a space (the id
//!   grammar `schemas/action-ids/v1/spec.md` pins it), while a game-native name
//!   routinely does (A13), so the split point is unique and the name is the whole
//!   remainder. That name is the format's only free-text column, so it carries its
//!   own grammar — `MAX_NATIVE_NAME_OCTETS`, no ASCII control character, no leading
//!   or trailing space — and it round-trips byte-exactly, `#` and every glyph the
//!   closed escape table decodes included (A4, A19).
//! - **One fact per document.** An `actions` row names a taxonomy id and a native
//!   name and nothing else: axis kind is a property of the id that `registry.toml`
//!   already owns (A7), so no case below puts it in a document, and no case puts a
//!   preset's layout or a shipped binding in one either (ADR-016, ADR-017).
//! - **Parse level versus load level.** Every rule the corpus can express refuses
//!   at `parse` alone, which is what keeps `schemas/manifests/v1/fixtures/invalid/`
//!   registry-free (A3, A10) — so the row, uniqueness, budget, identity, revision
//!   and fingerprint cases here all assert a parse-level refusal. Only the two
//!   cross-file rules wait for `load_manifest`, where the registry and the game
//!   directory are in hand: the join over `actions` and `docked-actions`, and the
//!   `id`-equals-directory-name rule.
//!
//! The refusal shape is the 012 precedent the proposal keeps: no new `CoreError`
//! variant enters the contract, so every refusal asserted here is `InvalidValue`,
//! `MissingEntry`, or the version gate's structured error, in the single-line,
//! escaped, truncating, path-free message `schemas/README.md` promises. Naming
//! *what* was refused is itself part of the design ("refused naming the element or
//! entry", "refused, each naming the offender", "refused, naming the id"), so the
//! cases assert an offender appears in the message without pinning the sentence
//! around it.
//!
//! This suite is unit-level: pure functions and scratch directories, no network
//! and no live server (ADR-004, repo practice). The legs that read the contract
//! tree — the `schemas/manifests/v1/` corpus through the harness walk, the closed
//! entry set in its per-format loop, and this format's per-format version refusal
//! — are in `conformance.rs`, `grammar_closure.rs` and `version_gate.rs`, per the
//! delta inventory. These tests are written from the proposal while
//! `crates/core/src/manifest.rs` does not exist, so they fail until the design is
//! implemented; that failure is the point, and none of them may be weakened to
//! make anything pass.

mod common;

use std::path::PathBuf;

use bindforge_core::{ActionRegistry, CoreError, Document, InputId, Manifest};
use common::{ScratchDir, format_majors};

/// A manifest carrying exactly the eleven entries the entry table defines, each
/// with a fault-free value: the shape `basic-manifest.toml` ships, and the
/// document every case below edits exactly one entry of.
const VALID_MANIFEST: &str = r#"format = "manifests"
version = "v1"
id = "remnant-2"
revision = 3
codec = "unreal-engine-5"
fingerprint = "sha256-2Z7Gx9m0YU3bJ8kT4sQw1n7dC6vH0pL2xR5yB8aN3eM="
actions = ["move.steer-left Turn Left", "aim.camera Mouse Camera", "combat.fire Primary Attack"]
exclusive-groups = ["keyboard:w keyboard:s"]
dead-keys = ["keyboard:x"]
docked-actions = ["ui.menu keyboard:y"]
presets = ["default", "southpaw"]
"#;

/// The defined entries' array values, each already spelled in the grammar's own
/// shape, so a case edits one entry and leaves the other ten legal.
const ROWS: &str =
    r#"["move.steer-left Turn Left", "aim.camera Mouse Camera", "combat.fire Primary Attack"]"#;
const DEAD_KEYS: &str = r#"["keyboard:x"]"#;
const DOCKED: &str = r#"["ui.menu keyboard:y"]"#;
const PRESETS: &str = r#"["default", "southpaw"]"#;
const FINGERPRINT: &str = r#""sha256-2Z7Gx9m0YU3bJ8kT4sQw1n7dC6vH0pL2xR5yB8aN3eM=""#;

/// A 32-octet digest spelled in standard base64, the same one the proposal's
/// example manifest carries. It is provenance data: nothing here consumes it, and
/// what bytes it covers is ADR-017's (A15).
const DIGEST: &str = "2Z7Gx9m0YU3bJ8kT4sQw1n7dC6vH0pL2xR5yB8aN3eM=";

/// An `action-ids` document for a scratch registry: the four ids
/// [`VALID_MANIFEST`] names, one per axis array where the axis kind fits and two
/// in `none` (`schemas/action-ids/v1/spec.md`: the arrays *are* the id set, and
/// membership and axis kind are one fact). The join cases grow this list, which is
/// the proposal's registry growth order read from the loader's side.
const REGISTRY_DOC: &str = r#"format = "action-ids"
version = "v1"
discrete-lateral = ["move.steer-left"]
continuous-lateral = ["aim.camera"]
none = ["combat.fire", "ui.menu"]
"#;

/// A grammar-valid action id the shipped `registry.toml` does not list: exactly
/// two lowercase segments, so the refusal it produces is the load-time join's,
/// never the id grammar's.
const UNREGISTERED_ACTION: &str = "move.steer-up";

/// A native name of `len` octets, for the `MAX_NATIVE_NAME_OCTETS` boundary.
fn name_of(len: usize) -> String {
    "N".repeat(len)
}

/// The `entry = value` line this suite writes: `entry` names the entry and
/// `value` is already spelled in the grammar's shape — the same helper shape the
/// `action-ids` suite draws.
fn entry<'a>(name: &'a str, value: &str) -> (&'a str, String) {
    (name, format!("{name} = {value}"))
}

/// The manifest document made of these entries, one per line, in the order given:
/// document order, which is also the order `serialize` must write them and their
/// array elements back out in (The model → the `serialize` branch).
fn manifest_doc(entries: &[(&str, String)]) -> String {
    entries
        .iter()
        .map(|(_, line)| format!("{line}\n"))
        .collect()
}

/// The eleven defined entries, so a case starts from a document whose only fault
/// is the entry it edits.
fn complete_entries() -> Vec<(&'static str, String)> {
    vec![
        entry("format", "\"manifests\""),
        entry("version", "\"v1\""),
        entry("id", "\"remnant-2\""),
        entry("revision", "3"),
        entry("codec", "\"unreal-engine-5\""),
        entry("fingerprint", FINGERPRINT),
        entry("actions", ROWS),
        entry("exclusive-groups", &array(&["keyboard:w keyboard:s"])),
        entry("dead-keys", DEAD_KEYS),
        entry("docked-actions", DOCKED),
        entry("presets", PRESETS),
    ]
}

/// The defined eleven with `name` set to `value`: a defined entry is replaced and
/// one the entry table does not define is appended, so every value-level and
/// unknown-entry case is one fault exactly.
fn doc_with_entry(name: &str, value: &str) -> String {
    let mut entries: Vec<(&str, String)> = complete_entries();
    let mut defined = false;
    for (entry_name, line) in entries.iter_mut() {
        if *entry_name == name {
            *line = format!("{name} = {value}");
            defined = true;
        }
    }
    if !defined {
        entries.push((name, format!("{name} = {value}")));
    }
    manifest_doc(&entries)
}

/// The defined eleven with `name` dropped, for the required-entry cases.
fn doc_without_entry(name: &str) -> String {
    let entries: Vec<(&str, String)> = complete_entries()
        .into_iter()
        .filter(|(entry_name, _)| *entry_name != name)
        .collect();
    manifest_doc(&entries)
}

/// A one-line string array of these elements. The elements are written exactly as
/// the file writes them — the closed escape table's `\"` and `\\` included —
/// because this names what the contract file lists, it is not a second parser.
fn array(elements: &[&str]) -> String {
    let items = elements
        .iter()
        .map(|element| format!("\"{element}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}

/// The `actions` array with `count` legal rows, each naming a distinct
/// grammar-valid action id and a distinct native name: enough to sit on either
/// side of a row-count boundary without tripping the one-fact-per-entry rules.
fn legal_rows(count: usize) -> String {
    let elements: Vec<String> = (1..=count)
        .map(|index| format!("move.action-{index} Native name {index}"))
        .collect();
    let owned: Vec<&str> = elements.iter().map(String::as_str).collect();
    array(&owned)
}

/// The `presets` array with `count` distinct slug-grammar preset ids.
fn legal_presets(count: usize) -> String {
    let elements: Vec<String> = (1..=count).map(|index| format!("preset-{index}")).collect();
    let owned: Vec<&str> = elements.iter().map(String::as_str).collect();
    array(&owned)
}

/// A `dead-keys` array of `count` members drawn from the closed vocabulary and
/// repeated past its end. The vocabulary has 73 members (A8), so a case past the
/// row budget cannot name 257 distinct keys; it reaches either the budget's own
/// refusal or a duplicate's, and both are `InvalidValue`.
fn dead_keys(count: usize) -> String {
    let vocabulary = [
        "keyboard:w",
        "keyboard:a",
        "keyboard:s",
        "keyboard:d",
        "keyboard:x",
        "keyboard:c",
        "keyboard:v",
        "keyboard:b",
        "keyboard:n",
        "keyboard:m",
        "keyboard:space",
        "keyboard:1",
        "keyboard:2",
        "keyboard:3",
        "mouse:left",
        "mouse:right",
        "gamepad:north",
    ];
    let elements: Vec<&str> = (0..count)
        .map(|index| vocabulary[index % vocabulary.len()])
        .collect();
    array(&elements)
}

/// The `docked-actions` array with `count` rows, each naming a distinct
/// grammar-valid action id.
fn legal_docked(count: usize) -> String {
    let elements: Vec<String> = (1..=count)
        .map(|index| format!("ui.action-{index} keyboard:y"))
        .collect();
    let owned: Vec<&str> = elements.iter().map(String::as_str).collect();
    array(&owned)
}

/// The `exclusive-groups` array with `count` groups. Membership distinctness
/// cannot be maintained past the vocabulary's 73 members, so this only ever
/// crosses `MAX_CONSTRAINT_GROUPS` from above.
fn groups(count: usize) -> String {
    let elements: Vec<&str> = (0..count).map(|_| "keyboard:w keyboard:s").collect();
    array(&elements)
}

/// A one-element `exclusive-groups` array holding one group of these members.
fn one_group(members: &[&str]) -> String {
    array(&[members.join(" ").as_str()])
}

/// The first `count` keyboard letters as input ids — `MAX_GROUP_MEMBERS` distinct
/// members and the one past it.
fn letters(count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("keyboard:{}", char::from(b'a' + index as u8)))
        .collect()
}

/// Parses `text` and hands back its `Manifest`, failing the case if the arm is
/// not a manifest at all.
fn parse_manifest(text: &str) -> Manifest {
    let document = bindforge_core::parse(text)
        .unwrap_or_else(|err| panic!("a document the entry table defines must parse: {err}"));
    match document {
        Document::Manifest(manifest) => manifest,
        _ => panic!("a `format = \"manifests\"` document must parse into the manifest arm"),
    }
}

/// A parsed manifest's rows as `(action id, native name)` pairs, in document
/// order — the comparison every row-encoding and round-trip case reads.
fn rows_of(manifest: &Manifest) -> Vec<(String, String)> {
    manifest
        .actions
        .iter()
        .map(|row| (row.action.as_str().to_string(), row.native_name.clone()))
        .collect()
}

/// An input-id list as spellings, for the constraint comparisons.
fn spellings(ids: &[InputId]) -> Vec<String> {
    ids.iter().map(|id| id.as_str().to_string()).collect()
}

/// A parsed manifest's exclusivity groups as membership lists, in document order.
fn groups_of(manifest: &Manifest) -> Vec<Vec<String>> {
    manifest
        .exclusive_groups
        .iter()
        .map(|group| spellings(&group.inputs))
        .collect()
}

/// A parsed manifest's docked rows as `(action id, input id)` pairs.
fn docked_of(manifest: &Manifest) -> Vec<(String, String)> {
    manifest
        .docked_actions
        .iter()
        .map(|row| {
            (
                row.action.as_str().to_string(),
                row.input.as_str().to_string(),
            )
        })
        .collect()
}

/// The parse-level refusal the design promises for `text`: the document must not
/// parse, and the refusal must be the contract's value error — no new `CoreError`
/// variant enters with this format, and a value-level fault is `InvalidValue`.
fn parse_refusal(text: &str) -> CoreError {
    let Err(err) = bindforge_core::parse(text) else {
        panic!("this document is an invalid case and must be refused, not parsed: {text}");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "every value-level manifest refusal is `InvalidValue` (The model, error rule), got {err:?}"
    );
    err
}

/// The budget cases, where the design pins only that the refusal happens and as
/// which variant (Test strategy #4): what the message says about a cap is not
/// part of the contract, so nothing here reads it.
fn assert_parse_refused(text: &str) {
    drop(parse_refusal(text));
}

/// proves: Test strategy #1, #2, #3 — a refusal names what it refused: the
/// offending element or entry, in the one-line, control-free message the contract
/// promises.
fn assert_refusal_names(err: &CoreError, needles: &[&str]) {
    let detail = err.to_string();
    assert!(
        needles.iter().any(|needle| detail.contains(needle)),
        "the refusal must name the offender it refused, one of {needles:?}, got `{detail}`"
    );
    assert!(
        !detail.contains('\n') && !detail.contains('\r'),
        "a refusal is one bounded line: {detail:?}"
    );
    assert!(
        detail.chars().all(|ch| !ch.is_control()),
        "a refusal escapes control characters rather than emitting them: {detail:?}"
    );
}

/// Stands up a scratch game directory named `dir_name` holding `text` as
/// `manifest.toml`, and returns the scratch tree with the directory path — the
/// shape `load_manifest(game_dir, registry)` reads. The scratch tree outlives the
/// path: dropping it first would remove the directory out from under the loader.
fn game_dir(label: &str, dir_name: &str, text: &str) -> (ScratchDir, PathBuf) {
    let scratch = ScratchDir::new(label);
    scratch.write(&format!("{dir_name}/manifest.toml"), text);
    let path = scratch.path().join(dir_name);
    (scratch, path)
}

/// A scratch `action-ids/v1` registry built from `doc` — the join's other side.
fn registry(label: &str, doc: &str) -> ActionRegistry {
    let scratch = ScratchDir::new(label);
    scratch.write("v1/registry.toml", doc);
    bindforge_core::load_action_registry(&scratch.path().join("v1"))
        .unwrap_or_else(|err| panic!("a registry the grammar defines must load: {err}"))
}

/// proves: Test strategy #1 — the row encoding splits at the *first* space: the
/// head is the taxonomy id and the remainder is the native name, each returned
/// exactly as written, and the row carries nothing else — axis kind is answered
/// by the registry, never restated in the document (A7).
#[test]
fn parses_an_actions_element_into_an_action_id_and_a_native_name() {
    let manifest = parse_manifest(VALID_MANIFEST);

    assert_eq!(
        rows_of(&manifest),
        vec![
            ("move.steer-left".to_string(), "Turn Left".to_string()),
            ("aim.camera".to_string(), "Mouse Camera".to_string()),
            ("combat.fire".to_string(), "Primary Attack".to_string()),
        ],
        "each `actions` element is one `\"<action-id> <native-name>\"` row, split once"
    );
    assert_eq!(manifest.id, "remnant-2");
    assert_eq!(manifest.revision, 3);
    assert_eq!(manifest.codec, "unreal-engine-5");
    assert_eq!(
        manifest.fingerprint.as_deref(),
        Some(format!("sha256-{DIGEST}").as_str()),
        "a fingerprint is recorded exactly as written — provenance data, not a guard (A16)"
    );
}

/// proves: Test strategy #1 — a native name may hold spaces, because the split
/// point is the first space and an action id never carries one (A13). A name of
/// several words is the ordinary case, not an escape hatch.
#[test]
fn accepts_a_native_name_containing_spaces() {
    let text = doc_with_entry("actions", &array(&["aim.camera Mouse camera look left"]));

    let manifest = parse_manifest(&text);
    assert_eq!(
        rows_of(&manifest),
        vec![(
            "aim.camera".to_string(),
            "Mouse camera look left".to_string()
        )],
        "everything after the first space is the name"
    );
}

/// proves: Test strategy #1 + Chosen design (Row encoding) — the name keeps the
/// glyphs the grammar already decodes: a `#` inside a quoted array element is
/// data, never a comment (A19), and a `"` and a `\` ride the closed escape table
/// (A4). This is the case `spaced-native-names.toml` pins for third-party
/// readers.
#[test]
fn accepts_a_native_name_containing_a_hash_a_quote_and_a_backslash() {
    let text = doc_with_entry(
        "actions",
        &array(&[r#"aim.camera Say \"turn\" \\ left # look"#]),
    );

    let manifest = parse_manifest(&text);
    assert_eq!(
        rows_of(&manifest),
        vec![(
            "aim.camera".to_string(),
            "Say \"turn\" \\ left # look".to_string()
        )],
        "`#`, `\"` and `\\` inside a name are data, and the row still splits once"
    );
}

/// proves: Test strategy #1 — an element with no space at all is a row with no
/// name, refused naming the element or the entry: a reader that took the whole
/// element as an id would silently drop the game's own name for its action, which
/// is the name ADR-014's `seeded-from` and ADR-015's explanations depend on.
#[test]
fn refuses_an_actions_element_with_no_space_naming_the_offender() {
    let text = doc_with_entry("actions", &array(&["move.steer-left"]));

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["actions", "move.steer-left"]);
}

/// proves: Test strategy #1 — an empty remainder is refused: an action with no
/// native name is a document that answers "what does this game call it" with
/// nothing.
#[test]
fn refuses_an_actions_element_with_an_empty_native_name_naming_the_offender() {
    let text = doc_with_entry("actions", &array(&["move.steer-left "]));

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["actions", "move.steer-left"]);
}

/// proves: Test strategy #1 — a trailing space in a name is refused, because the
/// name must round-trip byte-exactly: two documents differing only by a trailing
/// space would be two different facts about one game's label.
#[test]
fn refuses_an_actions_element_with_a_trailing_space_in_the_name_naming_the_offender() {
    let text = doc_with_entry("actions", &array(&["aim.camera Mouse Camera "]));

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["actions", "aim.camera"]);
}

/// proves: Test strategy #1 — an ASCII control character in a name is refused
/// even where the escape table decodes it (`\u0007`), so a hostile manifest
/// cannot carry a control character into a displayed native name.
#[test]
fn refuses_an_actions_element_with_a_control_character_in_the_name_naming_the_offender() {
    let text = doc_with_entry("actions", &array(&[r"move.forward Bell\u0007Name"]));

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["actions", "move.forward"]);
    assert!(
        !err.to_string().contains('\u{7}'),
        "the refusal must escape the control character rather than emit it"
    );
}

/// proves: Test strategy #1 + Chosen design (Budgets) — a native name longer than
/// `MAX_NATIVE_NAME_OCTETS` is refused: the cap is what bounds a refusal that has
/// to echo the name, so one hostile row cannot inflate log volume (A14).
#[test]
fn refuses_an_actions_element_with_a_name_past_max_native_name_octets_naming_the_offender() {
    let element = format!(
        "aim.camera {}",
        name_of(bindforge_core::MAX_NATIVE_NAME_OCTETS + 1)
    );
    let text = doc_with_entry("actions", &array(&[element.as_str()]));

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["actions", "aim.camera"]);
}

/// proves: Chosen design (Budgets) — the cap is a limit, not an off-by-one wall: a
/// native name of exactly `MAX_NATIVE_NAME_OCTETS` parses, and comes back
/// unchanged.
#[test]
fn accepts_a_native_name_of_exactly_max_native_name_octets() {
    let name = name_of(bindforge_core::MAX_NATIVE_NAME_OCTETS);
    let element = format!("aim.camera {name}");
    let text = doc_with_entry("actions", &array(&[element.as_str()]));

    let manifest = parse_manifest(&text);
    assert_eq!(
        rows_of(&manifest),
        vec![("aim.camera".to_string(), name)],
        "a name at the cap is legitimate input, not a refusal"
    );
}

/// proves: Test strategy #1 — the element's head is grammar-checked as an action
/// id: an uppercase spelling, an underscore, a single segment and a third segment
/// are each refused, never repaired and never left for the load-time join to
/// catch. A row whose head is malformed cannot be joined to the registry at all.
#[test]
fn refuses_an_actions_element_with_a_malformed_action_id_in_its_head_naming_the_offender() {
    for head in ["Move.Left", "move_forward", "steer-left", "move.steer.left"] {
        let element = format!("{head} Turn Left");
        let text = doc_with_entry("actions", &array(&[element.as_str()]));

        let err = parse_refusal(&text);
        assert_refusal_names(&err, &["actions", head]);
    }
}

/// proves: Test strategy #1 + Chosen design (The contract) — closing the entry
/// set is not shrinking the grammar: a `basic-manifest.toml`-shaped document
/// using exactly the defined eleven entries parses.
#[test]
fn parses_a_manifest_carrying_exactly_the_defined_entries() {
    if let Err(err) = bindforge_core::parse(VALID_MANIFEST) {
        panic!(
            "a manifest using only the entries the entry table defines must parse; it was \
             refused: {err}"
        );
    }
}

/// proves: Chosen design (The contract, Required column) — the optional entries
/// really are optional: a manifest carrying only `format`, `version`, `id`,
/// `revision`, `codec` and `actions` parses, with no fingerprint and no
/// constraints. That is `minimal-manifest.toml`: a manifest never read against a
/// config has no fingerprint to record (A15), and a game with no docked UI has no
/// constraint to state.
#[test]
fn parses_a_manifest_carrying_only_the_required_entries() {
    let text = manifest_doc(&[
        entry("format", "\"manifests\""),
        entry("version", "\"v1\""),
        entry("id", "\"remnant-2\""),
        entry("revision", "1"),
        entry("codec", "\"unreal-engine-5\""),
        entry("actions", "[]"),
    ]);

    let manifest = parse_manifest(&text);
    assert_eq!(manifest.id, "remnant-2");
    assert_eq!(manifest.revision, 1);
    assert_eq!(manifest.fingerprint, None, "absent means never read");
    assert!(manifest.actions.is_empty());
    assert!(manifest.exclusive_groups.is_empty());
    assert!(manifest.dead_keys.is_empty());
    assert!(manifest.docked_actions.is_empty());
    assert!(manifest.presets.is_empty());
}

/// proves: Chosen design (The contract) — `actions` is required and may be empty:
/// the empty array is a game whose action table this manifest names no rows of,
/// not a missing entry.
#[test]
fn accepts_an_empty_actions_array() {
    let text = doc_with_entry("actions", "[]");

    let manifest = parse_manifest(&text);
    assert!(
        manifest.actions.is_empty(),
        "an empty `actions` array is legal and yields no rows"
    );
}

/// proves: Chosen design (The contract, Required column) — a required entry that
/// is absent is `MissingEntry` naming the entry, never defaulted: a manifest with
/// no `revision` pins nothing, and one with no `codec` names no family for
/// ADR-017 to read the game with.
#[test]
fn refuses_a_manifest_missing_a_required_entry_naming_the_entry() {
    for missing in ["id", "revision", "codec", "actions"] {
        let text = doc_without_entry(missing);

        let Err(err) = bindforge_core::parse(&text) else {
            panic!("a manifest with no `{missing}` entry must be refused");
        };
        assert!(
            matches!(err, CoreError::MissingEntry { .. }),
            "an absent required entry is `MissingEntry` (The model, error rule), got {err:?}"
        );
        let detail = err.to_string();
        assert!(
            detail.contains(missing),
            "the refusal must name the entry it found missing, got `{detail}`"
        );
    }
}

/// proves: Test strategy #2 + Chosen design (One fact per document) — each action
/// id appears once in `actions`: two rows under one id are two native names for
/// one action, the two-answers-to-one-fact drift the contract's closed sets exist
/// to refuse.
#[test]
fn refuses_a_repeated_action_id_in_actions_naming_the_offender() {
    let text = doc_with_entry(
        "actions",
        &array(&["move.steer-left Turn left", "move.steer-left Wheel left"]),
    );

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["move.steer-left"]);
}

/// proves: Test strategy #2 + Chosen design (One fact per document) — each native
/// name appears once: one name on two ids is a game answering one of its own
/// labels with two actions, and a manifest that claims it cannot be trusted for
/// its other facts.
#[test]
fn refuses_a_repeated_native_name_in_actions_naming_the_offender() {
    let text = doc_with_entry(
        "actions",
        &array(&["move.steer-left Turn Left", "move.steer-right Turn Left"]),
    );

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["Turn Left"]);
}

/// proves: Test strategy #2 — each dead key appears once: a repeated member
/// states the same game fact twice, and ADR-015's set-membership lookup would read
/// it as two facts.
#[test]
fn refuses_a_repeated_dead_key_naming_the_offender() {
    let text = doc_with_entry(
        "dead-keys",
        &array(&["keyboard:x", "keyboard:c", "keyboard:x"]),
    );

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["keyboard:x"]);
}

/// proves: Test strategy #2 — each docked action id appears once: two docked rows
/// for one action are two answers to "what does this game hard-dock this action
/// to", and the action side is what explains a refused transform target (A11).
#[test]
fn refuses_a_repeated_docked_action_id_naming_the_offender() {
    let text = doc_with_entry(
        "docked-actions",
        &array(&["ui.menu keyboard:y", "ui.menu keyboard:z"]),
    );

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["ui.menu"]);
}

/// proves: Test strategy #2 — each preset id appears once: a repeated id lists one
/// built-in twice, which ADR-014's `seeded-from` would join twice.
#[test]
fn refuses_a_repeated_preset_id_naming_the_offender() {
    let text = doc_with_entry("presets", &array(&["default", "southpaw", "default"]));

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["default"]);
}

/// proves: Test strategy #2 + Chosen design (One fact per document) — a group's
/// members are distinct: `keyboard:w keyboard:w` claims one key is exclusive with
/// itself, which is not a constraint but a document that means nothing.
#[test]
fn refuses_an_exclusivity_group_with_a_repeated_member_naming_the_offender() {
    let text = doc_with_entry(
        "exclusive-groups",
        &array(&["keyboard:w keyboard:w keyboard:s"]),
    );

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["keyboard:w"]);
}

/// proves: Test strategy #2 — an input id appears in at most one group: two groups
/// sharing a key make "is this key exclusive" answer two ways, and ADR-015's
/// refusal of a shared key as a transform target would depend on which group it
/// consulted.
#[test]
fn refuses_an_input_id_appearing_in_two_exclusivity_groups_naming_the_offender() {
    let text = doc_with_entry(
        "exclusive-groups",
        &array(&["keyboard:w keyboard:s", "keyboard:s keyboard:d"]),
    );

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["keyboard:s"]);
}

/// proves: Test strategy #2 + Chosen design (One fact per document) — `dead-keys`
/// and `docked-actions` are disjoint: a key the game both will not honour at all
/// and hard-docks to an action is two answers to one fact about one key — exactly
/// the disagreement the research flags as the drift channel.
#[test]
fn refuses_an_input_id_that_is_both_a_dead_key_and_a_docked_input_naming_the_offender() {
    let text = manifest_doc(&[
        entry("format", "\"manifests\""),
        entry("version", "\"v1\""),
        entry("id", "\"remnant-2\""),
        entry("revision", "3"),
        entry("codec", "\"unreal-engine-5\""),
        entry("actions", ROWS),
        entry("dead-keys", &array(&["keyboard:z"])),
        entry("docked-actions", &array(&["ui.menu keyboard:z"])),
    ]);

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["keyboard:z"]);
}

/// proves: Test strategy #2 + Chosen design (The contract, 2..=16 members) — a
/// group of one member is refused: exclusivity is a relation between at least two
/// keys, so a one-member group is a malformed claim, not an empty constraint.
#[test]
fn refuses_an_exclusivity_group_with_one_member_naming_the_offender() {
    let text = doc_with_entry("exclusive-groups", &one_group(&["keyboard:w"]));

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["exclusive-groups", "keyboard:w"]);
}

/// proves: Test strategy #2 + Chosen design (Budgets) — a group wider than
/// `MAX_GROUP_MEMBERS` is refused before any membership set is built: the bound is
/// what stops a hostile group allocating an unbounded set.
#[test]
fn refuses_an_exclusivity_group_past_max_group_members() {
    let members = letters(bindforge_core::MAX_GROUP_MEMBERS + 1);
    let owned: Vec<&str> = members.iter().map(String::as_str).collect();
    let text = doc_with_entry("exclusive-groups", &one_group(&owned));

    assert_parse_refused(&text);
}

/// proves: Chosen design (Budgets) — the bound is a limit, not an off-by-one wall:
/// a group of exactly `MAX_GROUP_MEMBERS` distinct members parses, with its
/// membership kept in order.
#[test]
fn accepts_an_exclusivity_group_of_exactly_max_group_members() {
    let members = letters(bindforge_core::MAX_GROUP_MEMBERS);
    let owned: Vec<&str> = members.iter().map(String::as_str).collect();
    let text = doc_with_entry("exclusive-groups", &one_group(&owned));

    let manifest = parse_manifest(&text);
    assert_eq!(
        groups_of(&manifest),
        vec![members],
        "the group's membership must survive parsing, in order"
    );
}

/// proves: Test strategy #3 + Chosen design (Identity grammar) — a manifest `id`
/// that breaks the slug grammar is refused: an uppercase spelling, a dot, a leading
/// hyphen, and one past the 63-octet label bound. The id is a path component and
/// the left half of every ADR-014 `for` pin, so a spelling that merely *looks*
/// lowercase is the collision the rule exists to prevent.
#[test]
fn refuses_a_manifest_id_that_breaks_the_slug_grammar_naming_the_offender() {
    let long_id = "a".repeat(64);

    for id in ["Remnant-2", "remnant.2", "-remnant", long_id.as_str()] {
        let text = doc_with_entry("id", &format!("\"{id}\""));

        let err = parse_refusal(&text);
        assert_refusal_names(&err, &["id", id]);
    }
}

/// proves: Chosen design (Identity grammar) — the bound is a limit: a slug `id` of
/// exactly 63 octets, the DNS-label bound RFC 1035 §2.3.1 publishes and the
/// action-id grammar already borrows, parses and is kept byte-exact.
#[test]
fn accepts_a_manifest_id_at_the_slug_length_bound() {
    let id = "g".repeat(63);
    let text = doc_with_entry("id", &format!("\"{id}\""));

    let manifest = parse_manifest(&text);
    assert_eq!(manifest.id, id, "the id is kept exactly as written");
}

/// proves: Test strategy #3 — `codec` rides the same slug grammar: the codec id
/// space is reserved here and implemented by ADR-017 (A15), so a malformed codec
/// id is refused at parse, while the document still has one answer for it.
#[test]
fn refuses_a_codec_that_breaks_the_slug_grammar_naming_the_offender() {
    let long_codec = "c".repeat(64);

    for codec in [
        "Unreal-Engine",
        "unreal.engine",
        "-unreal",
        long_codec.as_str(),
    ] {
        let text = doc_with_entry("codec", &format!("\"{codec}\""));

        let err = parse_refusal(&text);
        assert_refusal_names(&err, &["codec", codec]);
    }
}

/// proves: Test strategy #3 — each preset id rides the same slug grammar, because
/// a preset id is what ADR-014's `seeded-from` names: a malformed one is a pin
/// that can never resolve.
#[test]
fn refuses_a_preset_id_that_breaks_the_slug_grammar_naming_the_offender() {
    let long_preset = "p".repeat(64);

    for preset in ["Default", "south.paw", "-southpaw", long_preset.as_str()] {
        let text = doc_with_entry("presets", &array(&["default", preset]));

        let err = parse_refusal(&text);
        assert_refusal_names(&err, &["presets", preset]);
    }
}

/// proves: Test strategy #3 + Chosen design (Identity, revision, and the series
/// pins) — `revision` is a decimal integer at least 1: zero is refused, because a
/// pin of `"<id>@0"` names a revision no bump rule can reach; a negative revision
/// is refused; and a revision written as a string is refused, because the
/// grammar's integers are what make the pin comparable.
#[test]
fn refuses_a_revision_that_is_not_a_positive_integer_naming_the_offender() {
    for revision in ["0", "-1", "\"3\""] {
        let text = doc_with_entry("revision", revision);

        let err = parse_refusal(&text);
        assert_refusal_names(&err, &["revision"]);
    }
}

/// proves: Chosen design (Identity) — 1 is the first revision of a new game, and
/// it parses: the rule is `>= 1`, not `> 1`.
#[test]
fn accepts_a_revision_of_one() {
    let text = doc_with_entry("revision", "1");

    let manifest = parse_manifest(&text);
    assert_eq!(manifest.revision, 1);
}

/// proves: Test strategy #3 + Chosen design (Fingerprint grammar) — the algorithm
/// set is closed at `sha256-`: a `sha512-` body is refused, because algorithm
/// agility is a future format major and a reader that accepted both would record a
/// digest it cannot compare (A16).
#[test]
fn refuses_a_fingerprint_with_an_unknown_algorithm_prefix_naming_the_offender() {
    let text = doc_with_entry("fingerprint", &format!("\"sha512-{DIGEST}\""));

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["fingerprint"]);
}

/// proves: Test strategy #3 — the body must be standard base64: a body carrying a
/// character outside the alphabet is refused rather than decoded through a
/// permissive fallback, which would let two spellings claim one digest.
#[test]
fn refuses_a_fingerprint_whose_body_is_not_base64_naming_the_offender() {
    let text = doc_with_entry("fingerprint", "\"sha256-not!base64==\"");

    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["fingerprint"]);
}

/// proves: Test strategy #3 — the body must decode to exactly 32 octets: valid
/// base64 that decodes to 31 or to 16 is refused, since a digest of another length
/// is a different provenance claim wearing the same spelling.
#[test]
fn refuses_a_fingerprint_that_decodes_to_other_than_32_octets_naming_the_offender() {
    for body in [
        "AwsTGyMrMztDS1NbY2tze4OLk5ujq7O7w8vT2+Pr8w==",
        "AQIDBAUGBwgJCgsMDQ4PEA==",
    ] {
        let text = doc_with_entry("fingerprint", &format!("\"sha256-{body}\""));

        let err = parse_refusal(&text);
        assert_refusal_names(&err, &["fingerprint"]);
    }
}

/// proves: Test strategy #3 + A2 — the version gate is per-format: a `manifests`
/// document declaring `v2` is refused with the gate's structured error naming
/// *manifests*' majors, and `manifests` ships `v1` alone, so the refusal cannot
/// advertise a `v2` grammar that does not exist. `reach-classes`' `v2` never
/// widens what this format accepts, and the majors-equality invariant is
/// `version_gate.rs`'s to keep.
#[test]
fn refuses_a_manifest_declaring_a_major_the_manifests_format_does_not_ship() {
    let text = doc_with_entry("version", "\"v2\"");

    let Err(err) = bindforge_core::parse(&text) else {
        panic!("a manifest declaring `version = \"v2\"` must be refused, not parsed");
    };
    assert!(
        matches!(err, CoreError::UnsupportedVersion { .. }),
        "expected the version gate's structured error, got {err:?}"
    );

    let supported = format_majors("manifests");
    assert!(
        !supported.is_empty(),
        "the walk must find `schemas/manifests/v1/` to name this format's majors"
    );
    let message = err.to_string();
    for major in &supported {
        assert!(
            message.contains(major.as_str()),
            "the refusal must name this format's supported major `{major}`: `{message}`"
        );
    }
}

/// proves: Test strategy #4 + Chosen design (Load-time validation, step 3) — the
/// registry join: `load_manifest` refuses a grammar-valid action id the loaded
/// registry does not carry, naming the id. A manifest naming an action the
/// taxonomy does not own joins nothing on the profile side, and ADR-011 grows the
/// taxonomy only through manifest review — that order is what this refusal
/// enforces.
#[test]
fn load_refuses_an_unregistered_action_id_naming_the_id() {
    let registry = registry("manifest-join-registry", REGISTRY_DOC);
    let element = format!("{UNREGISTERED_ACTION} Turn Left");
    let text = doc_with_entry("actions", &array(&[element.as_str()]));
    let (_scratch, directory) = game_dir("manifest-join-game", "remnant-2", &text);

    let Err(err) = bindforge_core::load_manifest(&directory, &registry) else {
        panic!(
            "`{UNREGISTERED_ACTION}` is grammar-valid but the registry does not list it, so the \
             manifest must be refused"
        );
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "no new `CoreError` variant enters with this format, got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains(UNREGISTERED_ACTION),
        "the refusal must name the id the registry does not back, got `{detail}`"
    );
}

/// proves: Test strategy #4 — and it accepts the same document once a registry
/// built from entries lists that id: the join is against the loaded registry, not
/// a hard-coded list, which is what makes the proposal's same-PR `registry.toml`
/// row the whole of what a growing manifest needs.
#[test]
fn load_accepts_an_action_id_once_the_registry_lists_it() {
    let grown = REGISTRY_DOC.replace(
        r#"none = ["combat.fire", "ui.menu"]"#,
        &format!("none = [\"combat.fire\", \"ui.menu\", \"{UNREGISTERED_ACTION}\"]"),
    );
    let registry = registry("manifest-join-grown-registry", &grown);
    let element = format!("{UNREGISTERED_ACTION} Turn Left");
    let text = doc_with_entry("actions", &array(&[element.as_str()]));
    let (_scratch, directory) = game_dir("manifest-join-grown-game", "remnant-2", &text);

    let manifest = bindforge_core::load_manifest(&directory, &registry)
        .unwrap_or_else(|err| panic!("a manifest the registry backs must load: {err}"));
    assert_eq!(
        rows_of(&manifest),
        vec![(UNREGISTERED_ACTION.to_string(), "Turn Left".to_string())],
        "the joined row keeps its id and its native name"
    );
}

/// proves: Chosen design (Load-time validation, step 3) — the join covers every
/// taxonomy id the document names, `docked-actions` included: a docked row naming
/// an unregistered action explains a refused transform target by referring to an
/// action the taxonomy does not own.
#[test]
fn load_refuses_an_unregistered_docked_action_id_naming_the_id() {
    let registry = registry("manifest-dock-registry", REGISTRY_DOC);
    let text = doc_with_entry("docked-actions", &array(&["ui.inventory keyboard:y"]));
    let (_scratch, directory) = game_dir("manifest-dock-game", "remnant-2", &text);

    let Err(err) = bindforge_core::load_manifest(&directory, &registry) else {
        panic!("`ui.inventory` is not in the registry, so the manifest must be refused");
    };
    let detail = err.to_string();
    assert!(
        detail.contains("ui.inventory"),
        "the refusal must name the unregistered id, got `{detail}`"
    );
}

/// proves: Chosen design (Load-time validation, step 3) — the join is only over
/// taxonomy ids: `codec` and the preset ids are *not* registry-checked, because no
/// implementation set exists to check them against until ADR-017 (A15). A loader
/// that refused an unknown codec would be applying a rule the contract does not
/// have.
#[test]
fn load_does_not_registry_check_codec_or_preset_ids() {
    let registry = registry("manifest-codec-registry", REGISTRY_DOC);
    let (_scratch, directory) = game_dir("manifest-codec-game", "remnant-2", VALID_MANIFEST);

    let manifest = bindforge_core::load_manifest(&directory, &registry).unwrap_or_else(|err| {
        panic!("`codec` and `presets` name nothing the registry owns, so this must load: {err}")
    });
    assert_eq!(manifest.codec, "unreal-engine-5");
    assert_eq!(
        manifest.presets,
        vec!["default".to_string(), "southpaw".to_string()]
    );
}

/// proves: Test strategy #4 + Chosen design (Load-time validation, step 2) — the
/// location rule: the game directory's file-name segment must equal the `id` entry
/// byte-for-byte. A directory spelled `Remnant-2` and a directory named `other`
/// are both refused, and neither refusal echoes a local path — the error
/// contract's channel for paths is `source_path`. This is the name↔location
/// mismatch the Elite-Binds validator exists to catch (01-research.md).
#[test]
fn load_refuses_a_directory_name_that_is_not_the_manifest_id_without_echoing_a_path() {
    let registry = registry("manifest-location-registry", REGISTRY_DOC);

    for dir_name in ["Remnant-2", "other"] {
        let (scratch, directory) = game_dir("manifest-location-game", dir_name, VALID_MANIFEST);
        let shown = scratch.path().display().to_string();

        let Err(err) = bindforge_core::load_manifest(&directory, &registry) else {
            panic!(
                "a `remnant-2` manifest sitting in a `{dir_name}` directory contradicts its own \
                 id and must be refused"
            );
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a location mismatch is `InvalidValue` (Chosen design → Load-time validation), got \
             {err:?}"
        );
        let detail = err.to_string();
        assert!(
            !detail.contains(shown.as_str()),
            "the refusal must never echo a local filesystem path: {detail}"
        );
        assert!(
            !detail.contains('\n') && !detail.contains('\r'),
            "the refusal is one bounded line: {detail:?}"
        );
    }
}

/// proves: Chosen design (Load-time validation, step 2) — the rule's other side: a
/// directory named exactly its `id` loads, so the location rule is a guard and not
/// a cap on legitimate data.
#[test]
fn load_accepts_a_manifest_from_a_directory_named_for_its_id() {
    let registry = registry("manifest-match-registry", REGISTRY_DOC);
    let (_scratch, directory) = game_dir("manifest-match-game", "remnant-2", VALID_MANIFEST);

    let manifest = bindforge_core::load_manifest(&directory, &registry).unwrap_or_else(|err| {
        panic!("a manifest in a directory named for its id must load: {err}")
    });
    assert_eq!(manifest.id, "remnant-2");
    assert_eq!(
        rows_of(&manifest),
        vec![
            ("move.steer-left".to_string(), "Turn Left".to_string()),
            ("aim.camera".to_string(), "Mouse Camera".to_string()),
            ("combat.fire".to_string(), "Primary Attack".to_string()),
        ],
        "loading keeps the action table the document wrote"
    );
}

/// proves: Test strategy #4 + Chosen design (Budgets) — more `actions` rows than
/// `MAX_MANIFEST_ROWS` is refused before any row is built: the cap is what stops a
/// hostile manifest driving the registry join over an unbounded table.
#[test]
fn refuses_more_actions_rows_than_max_manifest_rows() {
    let text = doc_with_entry(
        "actions",
        &legal_rows(bindforge_core::MAX_MANIFEST_ROWS + 1),
    );

    assert_parse_refused(&text);
}

/// proves: Test strategy #4 + Revision log r1 — the same cap bounds `presets`, so
/// the 257th preset id is refused: `MAX_MANIFEST_ROWS` covers every array-shaped
/// entry, which is why the design adds no fifth constant.
#[test]
fn refuses_a_preset_array_past_max_manifest_rows() {
    let text = doc_with_entry(
        "presets",
        &legal_presets(bindforge_core::MAX_MANIFEST_ROWS + 1),
    );

    assert_parse_refused(&text);
}

/// proves: Chosen design (Budgets) — the cap bounds each array-shaped entry, so
/// `dead-keys` and `docked-actions` are refused past it too.
#[test]
fn refuses_more_dead_keys_or_docked_rows_than_max_manifest_rows() {
    for (name, value) in [
        (
            "dead-keys",
            dead_keys(bindforge_core::MAX_MANIFEST_ROWS + 1),
        ),
        (
            "docked-actions",
            legal_docked(bindforge_core::MAX_MANIFEST_ROWS + 1),
        ),
    ] {
        let text = doc_with_entry(name, &value);

        assert_parse_refused(&text);
    }
}

/// proves: Chosen design (Budgets) — the caps are limits, not off-by-one walls:
/// exactly `MAX_MANIFEST_ROWS` rows, and exactly `MAX_MANIFEST_ROWS` preset ids,
/// parse — each well above the baselines the proposal cites the cap sitting over
/// (the 40-id registry, A7; TF2-scale bind lists, A9).
#[test]
fn accepts_an_array_at_max_manifest_rows() {
    for (name, value) in [
        ("actions", legal_rows(bindforge_core::MAX_MANIFEST_ROWS)),
        ("presets", legal_presets(bindforge_core::MAX_MANIFEST_ROWS)),
    ] {
        let text = doc_with_entry(name, &value);

        let manifest = parse_manifest(&text);
        let length = match name {
            "actions" => manifest.actions.len(),
            _ => manifest.presets.len(),
        };
        assert_eq!(
            length,
            bindforge_core::MAX_MANIFEST_ROWS,
            "an array sitting exactly on the cap must parse"
        );
    }
}

/// proves: Test strategy #4 + Chosen design (Budgets) — a 65th exclusivity group
/// is refused: `MAX_CONSTRAINT_GROUPS` is the bound the existing budget family does
/// not cover, the hole the research flags for set-shaped data.
#[test]
fn refuses_an_exclusivity_group_array_past_max_constraint_groups() {
    let text = doc_with_entry(
        "exclusive-groups",
        &groups(bindforge_core::MAX_CONSTRAINT_GROUPS + 1),
    );

    assert_parse_refused(&text);
}

/// proves: Test strategy #5 + A3 — the round-trip obligation the harness holds
/// every format but `reach-classes` to: `parse(serialize(parse(text)))` equals the
/// first parse, for a document carrying all eleven entries, native names with
/// spaces and punctuation included and each array's document order preserved — so
/// the harness's comparison is equality, not luck.
#[test]
fn round_trips_a_manifest_carrying_every_defined_entry() {
    let first = bindforge_core::parse(VALID_MANIFEST)
        .expect("the eleven-entry manifest is a valid document");
    let round_tripped = bindforge_core::parse(&bindforge_core::serialize(&first))
        .unwrap_or_else(|err| panic!("a serialized manifest must re-parse: {err}"));

    assert_eq!(
        round_tripped, first,
        "serializing a manifest and reading it back must yield a semantically equal value"
    );

    let Document::Manifest(manifest) = round_tripped else {
        panic!("a round-tripped `manifests` document must still be a manifest");
    };
    assert_eq!(manifest.id, "remnant-2");
    assert_eq!(manifest.revision, 3);
    assert_eq!(manifest.codec, "unreal-engine-5");
    assert_eq!(
        manifest.fingerprint.as_deref(),
        Some(format!("sha256-{DIGEST}").as_str())
    );
    assert_eq!(rows_of(&manifest), rows_of(&parse_manifest(VALID_MANIFEST)));
    assert_eq!(
        groups_of(&manifest),
        vec![vec!["keyboard:w".to_string(), "keyboard:s".to_string()]]
    );
    assert_eq!(spellings(&manifest.dead_keys), vec!["keyboard:x"]);
    assert_eq!(
        docked_of(&manifest),
        vec![("ui.menu".to_string(), "keyboard:y".to_string())]
    );
    assert_eq!(
        manifest.presets,
        vec!["default".to_string(), "southpaw".to_string()]
    );
}

/// proves: Test strategy #5 + The model (the `serialize` branch) — serialization
/// writes the entries in its fixed order and preserves each array's document
/// order: a manifest whose rows, groups, dead keys, docked rows and presets are
/// deliberately unsorted comes back in the same order, not a canonicalized one.
#[test]
fn round_trips_a_manifest_preserving_document_order() {
    let text = manifest_doc(&[
        entry("format", "\"manifests\""),
        entry("version", "\"v1\""),
        entry("id", "\"remnant-2\""),
        entry("revision", "7"),
        entry("codec", "\"unreal-engine-5\""),
        entry(
            "actions",
            &array(&[
                "combat.fire Primary attack",
                "aim.camera Mouse camera",
                "move.steer-left Turn left",
            ]),
        ),
        entry(
            "exclusive-groups",
            &array(&["keyboard:s keyboard:w", "keyboard:a keyboard:d"]),
        ),
        entry("dead-keys", &array(&["keyboard:c", "keyboard:x"])),
        entry(
            "docked-actions",
            &array(&["ui.map keyboard:z", "ui.menu keyboard:y"]),
        ),
        entry("presets", &array(&["southpaw", "default"])),
    ]);

    let before = parse_manifest(&text);
    let document = bindforge_core::parse(&text)
        .expect("the deliberately unsorted manifest is a valid document");
    let re_parsed =
        bindforge_core::parse(&bindforge_core::serialize(&document)).unwrap_or_else(|err| {
            panic!("the unsorted manifest must re-parse after serializing: {err}")
        });
    let Document::Manifest(after) = re_parsed else {
        panic!("a round-tripped manifest must still be a manifest document");
    };

    assert_eq!(rows_of(&after), rows_of(&before), "row order is data");
    assert_eq!(
        groups_of(&after),
        vec![
            vec!["keyboard:s".to_string(), "keyboard:w".to_string()],
            vec!["keyboard:a".to_string(), "keyboard:d".to_string()],
        ],
        "group order and each group's member order are data"
    );
    assert_eq!(
        spellings(&after.dead_keys),
        vec!["keyboard:c", "keyboard:x"],
        "dead-key order is data"
    );
    assert_eq!(
        docked_of(&after),
        vec![
            ("ui.map".to_string(), "keyboard:z".to_string()),
            ("ui.menu".to_string(), "keyboard:y".to_string()),
        ],
        "docked-row order is data"
    );
    assert_eq!(
        after.presets,
        vec!["southpaw".to_string(), "default".to_string()],
        "preset order is data"
    );
    assert_eq!(
        after.revision, 7,
        "the integer revision survives serialization"
    );
    assert_eq!(after.fingerprint, None, "an absent entry stays absent");
}

/// proves: Test strategy #5 + A13, A19 — the hardest round-trip case: a native
/// name carrying interior spaces, a `#`, a quote and a backslash comes back
/// byte-exactly, because the name's grammar is what makes the remainder-split
/// encoding reversible. A name that came back mangled would rename a game's
/// action in place.
#[test]
fn round_trips_a_native_name_carrying_spaces_punctuation_quotes_and_a_backslash() {
    let text = doc_with_entry(
        "actions",
        &array(&[
            r#"aim.camera Say \"turn\" \\ left # look"#,
            "move.steer-left Turn   Left",
        ]),
    );
    let before = parse_manifest(&text);
    assert_eq!(
        rows_of(&before),
        vec![
            (
                "aim.camera".to_string(),
                "Say \"turn\" \\ left # look".to_string()
            ),
            ("move.steer-left".to_string(), "Turn   Left".to_string()),
        ],
        "the name keeps its `#`, its quotes and its backslash exactly as written"
    );

    let document =
        bindforge_core::parse(&text).expect("the punctuation-bearing manifest is a valid document");
    let re_parsed = bindforge_core::parse(&bindforge_core::serialize(&document))
        .unwrap_or_else(|err| panic!("a name carrying escapes must re-parse: {err}"));
    let Document::Manifest(after) = re_parsed else {
        panic!("a round-tripped manifest must still be a manifest document");
    };

    assert_eq!(
        rows_of(&after),
        rows_of(&before),
        "every native name must survive serialization byte-exactly"
    );
}
