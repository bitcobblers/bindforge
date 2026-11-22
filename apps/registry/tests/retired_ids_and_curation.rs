//! Retired ids (`027-L2`) and the curation face that creates them:
//! `DELETE /v1/admin/issued/<game_id>`, the two-persists-under-one-hold delete, and
//! the promote-then-delete sequence an operator actually runs.
//!
//! proves: design 027's Test strategy #5 — "issued and corpus deletes append to
//! `retired-ids-v1.json` as the first of the hold's two persists; a derived id in
//! the retired set takes the suffix spelling; an id retired under both spellings
//! refuses `mint identity held`; a write that breaches `MAX_RETIRED_IDS_OCTETS`
//! against the rename buffer answers `retired id list full` and mutates nothing,
//! while a no-op re-delete — even against an at-cap list — still completes the
//! removal; a planted `games ∩ retired` overlap keeps the record answering; absent
//! file = empty set"; Test strategy #10 — "auth posture identical to the corpus
//! routes; absent id → the `Absent` refusal; success retires the id, bumps the
//! issued `revision`, persists atomically; the deleted key answers from the corpus
//! when curated, else re-mints under a suffix id that is *not* the retired one";
//! and Test strategy #14 — the curation flow over one live loopback socket; with
//! §The curation face's clause on the route itself — "`DELETE
//! /v1/admin/issued/<game_id>`: percent-decode the segment, validate the `game_id`
//! grammar (A10), refuse an absent record with the same `Absent` refusal shape the
//! corpus delete answers".

mod common;

use common::{
    Registry, TempDir, corpus_doc, game_path, header_values, json_body, now_unix, record_body,
    registry_document, status_code, status_line,
};
use std::path::{Path, PathBuf};

const KEY: &str = "bindforge-test-steam-key-0f1e2d3c";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";
const ISSUED: &str = "bindforge/registry/issued-v1.json";
const RETIRED: &str = "bindforge/registry/retired-ids-v1.json";

/// `MAX_RETIRED_IDS_OCTETS`, §Budgets.
const MAX_RETIRED_IDS_OCTETS: usize = 33_554_432;

fn xdg_of(data: &TempDir) -> String {
    data.path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string()
}

fn plant_snapshot(data: &TempDir, apps: &[(u64, &str)]) {
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

/// The `{version: 1, ids: […]}` document, in byte order as §Issued entries fixes it.
fn plant_retired(data: &TempDir, ids: &[String]) {
    let mut ordered = ids.to_vec();
    ordered.sort();
    assert_eq!(
        ordered,
        ids.to_vec(),
        "the retired list fixture must be in byte order"
    );
    let document = serde_json::to_string(&serde_json::json!({"version": 1, "ids": ids}))
        .expect("a retired-id document serialises");
    data.write(RETIRED, document.as_bytes());
}

fn corpus_in(data: &TempDir, games: &[(&str, &str, &[&str])]) -> PathBuf {
    data.write("corpus.json", corpus_doc(3, games).as_bytes())
}

fn serve_with(data: &TempDir, corpus: &Path, extra: &[&str]) -> Registry {
    let mut args = vec!["serve".to_string(), "--root".to_string()];
    args.extend(extra.iter().map(|arg| arg.to_string()));
    args.extend(["--corpus".to_string(), corpus.display().to_string()]);
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    Registry::start(
        "retired",
        &borrowed,
        &[("XDG_DATA_HOME", xdg_of(data).as_str()), (KEY_VAR, KEY)],
    )
}

fn serve(data: &TempDir, corpus: &Path) -> Registry {
    serve_with(data, corpus, &[])
}

fn document_at(data: &TempDir, relative: &str) -> serde_json::Value {
    let path = data.join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    serde_json::from_str(&text).expect("a persisted registry document is JSON")
}

fn retired_ids(data: &TempDir) -> Vec<String> {
    document_at(data, RETIRED)["ids"]
        .as_array()
        .expect("an ids list")
        .iter()
        .map(|id| id.as_str().expect("an id string").to_string())
        .collect()
}

fn issued_ids(data: &TempDir) -> Vec<String> {
    document_at(data, ISSUED)["games"]
        .as_array()
        .expect("a games list")
        .iter()
        .map(|game| game["game_id"].as_str().expect("a game_id").to_string())
        .collect()
}

fn mint(server: &Registry, appid: u64) -> String {
    let answer = server.get(&common::slug_query(&format!("steam/{appid}")));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "a snapshot-known key mints: {}",
        status_line(&answer)
    );
    let ids = common::game_ids(&answer);
    assert_eq!(ids.len(), 1, "exactly one candidate answers");
    ids[0].clone()
}

