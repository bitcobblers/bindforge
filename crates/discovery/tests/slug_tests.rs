//! The module-mirrored suite for `crates/discovery/src/slug.rs` (ADR-004): the
//! canonical `024-C2` derivation that replaces the shipped `normalize_slug`.
//!
//! proves: design 024's Test strategy #1 (the derivation table), #2 (the bound half
//! that belongs at the derivation: length is never truncated), and #11 (the
//! platform-independent half: the same inputs derive the same slugs on every runner,
//! and the Windows-reserved-name derivation is pinned as *known current behaviour*
//! while the reserved-name refusal stays deferred).
//!
//! The survey cases below are reproduced exactly from `01-research.md` Findings
//! (Valve store-search, 143 titles, 2026-11-18), including the 64-octet slug that
//! today's 63-octet bound refused and the `g-` leading-digit spellings the `024-C2`
//! rule produces. The fold-table cases pin the design's stated coverage boundary:
//! Latin-1 Supplement and Latin Extended-A single-base-letter decompositions fold to
//! their lower-case ASCII bases, nothing is transliterated to more than one character,
//! and every other character — including Latin Extended Additional and every non-Latin
//! script — becomes `-`, which is also what NFKD-with-dropped-marks yields for those
//! bases.
//!
//! `derive_slug` is pure over a `&str`, so this suite is a unit test with no
//! filesystem, no network, and no host Steam (repo practice).

use bindforge_core::{MAX_SLUG_OCTETS, is_slug_name};
use discovery::slug::derive_slug;

/// A display name that derives to exactly `want` octets of slug: `head` letters, a
/// separator, then `tail` letters.
fn letters(head: usize, tail: usize) -> String {
    format!("{} {}", "k".repeat(head), "k".repeat(tail))
}

/// proves: design 024's Test strategy #1 + Findings (survey) — the surveyed titles
/// derive to the slugs the survey recorded, byte for byte, prefix-free: the 64-octet
/// slug ADR-024's raised bound exists to admit, the three `g-` leading-digit spellings,
/// and the trade-mark-folded `NieR:Automata™`.
#[test]
fn derives_the_surveyed_titles_to_their_recorded_slugs() {
    for (name, slug) in [
        (
            "Warhammer 40,000: Darktide - Sisters Novitiate Class Deluxe Edition",
            "warhammer-40-000-darktide-sisters-novitiate-class-deluxe-edition",
        ),
        ("7 Days to Die", "g-7-days-to-die"),
        ("8BitBoy™", "g-8bitboy"),
        ("8-Bit Armies", "g-8-bit-armies"),
        ("NieR:Automata™", "nier-automata"),
        (
            "S.T.A.L.K.E.R.: Shadow of Chernobyl",
            "s-t-a-l-k-e-r-shadow-of-chernobyl",
        ),
        ("Assassin’s Creed III", "assassin-s-creed-iii"),
    ] {
        assert_eq!(
            derive_slug(name).as_deref(),
            Some(slug),
            "`{name}` derives to the surveyed slug, with no `unknown/` prefix anywhere. Every \
             character outside `[a-z0-9]` becomes `-` and runs collapse, so the `S.T.A.L.K.E.R.` \
             dots stay one-hyphen-apart"
        );
    }
}

/// proves: design 024's Test strategy #1 + Chosen design (Derivation) — the fold
/// table runs *before* the ASCII case-fold, so an accented capital keeps its base
/// letter instead of folding to `-`: `é`→`e`, `ü`→`u`, `ñ`→`n`, and the
/// Latin-1/Latin-Extended-A capitals `Å`→`a` and `É`→`e`.
#[test]
fn folds_single_base_letter_latin_to_its_lower_case_ascii_base() {
    for (name, slug) in [
        ("Café du Monde", "cafe-du-monde"),
        ("Kühlstraße Depot", "kuhlstra-e-depot"),
        ("El Niño", "el-nino"),
        ("Åre Mountain Pass", "are-mountain-pass"),
        ("Vårgården", "vargarden"),
        ("Éclair Station", "eclair-station"),
        ("Über Driver", "uber-driver"),
        ("Zażółć Gęślą Jaźń", "zazo-c-gesla-jazn"),
    ] {
        assert_eq!(
            derive_slug(name).as_deref(),
            Some(slug),
            "`{name}` folds to its ASCII bases: the design's covered blocks are Latin-1 \
             Supplement and Latin Extended-A, both cases, and a stroke letter like `ł` — which \
             has no single-letter canonical base — is the `-` the unmapped rule promises"
        );
    }
}

