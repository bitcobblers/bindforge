//! Shared harness for the `apps/registry` suites (design 026 / ADR-026).
//!
//! This is not a test target of its own: cargo compiles `tests/*.rs`, and each
//! suite pulls this module in with `mod common;`.
//!
//! The registry is a binary (`026-C1`: "one binary, `registry`"), so every suite
//! here drives it as a subprocess bound to an ephemeral `127.0.0.1` port and
//! speaks HTTP/1.1 over a raw loopback socket — the framing itself is under
//! test (`026-C2`, A1), so no client library sits between the test and the
//! server. The only upstream a caching instance is ever pointed at is another
//! loopback listener living in the test process, or a bound-and-closed loopback
//! port: **no test dials or resolves a non-loopback host** (the invariant the
//! shipped gate suite keeps and ADR-026 re-asserts, A15). Documents are
//! relocated per run through `XDG_DATA_HOME`/`HOME`/`APPDATA` in the child
//! environment — the only relocation seam Test strategy #13 names — and every
//! run gets a temp directory that is removed when it goes out of scope.
//!
//! # Why the module-level `allow(dead_code)` stays
//!
//! Each test target compiles this module on its own and uses a different subset
//! of these helpers, so every helper is dead code in some target.

#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Wall-clock seconds since the Unix epoch — the clock `026-B3`'s TTL arithmetic
/// uses (Chosen design: Budgets, "TTL arithmetic uses the wall clock").
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("a post-epoch clock")
        .as_secs()
}

fn unique_temp(label: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "bindforge-registry-{label}-{}-{nonce}-{nanos}",
        std::process::id()
    ))
}

/// A unique temp directory, removed when it goes out of scope.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> Self {
        let path = unique_temp(label);
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|err| panic!("creating {}: {err}", path.display()));
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn join(&self, relative: &str) -> PathBuf {
        self.path.join(relative)
    }

    /// Writes one file (parents created), returning its path.
    pub fn write(&self, relative: &str, contents: &[u8]) -> PathBuf {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("creating {}: {err}", parent.display()));
        }
        std::fs::write(&path, contents)
            .unwrap_or_else(|err| panic!("writing {}: {err}", path.display()));
        path
    }

    pub fn read(&self, relative: &str) -> Option<Vec<u8>> {
        std::fs::read(self.path.join(relative)).ok()
    }

    pub fn read_to_string(&self, relative: &str) -> Option<String> {
        std::fs::read_to_string(self.path.join(relative)).ok()
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.path.join(relative).exists()
    }

    /// Every regular file below `relative`, recursively — the seam for "no tmp
    /// remains" and "nothing else was written" (A7's persist discipline).
    pub fn files_below(&self, relative: &str) -> Vec<PathBuf> {
        fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, files);
                } else {
                    files.push(path);
                }
            }
        }
        let mut files = Vec::new();
        walk(&self.path.join(relative), &mut files);
        files.sort();
        files
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// One scratch Steam root: `<root>/steamapps/appmanifest_<appid>.acf`, removed on
/// drop. Only the #16/#17 end-to-end suites need it: they point the shipped
/// `discovery` client at a registry.
pub struct SteamRoot {
    path: PathBuf,
}

impl SteamRoot {
    pub fn new(label: &str) -> Self {
        let path = unique_temp(&format!("steam-{label}"));
        std::fs::create_dir_all(path.join("steamapps"))
            .unwrap_or_else(|err| panic!("creating {}: {err}", path.display()));
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// One manifest whose `AppState.appid` is `appid`.
    pub fn acf(&self, appid: &str, name: Option<&str>, installdir: &str) {
        let mut text =
            format!("\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"Universe\"\t\t\"1\"\n");
        if let Some(name) = name {
            text.push_str(&format!("\t\"name\"\t\t\"{name}\"\n"));
        }
        text.push_str(&format!(
            "\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"{installdir}\"\n\t\
             \"SizeOnDisk\"\t\t\"1000000\"\n}}\n"
        ));
        std::fs::write(
            self.path
                .join("steamapps")
                .join(format!("appmanifest_{appid}.acf")),
            text,
        )
        .expect("temp appmanifest");
    }
}

impl Drop for SteamRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// The repository root, from this package's manifest directory
/// (`apps/registry`), for the suites that read `schemas/` and `data/`.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|apps| apps.parent())
        .expect("apps/registry sits at <root>/apps/registry")
        .to_path_buf()
}

