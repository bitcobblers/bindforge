//! The hosted lookup policy: the ordered decision table one decoded key walks —
//! corpus, issued, then the per-source token postures — and the honesty of every
//! miss (`027-L3`).
//!
//! proves: design 027's Test strategy #3 — "fresh snapshot + appid hit → mint
//! path; fresh + appid miss (and a non-decimal external id) → `200 []`; stale
//! snapshot → `503 steam snapshot stale`; absent → `503 no steam snapshot`;
//! `--no-steam` → `503 steam resolver inactive`; `epic/…` → `503 epic resolver
//! inactive`; `local/…` → `200 []`; a curated steam key answers `corpus` with the
//! issued copy invisible (`027-L5`)" — with Test strategy #13's stale-posture half
//! ("a planted snapshot whose `fetched_at` is older than `027-B2`'s age bound turns
//! steam misses into the named `503` while corpus and issued answers keep serving").
//!
//! Two notes on what this suite can and cannot reach:
//!
//! * The policy's two cold rows — a snapshot past `027-B2`'s age bound, and no
//!   snapshot at all — have **no route through a booting binary**. §The steam
//!   snapshot resolver's startup order loads a snapshot only when it is "present,
//!   loadable, **and** fresh, else pull in full", so a hosted run planted on a stale
//!   or absent snapshot pulls the compile-time-pinned authority before it binds — a
//!   dial Test strategy #17 rules out of every test. Those rows are pinned against
//!   the policy's own in-process face, `bindforge_registry::state`, exactly as
//!   `tests/steam.rs` drives the pull codec there: the design's decision table is
//!   that face's function over the loaded models, and `027-L1` guarantees a request
//!   thread can reach nothing else. The reachable cold posture — `--no-steam` — keeps
//!   its socket legs, and the snapshot loader's own refusals are pinned in
//!   `tests/steam.rs`.
//! * A stale snapshot in *service* is still a real state — §The store pull keeps an
//!   old snapshot serving through a failed delta until the age bound converts its
//!   misses — but the refresh thread has no loopback seam in the binary, so that
//!   posture is pinned where the design puts it: on the decision table, and on
//!   `health`'s body in `tests/health_route.rs`.
//!
//! Nothing here dials or resolves anything: a fresh planted snapshot keeps every
//! hosted run off the pull path, and `--no-steam` never reads the file at all
//! (Test strategy #17).

mod common;

use bindforge_registry::constants::MAX_CATALOG_DOCUMENT_OCTETS;
use bindforge_registry::corpus::Catalog;
use bindforge_registry::retired::RetiredIds;
use bindforge_registry::state::{Answer, Provenance, State, SteamPosture};
use bindforge_registry::steam::SteamSnapshot;
use common::{
    Registry, TempDir, body_text, corpus_doc, json_body, now_unix, status_code, status_line,
};
use std::path::{Path, PathBuf};

const KEY: &str = "bindforge-test-steam-key-0f1e2d3c";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";
const SNAPSHOT: &str = "bindforge/registry/steam-snapshot.json";
const ISSUED: &str = "bindforge/registry/issued-v1.json";

/// `027-B2`'s age bound, in seconds: freshness is `now − fetched_at ≤` this.
const MAX_AGE_SECONDS: u64 = 604_800;

fn xdg_of(data: &TempDir) -> String {
    data.path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string()
}

fn hosted_args<'a>(corpus: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["serve", "--root", "--corpus", corpus];
    args.extend_from_slice(extra);
    args
}

fn plant_snapshot(data: &TempDir, fetched_at: u64, apps: &[(u64, &str)]) {
    let mut ordered: Vec<(u64, &str)> = apps.to_vec();
    ordered.sort_by_key(|(appid, _)| *appid);
    assert_eq!(
        ordered,
        apps.to_vec(),
        "a snapshot fixture out of `appid` order is a broken fixture: the loader refuses it"
    );
    let entries: Vec<serde_json::Value> = ordered
        .iter()
        .map(|(appid, name)| serde_json::json!({ "appid": appid, "name": name }))
        .collect();
    let document = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "fetched_at": fetched_at,
        "apps": entries,
    }))
    .expect("a snapshot document serialises");
    data.write(SNAPSHOT, document.as_bytes());
}

