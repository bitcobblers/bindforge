//! The two ends meeting over a loopback socket: the shipped `discovery` CLI driven
//! against a live hosted registry, in the postures ADR-027 leaves the CLI with.
//!
//! proves: design 027's Test strategy #12 — "**Hosted end-to-end**: `serve --root
//! --corpus <seed>` … response shape stays byte-exact to the shipped candidate
//! contract (A8)" and #16 — "**Linux** is exercised directly …; the shipped CLI is
//! what the `025-C4`/`026-C2` face serves verbatim" — read as the *client's* half of
//! that contract: what a real scan against a hosted instance prints, and what it
//! refuses to print. The byte-exact response shape and the health document shape
//! #12 also names are pinned by `response_shape.rs` and `hosted_health_stats.rs`
//! against the same routes, and the `registry query` one-shot (#15) is
//! `hosted_end_to_end.rs`; none of that is restated here. What this suite adds is
//! the other process on the socket.
//!
//! # What ADR-027 retired out of this file
//!
//! Four legs of this file were the caching pair: a root chained behind a caching
//! instance through `--upstream` (cold-fetches-and-caches, warm-makes-zero-requests),
//! an upstream-down caching `503` reaching the CLI, `query` naming the chained root
//! and then the cache, the pair answering one request the same way while differing
//! only in `health`, and the pair growing one slug at a time. ADR-027 retires
//! `--upstream`, registry-to-registry federation (`026-C6`, including gauging through
//! `/v1/health`) and the caching mode with them (A15 names this file among the suites
//! that re-pin), so those legs went. Their surviving *claims* are pinned by name
//! where they now live: the one-lookup-per-key discipline against the store is
//! `027-L4`'s assertion in `steam.rs`; the provenance a hosted `query` prints, and
//! its non-zero exit on every named `503`, are #15 in `hosted_end_to_end.rs`; the
//! "one mode's answer is the other mode's answer" claim has no two modes left to
//! compare, so it retires with them.
//!
//! What replaces the pair's fail-closed leg is the claim §Compatibility matrix makes
//! for hosted mode: the new `503`s "render as the shipped status-line-only refusal
//! **and fail the whole scan closed** (`025-C4`'s fail-closed rule)". That is the
//! same CLI-side promise the chained pair pinned, now sourced from the registry's own
//! cold store supply rather than from a dead upstream — and sourced from the one cold
//! spelling a test can put a live instance into (`--no-steam`), because §The steam
//! snapshot resolver's startup order will not let a hosted run boot on a stale
//! snapshot: it pulls the compile-time-pinned authority instead, which #17 forbids
//! here. Which cold refusal names which condition is #13's legs in
//! `hosted_lookup_policy.rs` and #3's `steam resolver inactive` cell; what this file
//! owns is that a named `503` reaches this client as one line and costs the whole scan.
//!
//! Every server is an ephemeral `127.0.0.1` listener started by this test and killed
//! when it goes out of scope, every one of them is a hosted instance sandboxed to its
//! own `XDG_DATA_HOME` and given a planted snapshot — so its resolver has something to
//! *load* and no reason to reach the store (or runs `--no-steam`, which never reads it)
//! — and every CLI run gets a temp
//! `XDG_DATA_HOME` and `HOME`, so no run reaches a developer's match store and no test
//! dials or resolves a non-loopback host (Test strategy #17). The key each instance
//! is given is a throwaway that no run in this file spends.
//! `discovery_cli_bin()` is the harness's pin that the shipped CLI was built beside
//! the registry, which is what `cargo test --workspace` gives.

mod common;

use common::{
    Registry, SteamRoot, TempDir, closed_loopback_port, corpus_doc, game_ids, now_unix,
    run_discovery, slug_query, status_code, status_line,
};
use std::path::{Path, PathBuf};

const KEY: &str = "bindforge-test-steam-key-2b3c4d5e";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";