/// The sibling `discovery-cli` binary the #16/#17 end-to-end suites drive against
/// a registry (`cargo test --workspace` builds both binaries into one profile
/// directory). The profile directory is `deps/..` in the plain layout but can
/// sit further up in nested target layouts, so the walk goes up as many levels
/// as it takes to find it.
pub fn discovery_cli_bin() -> PathBuf {
    let exe = std::env::current_exe().expect("the test binary has a path");
    let wanted = format!("discovery-cli{}", std::env::consts::EXE_SUFFIX);
    let mut directory = exe.parent();
    while let Some(dir) = directory {
        let candidate = dir.join(&wanted);
        if candidate.is_file() {
            return candidate;
        }
        directory = dir.parent();
    }
    panic!(
        "`{wanted}` was not found beside the built test binary: the #16/#17 \
         end-to-end suites need the shipped CLI in the profile directory beside \
         the registry binary, so run them under `cargo test --workspace`"
    );
}

// ---------------------------------------------------------------------------
// Request framing and response reading — the assertions read raw bytes, never a
// client library, because `026-C2`'s contract is the byte shape (A1).
// ---------------------------------------------------------------------------

/// Builds one HTTP/1.1 request: request line, `Host`/`Accept`/`Connection`
/// defaults, the caller's extra header lines verbatim, a decimal
/// `Content-Length`, then the body.
pub fn build_request(
    method: &str,
    target: &str,
    host: &str,
    extra: &[&str],
    body: &str,
) -> Vec<u8> {
    let mut text = format!("{method} {target} HTTP/1.1\r\nHost: {host}\r\n");
    text.push_str("Accept: application/json\r\nConnection: close\r\n");
    for line in extra {
        text.push_str(line);
        text.push_str("\r\n");
    }
    text.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    text.push_str(body);
    text.into_bytes()
}

/// A request with no `Content-Length` at all — the framing refusal of #11.
pub fn build_request_without_length(method: &str, target: &str, host: &str, body: &str) -> Vec<u8> {
    let text = format!(
        "{method} {target} HTTP/1.1\r\nHost: {host}\r\nAccept: application/json\r\n\
         Connection: close\r\n\r\n{body}"
    );
    text.into_bytes()
}

/// A request naming a caller-chosen `Content-Length` value — the seam for
/// "an over-`MAX_REQUEST_BODY_OCTETS` body is refused" without sending it.
pub fn build_request_with_length_claim(
    method: &str,
    target: &str,
    host: &str,
    claim: &str,
    body: &str,
) -> Vec<u8> {
    let text = format!(
        "{method} {target} HTTP/1.1\r\nHost: {host}\r\nAccept: application/json\r\n\
         Connection: close\r\nContent-Length: {claim}\r\n\r\n{body}"
    );
    text.into_bytes()
}

pub fn status_line(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);
    text.lines().next().unwrap_or_default().to_string()
}

pub fn status_code(raw: &[u8]) -> Option<u16> {
    let line = status_line(raw);
    let mut fields = line.split_whitespace();
    let version = fields.next()?;
    if !version.starts_with("HTTP/") {
        return None;
    }
    fields.next()?.parse::<u16>().ok()
}

