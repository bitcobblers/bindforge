//! The byte shape of the serving face: the framing the shipped client pins (A1),
//! the `404`s and `405`s `026-C2`/`026-C4` name, and the empty answer's plain status
//! line.
//!
//! proves: design 026's Test strategy #16's shape clause ("response shape pinned
//! byte-exact to A1 (single decimal `Content-Length`, no `Transfer-Encoding`,
//! caps)") and #2/#3's status-code clauses, read against A1 ("accepts only status
//! token `200`, rejects `Transfer-Encoding`, rejects a duplicated or non-decimal
//! `Content-Length`, caps the header block *and* body at `MAX_RESPONSE_OCTETS =
//! 65_536`, refuses more than `MAX_CANDIDATES = 256` candidates, and `read_exact`s
//! exactly `Content-Length` octets") and A2 ("on a non-`200` the client surfaces
//! only the status line"), plus A23 (one answer per connection, echoing the
//! client's own `Connection: close`).
//!
//! # What ADR-027 changed in this file
//!
//! A8 is the reason this suite exists unchanged: the hosted answer is byte-exact to the
//! shipped candidate contract, so the framing, the `404`/`405` pins and the one-answer
//! rule are the shipped pins. One leg carried the caching mode's cold `503` as the
//! status a root instance must never reach; that `503` retired with the mode (A15), and
//! the leg now pins the half that stands — the empty answer is a `200 []` body, asked of
//! a `local/…` key so it is the join's verdict and not the resolver's. The `503`s hosted
//! mode does answer are `027-C5`'s named resolver refusals, which
//! `hosted_lookup_policy.rs` pins by text.

mod common;

use common::{
    Registry, TempDir, body_of, corpus_doc, header_names, header_values, status_code, status_line,
};

const SHARED: [(&str, &str, &[&str]); 2] = [
    ("example-game", "Example Game", &["steam/900001"]),
    ("other-game", "Other Game", &["epic/77"]),
];

fn root(label: &str) -> (TempDir, std::path::PathBuf, Registry) {
    let data = TempDir::new(label);
    let corpus = data.write("corpus.json", corpus_doc(0, &SHARED).as_bytes());
    let server = Registry::root(label, &corpus, &[]);
    (data, corpus, server)
}

/// proves: A1 / `026-C2` (`026`'s Test strategy #16 shape clause) — a successful
/// answer carries exactly one `Content-Length`, spelled in decimal, naming the
/// exact body length the client will `read_exact`.
#[test]
fn an_answer_carries_exactly_one_decimal_content_length_naming_the_body() {
    let (_data, _corpus, server) = root("shape-length");

    let answer = server.get("/v1/candidates?slug=steam%2F900001");
    let lengths = header_values(&answer, "Content-Length");
    assert_eq!(
        lengths.len(),
        1,
        "exactly one `Content-Length` — the shipped client refuses a duplicated one: {:?}",
        lengths
    );
    let claimed = lengths[0].parse::<u64>().unwrap_or_else(|err| {
        panic!(
            "`Content-Length: {}` is not a decimal number the client can read_exact ({err})",
            lengths[0]
        )
    });
    assert_eq!(
        claimed as usize,
        body_of(&answer).len(),
        "the promised length is the body the client reads"
    );
    assert!(
        header_names(&answer)
            .iter()
            .any(|name| name == "content-length"),
        "the header is present at all: {:?}",
        header_names(&answer)
    );
    assert_eq!(
        status_line(&answer),
        "HTTP/1.1 200 OK",
        "the status token is `200`"
    );
}

/// proves: A1 / RFC 9112 (`026-C2`) — the server never emits `Transfer-Encoding`,
/// in either casing: the shipped client refuses it, so a registry that sent it
/// would be unusable by its own client.
#[test]
fn no_answer_ever_carries_transfer_encoding() {
    let (_data, _corpus, server) = root("shape-te");

    for answer in [
        server.get("/v1/candidates?slug=steam%2F900001"),
        server.get("/v1/candidates?slug=steam%2F999999"),
        server.get("/v1/health"),
        server.get("/v1/nope"),
        server.request("POST", "/v1/candidates", &[], ""),
    ] {
        let names = header_names(&answer);
        assert!(
            !names.iter().any(|name| name == "transfer-encoding"),
            "no route answers with `Transfer-Encoding`: {:?} in {}",
            names,
            status_line(&answer)
        );
    }
}

