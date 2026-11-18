//! The cross-file chain machinery of ADR-014: `schemas/profile/v2/` read
//! through `bindforge-core`, the `chain_ops` module's `load_chain`, `resolve`,
//! `fork`, `diff`, and `merge`, and the load-time joins a single-document
//! fixture corpus cannot carry.
//!
//! proves: design 014's Test strategy #1 (the v2 entry set and every per-link
//! parse rule), #2 (resolution semantics as the determinism rules state them),
//! #3 (the `load_chain` guards over a resolved chain of real files), #4 (the
//! load-time joins), #5 (`fork`, `diff`, and `merge`), and #6 (the
//! parse→serialize→re-parse round-trip, delta array order included, and the
//! untouched v1 `serialize`).
//!
//! The canonical statement of every rule here is the proposal (Chosen design →
//! The contract, The model, Cross-file loading, Resolution → the determinism
//! rules, Chain operations, Provenance labels), read against 01-research.md's
//! Findings. Four decisions bound the whole suite:
//!
//! - **The single chain-major directory.** Every link is
//!   `<profile-id>.toml` directly in the head's `vN/` directory, so a v2 chain
//!   is single-major by construction (Options: where ancestor documents come
//!   from). The chains below are laid out that way in a scratch tree, and a
//!   link that lives outside it — in a `v1/` directory, say — refuses exactly
//!   as one that lives nowhere does (A5).
//! - **The determinism rules are cited by number** (Resolution → the
//!   determinism rules 1–6): links deepest-first; `set` → `clear` → `swap`
//!   within a link, elements sequentially in array order, array order is data;
//!   last write wins, an unbound side stays unbound; an unbound action
//!   contributes nothing; output in the manifest's `actions` row order. The
//!   resolution cases below are that list, not an implementation's reading of
//!   it, and they are part of the compatibility surface from merge on
//!   (A10, A11).
//! - **Parse level versus load level.** Every rule the corpus can express
//!   refuses at `parse` alone — which is what keeps `profile/v2/fixtures/`
//!   single-document like every other format's (A16) — so the grammar cases
//!   here drive `bindforge_core::parse` directly. Only the chain-level rules
//!   (the resolved-chain guards, the position rules, the joins) wait for
//!   `load_chain` and `resolve`, where the other documents are in hand; those
//!   cases drive real files in a scratch tree (repo practice: unit tests, temp
//!   directories, no network).
//! - **The refusal shapes are pinned once in The model:** one new variant,
//!   `CoreError::ChainLinkMissing` (A18) — and only for a link the walk
//!   constructed from a `derives` element, never for the head the caller
//!   named: an absent head stays the read refusal whose payload carries the
//!   path (`source_path`, A13). Every other v2 fault is `InvalidValue` or
//!   `MissingEntry`, and the directory-major rule is answered by `InvalidValue`
//!   like every other loader answers it (`schemas/profile/v2/spec.md` →
//!   Loading a chain, rule 2) — `UnsupportedVersion` stays the refusal for a
//!   major the format does not ship. A shipped variant's message is a frozen
//!   contract string (A15): the suite pins the *variant*, never wording it
//!   cannot carry. Every message is the one-line, escaped, truncating,
//!   path-free line the error contract ships (A13). Cases that need an
//!   offender named assert it appears in the message, not the sentence
//!   around it.
//!
//! The v2 fixture corpus itself is pinned through the unchanged conformance
//! walk (`conformance.rs`, Test strategy #7) and the entry-set cases in
//! `grammar_closure.rs` (#8); this suite is the module-mirrored half (ADR-004)
//! and the only place the join-level rules exist as tests.
//!
//! The design ships: `crates/core/src/chain_ops.rs` exists and the gate runs
//! this suite against it. What the proposal states, these cases pin — and none
//! of them may be weakened or deleted to make an implementation pass: where the
//! two disagree, the contract moves the code, not the test.

mod common;

use std::path::{Path, PathBuf};

use bindforge_core::{Chain, CoreError, Document, Manifest, Profile, ProfileKind};
use common::{ScratchDir, profile_doc};

/// The manifest pin every conforming chain document in this suite carries —
/// design 013's canonical identity spelling `"<id>@<revision>"` (A7).
const PIN: &str = "remnant-2@3";

/// The manifest the load-time joins are pinned against: the rows `move.a`,
/// `move.b`, `aim.c`, `ui.d` — that row order is the output order every
/// resolution comparison reads (rule 5) — and the presets a `seeded-from` may
/// name (A7).
const MANIFEST_TEXT: &str = r#"format = "manifests"
version = "v1"
id = "remnant-2"
revision = 3
codec = "unreal-engine-5"
actions = ["move.a Turn left", "move.b Turn right", "aim.c Fire", "ui.d Menu"]
presets = ["default", "southpaw"]
"#;

/// The `entry = value` line this suite writes: `entry` names the entry and
/// `value` is already spelled in the grammar's shape — the same helper shape
/// the manifest suite draws.
fn entry<'a>(name: &'a str, value: &str) -> (&'a str, String) {
    (name, format!("{name} = {value}"))
}

/// A basic string spelled as the grammar spells it.
fn quoted(value: &str) -> String {
    format!("\"{value}\"")
}

/// A one-line string array of these already-spelled elements.
fn quoted_array(elements: impl IntoIterator<Item = String>) -> String {
    let items = elements
        .into_iter()
        .map(|element| quoted(&element))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}

/// A one-line string array quoting these borrowed elements.
fn array<'a>(elements: impl IntoIterator<Item = &'a str>) -> String {
    quoted_array(elements.into_iter().map(str::to_string))
}

/// The document made of these entries, one per line, in the order given.
fn doc(entries: &[(&str, String)]) -> String {
    entries
        .iter()
        .map(|(_, line)| format!("{line}\n"))
        .collect()
}

/// The eleven defined v2 entries, each with a fault-free value: the shape
/// `full-patch.toml` ships, and the document every grammar case below edits
/// exactly one entry of.
fn complete_entries() -> Vec<(&'static str, String)> {
    vec![
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"full-chain-link\""),
        entry("derives", &array(["game-base"])),
        entry("kind", "\"patch\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(["move.a keyboard:w"])),
        entry("clear", &array(["move.b"])),
        entry("swap", &array(["move.a move.b"])),
        entry("seeded-from", "\"default\""),
        entry("generated-by", "\"human\""),
    ]
}

/// The defined eleven with these entries replaced (an undefined entry is
/// appended), so every value-level and unknown-entry case is exactly the
/// faults listed.
fn doc_with_entries(edits: &[(&str, String)]) -> String {
    let mut entries: Vec<(String, String)> = complete_entries()
        .into_iter()
        .map(|(name, line)| (name.to_string(), line))
        .collect();
    for (name, value) in edits {
        let mut replaced = false;
        for (entry_name, line) in entries.iter_mut() {
            if entry_name == name {
                *line = format!("{name} = {value}");
                replaced = true;
            }
        }
        if !replaced {
            entries.push((name.to_string(), format!("{name} = {value}")));
        }
    }
    entries
        .iter()
        .map(|(_, line)| format!("{line}\n"))
        .collect()
}

/// The defined eleven with one entry edited.
fn doc_with_entry(name: &str, value: &str) -> String {
    doc_with_entries(&[(name, value.to_string())])
}

/// The defined eleven with these entries dropped, for the required-entry
/// cases.
fn doc_without_entries(names: &[&str]) -> String {
    let entries: Vec<(&str, String)> = complete_entries()
        .into_iter()
        .filter(|(name, _)| !names.contains(name))
        .collect();
    doc(&entries)
}

/// A v2 `base`: the entries a base carries, its layout its `set` list over the
/// all-unbound baseline (determinism rule 6).
fn base_doc(id: &str, set: &[&str]) -> String {
    doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", &quoted(id)),
        entry("kind", "\"base\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(set.iter().copied())),
    ])
}

/// A v2 `patch` over its one link below: exactly one `derives` element (The
/// contract → base/patch shape).
fn patch_doc(id: &str, derives: &str, set: &[&str], clear: &[&str], swap: &[&str]) -> String {
    doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", &quoted(id)),
        entry("derives", &array([derives])),
        entry("kind", "\"patch\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(set.iter().copied())),
        entry("clear", &array(clear.iter().copied())),
        entry("swap", &array(swap.iter().copied())),
    ])
}

/// Write a `Profile` back out as a contract file: the fixed entry order and
/// the space-joined pair spelling the entry table pins. This is the suite's
/// own writer — the grammar it spells is the entry table's, not an
/// implementation's — so a `fork` shell or `diff` output can be written beside
/// the chain it extends, which is the file-level move Test strategy #5
/// describes ("write it beside a's head, `load_chain` it as the new head").
fn profile_file_text(profile: &Profile) -> String {
    let mut lines: Vec<String> = vec![
        "format = \"profile\"".to_string(),
        format!("version = {}", quoted(&profile.version)),
        format!("id = {}", quoted(&profile.id)),
    ];
    if !profile.derives.is_empty() {
        lines.push(format!(
            "derives = {}",
            quoted_array(profile.derives.iter().cloned())
        ));
    }
    if let Some(kind) = &profile.kind {
        let spelled = match kind {
            ProfileKind::Base => "base",
            ProfileKind::Patch => "patch",
        };
        lines.push(format!("kind = {}", quoted(spelled)));
        lines.push(format!("for = {}", quoted(&profile.for_manifest)));
    }
    if !profile.set.is_empty() {
        lines.push(format!(
            "set = {}",
            quoted_array(profile.set.iter().map(|(action, input)| format!(
                "{} {}",
                action.as_str(),
                input.as_str()
            )))
        ));
    }
    if !profile.clear.is_empty() {
        lines.push(format!(
            "clear = {}",
            quoted_array(
                profile
                    .clear
                    .iter()
                    .map(|action| action.as_str().to_string())
            )
        ));
    }
    if !profile.swap.is_empty() {
        lines.push(format!(
            "swap = {}",
            quoted_array(profile.swap.iter().map(|(first, second)| format!(
                "{} {}",
                first.as_str(),
                second.as_str()
            )))
        ));
    }
    if let Some(seeded_from) = &profile.seeded_from {
        lines.push(format!("seeded-from = {}", quoted(seeded_from)));
    }
    if let Some(generated_by) = &profile.generated_by {
        lines.push(format!("generated-by = {}", quoted(generated_by)));
    }
    lines.iter().map(|line| format!("{line}\n")).collect()
}

/// A `(String, String)` list from borrowed pairs, for the delta comparisons.
fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(first, second)| (first.to_string(), second.to_string()))
        .collect()
}

/// A `Vec<String>` from borrowed words.
fn words(items: &[&str]) -> Vec<String> {
    items.iter().map(|word| word.to_string()).collect()
}

/// Parse `text` and hand back the `Document`, failing the case if a document
/// the entry table defines is refused.
fn parse_ok(text: &str) -> Document {
    bindforge_core::parse(text)
        .unwrap_or_else(|err| panic!("a document the entry table defines must parse: {err}"))
}

/// The `Profile` a parsed profile document carries.
fn profile_in(document: &Document) -> &Profile {
    document
        .profile()
        .expect("a `format = \"profile\"` document must parse into the profile arm (The model)")
}

