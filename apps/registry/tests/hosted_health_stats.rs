//! The two summary faces of a hosted instance: `GET /v1/health`, re-pinned to
//! `{status, mode: "hosted", corpus_entries, steam_snapshot}`, and
//! `GET /v1/admin/stats`, grown to `{games, source_keys, revision, issued}`.
//!
//! proves: design 027's Test strategy #9 — "health is exactly `{status, mode:
//! "hosted", corpus_entries, steam_snapshot}` with the `{games, fetched_at}` and
//! `"none"` spellings and none of the three retired facts; `stats` is `{games,
//! source_keys, revision, issued}`" — with §Hosted mode's health re-pin (the
//! `027-C3` shape "verbatim", the retired `upstream: "none"` precedent for the
//! `"none"` spelling, and "the issued count is deliberately **not** public health")
//! and §The curation face's "counts only; no key or name lists".

mod common;

use common::{Registry, TempDir, corpus_doc, json_body, now_unix, registry_document, status_code};
use std::path::PathBuf;

const KEY: &str = "bindforge-test-steam-key-0f1e2d3c";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";

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

/// The `027-C3` snapshot document shape: `{version, fetched_at, apps: [{appid,
/// name}]}` in ascending `appid` order.
fn plant_snapshot(data: &TempDir, fetched_at: u64, apps: &[(u64, &str)]) {
    let mut ordered: Vec<(u64, &str)> = apps.to_vec();
    ordered.sort_by_key(|(appid, _)| *appid);
    assert_eq!(
        ordered,
        apps.to_vec(),
        "a fixture that is not in ascending `appid` order is a broken fixture"
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
    data.write(
        "bindforge/registry/steam-snapshot.json",
        document.as_bytes(),
    );
}

fn corpus_in(data: &TempDir, games: &[(&str, &str, &[&str])]) -> PathBuf {
    data.write("corpus.json", corpus_doc(3, games).as_bytes())
}

/// One hosted instance with a fresh snapshot and a configured, never-used key.
fn hosted(
    label: &str,
    data: &TempDir,
    games: &[(&str, &str, &[&str])],
    extra: &[&str],
) -> (Registry, PathBuf) {
    let corpus = corpus_in(data, games);
    plant_snapshot(data, now_unix(), &[(9_100_001, "Fresh Snapshot Game")]);
    let server = Registry::start(
        label,
        &hosted_args(&corpus.display().to_string(), extra),
        &[("XDG_DATA_HOME", xdg_of(data).as_str()), (KEY_VAR, KEY)],
    );
    (server, corpus)
}

fn health_keys(server: &Registry) -> Vec<String> {
    let document = json_body(&server.health());
    let object = document
        .as_object()
        .expect("the health document is a JSON object");
    let mut names: Vec<String> = object.keys().cloned().collect();
    names.sort();
    names
}

/// proves: Test strategy #9's first clause — health is *exactly* the four-field
/// hosted document, `mode` is `hosted`, and `corpus_entries` counts the curated
/// records.
#[test]
fn health_is_exactly_the_four_field_hosted_document() {
    let data = TempDir::new("health-shape");
    let (server, _corpus) = hosted(
        "health-shape",
        &data,
        &[
            ("example-game", "Example Game", &["steam/900001"][..]),
            (
                "other-game",
                "Other Game",
                &["epic/77", "local/Some.Dir"][..],
            ),
        ],
        &[],
    );

    let answer = server.health();
    assert_eq!(status_code(&answer), Some(200), "health always answers 200");
    assert_eq!(
        health_keys(&server),
        vec![
            "corpus_entries".to_string(),
            "mode".to_string(),
            "status".to_string(),
            "steam_snapshot".to_string(),
        ],
        "`027-C1` re-pins health to the enumerated list {{status, mode, corpus_entries, \
         steam_snapshot}} and nothing else"
    );
    let document = json_body(&answer);
    // `027-C1` re-pins health's *field list*, not its vocabulary: the shipped health
    // document's own `status` value (A1) carries into hosted mode untouched, so the
    // pin is "a non-empty string", and inventing one here would test a word the
    // design never names.
    assert!(
        document["status"]
            .as_str()
            .is_some_and(|status| !status.is_empty()),
        "`status` is the shipped health document's own string: {document}"
    );
    assert_eq!(
        document["mode"].as_str(),
        Some("hosted"),
        "`Mode::Hosted::as_str()` is `\"hosted\"` (§Hosted mode)"
    );
    assert_eq!(
        document["corpus_entries"].as_u64(),
        Some(2),
        "the curated records are counted (`027-C6`'s counts-only posture)"
    );
}

/// proves: Test strategy #9's `steam_snapshot` clause — the `027-C3` shape
/// verbatim: `{"games": <count>, "fetched_at": <unix>}`.
#[test]
fn health_carries_the_snapshot_fact_in_the_pinned_shape() {
    let data = TempDir::new("health-snapshot");
    let fetched_at = now_unix();
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(
        &data,
        fetched_at,
        &[
            (9_100_001, "One Game"),
            (9_100_002, "Two Game"),
            (9_100_003, "Three Game"),
        ],
    );
    let server = Registry::start(
        "health-snapshot",
        &hosted_args(&corpus.display().to_string(), &[]),
        &[("XDG_DATA_HOME", xdg_of(&data).as_str()), (KEY_VAR, KEY)],
    );

    let snapshot = json_body(&server.health())["steam_snapshot"].clone();
    let object = snapshot.as_object().unwrap_or_else(|| {
        panic!("`steam_snapshot` is the {{games, fetched_at}} object: {snapshot}")
    });
    let mut names: Vec<String> = object.keys().cloned().collect();
    names.sort();
    assert_eq!(
        names,
        vec!["fetched_at".to_string(), "games".to_string()],
        "`027-C3` fixes the health snapshot shape verbatim: {snapshot}"
    );
    assert_eq!(
        snapshot["games"].as_u64(),
        Some(3),
        "`games` is the snapshot's record count: {snapshot}"
    );
    assert_eq!(
        snapshot["fetched_at"].as_u64(),
        Some(fetched_at),
        "`fetched_at` is the snapshot's own stamp, read from the loaded document: {snapshot}"
    );
}

/// proves: Test strategy #9's `"none"` clause with §Hosted mode's retired
/// `upstream: "none"` precedent — the resolver-inactive spelling is the string
/// `"none"`, and the retired `last_upstream_fetch` successor is `steam_snapshot`.
#[test]
fn health_spells_an_inactive_resolver_none() {
    let data = TempDir::new("health-none");
    let (server, _corpus) = hosted(
        "health-none",
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
        &["--no-steam"],
    );

    let document = json_body(&server.health());
    assert_eq!(
        document["steam_snapshot"],
        serde_json::json!("none"),
        "the resolver-inactive / no-snapshot spelling is the string `\"none\"` \
         (§Hosted mode): {document}"
    );
    assert_eq!(document["mode"].as_str(), Some("hosted"));
}

/// proves: Test strategy #9's "none of the three retired facts" clause with
/// §Hosted mode's retirement list — `upstream`, `cached_slugs` and
/// `last_upstream_fetch` are gone (and the old `local_entries` count went with
/// them: the enumerated list is exactly four fields).
#[test]
fn health_publishes_none_of_the_retired_caching_facts() {
    let data = TempDir::new("health-retired");
    let (server, _corpus) = hosted(
        "health-retired",
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
        &[],
    );

    let document = json_body(&server.health());
    let object = document.as_object().expect("a JSON object");
    for retired in [
        "upstream",
        "cached_slugs",
        "last_upstream_fetch",
        "local_entries",
        "issued",
    ] {
        assert!(
            !object.contains_key(retired),
            "`{retired}` retires with the caching mode (`027-C1`); health published it: \
             {document}"
        );
    }
    let text = common::body_text(&server.health()).to_ascii_lowercase();
    assert!(
        !text.contains("caching") && !text.contains("upstream"),
        "no retired spelling survives anywhere in the health body: {text:?}"
    );
    assert!(
        !text.contains(KEY),
        "the Web API key never appears in a public document (`027-L4`): {text:?}"
    );
}

/// proves: Test strategy #9's second clause with `027-C6` — `stats` is exactly
/// `{games, source_keys, revision, issued}`, the first three from the corpus as
/// shipped and `issued` the issued-document record count.
#[test]
fn stats_is_the_four_field_shape_with_the_issued_count() {
    let data = TempDir::new("stats-shape");
    let (server, _corpus) = hosted(
        "stats-shape",
        &data,
        &[
            (
                "example-game",
                "Example Game",
                &["steam/900001", "epic/77"][..],
            ),
            ("other-game", "Other Game", &["local/Some.Dir"][..]),
        ],
        &[],
    );

    let answer = server.stats(None);
    assert_eq!(
        status_code(&answer),
        Some(200),
        "loopback stats answers: {}",
        common::status_line(&answer)
    );
    let counts = json_body(&answer);
    let object = counts.as_object().expect("a JSON object");
    let mut names: Vec<String> = object.keys().cloned().collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "games".to_string(),
            "issued".to_string(),
            "revision".to_string(),
            "source_keys".to_string(),
        ],
        "`027-C6` grows the shipped three-field shape by `issued` and nothing else: {counts}"
    );
    for field in ["games", "source_keys", "revision", "issued"] {
        assert!(
            counts[field].as_u64().is_some(),
            "`{field}` is an integer count (`026-L3` allows summary counts only): {counts}"
        );
    }
    assert_eq!(counts["games"].as_u64(), Some(2));
    assert_eq!(counts["source_keys"].as_u64(), Some(3));
    assert_eq!(counts["revision"].as_u64(), Some(3));
    assert_eq!(
        counts["issued"].as_u64(),
        Some(0),
        "a freshly started instance has issued nothing"
    );

    let minted = server.get("/v1/candidates?slug=steam%2F9100001");
    assert_eq!(
        status_code(&minted),
        Some(200),
        "the snapshot-known key answers: {}",
        common::status_line(&minted)
    );
    let after = json_body(&server.stats(None));
    assert_eq!(
        after["issued"].as_u64(),
        Some(1),
        "`stats.issued` is the issued-document record count, and `027-B3`'s approach is \
         visible there (§The curation face): {after}"
    );
    assert_eq!(
        after["games"].as_u64(),
        Some(2),
        "issued records are not curated entries: {after}"
    );
}

