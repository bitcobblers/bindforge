//! The accept loop and the route table (design 026, Chosen design: §Request
//! handling, §Concurrency).
//!
//! One process, one accept loop, one thread per admitted connection, and each
//! connection answers exactly one request and closes (`A23`) — an idle client can
//! never hold an admitted slot, and the ceiling `MAX_CONCURRENT_CONNECTIONS`
//! states is enforced at accept by closing the surplus connection without writing
//! a single byte, rather than by queueing it.

use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::admin::{self, Admin, Dispatch, Route};
use crate::constants;
use crate::http::{self, Refused, Request, RequestHead, json_object, refusal_body};
use crate::keys::key_is_admissible;
use crate::persist;
use crate::state::{Answer, State};

/// The connection slots, counted so the ceiling is a fact rather than an
/// intention. A guard is held for the life of one connection thread.
#[derive(Debug)]
struct Slots {
    in_use: Arc<AtomicUsize>,
}

impl Slots {
    fn new() -> Self {
        Self {
            in_use: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Claims a slot, or returns `None` when the ceiling is already reached. The
    /// claim and the release are both atomic, so the count cannot drift when
    /// connections arrive and leave at speed.
    fn try_claim(&self) -> Option<Claim> {
        let previous = self.in_use.fetch_add(1, Ordering::SeqCst);
        if previous >= constants::MAX_CONCURRENT_CONNECTIONS {
            self.in_use.fetch_sub(1, Ordering::SeqCst);
            return None;
        }
        Some(Claim {
            in_use: Arc::clone(&self.in_use),
        })
    }
}

/// One claimed slot. Releasing it is a `Drop`, so no path out of a connection
/// thread — including a panic — can leak the count.
#[derive(Debug)]
struct Claim {
    in_use: Arc<AtomicUsize>,
}

impl Drop for Claim {
    fn drop(&mut self) {
        self.in_use.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Accepts until the process is stopped. Every admitted connection gets its own
/// thread; there is no timer thread and no worker pool.
///
/// # Errors
///
/// Returns a listener error only when `accept` fails in a way the loop cannot
/// ride out; a per-connection failure is reported and the loop continues.
pub fn serve(listener: TcpListener, state: Arc<State>, admin: Arc<Admin>) -> std::io::Result<()> {
    let slots = Slots::new();
    for connection in listener.incoming() {
        let stream = match connection {
            Ok(stream) => stream,
            Err(error) => {
                // A per-connection accept error (a descriptor limit, a peer that
                // went away) is reported and ridden out: the instance keeps serving.
                eprintln!(
                    "warning: accepting a connection failed: {}",
                    persist::echo_safe(&error.to_string())
                );
                continue;
            }
        };
        let Some(claim) = slots.try_claim() else {
            // Over the ceiling: close without a status line. The peer learns the
            // instance is busy from the closed connection, which is what the
            // shipped client's own reader expects from an unreachable catalog.
            drop(stream);
            continue;
        };
        let state = Arc::clone(&state);
        let admin = Arc::clone(&admin);
        let builder = std::thread::Builder::new()
            .name("registry-connection".to_string())
            .stack_size(256 * 1024);
        match builder.spawn(move || {
            let _claim = claim;
            serve_one_connection(stream, &state, &admin);
        }) {
            Ok(_) => {}
            Err(error) => {
                // The slot comes back with the guard when this scope exits.
                eprintln!(
                    "warning: no thread for a connection: {}",
                    persist::echo_safe(&error.to_string())
                );
            }
        }
    }
    Ok(())
}

/// Reads the one request a connection may make, answers it, and closes.
///
/// The admin face is gated from the header block alone, before its declared body is
/// read: the credential is a header, so an unauthenticated peer never buys the
/// `MAX_REQUEST_BODY_OCTETS` buffer of §Budgets.
fn serve_one_connection(mut stream: std::net::TcpStream, state: &State, admin: &Admin) {
    let _ = stream.set_read_timeout(Some(http::request_read_timeout()));
    let _ = stream.set_write_timeout(Some(http::response_write_timeout()));
    match http::read_request_head(&mut stream) {
        Ok(None) => {}
        Ok(Some(head)) => {
            if admin::is_admin_path(&head.path) {
                admin_connection(&mut stream, state, admin, head);
            } else {
                match head.into_request(&mut stream) {
                    Ok(request) => route(&mut stream, state, &request),
                    Err(refused) => {
                        let _ = http::write_refused(&mut stream, &refused);
                    }
                }
            }
        }
        Err(refused) => {
            let _ = http::write_refused(&mut stream, &refused);
        }
    }
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

/// The route table. `/v1/admin/**` never reaches here: this face's own rules,
/// including the flat `404` a disabled admin face answers, run in
/// [`admin_connection`] ahead of any body read. Everything else is `026-C2`.
fn route(stream: &mut std::net::TcpStream, state: &State, request: &Request) {
    let answer = match request.path() {
        "/v1/candidates" => {
            if request.method() == "GET" {
                candidates_answer(state, request)
            } else {
                refused(Refused::method_not_allowed(
                    "`/v1/candidates` is a `GET` route",
                    "GET",
                ))
            }
        }
        "/v1/health" => {
            if request.method() == "GET" {
                Sent::Json(json_object(&state.health()))
            } else {
                refused(Refused::method_not_allowed(
                    "`/v1/health` is a `GET` route",
                    "GET",
                ))
            }
        }
        _ => refused(Refused::not_found(
            "no route this registry defines answers that path",
        )),
    };
    write(stream, answer);
}

/// One `/v1/admin/**` request: gate first from the headers, and only then read the
/// record a carried request is offering.
fn admin_connection(
    stream: &mut std::net::TcpStream,
    state: &State,
    admin: &Admin,
    head: RequestHead,
) {
    let route = match admin_gate(admin, &head) {
        Ok(route) => route,
        Err(refused) => {
            let _ = http::write_refused(stream, &refused);
            http::drain_unread_body(stream, head.declared_body_octets());
            return;
        }
    };
    let request = match head.into_request(stream) {
        Ok(request) => request,
        Err(refused) => {
            let _ = http::write_refused(stream, &refused);
            return;
        }
    };
    let handled = admin::handle(route, state, &request);
    write(
        stream,
        Sent::Refusal {
            status: handled.status,
            reason: handled.reason,
            body: handled.body,
            allow: None,
        },
    );
}

/// The admin decision the header block alone answers: routing, enablement, the
/// bearer credential, and the two browser-shape refusals. `Err` is the refusal this
/// face owes; `Ok` names the carried route the body is for.
fn admin_gate(admin: &Admin, head: &RequestHead) -> Result<Route, Refused> {
    if !admin.is_enabled() {
        // A disabled face is a flat `404` for *every* dispatch outcome, including a
        // method the route does not carry (`026-C5`): a `405` with its `Allow` list
        // would confirm which methods the face this instance says it does not have
        // really does answer, which is the existence oracle the rule forbids. The
        // `405`/`Allow` behavior stays for an enabled face.
        return Err(Refused::not_found(
            "this instance's admin face is not enabled",
        ));
    }
    match admin::dispatch(&head.path, &head.method) {
        Dispatch::NotARoute => Err(Refused::not_found(
            "no admin route this registry defines answers that path",
        )),
        Dispatch::WrongMethod { allow } => Err(Refused::method_not_allowed(
            "that admin route does not carry this method",
            allow,
        )),
        Dispatch::Carried(route) => {
            if !admin.admits(head) {
                return Err(Refused::unauthorized(
                    "a bearer token is required for the admin face",
                ));
            }
            if head.header("origin").is_some() {
                // The face is for an operator and their tools, not for a page in a
                // browser. Every method this face carries is non-`GET`, so a
                // cross-origin — or DNS-rebound, which is the same request from the
                // browser's point of view — write names the origin it came from, and
                // this server never answers a preflight; the named `400` is the whole
                // of that posture.
                return Err(Refused::bad_request(
                    "the admin face does not answer a request that carries `Origin`",
                ));
            }
            if matches!(route, Route::Upsert)
                && let Some(media_type) = head.header("content-type")
                && !media_type_spells_json(media_type)
            {
                // The one body this face parses is a whole `026-C3` record. The write a
                // browser can post cross-origin *without* a preflight is exactly the one
                // that declares `text/plain`, `application/x-www-form-urlencoded` or
                // `multipart/form-data` — the three spellings an HTML form produces — and
                // this face parses none of them. So a declaration that names another type
                // is refused on the header block, and the record body is never read.
                // A caller that sends no `Content-Type` at all is answered: the contract
                // names no such header, so omitting a header this server does not need
                // stays legal, and no browser form can omit one.
                return Err(Refused::unsupported_media_type(
                    "the admin face reads a request body declared as `application/json`",
                ));
            }
            Ok(route)
        }
    }
}

/// Whether a declared `Content-Type` names the one media type the admin face parses.
/// The parameters after the `;` (`charset`, `boundary`) qualify the sender's payload
/// rather than name a different type, so the token before them is the whole test.
fn media_type_spells_json(media_type: &str) -> bool {
    media_type
        .split(';')
        .next()
        .is_some_and(|base| base.trim().eq_ignore_ascii_case("application/json"))
}

/// `GET /v1/candidates?slug=`: decode-then-validate, then the mode's priority
/// order. An inadmissible key is the `404` `026-C2` names, and its spelling is
/// never repeated on the wire.
///
/// The body is built by [`crate::corpus::candidates_body`], which trims the stable
/// window to A1's `MAX_RESPONSE_OCTETS`: the count cap alone can emit a body the
/// shipped client refuses, and a face that does that has stopped serving the face
/// `026-C1` pins byte-identical.
fn candidates_answer(state: &State, request: &Request) -> Sent {
    let slug = request.query_param("slug").unwrap_or_default();
    if slug.is_empty() || !key_is_admissible(slug) {
        return refused(Refused::not_found(
            "`slug` is missing, empty, or not a lookup key this registry answers",
        ));
    }
    match state.candidates(slug, crate::now_unix()) {
        Answer::Candidates { candidates, .. } => {
            // Both of A1's caps are applied here, on the stable window: the count
            // cap the index already enforced and the octet cap only this seam can
            // see. `query` prints the same projection it answers with, so no second
            // copy of the arithmetic exists.
            Sent::Json(crate::corpus::candidates_body(&candidates))
        }
        // ADR-027's named `503`s: the reason phrase carries the name, because the
        // shipped CLI shows the user nothing but the status line (A2), and the body
        // is a named refusal rather than the empty list the policy must never
        // degrade into. The empty list it *does* answer is the other arm.
        Answer::Refusal(named) => Sent::Refusal {
            status: 503,
            reason: named.reason(),
            body: refusal_body(named.code(), &named.detail()),
            allow: None,
        },
    }
}

/// What the router decided to answer: a `200` JSON body, or a status line with a
/// refusal body already attached and the `Allow` list a `405` names. This is the
/// router's own shape; the policy's `crate::state::Answer` is narrower.
enum Sent {
    Json(Vec<u8>),
    Refusal {
        status: u16,
        reason: &'static str,
        body: Vec<u8>,
        allow: Option<&'static str>,
    },
}

fn refused(refused: Refused) -> Sent {
    Sent::Refusal {
        status: refused.status,
        reason: refused.reason,
        body: refusal_body(error_token(refused.status), &refused.detail),
        allow: refused.allow,
    }
}

/// The `error` token for a status, shared with the reader's own refusals so a
/// refusal body looks the same whoever produced it.
fn error_token(status: u16) -> &'static str {
    match status {
        401 => "unauthorized",
        404 => "not_found",
        405 => "method_not_allowed",
        415 => "unsupported_media_type",
        422 => "unprocessable",
        503 => "unavailable",
        _ => "bad_request",
    }
}

fn write(stream: &mut std::net::TcpStream, answer: Sent) {
    let outcome = match answer {
        Sent::Json(body) => http::write_response(stream, 200, "OK", &body, &[]),
        Sent::Refusal {
            status,
            reason,
            body,
            allow,
        } => {
            let mut extra = Vec::new();
            if let Some(allow) = allow {
                extra.push(format!("Allow: {allow}"));
            }
            if status == 401 {
                extra.push("WWW-Authenticate: Bearer".to_string());
            }
            http::write_response(stream, status, reason, &body, &extra)
        }
    };
    if let Err(error) = outcome {
        // A peer that hung up mid-answer is not an incident; the answer was made.
        eprintln!(
            "warning: writing the answer failed: {}",
            persist::echo_safe(&error.to_string())
        );
    }
}

#[cfg(test)]
mod tests {
    //! The route table and the slot ceiling, in-process; the wire-level versions are
    //! in `tests/response_shape.rs` and `tests/persist_and_concurrency.rs`.
    use super::Slots;
    use super::admin_gate;
    use super::error_token;
    use crate::admin::{Admin, Route};
    use crate::constants;
    use crate::http::RequestHead;
    use std::sync::Arc;
    use std::sync::Barrier;

    fn head(method: &str, path: &str, headers: &[(&str, &str)]) -> RequestHead {
        RequestHead {
            method: method.to_string(),
            path: path.to_string(),
            query: Vec::new(),
            headers: headers
                .iter()
                .map(|(name, value)| (name.to_ascii_lowercase(), value.to_string()))
                .collect(),
        }
    }

    #[test]
    fn a_disabled_face_refuses_every_dispatch_shape_with_the_same_404() {
        // `026-C5`: the disabled case is flatly a `404`. A `405` with its `Allow`
        // list would be an existence oracle for the face the instance says it lacks.
        let disabled = Admin::new(false, None);
        for (method, path) in [
            ("GET", "/v1/admin/stats"),
            ("PUT", "/v1/admin/stats"),
            ("POST", "/v1/admin/games"),
            ("GET", "/v1/admin/games"),
            ("DELETE", "/v1/admin/games/example-game"),
            ("GET", "/v1/admin/catalog"),
        ] {
            let refused = admin_gate(&disabled, &head(method, path, &[]))
                .expect_err("a disabled face carries nothing");
            assert_eq!(
                refused.status, 404,
                "`{method} {path}` on a disabled face is the named `404`"
            );
            assert!(
                refused.allow.is_none(),
                "no `Allow` list leaks from a disabled face: {method} {path}"
            );
        }
    }

    #[test]
    fn an_enabled_face_keeps_the_allow_list_the_bearer_rule_and_the_route_table() {
        let gated = Admin::new(true, Some("a-token".to_string()));
        let admitted = [("authorization", "Bearer a-token")];
        let wrong_method = admin_gate(&gated, &head("PUT", "/v1/admin/stats", &admitted))
            .expect_err("a method the route does not carry is refused");
        assert_eq!(wrong_method.status, 405);
        assert_eq!(
            wrong_method.allow,
            Some("GET"),
            "an enabled face still names the methods the route carries"
        );
        let no_credential = admin_gate(&gated, &head("GET", "/v1/admin/stats", &[]))
            .expect_err("a configured token is required everywhere");
        assert_eq!(no_credential.status, 401);
        assert!(matches!(
            admin_gate(&gated, &head("GET", "/v1/admin/stats", &admitted)),
            Ok(Route::Stats)
        ));
        // A parameterized path is a route only for `DELETE`: the other methods are
        // the named `404`, by the rule the module doc states.
        let other_method = admin_gate(
            &gated,
            &head("PUT", "/v1/admin/games/example-game", &admitted),
        )
        .expect_err("no route claims that path for that method");
        assert_eq!(other_method.status, 404);
    }

    #[test]
    fn a_request_that_names_an_origin_is_not_a_request_the_admin_face_answers() {
        // Every method this face carries is non-`GET`, so a cross-origin — or
        // DNS-rebounded — write always carries `Origin`, and no preflight is ever
        // answered here: the named `400` closes the browser path.
        let open = Admin::new(true, None);
        let refused = admin_gate(
            &open,
            &head(
                "POST",
                "/v1/admin/games",
                &[("origin", "http://evil.example")],
            ),
        )
        .expect_err("a browser-shaped write is refused");
        assert_eq!(refused.status, 400);
        assert!(refused.detail.contains("Origin"), "{}", refused.detail);
        assert!(
            admin_gate(
                &open,
                &head(
                    "POST",
                    "/v1/admin/games",
                    &[("content-type", "application/json")]
                )
            )
            .is_ok(),
            "a tool that sends no `Origin` is answered"
        );
    }

    #[test]
    fn an_upsert_body_declared_as_any_other_type_is_refused_on_the_head() {
        // The three types an HTML form produces are the requests a browser can send
        // cross-origin with no preflight, and this face parses a `026-C3` record and
        // nothing else. The header stays optional: it names no obligation on a tool
        // that sends none, and no browser form can omit it.
        let open = Admin::new(true, None);
        for declared in [
            "text/plain",
            "application/x-www-form-urlencoded",
            "multipart/form-data; boundary=-abc",
            "application/xml",
            "",
        ] {
            let refused = admin_gate(
                &open,
                &head("POST", "/v1/admin/games", &[("content-type", declared)]),
            )
            .expect_err("a form-shaped body is not the record this face reads");
            assert_eq!(refused.status, 415, "`{declared}` is refused: {declared}");
            assert_eq!(
                error_token(refused.status),
                "unsupported_media_type",
                "the refusal body names itself"
            );
        }
        for admitted in [
            "application/json",
            "application/json; charset=utf-8",
            "APPLICATION/JSON",
        ] {
            assert!(
                matches!(
                    admin_gate(
                        &open,
                        &head("POST", "/v1/admin/games", &[("content-type", admitted)])
                    ),
                    Ok(Route::Upsert)
                ),
                "`{admitted}` names the type this face reads"
            );
        }
        assert!(
            matches!(
                admin_gate(&open, &head("POST", "/v1/admin/games", &[])),
                Ok(Route::Upsert)
            ),
            "a caller that sends no `Content-Type` is answered"
        );
        assert!(
            matches!(
                admin_gate(
                    &open,
                    &head("GET", "/v1/admin/stats", &[("content-type", "text/plain")])
                ),
                Ok(Route::Stats)
            ),
            "a route that parses no body asks nothing about one"
        );
    }

    #[test]
    fn the_slot_ceiling_is_the_ceiling() {
        let slots = Slots::new();
        let mut held = Vec::new();
        for _ in 0..constants::MAX_CONCURRENT_CONNECTIONS {
            held.push(
                slots
                    .try_claim()
                    .expect("a slot inside the ceiling is granted"),
            );
        }
        assert!(
            slots.try_claim().is_none(),
            "one connection over the ceiling is refused at accept"
        );
        drop(held.pop());
        assert!(
            slots.try_claim().is_some(),
            "a released slot is available again, so the count cannot drift"
        );
    }

    #[test]
    fn concurrent_claims_never_overshoot_the_ceiling() {
        let slots = Arc::new(Slots::new());
        let workers = constants::MAX_CONCURRENT_CONNECTIONS * 2;
        let start = Arc::new(Barrier::new(workers));
        // Each claim is *held* until the test ends, so the count is the number of
        // slots held at once — which is what the ceiling is a ceiling on.
        let held = Arc::new(std::sync::Mutex::new(Vec::new()));
        let granted = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut threads = Vec::new();
        for _ in 0..workers {
            let slots = Arc::clone(&slots);
            let start = Arc::clone(&start);
            let granted = Arc::clone(&granted);
            let held = Arc::clone(&held);
            threads.push(std::thread::spawn(move || {
                start.wait();
                let Some(claim) = slots.try_claim() else {
                    return;
                };
                granted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                held.lock()
                    .expect("no panic while a claim is held")
                    .push(claim);
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(
            granted.load(std::sync::atomic::Ordering::SeqCst),
            constants::MAX_CONCURRENT_CONNECTIONS,
            "exactly the ceiling's worth of connections is admitted, and the rest is refused"
        );
    }
}