fn plant_issued(data: &TempDir, revision: i64, games: &[(&str, &str, &[&str])]) {
    data.write(ISSUED, corpus_doc(revision, games).as_bytes());
}

fn corpus_in(data: &TempDir, games: &[(&str, &str, &[&str])]) -> PathBuf {
    data.write("corpus.json", corpus_doc(7, games).as_bytes())
}

fn serve(data: &TempDir, corpus: &Path, extra: &[&str]) -> Registry {
    Registry::start(
        "policy",
        &hosted_args(&corpus.display().to_string(), extra),
        &[("XDG_DATA_HOME", xdg_of(data).as_str()), (KEY_VAR, KEY)],
    )
}

/// Does one answer's own words name this condition? The reason phrase is what the
/// shipped CLI shows a user, and the design names these refusals in prose, so the
/// check tolerates `-`/`_`/space spellings.
fn names(answer: &[u8], condition: &str) -> bool {
    let needle = condition.to_ascii_lowercase().replace('_', " ");
    for text in [
        status_line(answer).to_ascii_lowercase(),
        body_text(answer).to_ascii_lowercase(),
    ] {
        if text.replace('_', " ").contains(&needle) {
            return true;
        }
    }
    false
}

fn empty_answer(answer: &[u8]) -> bool {
    status_code(answer) == Some(200) && json_body(answer).as_array().is_some_and(Vec::is_empty)
}

fn issued_document(data: &TempDir) -> serde_json::Value {
    let text = std::fs::read_to_string(data.join(ISSUED)).expect("the issued document is readable");
    serde_json::from_str(&text).expect("the persisted issued document is a catalog document")
}

fn issued_ids(document: &serde_json::Value) -> Vec<String> {
    document["games"]
        .as_array()
        .expect("a games list")
        .iter()
        .map(|game| game["game_id"].as_str().expect("a game_id").to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// The policy's own face.
//
// §The hosted lookup policy is one function over the loaded models — corpus, issued,
// retired, snapshot, resolver posture — and the two cold rows have no route through a
// booting binary (the header says why). These helpers load the same three documents a
// hosted run loads and hand them to `State::hosted`, so a leg can ask the policy a
// question the design answers without a process that would have to dial the store.
// ---------------------------------------------------------------------------

/// The models of one hosted instance: a corpus holding `curated-name` at
/// `steam/900001`, an issued document holding `issued-name` at `steam/9100802`, an
/// absent (therefore empty) retired list, and the snapshot and resolver posture the
/// leg is asking about.
fn policy(data: &TempDir, snapshot: Option<SteamSnapshot>, posture: SteamPosture) -> State {
    let corpus = data.write(
        "policy-corpus.json",
        corpus_doc(
            3,
            &[("curated-name", "Curated Name", &["steam/900001"][..])],
        )
        .as_bytes(),
    );
    let issued = data.write(
        "policy-issued.json",
        corpus_doc(2, &[("issued-name", "Issued Name", &["steam/9100802"][..])]).as_bytes(),
    );
    State::hosted(
        Catalog::load(&corpus, MAX_CATALOG_DOCUMENT_OCTETS).expect("the corpus fixture loads"),
        Catalog::load(&issued, MAX_CATALOG_DOCUMENT_OCTETS).expect("the issued fixture loads"),
        RetiredIds::load(&data.join("policy-retired-ids.json"))
            .expect("an absent retired list is the empty set"),
        snapshot,
        posture,
    )
}

/// One snapshot fixture for the in-process face, in the `027-C3` shape.
fn snapshot_at(fetched_at: u64, apps: &[(u64, &str)]) -> SteamSnapshot {
    SteamSnapshot::for_test(fetched_at, apps)
}

/// The `027-L3` name of a refusal the policy answered with — the design's own words,
/// because `reason()` is what the shipped status line shows a user.
fn refusal_reason(answer: &Answer) -> String {
    match answer {
        Answer::Refusal(named) => named.reason().to_string(),
        Answer::Candidates {
            candidates,
            provenance,
        } => panic!(
            "the design promises a named refusal here; the policy answered {provenance:?} with \
             {candidates:?} instead"
        ),
    }
}

/// The `game_id`s of an answer the policy must not have refused.
fn answered_ids(answer: &Answer) -> Vec<String> {
    match answer {
        Answer::Candidates { candidates, .. } => candidates
            .iter()
            .map(|candidate| candidate.game_id.clone())
            .collect(),
        Answer::Refusal(named) => panic!(
            "the design promises a candidate list here; the policy refused `{}`",
            named.reason()
        ),
    }
}

/// The provenance an answer carries, in the spelling §Hosted mode gives `query`.
fn answered_from(answer: &Answer) -> &'static str {
    let provenance = match answer {
        Answer::Candidates { provenance, .. } => provenance,
        Answer::Refusal(named) => panic!(
            "the design promises a candidate list here; the policy refused `{}`",
            named.reason()
        ),
    };
    match provenance {
        Provenance::Corpus => "corpus",
        Provenance::Issued => "issued",
        Provenance::None => "none",
    }
}

/// proves: Test strategy #3's first cell — a fresh snapshot with a matching appid
/// sends the key down the mint path: the answer is the minted identity and the
/// issued document holds the whole record.
#[test]
fn a_fresh_snapshot_appid_hit_answers_the_mint_path() {
    let data = TempDir::new("policy-mint-hit");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_100_501, "Left 4 Dead 2")]);
    let server = serve(&data, &corpus, &[]);

    let answer = server.get("/v1/candidates?slug=steam%2F9100501");
    assert_eq!(
        status_code(&answer),
        Some(200),
        "a fresh snapshot is entitled to answer its own appids: {}",
        status_line(&answer)
    );
    assert_eq!(
        common::game_ids(&answer),
        vec!["left-4-dead-2".to_string()],
        "the mint rule's answer, and nothing else"
    );

    let document = issued_document(&data);
    assert_eq!(
        issued_ids(&document),
        vec!["left-4-dead-2".to_string()],
        "`027-C4`: the whole issued record is persisted, not just served"
    );
    let record = &document["games"][0];
    assert_eq!(
        record["name"].as_str(),
        Some("Left 4 Dead 2"),
        "the record carries the snapshot's display name (`027-C5`)"
    );
    assert_eq!(
        record["source_keys"].as_array().expect("source_keys").len(),
        1,
        "the minted record is keyed by the queried `steam/<appid>`"
    );
    assert!(
        record["source_keys"]
            .as_array()
            .expect("source_keys")
            .iter()
            .any(|key| key.as_str() == Some("steam/9100501")),
        "and by exactly that key: {record}"
    );
}

