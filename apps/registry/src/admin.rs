//! The admin face: enablement, the bearer check, and the three routes
//! (design 026, Chosen design: §The admin face and auth).
//!
//! Present in both modes, mutating that instance's own catalog document — the
//! `--corpus` file for a root, the local-entries document for a caching instance.
//! Enablement is judged from the address actually bound together with the presence
//! of a configured token, a `401` never echoes the credential, and the comparison
//! runs under a fold that does not return early on a length mismatch.
//!
//! No route here answers a whole document: `stats` is three counts and the only
//! bulk-shaped path, `/v1/admin/games/<anything>`, is a `404` even to a correctly
//! authenticated caller (`026-L3`).
//!
//! Two shapes here are deliberate, and a reader who expects RFC-shape symmetry from
//! `026-C4`'s "`405` names any method the route does not carry" should read them as
//! decisions rather than gaps:
//!
//! - **A parameterized path is a route only for the method it carries.**
//!   `/v1/admin/games/<game_id>` answers `DELETE`; `GET`/`PUT`/`POST` asked of it is
//!   the named `404`, not a `405` with `Allow: DELETE`. On the collection path
//!   (`/v1/admin/games`) the route exists and only the method is wrong, so `405` is
//!   right there; on a parameterized path the *path* is what no route claims for the
//!   method — and a `405` would publish which segments this face parses as `game_id`s
//!   to a caller that has not been admitted.
//! - **A disabled face answers the named `404` flatly**, for every dispatch outcome
//!   including a method the route does not carry: no `Allow` list is published for a
//!   face this instance has just said it does not have, which is the same
//!   never-an-existence-oracle rule `026-C5` states for `401`.
//!
//! The face is also not for browsers. Every method it carries is non-`GET`, so a
//! cross-origin — or DNS-rebounded, which is the same request as far as the browser
//! is concerned — write always carries `Origin`, and this server never answers a
//! preflight; a carried admin request that carries `Origin` is refused the named
//! `400`. And because the write a browser *can* post with no preflight is the one
//! that declares `text/plain`, `application/x-www-form-urlencoded` or
//! `multipart/form-data`, an upsert whose body is declared as anything but
//! `application/json` is refused the named `415` — on the header block, so the body
//! is never read. A caller that sends no `Content-Type` is answered: that header
//! names no obligation in this contract, and no browser form omits it (see
//! `serve::admin_gate`).

use bindforge_core::escape_and_truncate;

use crate::corpus::{game_id_is_valid, record_from_value};
use crate::http::{Request, RequestHead, json_object, refusal_body};
use crate::state::{AdminRefusal, State};

/// The scheme token the compared value follows.
const BEARER: &str = "Bearer ";
/// The collection route: one whole record per `POST`.
const GAMES: &str = "/v1/admin/games";
/// The count route.
const STATS: &str = "/v1/admin/stats";
/// The parameter prefix, whose one segment is a `game_id`.
const GAME_PREFIX: &str = "/v1/admin/games/";

/// An admin route the caller is standing in front of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// `GET /v1/admin/stats`.
    Stats,
    /// `POST /v1/admin/games`.
    Upsert,
    /// `DELETE /v1/admin/games/<game_id>`, with the path segment already
    /// percent-decoded by the request reader.
    Delete { game_id: String },
}

/// What the routing decision for one `/v1/admin/**` path and method is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dispatch {
    /// A route this face carries, ready to be authorized and handled.
    Carried(Route),
    /// A route the contract defines, asked for with a method it does not carry: a
    /// `405` naming the methods it does.
    WrongMethod { allow: &'static str },
    /// Not a route this contract defines: the named `404`, which is what a
    /// bulk-shaped path like `/v1/admin/games/all` gets whatever credential it
    /// arrives with.
    NotARoute,
}

/// Classifies one admin path against one method. A parameterized path is only a
/// route for the method it carries, so `GET /v1/admin/games/<seg>` is not a method
/// on a known route but a path that does not exist — deliberately, as the module doc
/// spells out.
#[must_use]
pub fn dispatch(path: &str, method: &str) -> Dispatch {
    if path == STATS {
        return if method == "GET" {
            Dispatch::Carried(Route::Stats)
        } else {
            Dispatch::WrongMethod { allow: "GET" }
        };
    }
    if path == GAMES {
        return if method == "POST" {
            Dispatch::Carried(Route::Upsert)
        } else {
            Dispatch::WrongMethod { allow: "POST" }
        };
    }
    if let Some(segment) = path.strip_prefix(GAME_PREFIX)
        && !segment.is_empty()
        && !segment.contains('/')
    {
        return if method == "DELETE" {
            Dispatch::Carried(Route::Delete {
                game_id: segment.to_string(),
            })
        } else {
            Dispatch::NotARoute
        };
    }
    Dispatch::NotARoute
}

