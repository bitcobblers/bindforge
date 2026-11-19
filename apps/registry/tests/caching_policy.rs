//! The caching lookup policy: local authority, cache freshness, serve-stale on a
//! failed refresh, and the cold case that must never answer `[]`.
//!
//! proves: design 026's Test strategy #6 — "Lookup policy (`026-C6`/`026-L4`/
//! `026-L5`): local entry answers with zero upstream requests; fresh cache answers;
//! stale-cache + failed-refresh answers stale and sets `upstream: stale`; cold
//! miss + no upstream ⇒ the named `503` (never `[]`), against an injected upstream
//! fake — with the fresh/stale pair run under `--refresh-interval 1`, so the
//! override is what turns a fresh entry stale on the next query and re-times the
//! once-per-TTL upstream probe" — plus the `registry query` half of `026-C6` (open
//! question 12: it writes the cache and exits non-zero on the cold case).
//!
//! Every upstream is a loopback fake, so "zero upstream requests" is a countable
//! fact about that fake. A locally answered or cache-hit lookup must never put a
//! candidate request on it; the once-per-TTL `/v1/health` reachability probe is a
//! different route, and these tests say which is which.

mod common;

use common::{
    FakeUpstream, Registry, TempDir, cache_doc_value, candidates_body, game_ids, json_body,
    now_unix, ok, percent_encode, record_body, registry_document, respond_to, slug_query,
    status_code, status_line, unreachable_upstream,
};

/// A caching instance with documents in a sandbox and `upstream` behind `--upstream`.
fn caching(label: &str, upstream: &str, extra: &[&str]) -> (TempDir, Registry) {
    let data = TempDir::new(label);
    let server = Registry::caching(label, upstream, &data, extra);
    (data, server)
}

fn seed_cache(data: &TempDir, entries: Vec<serde_json::Value>) -> std::path::PathBuf {
    let cache = registry_document(data.path(), "cache-v1.json");
    std::fs::create_dir_all(cache.parent().expect("the registry directory"))
        .expect("the registry directory is creatable");
    std::fs::write(
        &cache,
        cache_doc_value(serde_json::json!(1), entries).as_bytes(),
    )
    .expect("the seeded cache lands");
    cache
}

fn cache_entry(slug: &str, game_id: &str, fetched_at: u64) -> serde_json::Value {
    serde_json::json!({
        "slug": slug,
        "candidates": [{"game_id": game_id, "name": "Cached Game"}],
        "fetched_at": fetched_at,
    })
}

fn persisted_cache(cache: &std::path::Path) -> serde_json::Value {
    let text = std::fs::read_to_string(cache).expect("the cache document is written");
    serde_json::from_str(&text).expect("the cache document is JSON")
}

/// proves: `026-L5` and Test strategy #6's first clause — a locally registered
/// entry answers with **zero** upstream candidate requests, and is never merged
/// with what the upstream would have said.
#[test]
fn a_local_entry_answers_without_a_single_upstream_lookup() {
    let upstream = FakeUpstream::start(respond_to(
        vec![(
            "slug=steam%2F900001".to_string(),
            ok(&candidates_body(&[("upstream-game", "Upstream Game")])),
        )],
        ok("[]"),
    ));
    let (data, server) = caching("policy-local", &upstream.url(), &[]);

    let upsert = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("mine", "Mine", &["steam/900001"]),
    );
    assert!(matches!(status_code(&upsert), Some(200..=299)));

    let answer = server.get(&slug_query("steam/900001"));
    assert_eq!(status_code(&answer), Some(200), "the local entry answers");
    assert_eq!(
        game_ids(&answer),
        vec!["mine".to_string()],
        "a local entry is never merged with upstream candidates (`026-L5`)"
    );
    assert_eq!(
        upstream.lines_for("GET /v1/candidates"),
        Vec::<String>::new(),
        "`026-L5`: a local entry answers without any upstream lookup — the fake saw {:?}",
        upstream.request_lines()
    );
    assert_eq!(
        json_body(&server.health())["cached_slugs"].as_u64(),
        Some(0),
        "answering from a local entry caches nothing (`026-L5` puts it ahead of the \
         cache, not into it)"
    );
    assert!(
        !registry_document(data.path(), "cache-v1.json").exists(),
        "and no cache document is conjured by a locally-answered lookup"
    );
}