/// Every header line of a response as `(lowercased name, value)`, in order.
pub fn headers(raw: &[u8]) -> Vec<(String, String)> {
    let text = String::from_utf8_lossy(raw);
    let Some(head) = text.split("\r\n\r\n").next() else {
        return Vec::new();
    };
    head.lines()
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        .collect()
}

pub fn header_values(raw: &[u8], name: &str) -> Vec<String> {
    let wanted = name.to_ascii_lowercase();
    headers(raw)
        .into_iter()
        .filter(|(header, _)| header == &wanted)
        .map(|(_, value)| value)
        .collect()
}

pub fn header_names(raw: &[u8]) -> Vec<String> {
    headers(raw).into_iter().map(|(name, _)| name).collect()
}

pub fn body_of(raw: &[u8]) -> Vec<u8> {
    for window in raw.windows(4) {
        if window == b"\r\n\r\n" {
            let offset = window.as_ptr() as usize - raw.as_ptr() as usize;
            return raw[offset + 4..].to_vec();
        }
    }
    Vec::new()
}

pub fn body_text(raw: &[u8]) -> String {
    String::from_utf8_lossy(&body_of(raw)).into_owned()
}

/// The body parsed as JSON — every suite that reads a document or an answer goes
/// through here, so a malformed answer fails the test loudly.
pub fn json_body(raw: &[u8]) -> serde_json::Value {
    let text = body_text(raw);
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("body is not JSON ({err}): {text:?}"))
}

