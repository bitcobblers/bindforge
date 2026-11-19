//! The catalog client of design 025 / ADR-025: a deliberately small, std-only
//! HTTP/1.1 loopback client plus the endpoint policy that gates it.
//!
//! The real transport is ADR-026's problem. This module exists so the matching
//! flow can run *today* against a local catalog: it speaks one request shape
//! (`GET <base>/v1/candidates?slug=<percent-encoded>`), understands exactly one
//! response shape (`200`, no `Transfer-Encoding`, one small `Content-Length`),
//! and refuses everything else by name. The endpoint policy admits exactly the
//! `http://` loopback spelling, and [`HttpCatalog::probe`] resolves the host and
//! refuses unless every address is loopback, so the client cannot be pointed at
//! the wider network by flag or env var (`025-L1`).

use crate::matches::game_id_is_valid;
use bindforge_core::escape_and_truncate;
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

/// The whole response budget, header block and body each capped at it; a body
/// of at most this many octets is read (`025-C4`).
pub const MAX_RESPONSE_OCTETS: usize = 65_536;

/// The most candidates one response may carry before the body is refused.
pub const MAX_CANDIDATES: usize = 256;

/// Socket timeouts: a hung loopback listener must not hang the scan.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// The only host spellings the endpoint policy admits, as literals — no
/// resolution-based trust, so `0177.0.0.1`-style spellings are refused.
const LOOPBACK_HOSTS: [&str; 3] = ["127.0.0.1", "[::1]", "localhost"];

/// One catalog candidate: the curated id a confirmation stores, and the name
/// the prompt shows. Serialized shape `{"game_id","name"}` (`025-C6`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub game_id: String,
    pub name: String,
}

/// The two named failures of the catalog seam, kept apart because their
/// operator-facing messages differ: a configuration the endpoint policy will
/// not run at all, and an endpoint that was tried but did not answer (or
/// answered with something undecodable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogError {
    /// The endpoint itself is outside the policy: not the loopback `http://`
    /// spelling, a non-loopback resolution, or no endpoint at all.
    TransportUnavailable(String),
    /// The endpoint was accepted but the request or response failed.
    Unreachable(String),
}

impl CatalogError {
    pub fn transport_unavailable(reason: impl Into<String>) -> Self {
        Self::TransportUnavailable(reason.into())
    }

    pub fn unreachable(reason: impl Into<String>) -> Self {
        Self::Unreachable(reason.into())
    }

    /// The single stderr line this failure is reported as (the `error: ` prefix
    /// is the caller's). The transport-unavailable line names the flag, the
    /// env var, and ADR-026 as the owner of the real transport.
    pub fn line(&self) -> String {
        match self {
            Self::TransportUnavailable(reason) => format!(
                "catalog transport unavailable: {reason}; pass --catalog-url or set \
                 BINDFORGE_CATALOG_URL with a loopback http:// endpoint — ADR-026 owns \
                 the real transport"
            ),
            Self::Unreachable(reason) => format!("catalog unreachable: {reason}"),
        }
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.line())
    }
}

impl Error for CatalogError {}

/// Splits an authority into its host and explicit port, keeping bracketed IPv6
/// literals intact. `None` port means "use the scheme default".
fn split_authority(authority: &str) -> Option<(&str, Option<&str>)> {
    if let Some(rest) = authority.strip_prefix('[') {
        let end = rest.find(']')?;
        // `rest` drops the `[`, so the `]` sits at `end + 1` in `authority`.
        let host = &authority[..=end + 1];
        let tail = &rest[end + 1..];
        if tail.is_empty() {
            return Some((host, None));
        }
        return Some((host, tail.strip_prefix(':')));
    }
    match authority.split_once(':') {
        Some((host, port)) => Some((host, Some(port))),
        None => Some((authority, None)),
    }
}

