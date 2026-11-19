//! The registry's own upstream client: the response discipline it copies from the
//! shipped client (A1), the refusal of redirects, the percent-encoded request
//! target (A19), the once-per-TTL reachability probe, and the validation that runs
//! before anything an upstream said is cached.
//!
//! proves: design 026's Test strategy #7 — "Upstream client discipline: rejects
//! `Transfer-Encoding`, duplicated/non-decimal `Content-Length`, over-budget body,
//! over-count candidates, non-`200`; refuses a `3xx` as an upstream error (A1); a
//! space- or %-bearing `slug` reaches upstream only percent-encoded from the
//! decoded key (A19)" — and #8 — "Upstream ingest validation: a malformed upstream
//! `game_id` or `name` is dropped before the cache write, with one escaped
//! single-line echo (A4, A5)" — plus #6's reachability gauge rule (Chosen design:
//! Caching mode, "probed **at most once per TTL** … a health read only reports the
//! verdict as it stands").
//!
//! The upstream is always a loopback fake inside the test process, so a hostile
//! response is a byte string the fake writes, and every refusal is observed as the
//! caching instance's own cold `503` (never `[]`) and its `health.upstream` verdict.

mod common;

use common::{
    FakeUpstream, Registry, TempDir, candidates_body, game_ids, json_body, ok, percent_encode,
    registry_document, respond_to, slug_query, status_code, status_line, unreachable_upstream,
};

/// Runs one cold lookup against a caching instance whose upstream answers `response`
/// and returns the raw answer.
fn cold_lookup(label: &str, response: Vec<u8>) -> (TempDir, FakeUpstream, Registry, Vec<u8>) {
    let data = TempDir::new(label);
    let upstream = FakeUpstream::start(move |request| {
        if request.starts_with("GET /v1/health") {
            return ok(&common::root_health());
        }
        response.clone()
    });
    let server = Registry::caching(label, &upstream.url(), &data, &[]);
    let answer = server.get(&slug_query("steam/900500"));
    (data, upstream, server, answer)
}

