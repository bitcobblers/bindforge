//! The mint rule (`027-C5`), seen through the serving face: plant a snapshot whose
//! display names are the vectors, query their appids, and read the identity that
//! comes back.
//!
//! proves: design 027's Test strategy #1 — "the retired suite's survey vectors
//! derive byte-identically (re-pinned from `crates/discovery/tests/slug_tests.rs`),
//! including the `g-` leading-digit spellings and the fold table; a non-Latin-only
//! name mints `steam-<appid>`; collision yields `<slug>-steam-<appid>` against a
//! holder in either document; every output satisfies `is_slug_name` ∧
//! `≤ MAX_SLUG_OCTETS` — including the over-bound cases the arithmetic allows …; a
//! suffixed id already held takes `steam-<appid>`, a held `steam-<appid>` refuses
//! `mint identity held`" — re-pinned from the `024-C2` table the design names as
//! its behavioral record (A9), and read against §Issued entries' mint steps 1–6.
//!
//! One clause of #1 has no route through the serving face and is therefore **not**
//! pinned here or anywhere else in this package: `mint identity over bound` needs a
//! `steam-<external id>` longer than `MAX_SLUG_OCTETS − 6`, and an external id that
//! long can never be a *decimal* appid the snapshot's loader admits (§Issued entries'
//! step-5 arithmetic caps the joined key at 128), so no request can put the rule on
//! that rung. It is recorded rather than papered over: the reachable half of the same
//! step — every minted id satisfies `is_slug_name` ∧ `≤ MAX_SLUG_OCTETS`, and a fully
//! held spelling is the named `mint identity held` — is pinned below, and the last
//! rung needs a mint seam of its own (`mint.rs::mint_id`) to be tested at all.

mod common;

use bindforge_core::{MAX_SLUG_OCTETS, is_slug_name};
use common::{Registry, body_text, corpus_doc, now_unix, status_code, status_line};
use std::path::Path;

const KEY: &str = "bindforge-test-steam-key-0f1e2d3c";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";

fn xdg_of(data: &common::TempDir) -> String {
    data.path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string()
}

/// One `027-C3` snapshot holding exactly these records, in ascending appid order.
fn plant_snapshot(data: &common::TempDir, apps: &[(u64, &str)]) {
    let mut ordered: Vec<(u64, &str)> = apps.to_vec();
    ordered.sort_by_key(|(appid, _)| *appid);
    assert_eq!(
        ordered,
        apps.to_vec(),
        "the fixture must be in `appid` order"
    );
    let entries: Vec<serde_json::Value> = ordered
        .iter()
        .map(|(appid, name)| serde_json::json!({ "appid": appid, "name": name }))
        .collect();
    let document = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "fetched_at": now_unix(),
        "apps": entries,
    }))
    .expect("a snapshot document serialises");
    data.write(
        "bindforge/registry/steam-snapshot.json",
        document.as_bytes(),
    );
}

/// What one `steam/<appid>` query came back with.
enum Outcome {
    /// The single `game_id` the instance answered with.
    Minted(String),
    /// The status line of the `503` it refused with.
    Refused(String),
}

impl Outcome {
    fn minted(&self) -> String {
        match self {
            Outcome::Minted(id) => id.clone(),
            Outcome::Refused(line) => panic!("expected a minted identity, got the refusal {line}"),
        }
    }

    fn refused(&self) -> String {
        match self {
            Outcome::Refused(line) => line.clone(),
            Outcome::Minted(id) => panic!("expected a named refusal, got the minted id {id}"),
        }
    }
}

/// Boots a hosted instance over one snapshot holding `apps` and a corpus holding
/// `curated`, then asks for `appid` and reports what came back.
fn ask(
    label: &str,
    apps: &[(u64, &str)],
    curated: &[(&str, &str, &[&str])],
    appid: u64,
) -> Outcome {
    let data = common::TempDir::new(label);
    let corpus = data.write("corpus.json", corpus_doc(3, curated).as_bytes());
    plant_snapshot(&data, apps);
    let server = serve(label, &data, &corpus);
    let answer = server.get(&common::slug_query(&format!("steam/{appid}")));
    match status_code(&answer) {
        Some(200) => {
            let ids = common::game_ids(&answer);
            assert_eq!(
                ids.len(),
                1,
                "one snapshot record answers exactly one candidate: {}",
                body_text(&answer)
            );
            Outcome::Minted(ids[0].clone())
        }
        Some(503) => Outcome::Refused(status_line(&answer)),
        Some(code) => panic!(
            "`steam/{appid}` answered {code}, which is neither the answer nor the named \
             `503`: {}",
            status_line(&answer)
        ),
        None => panic!("no status line: {answer:?}"),
    }
}