/// The endpoint policy (`025-C4`): admits exactly a literal `http://` scheme, a
/// loopback host spelling, and a digits-only port; everything else — `https`,
/// another scheme, any non-loopback authority, an `@`-bearing authority, an
/// empty host, a non-numeric port — is the named `TransportUnavailable`.
/// Returns the normalized `<scheme>://<host>:<port>` base without any path.
pub fn parse_endpoint(url: &str) -> Result<String, CatalogError> {
    let rest = url.strip_prefix("http://").ok_or_else(|| {
        CatalogError::transport_unavailable(format!(
            "`{}` does not start with the literal `http://` scheme",
            escape_and_truncate(url)
        ))
    })?;
    let authority = match rest.find(['/', '?', '#']) {
        Some(index) => &rest[..index],
        None => rest,
    };
    if authority.is_empty() {
        return Err(CatalogError::transport_unavailable(
            "the authority is empty — a host is required",
        ));
    }
    if authority.contains('@') {
        return Err(CatalogError::transport_unavailable(format!(
            "the authority `{}` carries credentials",
            escape_and_truncate(authority)
        )));
    }
    let (host, port) = split_authority(authority).ok_or_else(|| {
        CatalogError::transport_unavailable(format!(
            "the authority `{}` is not a parseable host",
            escape_and_truncate(authority)
        ))
    })?;
    if !LOOPBACK_HOSTS
        .iter()
        .any(|allowed| host.eq_ignore_ascii_case(allowed))
    {
        return Err(CatalogError::transport_unavailable(format!(
            "`{}` is not a loopback host — only `127.0.0.1`, `[::1]` and `localhost` are admitted",
            escape_and_truncate(host)
        )));
    }
    let port = match port {
        None => 80,
        Some(digits) => {
            if digits.is_empty()
                || digits.len() > 5
                || !digits.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(CatalogError::transport_unavailable(format!(
                    "the port `{}` is not a decimal number",
                    escape_and_truncate(digits)
                )));
            }
            digits.parse::<u16>().map_err(|_| {
                CatalogError::transport_unavailable(format!(
                    "the port `{digits}` is outside 0–65535"
                ))
            })?
        }
    };
    Ok(format!("http://{host}:{port}"))
}

/// Percent-encodes everything outside the RFC 3986 unreserved set, so a lookup
/// key travels with its `/` as `%2F` and never as a path separator.
fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// The one request target, pure (`025-C4`): `<base>/v1/candidates?slug=<enc>`.
/// The client below sends exactly the tail this composes, so the pinned URL
/// spelling and the wire spelling are one definition.
pub fn candidates_url(base: &str, slug: &str) -> String {
    format!(
        "{}/v1/candidates?slug={}",
        base.trim_end_matches('/'),
        percent_encode(slug)
    )
}

/// The trait the scan flow talks to, so the flow's tests and the fake catalog
/// need no socket and the real client stays behind one narrow surface.
pub trait CatalogClient {
    /// The candidates cataloged for `slug`, in the response's order.
    fn candidates(&self, slug: &str) -> Result<Vec<Candidate>, CatalogError>;
}

/// The std-only loopback client built from `--catalog-url` /
/// `BINDFORGE_CATALOG_URL` (`025-C4`, `025-L1`).
#[derive(Debug)]
pub struct HttpCatalog {
    base: String,
}

impl HttpCatalog {
    /// Builds the client through the endpoint policy; a rejected endpoint is the
    /// named `TransportUnavailable` before any socket exists.
    pub fn new(url: &str) -> Result<Self, CatalogError> {
        Ok(Self {
            base: parse_endpoint(url)?,
        })
    }

    /// The host and port of the normalized base, for connecting.
    fn authority(&self) -> (&str, u16) {
        let authority = self.base.trim_start_matches("http://");
        let (host, port) = split_authority(authority).unwrap_or((authority, Some("80")));
        // `to_socket_addrs` wants the bare address; brackets belong to the URL.
        let host = host.trim_start_matches('[').trim_end_matches(']');
        (host, port.unwrap_or("80").parse().unwrap_or(80))
    }

