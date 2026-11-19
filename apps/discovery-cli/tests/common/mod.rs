//! Shared harness for the `apps/discovery-cli` suites (ADR-025 shape).
//!
//! This is not a test target of its own: cargo compiles `tests/*.rs`, and each
//! suite pulls this module in with `mod common;`.
//!
//! ADR-025 moves the CLI behind two seams every subprocess run here must drive:
//! the connectivity gate (a `--catalog-url`/`BINDFORGE_CATALOG_URL` pointing at a
//! loopback `TcpListener` fake living in the test process — Chosen design: "the CLI's
//! own tests drive it with an in-process fake and one subprocess test drives the real
//! implementation over a socket") and the per-user match store (sandboxed by giving
//! every subprocess a temp `XDG_DATA_HOME`/`HOME`). Repo practice: no test performs
//! a non-loopback request; temp directories clean up after themselves.
//!
//! # Why the module-level `allow(dead_code)` stays
//!
//! Each test target compiles this module on its own and uses a different subset of
//! these helpers, so every helper is dead code in some target.

#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A unique temp directory, removed when it goes out of scope.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> Self {
        let path = unique_temp(&format!("bindforge-cli-{label}"));
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

    pub fn exists(&self, relative: &str) -> bool {
        self.path.join(relative).exists()
    }

    /// Every regular file below `relative`, recursively (for "no `.tmp` remains").
    pub fn files_below(&self, relative: &str) -> Vec<PathBuf> {
        fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
            let entries = std::fs::read_dir(dir)
                .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()));
            for entry in entries {
                let path = entry.expect("readable entry").path();
                if path.is_dir() {
                    walk(&path, files);
                } else {
                    files.push(path);
                }
            }
        }
        let mut files = Vec::new();
        let dir = self.path.join(relative);
        if dir.is_dir() {
            walk(&dir, &mut files);
        }
        files.sort();
        files
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn unique_temp(label: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nonce}-{nanos}", std::process::id()))
}

/// One scratch Steam root: `<root>/steamapps/appmanifest_<file>.acf`, removed on
/// drop. The CLI is pointed at it with `--steam-root`.
pub struct SteamRoot {
    path: PathBuf,
}