fn serve(label: &str, data: &common::TempDir, corpus: &Path) -> Registry {
    Registry::start(
        label,
        &["serve", "--root", "--corpus", &corpus.display().to_string()],
        &[("XDG_DATA_HOME", xdg_of(data).as_str()), (KEY_VAR, KEY)],
    )
}

/// proves: Test strategy #1's first clause — the retired `024-C2` survey table
/// re-pinned as the registry's mint: the surveyed titles, the three `g-`
/// leading-digit spellings, and the fold-table coverage boundary all mint
/// byte-identically to the record A9 names.
#[test]
fn mints_the_surveyed_vectors_the_retired_table_recorded() {
    for (index, (name, slug)) in [
        (
            "Warhammer 40,000: Darktide - Sisters Novitiate Class Deluxe Edition",
            "warhammer-40-000-darktide-sisters-novitiate-class-deluxe-edition",
        ),
        ("7 Days to Die", "g-7-days-to-die"),
        ("8BitBoy™", "g-8bitboy"),
        ("8-Bit Armies", "g-8-bit-armies"),
        ("0 A.D.", "g-0-a-d"),
        ("1000 Heads", "g-1000-heads"),
        ("3D Print Miniatures", "g-3d-print-miniatures"),
        ("NieR:Automata™", "nier-automata"),
        (
            "S.T.A.L.K.E.R.: Shadow of Chernobyl",
            "s-t-a-l-k-e-r-shadow-of-chernobyl",
        ),
        ("Assassin’s Creed III", "assassin-s-creed-iii"),
        ("COUNTER-STRIKE 2", "counter-strike-2"),
        ("HALF-LIFE 2: Episode One", "half-life-2-episode-one"),
        ("Super Mario Odyssey!", "super-mario-odyssey"),
        ("Game--Name", "game-name"),
        ("  Hello World!!!  ", "hello-world"),
    ]
    .iter()
    .enumerate()
    {
        let appid = 9_200_000 + index as u64;
        let outcome = ask(
            &format!("mint-survey-{index}"),
            &[(appid, name)],
            &[("example-game", "Example Game", &["steam/900001"][..])],
            appid,
        );
        let minted = outcome.minted();
        assert_eq!(
            minted, *slug,
            "`{name}` mints the slug the retired table recorded (§Issued entries' mint \
             rule, re-pinned per A9)"
        );
        assert!(
            is_slug_name(&minted) && minted.len() <= MAX_SLUG_OCTETS,
            "`{minted}` is not a product id: the grammar is `is_slug_name` and the bound is \
             {MAX_SLUG_OCTETS} (A10)"
        );
        assert!(
            minted.is_ascii(),
            "`{name}` minted a non-ASCII id — the mint's output is pure ASCII, so octets \
             and chars are equal: {minted:?}"
        );
    }
}

/// proves: Test strategy #1 with §Issued entries' step 1 — the finite
/// Latin-1/Latin-Extended-A single-base-letter fold table, and the unmapped
/// characters that become exactly one `-` and are never transliterated.
#[test]
fn mints_the_fold_table_and_never_transliterates_outside_it() {
    for (index, (name, slug)) in [
        ("Café du Monde", "cafe-du-monde"),
        ("Kühlstraße Depot", "kuhlstra-e-depot"),
        ("El Niño", "el-nino"),
        ("Åre Mountain Pass", "are-mountain-pass"),
        ("Vårgården", "vargarden"),
        ("Éclair Station", "eclair-station"),
        ("Über Driver", "uber-driver"),
        ("Zażółć Gęślą Jaźń", "zazo-c-gesla-jazn"),
        ("Mølly", "m-lly"),
        ("ænigmatix", "nigmatix"),
        ("straße", "stra-e"),
        ("Doppelgänger ßß", "doppelganger"),
        ("Cḗadme", "c-adme"),
    ]
    .iter()
    .enumerate()
    {
        let appid = 9_210_000 + index as u64;
        let minted = ask(
            &format!("mint-fold-{index}"),
            &[(appid, name)],
            &[("example-game", "Example Game", &["steam/900001"][..])],
            appid,
        )
        .minted();
        assert_eq!(
            minted, *slug,
            "`{name}` mints the fold-table spelling: `ø`, `æ` and `ß` are one `-` each, and \
             Latin Extended Additional is outside the covered blocks (A9)"
        );
    }
}

