//! The serving face, end to end, in this process (design 026: §The serving face,
//! §The admin face and auth).
//!
//! The suites under `tests/` drive the same routes through the built binary; this
//! module pins them against the real `std::net` accept loop, which is the only
//! place the crate can check its own framing without a subprocess: the byte shape
//! `026-C2` fixes (one decimal `Content-Length`, no `Transfer-Encoding`, one answer
//! per connection), the `404`/`405`/`401` posture, the projection the join is
//! allowed to emit, and the admin mutation's persist.
//!
//! Every corpus lives in a private temp directory that is removed when the test
//! ends, and every socket is a loopback listener in this process: no test here
//! resolves or dials anything off-host (A15).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::admin::Admin;
use crate::serve;
use crate::state::State;

/// A corpus document in a private temp directory, removed when the test ends.
struct Corpus {
    path: PathBuf,
}

impl Corpus {
    fn write(label: &str, document: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "bindforge-wire-{label}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("a temp corpus directory");
        let path = directory.join("catalog.json");
        std::fs::write(&path, document).expect("the corpus is writable");
        Self { path }
    }

    fn text(&self) -> String {
        std::fs::read_to_string(&self.path).expect("the corpus is readable")
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(
            self.path
                .parent()
                .expect("the corpus sits in its own directory"),
        );
    }
}

fn catalog_document(revision: u64, records: &[(&str, &str, &[&str])]) -> String {
    let games: Vec<serde_json::Value> = records
        .iter()
        .map(|(game_id, name, keys)| {
            serde_json::json!({"game_id": game_id, "name": name, "source_keys": keys})
        })
        .collect();
    serde_json::to_string(&serde_json::json!({
        "version": 1, "revision": revision, "games": games,
    }))
    .expect("a catalog document serialises")
}

/// The three-record corpus most of these checks read: two games share one key, so
/// the join's order is observable.
fn shared_corpus(label: &str) -> Corpus {
    Corpus::write(
        label,
        &catalog_document(
            4,
            &[
                ("zeta-game", "Zeta Game", &["steam/900001"]),
                ("alpha-game", "Alpha Game", &["steam/900001", "epic/77"]),
                ("beta-game", "Beta Game", &["local/Some.Dir"]),
            ],
        ),
    )
}

/// Starts the accept loop over a root-mode instance on `corpus`, on an ephemeral
/// loopback port, with the admin face enabled the way a loopback bind enables it.
fn start(corpus: &Corpus, token: Option<&str>) -> SocketAddr {
    start_with(corpus, Admin::new(true, token.map(str::to_string)))
}

/// The same accept loop over an admin posture the caller names. `026-C5`'s disabled
/// case comes from a routable bind with no token, which a test may not bind here;
/// enablement is the state that bind produces, so this seam produces the state.
fn start_with(corpus: &Corpus, admin: Admin) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener binds");
    let address = listener.local_addr().expect("the bound address");
    let state = Arc::new(State::root(corpus.path.clone()).expect("the corpus loads"));
    let admin = Arc::new(admin);
    thread::spawn(move || {
        let _outcome: std::io::Result<()> = serve::serve(listener, state, admin);
    });
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_ok() {
            return address;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "nothing bound {address}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn framed(method: &str, target: &str, extra: &[String], body: &str) -> String {
    let mut text = format!("{method} {target} HTTP/1.1\r\nHost: 127.0.0.1\r\n");
    for line in extra {
        text.push_str(line);
        text.push_str("\r\n");
    }
    text.push_str(&format!("Content-Length: {}\r\n\r\n{body}", body.len()));
    text
}

/// One connection, one request, read to the close the server issues (A23).
fn exchange(address: SocketAddr, raw: &str) -> Vec<u8> {
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(5)).expect("the listener accepts");
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream
        .write_all(raw.as_bytes())
        .expect("the request goes out");
    stream.flush().ok();
    let mut answer = Vec::new();
    let mut window = [0u8; 4096];
    loop {
        match stream.read(&mut window) {
            Ok(0) | Err(_) => break,
            Ok(read) => answer.extend_from_slice(&window[..read]),
        }
    }
    answer
}

