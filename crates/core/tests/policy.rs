//! The `policy/v1` grammar, the `Policy` model, and `load_policy` — the
//! module-mirrored half of ADR-015 in `crates/core` (ADR-004: this file is the
//! unit test of `crates/core/src/policy.rs`, and the `document.rs` policy arms
//! it drives).
//!
//! proves: design 015's Test strategy #1 — a document using exactly the defined
//! entries parses, and each invalid shape the contract names refuses naming the
//! entry, row, or field: an unknown transform kind, an empty row, a
//! `relocate-cluster` row under six fields or past `MAX_RELOCATE_TARGETS`
//! targets, a hand outside `{left, right}`, a bound that is a letter or past the
//! v2 scale, a target outside the closed input vocabulary, an extra field on
//! `swap-polarity`, any argument on `swap-mouse-buttons`, the four malformed-tag
//! shapes, a missing / empty / past-cap `transforms` array, and an uppercase or
//! past-the-shared-slug-bound `id` (A1, A15, A19, A22, A24).
//!
//! proves: design 015's Test strategy #2 — `parse(serialize(parse(text)))`
//! equals the first parse for every valid policy document: the `transforms`
//! array order survives ("order is data") and each row comes back re-spelled
//! from its typed fields in the contract's canonical spelling (A12).
//!
//! proves: design 015's Test strategy #3 — `load_policy` mirrors
//! `load_geometry`'s discipline: the `id` is slug-checked *before* any
//! filesystem question, a file whose stem contradicts its `id` entry refuses, a
//! document whose declared major differs from its `vN/` directory refuses, and a
//! non-policy document at the policy path refuses on the format arm — every
//! refusal an existing `CoreError` variant, one bounded line, no local path
//! (A9, A11, A14, A15).
//!
//! Four decisions bound the suite, and all four are the proposal's:
//!
//! - **Parse level, not load level.** Every grammar rule refuses at
//!   `bindforge_core::parse` alone, which is what keeps
//!   `schemas/policy/v1/fixtures/` single-document like every other format's
//!   (A12). Only the location rules wait for `load_policy`, and they run over
//!   real files in a scratch directory (repo practice: unit tests, temp
//!   directories, no network).
//! - **No new `CoreError` variant.** A row-level fault is `InvalidValue`, a
//!   missing required entry is `MissingEntry`, the directory-major rule is the
//!   loader's `InvalidValue`, and the undeclared-major rule is the shipped
//!   version gate (A9). The suite pins the variant, never the sentence around
//!   it, and pins the one-line, escaped, truncating, path-free message shape
//!   (A14).
//! - **The budgets are named constants.** `MAX_POLICY_TRANSFORMS` and
//!   `MAX_RELOCATE_TARGETS` are read from `bindforge_core`, never as literals,
//!   so a cap change moves the boundary with it (A13).
//! - **Registry membership is not grammar.** A tag is grammar-checked at parse;
//!   whether it matches a loaded id is an application fact `crates/transforms`
//!   answers (`apply.rs`, Test strategy #4). So the grammar cases here use tags
//!   that name no shipped registry id, and the matching cases call
//!   `TagPattern::matches` directly.
//!
//! The corpus itself — `schemas/policy/v1/spec.md` plus the fixtures the
//! contract names — is what makes `policy/v1` *supported* (`schemas/README.md`);
//! this file pins that the named files exist, and `conformance.rs` (Test
//! strategy #9) is what asserts the walk's behavior over them.

mod common;

use bindforge_core::{ActionId, CoreError, Document, Hand, Policy, TagPattern, Transform};
use common::{ScratchDir, quoted_array, version_dir};

/// The six valid fixture names `schemas/policy/v1/` ships, and the thirty
/// invalid ones (Chosen design → The contract). The names are contract: a
/// third-party reader is held to the same corpus by the walk.
const VALID_FIXTURE_NAMES: [&str; 6] = [
    "minimal-policy",
    "polarity-and-mouse",
    "cluster-to-the-right-hand",
    "lefty-stack",
    "same-tag-stack",
    "exact-tag-policy",
];

const INVALID_FIXTURE_NAMES: [&str; 30] = [
    "unknown-entry",
    "major-not-shipped",
    "missing-version",
    "missing-id",
    "missing-transforms",
    "transforms-empty",
    "transforms-past-cap",
    "transform-unknown-kind",
    "transform-empty-row",
    "relocate-few-fields",
    "relocate-target-past-cap",
    "relocate-bad-hand",
    "relocate-bound-letter",
    "relocate-bound-past-scale",
    "relocate-target-unknown-input",
    "swap-polarity-extra-field",
    "mouse-row-carries-tag",
    "tag-embedded-glob",
    "tag-uppercase",
    "tag-empty-prefix",
    "tag-three-segments",
    "id-uppercase",
    "id-past-cap",
    "forged-log-line",
    "table-header",
    "dotted-key",
    "multi-line-array",
    "float-value",
    "hex-integer",
    "nesting-past-limit",
];

/// The four defined entries, each with a fault-free value: the document every
/// grammar case below edits exactly one entry of.
fn complete_entries() -> Vec<(String, String)> {
    vec![
        ("format".to_string(), "format = \"policy\"".to_string()),
        ("version".to_string(), "version = \"v1\"".to_string()),
        ("id".to_string(), "id = \"full-policy\"".to_string()),
        (
            "transforms".to_string(),
            "transforms = [\"relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k\"]"
                .to_string(),
        ),
    ]
}