    /// The connectivity gate, run before any plugin is constructed (`025-C4`):
    /// resolves the host and refuses unless *every* address is loopback, then
    /// proves an answer with one connect. A failure is the named `Unreachable`
    /// (or `TransportUnavailable` for a now-non-loopback resolution).
    pub fn probe(&self) -> Result<(), CatalogError> {
        let (host, port) = self.authority();
        let addrs: Vec<SocketAddr> = (host, port)
            .to_socket_addrs()
            .map_err(|error| {
                CatalogError::unreachable(format!("cannot resolve host `{host}`: {error}"))
            })?
            .collect();
        if addrs.is_empty() {
            return Err(CatalogError::unreachable(format!(
                "host `{host}` resolves to no address"
            )));
        }
        for addr in &addrs {
            if !addr.ip().is_loopback() {
                return Err(CatalogError::transport_unavailable(format!(
                    "host `{host}` resolves to the non-loopback address {}",
                    addr.ip()
                )));
            }
        }
        for addr in &addrs {
            match TcpStream::connect_timeout(addr, CONNECT_TIMEOUT) {
                Ok(_) => return Ok(()),
                Err(error) => {
                    if addrs.len() == 1 || !is_connect_refusal(&error) {
                        return Err(CatalogError::unreachable(format!(
                            "no catalog answered at {}: {error}",
                            addr.ip()
                        )));
                    }
                }
            }
        }
        Err(CatalogError::unreachable(format!(
            "no catalog answered at `{host}:{port}`"
        )))
    }
}

fn is_connect_refusal(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::TimedOut
    )
}

/// Reads one HTTP/1.1 response header block from `stream`, refusing anything
/// outside the understood shape while staying inside the octet budget.
fn read_response(reader: &mut impl Read) -> Result<Vec<u8>, CatalogError> {
    let mut head: Vec<u8> = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let read = reader
            .read(&mut byte)
            .map_err(|error| CatalogError::unreachable(format!("reading the response: {error}")))?;
        if read == 0 {
            return Err(CatalogError::unreachable(
                "the connection closed before the header block ended",
            ));
        }
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
        if head.len() > MAX_RESPONSE_OCTETS {
            return Err(CatalogError::unreachable(format!(
                "the header block is over the {MAX_RESPONSE_OCTETS}-octet budget"
            )));
        }
    }

    let text = String::from_utf8_lossy(&head[..head.len() - 4]);
    let mut lines = text.split("\r\n");
    let status = lines.next().unwrap_or_default();
    let code = status.split_whitespace().nth(1).unwrap_or_default();
    if code != "200" {
        return Err(CatalogError::unreachable(format!(
            "the catalog answered `{}`",
            escape_and_truncate(status)
        )));
    }

    let mut content_length: Option<usize> = None;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (name, value) = match line.split_once(':') {
            Some(half) => half,
            None => {
                return Err(CatalogError::unreachable(format!(
                    "header line `{}` is not understood",
                    escape_and_truncate(line)
                )));
            }
        };
        let name = name.trim();
        let value = value.trim();
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(CatalogError::unreachable(
                "`Transfer-Encoding` is not a body framing this client understands",
            ));
        }
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err(CatalogError::unreachable(
                    "`Content-Length` appears more than once",
                ));
            }
            if value.is_empty() || value.len() > 5 || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err(CatalogError::unreachable(format!(
                    "`Content-Length: {}` is not a decimal length",
                    escape_and_truncate(value)
                )));
            }
            let length: usize = value.parse().map_err(|_| {
                CatalogError::unreachable(format!("`Content-Length: {value}` is out of range"))
            })?;
            if length > MAX_RESPONSE_OCTETS {
                return Err(CatalogError::unreachable(format!(
                    "the body promises {length} octets over the {MAX_RESPONSE_OCTETS}-octet budget"
                )));
            }
            content_length = Some(length);
        }
    }
    let length = content_length
        .ok_or_else(|| CatalogError::unreachable("the response carries no `Content-Length`"))?;

    let mut body = vec![0u8; length];
    reader
        .read_exact(&mut body)
        .map_err(|error| CatalogError::unreachable(format!("reading the body: {error}")))?;
    Ok(body)
}