/// proves: Test strategy #3's second cell — a fresh snapshot that knows every game
/// appid the games-only pull asked for answers "never heard of it" honestly for an
/// appid it does not carry, and writes nothing.
#[test]
fn a_fresh_snapshot_appid_miss_answers_an_empty_list_and_writes_nothing() {
    let data = TempDir::new("policy-fresh-miss");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_100_501, "Left 4 Dead 2")]);
    let server = serve(&data, &corpus, &[]);

    let answer = server.get("/v1/candidates?slug=steam%2F9199999");
    assert!(
        empty_answer(&answer),
        "every page was examined and every record handed over is in the snapshot, so this \
         is the claim it is entitled to make (`027-L3`): {}",
        status_line(&answer)
    );
    assert!(
        !data.join(ISSUED).exists(),
        "a key nothing mints must not create an issued document: {:?}",
        data.files_below("bindforge")
    );
}

/// proves: Test strategy #3's second cell, parenthesised half — a non-decimal
/// external id is not a game appid the games-only snapshot can ever hold, so it
/// joins the honest `200 []` rather than a refusal (§The hosted lookup policy step
/// 5).
#[test]
fn a_non_decimal_steam_external_id_answers_an_empty_list() {
    let data = TempDir::new("policy-non-decimal");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_100_501, "Left 4 Dead 2")]);
    let server = serve(&data, &corpus, &[]);

    let answer = server.get(&common::slug_query("steam/not-an-appid"));
    assert!(
        empty_answer(&answer),
        "a non-decimal external id is a `200 []` against a fresh snapshot, never a `503`: \
         {}",
        status_line(&answer)
    );
    assert!(
        !data.join(ISSUED).exists(),
        "and it mints nothing: {:?}",
        data.files_below("bindforge")
    );
}