/// Whether a path belongs to this face at all, which is what decides whether the
/// serving face's `404` or this face's rules answer.
#[must_use]
pub fn is_admin_path(path: &str) -> bool {
    path == "/v1/admin" || path.starts_with("/v1/admin/")
}

/// The configured admin posture: whether the face is enabled at all, and the token
/// a caller has to present.
#[derive(Debug, Clone)]
pub struct Admin {
    token: Option<String>,
    enabled: bool,
}

impl Admin {
    /// Judges enablement from the address actually bound and the presence of a
    /// configured token (`026-C5`, resolving open question 11): loopback-only with
    /// no token is enabled; a routable bind with no token disables the face; a
    /// token, once configured, is required on every admin request regardless of
    /// bind.
    #[must_use]
    pub fn new(bound_loopback: bool, token: Option<String>) -> Self {
        let token = token.filter(|value| !value.is_empty());
        let enabled = match &token {
            Some(_) => true,
            None => bound_loopback,
        };
        Self { token, enabled }
    }

    /// Whether this instance serves the admin face at all.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Whether the request carries the configured credential, or any credential at
    /// all when the face is open on loopback with no token configured. The compared
    /// value is what follows `Bearer `, never the whole header. The decision reads
    /// the [`RequestHead`] alone: it has to be makeable before a body is buffered,
    /// or the request-body budget becomes an unauthenticated peer's allowance.
    #[must_use]
    pub fn admits(&self, head: &RequestHead) -> bool {
        if !self.enabled {
            return false;
        }
        let Some(configured) = self.token.as_ref() else {
            return true;
        };
        let Some(header) = head.header("authorization") else {
            return false;
        };
        let Some(submitted) = header.strip_prefix(BEARER) else {
            return false;
        };
        token_matches(submitted.trim(), configured)
    }
}

/// The constant-time fold: no early return, no branch on which byte differs, and no
/// length comparison that could stand in for one. Time still grows with the longer
/// of the two values, which is the only shape this reveals.
#[must_use]
fn token_matches(submitted: &str, configured: &str) -> bool {
    let submitted = submitted.as_bytes();
    let configured = configured.as_bytes();
    let longest = submitted.len().max(configured.len());
    let mut difference = submitted.len() ^ configured.len();
    for index in 0..longest {
        let left = usize::from(submitted.get(index).copied().unwrap_or(0));
        let right = usize::from(configured.get(index).copied().unwrap_or(0));
        difference |= left ^ right;
    }
    difference == 0
}

/// What an admin route answers: status line and body, both already framed.
pub struct Answer {
    pub status: u16,
    pub reason: &'static str,
    pub body: Vec<u8>,
}

impl Answer {
    fn ok(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            reason: "OK",
            body,
        }
    }

    fn refused(status: u16, detail: impl Into<String>) -> Self {
        let reason = match status {
            404 => "Not Found",
            422 => "Unprocessable Entity",
            500 => "Internal Server Error",
            _ => "Bad Request",
        };
        let token = match status {
            404 => "not_found",
            422 => "unprocessable",
            500 => "internal",
            _ => "bad_request",
        };
        Self {
            status,
            reason,
            body: refusal_body(token, &detail.into()),
        }
    }
}

/// Handles one authorized admin request against the instance's own document.
pub fn handle(route: Route, state: &State, request: &Request) -> Answer {
    match route {
        Route::Stats => Answer::ok(json_object(&state.stats())),
        Route::Upsert => handle_upsert(state, &request.body),
        Route::Delete { game_id } => handle_delete(state, &game_id),
    }
}

fn handle_upsert(state: &State, body: &[u8]) -> Answer {
    let Ok(document) = serde_json::from_slice::<serde_json::Value>(body) else {
        return Answer::refused(400, "the request body is not readable JSON");
    };
    let record = match record_from_value(&document) {
        Ok(record) => record,
        Err(detail) => return Answer::refused(422, detail),
    };
    match state.upsert(record) {
        Ok(()) => Answer::ok(json_object(&state.stats())),
        Err(rejected) => refusal(rejected),
    }
}