/// The parse-level refusal the design promises for `text`: the document must
/// not parse, and the refusal must be the contract's value error — the only
/// new `CoreError` variant this design adds is `ChainLinkMissing`, and every
/// other v2 refusal is `InvalidValue` or `MissingEntry` (The model, error
/// rule).
fn parse_refusal(text: &str) -> CoreError {
    let Err(err) = bindforge_core::parse(text) else {
        panic!("this document is an invalid case and must be refused, not parsed: {text}");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "every v2 value-level refusal is `InvalidValue` (The model, error rule), got {err:?}"
    );
    err
}

/// A required entry that is absent is `MissingEntry` naming the entry.
fn missing_entry_refusal(text: &str, entry_name: &str) {
    let Err(err) = bindforge_core::parse(text) else {
        panic!("a v2 profile with no `{entry_name}` entry must be refused");
    };
    assert!(
        matches!(err, CoreError::MissingEntry { .. }),
        "an absent required entry is `MissingEntry` (The model, error rule), got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains(entry_name),
        "the refusal must name the entry it found missing, got `{detail}`"
    );
}

/// The refusal shape every named refusal shares: it names the offender it
/// refused, in the one-line, control-free message the contract promises
/// (A13). Which sentence names it is not part of the contract.
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

/// Lay out one chain directory in a scratch tree: each `(file name, text)`
/// becomes `<name>.toml` directly in `v2/` — the single-major directory rule
/// (Options: where ancestor documents come from) — with each file name equal
/// to the id its document carries. The scratch tree outlives the path:
/// dropping it first would remove the chain out from under `load_chain`.
fn chain_dir(label: &str, docs: &[(&str, String)]) -> (ScratchDir, PathBuf) {
    let scratch = ScratchDir::new(label);
    for (name, text) in docs {
        scratch.write(&format!("v2/{name}.toml"), text);
    }
    let directory = scratch.path().join("v2");
    (scratch, directory)
}

/// `load_chain` of one named head from a chain directory.
fn load_head(directory: &Path, head: &str) -> Result<Chain, CoreError> {
    bindforge_core::load_chain(&directory.join(format!("{head}.toml")))
}

/// A chain that must load, or the case fails with the refusal it got.
fn loaded(directory: &Path, head: &str) -> Chain {
    load_head(directory, head).unwrap_or_else(|err| {
        panic!("the chain rooted at `{head}` is a legal v2 chain and must load: {err}")
    })
}

/// The manifest every join and resolution case reads.
fn manifest() -> Manifest {
    let document = parse_ok(MANIFEST_TEXT);
    match document {
        Document::Manifest(manifest) => manifest,
        _ => panic!("a `format = \"manifests\"` document must parse into the manifest arm"),
    }
}

/// A resolved layout as `(action id, input id)` pairs, in the order `resolve`
/// returns them — the comparison the determinism rules are pinned with.
fn layout(chain: &Chain, manifest: &Manifest) -> Vec<(String, String)> {
    bindforge_core::resolve(chain, manifest)
        .unwrap_or_else(|err| panic!("a chain joined to its manifest must resolve: {err}"))
        .into_iter()
        .map(|binding| {
            (
                binding.action.as_str().to_string(),
                binding.input.as_str().to_string(),
            )
        })
        .collect()
}

/// A profile's delta lists as id spellings, for the grammar and chain-
/// operation comparisons.
fn sets_of(profile: &Profile) -> Vec<(String, String)> {
    profile
        .set
        .iter()
        .map(|(action, input)| (action.as_str().to_string(), input.as_str().to_string()))
        .collect()
}

fn clears_of(profile: &Profile) -> Vec<String> {
    profile
        .clear
        .iter()
        .map(|action| action.as_str().to_string())
        .collect()
}

fn swaps_of(profile: &Profile) -> Vec<(String, String)> {
    profile
        .swap
        .iter()
        .map(|(first, second)| (first.as_str().to_string(), second.as_str().to_string()))
        .collect()
}

/// The base every resolution case starts from: `move.a`, `move.b`, and `aim.c`
/// bound, `ui.d` left unbound — the layout is partial by construction
/// (determinism rule 4).
fn base_chain_doc() -> String {
    base_doc(
        "game-base",
        &["move.a keyboard:w", "move.b keyboard:e", "aim.c keyboard:r"],
    )
}

/// A two-link chain — `game-base` plus the head patch `retune` over it — the
/// shape every chain-operation case forks, diffs, and merges around.
fn two_link_docs() -> Vec<(&'static str, String)> {
    vec![
        ("game-base", base_chain_doc()),
        (
            "retune",
            patch_doc(
                "retune",
                "game-base",
                &["move.a keyboard:q"],
                &["aim.c"],
                &[],
            ),
        ),
    ]
}

/// Write a chain of `links` files — `link-1` a base, each `link-<n>` a patch
/// deriving the one below — into a scratch tree, and return the head's name.
/// The guard's unit is documents opened, one per link (Cross-file loading,
/// step 3).
fn write_depth_chain(scratch: &ScratchDir, links: usize) -> String {
    for index in 1..=links {
        let name = format!("link-{index}");
        let text = if index == 1 {
            base_doc(&name, &["move.a keyboard:w"])
        } else {
            patch_doc(
                &name,
                &format!("link-{}", index - 1),
                &["move.a keyboard:w"],
                &[],
                &[],
            )
        };
        scratch.write(&format!("v2/{name}.toml"), &text);
    }
    format!("link-{links}")
}

/// Two patch texts over one base (`game-base`, pin `PIN`) with caller-supplied
/// deltas — the operands `merge` composes in chain order, `p1` below, `p2`
/// above (Chain operations → `merge`).
fn merge_operands(
    p1_set: &[&str],
    p1_clear: &[&str],
    p1_swap: &[&str],
    p2_set: &[&str],
    p2_clear: &[&str],
    p2_swap: &[&str],
) -> (String, String) {
    (
        patch_doc("p-one", "game-base", p1_set, p1_clear, p1_swap),
        patch_doc("p-two", "game-base", p2_set, p2_clear, p2_swap),
    )
}

/// The load-level refusal for a chain that must not load: `load_chain` must
/// answer with a typed error, never a `Chain`.
fn load_refusal(label: &str, directory: &Path, head: &str) -> CoreError {
    let Err(err) = load_head(directory, head) else {
        panic!("{label}: the chain rooted at `{head}` must be refused, not loaded");
    };
    err
}

// ---------------------------------------------------------------------------
// Test strategy #1 — the v2 entry set and the per-link parse rules, driven at
// `parse` alone (A16 keeps the corpus single-document).
// ---------------------------------------------------------------------------

/// proves: Test strategy #1 + The contract (closed entry set) — a document
/// using exactly the eleven defined entries parses, every entry landing in the
/// typed model: `kind`, the `for` pin, the three delta lists split into typed
/// ids, and both provenance labels.
#[test]
fn parses_a_profile_v2_document_using_exactly_the_defined_entries() {
    let document = parse_ok(&doc(&complete_entries()));
    let profile = profile_in(&document);

    assert_eq!(profile.id, "full-chain-link");
    assert_eq!(profile.version, "v2");
    assert!(
        matches!(profile.kind, Some(ProfileKind::Patch)),
        "`kind = \"patch\"` must land in the typed `ProfileKind` (The model), got {:?}",
        profile.kind
    );
    assert_eq!(profile.derives, words(&["game-base"]));
    assert_eq!(
        profile.for_manifest, PIN,
        "`for` is design 013's canonical identity pin (A7)"
    );
    assert_eq!(sets_of(profile), pairs(&[("move.a", "keyboard:w")]));
    assert_eq!(clears_of(profile), words(&["move.b"]));
    assert_eq!(swaps_of(profile), pairs(&[("move.a", "move.b")]));
    assert_eq!(profile.seeded_from.as_deref(), Some("default"));
    assert_eq!(profile.generated_by.as_deref(), Some("human"));
}

/// proves: Test strategy #1 + The contract (Required column) — the minimal
/// base is `minimal-base.toml`'s shape: `format`, `version`, `id`, `kind`,
/// `for`; `derives` and the delta lists default empty and the provenance
/// labels default absent. An unbound base is legal (determinism rule 6).
#[test]
fn parses_a_minimal_v2_base() {
    let text = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"bare-base\""),
        entry("kind", "\"base\""),
        entry("for", &quoted(PIN)),
    ]);

    let document = parse_ok(&text);
    let profile = profile_in(&document);
    assert!(
        matches!(profile.kind, Some(ProfileKind::Base)),
        "the label must land typed as `base`, got {:?}",
        profile.kind
    );
    assert!(
        profile.derives.is_empty(),
        "a base stands alone; `derives` defaults empty (A1)"
    );
    assert!(
        profile.set.is_empty() && profile.clear.is_empty() && profile.swap.is_empty(),
        "absent delta lists default to empty lists, not refusals"
    );
    assert!(profile.seeded_from.is_none() && profile.generated_by.is_none());
}

/// proves: Test strategy #1 (`missing-kind.toml`) — `kind` is required: its
/// absence is `MissingEntry` naming the entry, never a default. The label is
/// what the position rule checks against, so guessing it is the drift the
/// entry table refuses.
#[test]
fn refuses_a_v2_profile_missing_the_kind_entry_naming_the_entry() {
    missing_entry_refusal(&doc_without_entries(&["kind"]), "kind");
}

/// proves: Test strategy #1 (`missing-for.toml`) — `for` is required: a link
/// that pins nothing cannot be joined to a manifest at load, which is the
/// join every resolution runs (Load-time joins).
#[test]
fn refuses_a_v2_profile_missing_the_for_entry_naming_the_entry() {
    missing_entry_refusal(&doc_without_entries(&["for"]), "for");
}

/// proves: Test strategy #1 (`kind-outside-vocabulary.toml`) — `kind` is the
/// closed pair `{base, patch}`: a third label is `InvalidValue` naming the
/// value, never a label whose position gets inferred.
#[test]
fn refuses_a_v2_kind_outside_the_base_patch_vocabulary_naming_the_value() {
    let err = parse_refusal(&doc_with_entry("kind", "\"layer\""));
    assert_refusal_names(&err, &["kind", "layer"]);
}

/// proves: Test strategy #1 (`base-derives-nonempty.toml`) — a `base` has
/// empty `derives`: a base that names a link below it contradicts its own
/// label, the contradiction the contract answers with a refusal (Options:
/// what `kind` means).
#[test]
fn refuses_a_base_with_a_nonempty_derives_list_naming_the_fault() {
    let text = doc_with_entries(&[
        ("kind", "\"base\"".to_string()),
        ("clear", "[]".to_string()),
        ("swap", "[]".to_string()),
    ]);
    let err = parse_refusal(&text);
    assert_refusal_names(&err, &["derives", "game-base"]);
}

/// proves: Test strategy #1 (`base-clear-element.toml`, `base-swap-element.toml`)
/// — a base has no `clear` or `swap` elements: there is nothing below it to
/// drop or exchange, so such an element is a claim with nothing to act on.
#[test]
fn refuses_a_base_with_a_clear_or_swap_element_naming_the_element() {
    let with_clear = doc_with_entries(&[
        ("kind", "\"base\"".to_string()),
        ("derives", "[]".to_string()),
        ("swap", "[]".to_string()),
    ]);
    let with_swap = doc_with_entries(&[
        ("kind", "\"base\"".to_string()),
        ("derives", "[]".to_string()),
        ("clear", "[]".to_string()),
    ]);
    assert_refusal_names(&parse_refusal(&with_clear), &["clear", "move.b"]);
    assert_refusal_names(&parse_refusal(&with_swap), &["swap", "move.a move.b"]);
}