/// The defined four with these entries replaced (an undefined entry is
/// appended), so every value-level case is exactly the fault listed.
fn policy_with_entries(edits: &[(&str, String)]) -> String {
    let mut entries: Vec<(String, String)> = complete_entries();
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

/// The defined four with one entry edited.
fn policy_with_entry(name: &str, value: &str) -> String {
    policy_with_entries(&[(name, value.to_string())])
}

/// The defined four with one entry edited to a one-line string array of these
/// rows — the row-level cases, where the stack itself is the fault.
fn policy_with_row(row: &str) -> String {
    policy_with_entry("transforms", &format!("[\"{row}\"]"))
}

/// A stack of exactly these rows.
fn policy_with_rows(rows: &[&str]) -> String {
    let list = rows
        .iter()
        .map(|row| format!("\"{row}\""))
        .collect::<Vec<_>>()
        .join(", ");
    policy_with_entry("transforms", &format!("[{list}]"))
}

/// The defined four with these entries dropped, for the required-entry cases.
fn policy_without_entries(names: &[&str]) -> String {
    let entries: Vec<(String, String)> = complete_entries()
        .into_iter()
        .filter(|(name, _)| !names.contains(&name.as_str()))
        .collect();
    entries
        .iter()
        .map(|(_, line)| format!("{line}\n"))
        .collect()
}

/// A policy document under its own `id`, its `transforms` exactly these rows.
fn policy_doc(id: &str, rows: &[&str]) -> String {
    let mut text = format!("format = \"policy\"\nversion = \"v1\"\nid = \"{id}\"\ntransforms = [");
    let list = rows
        .iter()
        .map(|row| format!("\"{row}\""))
        .collect::<Vec<_>>()
        .join(", ");
    text.push_str(&list);
    text.push_str("]\n");
    text
}

/// The stacks every accept-and-round-trip case reads: the six shapes the valid
/// corpus names, spelled here so the grammar and the round-trip are pinned
/// against documents this suite can show in full.
fn valid_stacks() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("minimal-policy", vec!["swap-mouse-buttons"]),
        (
            "polarity-and-mouse",
            vec!["swap-polarity move.steer-*", "swap-mouse-buttons"],
        ),
        (
            "cluster-to-the-right-hand",
            vec!["relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k"],
        ),
        (
            "lefty-stack",
            vec![
                "relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k",
                "swap-polarity move.strafe-*",
                "swap-mouse-buttons",
            ],
        ),
        (
            "same-tag-stack",
            vec![
                "relocate-cluster move.steer-* left left 0 keyboard:s keyboard:d",
                "swap-polarity move.steer-*",
            ],
        ),
        (
            "exact-tag-policy",
            vec![
                "swap-polarity move.steer-left",
                "relocate-cluster move.strafe-right left right 4 keyboard:i",
            ],
        ),
    ]
}

/// The contract's rejection shape, pinned once: one bounded line, control-free
/// (`schemas/README.md`, A14).
fn assert_message_shape(err: &CoreError) {
    let detail = err.to_string();
    assert!(
        !detail.contains('\n') && !detail.contains('\r'),
        "a refusal is one bounded line: {detail:?}"
    );
    assert!(
        detail.chars().all(|ch| !ch.is_control()),
        "a refusal escapes control characters rather than emitting them: {detail:?}"
    );
}

/// The refusal `text` must be: it does not parse, and the fault is
/// value-level — `InvalidValue`, one of the shipped variants, since this design
/// adds none (A9).
fn value_refusal(text: &str) -> CoreError {
    let Err(err) = bindforge_core::parse(text) else {
        panic!("this document is an invalid policy case and must be refused, not parsed: {text}");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "a policy row-level fault is `InvalidValue` (A9: no new `CoreError` variant), got {err:?}"
    );
    assert_message_shape(&err);
    err
}

/// The refusal for an absent required entry: `MissingEntry`, naming the entry.
fn missing_entry_refusal(text: &str, entry_name: &str) {
    let Err(err) = bindforge_core::parse(text) else {
        panic!("a policy document with no `{entry_name}` entry must be refused");
    };
    assert!(
        matches!(err, CoreError::MissingEntry { .. }),
        "an absent required entry is `MissingEntry` (A9), got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains(entry_name),
        "the refusal must name the entry it found missing, got `{detail}`"
    );
    assert_message_shape(&err);
}

/// The refusal names one of `needles` — the offender, the row, or the entry it
/// refused. Which sentence names it is not part of the contract.
fn assert_refusal_names(err: &CoreError, needles: &[&str]) {
    let detail = err.to_string();
    assert!(
        needles.iter().any(|needle| detail.contains(needle)),
        "the refusal must name what it refused, one of {needles:?}, got `{detail}`"
    );
    assert_message_shape(err);
}

/// The `Policy` a parsed policy document carries.
fn policy_of(text: &str) -> Policy {
    let document = bindforge_core::parse(text)
        .unwrap_or_else(|err| panic!("a document the entry table defines must parse: {err}"));
    match document {
        Document::Policy(policy) => policy,
        _ => panic!("a `format = \"policy\"` document must parse into the policy arm (The model)"),
    }
}

