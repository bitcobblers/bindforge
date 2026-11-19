//! The HTTP/1.1 surface (proposal §Internal components, §Request handling).
//!
//! Deliberately a small subset: the absolute-form request target is not
//! supported, so a client can never aim this server at somebody else's host;
//! there is no chunked coding to be smuggled through; one connection answers
//! exactly one request and closes (`A23`), so an idle client cannot hold an
//! admitted slot. Bodies are read only for the methods that carry one, and only
//! after the framing rules pass, with the octet budget checked against the
//! declared length before a single body octet is buffered.
//!
//! The response side keeps its own discipline: a status line, a small JSON
//! object body, framed by exactly the octets it sends, and never a raw value
//! from the request or from a document inside the status line.

use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

use crate::constants;
use crate::keys::percent_decode;

/// A parsed request. Its framing and routing live in [`RequestHead`]; the body is
/// the part a connection may be refused *before* sending.
#[derive(Debug)]
pub struct Request {
    /// Everything the framing pass produced.
    pub head: RequestHead,
    pub body: Vec<u8>,
}

/// The part of a request the server can act on before a single body octet is
/// buffered: the routing decision, the admin enablement, and the bearer credential
/// are all in the header block, so a request the headers already condemn never has
/// its declared body read (§Budgets: `MAX_REQUEST_BODY_OCTETS` is a load-side
/// figure, not an unauthenticated peer's memory allowance).
#[derive(Debug)]
pub struct RequestHead {
    pub method: String,
    /// The request target's path, percent-decoded once during target parsing.
    pub path: String,
    /// Query parameters, percent-decoded once, split on `&` *before* decoding so
    /// an encoded `%26` stays inside the value it arrived in. A parameter that
    /// arrives twice keeps its first value.
    pub query: Vec<(String, String)>,
    /// Lower-cased header names, in the order received.
    pub headers: Vec<(String, String)>,
}

impl RequestHead {
    /// The value of one header, or `None` when it is absent.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        find_header(&self.headers, name)
    }

    /// The value of one query parameter, or `None` when it is absent.
    #[must_use]
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The length the request declares for its body, when it declares one this
    /// server could honour. Only used to bound the discard on a refused request:
    /// an unusable declaration is refused on the read path, not here.
    #[must_use]
    pub fn declared_body_octets(&self) -> Option<usize> {
        self.header("content-length")
            .and_then(|value| value.parse::<usize>().ok())
    }

    /// Reads the declared body and completes the request. The declared length has
    /// already been checked against the budget before any octet of it is buffered.
    ///
    /// # Errors
    ///
    /// Refuses the framings this server does not read, and a body that ends early.
    pub fn into_request(self, stream: &mut TcpStream) -> Result<Request, Refused> {
        let body = read_body(stream, &self.method, &self.headers)?;
        Ok(Request { head: self, body })
    }
}

/// The name/value lookup both request views share.
fn find_header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(header, _)| header == name)
        .map(|(_, value)| value.as_str())
}

impl Request {
    /// The method the request arrived with.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.head.method
    }

    /// The percent-decoded path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.head.path
    }

    /// The value of one query parameter, or `None` when it is absent.
    #[must_use]
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.head.query_param(name)
    }
}

/// A request this server will not act on, with the status to answer it with. The
/// framing refusals are `400`s; the method refusals are `405`s and carry the
/// methods the path does answer to. Every refusal still gets a well-formed small
/// JSON body, because Test strategy #16's shape clause applies to refusals too.
#[derive(Debug)]
pub struct Refused {
    pub status: u16,
    pub reason: &'static str,
    pub detail: String,
    /// The `Allow` list a `405` names, when the refusal came from routing.
    pub allow: Option<&'static str>,
}