/// `/v1/admin/issued/<percent-encoded>` for one `<game_id>` segment (`027-C6`).
fn issued_path(game_id: &str) -> String {
    format!("/v1/admin/issued/{}", common::percent_encode(game_id))
}

fn delete_issued(server: &Registry, game_id: &str) -> Vec<u8> {
    server.request("DELETE", &issued_path(game_id), &[], "")
}

/// proves: Test strategy #5's first clause with #10's success clause — an issued
/// delete retires the id and removes the record, in that order, under one hold,
/// bumping the issued `revision`; the absent file was an empty set before it.
#[test]
fn an_issued_delete_retires_the_id_and_removes_the_record() {
    let data = TempDir::new("retired-issued-delete");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_400_001, "Disco Elysium")]);
    let server = serve(&data, &corpus);
    let id = mint(&server, 9_400_001);
    assert_eq!(id, "disco-elysium");
    assert!(
        !registry_document(data.path(), "retired-ids-v1.json").exists(),
        "nothing has been retired yet"
    );

    let answer = delete_issued(&server, &id);
    assert!(
        matches!(status_code(&answer), Some(200..=299)),
        "the issued delete is accepted: {}",
        status_line(&answer)
    );

    let document = document_at(&data, RETIRED);
    assert_eq!(
        document["version"].as_u64(),
        Some(1),
        "§Issued entries fixes `retired-ids-v1.json` as `{{version: 1, ids: […]}}`"
    );
    assert_eq!(
        retired_ids(&data),
        vec![id.clone()],
        "the id is in the retired list"
    );
    assert_eq!(
        issued_ids(&data),
        Vec::<String>::new(),
        "and the record is gone from the issued document"
    );
    assert_eq!(
        document_at(&data, ISSUED)["revision"].as_u64(),
        Some(2),
        "the issued `revision` is bumped by every mint and every issued delete"
    );

    assert_eq!(
        status_code(&server.get("/v1/candidates?slug=steam%2F9400001")),
        Some(200),
        "the deleted key answers again — under a minted identity, since the snapshot still \
         knows it: {}",
        status_line(&server.get("/v1/candidates?slug=steam%2F9400001"))
    );
    let reminted = common::game_ids(&server.get("/v1/candidates?slug=steam%2F9400001"));
    assert_eq!(
        reminted,
        vec!["disco-elysium-steam-9400001".to_string()],
        "`027-L2`: deleting a record never frees its id, so the re-mint takes the suffix \
         spelling, never the retired one"
    );
    assert_eq!(
        retired_ids(&data),
        vec![id],
        "and the retired list still holds the first identity"
    );
}

/// proves: Test strategy #5's first clause, corpus half — "**Every** delete —
/// issued or corpus — is two persists under one mutation-lock hold, retired first"
/// (§Issued entries' retired ids).
#[test]
fn a_corpus_delete_retires_its_id_too() {
    let data = TempDir::new("retired-corpus-delete");
    let corpus = corpus_in(
        &data,
        &[(
            "curated-understudy",
            "Curated Understudy",
            &["steam/9410001"][..],
        )],
    );
    plant_snapshot(&data, &[(9_410_001, "Curated Understudy")]);
    let server = serve(&data, &corpus);

    let answer = server.request("DELETE", &game_path("curated-understudy"), &[], "");
    assert!(
        matches!(status_code(&answer), Some(200..=299)),
        "the corpus delete is accepted: {}",
        status_line(&answer)
    );
    assert_eq!(
        retired_ids(&data),
        vec!["curated-understudy".to_string()],
        "a corpus delete appends to `retired-ids-v1.json` exactly as an issued delete does"
    );
    assert_eq!(
        common::game_ids(&server.get("/v1/candidates?slug=steam%2F9410001")),
        vec!["curated-understudy-steam-9410001".to_string()],
        "and the retired id is never minted again: the next identity is the suffix spelling"
    );
}