/// proves: Test strategy #7's rejection list, one row each — an upstream answer the
/// copied reader must not accept is an upstream error: a cold lookup therefore
/// answers `503` (never `[]`, `026-L4`) and reports `upstream: down`.
#[test]
fn every_ununderstood_upstream_response_is_an_upstream_error() {
    let over_long_body = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        65_537,
        "x".repeat(65_537)
    );
    let candidates_257: Vec<String> = (1..=257)
        .map(|index| format!(r#"{{"game_id":"cand-{index}","name":"Candidate {index}"}}"#))
        .collect();
    let too_many = ok(&format!("[{}]", candidates_257.join(",")));

    let hostile: Vec<(&str, Vec<u8>)> = vec![
        (
            "404",
            b"HTTP/1.1 404 Not Found\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]".to_vec(),
        ),
        (
            "500",
            b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
                .to_vec(),
        ),
        (
            "201-created",
            b"HTTP/1.1 201 Created\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
                .to_vec(),
        ),
        (
            "transfer-encoding",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n0\r\n\r\n"
                .to_vec(),
        ),
        (
            "lowercase-transfer-encoding",
            b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\nConnection: close\r\n\r\n0\r\n\r\n"
                .to_vec(),
        ),
        (
            "missing-content-length",
            b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n[]".to_vec(),
        ),
        (
            "duplicated-content-length",
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
                .to_vec(),
        ),
        (
            "non-decimal-content-length",
            b"HTTP/1.1 200 OK\r\nContent-Length: 0x2\r\nConnection: close\r\n\r\n[]".to_vec(),
        ),
        ("over-budget-body", over_long_body.into_bytes()),
        ("over-count-candidates", too_many),
        ("non-json-body", ok("this is not JSON").to_vec()),
        ("empty-response", Vec::new()),
        (
            "body-shorter-than-its-length",
            b"HTTP/1.1 200 OK\r\nContent-Length: 200\r\nConnection: close\r\n\r\n[]".to_vec(),
        ),
    ];

    for (label, response) in hostile {
        let (_data, _upstream, server, answer) = cold_lookup(&format!("up-{label}"), response);
        assert_eq!(
            status_code(&answer),
            Some(503),
            "the `{label}` upstream response is an upstream error, so the cold slug \
             gets `026-L4`'s `503` and never `[]`: {}",
            status_line(&answer)
        );
        assert!(
            !common::body_text(&answer).contains("[]"),
            "the `{label}` refusal never degrades to an empty candidate list: {:?}",
            common::body_text(&answer)
        );
        assert_eq!(
            json_body(&server.health())["upstream"].as_str(),
            Some("down"),
            "the `{label}` failure moves the reachability gauge to `down`"
        );
        assert!(
            !registry_document(_data.path(), "cache-v1.json").exists()
                || persisted_slugs(&_data).is_empty(),
            "an unusable upstream answer is never cached (`026-C7`): {:?}",
            persisted_slugs(&_data)
        );
    }
}

fn persisted_slugs(data: &TempDir) -> Vec<String> {
    let path = registry_document(data.path(), "cache-v1.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let document: serde_json::Value = serde_json::from_str(&text).expect("the cache is JSON");
    document["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|entry| entry["slug"].as_str().expect("a slug").to_string())
        .collect()
}

/// proves: Test strategy #7's redirect clause — a `3xx` from the configured
/// endpoint is refused as an upstream error rather than followed, because leaving
/// the configured endpoint would break `026-L2`'s disclosure model.
#[test]
fn a_redirect_from_the_upstream_is_refused_as_an_upstream_error() {
    for status in [
        "301 Moved Permanently",
        "302 Found",
        "307 Temporary Redirect",
        "308 Permanent Redirect",
    ] {
        let response = format!(
            "HTTP/1.1 {status}\r\nLocation: http://127.0.0.1:1/v1/candidates?slug=x\r\n\
             Content-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .into_bytes();
        let (_data, upstream, _server, answer) = cold_lookup(
            &format!(
                "up-redirect-{}",
                status.split_whitespace().next().unwrap_or("3xx")
            ),
            response,
        );
        assert_eq!(
            status_code(&answer),
            Some(503),
            "`{status}` is an upstream error (`026-C6`'s answer must come from the \
             configured endpoint): {}",
            status_line(&answer)
        );
        assert!(
            upstream.lines_for("GET /v1/candidates").len() <= 1,
            "a refused redirect is not chased with a second candidate request: {:?}",
            upstream.request_lines()
        );
    }
}

/// proves: Test strategy #7's encoding clause (A19, and the sixth copied invariant,
/// `percent_encode`/`candidates_url`) — the outbound request target is built from
/// the *decoded* slug through the shipped escape-everything rule, so a space- or
/// `%`-bearing key travels only as `%20` / `%25`, and a dot stays a dot.
#[test]
fn the_upstream_request_target_is_built_percent_encoded_from_the_decoded_slug() {
    let data = TempDir::new("up-encoding");
    let upstream = FakeUpstream::start(|_| ok("[]"));
    let server = Registry::caching("up-encoding", &upstream.url(), &data, &[]);

    for (decoded, expected_target) in [
        ("steam/900001", "/v1/candidates?slug=steam%2F900001"),
        ("local/Steam.SE", "/v1/candidates?slug=local%2FSteam.SE"),
        ("local/100%", "/v1/candidates?slug=local%2F100%25"),
        ("local/two words", "/v1/candidates?slug=local%2Ftwo%20words"),
    ] {
        assert!(
            matches!(status_code(&server.get(&slug_query(decoded))), Some(200)),
            "`{decoded:?}` is inside the admitted alphabet, so the lookup is made"
        );
        let targets: Vec<String> = upstream
            .lines_for("GET /v1/candidates")
            .iter()
            .filter_map(|line| line.split_whitespace().nth(1))
            .map(|target| target.to_string())
            .collect();
        assert!(
            targets.iter().any(|seen| seen == expected_target),
            "`{decoded:?}` must reach upstream as `{expected_target}`, never as a raw \
            separator, space, or `%`: {targets:?}"
        );
    }

    // The inbound form the client sent and the outbound form are the same
    // encoding of the same decoded key: no double-encoding, no decoding pass.
    let seen = upstream.lines_for("GET /v1/candidates").join(" ");
    assert!(
        !seen.contains("%252F"),
        "a key is encoded once, from its decoded bytes: {seen}"
    );
    assert_eq!(
        percent_encode("local/100%"),
        "local%2F100%25",
        "the rule the test asserts is the shipped client's rule (A19)"
    );
}

/// proves: Test strategy #8 (`026-C6`'s write seam, open question 9 resolved: yes,
/// both checks) — an upstream `game_id` outside the grammar, or a `name` over
/// `MAX_NATIVE_NAME_OCTETS`, is dropped before the cache write with exactly one
/// escaped single-line echo each, and the survivors keep their order.
#[test]
fn invalid_upstream_records_are_dropped_before_the_cache_write() {
    let long_name = "n".repeat(129);
    let data = TempDir::new("up-ingest");
    let upstream = FakeUpstream::start(respond_to(
        vec![(
            "slug=steam%2F900600".to_string(),
            ok(&candidates_body(&[
                ("../x", "Traversal Twin"),
                ("UPPER-CASE", "Upper Case"),
                ("ok-game", "Ok Game"),
                ("fine-game", long_name.as_str()),
            ])),
        )],
        ok("[]"),
    ));
    let server = Registry::caching("up-ingest", &upstream.url(), &data, &[]);

    let answer = server.get(&slug_query("steam/900600"));
    assert_eq!(status_code(&answer), Some(200), "the survivors are served");
    assert_eq!(
        game_ids(&answer),
        vec!["ok-game".to_string()],
        "a dropped candidate never rides along, and the answer is not failed by it"
    );

    let slugs = persisted_slugs(&data);
    assert_eq!(
        slugs,
        vec!["steam/900600".to_string()],
        "the lookup is cached — with only the valid record inside it"
    );
    let cache_text = std::fs::read_to_string(registry_document(data.path(), "cache-v1.json"))
        .expect("the cache document is written");
    assert!(
        cache_text.contains("ok-game"),
        "the valid record is what got cached: {cache_text}"
    );
    for dropped in ["../x", "UPPER-CASE"] {
        assert!(
            !cache_text.contains(dropped),
            "a grammar-invalid id never enters `cache-v1.json` (A4): {cache_text}"
        );
    }
    assert!(
        !cache_text.contains(&long_name),
        "an over-`013-B4` name never enters `cache-v1.json` (A5)"
    );

    let lines = server.stderr_lines();
    let traversal_lines = lines.iter().filter(|line| line.contains("..")).count();
    assert!(
        (1..=2).contains(&traversal_lines),
        "the dropped `game_id` is echoed once, on a single line of its own: {lines:?}"
    );
    let over_long_lines = lines
        .iter()
        .filter(|line| line.contains("013-B4") || line.contains("name"))
        .count();
    assert!(
        over_long_lines >= 1,
        "the over-long `name` drop is echoed too: {lines:?}"
    );
    for line in &lines {
        assert!(
            !line.contains('\0') && line.trim().is_ascii(),
            "every echo is one escaped, printable line so one hostile upstream cannot \
             forge extra stderr lines: {line:?}"
        );
    }
}

/// proves: Chosen design's Caching-mode reachability rule (Test strategy #6's
/// once-per-TTL probe, `026-C6`) — with the default `026-B3` TTL, five cold lookups
/// cost exactly one `/v1/health` probe (the first lookup fires it lazily) plus one
/// candidate request each; there is no second round trip per lookup and no timer
/// thread probing on its own.
#[test]
fn the_upstream_health_probe_costs_at_most_one_round_trip_per_ttl() {
    let data = TempDir::new("up-probe");
    let upstream = FakeUpstream::start(|request| {
        if request.starts_with("GET /v1/health") {
            return ok(&common::root_health());
        }
        ok(&candidates_body(&[("probed", "Probed Game")]))
    });
    let server = Registry::caching("up-probe", &upstream.url(), &data, &[]);

    // Before any lookup, no probe has been paid for.
    assert_eq!(status_code(&server.health()), Some(200));
    assert_eq!(
        upstream.lines_for("GET /v1/health"),
        Vec::<String>::new(),
        "a health read never fires the probe"
    );

    for index in 0..5 {
        let key = format!("steam/9007{index:02}");
        assert_eq!(status_code(&server.get(&slug_query(&key))), Some(200));
    }

    assert_eq!(
        upstream.lines_for("GET /v1/candidates").len(),
        5,
        "one candidate request per distinct slug: {:?}",
        upstream.request_lines()
    );
    assert_eq!(
        upstream.lines_for("GET /v1/health").len(),
        1,
        "`026-C6`'s gauge is paid once per TTL, by the first lookup after expiry — not \
         per cold lookup: {:?}",
        upstream.request_lines()
    );
    assert_eq!(
        json_body(&server.health())["upstream"].as_str(),
        Some("ok"),
        "and the verdict the gauge reports is the probed one"
    );
}

/// proves: `026-C6`'s reachability posture with no upstream to reach — an
/// unreachable endpoint is `down`, a cold lookup is the `503`, and the instance
/// still answers `200` on health forever (a status document, not a gate).
#[test]
fn an_unreachable_upstream_is_down_never_empty() {
    let (_data, server) = {
        let data = TempDir::new("up-down");
        let server = Registry::caching("up-down", &unreachable_upstream(), &data, &[]);
        (data, server)
    };

    let answer = server.get(&slug_query("steam/900999"));
    assert_eq!(status_code(&answer), Some(503));
    for _ in 0..3 {
        assert_eq!(
            status_code(&server.health()),
            Some(200),
            "an unreachable upstream never turns health into a failure code"
        );
    }
    assert_eq!(
        json_body(&server.health())["upstream"].as_str(),
        Some("down"),
        "a connect failure on a cold fetch reveals reachability too"
    );
}