/// proves: A23 (`http.rs` closes the connection after one answer, echoing the
/// shipped client's `Connection: close`) — a second request on the same
/// connection is never answered, so no idle client holds an admitted slot.
#[test]
fn one_connection_answers_exactly_one_request_and_closes() {
    use std::io::{Read, Write};

    let (_data, _corpus, server) = root("shape-once");
    let mut connection = server.connection();
    connection
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();

    connection
        .write_all(b"GET /v1/health HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .expect("the first request is written");
    let mut first = Vec::new();
    let mut window = [0_u8; 4096];
    loop {
        match connection.read(&mut window) {
            Ok(0) => break,
            Ok(read) => first.extend_from_slice(&window[..read]),
            Err(err) => panic!("the first answer ends cleanly, not on {err}"),
        }
    }
    assert_eq!(status_code(&first), Some(200), "the first answer arrives");

    let again = connection.write_all(b"GET /v1/health HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    if again.is_ok() {
        let mut rest = Vec::new();
        let _ = connection.read_to_end(&mut rest);
        assert!(
            !status_line(&rest).starts_with("HTTP/"),
            "the connection answers one request and closes; it never serves a second: {}",
            status_line(&rest)
        );
    }
}

/// proves: `026-C2` ("any other route … is a `404`") and #18's no-bulk surface —
/// no invented bulk or listing route answers.
#[test]
fn every_route_the_contract_does_not_define_is_a_404() {
    let (_data, _corpus, server) = root("shape-404");

    for target in [
        "/v1/catalog",
        "/v1/admin/catalog",
        "/v1/games",
        "/v1/candidates/all",
        "/health",
        "/v1/health/extras",
    ] {
        let answer = server.get(target);
        assert_eq!(
            status_code(&answer),
            Some(404),
            "`{target}` is not a route `026-C2` defines (`026-L3` forbids the bulk \
             ones): {}",
            status_line(&answer)
        );
    }
}

/// proves: `026-C4`'s "a `405` names any method the route does not carry" (and
/// A21's `Allow` requirement, stated as a design choice) — the method list is the
/// pinned thing, so these assertions read the header's *values*.
#[test]
fn a_method_a_route_does_not_carry_is_405_naming_the_methods_it_does() {
    let (_data, _corpus, server) = root("shape-405");

    for (method, target, must_name) in [
        ("POST", "/v1/candidates?slug=steam%2F900001", "GET"),
        ("DELETE", "/v1/health", "GET"),
        ("POST", "/v1/health", "GET"),
        ("GET", "/v1/admin/games", "POST"),
        ("PUT", "/v1/admin/stats", "GET"),
    ] {
        let answer = server.request(method, target, &[], "");
        assert_eq!(
            status_code(&answer),
            Some(405),
            "`{method} {target}` is a method the route does not carry: {}",
            status_line(&answer)
        );
        let allow = header_values(&answer, "Allow").join(",");
        assert!(
            allow.to_ascii_uppercase().contains(must_name),
            "the `405` names the methods the route does carry (it must mention \
             `{must_name}`): {allow:?}"
        );
    }
}

/// proves: A2's "`503` is never an empty array" boundary as hosted mode serves it —
/// the *cold* `503` this leg used to keep out of root mode retired with the caching
/// mode that wrote it (A15), and the empty answer is still a plain `200 []`. It is
/// asked of a key that is never a resolver candidate, so the answer is the join's own;
/// the `503`s hosted mode does answer are the resolver's (`steam resolver inactive`,
/// an `epic/…` key outside the corpus), which `hosted_lookup_policy.rs` pins.
#[test]
fn a_hosted_empty_answer_is_200_and_never_a_503() {
    let (_data, _corpus, server) = root("shape-no-503");

    let answer = server.get("/v1/candidates?slug=local%2F999999");
    assert_eq!(status_code(&answer), Some(200));
    assert_eq!(
        body_of(&answer),
        b"[]".as_slice(),
        "the empty answer is the body the shipped client reads, not a status"
    );
}

/// proves: Test strategy #16's shape clause for the error surfaces too — a `404`
/// still carries exactly one decimal `Content-Length` and no
/// `Transfer-Encoding`, so the shipped client's reader never trips on framing.
#[test]
fn every_answer_is_well_framed_whatever_its_status() {
    let (_data, _corpus, server) = root("shape-framing");

    for answer in [
        server.get("/v1/candidates?slug=steam%2F900001"),
        server.get("/v1/candidates?slug=notakey"),
        server.get("/v1/health"),
        server.get("/v1/admin/stats"),
        server.request("POST", "/v1/candidates", &[], ""),
        server.get("/v1/no-such-route"),
    ] {
        let code = status_code(&answer).unwrap_or_else(|| panic!("a status line: {answer:?}"));
        let lengths = header_values(&answer, "Content-Length");
        assert_eq!(
            lengths.len(),
            1,
            "status {code} answers with exactly one `Content-Length` (A1's reader \
             refuses zero or two): {lengths:?}"
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
            "status {code} stays inside `MAX_RESPONSE_OCTETS`"
        );
    }
}