/// proves: Test strategy #5's second clause — a derived id that is in the retired
/// set takes `<slug>-steam-<appid>`, and the check spans corpus ∪ issued ∪ retired
/// (§Issued entries' step 4/6 held set).
#[test]
fn a_retired_plain_id_sends_the_mint_to_the_suffix_spelling() {
    let data = TempDir::new("retired-suffix");
    let corpus = corpus_in(&data, &[("filler", "Filler", &["steam/900001"][..])]);
    plant_snapshot(&data, &[(9_420_001, "Example Game")]);
    plant_retired(&data, &["example-game".to_string()]);
    let server = serve(&data, &corpus);

    assert_eq!(
        mint(&server, 9_420_001),
        "example-game-steam-9420001",
        "the plain spelling is retired, so the mint falls to the appid suffix"
    );
}

/// proves: Test strategy #5's third clause — an id retired under *both* spellings
/// is the named `503 mint identity held`: an ordinal fallback is not revived.
#[test]
fn a_retired_pair_of_spellings_refuses_the_mint_by_name() {
    let data = TempDir::new("retired-both");
    let corpus = corpus_in(&data, &[("filler", "Filler", &["steam/900001"][..])]);
    plant_snapshot(&data, &[(9_430_001, "Example Game")]);
    plant_retired(
        &data,
        &[
            "example-game".to_string(),
            "example-game-steam-9430001".to_string(),
            "steam-9430001".to_string(),
        ],
    );
    let server = serve(&data, &corpus);

    let answer = server.get("/v1/candidates?slug=steam%2F9430001");
    assert_eq!(
        status_code(&answer),
        Some(503),
        "every spelling the rule can land on is held or retired: the key is curation's to \
         solve (§Issued entries' step 6): {}",
        status_line(&answer)
    );
    assert!(
        status_line(&answer)
            .to_ascii_lowercase()
            .replace('-', " ")
            .contains("held"),
        "the refusal names `mint identity held`: {}",
        status_line(&answer)
    );
    assert!(
        !data.join(ISSUED).exists(),
        "and nothing is persisted: {:?}",
        data.files_below("bindforge")
    );
}

/// proves: Test strategy #10's `Absent` clause — a `<game_id>` the issued document
/// does not hold is refused in the shape the corpus delete answers, and retires
/// nothing.
#[test]
fn deleting_an_absent_issued_id_retires_nothing() {
    let data = TempDir::new("retired-absent");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_440_001, "Hades")]);
    let server = serve(&data, &corpus);

    let answer = delete_issued(&server, "never-issued");
    assert!(
        matches!(status_code(&answer), Some(400..=499)),
        "an absent issued record is refused, not silently accepted: {}",
        status_line(&answer)
    );
    assert!(
        !data.join(RETIRED).exists(),
        "a refused delete retires nothing: {:?}",
        data.files_below("bindforge")
    );
    assert!(
        !data.join(ISSUED).exists(),
        "and writes no issued document either"
    );

    let curated = server.request("DELETE", &game_path("example-game"), &[], "");
    assert!(
        matches!(status_code(&curated), Some(200..=299)),
        "the corpus route is the one that holds this id: {}",
        status_line(&curated)
    );
}