/// The `game_id`s of one `[{"game_id":…,"name":…}]` candidate answer, in order.
pub fn game_ids(raw: &[u8]) -> Vec<String> {
    json_body(raw)
        .as_array()
        .expect("a candidate answer is a JSON array")
        .iter()
        .map(|candidate| {
            candidate["game_id"]
                .as_str()
                .expect("each candidate carries a game_id string")
                .to_string()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Document builders — the shapes `026-C3` and `026-C7` fix.
// ---------------------------------------------------------------------------

/// A `026-C3` catalog document: `{version: 1, revision, games: [{game_id, name,
/// source_keys}]}`.
pub fn corpus_doc(revision: i64, games: &[(&str, &str, &[&str])]) -> String {
    let records: Vec<serde_json::Value> = games
        .iter()
        .map(|(game_id, name, source_keys)| {
            serde_json::json!({
                "game_id": game_id,
                "name": name,
                "source_keys": source_keys,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "version": 1,
        "revision": revision,
        "games": records,
    }))
    .expect("a catalog document serialises")
}

/// One `026-C3` record as an admin upsert body.
pub fn record_body(game_id: &str, name: &str, source_keys: &[&str]) -> String {
    serde_json::to_string(&serde_json::json!({
        "game_id": game_id,
        "name": name,
        "source_keys": source_keys,
    }))
    .expect("a record serialises")
}

/// One candidate's `{game_id, name}` pair, as a suite spells it for a builder.
type Candidate<'a> = (&'a str, &'a str);

/// One cache-document entry a typed builder can express: `(slug, candidates,
/// fetched_at)`.
type CacheEntry<'a> = (&'a str, &'a [Candidate<'a>], u64);

/// A `026-C7` cache document: `{version, entries: [{slug, candidates,
/// fetched_at}]}`.
pub fn cache_doc(entries: &[CacheEntry<'_>]) -> String {
    cache_doc_json(&cache_entries_json(entries))
}

fn cache_entries_json(entries: &[CacheEntry<'_>]) -> Vec<serde_json::Value> {
    entries
        .iter()
        .map(|(slug, candidates, fetched_at)| {
            let projected: Vec<serde_json::Value> = candidates
                .iter()
                .map(|(game_id, name)| serde_json::json!({"game_id": game_id, "name": name}))
                .collect();
            serde_json::json!({
                "slug": slug,
                "candidates": projected,
                "fetched_at": fetched_at,
            })
        })
        .collect()
}

pub fn cache_doc_json(entries: &[serde_json::Value]) -> String {
    serde_json::to_string_pretty(&serde_json::json!({"version": 1, "entries": entries}))
        .expect("a cache document serialises")
}

/// One cache entry as JSON, for the refusals the typed builder cannot express.
pub fn cache_entry(
    slug: &str,
    candidates: &[(&str, &str)],
    fetched_at: serde_json::Value,
) -> serde_json::Value {
    let projected: Vec<serde_json::Value> = candidates
        .iter()
        .map(|(game_id, name)| serde_json::json!({"game_id": game_id, "name": name}))
        .collect();
    serde_json::json!({"slug": slug, "candidates": projected, "fetched_at": fetched_at})
}

/// A cache document from already-built entries, for the shape refusals.
pub fn cache_doc_value(version: serde_json::Value, entries: Vec<serde_json::Value>) -> String {
    serde_json::to_string_pretty(&serde_json::json!({"version": version, "entries": entries}))
        .expect("a cache document serialises")
}

/// The `<data>/bindforge/registry/<file>` spelling of a registry document
/// (`026-C7`).
pub fn registry_document(data_home: &Path, file: &str) -> PathBuf {
    data_home.join("bindforge").join("registry").join(file)
}

// ---------------------------------------------------------------------------
// The registry subprocess.
// ---------------------------------------------------------------------------

/// One `registry serve` subprocess on an ephemeral loopback port, with both
/// output streams drained so the server never blocks on a full pipe.
pub struct Registry {
    child: Child,
    port: u16,
    stdout: Arc<Mutex<Vec<String>>>,
    stderr: Arc<Mutex<Vec<String>>>,
}

impl Registry {
    /// `registry serve --root --corpus <path>` over a `026-C3` corpus.
    pub fn root(label: &str, corpus: &Path, extra: &[&str]) -> Self {
        let args = vec![
            "serve".to_string(),
            "--root".to_string(),
            "--corpus".to_string(),
            corpus.display().to_string(),
        ];
        let mut args = args;
        args.extend(extra.iter().map(|arg| arg.to_string()));
        let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
        Self::start(label, &borrowed, &[])
    }

    /// `registry serve --upstream <url>` in caching mode, sandboxed to `data`
    /// through `XDG_DATA_HOME` — the only relocation seam #13 names.
    pub fn caching(label: &str, upstream: &str, data: &TempDir, extra: &[&str]) -> Self {
        let xdg = data
            .path()
            .to_str()
            .expect("a utf-8 temp data directory")
            .to_string();
        let mut args = vec![
            "serve".to_string(),
            "--upstream".to_string(),
            upstream.to_string(),
        ];
        args.extend(extra.iter().map(|arg| arg.to_string()));
        let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
        Self::start(label, &borrowed, &[("XDG_DATA_HOME", xdg.as_str())])
    }

    /// Any `registry` invocation, with the ambient endpoint/token/data variables
    /// removed first so no run inherits a developer's documents or reaches
    /// anything the test did not stand up.
    pub fn start(label: &str, args: &[&str], envs: &[(&str, &str)]) -> Self {
        let _ = label;
        let mut command = Command::new(env!("CARGO_BIN_EXE_registry"));
        command.args(args);
        for name in AMBIENT_ENV_VARS {
            command.env_remove(name);
        }
        for (key, value) in envs {
            command.env(key, value);
        }
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .unwrap_or_else(|err| panic!("failed to execute the registry with {args:?}: {err}"));
        let stdout_pipe = child.stdout.take().expect("piped stdout");
        let stderr_pipe = child.stderr.take().expect("piped stderr");
        let stdout = Arc::new(Mutex::new(Vec::<String>::new()));
        let stderr = Arc::new(Mutex::new(Vec::<String>::new()));
        drain_lines(stdout_pipe, stdout.clone());
        drain_err_lines(stderr_pipe, stderr.clone());

        // `--port` defaults to 0 and "the *printed* value is the contract"
        // (Chosen design: CLI surface), so the port is read from the child's own
        // output before anything is asked of it.
        let deadline = Instant::now() + Duration::from_secs(20);
        let port = loop {
            if let Some(port) = scan_printed_port(&stdout.lock().expect("record")) {
                break port;
            }
            if let Ok(Some(status)) = child.try_wait() {
                panic!(
                    "registry {args:?} exited ({status}) before printing its bound port; \
                     stderr: {}",
                    joined(&stderr.lock().expect("record"))
                );
            }
            if Instant::now() > deadline {
                panic!(
                    "registry {args:?} printed no bound port within 20s; stdout: {}\nstderr: {}",
                    joined(&stdout.lock().expect("record")),
                    joined(&stderr.lock().expect("record"))
                );
            }
            thread::sleep(Duration::from_millis(20));
        };
        let server = Self {
            child,
            port,
            stdout,
            stderr,
        };
        server.wait_for_listener();
        server
    }

    fn wait_for_listener(&self) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if self.try_connection().is_ok() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!(
            "nothing accepted connections on 127.0.0.1:{}; stderr: {}",
            self.port,
            self.stderr_text()
        );
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// The `http://127.0.0.1:<port>` authority of this instance.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn host_header(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    pub fn stdout_lines(&self) -> Vec<String> {
        self.stdout.lock().expect("record").clone()
    }

    pub fn stderr_lines(&self) -> Vec<String> {
        self.stderr.lock().expect("record").clone()
    }

    pub fn stdout_text(&self) -> String {
        joined(&self.stdout.lock().expect("record"))
    }

    pub fn stderr_text(&self) -> String {
        joined(&self.stderr.lock().expect("record"))
    }

    /// A connection the caller frames by hand — the seam for the
    /// one-answer-per-connection pin and the connection-cap pin.
    pub fn try_connection(&self) -> std::io::Result<TcpStream> {
        let address = format!("127.0.0.1:{}", self.port)
            .parse()
            .expect("a loopback address");
        TcpStream::connect_timeout(&address, Duration::from_secs(5))
    }

    pub fn connection(&self) -> TcpStream {
        self.try_connection().expect("the listener accepts")
    }

    /// Sends one framed request and reads the answer to end-of-stream (the server
    /// closes each connection after one answer, A23).
    pub fn exchange(&self, raw: &[u8]) -> Vec<u8> {
        let mut stream = self.connection();
        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        stream
            .write_all(raw)
            .unwrap_or_else(|err| panic!("writing {:?}: {err}", String::from_utf8_lossy(raw)));
        stream.flush().ok();
        read_to_end(&mut stream)
    }

    /// `GET` with the framing defaults.
    pub fn get(&self, target: &str) -> Vec<u8> {
        self.exchange(&build_request("GET", target, &self.host_header(), &[], ""))
    }

    /// A request with caller-supplied header lines and body.
    pub fn request(&self, method: &str, target: &str, extra: &[&str], body: &str) -> Vec<u8> {
        self.exchange(&build_request(
            method,
            target,
            &self.host_header(),
            extra,
            body,
        ))
    }

    /// A bearer-authenticated `GET`.
    pub fn get_with_token(&self, target: &str, token: &str) -> Vec<u8> {
        self.request(
            "GET",
            target,
            &[&format!("Authorization: Bearer {token}")],
            "",
        )
    }

    /// A bearer-authenticated request of any method.
    pub fn with_token(&self, method: &str, target: &str, token: &str, body: &str) -> Vec<u8> {
        self.request(
            method,
            target,
            &[&format!("Authorization: Bearer {token}")],
            body,
        )
    }

    pub fn get_candidates(&self, key: &str) -> Vec<u8> {
        self.get(&slug_query(key))
    }

    pub fn stats(&self, token: Option<&str>) -> Vec<u8> {
        match token {
            Some(token) => self.get_with_token("/v1/admin/stats", token),
            None => self.get("/v1/admin/stats"),
        }
    }

    pub fn health(&self) -> Vec<u8> {
        self.get("/v1/health")
    }

    /// Holds `count` connections open without sending a byte (the
    /// `MAX_CONCURRENT_CONNECTIONS` pin).
    pub fn hold_connections(&self, count: usize) -> Vec<TcpStream> {
        (0..count).map(|_| self.connection()).collect()
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The environment a registry run must never inherit from the developer's shell.
const AMBIENT_ENV_VARS: [&str; 5] = [
    "BINDFORGE_CATALOG_URL",
    "BINDFORGE_REGISTRY_ADMIN_TOKEN",
    "XDG_DATA_HOME",
    "HOME",
    "APPDATA",
];

impl Drop for Registry {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn joined(lines: &[String]) -> String {
    lines.join("\n")
}

/// Reads the assigned port out of whatever the server has printed so far: an
/// `http://<host>:<port>`, a bare `<host>:<port>`, or a bare port.
fn scan_printed_port(lines: &[String]) -> Option<u16> {
    for line in lines {
        for needle in ["127.0.0.1:", "0.0.0.0:", "localhost:", "[::1]:"] {
            let digits: String = match line.split_once(needle) {
                Some((_, rest)) => rest.chars().take_while(char::is_ascii_digit).collect(),
                None => continue,
            };
            if let Ok(port) = digits.parse::<u16>()
                && port != 0
            {
                return Some(port);
            }
        }
        if let Ok(port) = line.trim().parse::<u16>()
            && port != 0
        {
            return Some(port);
        }
    }
    None
}

fn drain_lines(pipe: ChildStdout, sink: Arc<Mutex<Vec<String>>>) {
    thread::spawn(move || {
        let reader = BufReader::new(pipe);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            sink.lock().expect("record").push(line);
        }
    });
}

fn drain_err_lines(pipe: ChildStderr, sink: Arc<Mutex<Vec<String>>>) {
    thread::spawn(move || {
        let reader = BufReader::new(pipe);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            sink.lock().expect("record").push(line);
        }
    });
}

pub fn read_to_end(stream: &mut TcpStream) -> Vec<u8> {
    let mut collected = Vec::new();
    let mut window = [0_u8; 8192];
    loop {
        match stream.read(&mut window) {
            Ok(0) => break,
            Ok(read) => collected.extend_from_slice(&window[..read]),
            Err(_) => break,
        }
    }
    collected
}

/// Runs a `registry serve …` invocation that must **refuse to start**: the
/// process has to exit non-zero on its own (a validating loader refuses its
/// document before it binds, A22's metadata-gated load discipline). A server
/// that comes up anyway is the failure this helper reports.
pub fn expect_refused_startup(args: &[&str], envs: &[(&str, &str)]) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_registry"));
    command.args(args);
    for name in AMBIENT_ENV_VARS {
        command.env_remove(name);
    }
    for (key, value) in envs {
        command.env(key, value);
    }
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .unwrap_or_else(|err| panic!("failed to execute registry with {args:?}: {err}"));
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Ok(Some(status)) = child.try_wait() {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .unwrap_or_else(|err| panic!("registry {args:?} killed: {err}"));
            panic!(
                "registry {args:?} started and kept serving, but the loader must refuse \
                 this document by name; stdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(20));
    };
    let output = child
        .wait_with_output()
        .unwrap_or_else(|err| panic!("registry {args:?} finished unread: {err}"));
    assert!(
        !status.success(),
        "registry {args:?} exited successfully; stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        output,
    }
}

/// One one-shot `registry` run and its captured streams.
pub struct Run {
    pub output: std::process::Output,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    pub fn success(&self) -> bool {
        self.output.status.success()
    }

    pub fn code(&self) -> Option<i32> {
        self.output.status.code()
    }

    pub fn stderr_lines(&self) -> Vec<&str> {
        self.stderr.lines().collect()
    }
}

/// Runs `registry` with `args` in a clean environment: the ambient endpoint,
/// token, and data-directory variables are removed first, then `envs` are set —
/// the seam #13 and #17 use to point a one-shot run at a sandbox, since there is
/// no path knob.
pub fn run_registry(args: &[&str], envs: &[(&str, &str)]) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_registry"));
    command.args(args);
    for name in AMBIENT_ENV_VARS {
        command.env_remove(name);
    }
    for (key, value) in envs {
        command.env(key, value);
    }
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let output = command
        .spawn()
        .and_then(|child| child.wait_with_output())
        .unwrap_or_else(|err| panic!("failed to execute registry with {args:?}: {err}"));
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        output,
    }
}

/// Runs the shipped `discovery` CLI in a clean environment (the #16/#17 suites),
/// with `stdin` piped so a run that has a candidate to confirm answers it exactly the
/// way the shipped CLI suites drive it.
pub fn run_discovery(args: &[&str], envs: &[(&str, &str)], stdin: &str) -> Run {
    let mut command = Command::new(discovery_cli_bin());
    command.args(args);
    for name in ["BINDFORGE_CATALOG_URL", "XDG_DATA_HOME", "HOME", "APPDATA"] {
        command.env_remove(name);
    }
    for (key, value) in envs {
        command.env(key, value);
    }
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .unwrap_or_else(|err| panic!("failed to execute discovery-cli with {args:?}: {err}"));
    {
        let mut pipe = child.stdin.take().expect("piped stdin");
        pipe.write_all(stdin.as_bytes()).expect("stdin is writable");
    }
    let output = child
        .wait_with_output()
        .unwrap_or_else(|err| panic!("failed to execute discovery-cli with {args:?}: {err}"));
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        output,
    }
}

// ---------------------------------------------------------------------------
// The upstream fake — the only "root" a caching instance is ever pointed at.
// ---------------------------------------------------------------------------

/// A loopback `TcpListener` fake for `--upstream`: it answers through the
/// caller's responder and records every request, so "zero upstream lookups"
/// (`026-L5`), the once-per-TTL `/v1/health` probe (`026-C6`), and the
/// percent-encoded request target (A19) are all observable.
pub struct FakeUpstream {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
}

impl FakeUpstream {
    pub fn start(responder: impl Fn(&str) -> Vec<u8> + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .expect("a loopback listener must bind (no test off-host)");
        let address = listener.local_addr().expect("listener address");
        let url = format!("http://{address}");
        listener
            .set_nonblocking(true)
            .expect("nonblocking accept loop");
        let requests = Arc::new(Mutex::new(Vec::<String>::new()));
        let stop = Arc::new(AtomicBool::new(false));
        {
            let requests = requests.clone();
            let stop = stop.clone();
            thread::spawn(move || {
                loop {
                    match listener.accept() {
                        Ok((mut stream, _peer)) => {
                            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                            let mut bytes: Vec<u8> = Vec::new();
                            let mut window = [0_u8; 1024];
                            while !has_header_end(&bytes) && bytes.len() < 65_536 {
                                match stream.read(&mut window) {
                                    Ok(0) | Err(_) => break,
                                    Ok(read) => bytes.extend_from_slice(&window[..read]),
                                }
                            }
                            if !bytes.is_empty() {
                                let request = String::from_utf8_lossy(&bytes).into_owned();
                                requests.lock().expect("record").push(request.clone());
                                let answer = responder(&request);
                                let _ = stream.write_all(&answer);
                            }
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            if stop.load(Ordering::Relaxed) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(_) => break,
                    }
                }
            });
        }
        Self {
            url,
            requests,
            stop,
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    /// The first request line of every recorded request, in arrival order.
    pub fn request_lines(&self) -> Vec<String> {
        self.requests
            .lock()
            .expect("record")
            .iter()
            .map(|request| request.lines().next().unwrap_or_default().to_string())
            .collect()
    }

    pub fn request_count(&self) -> usize {
        self.requests.lock().expect("record").len()
    }

    /// Recorded request lines for one route prefix.
    pub fn lines_for(&self, prefix: &str) -> Vec<String> {
        self.request_lines()
            .into_iter()
            .filter(|line| line.starts_with(prefix))
            .collect()
    }

    /// Every recorded request, headers included — the seam for "no client
    /// identity and never the bulk catalog" (`026-L2`).
    pub fn raw_requests(&self) -> Vec<String> {
        self.requests.lock().expect("record").clone()
    }
}

fn has_header_end(bytes: &[u8]) -> bool {
    bytes.windows(4).any(|window| window == b"\r\n\r\n")
}

impl Drop for FakeUpstream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Ok(address) = self
            .url
            .replace("http://", "")
            .parse::<std::net::SocketAddr>()
        {
            let _ = TcpStream::connect_timeout(&address, Duration::from_millis(200));
        }
    }
}

/// A well-formed `200 OK` JSON response with the given body, closing the
/// connection — the byte shape A1 pins, used by every fake answer.
pub fn ok(body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

/// A `[{"game_id":…,"name":…}]` candidate body in the given order.
pub fn candidates_body(items: &[(&str, &str)]) -> String {
    let entries: Vec<String> = items
        .iter()
        .map(|(game_id, name)| format!("{{\"game_id\":\"{game_id}\",\"name\":\"{name}\"}}"))
        .collect();
    format!("[{}]", entries.join(","))
}

/// The `026-C2` health document a root answers with: the fake upstream's
/// reachability answer.
pub fn root_health() -> String {
    serde_json::to_string(&serde_json::json!({
        "status": "ok",
        "mode": "root",
        "upstream": "none",
        "local_entries": 0,
        "cached_slugs": 0,
        "last_upstream_fetch": 0,
    }))
    .expect("a health document serialises")
}

/// A responder answering any request whose text contains one `table` needle with
/// that needle's response, and everything else with `default`.
pub fn respond_to(
    table: Vec<(String, Vec<u8>)>,
    default: Vec<u8>,
) -> impl Fn(&str) -> Vec<u8> + Send + Sync + 'static {
    move |request| {
        for (needle, response) in &table {
            if request.contains(needle.as_str()) {
                return response.clone();
            }
        }
        default.clone()
    }
}

/// A bound-and-closed loopback port: the "no reachable upstream" of `026-L4`.
pub fn closed_loopback_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind to learn a free port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    port
}

/// The `http://127.0.0.1:<closed port>` spelling of an unreachable upstream.
pub fn unreachable_upstream() -> String {
    format!("http://127.0.0.1:{}", closed_loopback_port())
}

// ---------------------------------------------------------------------------
// Query-string and path-segment encoding for the test side.
// ---------------------------------------------------------------------------

/// Percent-encodes everything outside the RFC 3986 unreserved set — the rule the
/// shipped client applies (A19), spelled here so a test names the wire form it
/// sends independently of the registry's own copy of it.
pub fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(*byte as char);
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

/// `/v1/candidates?slug=<percent-encoded>` for one lookup key.
pub fn slug_query(key: &str) -> String {
    format!("/v1/candidates?slug={}", percent_encode(key))
}

/// `/v1/admin/games/<percent-encoded>` for one `<game_id>` segment.
pub fn game_path(game_id: &str) -> String {
    format!("/v1/admin/games/{}", percent_encode(game_id))
}

// ---------------------------------------------------------------------------
// Owner-only modes, asserted only where unix modes apply (A8).
// ---------------------------------------------------------------------------

/// The permission bits of `path`, for the owner-only assertions this design
/// promises only under `cfg(unix)`.
#[cfg(unix)]
pub fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()))
        .permissions()
        .mode()
        & 0o777
}