fn status_line(answer: &[u8]) -> String {
    String::from_utf8_lossy(answer)
        .lines()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn status_code(answer: &[u8]) -> Option<u16> {
    status_line(answer).split_whitespace().nth(1)?.parse().ok()
}

fn header_values(answer: &[u8], name: &str) -> Vec<String> {
    let head = String::from_utf8_lossy(answer)
        .split("\r\n\r\n")
        .next()
        .unwrap_or_default()
        .to_string();
    head.lines()
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .filter(|(header, _)| header.trim().eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim().to_string())
        .collect()
}

fn body(answer: &[u8]) -> Vec<u8> {
    for window in answer.windows(4) {
        if window == b"\r\n\r\n" {
            let offset = window.as_ptr() as usize - answer.as_ptr() as usize;
            return answer[offset + 4..].to_vec();
        }
    }
    Vec::new()
}

fn json(answer: &[u8]) -> serde_json::Value {
    let text = String::from_utf8_lossy(&body(answer)).into_owned();
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("the body is not JSON ({error}): {text:?}"))
}

/// The candidate-list GET, with the key escaped by this module's own rule (A19).
fn candidates(address: SocketAddr, key: &str) -> Vec<u8> {
    let mut encoded = String::new();
    for byte in key.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    exchange(
        address,
        &framed("GET", &format!("/v1/candidates?slug={encoded}"), &[], ""),
    )
}

#[test]
fn the_candidate_answer_is_the_ordered_projection_in_the_pinned_byte_shape() {
    let corpus = shared_corpus("join");
    let address = start(&corpus, None);
    let answer = candidates(address, "steam/900001");
    assert_eq!(status_code(&answer), Some(200), "{}", status_line(&answer));

    let lengths = header_values(&answer, "Content-Length");
    assert_eq!(
        lengths.len(),
        1,
        "exactly one `Content-Length`: {:?}",
        header_values(&answer, "content-length")
    );
    assert_eq!(
        lengths[0].parse::<usize>().expect("a decimal length"),
        body(&answer).len(),
        "the answer promises exactly the body it sends"
    );
    assert!(
        header_values(&answer, "Transfer-Encoding").is_empty(),
        "the answer never uses a chunked coding (A1)"
    );

    let candidates = json(&answer).as_array().expect("a candidate array").clone();
    let ids: Vec<&str> = candidates
        .iter()
        .map(|candidate| candidate["game_id"].as_str().expect("a game_id"))
        .collect();
    assert_eq!(
        ids,
        vec!["alpha-game", "zeta-game"],
        "the join is the postings in `game_id` byte order"
    );
    for candidate in &candidates {
        let object = candidate.as_object().expect("an object");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["game_id", "name"],
            "`{{game_id, name}}` is the whole projection (`026-C2`): {object:?}"
        );
    }
}

#[test]
fn an_empty_join_is_200_with_the_empty_list() {
    let corpus = shared_corpus("empty-join");
    let address = start(&corpus, None);
    let answer = candidates(address, "steam/999999");
    assert_eq!(
        status_code(&answer),
        Some(200),
        "a root-mode miss is `200 []`, never the caching `503`"
    );
    assert_eq!(json(&answer), serde_json::json!([]));
}

#[test]
fn an_unadmissible_or_absent_key_and_an_unknown_route_are_the_named_404() {
    let corpus = shared_corpus("not-found");
    let address = start(&corpus, None);
    for (label, target) in [
        ("unknown route", "/v1/admin/catalog"),
        ("bulk-shaped route", "/v1/candidates"),
        ("no slug", "/v1/candidates"),
        ("empty slug", "/v1/candidates?slug="),
        ("dot-dot", "/v1/candidates?slug=local%2F%2e%2e%2Fetc"),
        ("control byte", "/v1/candidates?slug=steam%2F%0a730"),
        (
            "second separator",
            "/v1/candidates?slug=steam%2F900001%2Feu",
        ),
        ("unknown source", "/v1/candidates?slug=gog%2F900001"),
    ] {
        let answer = exchange(address, &framed("GET", target, &[], ""));
        assert_eq!(
            status_code(&answer),
            Some(404),
            "`{label}` ({target}) is the named `404`: {}",
            status_line(&answer)
        );
    }
}

#[test]
fn a_dotted_external_id_is_admitted() {
    let corpus = shared_corpus("dotted");
    let address = start(&corpus, None);
    let answer = candidates(address, "local/Some.Dir");
    assert_eq!(
        status_code(&answer),
        Some(200),
        "a dotted `local/<dir>` key is admitted (A4): {}",
        status_line(&answer)
    );
    assert_eq!(json(&answer)[0]["game_id"], serde_json::json!("beta-game"));
}