/// proves: §The curation face's clause on the route, which Test strategy #10's
/// route summary carries — the issued delete "percent-decode[s] the segment,
/// validate[s] the `game_id` grammar (A10), [and] refuse[s] an absent record with
/// the same `Absent` refusal shape the corpus delete answers". Read with the face's
/// "identical enablement/auth rule", that is *one* segment rule for both routes, so
/// this leg pins the issued route's answer to be the corpus route's answer for the
/// same decoded `<game_id>`: outside A10's grammar (`is_slug_name` — first byte
/// ASCII-lowercase, bytes in `[a-z0-9-]`, `MAX_SLUG_OCTETS = 128`) in every spelling
/// the wire can carry (dot-dot, an encoded separator, a control byte, a dot, a
/// leading digit, an uppercase or underscore byte, the empty and the over-bound
/// segment), and inside it but unheld — the control row that keeps the table from
/// being a survey of refusals alone. None of them writes: the minted record is still
/// the only one there, byte-identical, and no id is retired. The corpus route's own
/// decode-then-validate twin is `slug_decoding.rs`.
#[test]
fn an_issued_delete_segment_answers_exactly_as_the_corpus_route_answers_it() {
    let data = TempDir::new("curation-segments");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_455_001, "Outer Wilds")]);
    let server = serve(&data, &corpus);
    let minted = mint(&server, 9_455_001);
    let issued_before = std::fs::read(data.join(ISSUED)).expect("the mint is on disk");
    let mut tree_before = data.files_below(".");
    tree_before.sort();

    // Each entry is the *decoded* segment; `issued_path`/`game_path` put its
    // percent-encoded form on the socket, so the registry must decide after
    // decoding — the rule slug_decoding.rs pins for the query side.
    let long_id = "a".repeat(129);
    let segments = [
        ("dot-dot", "../../etc/passwd"),
        ("encoded-separator", "steam/9455001"),
        ("control-byte", "outer\nwilds"),
        ("dot", "outer.wilds"),
        ("leading-digit", "9-outer-wilds"),
        ("uppercase", "Outer-Wilds"),
        ("underscore", "outer_wilds"),
        ("empty", ""),
        ("over-bound", long_id.as_str()),
        ("grammar-valid-but-unheld", "outer-wilds-again"),
    ];
    for (label, segment) in segments {
        let issued = server.request("DELETE", &issued_path(segment), &[], "");
        let curated = server.request("DELETE", &game_path(segment), &[], "");
        assert_eq!(
            status_code(&issued),
            status_code(&curated),
            "`{label}` (`{segment:?}` decoded) answers the issued route one way and the corpus \
             route another; the grammar and the `Absent` refusal are one rule for both faces: \
             {} against {}",
            status_line(&issued),
            status_line(&curated),
        );
        assert!(
            !matches!(status_code(&issued), Some(200..=299)),
            "`{label}` (`{segment:?}` decoded) names no held record and must not be answered \
             as if it did: {}",
            status_line(&issued),
        );
    }

    assert_eq!(
        std::fs::read(data.join(ISSUED)).expect("the issued document survives"),
        issued_before,
        "no refused segment rewrote the issued document"
    );
    assert_eq!(
        issued_ids(&data),
        vec![minted],
        "and the minted record is still the only one in it"
    );
    let mut tree_after = data.files_below(".");
    tree_after.sort();
    assert_eq!(
        tree_after, tree_before,
        "a refused segment — dot-dot and control-byte spellings included — leaves the \
         redirected `<data>` holding exactly what it held before: no new file, no stray \
         tmp, nothing outside `bindforge/registry/`"
    );
    assert!(
        !data.join(RETIRED).exists(),
        "nor did any of them retire an id: {:?}",
        data.files_below("bindforge")
    );
}