/// Parses a `200` body into candidates, keeping the response's order; any
/// body outside `[{game_id, name}, …]` — including a non-array or over-count —
/// is the named `Unreachable` refusal.
fn parse_candidates(body: &[u8]) -> Result<Vec<Candidate>, CatalogError> {
    let document: serde_json::Value = serde_json::from_slice(body)
        .map_err(|error| CatalogError::unreachable(format!("the body is not JSON: {error}")))?;
    let items = document
        .as_array()
        .ok_or_else(|| CatalogError::unreachable("the body is not a JSON array of candidates"))?;
    if items.len() > MAX_CANDIDATES {
        return Err(CatalogError::unreachable(format!(
            "the body carries {} candidates over the {MAX_CANDIDATES} budget",
            items.len()
        )));
    }
    let mut candidates = Vec::with_capacity(items.len());
    for item in items {
        let game_id = item.get("game_id").and_then(serde_json::Value::as_str);
        let name = item.get("name").and_then(serde_json::Value::as_str);
        match (game_id, name) {
            (Some(game_id), Some(name)) => candidates.push(Candidate {
                game_id: game_id.to_string(),
                name: name.to_string(),
            }),
            _ => {
                return Err(CatalogError::unreachable(
                    "a candidate is not a `{game_id, name}` object of strings",
                ));
            }
        }
    }
    Ok(candidates)
}

impl CatalogClient for HttpCatalog {
    /// One plain `GET` per call; a mid-scan failure is the named `Unreachable`
    /// and the run fails closed (the caller aborts, printing no partial table).
    fn candidates(&self, slug: &str) -> Result<Vec<Candidate>, CatalogError> {
        let (host, port) = self.authority();
        let addrs: Vec<SocketAddr> = (host, port)
            .to_socket_addrs()
            .map_err(|error| {
                CatalogError::unreachable(format!("cannot resolve host `{host}`: {error}"))
            })?
            .collect();
        let mut stream = None;
        let mut last_error = String::from("the host resolved to no address");
        for addr in &addrs {
            match TcpStream::connect_timeout(addr, CONNECT_TIMEOUT) {
                Ok(connected) => {
                    let _ = connected.set_read_timeout(Some(READ_TIMEOUT));
                    let _ = connected.set_write_timeout(Some(READ_TIMEOUT));
                    stream = Some(connected);
                    break;
                }
                Err(error) => last_error = format!("{error}"),
            }
        }
        let mut stream = stream.ok_or_else(|| {
            CatalogError::unreachable(format!("connecting to `{host}:{port}`: {last_error}"))
        })?;

        // The wire target is exactly the tail of the pinned `candidates_url`
        // spelling against this client's normalized base.
        let url = candidates_url(&self.base, slug);
        let target = &url[self.base.trim_end_matches('/').len()..];
        let request = format!(
            "GET {target} HTTP/1.1\r\nHost: {host}:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .and_then(|()| stream.flush())
            .map_err(|error| CatalogError::unreachable(format!("sending the request: {error}")))?;
        let body = read_response(&mut stream)?;
        parse_candidates(&body)
    }
}

/// The candidate-id gate (`025-L5`): a hostile `game_id` in a `200` body never
/// reaches the prompt or the store. Survivors keep the response's order; each
/// drop is one named stderr refusal line echoing the escaped value.
pub fn partition_candidates(slug: &str, candidates: &[Candidate]) -> (Vec<Candidate>, Vec<String>) {
    let mut kept = Vec::new();
    let mut refused = Vec::new();
    for candidate in candidates {
        if game_id_is_valid(&candidate.game_id) {
            kept.push(candidate.clone());
        } else {
            refused.push(format!(
                "dropped candidate for `{slug}` with an invalid game id: {}",
                escape_and_truncate(&candidate.game_id)
            ));
        }
    }
    (kept, refused)
}

#[cfg(test)]
mod tests {
    //! Unit tests for the pure seams of the client: the endpoint policy's
    //! admitted and refused spellings, the request URL's encoding, and the
    //! candidate partition. The socket half is pinned end-to-end by
    //! `cli_catalog_gate_tests` against a loopback listener.
    use super::{CatalogError, HttpCatalog, candidates_url, parse_endpoint, partition_candidates};
    use crate::catalog::Candidate;

