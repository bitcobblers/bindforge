//! Doc-ledger and spec-literal pins for design 024 / ADR-024: the corpus claims
//! the escalation repair gave gate teeth must stay true of the shipped tree.
//!
//! proves: design 024's Test strategy #13 — three claims asserted against the
//! corpus, the repo root resolved from `CARGO_MANIFEST_DIR` (via `common`):
//! (a) the `013-L5` unit line in `docs/adrs/013-game-manifests.md` carries
//! `amended-by 024-B1`; (b) no bound sentence in the four amended `spec.md`
//! files contains a literal `63` or `128`, each bound sentence names
//! `MAX_SLUG_OCTETS` and points at `schemas/README.md`, and
//! `schemas/action-ids/v1/spec.md` carries no `RFC 1035` string; (c) the
//! `schemas/README.md` bound bullet states `MAX_SLUG_OCTETS = 128` and its rider
//! set names a keyboard geometry name.
//!
//! These are document-ledger gates, not library behavior. The ledger annotation
//! (Rollout 3) and the spec-literal discipline (Chosen design: Specs, fixtures,
//! ledger) land as documents no other gate leg reads, so this suite reads the
//! corpus directly: "the annotation is gate-pinned, not convention" — the
//! workspace suite fails until the `amended-by 024-B1` string is on the `013-L5`
//! unit line, and the one stated home plus its riders are pinned against drift.
//!
//! Unit-level: reads checked-in corpus files, writes nothing, no network.

mod common;

use common::{fixture_text, repo_root, version_dir};

/// The four `spec.md` files ADR-024 amends in place (A3).
const AMENDED_SPECS: [(&str, &str); 4] = [
    ("manifests", "v1"),
    ("profile", "v2"),
    ("policy", "v1"),
    ("action-ids", "v1"),
];

/// The riders `024-B1` enumerates as the ids riding `MAX_SLUG_OCTETS`, in its
/// own spelling: a manifest `id`, `codec` or preset id, a profile `id` and each
/// `derives` element, a policy `id`, a geometry name, and an action-id segment.
/// The README bullet — the bound's one stated home — states this set, and the
/// Chosen design permits no scope narrowing of it.
const RIDERS: [&str; 7] = [
    "manifest",
    "codec",
    "preset",
    "derives",
    "policy",
    "geometry name",
    "action-id segment",
];

/// proves: design 024's Test strategy #13(a) + Chosen design (Specs, fixtures,
/// ledger) + Rollout 3 — the `013-L5` unit entry in `docs/adrs/013-game-manifests.md`
/// carries the `amended-by 024-B1` annotation `021-L4` requires of an amended
/// unit; `013`'s prose and `013-R1` stay untouched as history, so this suite
/// fails the workspace run until the runner lands the annotation.
#[test]
fn the_013_l5_unit_line_carries_the_amendment_annotation() {
    let path = repo_root()
        .join("docs")
        .join("adrs")
        .join("013-game-manifests.md");
    let text = fixture_text(&path);
    let entry = ledger_unit(&text, "013-L5");
    assert!(
        entry.contains("amended-by 024-B1"),
        "`013-L5` must carry `amended-by 024-B1` (ADR-024 Rollout 5, per `021-L4`): \
         this run raises the shared bound and amends `013-L5` (`024-B1`), and a branch \
         that records itself complete with the amended unit's ledger row un-annotated \
         is the drift this gate exists to refuse. Unit entry as shipped: {entry}"
    );
}

/// proves: design 024's Test strategy #13(b) + Chosen design (Specs, fixtures,
/// ledger, acceptance) — no bound sentence in the four amended `spec.md` files
/// contains a literal `63` or `128`; each file states the bound in a sentence
/// that names `MAX_SLUG_OCTETS` and points at `schemas/README.md`, so the
/// README bullet's claim that the four cross-reference it is true of the shipped
/// corpus.
#[test]
fn the_amended_specs_state_the_shared_bound_without_a_number_of_their_own() {
    for (format, major) in AMENDED_SPECS {
        let path = version_dir(format, major).path.join("spec.md");
        let text = fixture_text(&path);
        let chunks = sentences(&text);
        let bound: Vec<&String> = chunks
            .iter()
            .filter(|sentence| is_bound_sentence(sentence))
            .collect();
        assert!(
            !bound.is_empty(),
            "`{format}/{major}/spec.md` must state the shared slug bound: no sentence \
             names `MAX_SLUG_OCTETS`, the shared bound, or the slug bound"
        );
        assert!(
            bound
                .iter()
                .any(|sentence| sentence.contains("MAX_SLUG_OCTETS")
                    && sentence.contains("README.md")),
            "`{format}/{major}/spec.md` must state the bound in a sentence that names \
             `MAX_SLUG_OCTETS` and points at `schemas/README.md` — the bound's one \
             stated home — rather than carrying a number of its own"
        );
        for sentence in bound {
            assert!(
                !has_number(sentence, "63") && !has_number(sentence, "128"),
                "`{format}/{major}/spec.md`: a bound sentence carries a literal length \
                 — ADR-024's acceptance is that the number lives only at its one \
                 stated home: {sentence}"
            );
        }
    }
}