#[test]
fn a_method_a_route_does_not_carry_is_a_405_naming_the_methods_it_does() {
    let corpus = shared_corpus("method");
    let address = start(&corpus, None);
    let answer = exchange(address, &framed("POST", "/v1/candidates", &[], "{}"));
    assert_eq!(status_code(&answer), Some(405), "{}", status_line(&answer));
    let allow = header_values(&answer, "Allow");
    assert_eq!(allow.len(), 1, "one `Allow` line: {allow:?}");
    assert!(
        allow[0].contains("GET"),
        "`405` names the method the route carries: {allow:?}"
    );
}

#[test]
fn health_is_always_a_200_and_reports_the_status_document() {
    let corpus = shared_corpus("health");
    let address = start(&corpus, None);
    let answer = exchange(address, &framed("GET", "/v1/health", &[], ""));
    assert_eq!(status_code(&answer), Some(200), "{}", status_line(&answer));
    let document = json(&answer).as_object().expect("an object").clone();
    for field in [
        "status",
        "mode",
        "upstream",
        "local_entries",
        "cached_slugs",
        "last_upstream_fetch",
    ] {
        assert!(
            document.contains_key(field),
            "`{field}` is missing: {document:?}"
        );
    }
    assert_eq!(document["mode"].as_str(), Some("root"));
    assert_eq!(document["upstream"].as_str(), Some("none"));
    assert_eq!(document["local_entries"].as_u64(), Some(3));
    assert_eq!(document["cached_slugs"].as_u64(), Some(0));
    assert_eq!(document["last_upstream_fetch"].as_u64(), Some(0));
}

#[test]
fn one_connection_answers_exactly_one_request() {
    let corpus = shared_corpus("one-answer");
    let address = start(&corpus, None);
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(5)).expect("connects");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("a read timeout");
    stream
        .write_all(framed("GET", "/v1/health", &[], "").as_bytes())
        .expect("the first request goes out");
    stream.flush().ok();
    let mut first = Vec::new();
    stream
        .read_to_end(&mut first)
        .expect("the answer arrives before the close");
    assert_eq!(status_code(&first), Some(200), "{}", status_line(&first));
    // The connection is closed after one answer (A23): a second request on the same
    // socket cannot be answered.
    let second = stream.write_all(framed("GET", "/v1/health", &[], "").as_bytes());
    let mut rest = Vec::new();
    let _ = stream.read_to_end(&mut rest);
    assert!(
        rest.is_empty() || second.is_err(),
        "a second request on one connection gets no second answer"
    );
}

#[test]
fn the_admin_face_mutates_the_document_bumps_revision_and_persists() {
    let corpus = shared_corpus("admin");
    let before = corpus.text();
    let address = start(&corpus, None);

    let stats = exchange(address, &framed("GET", "/v1/admin/stats", &[], ""));
    assert_eq!(status_code(&stats), Some(200), "{}", status_line(&stats));
    let document = json(&stats).as_object().expect("an object").clone();
    assert_eq!(
        document.keys().map(String::as_str).collect::<Vec<_>>(),
        {
            let mut names: Vec<&str> = vec!["games", "revision", "source_keys"];
            names.sort_unstable();
            names
        },
        "`stats` is the three integer counts: {document:?}"
    );
    assert_eq!(document["revision"].as_u64(), Some(4));

    let record = serde_json::to_string(&serde_json::json!({
        "game_id": "added-game", "name": "Added Game", "source_keys": ["steam/900500"],
    }))
    .expect("a record serialises");
    let added = exchange(address, &framed("POST", "/v1/admin/games", &[], &record));
    assert_eq!(
        status_code(&added),
        Some(200),
        "an upsert answers with the new counts: {}",
        status_line(&added)
    );
    assert_eq!(
        json(&added)["revision"].as_u64(),
        Some(5),
        "`revision` bumped"
    );
    let persisted = corpus.text();
    assert_ne!(persisted, before, "the accepted mutation reached the file");
    assert!(
        persisted.contains("added-game"),
        "the record is in the file"
    );
    let answered = candidates(address, "steam/900500");
    assert_eq!(status_code(&answered), Some(200));
    assert_eq!(
        json(&answered)[0]["game_id"],
        serde_json::json!("added-game")
    );

    let deleted = exchange(
        address,
        &framed("DELETE", "/v1/admin/games/added-game", &[], ""),
    );
    assert_eq!(
        status_code(&deleted),
        Some(200),
        "a delete answers with the new counts: {}",
        status_line(&deleted)
    );
    assert_eq!(json(&deleted)["revision"].as_u64(), Some(6));
    assert!(
        !corpus.text().contains("added-game"),
        "the delete reached the file"
    );
    let _ = Path::new(&corpus.path);
}