    #[test]
    fn the_policy_admits_the_loopback_spellings_and_normalizes_them() {
        assert_eq!(
            parse_endpoint("http://127.0.0.1:8080").expect("v4 loopback"),
            "http://127.0.0.1:8080"
        );
        assert_eq!(
            parse_endpoint("http://localhost:9/").expect("named loopback"),
            "http://localhost:9"
        );
        assert_eq!(
            parse_endpoint("http://[::1]:8080/v1").expect("v6 loopback"),
            "http://[::1]:8080"
        );
        assert_eq!(
            parse_endpoint("http://127.0.0.1").expect("no port takes the default"),
            "http://127.0.0.1:80"
        );
    }

    #[test]
    fn the_policy_refuses_everything_beyond_the_loopback_spelling() {
        for url in [
            "https://127.0.0.1:8080/",
            "ftp://127.0.0.1/",
            "127.0.0.1:8080",
            "http://catalog.example",
            "http://0177.0.0.1/",
            "http://:8080/",
            "http://127.0.0.1:8080@host.example/",
            "http://127.0.0.1:99x/",
            "http://127.0.0.1:",
            "http://127.0.0.1:70000/",
        ] {
            let error = parse_endpoint(url).expect_err("`{url}` is outside the policy");
            assert!(
                matches!(error, CatalogError::TransportUnavailable(_)),
                "`{url}` must be a transport-unavailable refusal, got {error:?}"
            );
            let line = error.line();
            assert!(
                line.contains("--catalog-url")
                    && line.contains("BINDFORGE_CATALOG_URL")
                    && line.contains("ADR-026"),
                "the refusal names both config sources and the owning ADR: {line}"
            );
            assert!(!line.contains('\n'), "one refusal is one line: {line:?}");
        }
    }

    #[test]
    fn the_request_url_encodes_the_separator_of_the_lookup_key() {
        assert_eq!(
            candidates_url("http://127.0.0.1:8080", "steam/900001"),
            "http://127.0.0.1:8080/v1/candidates?slug=steam%2F900001"
        );
        assert_eq!(
            candidates_url("http://127.0.0.1:8080", "epic/fn\0weird"),
            "http://127.0.0.1:8080/v1/candidates?slug=epic%2Ffn%00weird",
            "anything unreserved-unsafe travels percent-encoded"
        );
    }

    #[test]
    fn a_client_built_from_a_bad_endpoint_refuses_before_any_socket() {
        let error = HttpCatalog::new("http://example.com").expect_err("not loopback");
        assert!(matches!(error, CatalogError::TransportUnavailable(_)));
    }

    #[test]
    fn the_partition_drops_invalid_ids_and_keeps_survivors_in_order() {
        let candidates = vec![
            Candidate {
                game_id: "../x".to_string(),
                name: "Evil Twin".to_string(),
            },
            Candidate {
                game_id: "good-game".to_string(),
                name: "Good".to_string(),
            },
            Candidate {
                game_id: "UPPER".to_string(),
                name: "Shouty".to_string(),
            },
        ];
        let (kept, refused) = partition_candidates("steam/1", &candidates);
        assert_eq!(
            kept.iter()
                .map(|candidate| candidate.game_id.as_str())
                .collect::<Vec<&str>>(),
            vec!["good-game"],
            "only the grammar-checked id survives, in response order"
        );
        assert_eq!(refused.len(), 2);
        assert!(refused[0].contains("../x"), "the drop echoes the value");
        assert!(refused.iter().all(|line| !line.contains('\n')));
    }
}