/// proves: Test strategy #9 with `026-L3`'s and `027-C6`'s "counts only; no key or
/// name lists" — nothing in `stats` carries a game name, a `game_id`, or a source
/// key, and the issued count is not public health.
#[test]
fn stats_publishes_counts_only_and_never_a_key_or_name_list() {
    let data = TempDir::new("stats-counts-only");
    let (server, _corpus) = hosted(
        "stats-counts-only",
        &data,
        &[(
            "curated-understudy",
            "Curated Understudy",
            &["steam/900009"][..],
        )],
        &[],
    );
    let minted = server.get("/v1/candidates?slug=steam%2F9100001");
    let minted_id = common::game_ids(&minted)
        .first()
        .cloned()
        .expect("the snapshot-known key mints one candidate");

    let body = common::body_text(&server.stats(None));
    for secret in [
        "Fresh Snapshot Game".to_string(),
        minted_id,
        "curated-understudy".to_string(),
        "Curated Understudy".to_string(),
        "steam/9100001".to_string(),
    ] {
        assert!(
            !body.contains(secret.as_str()),
            "`stats` answers counts only — `{secret}` leaked into {body}"
        );
    }

    let health = common::body_text(&server.health());
    assert!(
        !health.contains("issued"),
        "the issued count is deliberately not public health (§Hosted mode): {health:?}"
    );
    assert!(
        registry_document(data.path(), "issued-v1.json").is_file(),
        "the mint landed in the issued document the count reports"
    );
}
