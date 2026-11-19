//! `GET /v1/health` — a status document, never a readiness gate, never gated, and
//! never the thing that costs an upstream round trip.
//!
//! proves: design 026's Test strategy #12 — "Health (`026-C2`): always `200`;
//! reports `mode`/`upstream`/counts; the `upstream` verdict is read from the
//! last-known cached value and a health read never fires the TTL probe; a freshly
//! started caching instance answers `upstream: down`, `last_upstream_fetch: 0`" —
//! together with `026-L3`'s permission for summary counts and Chosen design: The
//! serving face ("it never answers non-`200`, so a `down` upstream is reported in
//! the body, not in the code").

mod common;

use common::{
    FakeUpstream, Registry, TempDir, corpus_doc, json_body, ok, root_health, status_code,
    status_line,
};

/// The six fields `026-C2` fixes, and nothing else load-bearing.
fn assert_health_shape(document: &serde_json::Value) {
    let object = document
        .as_object()
        .expect("the health document is a JSON object");
    for field in [
        "status",
        "mode",
        "upstream",
        "local_entries",
        "cached_slugs",
        "last_upstream_fetch",
    ] {
        assert!(
            object.contains_key(field),
            "`026-C2` fixes the health body as `{{status, mode, upstream, local_entries, \
             cached_slugs, last_upstream_fetch}}`; {field} is missing: {object:?}"
        );
    }
    assert!(object["status"].is_string(), "`status` is a string");
    for field in ["local_entries", "cached_slugs", "last_upstream_fetch"] {
        assert!(
            object[field].as_u64().is_some(),
            "`{field}` is a non-negative integer count (`026-L3` allows summary counts \
             only): {object:?}"
        );
    }
}

/// proves: Test strategy #12 (root mode's shape) — `mode: root`, `upstream: none`
/// (root has no upstream to be stale about), and the corpus counts.
#[test]
fn root_health_reports_the_named_shape() {
    let data = TempDir::new("health-root");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(
            4,
            &[
                ("example-game", "Example Game", &["steam/900001"][..]),
                (
                    "other-game",
                    "Other Game",
                    &["epic/77", "local/Some.Dir"][..],
                ),
            ],
        )
        .as_bytes(),
    );
    let server = Registry::root("health-root", &corpus, &[]);

    let answer = server.health();
    assert_eq!(status_code(&answer), Some(200), "health is always a 200");
    let document = json_body(&answer);
    assert_health_shape(&document);
    assert_eq!(document["mode"].as_str(), Some("root"));
    assert_eq!(
        document["upstream"].as_str(),
        Some("none"),
        "`upstream` is `none` in root mode (`026-C2`)"
    );
    assert_eq!(
        document["local_entries"].as_u64(),
        Some(2),
        "two curated games"
    );
    assert_eq!(
        document["cached_slugs"].as_u64(),
        Some(0),
        "a root holds no cache"
    );
}

/// proves: Test strategy #12's last clause (Chosen design names the never-fetched
/// state) — a caching instance that has not contacted its upstream reports
/// `down` with `last_upstream_fetch: 0`, and still answers `200`.
#[test]
fn a_freshly_started_caching_instance_reports_down_with_zero_last_fetch() {
    let data = TempDir::new("health-cold");
    let upstream = FakeUpstream::start(|_| ok(&root_health()));
    let server = Registry::caching("health-cold", &upstream.url(), &data, &[]);

    let answer = server.health();
    assert_eq!(
        status_line(&answer),
        "HTTP/1.1 200 OK",
        "a `down` upstream is reported in the body, never in the code"
    );
    let document = json_body(&answer);
    assert_health_shape(&document);
    assert_eq!(document["mode"].as_str(), Some("caching"));
    assert_eq!(
        document["upstream"].as_str(),
        Some("down"),
        "never contacted ⇒ `down` — the named value of the never-fetched state"
    );
    assert_eq!(document["last_upstream_fetch"].as_u64(), Some(0));
    assert_eq!(document["cached_slugs"].as_u64(), Some(0));
    assert_eq!(document["local_entries"].as_u64(), Some(0));
}

/// proves: Test strategy #12's central clause (revision-log r2: "only lookups
/// fire the once-per-TTL probe — a health read never does") — reading health, even
/// many times, on an instance that has never fetched, must not put a single
/// request on the upstream.
#[test]
fn reading_health_never_fires_the_upstream_probe() {
    let data = TempDir::new("health-no-probe");
    let upstream = FakeUpstream::start(|_| ok(&root_health()));
    let server = Registry::caching("health-no-probe", &upstream.url(), &data, &[]);

    for _ in 0..5 {
        assert_eq!(status_code(&server.health()), Some(200));
    }
    assert_eq!(
        upstream.request_count(),
        0,
        "answering `/v1/health` reads the last-known verdict and never fires the \
         TTL probe; the fake recorded {:?} after five health reads",
        upstream.request_lines()
    );
    assert_eq!(
        json_body(&server.health())["upstream"].as_str(),
        Some("down"),
        "and the reported verdict is the last-known one, not a freshly-probed one"
    );
}

/// proves: Test strategy #12 with `026-L3`/`026-C5` — health is never gated: with
/// an admin token configured and a non-loopback bind that disables the admin face,
/// `/v1/health` still answers `200` with no credential.
#[test]
fn health_is_never_gated() {
    let data = TempDir::new("health-ungated");
    let upstream = common::unreachable_upstream();
    let server = Registry::caching(
        "health-ungated",
        &upstream,
        &data,
        &["--host", "0.0.0.0", "--admin-token", "s3cret-token"],
    );

    let answer = server.health();
    assert_eq!(
        status_code(&answer),
        Some(200),
        "`026-L3` permits the summary counts, so health is never gated: {}",
        status_line(&answer)
    );
    let document = json_body(&answer);
    assert_health_shape(&document);
    assert_eq!(
        document["upstream"].as_str(),
        Some("down"),
        "an unreachable upstream is a body value, not a status code"
    );
}