/// proves: Test strategy #6 / `026-L5`'s "never expires" half — a local entry wins
/// over a stale cached answer for the same slug, whatever the TTL says.
#[test]
fn a_local_entry_wins_over_a_stale_cache_entry_for_the_same_slug() {
    let data = TempDir::new("policy-local-vs-stale");
    seed_cache(
        &data,
        vec![cache_entry(
            "steam/900001",
            "cached-game",
            now_unix() - 100_000,
        )],
    );
    let upstream = FakeUpstream::start(respond_to(
        vec![(
            "slug=steam%2F900001".to_string(),
            ok(&candidates_body(&[("upstream-game", "Upstream Game")])),
        )],
        ok("[]"),
    ));
    let server = Registry::caching(
        "policy-local-vs-stale",
        &upstream.url(),
        &data,
        &["--refresh-interval", "1"],
    );
    server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("mine", "Mine", &["steam/900001"]),
    );

    let answer = server.get(&slug_query("steam/900001"));
    assert_eq!(game_ids(&answer), vec!["mine".to_string()]);
    assert_eq!(
        upstream.lines_for("GET /v1/candidates"),
        Vec::<String>::new(),
        "the stale cache entry is neither served nor refreshed: the local entry owns \
         the slug"
    );
}

/// proves: Test strategy #6's second clause and `026-B3` — a cached entry inside
/// the TTL answers as-is, at zero upstream cost. The round-1 testing finding 2
/// (accepted) moved this leg off `--refresh-interval 1`: a one-second freshness
/// budget has to cover subprocess start and the printed-port poll, which it cannot
/// honestly do. The freshness subject here is "inside the TTL", so the leg runs on
/// the default `026-B3` TTL and the stamp is safe; the override stays pinned where
/// expiry is the subject (the stale and re-timing legs).
#[test]
fn a_fresh_cache_entry_answers_without_an_upstream_lookup() {
    let data = TempDir::new("policy-fresh");
    seed_cache(
        &data,
        vec![cache_entry("steam/900002", "cached-game", now_unix())],
    );
    let upstream = FakeUpstream::start(|_| ok("[]"));
    let server = Registry::caching("policy-fresh", &upstream.url(), &data, &[]);

    let answer = server.get(&slug_query("steam/900002"));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "the cached answer is served"
    );
    assert_eq!(game_ids(&answer), vec!["cached-game".to_string()]);
    assert_eq!(
        upstream.lines_for("GET /v1/candidates"),
        Vec::<String>::new(),
        "a hit inside `026-B3`'s TTL costs no candidate request: {:?}",
        upstream.request_lines()
    );
}

/// proves: Test strategy #6's third clause and `026-L5`'s staleness rule — a stale
/// entry whose refresh *fails* is served as-is, and `health.upstream` moves to
/// `stale`. Run under `--refresh-interval 1`, so it is the override, not the clock,
/// that makes the entry stale.
#[test]
fn a_failed_refresh_over_a_stale_entry_answers_stale_and_reports_stale() {
    let data = TempDir::new("policy-stale");
    seed_cache(
        &data,
        vec![cache_entry("steam/900003", "cached-game", now_unix() - 5)],
    );
    let upstream = FakeUpstream::start(|_| {
        b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
            .to_vec()
    });
    let server = Registry::caching(
        "policy-stale",
        &upstream.url(),
        &data,
        &["--refresh-interval", "1"],
    );

    let answer = server.get(&slug_query("steam/900003"));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "serve-stale-on-error, the standard posture: {}",
        status_line(&answer)
    );
    assert_eq!(game_ids(&answer), vec!["cached-game".to_string()]);
    assert!(
        !upstream.lines_for("GET /v1/candidates").is_empty(),
        "the refresh was actually attempted: {:?}",
        upstream.request_lines()
    );

    let health = json_body(&server.health());
    assert_eq!(
        health["upstream"].as_str(),
        Some("stale"),
        "`026-C6`: a failed refresh over a stale entry moves `health.upstream` to \
         `stale`: {health:?}"
    );
}