/// A row's kind as the contract spells it — the same spelling
/// `crates/transforms` puts on a `TransformReport`.
fn kind_of(row: &Transform) -> &'static str {
    match row {
        Transform::RelocateCluster { .. } => "relocate-cluster",
        Transform::SwapPolarity { .. } => "swap-polarity",
        Transform::SwapMouseButtons => "swap-mouse-buttons",
    }
}

/// The tag a parsed row carries, for the matching cases.
fn tag_of(row: &Transform) -> &TagPattern {
    match row {
        Transform::RelocateCluster { tags, .. } | Transform::SwapPolarity { tags } => tags,
        Transform::SwapMouseButtons => panic!("the probe row carries a tag"),
    }
}

/// An action id, or a test failure naming the spelling.
fn action(raw: &str) -> ActionId {
    ActionId::parse(raw).unwrap_or_else(|err| panic!("`{raw}` is a well-formed id: {err}"))
}

/// The `transforms` array a serialized document carries, in written order —
/// how the round-trip case reads "order is data" without a second TOML parser.
fn written_rows(text: &str) -> Option<Vec<String>> {
    quoted_array(text, "transforms")
}

/// proves: design 015's Test strategy #1 + Chosen design → The contract —
/// closing the entry set is not shrinking the grammar: a policy using exactly
/// `format`, `version`, `id`, and `transforms` parses, one row or three, a
/// prefix tag or an exact tag.
#[test]
fn accepts_a_policy_document_using_exactly_the_defined_entries() {
    for (id, rows) in valid_stacks() {
        let text = policy_doc(id, &rows);
        if let Err(err) = bindforge_core::parse(&text) {
            panic!(
                "`{id}` uses only the entries and rows the grammar defines; it was refused: {err}"
            );
        }
    }
}

/// proves: design 015's Test strategy #1 (`transform-unknown-kind.toml`) +
/// Options considered (a closed set compiled into core) — a row naming a fourth
/// kind is a parse-level refusal naming the row: an open name space would make
/// an unresolved name somebody else's semantics, the free-form-predicate risk
/// one layer up (A23).
#[test]
fn refuses_a_transform_row_naming_an_unknown_kind_naming_the_row() {
    let err = value_refusal(&policy_with_row(
        "resize-cluster move.steer-* left right 2 keyboard:i keyboard:k",
    ));
    assert_refusal_names(&err, &["resize-cluster"]);
}

/// proves: design 015's Test strategy #1 (`transform-empty-row.toml`) — a row
/// that names no kind at all is a refusal, not a no-op row: an empty row read
/// as "nothing to do" is the silent no-op the ADR refuses.
#[test]
fn refuses_an_empty_transform_row() {
    // An empty row has no offender to name — it carries nothing to quote — so
    // this case pins the refusal and the message shape only.
    value_refusal(&policy_with_row(""));
}

/// proves: design 015's Test strategy #1 (`relocate-few-fields.toml`) + A22 —
/// a `relocate-cluster` row is its six fixed fields plus targets: a row missing
/// the bound, or missing the hands, is a refusal naming the row, never a
/// default for the field it left out.
#[test]
fn refuses_a_relocate_cluster_row_with_too_few_fields() {
    for row in [
        "relocate-cluster move.steer-* left right",
        "relocate-cluster move.steer-* left right keyboard:i",
        "relocate-cluster move.steer-*",
    ] {
        let err = value_refusal(&policy_with_row(row));
        assert_refusal_names(&err, &["relocate-cluster"]);
    }
}

/// proves: design 015's Test strategy #1 (`relocate-target-past-cap.toml`) +
/// A13 — the target list is bounded by `MAX_RELOCATE_TARGETS`, read as the
/// constant, with the off-by-one wall pinned: the cap itself parses, one past
/// it refuses naming the row.
#[test]
fn refuses_a_relocate_cluster_row_past_max_relocate_targets() {
    let glyphs: Vec<String> = ('a'..='z')
        .take(bindforge_core::MAX_RELOCATE_TARGETS)
        .map(|letter| format!("keyboard:{letter}"))
        .collect();
    assert_eq!(
        glyphs.len(),
        bindforge_core::MAX_RELOCATE_TARGETS,
        "the closed vocabulary has more members than the target cap, so this case reaches the cap \
         and not a vocabulary fault"
    );
    let at_cap = format!(
        "relocate-cluster move.steer-* left right 0 {}",
        glyphs.join(" ")
    );
    if let Err(err) = bindforge_core::parse(&policy_with_row(&at_cap)) {
        panic!(
            "a `relocate-cluster` row with exactly MAX_RELOCATE_TARGETS ({}) targets must parse; \
             it was refused: {err}",
            bindforge_core::MAX_RELOCATE_TARGETS
        );
    }

    let mut past = glyphs;
    past.push("keyboard:q".to_string());
    let err = value_refusal(&policy_with_row(&format!(
        "relocate-cluster move.steer-* left right 0 {}",
        past.join(" ")
    )));
    assert_refusal_names(&err, &["relocate-cluster"]);
}