#[test]
fn a_record_the_contract_refuses_is_a_422_that_changes_nothing() {
    let corpus = shared_corpus("admin-refusal");
    let before = corpus.text();
    let address = start(&corpus, None);
    for (label, record) in [
        (
            "unknown source token",
            r#"{"game_id":"bad-game","name":"Bad","source_keys":["gog/1"]}"#,
        ),
        (
            "grammar-invalid id",
            r#"{"game_id":"Bad-Game","name":"Bad","source_keys":["steam/1"]}"#,
        ),
        (
            "repeated key",
            r#"{"game_id":"bad-game","name":"Bad","source_keys":["steam/1","steam/1"]}"#,
        ),
        (
            "missing name",
            r#"{"game_id":"bad-game","source_keys":["steam/1"]}"#,
        ),
    ] {
        let answer = exchange(address, &framed("POST", "/v1/admin/games", &[], record));
        assert_eq!(
            status_code(&answer),
            Some(422),
            "`{label}` is a named `422`: {}",
            status_line(&answer)
        );
    }
    assert_eq!(corpus.text(), before, "a refused upsert rewrote nothing");
    let stats = exchange(address, &framed("GET", "/v1/admin/stats", &[], ""));
    assert_eq!(
        json(&stats)["revision"].as_u64(),
        Some(4),
        "no phantom bump"
    );
}

#[test]
fn a_configured_token_is_required_everywhere_and_never_echoed() {
    const TOKEN: &str = "a-configured-token-of-eighteen";
    let corpus = shared_corpus("admin-token");
    let address = start(&corpus, Some(TOKEN));

    let missing = exchange(address, &framed("GET", "/v1/admin/stats", &[], ""));
    assert_eq!(
        status_code(&missing),
        Some(401),
        "{}",
        status_line(&missing)
    );
    assert!(
        !missing
            .windows(TOKEN.len())
            .any(|window| window == TOKEN.as_bytes()),
        "a refusal never echoes the configured token"
    );
    let challenge = header_values(&missing, "WWW-Authenticate");
    assert_eq!(challenge, vec!["Bearer".to_string()], "the scheme is named");

    let wrong = exchange(
        address,
        &framed(
            "GET",
            "/v1/admin/stats",
            &[format!("Authorization: Bearer not-{TOKEN}")],
            "",
        ),
    );
    assert_eq!(
        status_code(&wrong),
        Some(401),
        "a wrong token is still `401`"
    );

    let right = exchange(
        address,
        &framed(
            "GET",
            "/v1/admin/stats",
            &[format!("Authorization: Bearer {TOKEN}")],
            "",
        ),
    );
    assert_eq!(
        status_code(&right),
        Some(200),
        "the configured token admits the loopback admin face"
    );
}

#[test]
fn a_framing_the_server_does_not_understand_is_refused_before_any_mutation() {
    let corpus = shared_corpus("framing");
    let before = corpus.text();
    let address = start(&corpus, None);

    let chunked = exchange(
        address,
        "POST /v1/admin/games HTTP/1.1\r\nHost: 127.0.0.1\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n",
    );
    assert_eq!(
        status_code(&chunked),
        Some(400),
        "`Transfer-Encoding` is refused at the door: {}",
        status_line(&chunked)
    );

    let over_claimed = exchange(
        address,
        &format!(
            "POST /v1/admin/games HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\n\r\n{{}}",
            crate::constants::MAX_REQUEST_BODY_OCTETS + 1
        ),
    );
    assert_eq!(
        status_code(&over_claimed),
        Some(400),
        "an over-budget body is refused on its declaration: {}",
        status_line(&over_claimed)
    );
    assert_eq!(
        corpus.text(),
        before,
        "a refused request changed nothing on disk"
    );
}

