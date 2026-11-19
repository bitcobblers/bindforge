//! The admin face's enablement and auth rule: loopback answers with no token, a
//! configured token is required everywhere, a routable bind with no token has no
//! admin face at all, and a refusal never echoes the credential.
//!
//! proves: design 026's Test strategy #9 — "Admin auth (`026-C5`): loopback +
//! no-token enables without a credential; a configured token is required on every
//! admin request including loopback; non-loopback + no-token disables (admin ⇒
//! `404`); constant-time token compare; wrong/missing ⇒ `401` with
//! `WWW-Authenticate` and no echo of the configured token; `405` carries `Allow`"
//! — with the open-Q11 resolution stated in Chosen design: The admin face and auth
//! ("configured ⇒ required everywhere"; enablement judged from the address actually
//! bound via `IpAddr::is_loopback` on the listener's `local_addr()`).
//!
//! The routable bind is `0.0.0.0`, whose `local_addr()` is not loopback; every run
//! still talks to the instance over `127.0.0.1`, so nothing leaves the host. Only
//! methods a route actually carries are asked without a credential, so the auth
//! posture is measured without colliding with the `405` rule.

mod common;

use common::{
    Registry, TempDir, body_text, build_request_with_length_claim, corpus_doc, header_names,
    header_values, json_body, record_body, status_code, status_line,
};

const TOKEN: &str = "s3cret-registry-token";

fn root(label: &str, extra: &[&str]) -> (TempDir, std::path::PathBuf, Registry) {
    let data = TempDir::new(label);
    let corpus = data.write(
        "corpus.json",
        corpus_doc(
            0,
            &[("example-game", "Example Game", &["steam/900001"][..])],
        )
        .as_bytes(),
    );
    let server = Registry::root(label, &corpus, extra);
    (data, corpus, server)
}

/// The three admin requests, each with a method its route carries.
fn carried_admin_requests() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("GET", "/v1/admin/stats", ""),
        (
            "POST",
            "/v1/admin/games",
            "{\"game_id\":\"new-game\",\"name\":\"New Game\",\"source_keys\":[]}",
        ),
        ("DELETE", "/v1/admin/games/example-game", ""),
    ]
}

/// proves: `026-C5` first clause (Test strategy #9) — a loopback bind with no
/// token configured has a working admin face with no credential at all.
#[test]
fn a_loopback_admin_face_answers_with_no_credential() {
    let (_data, _corpus, server) = root("auth-loopback", &[]);

    let stats = server.get("/v1/admin/stats");
    assert_eq!(
        status_code(&stats),
        Some(200),
        "loopback + no token ⇒ enabled with no friction: {}",
        status_line(&stats)
    );

    let upsert = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("new-game", "New Game", &["steam/900002"]),
    );
    assert!(
        matches!(status_code(&upsert), Some(200..=299)),
        "an unauthenticated loopback upsert is accepted: {}",
        status_line(&upsert)
    );
}

/// proves: `026-C5` as design 026 resolves open Q11 (Test strategy #9) — once a
/// token is configured it is required on every admin request, including on a
/// loopback bind, on all three routes.
#[test]
fn a_configured_token_is_required_on_every_admin_request_including_loopback() {
    let (_data, corpus, server) = root("auth-required", &["--admin-token", TOKEN]);
    let before = std::fs::read(&corpus).expect("the corpus is readable");

    for (method, target, body) in carried_admin_requests() {
        let anonymous = server.request(method, target, &[], body);
        assert_eq!(
            status_code(&anonymous),
            Some(401),
            "`{method} {target}` with a token configured and no `Authorization` is `401`, \
             not a loopback free pass: {}",
            status_line(&anonymous)
        );
        let typo = format!("{TOKEN}-typo");
        let wrong = server.with_token(method, target, &typo, body);
        assert_eq!(
            status_code(&wrong),
            Some(401),
            "`{method} {target}` with a wrong token is `401`: {}",
            status_line(&wrong)
        );
        assert_eq!(
            std::fs::read(&corpus).expect("the corpus survives"),
            before,
            "a refused `{method} {target}` mutates nothing"
        );
    }

    let upsert = server.with_token(
        "POST",
        "/v1/admin/games",
        TOKEN,
        &record_body("new-game", "New Game", &["steam/900002"]),
    );
    assert!(
        matches!(status_code(&upsert), Some(200..=299)),
        "the configured token is accepted: {}",
        status_line(&upsert)
    );
    assert_eq!(
        status_code(&server.get_with_token("/v1/admin/stats", TOKEN)),
        Some(200)
    );
    assert!(
        std::fs::read(&corpus).expect("the corpus survives") != before,
        "an authenticated mutation lands"
    );
}