impl Refused {
    fn new(status: u16, reason: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            detail: detail.into(),
            allow: None,
        }
    }

    /// A `400` for a request this server will not parse or will not answer at all.
    #[must_use]
    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(400, "Bad Request", detail)
    }

    /// A `405` for a path that exists and does not carry this method.
    #[must_use]
    pub fn method_not_allowed(detail: impl Into<String>, allow: &'static str) -> Self {
        Self {
            status: 405,
            reason: "Method Not Allowed",
            detail: detail.into(),
            allow: Some(allow),
        }
    }

    /// A `404` body for a path this contract does not define.
    #[must_use]
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(404, "Not Found", detail)
    }

    /// A `401` for a missing or wrong admin credential.
    #[must_use]
    pub fn unauthorized(detail: impl Into<String>) -> Self {
        Self::new(401, "Unauthorized", detail)
    }

    /// A `415` for a request whose declared media type this server does not read.
    /// Payload-side, unlike the `400` framings: the request is well formed and its
    /// body is simply not JSON.
    #[must_use]
    pub fn unsupported_media_type(detail: impl Into<String>) -> Self {
        Self::new(415, "Unsupported Media Type", detail)
    }
}

/// Reads the request line and the header block, and nothing else. This is the
/// framing pass every request takes: the routing decision, the admin enablement,
/// and the bearer credential are all in this half, so a connection that is going to
/// be refused never has the body it declared buffered. `Ok(None)` means the peer
/// closed the connection cleanly without asking anything, which is not an error and
/// admits nothing to do.
pub fn read_request_head(stream: &mut TcpStream) -> Result<Option<RequestHead>, Refused> {
    let block = match read_header_block(stream)? {
        Some(block) => block,
        None => return Ok(None),
    };
    let mut lines = block.lines();
    let target_line = lines
        .next()
        .ok_or_else(|| Refused::bad_request("the request line is missing"))?;
    let mut parts = target_line.split_whitespace();
    let (Some(method), Some(target), Some(version), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(Refused::bad_request(
            "the request line is not `METHOD TARGET VERSION`",
        ));
    };
    if !version.eq_ignore_ascii_case("HTTP/1.1") {
        return Err(Refused::bad_request("only HTTP/1.1 is served"));
    }

    let mut headers: Vec<(String, String)> = Vec::new();
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            return Err(Refused::bad_request(
                "a header line has no `Name: value` form",
            ));
        };
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() {
            return Err(Refused::bad_request("a header line has an empty name"));
        }
        if name == "transfer-encoding" {
            return Err(Refused::bad_request(
                "Transfer-Encoding is not served; send Content-Length or nothing",
            ));
        }
        if headers.iter().any(|(seen, _)| *seen == name) {
            // RFC 9110 §5.3 allows a repeated field only where it is defined as a
            // comma-separated list, so a request that repeats a field has no single
            // meaning here. The shipped client refuses a duplicated `Content-Length`
            // on the way back; this refuses it on the way in.
            return Err(Refused::bad_request(format!(
                "the {name} header appears twice; send it once"
            )));
        }
        headers.push((name, value.trim().to_string()));
    }

    let (path, query) = split_target(target)?;
    Ok(Some(RequestHead {
        method: method.to_ascii_uppercase(),
        path,
        query,
        headers,
    }))
}

/// Splits the request target into a percent-decoded path and decoded query
/// parameters. The absolute form (`http://host/path`) is refused: this server
/// only ever serves its own paths.
fn split_target(target: &str) -> Result<(String, Vec<(String, String)>), Refused> {
    if target.starts_with("http://") || target.starts_with("https://") {
        return Err(Refused::bad_request(
            "the absolute-form request target is not served; send the path alone",
        ));
    }
    if !target.starts_with('/') {
        return Err(Refused::bad_request(
            "the request target must be a path beginning with `/`",
        ));
    }
    let (raw_path, raw_query) = match target.split_once('?') {
        Some((path, query)) => (path, query),
        None => (target, ""),
    };
    let path = percent_decode(raw_path)
        .ok_or_else(|| Refused::bad_request("the request target has an invalid % escape"))?;
    if path.contains('\0') {
        return Err(Refused::bad_request("the request target has a NUL octet"));
    }
    let mut query: Vec<(String, String)> = Vec::new();
    for pair in raw_query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (raw_name, raw_value) = match pair.split_once('=') {
            Some((name, value)) => (name, value),
            None => (pair, ""),
        };
        let name = percent_decode(raw_name).ok_or_else(|| {
            Refused::bad_request("a query parameter name has an invalid % escape")
        })?;
        let value = percent_decode(raw_value)
            .ok_or_else(|| Refused::bad_request("a query value has an invalid % escape"))?;
        if !query.iter().any(|(seen, _)| *seen == name) {
            query.push((name, value));
        }
    }
    Ok((path, query))
}