/// proves: Test strategy #1's second clause with §Issued entries' step 3 — a name
/// with no Latin base letter at all derives nothing, and the mint id is then
/// `steam-<appid>`: appid-unique, grammar-valid, and with no fallback chain.
#[test]
fn mints_steam_appid_for_a_name_with_no_latin_base_letter() {
    for (index, name) in [
        "ドラゴンズドグマ",
        "بازی",
        "🙂🙂",
        "!!!",
        "---",
        "ø",
        "ḗ",
        "  — · ,  ",
    ]
    .iter()
    .enumerate()
    {
        let appid = 9_220_000 + index as u64;
        let minted = ask(
            &format!("mint-empty-{index}"),
            &[(appid, name)],
            &[("example-game", "Example Game", &["steam/900001"][..])],
            appid,
        )
        .minted();
        assert_eq!(
            minted,
            format!("steam-{appid}"),
            "`{name}` derives nothing, so step 3 mints `steam-<appid>` — not a refusal, not \
             a bare separator, and no `g-` on the appid spelling"
        );
        assert!(is_slug_name(&minted) && minted.len() <= MAX_SLUG_OCTETS);
    }
}

/// proves: Test strategy #1's third clause with §Issued entries' step 4 — a
/// derived id already held by a *different* record, in the corpus or in the issued
/// document, becomes `<slug>-steam-<appid>`.
#[test]
fn mints_the_appid_suffix_when_the_plain_slug_is_held() {
    let held_in_corpus = ask(
        "mint-collision-corpus",
        &[(9_230_001, "Example Game")],
        &[
            ("example-game", "Some Other Game", &["steam/900001"][..]),
            ("filler", "Filler", &["steam/900002"][..]),
        ],
        9_230_001,
    )
    .minted();
    assert_eq!(
        held_in_corpus, "example-game-steam-9230001",
        "a corpus holder earns the appid suffix (`027-C5`'s collision spelling)"
    );

    let held_in_issued = {
        let data = common::TempDir::new("mint-collision-issued");
        let corpus = data.write(
            "corpus.json",
            corpus_doc(3, &[("filler", "Filler", &["steam/900002"][..])]).as_bytes(),
        );
        plant_snapshot(&data, &[(9_230_002, "Example Game")]);
        data.write(
            "bindforge/registry/issued-v1.json",
            corpus_doc(
                5,
                &[("example-game", "An Earlier Issue", &["steam/900003"][..])],
            )
            .as_bytes(),
        );
        let server = serve("mint-collision-issued", &data, &corpus);
        let answer = server.get("/v1/candidates?slug=steam%2F9230002");
        assert_eq!(
            status_code(&answer),
            Some(200),
            "a collision is minted, never refused: {}",
            status_line(&answer)
        );
        common::game_ids(&answer)[0].clone()
    };
    assert_eq!(
        held_in_issued, "example-game-steam-9230002",
        "…and an issued-document holder counts as held just the same (§Issued entries' \
         step 4: \"in the corpus **or** the issued document, or retired\")"
    );
}

/// proves: §Issued entries' step 5 as the arithmetic states it — a derived id that
/// the `g-` prefix or the `-steam-<appid>` suffix pushes past `MAX_SLUG_OCTETS`
/// mints `steam-<appid>` instead, while a name that lands exactly on the bound is
/// minted whole and never truncated.
#[test]
fn mints_steam_appid_when_the_bound_would_be_breached() {
    let plain_at_bound = "k".repeat(MAX_SLUG_OCTETS);
    let minted_plain = ask(
        "mint-bound-plain",
        &[(9_240_001, &plain_at_bound)],
        &[("example-game", "Example Game", &["steam/900001"][..])],
        9_240_001,
    )
    .minted();
    // §Issued entries' step 5 is the rule, and it fires on *"a mint whose chosen id
    // exceeds `MAX_SLUG_OCTETS`"*: the fold never grows a name's octets, so a name at
    // exactly the bound derives an id at exactly the bound and is minted whole. Test
    // strategy #1 lists this fixture with the two below as the cases that "all mint
    // `steam-<appid>`"; the equality case belongs to the rule text, which says
    // *exceeds*, so the boundary is pinned here as in budget and never truncated, and
    // the `g-` and suffix fixtures below are the ones the substitution exists for.
    assert_eq!(
        minted_plain, plain_at_bound,
        "a fold never exceeds the name's octets, so a name at exactly the bound is inside it: \
         step 5 refuses only what *exceeds* the bound, and truncation is not re-pinned here"
    );
    assert_eq!(minted_plain.len(), MAX_SLUG_OCTETS);
    assert!(is_slug_name(&minted_plain));

    let digit_leading = format!("9{}", "k".repeat(MAX_SLUG_OCTETS - 1));
    let minted_digit = ask(
        "mint-bound-digit",
        &[(9_240_002, &digit_leading)],
        &[("example-game", "Example Game", &["steam/900001"][..])],
        9_240_002,
    )
    .minted();
    assert_eq!(
        minted_digit, "steam-9240002",
        "step 2's `g-` adds two octets onto a name at the bound, so the chosen id is over it \
         and step 5 substitutes `steam-<appid>`"
    );

    let minted_colliding = ask(
        "mint-bound-collision",
        &[(9_240_003, &plain_at_bound)],
        &[(&plain_at_bound, "Some Other Game", &["steam/900001"][..])],
        9_240_003,
    )
    .minted();
    assert_eq!(
        minted_colliding, "steam-9240003",
        "step 4's suffix adds seven octets plus the appid's digits onto a name at the bound, \
         so the suffixed spelling is over it too"
    );

    for minted in [minted_digit, minted_colliding] {
        assert!(
            minted.len() <= MAX_SLUG_OCTETS && is_slug_name(&minted),
            "`{minted}` is the in-budget substitution id (A10)"
        );
    }
}

