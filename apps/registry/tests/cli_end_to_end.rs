//! The two ends meeting over a loopback socket: the shipped `discovery` CLI driven
//! against a live `registry serve`, and a root registry chained behind a caching one
//! through `--upstream` — the pair the design says must work together before either
//! half is interesting on its own.
//!
//! proves: design 026's Test strategy #16 — "Root end-to-end: `registry serve --root
//! --corpus <seed>` bound to an ephemeral `127.0.0.1` port; `discovery list
//! --catalog-url http://127.0.0.1:<port>` resolves a `steam/<appid>` to a curated id,
//! and an absent slug answers `200` with `[]`; response shape pinned byte-exact to A1
//! (single decimal `Content-Length`, no `Transfer-Encoding`, caps); `GET /v1/health`
//! shape" — and #17 — "Caching end-to-end: a root and a caching registry chained via
//! `--upstream` (both on `127.0.0.1` ephemeral ports); a cold slug fetches-then-caches;
//! a warm slug makes zero upstream requests; an upstream-down cold slug surfaces the
//! named `503` to the CLI as `catalog unreachable` and fails closed; `query` prints
//! provenance."
//!
//! The byte-exact response shape and the health document shape that #16 also names
//! are pinned by `response_shape.rs` and `health_route.rs` against the same routes, so
//! they are not restated here; what this suite adds is the other process on the
//! socket. Every server is an ephemeral `127.0.0.1` listener started by this test and
//! killed when it goes out of scope, and every CLI run gets a temp `XDG_DATA_HOME` and
//! `HOME`, so no run reaches a developer's match store and no test dials or resolves a
//! non-loopback host (A15). `discovery_cli_bin()` is the harness's pin that the shipped
//! CLI was built beside the registry, which is what `cargo test --workspace` gives.

mod common;

use common::{
    Registry, SteamRoot, TempDir, closed_loopback_port, corpus_doc, game_ids, json_body, now_unix,
    registry_document, run_discovery, run_registry, slug_query, status_code, status_line,
};
use std::path::{Path, PathBuf};

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

fn persisted(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{} is JSON ({err})", path.display()))
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

/// The CLI's match store for a sandbox: `<data>/bindforge/matches.json` (A6's
/// join), the file a confirmed candidate lands in.
fn match_store(data_home: &Path) -> PathBuf {
    data_home.join("bindforge").join("matches.json")
}

/// proves: Test strategy #16's first clause — the shipped CLI, pointed at a root
/// registry at its printed ephemeral port, resolves a `steam/<appid>` lookup key to
/// the curated `game_id` the corpus holds for it, and renders it. This is the
/// `026-C2` contract used, not a restatement of it: the CLI is the client A1 describes.
/// The rendering follows the shipped 025 rule the CLI's own suites pin — the scan
/// that earns a confirmation prints `-` (its row was rendered before any join) and
/// the id shows in the `Game Id` column of the next scan — so the resolution is
/// pinned both ways: the candidate the live registry served is stored, and the id is
/// what the next scan prints.
#[test]
fn the_shipped_cli_resolves_a_curated_id_against_a_live_root_registry() {
    let data = TempDir::new("e2e-cli-resolves");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/5001"][..])],
    );
    let server = Registry::root("e2e-cli-resolves", &corpus, &[]);
    let steam = one_game_root("e2e-cli-resolves", "5001", "Example Game");
    let store = TempDir::new("e2e-cli-resolves-store");

    let run = scan(&server, &steam, &store, "1\n");
    assert!(
        run.success(),
        "a scan of a reachable root registry succeeds; stderr: {}",
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
    let document: serde_json::Value = {
        let path = match_store(store.path());
        let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!(
                "confirming the candidate the root registry served writes {}: {err}",
                path.display()
            )
        });
        serde_json::from_str(&text).expect("the match store is JSON")
    };
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
        "the second scan of a reachable root registry succeeds; stderr: {}",
        second.stderr
    );
    assert_eq!(
        game_id_cell(&second.stdout, "steam/5001"),
        "example-game",
        "the corpus's curated `game_id` is what the CLI resolved for `steam/5001`: {:?}",
        second.stdout
    );
}

/// proves: Test strategy #16's second clause — a slug the corpus does not claim is a
/// `200` with an empty candidate list, and the CLI renders that as an unmatched row
/// rather than a failure. An empty answer and a transport failure must never be the
/// same event for a root registry.
#[test]
fn an_absent_slug_is_an_empty_answer_to_the_cli_not_a_failure() {
    let data = TempDir::new("e2e-cli-absent");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/5001"][..])],
    );
    let server = Registry::root("e2e-cli-absent", &corpus, &[]);
    let steam = one_game_root("e2e-cli-absent", "9999", "Unknown Game");
    let store = TempDir::new("e2e-cli-absent-store");

    let answer = server.get(&slug_query("steam/9999"));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "a root registry answers an unknown key, it does not fail it: {}",
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