/// proves: Test strategy #1 (`patch-empty-deltas.toml`) — a patch carries at
/// least one element across `set`/`clear`/`swap`: the empty patch is the
/// vacuous link the guard posture refuses, spelled as three empty lists or as
/// no delta entries at all.
#[test]
fn refuses_a_patch_with_no_delta_elements() {
    let empty_lists = doc_with_entries(&[
        ("set", "[]".to_string()),
        ("clear", "[]".to_string()),
        ("swap", "[]".to_string()),
    ]);
    let no_lists = doc_without_entries(&["set", "clear", "swap"]);
    for text in [empty_lists, no_lists] {
        let err = parse_refusal(&text);
        assert_refusal_names(&err, &["set", "clear", "swap"]);
    }
}

/// proves: Test strategy #1 (`derives-two-elements.toml`) — a v2 patch carries
/// exactly one `derives` element: a second ancestor has nothing to point to
/// (a v2 chain is one path, not a tree — Cross-file loading).
#[test]
fn refuses_a_patch_with_two_derives_elements_naming_the_second() {
    let err = parse_refusal(&doc_with_entry(
        "derives",
        &array(["game-base", "other-base"]),
    ));
    assert_refusal_names(&err, &["derives", "other-base"]);
}

/// proves: Test strategy #1 (`missing-derives-patch.toml`) — a patch with an
/// empty `derives` list names no link below it: as a link its position can
/// never terminate the walk at a base.
#[test]
fn refuses_a_patch_with_an_empty_derives_list_naming_the_entry() {
    let err = parse_refusal(&doc_with_entry("derives", "[]"));
    assert_refusal_names(&err, &["derives"]);
}

/// proves: Test strategy #1 (`for-without-at.toml`, `for-revision-zero.toml`,
/// `for-uppercase-id.toml`) — the `for` pin splits at its one `@`: a slug head
/// and a decimal revision of at least 1, design 013's canonical identity and
/// the shipped `revision-zero` refusal shape (A7).
#[test]
fn refuses_a_malformed_for_pin_naming_the_offender() {
    for pin in [
        "remnant-2",
        "remnant-2@0",
        "Remnant-2@3",
        "remnant-2@three",
        "remnant-2@3@4",
    ] {
        let err = parse_refusal(&doc_with_entry("for", &quoted(pin)));
        assert_refusal_names(&err, &["for", pin]);
    }
}

/// proves: Test strategy #6 + the accepted code-review r1 finding (the load-seam
/// fault-kind split) — `check_pin` refuses a *grammar-valid* `for` head one octet
/// past `MAX_SLUG_OCTETS` by naming the bound it broke, interpolated from the
/// exported constant: the head satisfies `[a-z][a-z0-9-]*`, so a refusal that
/// recited the manifest-identity grammar would name a rule the offender keeps.
#[test]
fn refuses_a_for_pin_head_one_octet_past_the_slug_bound_naming_the_bound() {
    let head = "m".repeat(bindforge_core::MAX_SLUG_OCTETS + 1);
    let err = parse_refusal(&doc_with_entry("for", &quoted(&format!("{head}@3"))));
    let detail = err.to_string();
    assert!(
        detail.contains(&bindforge_core::MAX_SLUG_OCTETS.to_string()),
        "the bound fault names the shared bound it enforced ({}, built from the exported \
         constant), not the grammar rule the head satisfies: `{detail}`",
        bindforge_core::MAX_SLUG_OCTETS
    );
    assert!(
        !detail.contains('\n') && detail.chars().all(|ch| !ch.is_control()),
        "a refusal stays one bounded printable line whatever the head carried (A13): \
         {detail:?}"
    );
}

/// proves: Test strategy #1 (`id-uppercase.toml`, `id-past-cap.toml`), with the cap
/// re-pinned by design 024's Test strategy #6 — a v2
/// `id` rides the shipped slug grammar (A14), capped at the shared
/// `MAX_SLUG_OCTETS` bound (128 from ADR-024), because a
/// v2 id is a chain lookup key and a file name: uppercase, a dot path, a
/// slash path, and a one-past-the-bound spelling are each refused as `InvalidValue`
/// naming the id, never carried into a lookup.
#[test]
fn refuses_a_v2_id_that_breaks_the_slug_grammar_naming_the_id() {
    let long_id = "a".repeat(bindforge_core::MAX_SLUG_OCTETS + 1);
    for id in ["My-Profile", long_id.as_str(), "..", "a/b"] {
        let err = parse_refusal(&doc_with_entry("id", &quoted(id)));
        assert_refusal_names(&err, &["id", id]);
    }
}

/// proves: Chosen design (Id grammars) + design 024's Test strategy #6 — the shared
/// cap is a limit, not an
/// off-by-one wall: a slug id of exactly `MAX_SLUG_OCTETS` octets parses and survives.
#[test]
fn accepts_a_v2_id_at_the_slug_length_bound() {
    let id = "p".repeat(bindforge_core::MAX_SLUG_OCTETS);
    let document = parse_ok(&doc_with_entry("id", &quoted(&id)));
    assert_eq!(
        profile_in(&document).id,
        id,
        "the id is kept exactly as written"
    );
}

/// proves: Test strategy #1 (`derives-element-uppercase.toml`) + Cross-file
/// loading step 2 — every `derives` element is slug-checked at parse, which is
/// what makes the constructed link path always a direct child of the major
/// directory: `..`, `a/b`, an uppercase spelling, and a one-past-the-shared-bound
/// element are
/// refused before any path is built.
#[test]
fn refuses_a_derives_element_that_breaks_the_slug_grammar_naming_the_element() {
    let long_id = "a".repeat(bindforge_core::MAX_SLUG_OCTETS + 1);
    for element in ["..", "a/b", "Game-Base", long_id.as_str()] {
        let err = parse_refusal(&doc_with_entry("derives", &array([element])));
        assert_refusal_names(&err, &["derives", element]);
    }
}

/// proves: Test strategy #1 (`set-one-field.toml`, `set-three-fields.toml`) —
/// a `set` element splits at the first space into exactly two fields (the
/// shipped pair encoding, A8): one field and three fields are each refused
/// naming the element.
#[test]
fn refuses_a_set_element_with_the_wrong_field_count_naming_the_element() {
    for element in ["move.a", "move.a keyboard:w extra"] {
        let err = parse_refusal(&doc_with_entry("set", &array([element])));
        assert_refusal_names(&err, &["set", element]);
    }
}

/// proves: Test strategy #1 (`clear-two-fields.toml`) — a `clear` element is
/// one action id: a second field is a malformed delta, refused naming it.
#[test]
fn refuses_a_clear_element_that_names_two_fields_naming_the_element() {
    let err = parse_refusal(&doc_with_entry("clear", &array(["move.a move.b"])));
    assert_refusal_names(&err, &["clear", "move.a move.b"]);
}

/// proves: Test strategy #1 (`swap-one-field.toml`, `swap-three-fields.toml`)
/// — a `swap` element is two space-joined action ids: one field and three
/// fields are each refused naming the element.
#[test]
fn refuses_a_swap_element_with_the_wrong_field_count_naming_the_element() {
    for element in ["move.a", "move.a move.b keyboard:w"] {
        let err = parse_refusal(&doc_with_entry("swap", &array([element])));
        assert_refusal_names(&err, &["swap", element]);
    }
}

/// proves: Test strategy #1 (`swap-pair-same-action.toml`) — a `swap` pair
/// names two distinct actions: exchanging an action with itself is a delta
/// that means nothing, not a legal no-op.
#[test]
fn refuses_a_swap_pair_that_names_the_same_action_twice_naming_the_pair() {
    let err = parse_refusal(&doc_with_entry("swap", &array(["move.a move.a"])));
    assert_refusal_names(&err, &["swap", "move.a"]);
}

/// proves: Test strategy #1 + A17 — every action half of a delta element is
/// grammar-validated by `ActionId::parse` at parse: a single-segment id, an
/// underscore spelling, and a three-segment id are refused naming the
/// element, never left for the load-time row join to sort out.
#[test]
fn refuses_a_delta_element_that_names_a_malformed_action_id_naming_the_element() {
    for (list, element) in [
        ("set", "steer-left keyboard:w"),
        ("clear", "Move_a"),
        ("swap", "move.steer.left move.b"),
    ] {
        let err = parse_refusal(&doc_with_entry(list, &array([element])));
        assert_refusal_names(&err, &[list, element]);
    }
}

/// proves: Test strategy #1 (`set-unknown-input.toml`) + A9 — the input half
/// of a `set` is validated against the closed vocabulary at parse: `keyboard:`
/// has no `tab` member, so a v2 `set` still cannot name Tab — the same 012
/// gap the Deferred section records, now pinned as a refusal naming the id.
#[test]
fn refuses_a_set_element_naming_an_input_outside_the_closed_vocabulary_naming_the_input() {
    let err = parse_refusal(&doc_with_entry("set", &array(["move.a keyboard:tab"])));
    assert_refusal_names(&err, &["keyboard:tab"]);
}

/// proves: Chosen design (Delta elements) + A9 — an unknown input namespace is
/// refused at parse too: the vocabulary is closed, and `touch:` names no set
/// any reader has members for.
#[test]
fn refuses_a_set_element_naming_an_unknown_input_namespace_naming_the_input() {
    let err = parse_refusal(&doc_with_entry("set", &array(["move.a touch:tap"])));
    assert_refusal_names(&err, &["touch:tap"]);
}

/// proves: Test strategy #1 (`seeded-from-uppercase.toml`) — `seeded-from`
/// rides the slug grammar: a malformed preset id is a pin that can never
/// resolve, refused at parse naming the value (A14).
#[test]
fn refuses_a_seeded_from_that_breaks_the_slug_grammar_naming_the_value() {
    let err = parse_refusal(&doc_with_entry("seeded-from", "\"Default\""));
    assert_refusal_names(&err, &["seeded-from", "Default"]);
}

/// proves: Test strategy #1 (`generated-by-past-cap.toml`) — `generated-by` is
/// capped at `MAX_GENERATED_BY_OCTETS`: the cap is what bounds a refusal that
/// has to echo the label (A13).
#[test]
fn refuses_a_generated_by_past_the_octet_cap() {
    let long_label = "g".repeat(bindforge_core::MAX_GENERATED_BY_OCTETS + 1);
    let err = parse_refusal(&doc_with_entry("generated-by", &quoted(&long_label)));
    assert_refusal_names(&err, &["generated-by"]);
}

/// proves: Chosen design (The contract, `generated-by`) — the cap is a limit:
/// a label of exactly `MAX_GENERATED_BY_OCTETS` parses and survives
/// byte-exactly.
#[test]
fn accepts_a_generated_by_at_the_octet_bound() {
    let label = "g".repeat(bindforge_core::MAX_GENERATED_BY_OCTETS);
    let document = parse_ok(&doc_with_entry("generated-by", &quoted(&label)));
    assert_eq!(
        profile_in(&document).generated_by.as_deref(),
        Some(label.as_str())
    );
}

