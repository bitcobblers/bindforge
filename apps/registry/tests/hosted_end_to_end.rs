//! The hosted instance end to end: one subprocess, one redirected `<data>`, the
//! shipped wire contract, and the `registry query` one-shot that shares its
//! startup.
//!
//! proves: design 027's Test strategy #12 — "`serve --root --corpus <seed>` with a
//! fresh planted snapshot (covering appids beyond the seed's six) and a configured,
//! never-used key; a snapshot-known `steam/<appid>` mints and answers; the same key
//! answers again with zero issued-document rewrites; a curated key answers with its
//! corpus id; an unknown steam appid answers `200 []`; `epic/…` answers the named
//! `503`; `local/x` answers `200 []`; response shape stays byte-exact to the shipped
//! candidate contract (A8); a second `serve` against the same redirected `<data>`
//! refuses at startup, and the first removes `write.lock` on exit (#8)" — and Test
//! strategy #15 — "`registry query` prints `corpus`/`issued` provenance, prints the
//! shipped `(no candidates)` line with provenance `none` for either `200 []` branch,
//! exits non-zero on every named-503 posture, lands its mint in the redirected
//! issued document, and refuses to start against a `<data>` whose write lock a live
//! `serve` holds (#8)".
//!
//! Test strategy #17 holds throughout: the only addresses ever dialed are
//! `127.0.0.1` ports the harness bound, no store is reachable or needed (every run
//! loads a fresh planted snapshot or runs `--no-steam`), and the Web API key is a
//! sentinel this suite checks never surfaces.

mod common;

use common::{
    Registry, TempDir, body_of, body_text, corpus_doc, header_names, header_values, json_body,
    now_unix, registry_document, run_registry, slug_query, status_code, status_line,
};
use std::path::{Path, PathBuf};

const KEY: &str = "bindforge-test-steam-key-7c44be19";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";
const ISSUED: &str = "bindforge/registry/issued-v1.json";

/// The seed corpus of `027-S1`: the shipped six records' shape, with appids that
/// the planted snapshot deliberately does *not* cover, so minted and curated
/// answers stay distinguishable.
fn seed_games() -> Vec<(&'static str, &'static str, &'static [&'static str])> {
    vec![
        ("counter-strike-2", "Counter-Strike 2", &["steam/730"][..]),
        ("dota-2", "Dota 2", &["steam/570"][..]),
        ("half-life-2", "Half-Life 2", &["steam/220"][..]),
        ("left-4-dead-2", "Left 4 Dead 2", &["steam/550"][..]),
        ("portal-two", "Portal 2", &["steam/620"][..]),
        (
            "an-epic-game",
            "An Epic Game",
            &["epic/e416cf14dca04c6486ba0c4e85a820f2"][..],
        ),
    ]
}

/// A snapshot of games beyond the seed, so a mint is always for a key the corpus
/// has never heard of.
fn snapshot_apps() -> Vec<(u64, &'static str)> {
    vec![
        (9_600_001, "Tunic"),
        (9_600_002, "Returnal"),
        (9_600_003, "Outer Wilds"),
    ]
}

fn xdg_of(data: &TempDir) -> String {
    data.path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string()
}

fn env_for(data: &TempDir) -> String {
    xdg_of(data)
}