/// One `discovery list` scan against a live registry, sandboxed away from any real
/// match store, answering the CLI's confirm prompt with `stdin`.
fn scan(server: &Registry, steam: &SteamRoot, store: &TempDir, stdin: &str) -> common::Run {
    let sandbox = store
        .path()
        .to_str()
        .expect("a utf-8 store sandbox")
        .to_string();
    let url = server.url();
    let root = steam.path().to_str().expect("a utf-8 steam root");
    run_discovery(
        &["list", "--steam-root", root, "--catalog-url", url.as_str()],
        &[
            ("XDG_DATA_HOME", sandbox.as_str()),
            ("HOME", sandbox.as_str()),
        ],
        stdin,
    )
}

fn one_game_root(label: &str, appid: &str, name: &str) -> SteamRoot {
    let root = SteamRoot::new(label);
    root.acf(appid, Some(name), name);
    root
}

fn corpus_in(data: &TempDir, games: &[(&str, &str, &[&str])]) -> std::path::PathBuf {
    data.write("corpus.json", corpus_doc(3, games).as_bytes())
}

/// A snapshot planted at `fetched_at` in `<data>/bindforge/registry/`, in the
/// `027-C3` shape the loader admits: this is #12's "fresh planted snapshot" leg, and
/// the reason no run in this file has anything to pull.
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
    data.write(
        "bindforge/registry/steam-snapshot.json",
        document.as_bytes(),
    );
}

/// A hosted instance (`027-C1`'s one mode) sandboxed to `data`, with the key
/// configured and never used. Extra flags — an explicit `--port` — come from the leg.
fn hosted(data: &TempDir, corpus: &Path, extra: &[&str]) -> Registry {
    let xdg = data
        .path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string();
    let mut args: Vec<String> = vec![
        "serve".to_string(),
        "--root".to_string(),
        "--corpus".to_string(),
        corpus.display().to_string(),
    ];
    args.extend(extra.iter().map(|arg| (*arg).to_string()));
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    Registry::start(
        "cli-e2e",
        &borrowed,
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
    )
}

/// The CLI's match store for a sandbox: `<data>/bindforge/matches.json` (A6's
/// join), the file a confirmed candidate lands in.
fn match_store(data_home: &Path) -> PathBuf {
    data_home.join("bindforge").join("matches.json")
}

/// The `Game Id` cell of the row for `slug` in a rendered `list` table, located
/// through the table's own header rather than a fixed column position.
fn game_id_cell(stdout: &str, slug: &str) -> String {
    let cells = |line: &str| -> Vec<String> {
        line.split('|')
            .map(|part| part.trim().to_string())
            .collect()
    };
    let column = stdout
        .lines()
        .map(&cells)
        .find_map(|row| row.iter().position(|cell| *cell == "Game Id"))
        .expect("the table renders a `Game Id` header");
    for row in stdout.lines().map(cells) {
        if row.iter().any(|cell| cell == slug) {
            return row.get(column).cloned().unwrap_or_default();
        }
    }
    panic!("the table renders a row for `{slug}`: {stdout:?}");
}