/// proves: Test strategy #1 (`generated-by-control-char.toml`,
/// `generated-by-bidi-char.toml`) — the label admits no ASCII control
/// character, even spelled through the closed escape table's `\\u0007`, and no
/// bidi-control character: a document-controlled label cannot steer a log
/// line (A13, Provenance labels).
#[test]
fn refuses_a_generated_by_carrying_control_or_bidi_characters() {
    let bell = doc_with_entry("generated-by", "\"Bell\\u0007tool\"");
    let bidi = doc_with_entry("generated-by", "\"tool \u{202e}name\"");
    for text in [bell, bidi] {
        let err = parse_refusal(&text);
        assert_refusal_names(&err, &["generated-by"]);
        assert!(
            !err.to_string().contains('\u{202e}'),
            "the refusal escapes a bidi-control character rather than emitting it: {:?}",
            err.to_string()
        );
    }
}

/// proves: Test strategy #1 — the 257th element in any delta list is refused:
/// each of `set`, `clear`, `swap` holds at most `MAX_PATCH_DELTA_ELEMENTS`,
/// the cap mirroring the manifest row cap that bounds one game's whole action
/// table (A17); the cap is stated against the constant, never the literal.
#[test]
fn rejects_a_delta_list_past_max_patch_delta_elements() {
    let over = bindforge_core::MAX_PATCH_DELTA_ELEMENTS + 1;
    for (name, element) in [
        ("set", "move.a keyboard:w"),
        ("clear", "move.b"),
        ("swap", "move.a move.b"),
    ] {
        let text = doc_with_entry(name, &array(std::iter::repeat_n(element, over)));
        let err = parse_refusal(&text);
        assert_refusal_names(&err, &[name]);
    }
}

/// proves: Chosen design (Composite budget) — the cap is a limit, not an
/// off-by-one wall: a delta list of exactly `MAX_PATCH_DELTA_ELEMENTS` parses
/// with every element kept.
#[test]
fn accepts_a_delta_list_at_max_patch_delta_elements() {
    let at = bindforge_core::MAX_PATCH_DELTA_ELEMENTS;
    let text = doc_with_entry("set", &array(std::iter::repeat_n("move.a keyboard:w", at)));
    let document = parse_ok(&text);
    assert_eq!(
        profile_in(&document).set.len(),
        at,
        "a list sitting exactly on the cap is legitimate input"
    );
}

/// proves: Test strategy #1 + A11 — the v2 entry set is closed: a `based-on`
/// entry, the alternative the ADR explicitly retired, is refused naming the
/// entry, never read as `derives` under another name.
#[test]
fn refuses_a_v2_profile_carrying_a_retired_based_on_entry_naming_the_entry() {
    let err = parse_refusal(&doc_with_entry("based-on", &array(["game-base"])));
    assert_refusal_names(&err, &["based-on"]);
}

// ---------------------------------------------------------------------------
// Test strategy #2 — resolution semantics, one case per determinism rule.
// ---------------------------------------------------------------------------

/// proves: Test strategy #2 + ADR Decision Two — the result is a function of
/// the chain: two full load-and-resolve passes over the same documents return
/// the same layout, which is what makes determinism a compatibility surface
/// (A10, A11).
#[test]
fn resolves_the_same_chain_identically_on_repeated_runs() {
    let (_scratch, directory) = chain_dir(
        "resolve-deterministic",
        &[
            ("game-base", base_chain_doc()),
            (
                "retune",
                patch_doc("retune", "game-base", &["ui.d keyboard:t"], &[], &[]),
            ),
        ],
    );
    let manifest = manifest();

    let first = layout(&loaded(&directory, "retune"), &manifest);
    let second = layout(&loaded(&directory, "retune"), &manifest);
    assert_eq!(first, second, "two runs over the same chain must agree");
}

/// proves: Test strategy #2 + determinism rule 1 — links apply deepest-first:
/// a patch's `set` replaces what the base bound, and actions no link binds
/// stay out of the output (`ui.d` unbound in the base, rule 4).
#[test]
fn applies_links_deepest_first_with_the_patch_winning_over_the_base() {
    let (_scratch, directory) = chain_dir(
        "resolve-deepest-first",
        &[
            ("game-base", base_chain_doc()),
            (
                "retune",
                patch_doc("retune", "game-base", &["move.a keyboard:q"], &[], &[]),
            ),
        ],
    );

    assert_eq!(
        layout(&loaded(&directory, "retune"), &manifest()),
        pairs(&[
            ("move.a", "keyboard:q"),
            ("move.b", "keyboard:e"),
            ("aim.c", "keyboard:r"),
        ]),
        "the base's `move.a keyboard:w` yields to the patch above it, and `ui.d` \
         contributes nothing"
    );
}

/// proves: Test strategy #2 + determinism rules 2 and 3 — repeated `set`s in
/// one list apply sequentially in array order and the last write wins: `set a`
/// then `set b` leaves `b`. Array order is data, pinned here and by
/// `set-twice-last-element.toml`.
#[test]
fn last_repeated_set_in_array_order_wins() {
    let (_scratch, directory) = chain_dir(
        "resolve-last-write-wins",
        &[
            ("game-base", base_chain_doc()),
            (
                "retune",
                patch_doc(
                    "retune",
                    "game-base",
                    &["move.a keyboard:w", "move.a keyboard:e"],
                    &[],
                    &[],
                ),
            ),
        ],
    );

    assert_eq!(
        layout(&loaded(&directory, "retune"), &manifest()),
        pairs(&[
            ("move.a", "keyboard:e"),
            ("move.b", "keyboard:e"),
            ("aim.c", "keyboard:r"),
        ]),
        "the second element is the last writer for `move.a`"
    );
}

/// proves: Test strategy #2 + determinism rule 2 — within one link the lists
/// apply in the fixed order `set` → `clear` → `swap`: a `set` and a `clear` for
/// one action in the same patch unbinds it, the retune-then-remove shape that
/// makes editing a patch legal (`set-then-clear-same-action.toml`).
#[test]
fn set_then_clear_in_one_patch_unbinds_the_action() {
    let (_scratch, directory) = chain_dir(
        "resolve-set-then-clear",
        &[
            ("game-base", base_chain_doc()),
            (
                "retune",
                patch_doc(
                    "retune",
                    "game-base",
                    &["move.a keyboard:q"],
                    &["move.a"],
                    &[],
                ),
            ),
        ],
    );

    assert_eq!(
        layout(&loaded(&directory, "retune"), &manifest()),
        pairs(&[("move.b", "keyboard:e"), ("aim.c", "keyboard:r")]),
        "`clear` runs after `set` in the fixed list order, so `move.a` ends unbound"
    );
}

/// proves: Test strategy #2 + determinism rule 3 — `swap` exchanges the two
/// named actions' current input bindings (Chosen design: `swap` is always
/// action↔action, the exchange primitive that avoids RFC 6902's `move`
/// intermediate states, A20).
#[test]
fn swap_exchanges_the_two_actions_current_bindings() {
    let (_scratch, directory) = chain_dir(
        "resolve-swap-bound",
        &[
            ("game-base", base_chain_doc()),
            (
                "retune",
                patch_doc("retune", "game-base", &[], &[], &["move.a move.b"]),
            ),
        ],
    );

    assert_eq!(
        layout(&loaded(&directory, "retune"), &manifest()),
        pairs(&[
            ("move.a", "keyboard:e"),
            ("move.b", "keyboard:w"),
            ("aim.c", "keyboard:r"),
        ]),
        "`move.a` takes `move.b`'s binding and `move.b` takes `move.a`'s"
    );
}

/// proves: Test strategy #2 + determinism rule 3 — an unbound side of a `swap`
/// stays unbound: the entry table's exact wording. A `swap` with one side
/// unbound moves nothing; resolution never invents or destroys a binding that
/// no `set` or `clear` wrote.
#[test]
fn swap_with_an_unbound_side_leaves_both_sides_as_they_are() {
    let (_scratch, directory) = chain_dir(
        "resolve-swap-one-unbound",
        &[
            ("game-base", base_doc("game-base", &["move.a keyboard:w"])),
            (
                "retune",
                patch_doc("retune", "game-base", &[], &[], &["move.a move.b"]),
            ),
        ],
    );

    assert_eq!(
        layout(&loaded(&directory, "retune"), &manifest()),
        pairs(&[("move.a", "keyboard:w")]),
        "`move.b` is unbound, so the swap leaves `move.a` bound and `move.b` unbound"
    );
}

/// proves: Chosen design (determinism rule 3) — a `swap` of two unbound
/// actions changes nothing: `swap` never fabricates a binding.
#[test]
fn swap_of_two_unbound_actions_changes_nothing() {
    let (_scratch, directory) = chain_dir(
        "resolve-swap-both-unbound",
        &[
            ("game-base", base_doc("game-base", &["move.a keyboard:w"])),
            (
                "retune",
                patch_doc("retune", "game-base", &[], &[], &["aim.c ui.d"]),
            ),
        ],
    );

    assert_eq!(
        layout(&loaded(&directory, "retune"), &manifest()),
        pairs(&[("move.a", "keyboard:w")]),
        "two unbound sides exchange nothing"
    );
}

/// proves: Test strategy #2 + determinism rule 5 — output is ordered by the
/// manifest's `actions` row order, with no sort rule of its own: the "row
/// order is data" precedent (A17), pinned with a base whose `set` list is
/// written in reverse row order.
#[test]
fn resolution_output_follows_the_manifest_action_rows_not_document_order() {
    let (_scratch, directory) = chain_dir(
        "resolve-row-order",
        &[(
            "game-base",
            base_doc(
                "game-base",
                &[
                    "ui.d keyboard:t",
                    "aim.c keyboard:r",
                    "move.b keyboard:e",
                    "move.a keyboard:w",
                ],
            ),
        )],
    );

    assert_eq!(
        layout(&loaded(&directory, "game-base"), &manifest()),
        pairs(&[
            ("move.a", "keyboard:w"),
            ("move.b", "keyboard:e"),
            ("aim.c", "keyboard:r"),
            ("ui.d", "keyboard:t"),
        ]),
        "the output follows the manifest's row order, not the document's list order"
    );
}

/// proves: Test strategy #2 + determinism rule 4 — an action with no binding
/// contributes nothing to the output: three of the manifest's four rows are
/// unbound and absent, the layout is partial by construction.
#[test]
fn unbound_actions_contribute_nothing_to_the_output() {
    let (_scratch, directory) = chain_dir(
        "resolve-partial-layout",
        &[("game-base", base_doc("game-base", &["move.a keyboard:w"]))],
    );

    assert_eq!(
        layout(&loaded(&directory, "game-base"), &manifest()),
        pairs(&[("move.a", "keyboard:w")]),
        "one bound row of four yields one output entry"
    );
}

/// proves: Test strategy #2 + determinism rule 6 — an unbound base is legal
/// and resolves to an empty layout: the all-unbound baseline is a document,
/// not an error (seed presets as real bases stay ADR-016/017's data).
#[test]
fn an_unbound_base_resolves_to_an_empty_layout() {
    let (_scratch, directory) = chain_dir(
        "resolve-empty-base",
        &[("game-base", base_doc("game-base", &[]))],
    );

    let empty: Vec<(String, String)> = Vec::new();
    assert_eq!(
        layout(&loaded(&directory, "game-base"), &manifest()),
        empty,
        "a base with no `set` list resolves to the all-unbound layout"
    );
}