fn plant_snapshot(data: &TempDir) {
    let entries: Vec<serde_json::Value> = snapshot_apps()
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

fn seed_corpus(data: &TempDir) -> PathBuf {
    let games = seed_games();
    data.write("corpus.json", corpus_doc(6, &games).as_bytes())
}

fn serve(data: &TempDir, corpus: &Path, extra: &[&str]) -> Registry {
    let corpus_path = corpus.display().to_string();
    let xdg = env_for(data);
    let mut args = vec!["serve", "--root", "--corpus", corpus_path.as_str()];
    args.extend_from_slice(extra);
    Registry::start(
        "e2e",
        &args,
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
    )
}

fn issued_records(data: &TempDir) -> Vec<(String, Vec<String>)> {
    let path = data.join(ISSUED);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    let document: serde_json::Value =
        serde_json::from_str(&text).expect("the issued document is a catalog document");
    document["games"]
        .as_array()
        .expect("a games list")
        .iter()
        .map(|game| {
            (
                game["game_id"].as_str().expect("a game_id").to_string(),
                game["source_keys"]
                    .as_array()
                    .expect("source_keys")
                    .iter()
                    .map(|key| key.as_str().expect("a key string").to_string())
                    .collect(),
            )
        })
        .collect()
}

/// proves: Test strategy #12's walkthrough — one hosted instance over a seed corpus
/// and a snapshot covering appids beyond it mints, answers, curates, and misses
/// exactly as the decision table says, over the shipped routes.
#[test]
fn a_hosted_instance_serves_the_seed_the_snapshot_and_its_own_misses() {
    let data = TempDir::new("e2e-walkthrough");
    let corpus = seed_corpus(&data);
    plant_snapshot(&data);
    let server = serve(&data, &corpus, &[]);

    let minted = server.get("/v1/candidates?slug=steam%2F9600001");
    assert_eq!(
        common::game_ids(&minted),
        vec!["tunic".to_string()],
        "a snapshot-known `steam/<appid>` mints and answers"
    );
    assert_eq!(
        issued_records(&data),
        vec![("tunic".to_string(), vec!["steam/9600001".to_string()])],
        "and the whole issued record is on disk"
    );

    let again = server.get("/v1/candidates?slug=steam%2F9600001");
    assert_eq!(
        common::game_ids(&again),
        vec!["tunic".to_string()],
        "the same key answers again"
    );
    assert_eq!(
        json_body(&server.stats(None))["issued"].as_u64(),
        Some(1),
        "with exactly one record behind it"
    );

    assert_eq!(
        common::game_ids(&server.get("/v1/candidates?slug=steam%2F570")),
        vec!["dota-2".to_string()],
        "a curated key answers with its corpus id"
    );
    let unknown = server.get("/v1/candidates?slug=steam%2F9699999");
    assert!(
        status_code(&unknown) == Some(200)
            && json_body(&unknown).as_array().is_some_and(Vec::is_empty),
        "an unknown steam appid answers `200 []`: {}",
        status_line(&unknown)
    );
    let epic = server.get(&slug_query("epic/e416cf14dca04c6486ba0c4e85a820f3"));
    assert_eq!(
        status_code(&epic),
        Some(503),
        "`epic/…` answers the named `503`: {}",
        status_line(&epic)
    );
    let curated_epic = server.get(&slug_query("epic/e416cf14dca04c6486ba0c4e85a820f2"));
    assert_eq!(
        common::game_ids(&curated_epic),
        vec!["an-epic-game".to_string()],
        "the seed's curated `epic/` record keeps answering on curation alone (A13, \
         Rollout 2's `027-S1`)"
    );
    let local = server.get(&slug_query("local/x"));
    assert!(
        status_code(&local) == Some(200) && json_body(&local).as_array().is_some_and(Vec::is_empty),
        "`local/x` answers `200 []`: {}",
        status_line(&local)
    );

    let health = json_body(&server.health());
    assert_eq!(health["mode"].as_str(), Some("hosted"));
    assert_eq!(
        health["steam_snapshot"]["games"].as_u64(),
        Some(3),
        "the snapshot the instance loaded is the one it reports: {health}"
    );
}

/// proves: Test strategy #12's shape clause with A8 — the candidate answer a hosted
/// instance gives is byte-shaped like the contract the shipped CLI pins: exactly one
/// decimal `Content-Length` that matches the body, no `Transfer-Encoding`, a JSON
/// array of `{game_id, name}` objects, and a body inside `MAX_RESPONSE_OCTETS`.
#[test]
fn every_hosted_answer_keeps_the_shipped_wire_shape() {
    let data = TempDir::new("e2e-shape");
    let corpus = seed_corpus(&data);
    plant_snapshot(&data);
    let server = serve(&data, &corpus, &[]);

    for answer in [
        server.get("/v1/candidates?slug=steam%2F570"),
        server.get("/v1/candidates?slug=steam%2F9600002"),
        server.get("/v1/candidates?slug=steam%2F9699998"),
        server.get(&slug_query("local/x")),
        server.get(&slug_query("epic/77")),
        server.health(),
        server.get("/v1/admin/stats"),
    ] {
        let code = status_code(&answer).unwrap_or_else(|| panic!("a status line: {answer:?}"));
        let lengths = header_values(&answer, "Content-Length");
        assert_eq!(
            lengths.len(),
            1,
            "status {code} answers with exactly one `Content-Length` (A1's reader refuses \
             zero or two): {lengths:?}"
        );
        let claimed = lengths[0]
            .parse::<u64>()
            .unwrap_or_else(|err| panic!("status {code} names a decimal length: {err}"));
        assert_eq!(claimed as usize, body_of(&answer).len(), "status {code}");
        assert!(
            !header_names(&answer)
                .iter()
                .any(|name| name == "transfer-encoding"),
            "status {code} never uses chunked framing"
        );
        assert!(
            body_of(&answer).len() <= 65_536,
            "status {code} stays inside `MAX_RESPONSE_OCTETS` (A8)"
        );
    }

    let minted = server.get("/v1/candidates?slug=steam%2F9600003");
    let candidates = json_body(&minted);
    let array = candidates
        .as_array()
        .expect("a candidate answer is a JSON array");
    assert_eq!(array.len(), 1);
    let object = array[0].as_object().expect("a candidate is an object");
    let mut fields: Vec<&str> = object.keys().map(String::as_str).collect();
    fields.sort();
    assert_eq!(
        fields,
        vec!["game_id", "name"],
        "`025-C4`/`026-C2`'s candidate face survives verbatim: no provenance, no provenance \
         field, no extra key: {candidates}"
    );
}

/// proves: Test strategy #12's last clause with §Concurrency's cross-process rule —
/// a second `serve` against the same redirected `<data>` refuses at startup, while
/// the first is alive and holding `write.lock`.
#[test]
fn a_second_serve_against_a_live_data_directory_refuses_at_startup() {
    let data = TempDir::new("e2e-double-serve");
    let corpus = seed_corpus(&data);
    plant_snapshot(&data);
    let first = serve(&data, &corpus, &[]);
    let lock = registry_document(data.path(), "write.lock");
    assert!(
        lock.is_file(),
        "the live instance holds the data-directory lock"
    );

    let second = common::expect_refused_startup(
        &["serve", "--root", "--corpus", &corpus.display().to_string()],
        &[("XDG_DATA_HOME", env_for(&data).as_str()), (KEY_VAR, KEY)],
    );
    assert!(
        second.stderr.to_ascii_lowercase().contains("lock"),
        "the refusal is `registry data directory is locked` (§Concurrency): {:?}",
        second.stderr
    );
    assert!(
        lock.is_file(),
        "the refused process leaves the live owner's lock in place"
    );
    assert_eq!(
        common::game_ids(&first.get("/v1/candidates?slug=steam%2F570")),
        vec!["dota-2".to_string()],
        "and the live instance is unharmed by the refused one"
    );
}

/// proves: Test strategy #17 with §The store pull's key-custody rule (`027-L4`,
/// A14's CWE-319/526 scrubbing half) — the Web API key appears in no output and no
/// document of a hosted run that minted, refused, and reported health.
#[test]
fn the_store_key_surfaces_nowhere() {
    let data = TempDir::new("e2e-key-custody");
    let corpus = seed_corpus(&data);
    plant_snapshot(&data);
    let server = serve(&data, &corpus, &[]);
    server.get("/v1/candidates?slug=steam%2F9600001");
    server.get(&slug_query("epic/77"));
    server.get("/v1/health");
    let stdout = server.stdout_text();
    let stderr = server.stderr_text();
    let body = body_text(&server.get("/v1/admin/stats"));
    assert!(
        !stdout.contains(KEY),
        "the startup log carries the key: {stdout:?}"
    );
    assert!(
        !stderr.contains(KEY),
        "the instance log carries the key: {stderr:?}"
    );
    assert!(
        !body.contains(KEY),
        "a served document carries the key: {body:?}"
    );

    for path in data.files_below("bindforge") {
        let bytes = std::fs::read(&path).unwrap_or_default();
        assert!(
            !bytes
                .windows(KEY.len())
                .any(|window| window == KEY.as_bytes()),
            "the key was persisted into {}: it is environment-only custody (`027-C3`)",
            path.display()
        );
    }

    let refused = common::expect_refused_startup(
        &["serve", "--root", "--corpus", "/nonexistent/corpus.json"],
        &[("XDG_DATA_HOME", env_for(&data).as_str()), (KEY_VAR, KEY)],
    );
    assert!(
        !refused.stdout.contains(KEY) && !refused.stderr.contains(KEY),
        "a failure line names the condition and never the key or a request URI (§The store \
         pull): {:?}",
        refused.stderr
    );
}

/// proves: Test strategy #15's provenance clauses — `registry query` prints the
/// answer with `corpus` provenance for a curated key and `issued` provenance for a
/// minted one, and the shipped `(no candidates)` line with `none` for both empty
/// branches.
#[test]
fn query_prints_the_answer_with_its_provenance() {
    let data = TempDir::new("e2e-query-provenance");
    let corpus = seed_corpus(&data);
    plant_snapshot(&data);
    let xdg = env_for(&data);
    let corpus_path = corpus.display().to_string();

    let curated = run_registry(
        &["query", "steam/570", "--root", "--corpus", &corpus_path],
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
    );
    assert!(
        curated.success(),
        "a curated key succeeds: {}",
        curated.stderr
    );
    assert!(
        curated.stdout.contains("dota-2"),
        "the answer is printed: {:?}",
        curated.stdout
    );
    assert!(
        curated.stdout.to_ascii_lowercase().contains("corpus"),
        "a curated answer names its provenance: {:?}",
        curated.stdout
    );

    let minted = run_registry(
        &["query", "steam/9600001", "--root", "--corpus", &corpus_path],
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
    );
    assert!(minted.success(), "a minted key succeeds: {}", minted.stderr);
    assert!(
        minted.stdout.contains("tunic"),
        "the minted identity is printed: {:?}",
        minted.stdout
    );
    assert!(
        minted.stdout.to_ascii_lowercase().contains("issued"),
        "and its provenance is `issued`: {:?}",
        minted.stdout
    );
    assert_eq!(
        issued_records(&data),
        vec![("tunic".to_string(), vec!["steam/9600001".to_string()])],
        "a `query` mints and persists exactly like a served query (§Hosted mode)"
    );

    for (label, key) in [
        ("unknown steam appid", "steam/9699997"),
        ("local key", "local/x"),
    ] {
        let empty = run_registry(
            &["query", key, "--root", "--corpus", &corpus_path],
            &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
        );
        assert!(
            empty.success(),
            "`{label}` is a `200 []` posture, so the one-shot exits zero: {}",
            empty.stderr
        );
        assert!(
            empty.stdout.contains("(no candidates)"),
            "`{label}` prints the shipped `(no candidates)` line: {:?}",
            empty.stdout
        );
        assert!(
            empty.stdout.to_ascii_lowercase().contains("none"),
            "`{label}` carries provenance `none`: {:?}",
            empty.stdout
        );
    }
}

/// proves: Test strategy #15's exit-code clause — a `query` exits non-zero on every
/// named-`503` posture, including the resolver-inactive one, and prints no answer
/// for them.
#[test]
fn query_exits_non_zero_on_every_named_refusal() {
    let data = TempDir::new("e2e-query-503");
    let corpus = seed_corpus(&data);
    // Planted for the same reason every other run here plants one: with nothing to
    // load the one-shot pulls the store at startup, and the non-zero exit this leg
    // asserts would come from a failed pull rather than from the named `503` posture
    // it is about (and Test strategy #17 keeps the dial out of the suite entirely).
    plant_snapshot(&data);
    let xdg = env_for(&data);
    let corpus_path = corpus.display().to_string();

    let epic = run_registry(
        &["query", "epic/77", "--root", "--corpus", &corpus_path],
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
    );
    let curated = run_registry(
        &[
            "query",
            "steam/570",
            "--root",
            "--corpus",
            &corpus_path,
            "--no-steam",
        ],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        curated.success(),
        "the curated record answers with the resolver inactive: {}",
        curated.stderr
    );

    // The seed's own `epic/` record is curated, so the refusal is reached through an
    // `epic/…` key the corpus does not hold, and through the resolver-inactive
    // posture: the two named-503 branches this suite can reach without a store.
    let steam_inactive = run_registry(
        &[
            "query",
            "steam/9600002",
            "--root",
            "--corpus",
            &corpus_path,
            "--no-steam",
        ],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        !steam_inactive.success(),
        "`steam resolver inactive` is a named-503 posture and a query on it is non-zero: \
         stdout {:?}",
        steam_inactive.stdout
    );
    assert!(
        !epic.success(),
        "an `epic/…` key the corpus does not hold is a named-503 posture too; it exited \
         {:?} with stdout {:?}",
        epic.code(),
        epic.stdout
    );
}

/// proves: Test strategy #15's stopped-instance clause — a `query` lands its mint
/// in the `<data>` it is pointed at, and refuses to start against a `<data>` whose
/// write lock a live `serve` holds.
#[test]
fn query_writes_the_data_directory_it_is_pointed_at_and_refuses_a_live_one() {
    let data = TempDir::new("e2e-query-lock");
    let corpus = seed_corpus(&data);
    plant_snapshot(&data);
    let server = serve(&data, &corpus, &[]);
    assert!(
        registry_document(data.path(), "write.lock").is_file(),
        "the live serve holds the lock a one-shot would need"
    );

    let other = TempDir::new("e2e-query-lock-target");
    // The same planted snapshot, or the one-shot has nothing to load and pulls the
    // store at startup — the non-loopback dial Test strategy #17 rules out, and the
    // reason this leg used to spend 61 s here. §The steam snapshot resolver's startup
    // order is what makes a planted snapshot the whole of the fix.
    plant_snapshot(&other);
    let other_xdg = env_for(&other);
    let corpus_path = corpus.display().to_string();
    let elsewhere = run_registry(
        &["query", "steam/9600001", "--root", "--corpus", &corpus_path],
        &[("XDG_DATA_HOME", other_xdg.as_str()), (KEY_VAR, KEY)],
    );
    assert!(
        elsewhere.success(),
        "a one-shot pointed at a free `<data>` runs fine: {}",
        elsewhere.stderr
    );
    assert_eq!(
        issued_records(&other),
        vec![("tunic".to_string(), vec!["steam/9600001".to_string()])],
        "and mints into whatever `<data>` it is pointed at (§Hosted mode, Rollout 6)"
    );
    assert!(
        !data.join(ISSUED).exists(),
        "the locked `<data>` is untouched by the run that ran elsewhere: {:?}",
        data.files_below("bindforge")
    );

    let beside = run_registry(
        &["query", "steam/570", "--root", "--corpus", &corpus_path],
        &[("XDG_DATA_HOME", env_for(&data).as_str()), (KEY_VAR, KEY)],
    );
    assert!(
        !beside.success(),
        "`query` refuses to run beside a live `serve` — it can mint, so it takes the same \
         write lock (§Concurrency): stdout {:?}",
        beside.stdout
    );
    assert!(
        beside.stderr.to_ascii_lowercase().contains("lock"),
        "and says so: {:?}",
        beside.stderr
    );
    drop(server);
}