/// proves: Test strategy #10's first clause — the issued delete route carries the
/// corpus routes' auth posture exactly: a routable bind without a token makes it
/// the flat `404`, and with a token configured every unauthenticated call is the
/// `401` that names the scheme.
#[test]
fn the_issued_delete_route_carries_the_admin_auth_posture() {
    let data = TempDir::new("retired-auth-off");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_450_001, "Hades")]);
    let disabled = serve_with(&data, &corpus, &["--host", "0.0.0.0"]);

    let answer = delete_issued(&disabled, "whatever");
    assert_eq!(
        status_code(&answer),
        Some(404),
        "an admin face disabled by an ungated routable bind answers `404`, never a `401` \
         oracle (`026-C5`, carried unchanged by `027-C6`): {}",
        status_line(&answer)
    );
    assert_eq!(
        status_line(&disabled.get("/v1/no-such-route")),
        status_line(&answer),
        "one flat `404`, publishing nothing about the new route"
    );
    drop(disabled);

    let token = "s3cret-token";
    let data = TempDir::new("retired-auth-on");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_450_002, "Hades")]);
    let gated = serve_with(
        &data,
        &corpus,
        &["--host", "0.0.0.0", "--admin-token", token],
    );
    let minted = mint(&gated, 9_450_002);

    let unauthenticated = delete_issued(&gated, &minted);
    assert_eq!(
        status_code(&unauthenticated),
        Some(401),
        "no credential on a routable bind is the `401`: {}",
        status_line(&unauthenticated)
    );
    assert!(
        header_values(&unauthenticated, "WWW-Authenticate")
            .join(",")
            .to_ascii_lowercase()
            .contains("bearer"),
        "the `401` names the scheme it expects (A20): {:?}",
        common::header_names(&unauthenticated)
    );
    assert!(
        !common::body_text(&unauthenticated).contains(token),
        "the configured token never appears in a refusal body"
    );
    assert_eq!(
        issued_ids(&data),
        vec![minted.clone()],
        "a refused delete mutates nothing"
    );

    let authorized = gated.with_token("DELETE", &issued_path(&minted), token, "");
    assert!(
        matches!(status_code(&authorized), Some(200..=299)),
        "the bearer token is accepted on the new route exactly as on the corpus routes: {}",
        status_line(&authorized)
    );
    assert_eq!(retired_ids(&data), vec![minted]);
}

/// proves: Test strategy #5's fourth clause — a retired write whose rename buffer
/// would breach `MAX_RETIRED_IDS_OCTETS` answers `retired id list full` and mutates
/// nothing, while a no-op re-delete against that same at-cap list still completes
/// the removal, because the append is a set insert.
#[test]
fn an_at_cap_retired_list_refuses_a_growing_delete_and_completes_a_no_op_one() {
    let data = TempDir::new("retired-cap");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_460_001, "Growing Name")]);

    // Fill the list to the largest set that still fits: every id is 128 octets, so
    // one more costs 131 octets and the rename buffer is what the cap measures.
    let prefix = "{\"version\":1,\"ids\":[";
    let suffix = "]}";
    let mut ids: Vec<String> = Vec::new();
    let mut length = prefix.len() + suffix.len();
    loop {
        let id = format!("r{:0>127}", ids.len());
        let cost = if ids.is_empty() {
            id.len() + 2
        } else {
            id.len() + 3
        };
        if length + cost > MAX_RETIRED_IDS_OCTETS {
            break;
        }
        length += cost;
        ids.push(id);
    }
    assert!(
        !ids.is_empty() && length <= MAX_RETIRED_IDS_OCTETS,
        "the planted list must fit: {length} octets"
    );
    assert!(
        length + 133 > MAX_RETIRED_IDS_OCTETS,
        "one more 128-octet id must breach the cap: {length} + 133"
    );
    plant_retired(&data, &ids);
    let before_retired = std::fs::read(data.join(RETIRED)).expect("the planted list is readable");
    assert!(
        before_retired.len() <= MAX_RETIRED_IDS_OCTETS,
        "the file the loader reads is inside its bound"
    );

    // One record whose id is *outside* the list (its insert grows the buffer) and
    // one whose id is already inside it (a set-insert no-op).
    let outside = format!("r{:0>127}", ids.len());
    let inside = ids[ids.len() - 1].clone();
    data.write(
        ISSUED,
        corpus_doc(
            9,
            &[
                (outside.as_str(), "Growing Name", &["steam/9460001"][..]),
                (inside.as_str(), "Already Retired", &["steam/9460002"][..]),
            ],
        )
        .as_bytes(),
    );
    let server = serve(&data, &corpus);
    let before_issued = std::fs::read(data.join(ISSUED)).expect("the planted issued doc");
    let before_stats = json_body(&server.stats(None));

    let growing = delete_issued(&server, &outside);
    let code = status_code(&growing);
    assert!(
        matches!(code, Some(400..=599)),
        "a delete whose retired write would breach the cap is refused, never failed open \
         (§Issued entries: \"failing open there re-opens a retired id\"): {}",
        status_line(&growing)
    );
    let line = status_line(&growing).to_ascii_lowercase();
    let body = common::body_text(&growing).to_ascii_lowercase();
    assert!(
        (line.contains("retired") || body.contains("retired"))
            && (line.contains("full")
                || line.contains("cap")
                || body.contains("full")
                || body.contains("cap")),
        "the refusal is the named `retired id list full`: {} / {:?}",
        status_line(&growing),
        body
    );
    assert_eq!(
        std::fs::read(data.join(RETIRED)).expect("the retired list survives"),
        before_retired,
        "the refusal mutates nothing"
    );
    assert_eq!(
        std::fs::read(data.join(ISSUED)).expect("the issued document survives"),
        before_issued,
        "and the record stays where it was: the delete never began"
    );
    assert_eq!(
        json_body(&server.stats(None)),
        before_stats,
        "no model or `revision` moved"
    );

    let no_op = delete_issued(&server, &inside);
    assert!(
        matches!(status_code(&no_op), Some(200..=299)),
        "a no-op re-delete against an at-cap list still completes the removal (revision \
         log r3): {}",
        status_line(&no_op)
    );
    assert_eq!(
        std::fs::read(data.join(RETIRED)).expect("the retired list is unchanged"),
        before_retired,
        "the id was already in the set, so the append is a no-op and the buffer does not grow"
    );
    assert_eq!(
        issued_ids(&data),
        vec![outside],
        "and the record is gone: the crash-window overlap is the recoverable state it is"
    );
}