/// proves: design 024's Test strategy #13(b), second half + Chosen design
/// (Specs, fixtures, ledger) — `schemas/action-ids/v1/spec.md` carries no
/// `RFC 1035` string anywhere: the superseded DNS-label derivation lives,
/// superseded, in the README bullet alone.
#[test]
fn the_action_ids_spec_carries_no_dns_label_derivation() {
    let path = version_dir("action-ids", "v1").path.join("spec.md");
    let text = fixture_text(&path);
    assert!(
        !text.contains("RFC 1035"),
        "`schemas/action-ids/v1/spec.md` must not name RFC 1035: the spec amendment \
         replaces the RFC 1035 §2.3.1 derivation with the cross-reference to \
         `schemas/README.md`, whose bullet carries the superseded origin"
    );
}

/// proves: design 024's Test strategy #13(c) + Chosen design (Shared grammar and
/// bound) — the `schemas/README.md` bound bullet states `MAX_SLUG_OCTETS = 128`
/// and its rider set names a keyboard geometry name beside the other ids
/// `024-B1` enumerates: the one stated home and its full rider set are pinned
/// against drift, and no scope narrowing may drop a rider from the bullet.
#[test]
fn the_readme_bound_bullet_states_the_constant_and_every_rider() {
    let path = repo_root().join("schemas").join("README.md");
    let text = fixture_text(&path);
    let bullet = bound_bullet(&text);
    assert!(
        bullet.contains("MAX_SLUG_OCTETS = 128"),
        "`schemas/README.md` must state the bound as `MAX_SLUG_OCTETS = 128` — the \
         bound's one stated home carries the number the whole corpus cross-references"
    );
    for rider in RIDERS {
        assert!(
            bullet.contains(rider),
            "the README bound bullet's rider set must enumerate `024-B1`'s ids \
             exactly, including `{rider}` — no scope narrowing may drop a rider: {bullet}"
        );
    }
}

/// The full text of one ledger unit entry in an ADR: the bullet naming it, with
/// every continuation line up to the next bullet or blank line, whitespace
/// normalized and code-span backticks removed — so the claim is about the
/// annotation being on the unit, not about which of its lines the runner
/// appended it to or how the id is quoted.
fn ledger_unit(text: &str, unit: &str) -> String {
    let marker = format!("**{unit}**");
    let mut entry: Vec<&str> = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if !inside {
            if trimmed.contains(&marker) {
                inside = true;
                entry.push(trimmed);
            }
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("- **") || trimmed.starts_with('#') {
            break;
        }
        entry.push(trimmed);
    }
    assert!(
        !entry.is_empty(),
        "the ledger text must carry a `{unit}` unit entry"
    );
    entry.join(" ").replace('`', "")
}

/// Sentence-ish chunks of a markdown text: wrapped prose is rejoined, and the
/// text is cut at every `.` that is followed by a space. A dot inside a code
/// span (`README.md`, `Move.Forward`, `§2.3.1`) is followed by a letter or a
/// digit and never cuts, so a chunk stays the sentence the contract states.
fn sentences(text: &str) -> Vec<String> {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut chars = normalized.chars().peekable();
    while let Some(ch) = chars.next() {
        current.push(ch);
        if ch == '.' && chars.peek() == Some(&' ') {
            chars.next();
            chunks.push(std::mem::take(&mut current));
        }
    }
    if !current.trim().is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Whether a sentence states the shared slug bound — by the exported constant or
/// by the name the four amendments give it. Other budgets (native names,
/// generated-by labels, relocate targets) state their own caps and are not
/// sentences about this bound.
fn is_bound_sentence(sentence: &str) -> bool {
    sentence.contains("MAX_SLUG_OCTETS")
        || sentence.contains("shared bound")
        || sentence.contains("slug bound")
}

/// Whether `text` carries `number` as a standalone numeric token — no digit on
/// either side — so unit ids (`024-B1`) and section numbers (`§2.3.1`) are not
/// readings of `63` or `128`.
fn has_number(text: &str, number: &str) -> bool {
    text.match_indices(number).any(|(index, _)| {
        let before = index
            .checked_sub(1)
            .is_some_and(|i| text.as_bytes()[i].is_ascii_digit());
        let after = text
            .as_bytes()
            .get(index + number.len())
            .is_some_and(|byte| byte.is_ascii_digit());
        !before && !after
    })
}

/// The `schemas/README.md` bullet that states the bound: the list item carrying
/// `MAX_SLUG_OCTETS = 128`, from its opening line to the blank line that ends
/// the item, whitespace normalized.
fn bound_bullet(text: &str) -> String {
    budget_bullet(text, "MAX_SLUG_OCTETS = 128")
}

/// The `schemas/README.md` bullet of the exported budget family that carries
/// `marker`, from its opening line to the blank line that ends the item,
/// whitespace normalized.
fn budget_bullet(text: &str, marker: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.contains(marker))
        .unwrap_or_else(|| {
            panic!("`schemas/README.md` must state `{marker}` in the exported budget family")
        });
    let mut bullet = String::new();
    for line in &lines[start..] {
        let trimmed = line.trim();
        if !bullet.is_empty() && (trimmed.starts_with("- ") || trimmed.is_empty()) {
            break;
        }
        bullet.push_str(trimmed);
        bullet.push(' ');
    }
    bullet
}