/// proves: design 015's Test strategy #1 (`relocate-bad-hand.toml`) + A4 — the
/// hands are the shipped grammar, `left` or `right`, and they stay *inputs* the
/// row declares: a hand outside the pair is a refusal naming the row, never a
/// hand derived from data.
#[test]
fn refuses_a_hand_outside_the_shipped_pair() {
    for hand in ["lefts", "both", "LEFT", "none"] {
        let err = value_refusal(&policy_with_row(&format!(
            "relocate-cluster move.steer-* {hand} right 2 keyboard:i"
        )));
        assert_refusal_names(&err, &["relocate-cluster", hand]);
    }
}

/// proves: design 015's Test strategy #1 (`relocate-bound-letter.toml`,
/// `relocate-bound-past-scale.toml`) + A24 — the bound is a decimal
/// `0..=4`, because the v2 taxonomy's five severity classes make four the
/// largest possible one-action degradation: a letter, a negative, and `5` are
/// refusals, and both ends of the scale parse.
#[test]
fn refuses_a_bound_outside_the_grade_scale() {
    for bound in ["x", "-1", "5", "0.5", "four"] {
        let err = value_refusal(&policy_with_row(&format!(
            "relocate-cluster move.steer-* left right {bound} keyboard:i"
        )));
        assert_refusal_names(&err, &["relocate-cluster", bound]);
    }

    for bound in ["0", "4"] {
        let row = format!("relocate-cluster move.steer-* left right {bound} keyboard:i");
        if let Err(err) = bindforge_core::parse(&policy_with_row(&row)) {
            panic!("bound `{bound}` is inside the v2 scale and must parse; it was refused: {err}");
        }
    }
}

/// proves: design 015's Test strategy #1 (`relocate-target-unknown-input.toml`) +
/// A19 — a target is a closed-vocabulary input id, grammar-checked at parse
/// through the shipped `InputId` rule: `keyboard:tab` is outside the 48-member
/// keyboard set, so a policy naming it refuses at the grammar rather than
/// waiting for an application that cannot place it.
#[test]
fn refuses_a_relocate_target_outside_the_closed_input_vocabulary() {
    let err = value_refusal(&policy_with_row(
        "relocate-cluster move.steer-* left right 0 keyboard:tab",
    ));
    assert_refusal_names(&err, &["keyboard:tab", "relocate-cluster"]);
}

/// proves: design 015's Test strategy #1 (`swap-polarity-extra-field.toml`) +
/// A22 — a `swap-polarity` row is exactly its kind and its tag: a third field
/// is a refusal naming the row, never an ignored extra (the fixed-field-count
/// posture the `set`/`swap`/manifest rows already pin).
#[test]
fn refuses_an_extra_field_on_a_swap_polarity_row() {
    let err = value_refusal(&policy_with_row("swap-polarity move.steer-* left"));
    assert_refusal_names(&err, &["swap-polarity"]);
}

/// proves: design 015's Test strategy #1 (`mouse-row-carries-tag.toml`) —
/// `swap-mouse-buttons` takes no argument at all: the mouse pair is the closed
/// vocabulary's `mouse:left`/`mouse:right`, so a tag on the row is a refusal
/// naming the row, not a second match language (A19).
#[test]
fn refuses_any_argument_on_a_swap_mouse_buttons_row() {
    for row in [
        "swap-mouse-buttons move.steer-*",
        "swap-mouse-buttons mouse:left",
        "swap-mouse-buttons 0",
    ] {
        let err = value_refusal(&policy_with_row(row));
        assert_refusal_names(&err, &["swap-mouse-buttons"]);
    }
}

/// proves: design 015's Test strategy #1 (`tag-embedded-glob.toml`,
/// `tag-uppercase.toml`, `tag-empty-prefix.toml`, `tag-three-segments.toml`) +
/// Options considered (Tags) — a tag is an exact action id or
/// `<segment>.<prefix>-*`, and the shape is checked at parse: a star anywhere
/// else, an uppercase spelling, an empty prefix, and a third segment are each a
/// refusal naming the tag. General wildcards would invent a match language over
/// a closed id set.
#[test]
fn refuses_each_malformed_tag_shape() {
    for (tag, label) in [
        ("move.steer-*left", "an embedded glob"),
        ("Move.Steer-*", "an uppercase tag"),
        ("move.-*", "an empty prefix"),
        ("move.steer.left", "a three-segment tag"),
    ] {
        let Err(err) = bindforge_core::parse(&policy_with_row(&format!("swap-polarity {tag}")))
        else {
            panic!("{label} `{tag}` is outside the tag grammar and must be refused");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a malformed tag is a value-level refusal (A9), got {err:?}"
        );
        assert_refusal_names(&err, &[tag, "swap-polarity"]);
    }
}