/// proves: Test strategy #1's fourth clause — the substitution ladder is
/// checked at every rung: a suffixed id that is *already held* takes
/// `steam-<appid>`, which earns the same held check before it is served.
#[test]
fn mints_steam_appid_when_the_suffixed_spelling_is_held() {
    let minted = ask(
        "mint-suffix-held",
        &[(9_250_001, "Example Game")],
        &[
            ("example-game", "First Holder", &["steam/900001"][..]),
            (
                "example-game-steam-9250001",
                "Second Holder",
                &["steam/900002"][..],
            ),
        ],
        9_250_001,
    )
    .minted();
    assert_eq!(
        minted, "steam-9250001",
        "corpus ids are operator-chosen grammar strings, so one can already spell the \
         suffixed form (§Issued entries' step 4)"
    );
}

/// proves: Test strategy #1's fifth clause with §Issued entries' step 6 — every
/// spelling the rule lands on held or retired by a different record is the named
/// `503 mint identity held`, and the key is curation's to solve.
#[test]
fn refuses_the_mint_when_every_spelling_is_held() {
    let outcome = ask(
        "mint-all-held",
        &[(9_260_001, "ドラゴンズドグマ")],
        &[("steam-9260001", "A Squatter", &["steam/900001"][..])],
        9_260_001,
    );
    let line = outcome.refused();
    assert!(
        line.to_ascii_lowercase().replace('-', " ").contains("held"),
        "an empty-derivation name whose `steam-<appid>` is already held is the named `503` \
         `mint identity held`: {line:?}"
    );

    let data = common::TempDir::new("mint-all-held-suffixed");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(
            3,
            &[
                ("example-game", "First Holder", &["steam/900001"][..]),
                (
                    "example-game-steam-9260002",
                    "Second Holder",
                    &["steam/900002"][..],
                ),
                ("steam-9260002", "Third Holder", &["steam/900003"][..]),
            ],
        )
        .as_bytes(),
    );
    plant_snapshot(&data, &[(9_260_002, "Example Game")]);
    let server = serve("mint-all-held-suffixed", &data, &corpus);
    let answer = server.get("/v1/candidates?slug=steam%2F9260002");
    assert_eq!(
        status_code(&answer),
        Some(503),
        "all three spellings held is a refusal, never a fourth invention: {}",
        status_line(&answer)
    );
    assert!(
        status_line(&answer)
            .to_ascii_lowercase()
            .replace('-', " ")
            .contains("held"),
        "and it names `mint identity held`: {}",
        status_line(&answer)
    );
    assert!(
        !data.join("bindforge/registry/issued-v1.json").exists(),
        "a refused mint persists nothing: {:?}",
        data.files_below("bindforge")
    );
}

/// proves: Test strategy #1's `027-L5`-adjacent convergence claim (§Options) —
/// the curated seed ids are what the rule mints from those names, `portal-two`
/// being the recorded exception, which the rule derives as `portal-2`.
#[test]
fn converges_on_the_curated_ids_it_was_measured_against() {
    for (index, (name, slug)) in [
        ("Counter-Strike 2", "counter-strike-2"),
        ("Left 4 Dead 2", "left-4-dead-2"),
        ("Dota 2", "dota-2"),
        ("Portal 2", "portal-2"),
    ]
    .iter()
    .enumerate()
    {
        let appid = 9_270_000 + index as u64;
        let minted = ask(
            &format!("mint-converge-{index}"),
            &[(appid, name)],
            &[("example-game", "Example Game", &["steam/900001"][..])],
            appid,
        )
        .minted();
        assert_eq!(
            minted, *slug,
            "mint-then-promote coheres with the curated ids because the rule derives them \
             (§Options' convergence evidence)"
        );
    }
}