#[test]
fn the_seed_corpus_serves_its_own_keys() {
    let seed = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|apps| apps.parent())
        .expect("apps/registry sits at <root>/apps/registry")
        .join("data")
        .join("catalog")
        .join("v1")
        .join("catalog.json");
    let corpus = Corpus::write(
        "seed",
        &std::fs::read_to_string(&seed).expect("the seed is readable"),
    );
    let address = start(&corpus, None);
    let answer = candidates(address, "steam/730");
    assert_eq!(status_code(&answer), Some(200), "{}", status_line(&answer));
    assert!(
        json(&answer)
            .as_array()
            .expect("an array")
            .iter()
            .any(|candidate| candidate["game_id"] == serde_json::json!("counter-strike-2")),
        "the seed joins a curated id: {}",
        String::from_utf8_lossy(&body(&answer))
    );
}

#[test]
fn a_wide_join_is_answered_inside_the_body_cap_the_pinned_reader_refuses() {
    // `MAX_CANDIDATES` records on every field bound sharing one key: the count
    // window alone serializes to ≈72,000 octets, a body the shipped client's reader
    // (A1) refuses as over budget. `026-C1`'s byte-identical face has to answer
    // inside that cap, by the same stable window the count cap uses.
    let games: Vec<serde_json::Value> = (0..crate::constants::MAX_CANDIDATES)
        .map(|index| {
            let letters: String = [
                (b'a' + (index / 26) as u8) as char,
                (b'a' + (index % 26) as u8) as char,
            ]
            .into_iter()
            .collect();
            serde_json::json!({
                "game_id": format!("{}{}", "g".repeat(125), letters),
                "name": "n".repeat(128),
                "source_keys": ["steam/900100"],
            })
        })
        .collect();
    let document = serde_json::to_string(&serde_json::json!({
        "version": 1, "revision": 1, "games": games,
    }))
    .expect("a max-bound corpus serialises");
    let corpus = Corpus::write("wide-answer", &document);
    let address = start(&corpus, None);

    let first = candidates(address, "steam/900100");
    assert_eq!(status_code(&first), Some(200), "{}", status_line(&first));
    let sent = body(&first);
    assert!(
        sent.len() <= crate::constants::MAX_RESPONSE_OCTETS,
        "the answer is inside the body cap the client pins: {} octets",
        sent.len()
    );
    let lengths = header_values(&first, "Content-Length");
    assert_eq!(lengths.len(), 1, "exactly one `Content-Length`");
    assert_eq!(
        lengths[0].parse::<usize>().expect("a decimal length"),
        sent.len(),
        "the answer promises what it sends"
    );
    let kept = json(&first).as_array().expect("a candidate array").len();
    assert!(
        kept > 0 && kept < crate::constants::MAX_CANDIDATES,
        "the octet cap binds without emptying a non-empty join: {kept} candidates"
    );
    let again = candidates(address, "steam/900100");
    assert_eq!(
        body(&again),
        sent,
        "the trimmed window is stable across requests"
    );
}

#[test]
fn a_disabled_admin_face_publishes_no_allow_list_for_any_method() {
    // `026-C5`: disabled is a flat `404`. A `405` with its `Allow` list would tell a
    // caller which methods the face this instance says it lacks really answers.
    let corpus = shared_corpus("admin-disabled-wire");
    let address = start_with(&corpus, Admin::new(false, None));
    for (method, target) in [
        ("GET", "/v1/admin/stats"),
        ("PUT", "/v1/admin/stats"),
        ("POST", "/v1/admin/games"),
        ("GET", "/v1/admin/games"),
        ("DELETE", "/v1/admin/games/alpha-game"),
        ("GET", "/v1/admin/catalog"),
    ] {
        let answer = exchange(address, &framed(method, target, &[], ""));
        assert_eq!(
            status_code(&answer),
            Some(404),
            "`{method} {target}` on a disabled face: {}",
            status_line(&answer)
        );
        assert!(
            header_values(&answer, "Allow").is_empty(),
            "`{method} {target}` leaks an `Allow` list: {:?}",
            header_values(&answer, "Allow")
        );
    }
    let serving = candidates(address, "steam/900001");
    assert_eq!(
        status_code(&serving),
        Some(200),
        "the serving face is unaffected: {}",
        status_line(&serving)
    );
}