/// proves: Test strategy #17's first two clauses — a root and a caching registry
/// chained through `--upstream` on two ephemeral loopback ports: a cold slug goes to
/// the root, answers, and is cached with `fetched_at` at the pinned path; the same
/// slug warm makes no second upstream request (`026-C2`'s `last_upstream_fetch` is
/// the counter) and answers the same candidates.
#[test]
fn a_cold_slug_fetches_from_the_chained_root_and_a_warm_slug_does_not() {
    let root_data = TempDir::new("e2e-chain-root");
    let root_corpus = corpus_in(
        &root_data,
        &[("chained-game", "Chained Game", &["steam/7001"][..])],
    );
    let root = Registry::root("e2e-chain-root", &root_corpus, &[]);

    let cache_data = TempDir::new("e2e-chain-cache");
    let caching = Registry::caching("e2e-chain-cache", &root.url(), &cache_data, &[]);

    let cold = caching.get(&slug_query("steam/7001"));
    assert_eq!(
        status_code(&cold),
        Some(200),
        "the chained root answers the cold slug: {}",
        status_line(&cold)
    );
    assert_eq!(game_ids(&cold), vec!["chained-game".to_string()]);

    let after_cold = json_body(&caching.health());
    let fetched = after_cold["last_upstream_fetch"]
        .as_u64()
        .expect("a fetch stamp");
    assert!(
        fetched > 0 && fetched <= now_unix(),
        "the cold lookup records its upstream fetch: {after_cold:?}"
    );
    assert_eq!(
        after_cold["cached_slugs"].as_u64(),
        Some(1),
        "and health counts the cached slug: {after_cold:?}"
    );
    let cache = registry_document(cache_data.path(), "cache-v1.json");
    assert!(
        cache.is_file(),
        "the fetched answer is cached at the pinned path (`026-C7`)"
    );
    let entries = persisted(&cache)["entries"]
        .as_array()
        .expect("an entries array")
        .clone();
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0]["slug"].as_str(), Some("steam/7001"));
    assert!(
        entries[0]["fetched_at"].as_u64().is_some(),
        "with `fetched_at`: {}",
        entries[0]
    );

    let warm = caching.get(&slug_query("steam/7001"));
    assert_eq!(status_code(&warm), Some(200));
    assert_eq!(
        game_ids(&warm),
        game_ids(&cold),
        "the warm answer is the cached one, unchanged"
    );
    assert_eq!(
        json_body(&caching.health())["last_upstream_fetch"]
            .as_u64()
            .expect("a fetch stamp"),
        fetched,
        "a warm slug makes zero upstream requests: the stamp did not move"
    );
}

/// proves: Test strategy #17's third clause — an upstream-down cold slug is the named
/// `503` out of the caching registry, and the shipped CLI renders that as the named
/// `catalog unreachable` refusal, exit non-zero, with nothing on stdout: it fails
/// closed rather than printing a table of blanks.
#[test]
fn an_upstream_down_cold_slug_reaches_the_cli_as_the_named_refusal() {
    let cache_data = TempDir::new("e2e-down-cache");
    let down = format!("http://127.0.0.1:{}", closed_loopback_port());
    let caching = Registry::caching("e2e-down-cache", &down, &cache_data, &[]);
    let steam = one_game_root("e2e-down-cache", "8888", "Nowhere Game");
    let store = TempDir::new("e2e-down-store");

    let answer = caching.get(&slug_query("steam/8888"));
    assert_eq!(
        status_code(&answer),
        Some(503),
        "`026-L4`'s cold case is the named `503`, never `[]`: {}",
        status_line(&answer)
    );

    let run = scan(&caching, &steam, &store, "");
    assert!(
        !run.success(),
        "the CLI fails the run on a cold upstream-down slug; stdout {:?}",
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
        run.stdout.is_empty(),
        "a mid-scan failure prints no partial table: {:?}",
        run.stdout
    );
}

/// proves: Test strategy #17's last clause, in the chained setting — a one-shot
/// `registry query` for a slug the local documents do not know goes to the chained
/// root and says so, and the same command once the answer is cached says where it came
/// from differently, which is what "prints the answer with its provenance" means.
#[test]
fn query_names_the_chained_root_the_first_time_and_the_cache_the_second() {
    let root_data = TempDir::new("e2e-query-root");
    let root_corpus = corpus_in(
        &root_data,
        &[("chained-game", "Chained Game", &["steam/7002"][..])],
    );
    let root = Registry::root("e2e-query-root", &root_corpus, &[]);

    let cache_data = TempDir::new("e2e-query-cache");
    let xdg = cache_data
        .path()
        .to_str()
        .expect("a utf-8 data directory")
        .to_string();

    let first = run_registry(
        &["query", "steam/7002", "--upstream", &root.url()],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        first.success(),
        "the chained root answers the query; stderr: {}",
        first.stderr
    );
    assert!(
        first.stdout.contains("chained-game"),
        "the answer is printed: {:?}",
        first.stdout
    );
    assert!(
        first.stdout.to_ascii_lowercase().contains("upstream"),
        "the first answer names where it came from: {:?}",
        first.stdout
    );

    let second = run_registry(
        &["query", "steam/7002", "--upstream", &root.url()],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        second.success(),
        "the warm query answers too; stderr: {}",
        second.stderr
    );
    assert!(
        second.stdout.contains("chained-game"),
        "the same answer: {:?}",
        second.stdout
    );
    assert!(
        second.stdout.to_ascii_lowercase().contains("cach"),
        "the warm answer is labelled as coming off the cache: {:?}",
        second.stdout
    );
    assert_ne!(
        first.stdout, second.stdout,
        "the two runs are distinguishable by provenance"
    );
    assert!(
        registry_document(cache_data.path(), "cache-v1.json").is_file(),
        "and the query wrote the cache, because it *is* the policy (`026-C6`)"
    );
}