/// The declared body length, checked against the budget before any body octet is
/// read, so an over-budget request is refused rather than buffered.
fn declared_body_octets(headers: &[(String, String)]) -> Result<Option<u64>, Refused> {
    let Some((_, value)) = headers.iter().find(|(name, _)| name == "content-length") else {
        return Ok(None);
    };
    let octets = value
        .parse::<u64>()
        .map_err(|_| Refused::bad_request("Content-Length must be a decimal number of octets"))?;
    if octets > u64::try_from(constants::MAX_REQUEST_BODY_OCTETS).unwrap_or(u64::MAX) {
        return Err(Refused::bad_request(format!(
            "the request body may be at most {} octets",
            constants::MAX_REQUEST_BODY_OCTETS
        )));
    }
    Ok(Some(octets))
}

fn read_body(
    stream: &mut TcpStream,
    method: &str,
    headers: &[(String, String)],
) -> Result<Vec<u8>, Refused> {
    let declared = declared_body_octets(headers)?;
    let octets = match declared {
        // A method that carries a body has to say how long it is: there is no
        // chunked coding here and the connection closes after one answer, so
        // there is nothing to read an end from.
        None if matches!(method, "POST" | "PUT") => {
            return Err(Refused::bad_request(
                "a request that carries a body must send Content-Length",
            ));
        }
        None => 0,
        Some(octets) => usize::try_from(octets).unwrap_or(usize::MAX),
    };
    if octets == 0 {
        return Ok(Vec::new());
    }
    let mut body = vec![0u8; octets];
    stream
        .read_exact(&mut body)
        .map_err(|error| Refused::bad_request(format!("the request body ended early: {error}")))?;
    Ok(body)
}

fn read_header_block(stream: &mut TcpStream) -> Result<Option<String>, Refused> {
    let mut buffer: Vec<u8> = Vec::with_capacity(1024);
    let mut byte = [0u8; 1];
    loop {
        if buffer.len() > constants::MAX_REQUEST_HEADER_OCTETS {
            return Err(Refused::bad_request(format!(
                "the header block may be at most {} octets",
                constants::MAX_REQUEST_HEADER_OCTETS
            )));
        }
        match stream.read(&mut byte) {
            Ok(0) => {
                return if buffer.is_empty() {
                    Ok(None)
                } else {
                    Err(Refused::bad_request(
                        "the connection closed in the middle of a request",
                    ))
                };
            }
            Ok(_) => buffer.push(byte[0]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                return Err(Refused::bad_request("the request did not arrive in time"));
            }
            Err(error) => return Err(Refused::bad_request(format!("read failed: {error}"))),
        }
        // One octet arrived, so the only terminator that can be *new* is the one
        // ending at it: testing the last four octets is the whole scan, and a
        // maximal header block costs one comparison per octet rather than a rescan
        // from offset `0`. The reader's own upstream side does the same
        // (`upstream.rs`), and the cap above still bounds the block.
        if buffer.ends_with(b"\r\n\r\n".as_slice()) {
            // The terminating `\r\n\r\n` names a blank line that is not a header
            // line, so the block the caller parses is the octets before it.
            let block = buffer[..buffer.len() - 4].to_vec();
            return String::from_utf8(block)
                .map(Some)
                .map_err(|_| Refused::bad_request("the header block is not UTF-8"));
        }
    }
}