/// proves: design 024's Test strategy #1 + Options considered (the std-only fold
/// table) — no multi-character transliteration exists here: `ø`, `æ` and `ß` are each
/// one `-`, never `o`, `ae` or `ss`, and a name made only of such a character has no
/// Latin base to fall back on.
#[test]
fn maps_letters_outside_the_fold_table_to_a_single_hyphen_never_to_two_letters() {
    for (name, slug) in [
        ("Mølly", "m-lly"),
        ("ænigmatix", "nigmatix"),
        ("straße", "stra-e"),
        ("Doppelgänger ßß", "doppelganger"),
    ] {
        assert_eq!(
            derive_slug(name).as_deref(),
            Some(slug),
            "`{name}` must not be transliterated: `ø`, `æ` and `ß` each become one `-`"
        );
    }

    assert_eq!(
        derive_slug("ø"),
        None,
        "a name that is only `ø` has no base letter left and is refused, not invented"
    );
}

/// proves: design 024's Test strategy #1 + A15 (the coverage boundary the design
/// names) — Latin Extended Additional is outside the table, so `ḗ` folds to `-` the
/// way every unmapped character does.
#[test]
fn folds_latin_extended_additional_to_a_hyphen_beyond_the_covered_blocks() {
    assert_eq!(
        derive_slug("Cḗadme").as_deref(),
        Some("c-adme"),
        "`ḗ` (U+1E17, Latin Extended Additional) is outside the two covered blocks, so it \
         becomes `-`"
    );
    assert_eq!(
        derive_slug("ḗ"),
        None,
        "a name that folds to nothing but a `-` derives no slug"
    );
}

/// proves: design 024's Test strategy #1 + Risks (the fold table's divergence) — an
/// input with no Latin base letter at all derives nothing: non-Latin scripts, symbol
/// runs, and empty or separator-only input are refusals for the caller to fall back
/// on or count, never an empty slug.
#[test]
fn derives_nothing_from_a_name_with_no_latin_base_letter() {
    for name in [
        "",
        "   ",
        "---",
        "!!!",
        "ドラゴンズドグマ",
        "بازی",
        "🙂🙂",
        "  — · ,  ",
    ] {
        assert_eq!(
            derive_slug(name),
            None,
            "`{name}` has nothing to slug: an empty result is a refusal, not a bare separator"
        );
    }
}

/// proves: design 024's Test strategy #1 (re-pinned spellings) + Goals — hyphen runs
/// collapse, both ends are trimmed, ASCII case-folds, and no slug carries the retired
/// `unknown/` namespace: the five shapes the shipped `normalize_slug` tests pinned,
/// minus the prefix.
#[test]
fn collapses_and_trims_hyphens_and_never_emits_the_retired_namespace() {
    for (name, slug) in [
        ("Elden Ring", "elden-ring"),
        ("Super Mario Odyssey!", "super-mario-odyssey"),
        ("COUNTER-STRIKE 2", "counter-strike-2"),
        ("Game--Name", "game-name"),
        ("  Hello World!!!  ", "hello-world"),
        ("HALF-LIFE 2: Episode One", "half-life-2-episode-one"),
    ] {
        let derived = derive_slug(name)
            .unwrap_or_else(|| panic!("`{name}` has Latin base letters, so it must derive"));
        assert_eq!(derived, slug, "`{name}` slugs to the prefix-free spelling");
        assert!(
            !derived.contains("unknown"),
            "`{name}` must not carry the retired `unknown/` namespace: {derived}"
        );
        assert!(
            is_slug_name(&derived) && derived.len() <= MAX_SLUG_OCTETS,
            "every derived slug is a product id: `{derived}` fails the shared grammar and bound"
        );
    }
}