/// proves: `026-L4` and Test strategy #6's last clause — a cold miss with no
/// reachable upstream is the named `503`, never `[]`, and it never writes a cache
/// entry.
#[test]
fn a_cold_miss_with_no_reachable_upstream_is_the_named_503_never_an_empty_array() {
    let (_data, server) = caching("policy-cold", &unreachable_upstream(), &[]);

    let answer = server.get(&slug_query("steam/900004"));
    let line = status_line(&answer);
    assert_eq!(
        status_code(&answer),
        Some(503),
        "`026-L4`: a caching registry with nothing to answer from is the named `503`: \
         {line}"
    );
    assert!(
        line.starts_with("HTTP/1.1 503 ") && line.len() > "HTTP/1.1 503".len() + 1,
        "the reason phrase is the only thing the shipped CLI shows a user (A2), so it \
         must be legible and single-line: {line:?}"
    );
    assert!(
        !line.contains('\0'),
        "the status line is one clean line: {line:?}"
    );
    assert!(
        !common::body_text(&answer).contains("[]"),
        "an empty candidate list states \"never heard of it\", which is not what \
         happened (`026-L4`): {:?}",
        common::body_text(&answer)
    );

    let health = json_body(&server.health());
    assert_eq!(
        health["upstream"].as_str(),
        Some("down"),
        "a failed cold fetch reveals unreachability: {health:?}"
    );
}