/// Discards at most [`constants::UNREAD_BODY_DRAIN_OCTETS`] of the body a refused
/// request declared, in bounded pieces, and never buffers it.
///
/// A request refused on the header block alone still has its body arriving, and
/// closing a socket with unread received octets resets it — which would take the
/// refusal the peer is owed down with it. So the octets are thrown away rather than
/// held: the memory a declared `Content-Length` could otherwise buy is capped by the
/// discard, and a peer that declared more than the discard covers learns what the
/// refused connection it was answered with looks like from the near end.
pub fn drain_unread_body(stream: &mut TcpStream, declared: Option<usize>) {
    let Some(remaining) = declared else {
        return;
    };
    let remaining = remaining.min(constants::UNREAD_BODY_DRAIN_OCTETS);
    if remaining == 0 {
        return;
    }
    let _ = stream.set_read_timeout(Some(constants::UNREAD_BODY_DRAIN_TIMEOUT));
    let mut scratch = [0u8; 4096];
    let mut thrown = 0usize;
    while thrown < remaining {
        // Only what the request declared is the request's: the read is clamped so a
        // discard never swallows octets past the declared length.
        let want = scratch.len().min(remaining - thrown);
        match stream.read(&mut scratch[..want]) {
            Ok(0) | Err(_) => return,
            Ok(read) => thrown += read,
        }
    }
}

/// Writes one answer with the given status line and JSON body, then the caller
/// closes the connection. `extra` lines are the per-answer headers (`Allow`,
/// `WWW-Authenticate`).
pub fn write_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    body: &[u8],
    extra: &[String],
) -> std::io::Result<()> {
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
        body.len()
    );
    for line in extra {
        response.push_str(line);
        response.push_str("\r\n");
    }
    response.push_str("Cache-Control: no-store\r\nConnection: close\r\n\r\n");
    stream.write_all(response.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

/// Answers a [`Refused`] the way its status says: a `405` names the methods the
/// route does carry, a `401` names the scheme it expects, and every refusal
/// carries a small JSON body that never repeats a value the caller sent.
pub fn write_refused(stream: &mut TcpStream, refused: &Refused) -> std::io::Result<()> {
    let mut extra = Vec::new();
    if let Some(allow) = refused.allow {
        extra.push(format!("Allow: {allow}"));
    }
    if refused.status == 401 {
        extra.push("WWW-Authenticate: Bearer".to_string());
    }
    let body = refusal_body(error_token(refused.status), &refused.detail);
    write_response(stream, refused.status, refused.reason, &body, &extra)
}

/// The `error` token of a refusal body, keyed on the status alone so the body
/// shape cannot leak anything about the request.
fn error_token(status: u16) -> &'static str {
    match status {
        400 => "bad_request",
        401 => "unauthorized",
        404 => "not_found",
        405 => "method_not_allowed",
        415 => "unsupported_media_type",
        422 => "unprocessable",
        503 => "unavailable",
        _ => "error",
    }
}

/// Serialises a JSON object from name/value pairs in the order given, so the
/// documents this server answers with have a stable key order and carry exactly
/// the keys the contract names — no extras.
#[must_use]
pub fn json_object(pairs: &[(&str, serde_json::Value)]) -> Vec<u8> {
    let mut map = serde_json::Map::new();
    for (name, value) in pairs {
        map.insert((*name).to_string(), value.clone());
    }
    serde_json::to_vec(&serde_json::Value::Object(map)).expect("a map of JSON values serialises")
}

/// The body of a refusal. `detail` is a server-chosen message, already free of
/// raw request octets.
#[must_use]
pub fn refusal_body(error: &str, detail: &str) -> Vec<u8> {
    json_object(&[
        ("error", serde_json::Value::String(error.to_string())),
        ("detail", serde_json::Value::String(detail.to_string())),
    ])
}

/// The read timeout an admitted connection gets before its one request must
/// arrive.
#[must_use]
pub fn request_read_timeout() -> std::time::Duration {
    constants::REQUEST_READ_TIMEOUT
}