/// proves: Test strategy #17 read with `026-C1` — mode is invisible on the serving
/// face except where the ADR allows it: the same `GET /v1/candidates` request answers
/// byte-shape-identically from a root and from a caching instance serving a cached
/// entry, and the chained pair reports its difference only in `health`.
#[test]
fn the_chained_pair_answers_the_same_request_the_same_way() {
    let root_data = TempDir::new("e2e-modes-root");
    let root_corpus = corpus_in(
        &root_data,
        &[("chained-game", "Chained Game", &["steam/7003"][..])],
    );
    let root = Registry::root("e2e-modes-root", &root_corpus, &[]);

    let cache_data = TempDir::new("e2e-modes-cache");
    let caching = Registry::caching("e2e-modes-cache", &root.url(), &cache_data, &[]);
    assert_eq!(
        status_code(&caching.get(&slug_query("steam/7003"))),
        Some(200),
        "warm it up first"
    );

    let from_root = root.get(&slug_query("steam/7003"));
    let from_cache = caching.get(&slug_query("steam/7003"));
    assert_eq!(
        game_ids(&from_cache),
        game_ids(&from_root),
        "one mode's cached answer is the other mode's curated answer"
    );
    for (label, answer) in [("root", &from_root), ("caching", &from_cache)] {
        assert_eq!(status_code(answer), Some(200), "{label} answers 200");
        assert_eq!(
            common::header_values(answer, "content-length").len(),
            1,
            "`{label}` answers with exactly one `Content-Length` (A1): {:?}",
            common::header_names(answer)
        );
        assert!(
            common::header_values(answer, "transfer-encoding").is_empty(),
            "`{label}` never uses `Transfer-Encoding` (A1): {:?}",
            common::header_names(answer)
        );
    }
    assert_eq!(
        json_body(&root.health())["mode"].as_str(),
        Some("root"),
        "the modes differ where `026-C2` says they may"
    );
    assert_eq!(
        json_body(&caching.health())["mode"].as_str(),
        Some("caching")
    );
}

/// proves: Test strategy #17's chained pair at the granularity `026-C6` fixes — one
/// queried slug is one upstream lookup and one cache entry, in order: the caching
/// instance never trades a lookup for a bulk pull of the root's document (the
/// request-side proof is `caching_policy.rs`, which counts what a fake upstream
/// actually saw; `026-L3`'s "no route answers a whole document" is `admin_auth.rs`).
#[test]
fn the_chained_pair_grows_one_slug_at_a_time() {
    let root_data = TempDir::new("e2e-quiet-root");
    let root_corpus = corpus_in(
        &root_data,
        &[
            ("asked-for", "Asked For", &["steam/7004"][..]),
            ("other-one", "Other One", &["steam/7005"][..]),
            ("other-two", "Other Two", &["epic/7006"][..]),
        ],
    );
    let root = Registry::root("e2e-quiet-root", &root_corpus, &[]);
    let cache_data = TempDir::new("e2e-quiet-cache");
    let caching = Registry::caching("e2e-quiet-cache", &root.url(), &cache_data, &[]);

    let answer = caching.get(&slug_query("steam/7004"));
    assert_eq!(game_ids(&answer), vec!["asked-for".to_string()]);
    assert_eq!(
        json_body(&caching.health())["cached_slugs"].as_u64(),
        Some(1),
        "one lookup, one cached slug"
    );

    let second = caching.get(&slug_query("steam/7005"));
    let candidates = json_body(&second)
        .as_array()
        .expect("a candidate array")
        .clone();
    assert_eq!(
        candidates[0]["game_id"].as_str(),
        Some("other-one"),
        "a second key is answered on its own: {candidates:?}"
    );
    assert_eq!(
        candidates.len(),
        1,
        "with only its own candidates, not the root's whole corpus: {candidates:?}"
    );
    assert_eq!(
        json_body(&caching.health())["cached_slugs"].as_u64(),
        Some(2),
        "and it entered the cache as its own entry"
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
    // A port just learned from a bound-and-closed loopback listener: free to bind,
    // and no test dials or resolves a non-loopback host.
    let port = closed_loopback_port();
    let port_text = port.to_string();
    let server = Registry::root("e2e-explicit-port", &corpus, &["--port", &port_text]);

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