impl SteamRoot {
    pub fn new(label: &str) -> Self {
        let path = unique_temp(&format!("bindforge-cli-{label}"));
        std::fs::create_dir_all(path.join("steamapps"))
            .unwrap_or_else(|err| panic!("creating {}: {err}", path.display()));
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// One manifest whose `AppState.appid` is `appid`; `name` is written only when
    /// given. The file name follows the appid, mirroring a real Steam install.
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

    /// One manifest written verbatim — the seam for hostile bytes (an ESC byte in a
    /// `name`) that the ordinary writer must not have to express.
    pub fn acf_raw(&self, file_id: &str, bytes: &[u8]) {
        std::fs::write(
            self.path
                .join("steamapps")
                .join(format!("appmanifest_{file_id}.acf")),
            bytes,
        )
        .expect("temp appmanifest (raw)");
    }
}

impl Drop for SteamRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// The loopback catalog fake: a `TcpListener` in the test process behind
/// `--catalog-url`. It answers each HTTP request through the caller's responder and
/// records every request line, so "zero requests" (`025-L4`) and the exact
/// `GET /v1/candidates?slug=<encoded> HTTP/1.1` spelling (`025-C4`) are observable.
pub struct FakeCatalog {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
}

impl FakeCatalog {
    /// Starts the fake; `responder` receives the full request text and returns the
    /// raw bytes to write back. A bare connect (the gate's `probe()`, which asks the
    /// authority, not an endpoint) gets a closed socket and is not recorded.
    pub fn start(label: &str, responder: impl Fn(&str) -> Vec<u8> + Send + Sync + 'static) -> Self {
        let _ = label;
        let listener = TcpListener::bind("127.0.0.1:0")
            .expect("a loopback listener must bind (no test goes off-host)");
        let url = format!(
            "http://127.0.0.1:{}",
            listener.local_addr().expect("listener address").port()
        );
        listener
            .set_nonblocking(true)
            .expect("nonblocking accept loop");

        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        {
            let requests = requests.clone();
            let stop = stop.clone();
            thread::spawn(move || {
                loop {
                    match listener.accept() {
                        Ok((mut stream, _addr)) => {
                            let responder = &responder;
                            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                            let mut raw: Vec<u8> = Vec::new();
                            let mut window = [0_u8; 1024];
                            while !find_header_end(&raw).unwrap_or(false) && raw.len() < 65_536 {
                                match stream.read(&mut window) {
                                    Ok(0) | Err(_) => break,
                                    Ok(n) => raw.extend_from_slice(&window[..n]),
                                }
                            }
                            if !raw.is_empty() {
                                let request = String::from_utf8_lossy(&raw).into_owned();
                                let first_line =
                                    request.lines().next().unwrap_or_default().to_string();
                                requests.lock().expect("record").push(first_line);
                                let response = responder(&request);
                                let _ = stream.write_all(&response);
                            }
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            if stop.load(Ordering::Relaxed) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(_err) => break,
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

    /// The `--catalog-url` value to hand the CLI.
    pub fn url(&self) -> String {
        self.url.clone()
    }

    /// Every recorded HTTP request line, in arrival order.
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().expect("record").clone()
    }

    pub fn request_count(&self) -> usize {
        self.requests.lock().expect("record").len()
    }
}

impl Drop for FakeCatalog {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // Unblock the accept loop so the thread ends with the test.
        if let Ok(addr) = self
            .url
            .replace("http://", "")
            .parse::<std::net::SocketAddr>()
        {
            let _ = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(200));
        }
    }
}

fn find_header_end(raw: &[u8]) -> Option<bool> {
    Some(
        raw.windows(4)
            .any(|window| window == b"\r\n\r\n" || window == b"\n\n"),
    )
}

/// A well-formed `200 OK` JSON response with the given body.
pub fn ok(body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

/// A JSON `Vec<Candidate>` body: `[{"game_id":…,"name":…}, …]` in the given order.
pub fn candidates_body(items: &[(&str, &str)]) -> String {
    let entries: Vec<String> = items
        .iter()
        .map(|(game_id, name)| format!("{{\"game_id\":\"{game_id}\",\"name\":\"{name}\"}}"))
        .collect();
    format!("[{}]", entries.join(","))
}

/// A responder that returns the response bytes of the first `(needle, response)`
/// whose needle the request contains, and `default` otherwise.
pub fn respond_to(
    cases: Vec<(String, Vec<u8>)>,
    default: Vec<u8>,
) -> impl Fn(&str) -> Vec<u8> + Send + Sync + 'static {
    move |request| {
        for (needle, response) in &cases {
            if request.contains(needle.as_str()) {
                return response.clone();
            }
        }
        default.clone()
    }
}

/// A responder body answering every query with no candidates.
pub fn default_no_candidates() -> Vec<u8> {
    ok("[]")
}

/// A bound-and-closed loopback port: the gate test's "no listener behind
/// `--catalog-url`" — a connect to it is refused, as it must be for the fake.
pub fn closed_loopback_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind to learn a free port");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    port
}

/// One CLI subprocess run and its captured streams.
pub struct Run {
    pub output: Output,
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

/// Runs the built `discovery-cli` with `args`, the extra environment entries
/// `envs`, and `stdin` piped in (an empty string is EOF — a declined prompt).
///
/// The inherited store and endpoint variables are removed first: every test names
/// its own `XDG_DATA_HOME`/`HOME` sandbox and its own `--catalog-url` (or names
/// `BINDFORGE_CATALOG_URL`), so no run can touch the developer's real match store or
/// reach anything the test did not stand up.
pub fn run_cli(args: &[&str], envs: &[(&str, &str)], stdin: &str) -> Run {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_discovery-cli"));
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
        .unwrap_or_else(|err| panic!("failed to execute discovery-cli: {err}"));
    {
        let mut pipe = child.stdin.take().expect("piped stdin");
        pipe.write_all(stdin.as_bytes()).expect("stdin is writable");
    }
    let output = child.wait_with_output().expect("discovery-cli finished");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        output,
    }
}

/// The cells of one rendered table line, trimmed of the padding the renderer wrote.
pub fn cells(line: &str) -> Vec<String> {
    let inner = line
        .strip_prefix('|')
        .and_then(|rest| rest.strip_suffix('|'))
        .unwrap_or(line);
    inner
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

/// The percent-encoded query needle the client must send for `steam/<appid>`
/// (`candidates_url` encodes the `/`, Chosen design: Catalog client and gate).
pub fn slug_needle(appid: &str) -> String {
    format!("slug=steam%2F{appid}")
}

/// The store path, spelled against a sandbox: `<data>/bindforge/matches.json`.
pub fn store_path(data_home: &Path) -> PathBuf {
    data_home.join("bindforge").join("matches.json")
}