/// proves: Test strategy #3's `local/…` cell with `027-C2` and `026` root mode's
/// shipped posture — a `local/…` key is never a candidate for any resolver and the
/// registry holds no local knowledge, so it answers `200 []`, not a refusal.
#[test]
fn a_local_key_is_never_a_resolver_candidate() {
    let data = TempDir::new("policy-local");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_100_501, "Left 4 Dead 2")]);
    let server = serve(&data, &corpus, &[]);

    for key in ["local/Some.Dir", "local/Counter_2_Strike"] {
        let answer = server.get(&common::slug_query(key));
        assert!(
            empty_answer(&answer),
            "`{key}` is `200 []` exactly as `026` root mode answered it (§The hosted lookup \
             policy step 3): {}",
            status_line(&answer)
        );
    }
    assert!(
        !data.join(ISSUED).exists(),
        "a `local/…` key mints nothing: {:?}",
        data.files_below("bindforge")
    );
}

/// proves: Test strategy #3's `epic/…` cell with `027-C2` and `027-R1` — the
/// reserved, inactive slot answers the named `503 epic resolver inactive`, because
/// `[]` would be a "never heard of it" claim this registry cannot make.
#[test]
fn an_epic_key_is_the_named_refusal_never_an_empty_list() {
    let data = TempDir::new("policy-epic");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_100_501, "Left 4 Dead 2")]);
    let server = serve(&data, &corpus, &[]);

    for key in [
        "epic/77",
        "epic/f4a9d1c0b6e54f8f9a2d7c3b1e0a5d6c",
        "epic/Some.Storefront.AppName",
    ] {
        let answer = server.get(&common::slug_query(key));
        assert_eq!(
            status_code(&answer),
            Some(503),
            "`{key}` hits the reserved inactive slot: the named `503`, never `[]` \
             (`027-C2`): {}",
            status_line(&answer)
        );
        assert!(
            names(&answer, "epic resolver inactive"),
            "the refusal names its reason (`epic resolver inactive`) because the status \
             line is all the shipped CLI shows a user: {}",
            status_line(&answer)
        );
        assert!(
            !body_text(&answer).contains("[]"),
            "an empty list would be the fabricated claim this posture exists to avoid: {:?}",
            body_text(&answer)
        );
    }
    assert!(
        !data.join(ISSUED).exists(),
        "the inert slot mints nothing: {:?}",
        data.files_below("bindforge")
    );
}

/// proves: Test strategy #3's last cell with `027-L5` — curation outranks the
/// mint: a curated steam key answers from the corpus alone, and the issued record
/// for the same key stays invisible to clients.
#[test]
fn a_curated_steam_key_answers_alone_with_the_issued_copy_invisible() {
    let data = TempDir::new("policy-l5");
    let corpus = corpus_in(
        &data,
        &[("curated-name", "Curated Name", &["steam/9100601"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_100_601, "Curated Name")]);
    plant_issued(
        &data,
        4,
        &[("shadowed-name", "Curated Name", &["steam/9100601"][..])],
    );
    let server = serve(&data, &corpus, &[]);

    let answer = server.get("/v1/candidates?slug=steam%2F9100601");
    assert_eq!(
        common::game_ids(&answer),
        vec!["curated-name".to_string()],
        "the corpus answers alone; the issued copy is never consulted for that key \
         (`027-L5`): {}",
        body_text(&answer)
    );
    assert_eq!(
        issued_ids(&issued_document(&data)),
        vec!["shadowed-name".to_string()],
        "and the issued record stays where curation can see it, untouched"
    );
}

/// proves: §The hosted lookup policy step 2 with `027-B3`'s dedup half — a key the
/// issued document already holds answers from it with no mint and no write, "at the
/// cap or anywhere".
#[test]
fn an_issued_key_answers_again_with_zero_writes() {
    let data = TempDir::new("policy-issued-hit");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_100_701, "Half-Life 2")]);
    plant_issued(
        &data,
        11,
        &[("half-life-2", "Half-Life 2", &["steam/9100701"][..])],
    );
    let server = serve(&data, &corpus, &[]);

    let path = data.join(ISSUED);
    let before = std::fs::read(&path).expect("the planted issued document is readable");
    let before_inode = inode_of(&path);

    let answer = server.get("/v1/candidates?slug=steam%2F9100701");
    assert_eq!(
        common::game_ids(&answer),
        vec!["half-life-2".to_string()],
        "the issued join answers"
    );
    assert_eq!(
        common::game_ids(&server.get("/v1/candidates?slug=steam%2F9100701")),
        vec!["half-life-2".to_string()],
        "and keeps answering it"
    );
    assert_eq!(
        std::fs::read(&path).expect("the issued document survives"),
        before,
        "a key that already has a record never pays the mint path again"
    );
    assert_eq!(
        inode_of(&path),
        before_inode,
        "no rewrite happened either: the rename discipline always changes the inode"
    );
}