/// proves: Test strategy #9's `401` shape — a missing or wrong credential answers
/// `401` carrying a `WWW-Authenticate` naming the bearer scheme (A20's
/// conformance choice), and the configured token never appears in the answer or in
/// the instance's output.
#[test]
fn a_refused_admin_request_is_401_with_a_bearer_challenge_and_no_token_echo() {
    let (_data, _corpus, server) = root("auth-401", &["--admin-token", TOKEN]);

    let cases: Vec<(&str, Vec<String>)> = vec![
        ("missing", vec![]),
        (
            "wrong",
            vec!["Authorization: Bearer not-the-token".to_string()],
        ),
        ("empty-scheme", vec!["Authorization: Bearer".to_string()]),
        (
            "other-scheme",
            vec![format!("Authorization: Basic {TOKEN}")],
        ),
    ];
    for (label, headers) in cases {
        let owned: Vec<&str> = headers.iter().map(String::as_str).collect();
        let answer = server.request("GET", "/v1/admin/stats", &owned, "");
        assert_eq!(
            status_code(&answer),
            Some(401),
            "`{label}` is refused with `401`: {}",
            status_line(&answer)
        );
        let challenge = header_values(&answer, "WWW-Authenticate").join(",");
        assert!(
            challenge.to_ascii_lowercase().contains("bearer"),
            "the `401` names the scheme it expects (`WWW-Authenticate: Bearer`, A20): \
             {challenge:?}"
        );
        assert!(
            !body_text(&answer).contains(TOKEN),
            "the configured token never appears in a refusal body: {:?}",
            body_text(&answer)
        );
    }
    assert!(
        !server.stderr_text().contains(TOKEN),
        "the configured token never appears in the instance's log: {}",
        server.stderr_text()
    );
}

/// proves: `026-C5`'s second clause and design 026's resolution of the ADR's
/// ambiguity (Test strategy #9) — a non-loopback bind with **no** token leaves the
/// admin face disabled, a disabled admin route is a `404` (never a `401` oracle),
/// and the serving face keeps answering.
#[test]
fn a_non_loopback_bind_without_a_token_disables_the_admin_face() {
    let (_data, corpus, server) = root("auth-disabled", &["--host", "0.0.0.0"]);
    let before = std::fs::read(&corpus).expect("the corpus is readable");

    for (method, target, body) in carried_admin_requests() {
        let answer = server.request(method, target, &[], body);
        assert_eq!(
            status_code(&answer),
            Some(404),
            "admin disabled by an ungated routable bind ⇒ `{method} {target}` is the \
             named `404`, not a `401`: {}",
            status_line(&answer)
        );
    }
    let with_token_anyway = server.with_token("GET", "/v1/admin/stats", "whatever", "");
    assert_eq!(
        status_code(&with_token_anyway),
        Some(404),
        "an unconfigured token cannot unlock a disabled admin face"
    );
    assert_eq!(
        std::fs::read(&corpus).expect("the corpus survives"),
        before,
        "a disabled admin face mutates nothing"
    );

    let serving = server.get("/v1/candidates?slug=steam%2F900001");
    assert_eq!(
        status_code(&serving),
        Some(200),
        "disabling admin does not disable the serving face: {}",
        status_line(&serving)
    );
    assert_eq!(
        status_code(&server.health()),
        Some(200),
        "and health stays open (`026-L3`)"
    );
}