/// proves: Options considered (Tags) + The model (`TagPattern::matches`) —
/// matching is byte-exact prefixing over the two-segment id, and it is a
/// property of the pattern alone: the prefix form reaches every id whose second
/// segment begins `<prefix>-`, including one no registry lists, and an exact tag
/// matches its own id and nothing shaped like it. Whether a pattern matches a
/// *loaded* id is `crates/transforms'` fact, pinned in `apply.rs`.
#[test]
fn a_tag_pattern_matches_by_byte_exact_prefixing() {
    let policy = policy_of(&policy_doc(
        "tag-shapes",
        &[
            "swap-polarity move.steer-*",
            "swap-polarity move.strafe-left",
        ],
    ));

    let prefix = tag_of(&policy.transforms[0]);
    assert!(
        prefix.matches(&action("move.steer-left")),
        "`move.steer-*` must match an id whose second segment begins `steer-`"
    );
    assert!(
        prefix.matches(&action("move.steer-up")),
        "prefixing is byte-exact and registry-independent: `move.steer-up` matches the pattern \
         even though the shipped registry does not list it (A3)"
    );
    assert!(
        !prefix.matches(&action("move.strafe-left")),
        "`move.strafe-left` does not begin `steer-`, so the pattern must not reach it (011's \
         byte-exact posture)"
    );
    assert!(
        !prefix.matches(&action("aim.camera")),
        "the first segment is part of the pattern: a `move.` prefix must not match an `aim.` id"
    );

    let exact = tag_of(&policy.transforms[1]);
    assert!(
        exact.matches(&action("move.strafe-left")),
        "an exact tag matches the id it spells"
    );
    assert!(
        !exact.matches(&action("move.strafe-left-extra")),
        "an exact tag is not a prefix"
    );
}

/// proves: design 015's Test strategy #1 (`missing-transforms.toml`,
/// `missing-id.toml`) — the required entries are required: an absent `id` or
/// `transforms` is `MissingEntry` naming the entry, never a policy with no name
/// or an empty stack.
#[test]
fn refuses_a_policy_document_missing_a_required_entry() {
    missing_entry_refusal(&policy_without_entries(&["transforms"]), "transforms");
    missing_entry_refusal(&policy_without_entries(&["id"]), "id");
}

/// proves: design 015's Test strategy #1 (`transforms-empty.toml`) + A1 — the
/// stack is `1..=MAX_POLICY_TRANSFORMS`: an empty array is a refusal, because a
/// policy that names no transform is a document whose meaning nobody agreed.
#[test]
fn refuses_an_empty_transforms_array() {
    let err = value_refusal(&policy_with_entry("transforms", "[]"));
    assert_refusal_names(&err, &["transforms"]);
}

/// proves: design 015's Test strategy #1 (`transforms-past-cap.toml`) + A13 —
/// the stack is capped at `MAX_POLICY_TRANSFORMS`, read as the exported constant
/// (the composite cost `spec.md` states multiplies rows out of predicate work,
/// so an uncapped stack is the chain loophole again); the cap parses, one past
/// it refuses.
#[test]
fn refuses_a_transform_stack_past_max_policy_transforms() {
    let at_cap: Vec<String> = (1..=bindforge_core::MAX_POLICY_TRANSFORMS)
        .map(|index| format!("swap-polarity move.row-{index}"))
        .collect();
    let owned: Vec<&str> = at_cap.iter().map(String::as_str).collect();
    if let Err(err) = bindforge_core::parse(&policy_with_rows(&owned)) {
        panic!(
            "a stack of exactly MAX_POLICY_TRANSFORMS ({}) rows must parse; it was refused: {err}",
            bindforge_core::MAX_POLICY_TRANSFORMS
        );
    }

    let mut past_cap = owned;
    past_cap.push("swap-mouse-buttons");
    let err = value_refusal(&policy_with_rows(&past_cap));
    assert_refusal_names(&err, &["transforms"]);
}

/// proves: design 015's Test strategy #1 (`id-uppercase.toml`,
/// `id-past-cap.toml`) + A15, with the cap re-pinned by design 024's Test strategy
/// #6 — the policy `id` rides the shared slug grammar
/// `[a-z][a-z0-9-]*` and the shared `MAX_SLUG_OCTETS` cap, which ADR-024 raises from
/// 63 to 128 octets, the same one a manifest id and a profile id ride: the
/// `schemas/README.md` bullet states that bound once for the full rider set
/// `024-B1` enumerates, and it applies wherever the grammar does — a keyboard
/// geometry name is capped beside the grammar in `load_geometry` (Test strategy
/// #14), and no scope narrowing may drop a rider. It is also the file
/// stem `load_policy`
/// joins and the name a produced base carries in `generated-by`, so the grammar
/// check is what keeps it a file name on a case-insensitive filesystem.
#[test]
fn refuses_a_policy_id_outside_the_slug_grammar_or_past_the_octet_cap() {
    for id in [
        "Probe-Policy",
        "probe_policy",
        "probe policy",
        "",
        "..",
        "0-up",
    ] {
        let Err(err) = bindforge_core::parse(&policy_doc(id, &["swap-mouse-buttons"])) else {
            panic!("`{id}` is outside the slug grammar and must be refused");
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a bad `id` is a value-level refusal (A9), got {err:?}"
        );
        assert_message_shape(&err);
    }

    let at_cap = "p".repeat(bindforge_core::MAX_SLUG_OCTETS);
    if let Err(err) = bindforge_core::parse(&policy_doc(&at_cap, &["swap-mouse-buttons"])) {
        panic!(
            "a slug of exactly MAX_SLUG_OCTETS ({}) octets is at the cap and must parse; it was \
             refused: {err}",
            bindforge_core::MAX_SLUG_OCTETS
        );
    }
    let past_cap = "p".repeat(bindforge_core::MAX_SLUG_OCTETS + 1);
    let err = value_refusal(&policy_doc(&past_cap, &["swap-mouse-buttons"]));
    assert_refusal_names(&err, &["id"]);
}