/// proves: Test strategy #3's stale cell with #13's first half — a snapshot past
/// `027-B2`'s age bound turns every steam lookup into the named `503 steam snapshot
/// stale`, the unknown appid *and* the appid the snapshot itself carries (a cold
/// store-resolver never answers `[]`), while the corpus and the issued document keep
/// answering exactly as before. Pinned on the policy's own face: a booting instance
/// is never planted on a stale snapshot (§The steam snapshot resolver loads only a
/// snapshot that is "present, loadable, and fresh"), so this is the only route that
/// asks the question without dialling the store.
#[test]
fn a_snapshot_past_its_age_bound_turns_steam_misses_into_the_named_503() {
    let data = TempDir::new("policy-stale");
    let stale = now_unix().saturating_sub(MAX_AGE_SECONDS + 3_600);
    let state = policy(
        &data,
        Some(snapshot_at(stale, &[(9_100_801, "Aging Title")])),
        SteamPosture::Active,
    );
    let issued_path = data.join("policy-issued.json");
    let issued_before = std::fs::read(&issued_path).expect("the issued fixture is readable");
    let now = now_unix();

    let miss = state.candidates("steam/9199998", now);
    assert_eq!(
        refusal_reason(&miss),
        "steam snapshot stale",
        "a stale snapshot may not certify \"never heard of it\": every page of it was \
         fetched a week ago (`027-L3`)"
    );

    let known = state.candidates("steam/9100801", now);
    assert_eq!(
        refusal_reason(&known),
        "steam snapshot stale",
        "the age bound governs the *mint* path too, not just the `[]` one: an appid an \
         aging snapshot carries is not a persistable identity"
    );

    assert_eq!(
        answered_ids(&state.candidates("steam/900001", now)),
        vec!["curated-name".to_string()],
        "the corpus keeps serving through the degraded posture (§Risks: corpus and issued \
         are the half that stays up when the supply dies)"
    );
    assert_eq!(
        answered_from(&state.candidates("steam/900001", now)),
        "corpus",
        "and it stays the corpus's own answer (`027-L5`)"
    );
    assert_eq!(
        answered_ids(&state.candidates("steam/9100802", now)),
        vec!["issued-name".to_string()],
        "and so does the issued document: held keys answer forever, at the age bound or \
         anywhere (`027-B3`'s dedup half)"
    );
    assert_eq!(
        answered_from(&state.candidates("steam/9100802", now)),
        "issued",
        "from the issued document, with no mint to pay"
    );
    assert_eq!(
        std::fs::read(&issued_path).expect("the issued document survives"),
        issued_before,
        "the refusals mint nothing: an identity that cannot be persisted is not persisted, \
         and the document is not even rewritten"
    );
}