/// proves: Test strategy #2 + Cross-file loading step 4 — a link contradicting
/// its position is `InvalidValue` naming the link id: a document labelled
/// `base` that carries a `clear` element is the kind/derives contradiction the
/// research says the ADR left open (Risks: `kind` vs `derives`), refused at
/// load and naming the link it refused.
#[test]
fn refuses_a_base_carrying_a_clear_at_load_naming_the_link() {
    let bad_base = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"bad-base\""),
        entry("kind", "\"base\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(["move.a keyboard:w"])),
        entry("clear", &array(["move.a"])),
    ]);
    let (_scratch, directory) = chain_dir(
        "load-base-with-clear",
        &[
            ("bad-base", bad_base),
            (
                "over-bad",
                patch_doc("over-bad", "bad-base", &["move.b keyboard:e"], &[], &[]),
            ),
        ],
    );

    let err = load_refusal("a `base` carrying a `clear`", &directory, "over-bad");
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "a position/shape contradiction is `InvalidValue`, got {err:?}"
    );
    assert_refusal_names(&err, &["bad-base"]);
}

/// proves: Test strategy #2 + Cross-file loading step 4 — a chain whose
/// deepest link is a `patch` refuses naming it: the walk terminates at an
/// empty `derives`, and a `patch` sitting there contradicts its position.
#[test]
fn refuses_a_chain_whose_deepest_link_is_a_patch_naming_the_link() {
    let deep_patch = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"deep-patch\""),
        entry("derives", "[]"),
        entry("kind", "\"patch\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(["move.a keyboard:w"])),
    ]);
    let (_scratch, directory) = chain_dir("load-deepest-patch", &[("deep-patch", deep_patch)]);

    let err = load_refusal("a chain terminating at a `patch`", &directory, "deep-patch");
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the position contradiction is `InvalidValue`, got {err:?}"
    );
    assert_refusal_names(&err, &["deep-patch"]);
}

/// proves: Test strategy #2 + Cross-file loading step 4 — a `base` mid-chain
/// refuses naming it: a document labelled `base` that names a link below it
/// contradicts both its label and its position (Options: what `kind` means —
/// the label is checked against `derives` at parse and against chain position
/// at load).
#[test]
fn refuses_a_base_mid_chain_naming_it() {
    let mid_base = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"mid-base\""),
        entry("derives", &array(["game-base"])),
        entry("kind", "\"base\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(["move.a keyboard:w"])),
    ]);
    let (_scratch, directory) = chain_dir(
        "load-base-mid-chain",
        &[
            ("game-base", base_doc("game-base", &["move.b keyboard:e"])),
            ("mid-base", mid_base),
            (
                "over-mid",
                patch_doc("over-mid", "mid-base", &["ui.d keyboard:t"], &[], &[]),
            ),
        ],
    );

    let err = load_refusal("a `base` mid-chain", &directory, "over-mid");
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the position contradiction is `InvalidValue`, got {err:?}"
    );
    assert_refusal_names(&err, &["mid-base"]);
}

// ---------------------------------------------------------------------------
// Test strategy #3 — the `load_chain` guards over a resolved chain of real
// files: the shipped refusals, now over the resolved chain (A2), and the new
// named refusal for a missing link (A18).
// ---------------------------------------------------------------------------

/// proves: Test strategy #3 + A2 — a two-file cycle (A derives B, B derives
/// A), invisible to the shipped single-document guard, refuses as `Cycle`: the
/// resolved-chain walk keeps the shipped refusal (A15). The variant is the
/// whole pin: `Cycle` carries no payload, so its message names neither link,
/// and its text is design 008's frozen contract string (A15) — a test needle
/// must not pull it out of shape.
#[test]
fn refuses_a_two_file_cycle_as_a_cycle() {
    let (_scratch, directory) = chain_dir(
        "load-two-file-cycle",
        &[
            (
                "link-a",
                patch_doc("link-a", "link-b", &["move.a keyboard:w"], &[], &[]),
            ),
            (
                "link-b",
                patch_doc("link-b", "link-a", &["move.b keyboard:e"], &[], &[]),
            ),
        ],
    );

    let err = load_refusal("a two-file cycle", &directory, "link-a");
    assert!(
        matches!(err, CoreError::Cycle),
        "expected the `Cycle` variant over the resolved chain, got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        !detail.is_empty(),
        "the refusal must say what it refused, got `{detail}`"
    );
    assert!(
        !detail.contains('\n') && !detail.contains('\r'),
        "the shipped refusal keeps its one-line shape (A13): {detail:?}"
    );
    assert!(
        detail.chars().all(|ch| !ch.is_control()),
        "a refusal escapes control characters rather than emitting them (A13): {detail:?}"
    );
}

/// proves: Test strategy #3 — a chain of exactly `MAX_CHAIN_DEPTH` opened
/// links loads: the guard's unit is documents opened, one per link
/// (Cross-file loading step 3), and the bound is a limit, not an off-by-one
/// wall. This is the sole depth pin for a resolved chain (A16) — written
/// against the constant, never the literal 32.
#[test]
fn resolves_a_chain_of_exactly_max_chain_depth_opened_links() {
    let scratch = ScratchDir::new("load-depth-at-cap");
    let head = write_depth_chain(&scratch, bindforge_core::MAX_CHAIN_DEPTH);
    let directory = scratch.path().join("v2");

    loaded(&directory, &head);
}

/// proves: Test strategy #3 — one opened link past `MAX_CHAIN_DEPTH` refuses
/// as `ChainTooDeep`, before any deeper traversal (A1), counting documents
/// opened.
#[test]
fn refuses_one_opened_link_past_max_chain_depth_as_chain_too_deep() {
    let scratch = ScratchDir::new("load-depth-past-cap");
    let head = write_depth_chain(&scratch, bindforge_core::MAX_CHAIN_DEPTH + 1);
    let directory = scratch.path().join("v2");

    let err = load_refusal(
        "a chain past MAX_CHAIN_DEPTH opened links",
        &directory,
        &head,
    );
    assert!(
        matches!(err, CoreError::ChainTooDeep),
        "expected the `ChainTooDeep` variant over the resolved chain, got {err:?}"
    );
}

/// proves: Test strategy #3 + A18 — a missing ancestor refuses as the new
/// `ChainLinkMissing` variant naming the id: "link missing" is not an I/O
/// detail, and the refusal names the contract object, never a local path
/// (A13).
#[test]
fn refuses_a_missing_ancestor_as_chain_link_missing_naming_the_id() {
    let (_scratch, directory) = chain_dir(
        "load-missing-link",
        &[(
            "orphan",
            patch_doc("orphan", "absent-base", &["move.a keyboard:w"], &[], &[]),
        )],
    );

    let err = load_refusal("a chain with a missing ancestor", &directory, "orphan");
    assert!(
        matches!(err, CoreError::ChainLinkMissing { .. }),
        "a missing link is its own named refusal (A18), got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains("absent-base"),
        "the refusal must name the missing id, got `{detail}`"
    );
    assert!(
        !detail.contains('\n') && !detail.contains('\r'),
        "the refusal is one bounded line: {detail:?}"
    );
}

/// proves: Test strategy #3 + Cross-file loading step 1 (A13, A18) — the head is
/// the path the caller named, not one the walk constructed, so an absent head is
/// the read refusal whose payload carries that path: `ChainLinkMissing` belongs to
/// the completeness of a chain the loader assembled itself, and echoing a raw,
/// grammar-unvalidated file stem as a profile id would name a thing the entry
/// table never accepted.
#[test]
fn refuses_an_absent_head_as_the_read_refusal_carrying_its_path() {
    let (_scratch, directory) = chain_dir("load-absent-head", &[("game-base", base_chain_doc())]);
    let head = directory.join("no-such-head.toml");

    let Err(err) = bindforge_core::load_chain(&head) else {
        panic!("a head that is not in the directory must be refused, not loaded");
    };
    assert!(
        matches!(err, CoreError::Io { .. }),
        "an absent head is the read refusal, whose payload carries the path, got {err:?}"
    );
    let path_text = head.to_str().expect("a scratch path is valid UTF-8");
    assert_eq!(
        err.source_path().unwrap_or_default(),
        path_text,
        "`source_path` is the channel that names which head was missing (A13)"
    );
    let detail = err.to_string();
    assert!(
        !detail.contains(path_text),
        "the path stays out of a refusal that may be shown to whoever sent the \
         document (A13): {detail}"
    );
}

/// proves: Test strategy #3 + Cross-file loading step 2 — the lookup rule's
/// other half: a file whose `id` disagrees with its name (the file the walk
/// constructed from the `derives` element) is `InvalidValue` — the chain's
/// identity is its documents' contents, mirroring `load_manifest`'s
/// id↔directory equality — and the refusal stays path-free (A13).
#[test]
fn refuses_a_link_whose_id_disagrees_with_its_file_name() {
    let (_scratch, directory) = chain_dir(
        "load-id-name-mismatch",
        &[
            ("base-link", base_doc("other-base", &["move.a keyboard:w"])),
            (
                "under-name",
                patch_doc("under-name", "base-link", &["move.b keyboard:e"], &[], &[]),
            ),
        ],
    );

    let err = load_refusal(
        "a file whose id contradicts its name",
        &directory,
        "under-name",
    );
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the id/name mismatch is `InvalidValue`, got {err:?}"
    );
    assert_refusal_names(&err, &["base-link", "other-base"]);
}

/// proves: Test strategy #3 + A5 — no link of a v2 chain resolves outside the
/// head's own major directory: a v1 document named `game-base` sitting in the
/// scratch tree's `v1/` directory is never consulted, because every link is a
/// direct child of the head's `vN/` directory — the chain is single-major by
/// construction (Options: may a v2 chain include a v1 link — refuse).
#[test]
fn resolves_no_link_outside_the_head_major_directory() {
    let scratch = ScratchDir::new("load-mixed-major");
    scratch.write("v1/game-base.toml", &profile_doc("game-base", &[]));
    scratch.write(
        "v2/retune.toml",
        &patch_doc("retune", "game-base", &["move.a keyboard:w"], &[], &[]),
    );

    let err = load_refusal(
        "a chain reaching across majors",
        &scratch.path().join("v2"),
        "retune",
    );
    assert!(
        matches!(err, CoreError::ChainLinkMissing { .. }),
        "the v2 walk never consults the `v1/` directory, got {err:?}"
    );
    assert_refusal_names(&err, &["game-base"]);
}

/// proves: Chosen design (Cross-file loading, step 1) + A5 — the head's declared
/// major must equal the `vN/` directory it loads from: a v2 document sitting in a
/// `v1/` directory refuses as `InvalidValue`, the one refusal every loader in this
/// crate answers for that rule (`schemas/profile/v2/spec.md` → Loading a chain,
/// rule 2; the shipped tables pin the same shape in `reach_v2.rs`). It is *not*
/// the version gate's `UnsupportedVersion`, which stays the refusal for a major
/// the format does not ship at all (`version_gate.rs`).
#[test]
fn refuses_a_head_whose_directory_major_differs_from_its_declared_version() {
    let scratch = ScratchDir::new("load-head-dir-major");
    scratch.write(
        "v1/retune.toml",
        &patch_doc("retune", "game-base", &["move.a keyboard:w"], &[], &[]),
    );

    let err = load_refusal(
        "a v2 head in a v1 directory",
        &scratch.path().join("v1"),
        "retune",
    );
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the directory/declared-major mismatch is the one refusal every loader \
         answers for it, got {err:?}"
    );
    assert_refusal_names(&err, &["retune", "v1", "v2"]);
    let detail = err.to_string();
    assert!(
        !detail.contains(scratch.path().to_str().expect("a scratch path is UTF-8")),
        "the refusal names the major directory, never the local path (A13): {detail}"
    );
}