/// proves: the flat-404 posture of a disabled admin face, as the round-1 security
/// finding (accepted) states it — with the face disabled (a routable bind, no
/// token) every admin path answers the *same* `404` an unknown route gets, for
/// every method including ones an enabled route would answer with a `405`, and no
/// answer carries an `Allow` header. An `Allow` list on a disabled face is an
/// existence oracle, and `026-C5`'s rule is that the routes are *absent*.
#[test]
fn a_disabled_admin_face_publishes_no_allow_list_for_any_method() {
    let (_data, corpus, server) = root("auth-flat404", &["--host", "0.0.0.0"]);
    let before = std::fs::read(&corpus).expect("the corpus is readable");

    let unknown_route = server.get("/v1/no-such-route");
    let unknown_line = status_line(&unknown_route);
    assert_eq!(status_code(&unknown_route), Some(404), "baseline 404");

    let body = "{\"game_id\":\"new-game\",\"name\":\"New Game\",\"source_keys\":[]}";
    for (method, target) in [
        ("GET", "/v1/admin/stats"),
        ("POST", "/v1/admin/games"),
        ("PUT", "/v1/admin/games"),
        ("DELETE", "/v1/admin/games/example-game"),
        ("PUT", "/v1/admin/games/example-game"),
    ] {
        let answer = server.request(method, target, &[], body);
        assert_eq!(
            status_code(&answer),
            Some(404),
            "`{method} {target}` on a disabled face is the named `404`, not a `405`: \
             {}",
            status_line(&answer)
        );
        assert_eq!(
            status_line(&answer),
            unknown_line,
            "`{method} {target}` is indistinguishable from an unknown route — one flat \
             404, publishing nothing about the face: {}",
            status_line(&answer)
        );
        assert!(
            header_values(&answer, "Allow").is_empty(),
            "a disabled face names no methods: {:?}",
            header_names(&answer)
        );
    }
    assert_eq!(
        std::fs::read(&corpus).expect("the corpus survives"),
        before,
        "nothing behind a disabled face mutated"
    );
}

/// proves: the `Origin` half of the round-1 security finding (the browser-shaped
/// request this plain-HTTP bearer face can never authenticate) — an admin request
/// carrying an `Origin`, even with a valid credential, is the named `400`, decided
/// before any mutation: the document is byte-identical afterwards and the same
/// request without `Origin` still lands.
#[test]
fn an_admin_request_carrying_an_origin_is_400_and_mutates_nothing() {
    let (_data, corpus, server) = root("auth-origin", &["--admin-token", TOKEN]);
    let before = std::fs::read(&corpus).expect("the corpus is readable");
    let before_counts = json_body(&server.get_with_token("/v1/admin/stats", TOKEN));

    let body = record_body("no-forged-write", "No Forged Write", &["steam/900300"]);
    let bearer = format!("Authorization: Bearer {TOKEN}");
    let cases: Vec<(&str, &str, &str, &str)> = vec![
        ("POST", "/v1/admin/games", bearer.as_str(), &body),
        (
            "DELETE",
            "/v1/admin/games/example-game",
            bearer.as_str(),
            "",
        ),
        ("GET", "/v1/admin/stats", bearer.as_str(), ""),
    ];
    for (method, target, credential, request_body) in cases {
        for origin in ["http://evil.example", "null", "https://bindforge.example"] {
            let answer = server.request(
                method,
                target,
                &[credential, &format!("Origin: {origin}")],
                request_body,
            );
            assert_eq!(
                status_code(&answer),
                Some(400),
                "`{method} {target}` with `Origin: {origin:?}` is the named `400`: {}",
                status_line(&answer)
            );
        }
    }
    assert_eq!(
        std::fs::read(&corpus).expect("the corpus survives"),
        before,
        "an `Origin`-bearing admin request mutates nothing"
    );
    assert_eq!(
        json_body(&server.get_with_token("/v1/admin/stats", TOKEN)),
        before_counts,
        "and the model is untouched"
    );

    // The same authenticated write without an `Origin` still lands: the refusal is
    // the browser shape, not the credential or the record.
    let landed = server.request(
        "POST",
        "/v1/admin/games",
        &[&format!("Authorization: Bearer {TOKEN}")],
        &body,
    );
    assert!(
        matches!(status_code(&landed), Some(200..=299)),
        "the same request without `Origin` is accepted: {}",
        status_line(&landed)
    );
    assert_ne!(
        std::fs::read(&corpus).expect("the corpus survives"),
        before,
        "and it mutated the document"
    );
}