/// proves: design 024's Test strategy #1 + Goals — a leading digit takes the `g-`
/// prefix rather than being refused or dropped, which is what keeps `7 Days to Die`
/// adoptable as an id under the shared grammar's letter-first rule.
#[test]
fn prefixes_a_leading_digit_with_the_g_rule() {
    for (name, slug) in [
        ("3D Print Miniatures", "g-3d-print-miniatures"),
        ("1000 Heads", "g-1000-heads"),
        ("0 A.D.", "g-0-a-d"),
    ] {
        assert_eq!(
            derive_slug(name).as_deref(),
            Some(slug),
            "`{name}` opens with a digit and must take the `g-` prefix"
        );
    }
}

/// proves: design 024's Test strategy #2 + Chosen design (Derivation) — length is
/// never truncated: a name that derives to exactly the shared bound keeps all of it,
/// and a name one octet past the bound derives whole rather than being cut. The
/// bound check is the caller's named refusal, so `derive_slug` must not silently
/// shorten what the caller has to refuse.
#[test]
fn derives_whole_slugs_at_and_one_past_the_shared_bound() {
    let at_bound = derive_slug(&letters(MAX_SLUG_OCTETS - 64, 63))
        .expect("a name of Latin base letters derives");
    assert_eq!(
        at_bound.len(),
        MAX_SLUG_OCTETS,
        "a derivation at exactly the bound keeps every octet and passes the grammar"
    );
    assert!(
        is_slug_name(&at_bound),
        "a slug of exactly MAX_SLUG_OCTETS ({MAX_SLUG_OCTETS}) is grammar-valid: {at_bound}"
    );

    let past_bound = derive_slug(&letters(MAX_SLUG_OCTETS - 63, 63))
        .expect("a name of Latin base letters derives");
    assert_eq!(
        past_bound.len(),
        MAX_SLUG_OCTETS + 1,
        "a derivation past the bound is not truncated — it is refused by name at the emit \
         boundary, so the whole string must survive derivation"
    );
    assert!(
        past_bound.len() > MAX_SLUG_OCTETS,
        "the caller's named refusal is `len() > MAX_SLUG_OCTETS`, so a truncated derivation \
         would hide the fault it must report"
    );
}

/// proves: design 024's Test strategy #11 + Deferred (Windows reserved names) — the
/// reserved-device spellings are legal under the shared grammar and derive unchanged:
/// pinned as *known current behaviour*, because the refusal is deferred with a named
/// owner and no test may assert a platform-specific outcome.
#[test]
fn derives_windows_reserved_device_names_unchanged_as_known_current_behaviour() {
    for name in ["Con", "Prn", "Aux", "Nul", "Com1", "Lpt9"] {
        let derived = derive_slug(name).expect("a reserved-device name has Latin bases");
        assert_eq!(
            derived,
            name.to_lowercase(),
            "`{name}` derives to its lowercase spelling: the grammar does not refuse reserved \
             device names yet, and this pin is the deferral's paper trail"
        );
        assert!(
            is_slug_name(&derived),
            "`{derived}` is grammar-valid today, which is exactly the gap the deferred \
             reserved-name amendment ADR owns"
        );
    }
}

/// proves: design 024's Test strategy #11 + Compatibility matrix — derivation is
/// platform-independent: every slug it emits is pure ASCII, so a Linux CI run asserts
/// the same results a Windows or macOS run would.
#[test]
fn every_derived_slug_is_pure_ascii_so_its_result_cannot_vary_by_platform() {
    let past_cap_name = letters(MAX_SLUG_OCTETS - 63, 63);
    let corpus = [
        "Warhammer 40,000: Darktide - Sisters Novitiate Class Deluxe Edition",
        "Zażółć Gęślą Jaźń",
        "NieR:Automata™",
        "Mølly æænigmatix straße",
        "ドラゴンズドグマ",
        "7 Days to Die",
        "Con",
        past_cap_name.as_str(),
    ];

    for name in corpus {
        if let Some(slug) = derive_slug(name) {
            assert!(
                slug.is_ascii(),
                "`{name}` derived a non-ASCII slug — the output is promised pure ASCII, so \
                 octets and chars are equal: {slug:?}"
            );
            assert!(
                !slug.contains('/') && !slug.contains('\\'),
                "a slug is one path component and can never carry a separator: {slug:?}"
            );
        }
    }
}
