//! The registry's own upstream client (design 026, Chosen design: §Caching mode).
//!
//! The response discipline is the shipped discovery client's (A1), copied so the
//! two ends of a registry-to-registry call speak the same pinned subset: only the
//! `200` status token is accepted, a `3xx` is an upstream error rather than
//! something to chase (`026-L2`'s disclosure model lives at the configured
//! endpoint), `Transfer-Encoding` is refused, `Content-Length` must appear once
//! and in decimal, the header block and the body are each capped at
//! [`constants::MAX_RESPONSE_OCTETS`], more than [`constants::MAX_CANDIDATES`]
//! candidates is a refusal, and the body is `read_exact` to the promised length.
//!
//! Everything outbound is built from the *decoded* slug through
//! [`crate::keys::percent_encode`] (A19), so a key is escaped exactly once. The
//! configured admin token is never sent here: this is the serving face talking to
//! another serving face (`026-C5`).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use bindforge_core::{MAX_NATIVE_NAME_OCTETS, escape_and_truncate};

use crate::constants;
use crate::corpus::{Candidate, game_id_is_valid};
use crate::endpoint::authority_of;
use crate::keys::percent_encode;

/// An upstream answer this client will not act on: what it asked for, and the
/// reason in one line. The reason is what moves `health.upstream`, never what it
/// was told to print.
#[derive(Debug, Clone)]
pub struct UpstreamError(pub String);

/// One configured upstream origin, already through the endpoint gate.
#[derive(Debug, Clone)]
pub struct Upstream {
    base: String,
}

impl Upstream {
    /// Wraps a normalized `http://<host>:<port>` base from
    /// [`crate::endpoint::parse_endpoint`]. Building one contacts nothing: the
    /// gate is a parse, and the dial belongs to the lookup
    /// (Test strategy #14's "admits without dialling").
    #[must_use]
    pub fn new(base: String) -> Self {
        Self { base }
    }

    /// The answer to one slug, as the upstream gave it: at most
    /// [`constants::MAX_CANDIDATES`] `{game_id, name}` pairs in the response's
    /// order, each one already through the ingest validation of §Caching mode.
    pub fn candidates(&self, slug: &str) -> Result<Vec<Candidate>, UpstreamError> {
        let target = format!("/v1/candidates?slug={}", percent_encode(slug));
        let body = self.fetch(&target)?;
        parse_candidates(&body)
    }

    /// Whether the upstream's own `/v1/health` answers, which is the gauge
    /// `026-C6` names. A refusal of any kind — transport, framing, status — is
    /// simply "not reachable"; the detail stays in the returned verdict.
    #[must_use]
    pub fn answers_health(&self) -> bool {
        match self.fetch("/v1/health") {
            Err(_) => false,
            Ok(body) => serde_json::from_slice::<serde_json::Value>(&body).is_ok(),
        }
    }

    /// One `GET` against the configured origin, answered under the copied
    /// discipline. Never sends a credential, and never follows a `Location`.
    fn fetch(&self, target: &str) -> Result<Vec<u8>, UpstreamError> {
        let (host, port) = authority_of(&self.base);
        let authority = format!("{host}:{port}");
        let address = (host.as_str(), port)
            .to_socket_addrs()
            .map_err(|error| {
                UpstreamError(format!(
                    "`{}` cannot be resolved: {error}",
                    escape_and_truncate(&authority)
                ))
            })?
            .next()
            .ok_or_else(|| {
                UpstreamError(format!(
                    "`{}` resolved to no address",
                    escape_and_truncate(&authority)
                ))
            })?;
        let mut stream = TcpStream::connect_timeout(&address, constants::UPSTREAM_CONNECT_TIMEOUT)
            .map_err(|error| {
                UpstreamError(format!(
                    "connecting to `{}` failed: {error}",
                    escape_and_truncate(&authority)
                ))
            })?;
        stream
            .set_read_timeout(Some(constants::UPSTREAM_READ_TIMEOUT))
            .and_then(|()| stream.set_write_timeout(Some(constants::UPSTREAM_READ_TIMEOUT)))
            .map_err(|error| {
                UpstreamError(format!("setting the socket timeouts failed: {error}"))
            })?;
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {authority}\r\nAccept: application/json\r\n\
             User-Agent: bindforge-registry/1\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| UpstreamError(format!("sending the request failed: {error}")))?;
        stream
            .flush()
            .map_err(|error| UpstreamError(format!("sending the request failed: {error}")))?;
        read_response(&mut stream)
    }
}