/// proves: the credential-before-body half of the round-1 security finding
/// (accepted) — the admin gate runs on the request head, so an unauthenticated
/// `POST` that declares a body and then stops writing is refused before the body
/// arrives: the `401` lands in well under the socket timeout this test enforces
/// (the request-read timeout is orders of magnitude longer, and a server that
/// buffered the declared body would let this leg run into the timeout instead of
/// answering).
#[test]
fn an_unauthenticated_admin_request_is_refused_before_its_declared_body_arrives() {
    use std::io::{Read, Write};
    use std::time::{Duration, Instant};

    let (_data, _corpus, server) = root("auth-head-gate", &["--admin-token", TOKEN]);

    let raw = build_request_with_length_claim(
        "POST",
        "/v1/admin/games",
        &server.host_header(),
        "1048576",
        "",
    );
    let mut stream = server.connection();
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .expect("a socket read timeout");
    let started = Instant::now();
    stream.write_all(&raw).expect("the head is written");
    stream.flush().ok();

    let mut answer = Vec::new();
    let mut window = [0_u8; 4096];
    loop {
        match stream.read(&mut window) {
            Ok(0) => break,
            Ok(read) => answer.extend_from_slice(&window[..read]),
            Err(err) => panic!(
                "no refusal arrived for an unauthenticated POST with an unsent body \
                 within 20s of the head ({err}) — a gate that reads the declared body \
                 before checking the credential is exactly what this leg refuses"
            ),
        }
    }
    let elapsed = started.elapsed();
    assert_eq!(
        status_code(&answer),
        Some(401),
        "the head alone is enough to refuse: {}",
        status_line(&answer)
    );
    assert!(
        elapsed < Duration::from_secs(15),
        "the `401` is not separated from the request by waiting on a body that never \
         arrives: it took {elapsed:?}"
    );
    assert!(
        !header_values(&answer, "WWW-Authenticate").is_empty(),
        "and it is the bearer challenge of the enabled face: {:?}",
        header_names(&answer)
    );
}

/// proves: open question 11 as design 026 answers it (Test strategy #9) — the
/// compared value is the token *after* the `Bearer ` scheme token, not the whole
/// header.
#[test]
fn the_compared_value_is_the_token_after_the_bearer_scheme() {
    let (_data, _corpus, server) = root("auth-scheme", &["--admin-token", TOKEN]);

    let bare = server.request(
        "GET",
        "/v1/admin/stats",
        &[&format!("Authorization: {TOKEN}")],
        "",
    );
    assert_eq!(
        status_code(&bare),
        Some(401),
        "the bare token without its scheme is not the compared value: {}",
        status_line(&bare)
    );
    let prefixed = server.get_with_token("/v1/admin/stats", TOKEN);
    assert_eq!(
        status_code(&prefixed),
        Some(200),
        "`Authorization: Bearer <token>` is the accepted spelling: {}",
        status_line(&prefixed)
    );
}

/// proves: Test strategy #9's "constant-time token compare", as far as an external
/// observer can see it — a wrong token of the *same* length as the configured one
/// gets a refusal identical to a wrong token of a different length, so the answer
/// leaks nothing about where a comparison stopped. (The timing property itself is a
/// code property; this pins its whole observable half — A20, `026-C5`.)
#[test]
fn a_wrong_token_refuses_identically_whatever_its_length() {
    let (_data, _corpus, server) = root("auth-timing", &["--admin-token", TOKEN]);

    let same_length = format!("Authorization: Bearer {}", "x".repeat(TOKEN.len()));
    let shorter = "Authorization: Bearer xx".to_string();

    let long = server.request("GET", "/v1/admin/stats", &[same_length.as_str()], "");
    let short = server.request("GET", "/v1/admin/stats", &[shorter.as_str()], "");

    assert_eq!(status_line(&long), status_line(&short), "same status line");
    assert_eq!(body_text(&long), body_text(&short), "same body");
    let mut long_names = header_names(&long);
    let mut short_names = header_names(&short);
    long_names.sort();
    short_names.sort();
    assert_eq!(long_names, short_names, "same header names");
    assert_eq!(status_code(&long), Some(401));
}