/// proves: Test strategy #5's fifth clause — the `games ∩ retired ≠ ∅` state a run
/// that died between the two persists leaves is legal: the record keeps answering,
/// and the retried delete completes the removal.
#[test]
fn a_planted_games_retired_overlap_keeps_the_record_answering() {
    let data = TempDir::new("retired-overlap");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_470_001, "Outer Wilds")]);
    plant_retired(&data, &["outer-wilds".to_string()]);
    data.write(
        ISSUED,
        corpus_doc(5, &[("outer-wilds", "Outer Wilds", &["steam/9470001"][..])]).as_bytes(),
    );
    let server = serve(&data, &corpus);

    assert_eq!(
        common::game_ids(&server.get("/v1/candidates?slug=steam%2F9470001")),
        vec!["outer-wilds".to_string()],
        "the record keeps answering even though its id is already in the retired list \
         (§Issued entries: \"the legal, recoverable state\")"
    );

    let answer = delete_issued(&server, "outer-wilds");
    assert!(
        matches!(status_code(&answer), Some(200..=299)),
        "the retried delete completes the removal: {}",
        status_line(&answer)
    );
    assert_eq!(
        retired_ids(&data),
        vec!["outer-wilds".to_string()],
        "the already-present id is a no-op insert, never the duplicate that would break the \
         file's byte order"
    );
    assert_eq!(
        issued_ids(&data),
        Vec::<String>::new(),
        "and the record is removed"
    );
}

/// proves: Test strategy #5's last clause — an absent retired document is the empty
/// set: a mint with no `retired-ids-v1.json` present succeeds, and no retired
/// document is created by a mint alone.
#[test]
fn an_absent_retired_document_is_the_empty_set() {
    let data = TempDir::new("retired-absent-file");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_480_001, "Returnal")]);
    let server = serve(&data, &corpus);

    assert_eq!(
        mint(&server, 9_480_001),
        "returnal",
        "with no retired list, nothing is retired and the plain mint stands"
    );
    assert!(
        !registry_document(data.path(), "retired-ids-v1.json").exists(),
        "a mint retires nothing: {:?}",
        data.files_below("bindforge")
    );
}