/// proves: design 025's Test strategy #16 + Rollout 5 — the four amended units in
/// `docs/adrs/024-game-slug-identity.md` carry their `amended-by` annotations naming
/// the ADR-025 units, in `021-L4`'s shipped shape: `024-C1` → `025-C1`, `024-C2` →
/// `025-C2`, `024-L1` → `025-L3`, `024-L2` → `025-L3`. `024`'s prose and
/// `024-B1`/`024-L3` stay untouched as history — this gate reads only the unit
/// entries, and a branch whose ledger rows are un-annotated is the drift it refuses.
#[test]
fn the_024_amended_units_carry_the_adr_025_annotations() {
    let path = repo_root()
        .join("docs")
        .join("adrs")
        .join("024-game-slug-identity.md");
    let text = fixture_text(&path);
    for (unit, amendment) in [
        ("024-C1", "amended-by 025-C1"),
        ("024-C2", "amended-by 025-C2"),
        ("024-L1", "amended-by 025-L3"),
        ("024-L2", "amended-by 025-L3"),
    ] {
        let entry = ledger_unit(&text, unit);
        assert!(
            entry.contains(amendment),
            "`{unit}` must carry `{amendment}` (design 025 Rollout 5, per `021-L4`): \
             ADR-025 replaces the identity half of ADR-024, and the amended units' \
             ledger rows must say so. Unit entry as shipped: {entry}"
        );
    }
}

/// proves: design 025's Test strategy #16 + Rollout 5 — the `024-R1` reservation
/// entry records ADR-025 as filling its slot with the publisher-namespace half
/// superseded: the resolver reservation is now a named, taken reservation.
#[test]
fn the_024_r1_reservation_records_adr_025_as_filling_its_slot() {
    let path = repo_root()
        .join("docs")
        .join("adrs")
        .join("024-game-slug-identity.md");
    let text = fixture_text(&path);
    let entry = ledger_unit(&text, "024-R1");
    assert!(
        entry.contains("ADR-025"),
        "`024-R1` must name ADR-025 as the ADR that fills its slot (design 025 Rollout \
         5): {entry}"
    );
    assert!(
        entry.contains("superseded"),
        "`024-R1` must record that the publisher-namespace half is superseded while \
         ADR-025 fills the resolver slot (design 025 Rollout 5): {entry}"
    );
}

/// proves: design 025's Test strategy #16 (Chosen design: Bound — "`schemas/
/// README.md` gains a sibling bullet in the exported budget family") — the sibling
/// bullet states `MAX_DISCOVERY_SLUG_OCTETS = 128` and carries the not-a-grammar-bound
/// clause: the discovery key is exempt from the slug grammar and never a path
/// component. The bullet is its own item, so the `024-B1` bullet's rider set stays
/// exactly as shipped (gated by the existing test above) and the two cannot drift.
#[test]
fn the_readme_sibling_bullet_states_the_discovery_bound_is_not_a_grammar_bound() {
    let path = repo_root().join("schemas").join("README.md");
    let text = fixture_text(&path);
    let bullet = budget_bullet(&text, "MAX_DISCOVERY_SLUG_OCTETS = 128");
    assert!(
        bullet.contains("MAX_DISCOVERY_SLUG_OCTETS = 128"),
        "`schemas/README.md` must state the lookup-key cap as \
         `MAX_DISCOVERY_SLUG_OCTETS = 128` in the exported budget family"
    );
    assert!(
        bullet.contains("grammar") && bullet.contains("not"),
        "the sibling bullet must carry the explicit clause that this bound is *not* a \
         slug-grammar bound — the discovery key is exempt from the grammar and never a \
         path component (`025-B1`, `025-L5`): {bullet}"
    );
}