/// proves: Test strategy #3's `--no-steam` cell with #13's last clause — an
/// inactive resolver answers the named `503 steam resolver inactive` for steam keys
/// while corpus and issued answers keep serving, and `epic`/`local` keep their own
/// postures.
#[test]
fn an_inactive_resolver_names_itself_and_keeps_the_curated_half_serving() {
    let data = TempDir::new("policy-inactive");
    let corpus = corpus_in(
        &data,
        &[("curated-name", "Curated Name", &["steam/900001"][..])],
    );
    plant_issued(
        &data,
        2,
        &[("issued-name", "Issued Name", &["steam/9100802"][..])],
    );
    let server = Registry::start(
        "policy-inactive",
        &hosted_args(&corpus.display().to_string(), &["--no-steam"]),
        &[("XDG_DATA_HOME", xdg_of(&data).as_str())],
    );

    let answer = server.get("/v1/candidates?slug=steam%2F9100801");
    assert_eq!(
        status_code(&answer),
        Some(503),
        "a cold store-resolver never answers `[]` (§The hosted lookup policy): {}",
        status_line(&answer)
    );
    assert!(
        names(&answer, "steam resolver inactive"),
        "the refusal is the named `steam resolver inactive`, distinct from the stale and \
         missing-snapshot reasons: {}",
        status_line(&answer)
    );
    assert_eq!(
        common::game_ids(&server.get("/v1/candidates?slug=steam%2F900001")),
        vec!["curated-name".to_string()],
        "`--no-steam` is the operator's degraded option, and corpus and issued still serve \
         (§The steam snapshot resolver)"
    );
    assert_eq!(
        common::game_ids(&server.get("/v1/candidates?slug=steam%2F9100802")),
        vec!["issued-name".to_string()],
        "issued answers too"
    );
    assert!(
        empty_answer(&server.get(&common::slug_query("local/Some.Dir"))),
        "`local/…` keeps its `200 []` posture under `--no-steam`"
    );
    assert_eq!(
        status_code(&server.get(&common::slug_query("epic/77"))),
        Some(503),
        "`epic/…` keeps its named `503` under `--no-steam`"
    );
    assert!(
        !data.join(ISSUED).exists()
            || issued_ids(&issued_document(&data)) == vec!["issued-name".to_string()],
        "the inactive posture mints nothing"
    );
}

/// proves: §The hosted lookup policy step 5's three cold spellings — `steam snapshot
/// stale` / `no steam snapshot` / `steam resolver inactive` are three distinct named
/// refusals, not one blanket `503`, and none of them is ever `200 []`. The first two
/// rows are the ones no booting instance can be planted on, so they are asked of the
/// policy face; the third is `--no-steam`, and its words are checked where a client
/// actually reads them — on the status line the shipped CLI shows a user.
#[test]
fn the_steam_refusals_name_their_own_reason() {
    let data = TempDir::new("policy-reasons");
    let now = now_unix();
    let stale = now.saturating_sub(MAX_AGE_SECONDS + 60);

    let aged = policy(
        &data,
        Some(snapshot_at(stale, &[(9_100_901, "Some Game")])),
        SteamPosture::Active,
    );
    let absent = policy(&data, None, SteamPosture::Active);
    let inactive = policy(&data, None, SteamPosture::Inactive);

    let reasons = [
        refusal_reason(&aged.candidates("steam/9199997", now)),
        refusal_reason(&absent.candidates("steam/9199997", now)),
        refusal_reason(&inactive.candidates("steam/9199997", now)),
    ];
    assert_eq!(reasons[0], "steam snapshot stale");
    assert_eq!(reasons[1], "no steam snapshot");
    assert_eq!(reasons[2], "steam resolver inactive");
    for (left, right) in [(0, 1), (0, 2), (1, 2)] {
        assert_ne!(
            reasons[left], reasons[right],
            "the three cold postures are three distinct refusals, not one blanket `503`"
        );
    }

    // And the reachable one of the three says its own name on the wire.
    let socket_data = TempDir::new("policy-reason-inactive");
    let socket_corpus = corpus_in(
        &socket_data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    let socket_server = Registry::start(
        "policy-reason-inactive",
        &hosted_args(&socket_corpus.display().to_string(), &["--no-steam"]),
        &[("XDG_DATA_HOME", xdg_of(&socket_data).as_str())],
    );
    let line = status_line(&socket_server.get("/v1/candidates?slug=steam%2F9199997"));
    assert_eq!(
        status_code(&socket_server.get("/v1/candidates?slug=steam%2F9199997")),
        Some(503),
        "the `--no-steam` posture is a refusal, never an empty list: {line}"
    );
    assert!(
        line.to_ascii_lowercase()
            .replace('_', " ")
            .contains("inactive"),
        "the resolver-off refusal says `inactive` on the status line a bound CLI \
         shows a user: {line:?}"
    );
}

fn inode_of(path: &Path) -> u64 {
    #[cfg(unix)]
    {
        std::os::unix::fs::MetadataExt::ino(
            &std::fs::metadata(path)
                .unwrap_or_else(|err| panic!("reading {}: {err}", path.display())),
        )
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        0
    }
}