// ---------------------------------------------------------------------------
// Test strategy #4 — the load-time joins: pin equality, the manifest row
// join, and the preset join, each refusing by name at `resolve`.
// ---------------------------------------------------------------------------

/// proves: Test strategy #4 + Load-time joins (manifest identity) — a link's
/// `for` off by the manifest id refuses as `InvalidValue` naming the link: a
/// chain mid-adoption refuses rather than silently re-resolving "the same"
/// patch over another game's facts (A20's stop-visibly convergence).
#[test]
fn refuses_a_link_whose_pin_names_another_manifest_naming_the_link() {
    let wrong_game = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"wrong-game\""),
        entry("derives", &array(["game-base"])),
        entry("kind", "\"patch\""),
        entry("for", &quoted("other-game@3")),
        entry("set", &array(["move.a keyboard:q"])),
    ]);
    let (_scratch, directory) = chain_dir(
        "join-pin-wrong-id",
        &[("game-base", base_chain_doc()), ("wrong-game", wrong_game)],
    );

    let chain = loaded(&directory, "wrong-game");
    let Err(err) = bindforge_core::resolve(&chain, &manifest()) else {
        panic!("a chain pinned to `other-game@3` must refuse against the `remnant-2@3` manifest");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the pin refusal is `InvalidValue` (Load-time joins), got {err:?}"
    );
    assert_refusal_names(&err, &["wrong-game"]);
}

/// proves: Test strategy #4 — and off by the revision alone: pin equality is
/// id *and* revision (A7), so a chain still naming revision 2 under a
/// revision-3 manifest refuses naming the link, rather than resolving "the
/// same" patch over bumped game facts.
#[test]
fn refuses_a_link_whose_pin_names_another_revision_naming_the_link() {
    let wrong_revision = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"wrong-revision\""),
        entry("derives", &array(["game-base"])),
        entry("kind", "\"patch\""),
        entry("for", &quoted("remnant-2@2")),
        entry("set", &array(["move.a keyboard:q"])),
    ]);
    let (_scratch, directory) = chain_dir(
        "join-pin-wrong-revision",
        &[
            ("game-base", base_chain_doc()),
            ("wrong-revision", wrong_revision),
        ],
    );

    let chain = loaded(&directory, "wrong-revision");
    let Err(err) = bindforge_core::resolve(&chain, &manifest()) else {
        panic!("a chain pinned to `remnant-2@2` must refuse against the `remnant-2@3` manifest");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the pin refusal is `InvalidValue` (Load-time joins), got {err:?}"
    );
    assert_refusal_names(&err, &["wrong-revision"]);
}

/// proves: Test strategy #4 + Load-time joins (action join) + A17 — a delta
/// naming an action that is not a row of the joined manifest refuses naming
/// the id, in any of the three lists: grammar-valid is not joined.
#[test]
fn refuses_a_delta_naming_an_action_that_is_not_a_manifest_row_naming_the_id() {
    for (list, element) in [
        ("set", "ghost.action keyboard:w"),
        ("clear", "ghost.action"),
        ("swap", "ghost.action move.a"),
    ] {
        let patch = match list {
            "set" => patch_doc("ghost-delta", "game-base", &[element], &[], &[]),
            "clear" => patch_doc("ghost-delta", "game-base", &[], &[element], &[]),
            _ => patch_doc("ghost-delta", "game-base", &[], &[], &[element]),
        };
        let (_scratch, directory) = chain_dir(
            "join-action-row",
            &[("game-base", base_chain_doc()), ("ghost-delta", patch)],
        );

        let chain = loaded(&directory, "ghost-delta");
        let Err(err) = bindforge_core::resolve(&chain, &manifest()) else {
            panic!("a `{list}` element naming `ghost.action`, no row of the manifest, must refuse");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "the action-join refusal is `InvalidValue`, got {err:?}"
        );
        assert_refusal_names(&err, &["ghost.action"]);
    }
}

/// proves: Test strategy #4 + Load-time joins (preset join) + A7 — a
/// `seeded-from` that names nothing in the manifest's `presets` refuses
/// naming it: provenance must name a real thing, even though nothing consumes
/// it as trust (ADR Decision Four).
#[test]
fn refuses_a_seeded_from_that_is_not_a_manifest_preset_naming_it() {
    let seeded = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"seeded-base\""),
        entry("kind", "\"base\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(["move.a keyboard:w"])),
        entry("seeded-from", "\"ghost-preset\""),
    ]);
    let (_scratch, directory) = chain_dir("join-preset-absent", &[("seeded-base", seeded)]);

    let chain = loaded(&directory, "seeded-base");
    let Err(err) = bindforge_core::resolve(&chain, &manifest()) else {
        panic!("`ghost-preset` is no preset of the manifest, so the chain must refuse");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the preset-join refusal is `InvalidValue`, got {err:?}"
    );
    assert_refusal_names(&err, &["ghost-preset"]);
}

/// proves: Chosen design (Provenance labels) — the join is a guard, not a
/// cap: a `seeded-from` naming a real preset of the manifest resolves, and the
/// layout answers exactly as if the label were absent — validation stays the
/// only trust mechanism, and provenance is consumed by nothing (Decision
/// Four).
#[test]
fn accepts_a_seeded_from_that_names_a_manifest_preset_and_consumes_it_as_nothing() {
    let seeded = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"seeded-base\""),
        entry("kind", "\"base\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(["move.a keyboard:w"])),
        entry("seeded-from", "\"default\""),
    ]);
    let (_scratch, directory) = chain_dir("join-preset-present", &[("seeded-base", seeded)]);

    let chain = loaded(&directory, "seeded-base");
    assert_eq!(
        layout(&chain, &manifest()),
        pairs(&[("move.a", "keyboard:w")]),
        "the label must not change what resolves"
    );
}

// ---------------------------------------------------------------------------
// Test strategy #5 — the chain operations: `fork`, `diff`, and `merge` as
// operations over documents (core builds documents, not filesystems).
// ---------------------------------------------------------------------------

/// proves: Test strategy #5 + Chain operations (`fork`) — `fork` builds the
/// appended patch: the caller's fresh slug id, `derives = [head id]`, `kind =
/// patch`, the head's `for` inherited so the forked chain loads against the
/// same manifest, and empty delta lists.
#[test]
fn fork_derives_the_head_and_inherits_its_pin() {
    let (_scratch, directory) = chain_dir("fork-shape", &two_link_docs());
    let chain = loaded(&directory, "retune");

    let shell = bindforge_core::fork(&chain, "fresh-fork")
        .expect("forking a loaded chain under a fresh slug id must work");
    assert_eq!(shell.id, "fresh-fork");
    assert!(
        matches!(shell.kind, Some(ProfileKind::Patch)),
        "the fork is a patch, got {:?}",
        shell.kind
    );
    assert_eq!(
        shell.derives,
        words(&["retune"]),
        "the fork derives the chain's head"
    );
    assert_eq!(
        shell.for_manifest, PIN,
        "the head's pin is inherited so the forked chain is loadable against the same manifest"
    );
    assert!(
        sets_of(&shell).is_empty() && clears_of(&shell).is_empty() && swaps_of(&shell).is_empty(),
        "a fresh fork carries no deltas until the caller edits one in"
    );
}

/// proves: Test strategy #5 + Chain operations (`fork`) — a fresh id that
/// collides with any link id in the chain is refused: a chain's links are
/// looked up by id in one directory, and a second link with the same id is the
/// ambiguity the collision rule closes.
#[test]
fn fork_refuses_a_fresh_id_that_collides_with_a_chain_link() {
    let (_scratch, directory) = chain_dir("fork-collision", &two_link_docs());
    let chain = loaded(&directory, "retune");

    for colliding in ["retune", "game-base"] {
        let Err(err) = bindforge_core::fork(&chain, colliding) else {
            panic!("`{colliding}` is already a link id in the chain, so the fork must refuse");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a colliding fork id is `InvalidValue`, got {err:?}"
        );
        assert_refusal_names(&err, &[colliding]);
    }
}

/// proves: Test strategy #5 — a fresh fork id that breaks the slug grammar is
/// refused: the id becomes a file name and a chain lookup key like every other
/// v2 id (A14).
#[test]
fn fork_refuses_a_fresh_id_that_breaks_the_slug_grammar() {
    let (_scratch, directory) = chain_dir("fork-slug", &two_link_docs());
    let chain = loaded(&directory, "retune");

    for id in ["Fresh-Fork", "a/b", ".."] {
        let Err(err) = bindforge_core::fork(&chain, id) else {
            panic!("`{id}` is outside the slug grammar, so the fork must refuse");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a malformed fork id is `InvalidValue`, got {err:?}"
        );
        assert_refusal_names(&err, &[id]);
    }
}

/// proves: Test strategy #5 + Revision log r1 — the fork's empty-delta shell
/// is a document, not a chain: written beside its head it refuses to load as
/// the empty-patch case, and filling one delta element in makes the extended
/// chain load and resolve over the same base (the file-level move the
/// revision log pins for chain operations).
#[test]
fn fork_shell_refuses_to_load_until_one_delta_is_added() {
    let (scratch, directory) = chain_dir("fork-shell-load", &two_link_docs());
    let chain = loaded(&directory, "retune");
    let shell = bindforge_core::fork(&chain, "fresh-fork").expect("fork builds the shell document");

    scratch.write("v2/fresh-fork.toml", &profile_file_text(&shell));
    let err = load_refusal("the fork's empty shell", &directory, "fresh-fork");
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the shell is the vacuous empty patch (`patch-empty-deltas`), got {err:?}"
    );

    scratch.write(
        "v2/fresh-fork.toml",
        &patch_doc("fresh-fork", "retune", &["ui.d keyboard:t"], &[], &[]),
    );
    let extended = loaded(&directory, "fresh-fork");
    let manifest = manifest();
    let mut expected = layout(&chain, &manifest);
    expected.push(("ui.d".to_string(), "keyboard:t".to_string()));
    assert_eq!(
        layout(&extended, &manifest),
        expected,
        "with one delta filled in, the forked chain resolves to the head's layout plus it"
    );
}