fn handle_delete(state: &State, game_id: &str) -> Answer {
    if !game_id_is_valid(game_id) {
        return Answer::refused(
            422,
            format!(
                "the path segment `{}` is not a `game_id` the catalog contract admits",
                escape_and_truncate(game_id)
            ),
        );
    }
    match state.remove(game_id) {
        Ok(()) => Answer::ok(json_object(&state.stats())),
        Err(rejected) => refusal(rejected),
    }
}

/// Maps a state-level refusal onto its status line.
fn refusal(refusal: AdminRefusal) -> Answer {
    match refusal {
        AdminRefusal::Reject { status, detail } => Answer::refused(status, detail),
        AdminRefusal::Absent { detail } => Answer::refused(404, detail),
        AdminRefusal::Persist { detail } => Answer::refused(500, detail),
    }
}

#[cfg(test)]
mod tests {
    //! Enablement, the bearer rule, and the route table — the harness-level versions
    //! are in `tests/admin_auth.rs`.
    use super::{Admin, Dispatch, dispatch, is_admin_path, token_matches};
    use crate::http::RequestHead;

    const TOKEN: &str = "a-token-of-eighteen-chars";

    fn request(headers: &[(&str, &str)]) -> RequestHead {
        RequestHead {
            method: "GET".to_string(),
            path: "/v1/admin/stats".to_string(),
            query: Vec::new(),
            headers: headers
                .iter()
                .map(|(name, value)| ((*name).to_lowercase(), (*value).to_string()))
                .collect(),
        }
    }

    #[test]
    fn a_configured_token_is_required_whatever_the_bind() {
        let open_on_loopback = Admin::new(true, None);
        assert!(open_on_loopback.is_enabled());
        assert!(open_on_loopback.admits(&request(&[])));

        let gated = Admin::new(true, Some(TOKEN.to_string()));
        assert!(gated.admits(&request(&[("authorization", &format!("Bearer {TOKEN}"))])));
        assert!(!gated.admits(&request(&[])), "no credential is refused");
        assert!(
            !gated.admits(&request(&[("authorization", TOKEN)])),
            "the bare token without its scheme is not the compared value"
        );
        assert!(!gated.admits(&request(&[("authorization", "Bearer nope")])));

        let disabled = Admin::new(false, None);
        assert!(!disabled.is_enabled());
        assert!(
            !disabled.admits(&request(&[("authorization", &format!("Bearer {TOKEN}"))])),
            "an unconfigured token cannot unlock a disabled face"
        );
    }

    #[test]
    fn an_empty_configured_token_is_no_token_at_all() {
        let open = Admin::new(true, Some(String::new()));
        assert!(open.is_enabled(), "a blank token is not a configured one");
    }

    #[test]
    fn the_fold_compares_value_not_length() {
        assert!(token_matches(TOKEN, TOKEN));
        assert!(!token_matches(&TOKEN[..TOKEN.len() - 1], TOKEN));
        assert!(!token_matches(&format!("{TOKEN}x"), TOKEN));
        assert!(
            token_matches("", ""),
            "a blank submitted value equals a blank one"
        );
        assert!(!token_matches("x", "y"));
    }

    #[test]
    fn the_route_table_carries_exactly_three_routes() {
        assert_eq!(
            dispatch("/v1/admin/stats", "GET"),
            Dispatch::Carried(super::Route::Stats)
        );
        assert_eq!(
            dispatch("/v1/admin/stats", "PUT"),
            Dispatch::WrongMethod { allow: "GET" }
        );
        assert_eq!(
            dispatch("/v1/admin/games", "POST"),
            Dispatch::Carried(super::Route::Upsert)
        );
        assert_eq!(
            dispatch("/v1/admin/games", "GET"),
            Dispatch::WrongMethod { allow: "POST" }
        );
        assert_eq!(
            dispatch("/v1/admin/games/example-game", "DELETE"),
            Dispatch::Carried(super::Route::Delete {
                game_id: "example-game".to_string()
            })
        );
        for bulk in [
            "/v1/admin/games/all",
            "/v1/admin/catalog",
            "/v1/admin/games/",
        ] {
            assert_eq!(
                dispatch(bulk, "GET"),
                Dispatch::NotARoute,
                "`{bulk}` is a `404`, never a bulk route (`026-L3`)"
            );
        }
        assert!(is_admin_path("/v1/admin/stats"));
        assert!(!is_admin_path("/v1/candidates"));
    }
}