/// proves: Test strategy #12's client-facing half — the shipped CLI, pointed at a
/// hosted registry at its printed ephemeral port, resolves a `steam/<appid>` lookup
/// key to the curated `game_id` the corpus holds for it, and renders it. The design
/// promises the bound CLI needs no change (§Compatibility matrix: "No change required
/// or permitted"), so this is the shipped contract *used*, not restated: the CLI is
/// the client A1 describes, and the corpus half of a hosted instance answers it
/// exactly as the old root mode did. The rendering follows the shipped `025` rule the
/// CLI's own suites pin — the scan that earns a confirmation prints `-` (its row was
/// rendered before any join) and the id shows in the `Game Id` column of the next
/// scan — so the resolution is pinned both ways: the candidate the live registry
/// served is stored, and the id is what the next scan prints.
#[test]
fn the_shipped_cli_resolves_a_curated_id_against_a_live_hosted_registry() {
    let data = TempDir::new("e2e-cli-resolves");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/5001"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_400_001, "Unrelated Title")]);
    let server = hosted(&data, &corpus, &[]);
    let steam = one_game_root("e2e-cli-resolves", "5001", "Example Game");
    let store = TempDir::new("e2e-cli-resolves-store");

    let run = scan(&server, &steam, &store, "1\n");
    assert!(
        run.success(),
        "a scan of a reachable hosted registry succeeds; stderr: {}",
        run.stderr
    );
    assert!(
        run.stdout.contains("steam/5001"),
        "the looked-up slug is rendered: {:?}",
        run.stdout
    );
    assert!(
        !run.stderr.contains("catalog unreachable"),
        "nothing about this run is a transport failure: {}",
        run.stderr
    );

    // The store the confirmation writes is the live registry's answer, verbatim:
    // the curated `game_id` the corpus holds for `steam/5001`, joined over the
    // socket rather than served by a fake.
    let path = match_store(store.path());
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "confirming the candidate the hosted registry served writes {}: {err}",
            path.display()
        )
    });
    let document: serde_json::Value = serde_json::from_str(&text).expect("the match store is JSON");
    let records = document["matches"].as_array().expect("a matches array");
    assert!(
        records.iter().any(|record| {
            record["slug"].as_str() == Some("steam/5001")
                && record["game_id"].as_str() == Some("example-game")
        }),
        "the curated `game_id` the corpus serves for `steam/5001` is what the CLI \
         confirmed and stored: {document}"
    );

    // And the id is what the CLI renders for the slug once the join it earned is
    // in the store — the resolution a user sees, over the same live endpoint.
    let second = scan(&server, &steam, &store, "");
    assert!(
        second.success(),
        "the second scan of a reachable hosted registry succeeds; stderr: {}",
        second.stderr
    );
    assert_eq!(
        game_id_cell(&second.stdout, "steam/5001"),
        "example-game",
        "the corpus's curated `game_id` is what the CLI resolved for `steam/5001`: {:?}",
        second.stdout
    );
}

/// proves: §The hosted lookup policy's `200 []` branches end to end — a fresh
/// snapshot that does not know the appid answers the empty candidate list (#3), and
/// the CLI renders that as an unmatched row rather than a failure. An empty answer
/// and a transport failure must never be the same event, and `027-L3`'s whole point is
/// that the hosted mode keeps them apart: only a *fresh* snapshot may certify "never
/// heard of it".
#[test]
fn an_unknown_but_certifiable_key_is_an_empty_answer_to_the_cli_not_a_failure() {
    let data = TempDir::new("e2e-cli-absent");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/5001"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_400_002, "Unrelated Title")]);
    let server = hosted(&data, &corpus, &[]);
    let steam = one_game_root("e2e-cli-absent", "9999", "Unknown Game");
    let store = TempDir::new("e2e-cli-absent-store");

    let answer = server.get(&slug_query("steam/9999"));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "a fresh snapshot certifies the miss, it does not fail it: {}",
        status_line(&answer)
    );
    assert_eq!(
        game_ids(&answer),
        Vec::<String>::new(),
        "and the answer is the empty candidate list"
    );

    let run = scan(&server, &steam, &store, "");
    assert!(
        run.success(),
        "an empty candidate list is not a failed scan; stderr: {}",
        run.stderr
    );
    assert!(
        run.stdout.contains("steam/9999"),
        "the row is still rendered: {:?}",
        run.stdout
    );
    assert!(
        !run.stdout.contains("example-game"),
        "and nothing was resolved for it: {:?}",
        run.stdout
    );
}