/// Reads one response under the copied discipline (A1) and returns its body.
fn read_response(reader: &mut impl Read) -> Result<Vec<u8>, UpstreamError> {
    let refused = |reason: String| UpstreamError(reason);
    let mut head: Vec<u8> = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let read = reader
            .read(&mut byte)
            .map_err(|error| refused(format!("reading the response: {error}")))?;
        if read == 0 {
            return Err(refused(
                "the connection closed before the header block ended".to_string(),
            ));
        }
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
        if head.len() > constants::MAX_RESPONSE_OCTETS {
            return Err(refused(format!(
                "the header block is over the {}-octet budget",
                constants::MAX_RESPONSE_OCTETS
            )));
        }
    }

    let text = String::from_utf8_lossy(&head[..head.len() - 4]).to_string();
    let mut lines = text.split("\r\n");
    let status = lines.next().unwrap_or_default();
    let code = status.split_whitespace().nth(1).unwrap_or_default();
    if code != "200" {
        // A `3xx` lands here too: the redirect is refused rather than followed,
        // and the `Location` value is not repeated back.
        return Err(refused(format!(
            "the upstream answered `{}`",
            escape_and_truncate(status)
        )));
    }

    let mut content_length: Option<usize> = None;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(refused(format!(
                "the header line `{}` is not understood",
                escape_and_truncate(line)
            )));
        };
        let name = name.trim();
        let value = value.trim();
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(refused(
                "`Transfer-Encoding` is not a body framing this client understands".to_string(),
            ));
        }
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err(refused(
                    "`Content-Length` appears more than once".to_string(),
                ));
            }
            if value.is_empty()
                || value.len() > 5
                || !value.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(refused(format!(
                    "`Content-Length: {}` is not a decimal length",
                    escape_and_truncate(value)
                )));
            }
            let length: usize = value.parse().map_err(|_| {
                refused(format!(
                    "`Content-Length: {value}` is out of range for this reader"
                ))
            })?;
            if length > constants::MAX_RESPONSE_OCTETS {
                return Err(refused(format!(
                    "the body promises {length} octets over the {}-octet budget",
                    constants::MAX_RESPONSE_OCTETS
                )));
            }
            content_length = Some(length);
        }
    }
    let length = content_length
        .ok_or_else(|| refused("the response carries no `Content-Length`".to_string()))?;

    let mut body = vec![0u8; length];
    reader
        .read_exact(&mut body)
        .map_err(|error| refused(format!("the body ended before its length: {error}")))?;
    Ok(body)
}