/// proves: design 015's Test strategy #1 and #9 (`forged-log-line.toml`) +
/// A14 — a policy value carrying a forged newline and 4 KiB of filler is
/// refused in one bounded line that does not echo the filler: policy documents
/// are the fourth untrusted class reaching the network, and a report or refusal
/// that carries one cannot forge a second log line.
#[test]
fn refuses_a_policy_document_carrying_a_forged_log_line_with_a_truncated_message() {
    let tail = "tail-of-the-echoed-value";
    let text = policy_with_entry("id", &format!("\"probe\\n{} {tail}\"", "x".repeat(4096)));

    let Err(err) = bindforge_core::parse(&text) else {
        panic!(
            "an `id` carrying a decoded newline is outside the slug grammar and must be refused"
        );
    };
    let detail = err.to_string();
    assert!(
        !detail.contains(tail),
        "the refusal must truncate an untrusted value, not echo all 4 KiB of it: {detail}"
    );
    assert!(
        detail.chars().all(|ch| !ch.is_control()),
        "the refusal stays one bounded printable line for a value carrying a decoded newline: \
         {detail:?}"
    );
}

/// proves: design 015's Test strategy #2 + A12 — the round trip is equality:
/// for every valid document, `parse(serialize(parse(text)))` equals the first
/// parse, the serialized `transforms` array keeps the document's row order
/// ("order is data"), and each row comes back in the contract's canonical
/// spelling re-spelled from its typed fields.
#[test]
fn round_trips_every_valid_policy_document() {
    for (id, rows) in valid_stacks() {
        let text = policy_doc(id, &rows);
        let first =
            bindforge_core::parse(&text).unwrap_or_else(|err| panic!("`{id}` must parse: {err}"));
        let written = bindforge_core::serialize(&first);
        let again = bindforge_core::parse(&written)
            .unwrap_or_else(|err| panic!("`{id}` does not re-parse after serializing: {err}"));
        assert_eq!(
            again, first,
            "`{id}` must re-serialize to a semantically equal value (A12)"
        );

        let written_rows = written_rows(&written).unwrap_or_else(|| {
            panic!(
                "a serialized policy carries its `transforms` array as one one-line string \
                    array; it wrote: {written}"
            )
        });
        assert_eq!(
            written_rows, rows,
            "`{id}`: the `transforms` array order is application order, so `serialize` preserves \
             it and re-spells each row from its typed fields"
        );
    }
}

/// proves: The model (`document.rs` arms) + Options considered (Stack order) —
/// the typed model carries the stack in array order with the closed three kinds,
/// so a consumer reads the order the document states rather than a canonical
/// sort.
#[test]
fn types_the_stack_in_document_order() {
    let policy = policy_of(&policy_doc(
        "lefty-stack",
        &[
            "relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k",
            "swap-polarity move.strafe-*",
            "swap-mouse-buttons",
        ],
    ));

    assert_eq!(policy.id, "lefty-stack");
    assert_eq!(
        policy.transforms.iter().map(kind_of).collect::<Vec<_>>(),
        vec!["relocate-cluster", "swap-polarity", "swap-mouse-buttons"],
        "array order is application order (A12): the model keeps it, in the closed three kinds"
    );
}

/// proves: Chosen design → The model — a `relocate-cluster` row's typed fields
/// are the row's fields: both hands, the bound, and the target list in row
/// order; `swap-mouse-buttons` carries none.
#[test]
fn types_every_relocate_cluster_field() {
    let policy = policy_of(&policy_doc(
        "field-shapes",
        &[
            "relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k",
            "relocate-cluster move.strafe-* right left 0 keyboard:w keyboard:a keyboard:s keyboard:d",
        ],
    ));

    let Transform::RelocateCluster {
        from,
        to,
        bound,
        targets,
        ..
    } = &policy.transforms[0]
    else {
        panic!("the probe row is a `relocate-cluster` row");
    };
    assert!(
        matches!(from, Hand::Left),
        "`<from-hand> left` is `Hand::Left`, got {from:?}"
    );
    assert!(
        matches!(to, Hand::Right),
        "`<to-hand> right` is `Hand::Right`, got {to:?}"
    );
    assert_eq!(bound.0, 2, "the bound is the row's `GradeBound`");
    assert_eq!(
        targets.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        vec!["keyboard:i", "keyboard:k"],
        "the target list keeps row order"
    );

    let Transform::RelocateCluster {
        from,
        to,
        bound,
        targets,
        ..
    } = &policy.transforms[1]
    else {
        panic!("the probe row is a `relocate-cluster` row");
    };
    assert!(
        matches!(from, Hand::Right) && matches!(to, Hand::Left),
        "the hands are the row's, not a derivation from the data (A4)"
    );
    assert_eq!(bound.0, 0, "the scale's tightest bound is a real value");
    assert_eq!(
        targets.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        vec!["keyboard:w", "keyboard:a", "keyboard:s", "keyboard:d"],
        "four targets, in row order, paired at application by the match rule"
    );
}

