//! The `--upstream` endpoint gate: what spelling of an upstream endpoint the CLI
//! admits at all, and what the admission is *not* allowed to imply. Literal
//! `http://` with a parseable host and port is admitted for any host — the
//! loopback-only allowlist is gone with `026-L1` — while `https`, `ftp`, an
//! `@`, an empty value, and a non-numeric port are refused before the process
//! binds. Admission is not a connection: a freshly started caching instance has
//! dialled nothing.
//!
//! proves: design 026's Test strategy #14 — "`--upstream` endpoint gate: literal
//! `http://` + parseable host/port admitted (any host); `https`/`ftp`/`@`/empty/
//! non-numeric refused (A11)."
//!
//! `endpoint.rs` holds the copied `parse_endpoint` rules (A11) and `apps/registry`
//! is a bin-only crate (`026-C1`), so the gate is pinned from outside: a refused
//! spelling means the process exits non-zero instead of binding, and an admitted
//! one means it binds and prints its port. The refusal rows use the schemes the
//! gate rejects before it looks at a host, so no name in them is ever resolved;
//! the admission rows keep their hosts inside the loopback range and never issue a
//! lookup, which is what keeps the A15 invariant the shipped gate suite carries
//! ("no test dials or resolves a non-loopback host") true here too — the design
//! pins remote-host admission at `parse_endpoint` in a unit suite (#15), and the
//! `127.0.0.5` row is how far "any host" can be pinned from a subprocess without
//! breaking that rule.

mod common;

use common::{
    FakeUpstream, Registry, TempDir, closed_loopback_port, expect_refused_startup, json_body, ok,
    respond_to, slug_query, status_code, status_line,
};

/// proves: Test strategy #14's refusal list, one row per spelling the design names
/// (plus the two ways a `http://` prefix goes wrong) — the instance never binds, so
/// the process has exited on its own, and its refusal says which argument it is
/// about.
#[test]
fn the_upstream_gate_refuses_every_spelling_beyond_a_literal_http_origin() {
    let cases: Vec<(&str, &str)> = vec![
        ("https-scheme", "https://127.0.0.1:8080"),
        ("https-remote-host", "https://catalog.example"),
        ("ftp-scheme", "ftp://127.0.0.1:8080"),
        ("userinfo-in-the-url", "http://operator@127.0.0.1:8080"),
        ("empty-value", ""),
        ("non-numeric-port", "http://127.0.0.1:not-a-port"),
        ("no-scheme-at-all", "127.0.0.1:8080"),
        ("one-slash-after-the-colon", "http:/127.0.0.1:8080"),
        ("port-but-no-host", "http://:8080"),
    ];

    for (label, spelling) in cases {
        let run = expect_refused_startup(&["serve", "--upstream", spelling], &[]);
        let explanation = run.stderr.to_lowercase();
        assert!(
            explanation.contains("upstream"),
            "`{label}` ({spelling:?}) is refused as an `--upstream` value and says so; the \
             process said {:?}",
            run.stderr
        );
    }
}

/// proves: Test strategy #14's admission clause and the reachability rule that comes
/// with it — an admitted `http://` origin is *not* contacted: a freshly started
/// caching instance has made no request of its upstream at all, so the counted fake
/// stays at zero through startup and a `/v1/health` read (`026-C2` answers the
/// verdict from the last-known value, never from a probe fired here).
#[test]
fn an_admitted_upstream_origin_is_not_dialled_at_startup() {
    let data = TempDir::new("gate-admit");
    let upstream = FakeUpstream::start(respond_to(Vec::new(), ok("[]")));
    let server = Registry::caching("gate-admit", &upstream.url(), &data, &[]);

    let health_answer = server.health();
    let health = json_body(&health_answer);
    assert_eq!(
        status_code(&health_answer),
        Some(200),
        "the instance is serving: {}",
        status_line(&health_answer)
    );
    assert_eq!(
        health["mode"].as_str(),
        Some("caching"),
        "the flag chose the mode: {health:?}"
    );
    assert_eq!(
        health["last_upstream_fetch"].as_u64(),
        Some(0),
        "admission records no fetch: {health:?}"
    );
    assert_ne!(
        health["upstream"].as_str(),
        Some("ok"),
        "an endpoint that has never answered cannot report `ok`: {health:?}"
    );
    assert_eq!(
        upstream.request_count(),
        0,
        "startup and a health read put no request on the upstream: {:?}",
        upstream.request_lines()
    );

    // And the instance is genuinely the caching mode, not a root that ignored the
    // flag: the lookup is where the dial belongs, not startup.
    let cold = server.get(&slug_query("steam/900700"));
    assert_eq!(
        status_code(&cold),
        Some(200),
        "an admitted endpoint that answers `[]` answers it: {}",
        status_line(&cold)
    );
    assert_eq!(
        upstream.request_count(),
        1,
        "the lookup, not startup, is what reaches the upstream: {:?}",
        upstream.request_lines()
    );
}

/// proves: Test strategy #14's "(any host)" half, as far as a subprocess can pin it
/// under the A15 invariant — the gate is not a host allowlist any more (`026-L1`):
/// a loopback-range spelling that is not `127.0.0.1`, and the `localhost` spelling,
/// are both admitted with a parseable port, bind, and serve, while still having
/// dialled nothing.
#[test]
fn the_gate_admits_any_parseable_host_not_only_the_loopback_spelling() {
    for (label, host) in [
        ("loopback-range", "127.0.0.5"),
        ("localhost-name", "localhost"),
    ] {
        let data = TempDir::new(&format!("gate-host-{label}"));
        let url = format!("http://{host}:{}", closed_loopback_port());
        let server = Registry::caching(&format!("gate-host-{label}"), &url, &data, &[]);

        let answer = server.health();
        let health = json_body(&answer);
        assert_eq!(
            health["mode"].as_str(),
            Some("caching"),
            "`{label}` ({url}) was admitted as an upstream endpoint, so this is the caching \
             instance: {health:?}"
        );
        assert_eq!(
            health["last_upstream_fetch"].as_u64(),
            Some(0),
            "`{label}` is admitted, not contacted: {health:?}"
        );
    }
}