/// proves: Test strategy #5 + Chain operations (`diff`) — over two chains that
/// share the same deepest document, `diff` emits the patch from `a` to `b` as
/// a v2 patch document: the caller's id, `derives = [a's head id]`, the joined
/// pin, a `set` per differing action (including actions `b` binds and `a`
/// leaves unbound), and a `clear` per action `b` unbinds — none here.
#[test]
fn diff_emits_the_patch_from_a_to_b() {
    let (_scratch_a, dir_a) = chain_dir(
        "diff-shape-a",
        &[
            ("game-base", base_chain_doc()),
            (
                "a-retune",
                patch_doc(
                    "a-retune",
                    "game-base",
                    &["move.a keyboard:q"],
                    &["aim.c"],
                    &[],
                ),
            ),
        ],
    );
    let (_scratch_b, dir_b) = chain_dir(
        "diff-shape-b",
        &[
            ("game-base", base_chain_doc()),
            (
                "b-retune",
                patch_doc("b-retune", "game-base", &["ui.d keyboard:t"], &[], &[]),
            ),
        ],
    );
    let a = loaded(&dir_a, "a-retune");
    let b = loaded(&dir_b, "b-retune");

    let patch = bindforge_core::diff(&a, &b, &manifest(), "a-to-b")
        .expect("two chains over the same base document must diff");
    assert_eq!(patch.id, "a-to-b");
    assert!(
        matches!(patch.kind, Some(ProfileKind::Patch)),
        "the diff is a v2 patch, got {:?}",
        patch.kind
    );
    assert_eq!(
        patch.derives,
        words(&["a-retune"]),
        "the diff patch hangs off a's head, so appending it extends a"
    );
    assert_eq!(patch.for_manifest, PIN, "the diff carries the joined pin");
    let mut got = sets_of(&patch);
    got.sort();
    assert_eq!(
        got,
        pairs(&[
            ("aim.c", "keyboard:r"),
            ("move.a", "keyboard:w"),
            ("ui.d", "keyboard:t"),
        ]),
        "one `set` per action whose binding differs, `aim.c` re-bound from unbound, and \
         `ui.d` bound for the first time in b"
    );
    assert!(
        clears_of(&patch).is_empty() && swaps_of(&patch).is_empty(),
        "b unbinds nothing a bound, so the diff clears nothing"
    );
}

/// proves: Test strategy #5 — the diff round-trips as data: appending its
/// output to `a` is a file-level move — write it beside a's head, `load_chain`
/// it as the new head — and the result resolves to `resolve(b, manifest)`
/// ("the diff *is* the grammar", Open questions' apply/apply equality opened
/// by the ADR).
#[test]
fn appending_the_diff_output_to_a_resolves_to_bs_layout() {
    let (scratch_a, dir_a) = chain_dir(
        "diff-roundtrip-a",
        &[
            ("game-base", base_chain_doc()),
            (
                "a-retune",
                patch_doc(
                    "a-retune",
                    "game-base",
                    &["move.a keyboard:q"],
                    &["aim.c"],
                    &[],
                ),
            ),
        ],
    );
    let (_scratch_b, dir_b) = chain_dir(
        "diff-roundtrip-b",
        &[
            ("game-base", base_chain_doc()),
            (
                "b-retune",
                patch_doc("b-retune", "game-base", &["ui.d keyboard:t"], &[], &[]),
            ),
        ],
    );
    let manifest = manifest();
    let a = loaded(&dir_a, "a-retune");
    let b = loaded(&dir_b, "b-retune");

    let patch = bindforge_core::diff(&a, &b, &manifest, "a-to-b").expect("chains must diff");
    scratch_a.write("v2/a-to-b.toml", &profile_file_text(&patch));

    let moved = loaded(&dir_a, "a-to-b");
    assert_eq!(
        layout(&moved, &manifest),
        layout(&b, &manifest),
        "a's chain with the diff appended must resolve exactly to b's layout"
    );
}

/// proves: Test strategy #5 + Chain operations (`diff`) — two chains whose
/// deepest links are not the same document refuse naming both heads: by id,
/// and by contents with equal ids (ids are unique only within one directory,
/// and chain identity is the documents' contents).
#[test]
fn diff_refuses_chains_whose_deepest_links_differ_naming_both_heads() {
    let (_scratch_a, dir_a) = chain_dir("diff-base-a", &two_link_docs());
    let (_scratch_b, dir_b) = chain_dir(
        "diff-base-id-b",
        &[
            (
                "other-base",
                base_doc(
                    "other-base",
                    &["move.a keyboard:w", "move.b keyboard:e", "aim.c keyboard:r"],
                ),
            ),
            (
                "b-retune",
                patch_doc("b-retune", "other-base", &["ui.d keyboard:t"], &[], &[]),
            ),
        ],
    );
    let (_scratch_c, dir_c) = chain_dir(
        "diff-base-content-c",
        &[
            ("game-base", base_doc("game-base", &["move.a keyboard:w"])),
            (
                "b-retune",
                patch_doc("b-retune", "game-base", &["ui.d keyboard:t"], &[], &[]),
            ),
        ],
    );

    for (label, dir_b, head_b) in [
        ("by id", &dir_b, "b-retune"),
        ("by contents", &dir_c, "b-retune"),
    ] {
        let a = loaded(&dir_a, "retune");
        let b = loaded(dir_b, head_b);
        let Err(err) = bindforge_core::diff(&a, &b, &manifest(), "a-to-b") else {
            panic!("chains whose deepest links differ {label} must refuse to diff");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "the base mismatch is `InvalidValue`, got {err:?}"
        );
        let detail = err.to_string();
        assert!(
            detail.contains("retune") && detail.contains("b-retune"),
            "the refusal must name both heads it refused, got `{detail}`"
        );
        assert!(
            !detail.contains('\n') && !detail.contains('\r'),
            "the refusal is one bounded line: {detail:?}"
        );
    }
}

/// proves: Test strategy #5 + Chain operations (`diff`) — chains resolving to
/// equal layouts refuse to diff naming both heads: there is no legal non-empty
/// patch between them, and the empty patch is the vacuous link the grammar
/// refuses.
#[test]
fn diff_refuses_chains_with_equal_resolved_layouts_naming_both_heads() {
    let (_scratch_a, dir_a) = chain_dir("diff-equal-a", &two_link_docs());
    let (_scratch_b, dir_b) = chain_dir(
        "diff-equal-b",
        &[
            ("game-base", base_chain_doc()),
            (
                "b-twin",
                patch_doc(
                    "b-twin",
                    "game-base",
                    &["move.a keyboard:q"],
                    &["aim.c"],
                    &[],
                ),
            ),
        ],
    );
    let a = loaded(&dir_a, "retune");
    let b = loaded(&dir_b, "b-twin");

    let Err(err) = bindforge_core::diff(&a, &b, &manifest(), "a-to-b") else {
        panic!("chains with equal resolved layouts must refuse to diff");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the equal-layouts refusal is `InvalidValue`, got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains("retune") && detail.contains("b-twin"),
        "the refusal must name both heads, got `{detail}`"
    );
}

/// proves: Revision log r1 — `diff`'s fresh id obeys `fork`'s collision rule
/// against every link id of `a`: a diff output reusing one of them could never
/// be appended to `a` in the single-major directory.
#[test]
fn diff_refuses_a_fresh_id_colliding_with_a_link_of_a() {
    let (_scratch_a, dir_a) = chain_dir("diff-collision-a", &two_link_docs());
    let (_scratch_b, dir_b) = chain_dir(
        "diff-collision-b",
        &[
            ("game-base", base_chain_doc()),
            (
                "b-retune",
                patch_doc("b-retune", "game-base", &["ui.d keyboard:t"], &[], &[]),
            ),
        ],
    );
    let a = loaded(&dir_a, "retune");
    let b = loaded(&dir_b, "b-retune");

    for colliding in ["retune", "game-base"] {
        let Err(err) = bindforge_core::diff(&a, &b, &manifest(), colliding) else {
            panic!("`{colliding}` is a link id of a, so the diff output must refuse it");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a colliding diff id is `InvalidValue`, got {err:?}"
        );
        assert_refusal_names(&err, &[colliding]);
    }
}

/// proves: Test strategy #5 + Chain operations (`merge`) — merging two
/// compatible patches emits one patch with the caller's id, `derives` and
/// `for` inherited from the first, and the composed deltas in the
/// deterministic element order: first-patch touch order, then
/// second-patch-only actions, equal sets emitted once.
#[test]
fn merge_composes_compatible_patches_into_one_patch() {
    let (p1_text, p2_text) = merge_operands(
        &["move.a keyboard:w"],
        &["aim.c"],
        &[],
        &["move.a keyboard:w", "move.b keyboard:e"],
        &[],
        &[],
    );
    let p1 = parse_ok(&p1_text);
    let p2 = parse_ok(&p2_text);

    let merged = bindforge_core::merge(profile_in(&p1), profile_in(&p2), "merged-patch")
        .expect("compatible patches over one base must merge");
    assert_eq!(merged.id, "merged-patch");
    assert!(
        matches!(merged.kind, Some(ProfileKind::Patch)),
        "the merge output is a patch, got {:?}",
        merged.kind
    );
    assert_eq!(
        merged.derives,
        words(&["game-base"]),
        "`derives` is inherited from the first patch"
    );
    assert_eq!(
        merged.for_manifest, PIN,
        "`for` is inherited from the first patch"
    );
    assert_eq!(
        sets_of(&merged),
        pairs(&[("move.a", "keyboard:w"), ("move.b", "keyboard:e")]),
        "the equal `set` emits once, in first-patch touch order, then the \
         second-patch-only action"
    );
    assert_eq!(clears_of(&merged), words(&["aim.c"]));
    assert!(swaps_of(&merged).is_empty());
}

/// proves: Test strategy #5 + Chain operations (`merge`) — a composition whose
/// net effect keeps an exchange is a compatible overlap, not a conflict: `p1`
/// exchanges two actions the base binds, `p2` re-binds an action neither touches,
/// and the merge emits one patch carrying `p2`'s `set` plus `p1`'s exchange —
/// each kept pair written once, not refused against its own partner. Written
/// beside the base and `load_chain`ed, that patch resolves to exactly the layout
/// the two-patch chain resolves to ("Compatible overlaps are not conflicts"; the
/// deterministic element order is first-patch touch order, then second-patch-only
/// actions).
#[test]
fn merge_keeps_an_exchange_the_second_patch_does_not_touch() {
    let (p1_text, p2_text) = merge_operands(
        &[],
        &[],
        &["move.a move.b"],
        &["aim.c keyboard:t"],
        &[],
        &[],
    );
    let p1 = parse_ok(&p1_text);
    let p2 = parse_ok(&p2_text);

    let merged = bindforge_core::merge(profile_in(&p1), profile_in(&p2), "merged-swap")
        .expect("an exchange under an untouched action is compatible, not a conflict");
    assert_eq!(
        sets_of(&merged),
        pairs(&[("aim.c", "keyboard:t")]),
        "`p2`'s element is the merge's only `set`"
    );
    assert!(
        clears_of(&merged).is_empty(),
        "nothing is unbound, so nothing is cleared"
    );
    assert_eq!(
        swaps_of(&merged),
        pairs(&[("move.a", "move.b")]),
        "the kept exchange is emitted once, in `p1`'s touch order"
    );

    // The merged patch *is* the composition: as a file beside the base it resolves
    // exactly as the stacked chain `game-base ← p-one ← p-two` does.
    let merged_text = profile_file_text(&merged);
    let stacked_text = patch_doc("p-two", "p-one", &["aim.c keyboard:t"], &[], &[]);
    let (_scratch_merged, merged_dir) = chain_dir(
        "merge-swap-outcome",
        &[
            ("game-base", base_chain_doc()),
            ("merged-swap", merged_text),
        ],
    );
    let (_scratch_stack, stack_dir) = chain_dir(
        "merge-swap-stack",
        &[
            ("game-base", base_chain_doc()),
            ("p-one", p1_text),
            ("p-two", stacked_text),
        ],
    );
    let manifest = manifest();
    let merged_layout = layout(&loaded(&merged_dir, "merged-swap"), &manifest);
    let stacked_layout = layout(&loaded(&stack_dir, "p-two"), &manifest);
    assert_eq!(
        merged_layout, stacked_layout,
        "one merged patch must resolve over the base to the two-patch chain's layout"
    );
    assert_eq!(
        merged_layout,
        pairs(&[
            ("move.a", "keyboard:e"),
            ("move.b", "keyboard:w"),
            ("aim.c", "keyboard:t"),
        ]),
        "the exchange moved the base's two bindings, `p2` re-bound `aim.c`, and the \
         output follows the manifest's row order (determinism rules 2, 3 and 5)"
    );
}