/// Parses a `200` body into candidates, keeping the response's order, and drops —
/// with one escaped single-line echo each — the records that fail the ingest
/// validation §Caching mode puts at the cache's write seam (A4, A5).
fn parse_candidates(body: &[u8]) -> Result<Vec<Candidate>, UpstreamError> {
    let refused = |reason: String| UpstreamError(reason);
    let document: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| refused("the body is not JSON".to_string()))?;
    let items = document
        .as_array()
        .ok_or_else(|| refused("the body is not a JSON array of candidates".to_string()))?;
    if items.len() > constants::MAX_CANDIDATES {
        return Err(refused(format!(
            "the answer carries {} candidates over the {} limit",
            items.len(),
            constants::MAX_CANDIDATES
        )));
    }
    let mut candidates: Vec<Candidate> = Vec::with_capacity(items.len());
    for item in items {
        let Some(fields) = item.as_object() else {
            echo_drop("(not an object)", "the candidate is not a JSON object");
            continue;
        };
        let game_id = fields
            .get("game_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let name = fields
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        if !game_id_is_valid(game_id) {
            echo_drop(game_id, "`game_id` is outside the grammar `013-B4` admits");
            continue;
        }
        if name.is_empty() || name.len() > MAX_NATIVE_NAME_OCTETS {
            echo_drop(
                game_id,
                &format!("`name` is outside the 1-{MAX_NATIVE_NAME_OCTETS}-octet bound"),
            );
            continue;
        }
        candidates.push(Candidate {
            game_id: game_id.to_string(),
            name: name.to_string(),
        });
    }
    Ok(candidates)
}

/// The one-line echo of a dropped upstream record. The values pass through
/// [`echo_safe`], so a hostile upstream cannot forge extra stderr lines, smuggle a
/// control byte onto the operator's terminal, or push a warning line outside
/// printable ASCII.
fn echo_drop(game_id: &str, reason: &str) {
    eprintln!(
        "warning: dropped an upstream candidate: {}: {}",
        echo_safe(game_id),
        echo_safe(reason)
    );
}