/// proves: Test strategy #6's fourth clause and `026-C6`'s "caches it with
/// `fetched_at`" — a cold miss with a reachable upstream fetches, answers, and
/// persists the entry.
#[test]
fn a_cold_miss_with_a_reachable_upstream_fetches_then_caches() {
    let data = TempDir::new("policy-fetch");
    let upstream = FakeUpstream::start(respond_to(
        vec![(
            "slug=steam%2F900005".to_string(),
            ok(&candidates_body(&[
                ("fetched-one", "Fetched One"),
                ("fetched-two", "Fetched Two"),
            ])),
        )],
        ok("[]"),
    ));
    let server = Registry::caching("policy-fetch", &upstream.url(), &data, &[]);

    let answer = server.get(&slug_query("steam/900005"));
    assert_eq!(status_code(&answer), Some(200));
    assert_eq!(
        game_ids(&answer),
        vec!["fetched-one".to_string(), "fetched-two".to_string()],
        "the upstream's answer is served in its own order"
    );
    assert_eq!(
        upstream.lines_for("GET /v1/candidates"),
        vec!["GET /v1/candidates?slug=steam%2F900005 HTTP/1.1".to_string()],
        "exactly one candidate request for one slug"
    );

    let cache = registry_document(data.path(), "cache-v1.json");
    assert!(cache.is_file(), "the answer is cached (`026-C6`)");
    let document = persisted_cache(&cache);
    let entries = document["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 1, "{document:?}");
    assert_eq!(entries[0]["slug"].as_str(), Some("steam/900005"));
    assert!(
        entries[0]["fetched_at"].as_u64().is_some(),
        "the entry carries `fetched_at` (`026-C7`)"
    );
    assert_eq!(
        json_body(&server.health())["cached_slugs"].as_u64(),
        Some(1),
        "health counts what was cached"
    );

    // And the warm answer costs nothing.
    let warm = server.get(&slug_query("steam/900005"));
    assert_eq!(game_ids(&warm), game_ids(&answer));
    assert_eq!(
        upstream.lines_for("GET /v1/candidates").len(),
        1,
        "a warm slug makes no second upstream lookup"
    );
}

/// proves: Test strategy #6's `--refresh-interval` clause — the override is a bare
/// count of seconds that both ages a fresh entry on the next query and re-times the
/// once-per-TTL reachability probe: at `1` second a second lookup after the wait
/// goes back to the upstream, where the default `026-B3` would not. The round-1
/// testing findings 2 and 3 (accepted) reshaped this leg: the fresh entry is no
/// longer stamped before the subprocess starts (a one-second budget cannot honestly
/// cover start-up), it is the entry the instance itself caches on its first
/// lookup — so `fetched_at` is a server-side stamp the wait is measured against —
/// and the probe half of the leg's name is now asserted: `lines_for("GET
/// /v1/health")` must grow by exactly one probe across the wait and the refreshed
/// lookup.
#[test]
fn the_refresh_interval_override_ages_a_fresh_entry_and_retimes_the_probe() {
    let data = TempDir::new("policy-ttl");
    let upstream = FakeUpstream::start(|request| {
        if request.starts_with("GET /v1/health") {
            return ok(&common::root_health());
        }
        if request.contains("slug=steam%2F900006") {
            return ok(&candidates_body(&[("refreshed", "Refreshed Game")]));
        }
        ok("[]")
    });
    let server = Registry::caching(
        "policy-ttl",
        &upstream.url(),
        &data,
        &["--refresh-interval", "1"],
    );

    // The entry is cached by this very lookup, so its `fetched_at` post-dates the
    // subprocess start — the overridden TTL's one second is all wall clock.
    let first = server.get(&slug_query("steam/900006"));
    assert_eq!(
        game_ids(&first),
        vec!["refreshed".to_string()],
        "the cold lookup fetches and caches"
    );
    assert_eq!(
        upstream.lines_for("GET /v1/candidates").len(),
        1,
        "one candidate request for the cold slug: {:?}",
        upstream.request_lines()
    );

    // Still inside the overridden TTL, the entry answers from the cache.
    let warm = server.get(&slug_query("steam/900006"));
    assert_eq!(game_ids(&warm), game_ids(&first));
    assert_eq!(
        upstream.lines_for("GET /v1/candidates").len(),
        1,
        "a fresh entry costs no upstream lookup: {:?}",
        upstream.request_lines()
    );
    let probes_before = upstream.lines_for("GET /v1/health").len();

    std::thread::sleep(std::time::Duration::from_millis(1_500));

    let second = server.get(&slug_query("steam/900006"));
    assert_eq!(
        status_code(&second),
        Some(200),
        "the stale entry refreshes and answers: {}",
        status_line(&second)
    );
    assert_eq!(
        upstream.lines_for("GET /v1/candidates").len(),
        2,
        "one second past the overridden TTL, the entry is stale and is refreshed \
         (`026-B3` overridden by `--refresh-interval`) — under the default TTL this \
         lookup would have cost nothing: {:?}",
        upstream.request_lines()
    );

    // The re-timing half: the once-per-TTL probe is lazy and request-threaded, so
    // give it a moment to land before pinning that exactly one more was paid for.
    let mut probes_after = probes_before;
    for _ in 0..100 {
        probes_after = upstream.lines_for("GET /v1/health").len();
        if probes_after > probes_before {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        probes_after,
        probes_before + 1,
        "the override re-times the reachability probe: the first lookup after the \
         overridden TTL fired exactly one `/v1/health` probe: {:?}",
        upstream.request_lines()
    );
}

/// proves: `026-C6`'s operator surface and open question 12 (Chosen design: "`query`
/// … is the real policy and **does** write the cache, and it exits non-zero on the
/// `026-L4` cold case so scripts see failure") — `registry query <slug>` answers,
/// prints provenance, and persists.
#[test]
fn the_query_subcommand_runs_the_policy_prints_provenance_and_writes_the_cache() {
    let data = TempDir::new("query-fetch");
    let xdg = data
        .path()
        .to_str()
        .expect("a utf-8 data directory")
        .to_string();
    let upstream = FakeUpstream::start(respond_to(
        vec![(
            "slug=steam%2F900007".to_string(),
            ok(&candidates_body(&[("queried", "Queried Game")])),
        )],
        ok("[]"),
    ));

    let run = common::run_registry(
        &["query", "steam/900007", "--upstream", &upstream.url()],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        run.success(),
        "a query the upstream can answer succeeds; stderr: {}",
        run.stderr
    );
    assert!(
        run.stdout.contains("queried"),
        "the answer is printed: {:?}",
        run.stdout
    );
    let provenance = run.stdout.to_ascii_lowercase();
    assert!(
        ["cache", "upstream", "local", "root"]
            .iter()
            .any(|word| provenance.contains(word)),
        "`026-C6`: query prints the answer *with its provenance*; the output names \
         none of the provenance words: {:?}",
        run.stdout
    );
    let cache = registry_document(data.path(), "cache-v1.json");
    assert!(
        cache.is_file(),
        "query is the real policy, so it writes the cache"
    );
    assert_eq!(
        upstream.lines_for("GET /v1/candidates"),
        vec!["GET /v1/candidates?slug=steam%2F900007 HTTP/1.1".to_string()],
        "query runs the policy exactly once"
    );
}

/// proves: `026-C6`'s cold-case exit code (open question 12 resolved: non-zero) — a
/// one-shot `query` with nothing to answer from fails, rather than printing an empty
/// answer and exiting 0.
#[test]
fn the_query_subcommand_exits_non_zero_on_the_cold_case() {
    let data = TempDir::new("query-cold");
    let xdg = data
        .path()
        .to_str()
        .expect("a utf-8 data directory")
        .to_string();
    let upstream = unreachable_upstream();

    let run = common::run_registry(
        &["query", "steam/900008", "--upstream", &upstream],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        !run.success(),
        "`026-L4`'s cold case is a failed run for scripts; stdout {:?}, stderr {:?}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.stdout.contains("[]"),
        "and it never prints an empty answer as if it were a real one: {:?}",
        run.stdout
    );
}

/// proves: `026-C6`'s provenance for a locally-owned answer, one-shot: `query` on a
/// registered entry needs no upstream at all.
#[test]
fn the_query_subcommand_answers_a_registered_entry_without_an_upstream() {
    let data = TempDir::new("query-local");
    let local_entries = registry_document(data.path(), "catalog-v1.json");
    std::fs::create_dir_all(local_entries.parent().expect("the registry directory"))
        .expect("the registry directory is creatable");
    std::fs::write(
        &local_entries,
        common::corpus_doc(1, &[("mine", "Mine", &["steam/900009"][..])]).as_bytes(),
    )
    .expect("the local entries land");

    let xdg = data
        .path()
        .to_str()
        .expect("a utf-8 data directory")
        .to_string();
    let upstream = unreachable_upstream();
    let run = common::run_registry(
        &["query", "steam/900009", "--upstream", &upstream],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        run.success(),
        "a registered entry answers with no reachable upstream (`026-L5`); stderr: {}",
        run.stderr
    );
    assert!(
        run.stdout.contains("mine"),
        "the local record is printed: {:?}",
        run.stdout
    );
    assert!(
        run.stdout.to_ascii_lowercase().contains("local"),
        "and it is labelled as local knowledge: {:?}",
        run.stdout
    );
}

/// proves: `026-L2` as the caching registry's outbound half (Test strategy #6's
/// injected-fake posture) — the upstream sees the queried key and nothing
/// identifying: no credential header, no bulk request.
#[test]
fn the_caching_registry_sends_only_the_queried_key_upstream() {
    let data = TempDir::new("policy-disclosure");
    let upstream = FakeUpstream::start(respond_to(
        vec![(
            format!("slug={}", percent_encode("steam/900010")),
            ok(&candidates_body(&[("one", "One")])),
        )],
        ok("[]"),
    ));
    let server = Registry::caching(
        "policy-disclosure",
        &upstream.url(),
        &data,
        &["--admin-token", "a-secret-admin-token"],
    );

    assert_eq!(
        status_code(&server.get(&slug_query("steam/900010"))),
        Some(200)
    );
    let seen = upstream.raw_requests().join("\n---\n");
    assert!(
        seen.contains("steam%2F900010"),
        "the queried key travels, percent-encoded (`026-L2`'s consent act): {seen}"
    );
    assert!(
        !seen.contains("a-secret-admin-token"),
        "the configured admin token never rides the outbound hop: {seen}"
    );
    assert!(
        !upstream
            .lines_for("GET /v1/candidates")
            .iter()
            .any(|line| line.contains("slug=steam%2F900011")),
        "only the lookup the client asked for is fetched — never a bulk pull \
         (`026-L2`, `026-L3`): {:?}",
        upstream.request_lines()
    );
}