/// proves: `026-C5`'s configuration surface (`--admin-token` **or**
/// `BINDFORGE_REGISTRY_ADMIN_TOKEN`) — the env var configures the same requirement
/// and is the only other way in.
#[test]
fn the_admin_token_env_var_configures_the_same_requirement() {
    let data = TempDir::new("auth-env");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(
            0,
            &[("example-game", "Example Game", &["steam/900001"][..])],
        )
        .as_bytes(),
    );
    let server = Registry::start(
        "auth-env",
        &[
            "serve",
            "--root",
            "--corpus",
            corpus.to_str().expect("a utf-8 corpus path"),
        ],
        &[("BINDFORGE_REGISTRY_ADMIN_TOKEN", TOKEN)],
    );

    assert_eq!(
        status_code(&server.get("/v1/admin/stats")),
        Some(401),
        "the env var is a real token configuration"
    );
    assert_eq!(
        status_code(&server.get_with_token("/v1/admin/stats", TOKEN)),
        Some(200),
        "and it accepts the same bearer spelling"
    );
    assert!(
        !server.stderr_text().contains(TOKEN),
        "the configured token never appears in the instance's own output"
    );
}

/// proves: Test strategy #9's last clause on the admin routes — a method a route
/// does not carry is a `405` naming the methods it does, once the caller is
/// authenticated.
#[test]
fn an_authenticated_wrong_method_on_an_admin_route_is_405_naming_its_methods() {
    let (_data, _corpus, server) = root("auth-405", &["--admin-token", TOKEN]);

    let delete_of_the_collection = server.with_token("DELETE", "/v1/admin/games", TOKEN, "");
    assert_eq!(
        status_code(&delete_of_the_collection),
        Some(405),
        "`DELETE /v1/admin/games` is not a method that route carries: {}",
        status_line(&delete_of_the_collection)
    );
    let allow = header_values(&delete_of_the_collection, "Allow").join(",");
    assert!(
        allow.to_ascii_uppercase().contains("POST"),
        "the `405` names the method the route does carry: {allow:?}"
    );

    let get_of_the_collection = server.get_with_token("/v1/admin/games", TOKEN);
    assert_eq!(status_code(&get_of_the_collection), Some(405));
    assert!(
        !body_text(&get_of_the_collection).contains("steam/900001"),
        "and a `405` is not a back door to the corpus (`026-L3`): {}",
        body_text(&get_of_the_collection)
    );
}

/// proves: `026-L3` and Test strategy #18's admin half — no admin route answers a
/// whole document: `stats` is three counts, and every bulk-shaped path is a `404`
/// even to a correctly authenticated caller.
#[test]
fn no_admin_route_answers_the_whole_document() {
    let (_data, corpus, server) = root("auth-nobulk", &["--admin-token", TOKEN]);
    std::fs::read_to_string(&corpus).expect("the corpus is readable");

    for target in [
        "/v1/admin/catalog",
        "/v1/catalog",
        "/v1/admin/games/all",
        "/v1/export",
    ] {
        let answer = server.get_with_token(target, TOKEN);
        assert_eq!(
            status_code(&answer),
            Some(404),
            "`{target}` is not a route (`026-L3`: no bulk transfer in either mode): {}",
            status_line(&answer)
        );
    }

    let stats = server.get_with_token("/v1/admin/stats", TOKEN);
    let document = json_body(&stats);
    let counts = document.as_object().expect("stats is an object");
    assert_eq!(
        counts.keys().cloned().collect::<Vec<String>>().len(),
        3,
        "`stats` is exactly `{{games, source_keys, revision}}`: {document:?}"
    );
    assert!(
        !document.to_string().contains("steam/900001"),
        "`stats` never lists keys (`026-L3`): {document:?}"
    );
}