#[test]
fn a_carried_admin_request_that_names_an_origin_is_refused_and_writes_nothing() {
    let corpus = shared_corpus("admin-origin");
    let address = start(&corpus, None);
    let before = corpus.text();
    let record = r#"{"game_id":"new-game","name":"New Game","source_keys":["steam/900777"]}"#;

    let browser = exchange(
        address,
        &framed(
            "POST",
            "/v1/admin/games",
            &["Origin: http://evil.example".to_string()],
            record,
        ),
    );
    assert_eq!(
        status_code(&browser),
        Some(400),
        "a request that names an origin is not answered: {}",
        status_line(&browser)
    );
    assert_eq!(corpus.text(), before, "the refused write mutates nothing");

    let tool = exchange(address, &framed("POST", "/v1/admin/games", &[], record));
    assert_eq!(
        status_code(&tool),
        Some(200),
        "a client that sends no `Origin` is answered: {}",
        status_line(&tool)
    );
    assert_ne!(corpus.text(), before, "and it is the mutation it names");
}

#[test]
fn an_upsert_declared_as_a_form_media_type_is_refused_and_writes_nothing() {
    // The three types an HTML form produces are the requests a browser sends
    // cross-origin without a preflight, and this face parses a `026-C3` record and
    // nothing else. The refusal is a header-block decision, so the record body a
    // refused caller declared is never read; a caller that declares JSON — or sends no
    // `Content-Type` at all — is answered.
    let corpus = shared_corpus("admin-media-type");
    let address = start(&corpus, None);
    let before = corpus.text();
    let record = r#"{"game_id":"new-game","name":"New Game","source_keys":["steam/900777"]}"#;

    for declared in [
        "Content-Type: text/plain",
        "Content-Type: application/x-www-form-urlencoded",
        "Content-Type: multipart/form-data; boundary=-abc",
        "Content-Type: application/xml",
    ] {
        let refused = exchange(
            address,
            &framed("POST", "/v1/admin/games", &[declared.to_string()], record),
        );
        assert_eq!(
            status_code(&refused),
            Some(415),
            "`{declared}` is not a body this face reads: {}",
            status_line(&refused)
        );
        assert_eq!(
            json(&refused)["error"],
            serde_json::json!("unsupported_media_type"),
            "the refusal names itself"
        );
        assert_eq!(corpus.text(), before, "`{declared}` mutates nothing");
    }

    let declared = exchange(
        address,
        &framed(
            "POST",
            "/v1/admin/games",
            &["Content-Type: application/json; charset=utf-8".to_string()],
            record,
        ),
    );
    assert_eq!(
        status_code(&declared),
        Some(200),
        "a body declared as JSON is answered: {}",
        status_line(&declared)
    );
    assert_ne!(corpus.text(), before, "and it is the mutation it names");
}

#[test]
fn an_unauthenticated_admin_request_is_refused_without_its_declared_body() {
    // The credential is a header, so the refusal is owed before the body arrives:
    // a peer that declares megabytes and sends nothing still gets its `401`, and
    // the instance never buffers what it declared.
    let corpus = shared_corpus("admin-body-gate");
    let address = start(&corpus, Some("a-token-of-eighteen-chars"));
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(5)).expect("the listener accepts");
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .expect("a read timeout");
    stream
        .write_all(
            b"POST /v1/admin/games HTTP/1.1\r\nHost: 127.0.0.1\r\n\
              Content-Length: 5242880\r\n\r\n",
        )
        .expect("the header block goes out");
    stream.flush().ok();
    // No body octets are ever sent.
    let mut answer = Vec::new();
    stream
        .read_to_end(&mut answer)
        .expect("the refusal arrives without the body");
    assert_eq!(status_code(&answer), Some(401), "{}", status_line(&answer));
    assert!(!header_values(&answer, "WWW-Authenticate").is_empty());
    assert!(
        corpus.text().contains("alpha-game"),
        "the corpus is untouched by a refused request"
    );
}

#[test]
fn a_root_whose_corpus_is_absent_refuses_to_start_rather_than_serving_nothing() {
    // Absent-is-empty is `026-C7`'s rule for the local-entries document and the
    // cache. A root's `--corpus` is the one content argument it has, so its absence
    // is the named startup refusal every other corpus fault gets.
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let missing = std::env::temp_dir().join(format!("bindforge-no-such-corpus-{nonce}.json"));
    let error = State::root(missing).expect_err("an absent corpus refuses to start a root");
    let line = error.line();
    assert_eq!(line.lines().count(), 1, "one named line: {line}");
    assert!(line.contains("not there"), "{line}");
}