/// proves: Test strategy #14 — the curation flow over one live socket: a corpus
/// upsert shadowing an issued key, the issued delete, `stats.issued` decrementing,
/// the key still answered from the corpus, and a corpus delete of that key making
/// the next query mint a suffix id, never the retired one.
#[test]
fn the_promote_then_delete_sequence_keeps_the_key_answered_and_the_id_retired() {
    let data = TempDir::new("curation-flow");
    let corpus = corpus_in(&data, &[("filler", "Filler", &["steam/900001"][..])]);
    plant_snapshot(&data, &[(9_490_001, "Balatro")]);
    let corpus_path = corpus.clone();
    let server = serve(&data, &corpus_path);
    let minted = mint(&server, 9_490_001);
    assert_eq!(minted, "balatro");
    assert_eq!(json_body(&server.stats(None))["issued"].as_u64(), Some(1));

    let promoted = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("balatro", "Balatro", &["steam/9490001"]),
    );
    assert!(
        matches!(status_code(&promoted), Some(200..=299)),
        "promotion begins with the whole curated record: {}",
        status_line(&promoted)
    );
    assert_eq!(
        common::game_ids(&server.get("/v1/candidates?slug=steam%2F9490001")),
        vec!["balatro".to_string()],
        "until the delete lands the corpus answers the key alone, so the window is a no-op \
         for clients (`027-L5`)"
    );

    let deleted = delete_issued(&server, "balatro");
    assert!(
        matches!(status_code(&deleted), Some(200..=299)),
        "then the issued copy is deleted: {}",
        status_line(&deleted)
    );
    assert_eq!(
        json_body(&server.stats(None))["issued"].as_u64(),
        Some(0),
        "`stats.issued` decrements (`027-C6`)"
    );
    assert_eq!(
        common::game_ids(&server.get("/v1/candidates?slug=steam%2F9490001")),
        vec!["balatro".to_string()],
        "and the key is still answered, from the corpus"
    );

    let corpus_delete = server.request("DELETE", &game_path("balatro"), &[], "");
    assert!(
        matches!(status_code(&corpus_delete), Some(200..=299)),
        "the curated record is deleted in turn: {}",
        status_line(&corpus_delete)
    );
    assert_eq!(
        retired_ids(&data),
        vec!["balatro".to_string()],
        "one retired id, from both deletes — the set insert is a no-op the second time"
    );
    assert_eq!(
        mint(&server, 9_490_001),
        "balatro-steam-9490001",
        "and the next query mints a suffix id, never the retired one (Test strategy #10's \
         last clause, #5's last clause)"
    );
}

/// proves: `027-C6`'s "one issued record per call; no bulk retire" (`026-L3`'s
/// posture) — the issued delete route answers only `DELETE` on a `<game_id>`
/// segment, and a method it does not carry is the named `405` that names the ones
/// it does.
#[test]
fn the_issued_route_is_one_record_per_call_and_no_bulk_verb() {
    let data = TempDir::new("curation-no-bulk");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_500_001, "Inscryption")]);
    let server = serve(&data, &corpus);
    let minted = mint(&server, 9_500_001);

    for (method, target) in [
        ("GET", issued_path(&minted)),
        ("POST", "/v1/admin/issued".to_string()),
        ("DELETE", "/v1/admin/issued".to_string()),
    ] {
        let answer = server.request(method, &target, &[], "");
        let code = status_code(&answer);
        assert!(
            matches!(code, Some(400..=499)),
            "`{method} {target}` is not a route this face carries, and bulk retire does not \
             exist: {}",
            status_line(&answer)
        );
        if code == Some(405) {
            assert!(
                header_values(&answer, "Allow")
                    .join(",")
                    .to_ascii_uppercase()
                    .contains("DELETE"),
                "a `405` names the methods the route does carry: {:?}",
                header_values(&answer, "Allow")
            );
        }
    }
    assert_eq!(
        issued_ids(&data),
        vec![minted],
        "none of them mutated anything"
    );
}