/// proves: Test strategy #5 + Chain operations (`merge`) + Provenance labels — no
/// provenance label rides the merged document. The merge contract pins the
/// caller's `id`, the inherited `derives`/`for`, and the composed delta lists, and
/// nothing else; `generated-by` names what *produced the document*, so carrying an
/// operand's label onto the result — even one both operands agree on — states a
/// fact neither operand asserted for the merge, in a label the design insists is
/// never read as authority (ADR Decision Four).
#[test]
fn merge_leaves_the_provenance_labels_absent_from_its_output() {
    let p1 = parse_ok(&doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"p-one\""),
        entry("derives", &array(["game-base"])),
        entry("kind", "\"patch\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(["move.a keyboard:w"])),
        entry("seeded-from", "\"default\""),
        entry("generated-by", "\"human\""),
    ]));
    let p2 = parse_ok(&doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"p-two\""),
        entry("derives", &array(["game-base"])),
        entry("kind", "\"patch\""),
        entry("for", &quoted(PIN)),
        entry("set", &array(["move.b keyboard:e"])),
        entry("seeded-from", "\"default\""),
        entry("generated-by", "\"human\""),
    ]));

    let merged = bindforge_core::merge(profile_in(&p1), profile_in(&p2), "merged-labels")
        .expect("two patches over one base and one pin compose into a single patch");
    assert_eq!(
        sets_of(&merged),
        pairs(&[("move.a", "keyboard:w"), ("move.b", "keyboard:e")]),
        "the composition is what merge emits — the labels are not part of it"
    );
    assert_eq!(
        merged.seeded_from, None,
        "`seeded-from` describes a chain's base, not the document merge produced"
    );
    assert_eq!(
        merged.generated_by, None,
        "the merge result claims no producer, not even one both operands shared \
         (ADR Decision Four)"
    );
}

/// proves: Test strategy #5 + Chain operations (`merge`) — a `set` under a
/// `clear` composes to the lone `clear` (compatible overlap, not a conflict):
/// the merged patch carries one `clear` for that action and no `set`.
#[test]
fn merge_composes_a_set_under_a_clear_as_the_lone_clear() {
    let (p1_text, p2_text) = merge_operands(
        &["move.a keyboard:w"],
        &[],
        &[],
        &["move.b keyboard:e"],
        &["move.a"],
        &[],
    );
    let p1 = parse_ok(&p1_text);
    let p2 = parse_ok(&p2_text);

    let merged = bindforge_core::merge(profile_in(&p1), profile_in(&p2), "merged-patch")
        .expect("a set under a clear is compatible, not a conflict");
    assert_eq!(sets_of(&merged), pairs(&[("move.b", "keyboard:e")]));
    assert_eq!(clears_of(&merged), words(&["move.a"]));
    assert!(swaps_of(&merged).is_empty());
}

/// proves: Test strategy #5 + Chain operations (`merge`) — a composition whose
/// effect has no one-patch form under the fixed list order refuses the whole
/// merge as `InvalidValue` naming the colliding action id: lower `swap X Y`
/// with both sides bound, upper `clear X` — the net needs the `swap` before
/// the `clear`, and the fixed order says `clear` runs first. No markers, no
/// guessing (Options: merge's conflict rule, A20).
#[test]
fn merge_refuses_a_swap_under_a_clear_naming_the_colliding_action() {
    let (p1_text, p2_text) = merge_operands(
        &[],
        &[],
        &["move.a move.b"],
        &["ui.d keyboard:t"],
        &["move.a"],
        &[],
    );
    let p1 = parse_ok(&p1_text);
    let p2 = parse_ok(&p2_text);

    let Err(err) = bindforge_core::merge(profile_in(&p1), profile_in(&p2), "merged-patch") else {
        panic!(
            "a `swap` under a `clear` of one swapped action has no one-patch form and must refuse"
        );
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the conflict refusal is `InvalidValue`, got {err:?}"
    );
    assert_refusal_names(&err, &["move.a"]);
}

/// proves: Test strategy #5 + Revision log r1 — a composition that cancels to
/// the identity refuses as the empty-patch case: two patches each swapping the
/// same pair compose to a patch with no delta at all, the vacuous link the
/// grammar refuses.
#[test]
fn merge_refuses_an_identity_cancelling_composition_as_the_empty_patch() {
    let (p1_text, p2_text) =
        merge_operands(&[], &[], &["move.a move.b"], &[], &[], &["move.a move.b"]);
    let p1 = parse_ok(&p1_text);
    let p2 = parse_ok(&p2_text);

    let Err(err) = bindforge_core::merge(profile_in(&p1), profile_in(&p2), "merged-patch") else {
        panic!("two identical swaps compose to the identity and must refuse as the empty patch");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the empty-patch-case refusal is `InvalidValue`, got {err:?}"
    );
    assert!(
        !err.to_string().contains('\n') && !err.to_string().contains('\r'),
        "the refusal is one bounded line: {err:?}"
    );
}

/// proves: Test strategy #5 + Chain operations (`merge`) — `merge` takes two
/// patches naming the same `derives` base; patches over different bases refuse
/// naming the mismatch: composing over two different bases answers "on which
/// base?" twice.
#[test]
fn merge_refuses_patches_over_different_bases_naming_the_mismatch() {
    let p1 = parse_ok(&patch_doc(
        "p-one",
        "game-base",
        &["move.a keyboard:w"],
        &[],
        &[],
    ));
    let p2 = parse_ok(&patch_doc(
        "p-two",
        "other-base",
        &["move.b keyboard:e"],
        &[],
        &[],
    ));

    let Err(err) = bindforge_core::merge(profile_in(&p1), profile_in(&p2), "merged-patch") else {
        panic!("patches over different bases must refuse to merge");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the base mismatch is `InvalidValue`, got {err:?}"
    );
    assert_refusal_names(&err, &["other-base", "game-base"]);
}

/// proves: Test strategy #5 — and patches pinned to different manifest
/// revisions refuse naming the mismatch: the pin equality rule, cross-
/// referenced from the load-time joins.
#[test]
fn merge_refuses_patches_pinned_apart_naming_the_mismatch() {
    let p1 = parse_ok(&patch_doc(
        "p-one",
        "game-base",
        &["move.a keyboard:w"],
        &[],
        &[],
    ));
    let p2_off_pin = doc(&[
        entry("format", "\"profile\""),
        entry("version", "\"v2\""),
        entry("id", "\"p-two\""),
        entry("derives", &array(["game-base"])),
        entry("kind", "\"patch\""),
        entry("for", &quoted("remnant-2@2")),
        entry("set", &array(["move.b keyboard:e"])),
    ]);
    let p2 = parse_ok(&p2_off_pin);

    let Err(err) = bindforge_core::merge(profile_in(&p1), profile_in(&p2), "merged-patch") else {
        panic!("patches pinned apart must refuse to merge");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the pin mismatch is `InvalidValue`, got {err:?}"
    );
    assert_refusal_names(&err, &["remnant-2@2"]);
}

/// proves: Revision log r1 — `merge`'s fresh id obeys `fork`'s collision rule
/// against `p1` and `p2`: the merged patch replaces them over their base, and
/// reusing either id would leave two answers for one id.
#[test]
fn merge_refuses_a_fresh_id_colliding_with_either_patch() {
    let (p1_text, p2_text) =
        merge_operands(&["move.a keyboard:w"], &[], &[], &[], &["move.b"], &[]);
    let p1 = parse_ok(&p1_text);
    let p2 = parse_ok(&p2_text);

    for colliding in ["p-one", "p-two"] {
        let Err(err) = bindforge_core::merge(profile_in(&p1), profile_in(&p2), colliding) else {
            panic!("`{colliding}` is one of the operand ids, so the merge must refuse it");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a colliding merge id is `InvalidValue`, got {err:?}"
        );
        assert_refusal_names(&err, &[colliding]);
    }
}

// ---------------------------------------------------------------------------
// Test strategy #6 — the round-trip, delta array order as data, and the
// untouched v1 writer.
// ---------------------------------------------------------------------------

/// proves: Test strategy #6 + The model (the `serialize` branch) —
/// `parse(serialize(parse(text)))` equals the first parse for a hand-built
/// document carrying all eleven entries, with the delta arrays deliberately
/// unsorted and repeated: delta order is data, so the writer preserves each
/// array's document order and the harness's comparison is equality (A3).
#[test]
fn round_trips_a_v2_profile_carrying_every_defined_entry() {
    let text = doc_with_entries(&[
        (
            "set",
            array([
                "move.b keyboard:e",
                "move.a keyboard:w",
                "move.b keyboard:w",
            ]),
        ),
        ("swap", array(["move.b move.a"])),
    ]);

    let first = parse_ok(&text);
    let again = bindforge_core::parse(&bindforge_core::serialize(&first))
        .expect("a serialized v2 profile must re-parse");
    assert_eq!(
        again, first,
        "serializing and reading back must yield a semantically equal value"
    );
    assert_eq!(
        sets_of(profile_in(&again)),
        pairs(&[
            ("move.b", "keyboard:e"),
            ("move.a", "keyboard:w"),
            ("move.b", "keyboard:w"),
        ]),
        "the `set` array comes back in document order, repeats included — never a \
         canonicalized one"
    );
    assert_eq!(swaps_of(profile_in(&again)), pairs(&[("move.b", "move.a")]));
}

/// proves: Test strategy #6 + A15 — the v1 writer stays exactly as shipped:
/// a v1 `Profile` round-trips through `serialize` to a semantically equal
/// value, and the serialized text gains none of the v2-only entries — the
/// writer branches on `version` alone, and v1's bytes stay v1's bytes.
#[test]
fn keeps_the_v1_serialize_bytes_free_of_the_v2_entries() {
    let text = profile_doc("legacy-v1", &["ancestor-1".to_string()]);
    let first = bindforge_core::parse(&text)
        .expect("the shipped v1 profile shape parses today and must keep parsing");

    let written = bindforge_core::serialize(&first);
    for v2_entry in [
        "kind",
        "for",
        "set",
        "clear",
        "swap",
        "seeded-from",
        "generated-by",
    ] {
        assert!(
            !written.contains(&format!("{v2_entry} = ")),
            "v1's serialize output gains no `{v2_entry}` entry (A15); it wrote: {written}"
        );
    }

    let again = bindforge_core::parse(&written)
        .expect("a serialized v1 profile must re-parse under the v1 entry set");
    assert_eq!(
        again, first,
        "v1's round-trip keeps its shipped behavior (A15)"
    );
}