/// proves: §Compatibility matrix's bound-CLI row — "the new `503`s render as the
/// shipped status-line-only refusal **and fail the whole scan closed** (`025-C4`'s
/// fail-closed rule)". The registry side of that posture is Test strategy #13; this is
/// the user side, and it is the claim that makes `health.steam_snapshot` the operator's
/// availability dial: the degraded instance costs a whole scan, so the refusal must be
/// loud, one line, and non-zero — and it must never degrade into a table of blanks.
///
/// The named `503` here is the one a hosted instance can be made to answer from a
/// test's own vantage point, `steam resolver inactive` under `--no-steam`: the
/// stale-snapshot spelling of the same branch needs a run planted on an aging
/// snapshot, and §The steam snapshot resolver's startup order ("load the snapshot if
/// present, loadable, **and** fresh, else pull in full") sends that boot to the
/// compile-time-pinned authority, which Test strategy #17 rules out. Which of the
/// cold refusals names which condition is #13's own legs in
/// `hosted_lookup_policy.rs`; what this leg owns is that *any* of them reaches this
/// process as one refusal line and ends the run.
#[test]
fn a_named_503_from_the_store_half_fails_the_cli_scan_closed() {
    let data = TempDir::new("e2e-cli-refusal");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/5001"][..])],
    );
    let server = hosted(&data, &corpus, &["--no-steam"]);
    let steam = one_game_root("e2e-cli-refusal", "9998", "Nowhere Game");
    let store = TempDir::new("e2e-cli-refusal-store");

    let answer = server.get(&slug_query("steam/9998"));
    assert_eq!(
        status_code(&answer),
        Some(503),
        "the store half of this instance refuses by name rather than certifying \
         anything: {}",
        status_line(&answer)
    );
    assert!(
        status_line(&answer)
            .to_ascii_lowercase()
            .contains("inactive"),
        "the refusal is the named one, so the client's one line can say what is wrong: {}",
        status_line(&answer)
    );

    let run = scan(&server, &steam, &store, "");
    assert!(
        !run.success(),
        "the CLI fails the run on the named `503`; stdout {:?}",
        run.stdout
    );
    assert_eq!(
        run.code(),
        Some(1),
        "the refusal exits with `ExitCode::FAILURE`"
    );
    let lines = run.stderr_lines();
    assert_eq!(
        lines.len(),
        1,
        "exactly one named stderr line reports it: {:?}",
        run.stderr
    );
    assert!(
        lines[0].starts_with("error: catalog unreachable:"),
        "the `503` surfaces to the CLI as `error: catalog unreachable: <reason>`, got: {:?}",
        lines[0]
    );
    assert!(
        lines[0].to_ascii_lowercase().contains("inactive"),
        "and the reason survives the hop, because a user reads this line and nothing \
         else: {:?}",
        lines[0]
    );
    assert!(
        run.stdout.is_empty(),
        "a mid-scan failure prints no partial table: {:?}",
        run.stdout
    );
    assert!(
        !match_store(store.path()).exists(),
        "a refused scan confirms nothing into the match store"
    );
}

/// proves: Chosen design: `apps/registry` CLI surface (`--host`/`--port`) at the
/// explicit-bind branch the round-1 testing finding named unlegged — every other
/// suite rides the port-0 default and reads the printed address, so nothing pinned
/// that `--port <N>` answers on `<N>`. An instance started with an explicit
/// `127.0.0.1` port prints that port (the printed value remains the contract) and
/// answers its serving face on exactly it.
#[test]
fn an_explicit_port_bind_answers_on_the_port_it_was_given() {
    let data = TempDir::new("e2e-explicit-port");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/5009"][..])],
    );
    plant_snapshot(&data, now_unix(), &[(9_400_004, "Unrelated Title")]);
    // A port just learned from a bound-and-closed loopback listener: free to bind,
    // and no test dials or resolves a non-loopback host.
    let port = closed_loopback_port();
    let port_text = port.to_string();
    let server = hosted(&data, &corpus, &["--port", &port_text]);

    assert_eq!(
        server.port(),
        port,
        "the printed bound port is the one the operator asked for"
    );
    assert!(
        server.stdout_text().contains(&port_text),
        "the instance prints the address it is serving on — the printed value is the \
         contract: {:?}",
        server.stdout_text()
    );

    let health = server.health();
    assert_eq!(
        status_code(&health),
        Some(200),
        "the serving face answers on the explicit port: {}",
        status_line(&health)
    );
    assert_eq!(
        game_ids(&server.get(&slug_query("steam/5009"))),
        vec!["example-game".to_string()],
        "and so does the join"
    );
}