/// proves: design 015's Test strategy #3 + A15 + the security finding the
/// research names — `load_policy` checks `id` against the slug grammar *before*
/// joining it, so a traversal, a separator, or an absolute path is refused
/// without reading anything. The file a traversal would reach is a perfectly
/// valid policy document here, and the call still refuses, naming no path it
/// resolved and no file it opened (A14).
#[test]
fn a_policy_id_outside_the_slug_grammar_is_refused_before_any_file_is_read() {
    let scratch = ScratchDir::new("policy-name-grammar");
    scratch.write(
        "v1/probe-policy.toml",
        &policy_doc("probe-policy", &["swap-mouse-buttons"]),
    );
    // A valid policy document one step *outside* the directory the caller names:
    // if the name reached the join, this file would load cleanly.
    scratch.write(
        "outside.toml",
        &policy_doc("outside", &["swap-mouse-buttons"]),
    );
    let dir = scratch.path().join("v1");
    // The same file addressed absolutely, which `Path::join` takes over `dir`.
    let absolute = scratch.path().join("outside");

    for name in [
        "..",
        ".",
        "",
        "../outside",
        "v1/../outside",
        "./outside",
        "v1/../../outside",
        "PROBE-POLICY",
        "probe policy",
    ] {
        let Err(err) = bindforge_core::load_policy(&dir, name) else {
            panic!(
                "`{name}` is outside the `[a-z][a-z0-9-]*` policy grammar and must be refused \
                 before any file is read"
            );
        };
        assert!(
            matches!(err, CoreError::InvalidValue { .. }),
            "a name outside the grammar is refused as `InvalidValue`, got {err:?}"
        );
        let detail = err.to_string();
        assert!(
            !detail.contains(&scratch.path().display().to_string()),
            "a refusal for a bad `id` must not disclose the local path the loader resolved \
             (A14), got `{detail}`"
        );
        assert!(
            !detail.contains("outside.toml"),
            "the refusal must not name a file the loader read: the grammar check runs *before* \
             the join, so no file is opened at all, got `{detail}`"
        );
        assert_message_shape(&err);
    }

    // The absolute case is the hazard: `Path::join` with an absolute argument
    // replaces `dir` outright, so a caller forwarding a user-selected name could
    // read any `.toml` on the machine. The grammar check runs first, so the read
    // never happens.
    let Err(err) = bindforge_core::load_policy(&dir, absolute.to_str().unwrap_or("outside")) else {
        panic!(
            "an absolute `id` must be refused by the grammar check: `Path::join` would replace \
             the directory entirely and read any `.toml` on the machine"
        );
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "an absolute `id` is outside the grammar, so it is `InvalidValue`, got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        !detail.contains("outside.toml"),
        "the refusal must not name the file the traversal would have read, got `{detail}`"
    );
    assert_message_shape(&err);
}

/// proves: design 015's Test strategy #3 with design 024's Test strategy #6 and the
/// accepted code-review r1 finding — `load_policy` splits its two slug faults by
/// kind: a grammar-valid `id` one octet past `MAX_SLUG_OCTETS` is refused at the
/// load seam naming the *bound* it broke (interpolated from the exported constant,
/// inside the renderer's cut), not the grammar rule the offender satisfies — and,
/// like the grammar arm, before any file is read.
#[test]
fn load_policy_names_the_shared_bound_for_a_grammar_valid_id_one_octet_past_it() {
    let scratch = ScratchDir::new("policy-name-bound");
    let dir = scratch.path().join("v1");
    let past_cap = "p".repeat(bindforge_core::MAX_SLUG_OCTETS + 1);
    // The file the join would reach if the bound arm let the name through: written
    // here only so the refusal below proves it never opened it.
    scratch.write(
        &format!("v1/{past_cap}.toml"),
        &policy_doc(&past_cap, &["swap-mouse-buttons"]),
    );

    let Err(err) = bindforge_core::load_policy(&dir, &past_cap) else {
        panic!(
            "a grammar-valid `id` one octet past MAX_SLUG_OCTETS ({}) must be refused at \
             the load seam",
            bindforge_core::MAX_SLUG_OCTETS
        );
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "an over-bound policy `id` is a value-level refusal (A9), got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains(&bindforge_core::MAX_SLUG_OCTETS.to_string()),
        "the bound fault names the bound it broke ({}, built from the exported constant), \
         not the grammar rule the offender satisfies: `{detail}`",
        bindforge_core::MAX_SLUG_OCTETS
    );
    assert!(
        !detail.contains(&scratch.path().display().to_string()),
        "a refusal for a bad `id` must not disclose the local path the loader resolved \
         (A14), got `{detail}`"
    );
    assert_message_shape(&err);
}

/// proves: design 015's Test strategy #3 + A15 — the name↔location rule the
/// geometry and manifest loaders pin: the file is `<id>.toml` and its `id`
/// entry must name the same value as its stem, and the refusal keeps the local
/// path out of the message shown to whoever supplied the file.
#[test]
fn a_policy_file_whose_stem_contradicts_its_id_entry_refuses_without_naming_a_path() {
    let scratch = ScratchDir::new("policy-stem-contradiction");
    scratch.write(
        "v1/probe-policy.toml",
        &policy_doc("other-policy", &["swap-mouse-buttons"]),
    );

    let Err(err) = bindforge_core::load_policy(&scratch.path().join("v1"), "probe-policy") else {
        panic!(
            "`v1/probe-policy.toml` declares `id = \"other-policy\"`; the file is `<id>.toml` and \
             its entry must name the same value"
        );
    };
    let detail = err.to_string();
    assert!(
        !detail.contains(&scratch.path().display().to_string()),
        "a refusal shown to whoever sent the file must not disclose a local path, got `{detail}`"
    );
    assert!(
        detail.contains("probe-policy") || detail.contains("other-policy"),
        "the refusal must name the name it refused, got `{detail}`"
    );
    assert_message_shape(&err);
}