/// The write timeout for the answer.
#[must_use]
pub fn response_write_timeout() -> std::time::Duration {
    constants::RESPONSE_WRITE_TIMEOUT
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn a_well_formed_request_parses_into_its_parts() {
        let request = parse(
            b"GET /v1/candidates?slug=steam%2F900001&slug=steam%2F2 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        );
        assert_eq!(request.method(), "GET");
        assert_eq!(request.path(), "/v1/candidates");
        assert_eq!(
            request.query_param("slug"),
            Some("steam/900001"),
            "the query is decoded once, so an encoded separator stays inside the value, \
             and the first of two spellings wins"
        );
        assert_eq!(request.head.header("host"), Some("127.0.0.1"));
        assert!(request.body.is_empty());
    }

    #[test]
    fn the_reader_refuses_the_framings_it_does_not_understand() {
        for raw in [
            "GET /v1/health HTTP/1.0\r\n\r\n",
            "GET /v1/health HTTP/1.1 extra\r\n\r\n",
            "GET http://127.0.0.1:1/v1/health HTTP/1.1\r\n\r\n",
            "GET v1/health HTTP/1.1\r\n\r\n",
            "GET /v1/health?slug=%zz HTTP/1.1\r\n\r\n",
            "GET /v1/health?slug=%41 HTTP/1.1\r\nnot-a-header-line\r\n\r\n",
            "GET /v1/health\r\n\r\n",
            "GET /v1/health HTTP/1.1\r\nContent-Length: nope\r\n\r\n",
        ] {
            let error = parse_error(raw);
            assert_eq!(error.status, 400, "{raw:?} is a framing refusal");
        }
    }

    #[test]
    fn transfer_encoding_and_a_repeated_header_are_refused_at_the_door() {
        let chunked = parse_error(
            "POST /v1/admin/games HTTP/1.1\r\nTransfer-Encoding: chunked\r\nContent-Length: 2\r\n\r\n[]",
        );
        assert_eq!(chunked.status, 400);
        assert!(chunked.detail.contains("Transfer-Encoding"));

        let repeated = parse_error("GET /v1/health HTTP/1.1\r\nHost: one\r\nHost: two\r\n\r\n");
        assert_eq!(repeated.status, 400);
        assert!(repeated.detail.contains("appears twice"));
    }

    #[test]
    fn a_body_is_read_only_for_the_methods_that_carry_one() {
        let posted = parse(b"POST /v1/admin/games HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello");
        assert_eq!(posted.body, b"hello");

        let missing = parse_error("POST /v1/admin/games HTTP/1.1\r\nHost: x\r\n\r\n");
        assert!(missing.detail.contains("Content-Length"));

        // A `DELETE` may declare a length; it is read and ignored, which is what
        // lets the octet budget be breached from a method with no defined body.
        let deleted = parse(b"DELETE /v1/admin/games/x HTTP/1.1\r\nContent-Length: 0\r\n\r\n");
        assert!(deleted.body.is_empty());
    }

    #[test]
    fn the_header_pass_never_reads_the_declared_body() {
        // Routing, enablement and the credential are all in this half, so a request
        // the headers already condemn is refused without buffering a body the peer
        // declared.
        let (mut client, mut server) = pair();
        client
            .write_all(b"POST /v1/admin/games HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello")
            .unwrap();
        client.flush().unwrap();
        let head = read_request_head(&mut server)
            .expect("the head parses")
            .expect("a request arrived");
        assert_eq!(head.path, "/v1/admin/games");
        assert_eq!(head.method, "POST");
        assert_eq!(head.declared_body_octets(), Some(5));
        let mut still_on_the_wire = [0u8; 5];
        server.read_exact(&mut still_on_the_wire).unwrap();
        assert_eq!(&still_on_the_wire, b"hello");
    }

    #[test]
    fn a_refused_body_is_discarded_rather_than_buffered() {
        let (mut client, mut server) = pair();
        client.write_all(b"0123456789").unwrap();
        client.flush().unwrap();
        drain_unread_body(&mut server, Some(4));
        let mut rest = [0u8; 2];
        server.read_exact(&mut rest).unwrap();
        assert_eq!(&rest, b"45", "only the declared count is discarded");
        drain_unread_body(&mut server, None);
    }

    #[test]
    fn a_body_past_the_octet_budget_is_refused_before_it_is_read() {
        let error = parse_error(&format!(
            "POST /v1/admin/games HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            constants::MAX_REQUEST_BODY_OCTETS + 1
        ));
        assert_eq!(error.status, 400);
        assert!(error.detail.contains("octets"));
    }

    #[test]
    fn a_header_block_past_the_cap_is_refused() {
        let raw = format!(
            "GET /v1/health HTTP/1.1\r\nX-Pad: {}\r\n\r\n",
            "a".repeat(constants::MAX_REQUEST_HEADER_OCTETS + 1_000)
        );
        let error = parse_error(&raw);
        assert_eq!(error.status, 400);
        assert!(error.detail.contains("header block"));
    }

    #[test]
    fn an_answer_promises_exactly_the_body_it_sends() {
        let (mut client, mut server) = pair();
        let body = json_object(&[("status", serde_json::Value::String("ok".into()))]);
        write_response(&mut server, 200, "OK", &body, &[]).unwrap();
        drop(server);
        let mut answer = String::new();
        client.read_to_string(&mut answer).unwrap();
        let promised = answer
            .lines()
            .find(|line| line.starts_with("Content-Length:"))
            .and_then(|line| line.split_once(':'))
            .map(|(_, value)| value.trim().parse::<usize>().unwrap())
            .expect("an answer promises its body length");
        assert_eq!(promised, body.len());
        assert!(answer.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(answer.ends_with(String::from_utf8(body).unwrap().as_str()));
    }

    #[test]
    fn a_refusal_carries_its_allow_list_and_a_bearer_challenge() {
        let (mut client, mut server) = pair();
        let refused = Refused::method_not_allowed("POST is not carried here", "GET");
        write_refused(&mut server, &refused).unwrap();
        let (mut other_client, mut other_server) = pair();
        let unauthorized = Refused::unauthorized("no credential");
        write_refused(&mut other_server, &unauthorized).unwrap();
        drop(server);
        drop(other_server);

        let mut allowed = String::new();
        client.read_to_string(&mut allowed).unwrap();
        assert!(allowed.starts_with("HTTP/1.1 405 Method Not Allowed\r\n"));
        assert!(allowed.contains("Allow: GET\r\n"));

        let mut challenged = String::new();
        other_client.read_to_string(&mut challenged).unwrap();
        assert!(challenged.starts_with("HTTP/1.1 401 Unauthorized\r\n"));
        assert!(challenged.contains("WWW-Authenticate: Bearer\r\n"));
    }

    /// Reads one whole request the way a caller does: the framing pass, then the
    /// body it declares.
    fn read_whole(server: &mut TcpStream) -> Result<Option<Request>, Refused> {
        let Some(head) = read_request_head(server)? else {
            return Ok(None);
        };
        Ok(Some(head.into_request(server)?))
    }

    /// Parses raw request octets that are complete on their own.
    fn parse(raw: &[u8]) -> Request {
        let (mut client, mut server) = pair();
        client.write_all(raw).unwrap();
        client.flush().unwrap();
        match read_whole(&mut server) {
            Ok(Some(request)) => request,
            other => panic!("expected a request, got {other:?}"),
        }
    }

    fn parse_error(raw: &str) -> Refused {
        let (mut client, mut server) = pair();
        client.write_all(raw.as_bytes()).unwrap();
        client.flush().unwrap();
        match read_whole(&mut server) {
            Err(error) => error,
            Ok(other) => panic!("expected a refusal for {raw:?}, got {other:?}"),
        }
    }

    fn pair() -> (TcpStream, TcpStream) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let client = TcpStream::connect(address).unwrap();
        let (server, _) = listener.accept().unwrap();
        let timeout = Some(Duration::from_secs(5));
        client.set_read_timeout(timeout).unwrap();
        client.set_write_timeout(timeout).unwrap();
        server.set_read_timeout(timeout).unwrap();
        server.set_write_timeout(timeout).unwrap();
        (client, server)
    }
}