/// The stderr value filter every warning passes through: exactly one line of
/// printable ASCII. Control bytes become `\uXXXX` and long values are truncated
/// the way the error discipline truncates them (`escape_and_truncate`); common
/// typographic punctuation — the prose in this crate's own refusal text — folds to
/// its ASCII spelling, and anything else outside ASCII is escaped rather than
/// written. One hostile upstream, or one badly-worded cache document, therefore
/// cannot forge an extra stderr line or smuggle a non-ASCII byte into a log that
/// an operator's tooling reads as ASCII.
#[must_use]
pub(crate) fn echo_safe(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in escape_and_truncate(value).chars() {
        match c {
            '—' | '–' | '−' => out.push('-'),
            '…' => out.push_str("..."),
            '‘' | '’' => out.push('\''),
            '“' | '”' => out.push('"'),
            c if c.is_ascii() => out.push(c),
            other => out.push_str(&format!("\\u{:04x}", other as u32)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    //! The client's own rules, measured against a fake origin in this process —
    //! the harness-level versions are in `tests/upstream_client.rs`.
    use super::{Upstream, parse_candidates, read_response};
    use crate::constants;
    use std::io::{Read, Write};

    /// Answers every request on one connection with `raw`, then closes.
    fn origin_that_sends(raw: Vec<u8>) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
        std::thread::spawn(move || {
            for _ in 0..8 {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                // Read the request's header block, not to the end of the
                // connection: the client is waiting for the answer, not closing.
                let mut seen: Vec<u8> = Vec::new();
                let mut byte = [0u8; 1];
                while !seen.ends_with(b"\r\n\r\n") {
                    if stream.read(&mut byte).unwrap_or(0) == 0 {
                        break;
                    }
                    seen.push(byte[0]);
                }
                let _ = stream.write_all(&raw);
                let _ = stream.flush();
                drop(stream);
            }
        });
        base
    }

    fn framed(body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }

    #[test]
    fn every_echo_is_one_printable_ascii_line() {
        // A hostile upstream, or our own prose, cannot forge a second stderr
        // line, smuggle a control byte, or step outside printable ASCII.
        for value in [
            "ok-game",
            "../x\nfake: warning line",
            "\u{1b}][0mred\u{1b}[0m",
            "h\u{e9}llo \u{2014} w\u{f6}rld \u{2013} \u{2026}",
        ] {
            let echoed = super::echo_safe(value);
            assert!(echoed.is_ascii(), "`{echoed}` left printable ASCII");
            assert!(
                !echoed.contains('\n') && !echoed.contains('\r'),
                "`{echoed}` is not one line"
            );
            assert!(!echoed.contains('\u{1b}'), "a control byte survived");
        }
        assert_eq!(
            super::echo_safe("a \u{2014} b \u{2013} c \u{2026} d \u{e9}"),
            "a - b - c ... d \\u00e9",
            "our own typography folds to ASCII and anything else is escaped"
        );
    }

    #[test]
    fn an_unusable_answer_is_an_upstream_error() {
        // The framing cases are unusable *answers* and an unusable *health* reply
        // alike; the body-shape cases are a `200` whose candidates array this client
        // will not ingest, so the origin still answers health.
        let hostile: Vec<(&str, bool, Vec<u8>)> = vec![
            ("empty", false, Vec::new()),
            (
                "no-length",
                false,
                b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n[]".to_vec(),
            ),
            (
                "chunked",
                false,
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n0\r\n\r\n"
                    .to_vec(),
            ),
            (
                "duplicated-length",
                false,
                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]"
                    .to_vec(),
            ),
            ("not-json", false, framed("nonsense")),
            ("not-an-array", true, framed("{\"a\":1}")),
            (
                "short-body",
                false,
                b"HTTP/1.1 200 OK\r\nContent-Length: 40\r\nConnection: close\r\n\r\n[]"
                    .to_vec(),
            ),
        ];
        for (label, answers_health, raw) in hostile {
            let upstream = Upstream::new(origin_that_sends(raw));
            assert!(
                upstream.candidates("steam/1").is_err(),
                "the `{label}` answer is an upstream error"
            );
            assert_eq!(
                upstream.answers_health(),
                answers_health,
                "the `{label}` answer says what the gauge says it says"
            );
        }
    }

    #[test]
    fn a_non_200_and_a_redirect_are_upstream_errors() {
        for status in ["404 Not Found", "500 Internal Server Error", "302 Found"] {
            let raw =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    .into_bytes();
            let upstream = Upstream::new(origin_that_sends(raw));
            assert!(
                upstream.candidates("steam/1").is_err(),
                "`{status}` is not a `200`"
            );
        }
    }

    #[test]
    fn a_body_or_count_over_budget_is_refused() {
        let oversized = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            constants::MAX_RESPONSE_OCTETS + 1,
            "x".repeat(constants::MAX_RESPONSE_OCTETS + 1)
        );
        let upstream = Upstream::new(origin_that_sends(oversized.into_bytes()));
        assert!(upstream.candidates("steam/1").is_err());

        let items: Vec<String> = (1..=constants::MAX_CANDIDATES + 1)
            .map(|index| format!(r#"{{"game_id":"cand-{index}","name":"Cand {index}"}}"#))
            .collect();
        let over_count = format!("[{}]", items.join(","));
        assert!(
            parse_candidates(over_count.as_bytes()).is_err(),
            "one candidate over `MAX_CANDIDATES` is a refusal, not a truncation"
        );
    }

    #[test]
    fn valid_candidates_survive_and_the_body_is_parsed() {
        let candidates = parse_candidates(br#"[{"game_id":"ok-game","name":"Ok Game"}]"#)
            .expect("a well-formed answer");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].game_id, "ok-game");
    }

    #[test]
    fn a_response_reader_stops_at_the_header_budget() {
        let padded = format!(
            "HTTP/1.1 200 OK\r\nX-Pad: {}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]",
            "p".repeat(constants::MAX_RESPONSE_OCTETS)
        );
        let mut reader: &[u8] = padded.as_bytes();
        assert!(read_response(&mut reader).is_err());
    }

    #[test]
    fn an_unreachable_origin_is_reported_rather_than_panicking() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let upstream = Upstream::new(format!("http://127.0.0.1:{port}"));
        assert!(upstream.candidates("steam/1").is_err());
        assert!(!upstream.answers_health());
    }
}