/// proves: design 015's Test strategy #3 + A11 — the shipped per-format
/// directory rule: `check_dir_version` pins the declared major against the
/// `vN/` segment the file sits in, so a `v1` document inside `v2/` refuses even
/// though nothing else about it is wrong.
#[test]
fn a_policy_document_whose_declared_major_differs_from_its_directory_refuses() {
    let scratch = ScratchDir::new("policy-directory-major");
    scratch.write(
        "v2/probe-policy.toml",
        &policy_doc("probe-policy", &["swap-mouse-buttons"]),
    );

    let Err(err) = bindforge_core::load_policy(&scratch.path().join("v2"), "probe-policy") else {
        panic!(
            "a document declaring `version = \"v1\"` sitting in a `v2/` directory must refuse: \
             the directory segment is what makes a major shipped (A11)"
        );
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. }),
        "the directory-major rule is answered by the loader's `InvalidValue`, like every other \
         loader answers it (A9), got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains("v1") || detail.contains("v2"),
        "the refusal must name the major or the directory segment it checked, got `{detail}`"
    );
    assert!(
        !detail.contains(&scratch.path().display().to_string()),
        "the refusal keeps the local path out of the message (A14), got `{detail}`"
    );
    assert_message_shape(&err);
}

/// proves: design 015's Test strategy #3 + A8 — a non-policy document at the
/// policy path refuses on the format arm: `load_policy` does not reinterpret a
/// profile or a manifest as a policy, the same fail-closed posture a pre-015
/// reader brings to `format = "policy"`.
#[test]
fn a_non_policy_document_at_the_policy_path_refuses_on_the_format_arm() {
    let scratch = ScratchDir::new("policy-format-arm");
    scratch.write(
        "v1/probe-policy.toml",
        "format = \"profile\"\nversion = \"v1\"\nid = \"probe-policy\"\nderives = []\n",
    );

    let Err(err) = bindforge_core::load_policy(&scratch.path().join("v1"), "probe-policy") else {
        panic!("a profile document at the policy path must refuse on the format arm, not parse");
    };
    assert!(
        matches!(err, CoreError::InvalidValue { .. })
            || matches!(err, CoreError::MissingEntry { .. }),
        "the format-arm refusal reuses a shipped variant (A9: no new variant), got {err:?}"
    );
    let detail = err.to_string();
    assert!(
        detail.contains("policy") || detail.contains("profile") || detail.contains("format"),
        "the refusal names the format it refused, got `{detail}`"
    );
    assert_message_shape(&err);
}

/// proves: design 015's Test strategy #3 + The contract (`load_policy`) — the
/// happy path is a load: the file `<id>.toml` directly in the major directory
/// reads into the typed model, stack order and all, and the loader answers the
/// document the directory and the entry agree on.
#[test]
fn load_policy_answers_the_typed_stack_from_its_major_directory() {
    let scratch = ScratchDir::new("policy-load");
    scratch.write(
        "v1/lefty-stack.toml",
        &policy_doc(
            "lefty-stack",
            &[
                "relocate-cluster move.steer-* left right 2 keyboard:i keyboard:k",
                "swap-polarity move.strafe-*",
                "swap-mouse-buttons",
            ],
        ),
    );

    let policy = bindforge_core::load_policy(&scratch.path().join("v1"), "lefty-stack")
        .expect("a policy document in its own major directory must load");
    assert_eq!(policy.id, "lefty-stack");
    assert_eq!(
        policy.version, "v1",
        "the loaded model carries the declared major"
    );
    assert_eq!(
        policy.transforms.iter().map(kind_of).collect::<Vec<_>>(),
        vec!["relocate-cluster", "swap-polarity", "swap-mouse-buttons"],
        "`load_policy` answers the stack in document order"
    );
}

/// proves: Chosen design → The contract (the corpus) + design 015's Test
/// strategy #9 — the corpus the contract names ships, so the unchanged walk has
/// the policy half of the contract to run: six valid documents and thirty
/// invalid ones. What the walk asserts over them — parse, guards, the
/// round-trip, the one-line truncating refusal, the zero-fixture rule and the
/// case-unique name rule — is `conformance.rs`.
#[test]
fn the_policy_corpus_ships_every_fixture_the_contract_names() {
    let dir = version_dir("policy", "v1");
    for (kind, names) in [
        ("valid", VALID_FIXTURE_NAMES.as_slice()),
        ("invalid", INVALID_FIXTURE_NAMES.as_slice()),
    ] {
        for name in names {
            let path = dir
                .path
                .join("fixtures")
                .join(kind)
                .join(format!("{name}.toml"));
            assert!(
                path.is_file(),
                "policy/v1 must ship `fixtures/{kind}/{name}.toml`: presence in `fixtures/valid/` \
                 is the definition of \"supported\", and the invalid corpus is the contract for \
                 rejection behavior (missing at {})",
                path.display()
            );
        }
    }
}
