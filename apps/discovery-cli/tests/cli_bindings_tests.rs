//! The `discovery bindings <slug>` face (`017-C4`) — slug resolution, the
//! aligned table, `--json`, `--probe`, the no-network posture, and the
//! never-write discipline (`017-L1`, `017-L5`).
//!
//! proves: Test strategy #13 (rendering, probe, no-network, never-write) and
//! the CLI half of #12 and #14.
//!
//! Integration (CLI, subprocess): temp Steam roots driven with `--steam-root`,
//! no catalog listener on this face (§The CLI face: enumeration, no network
//! leg). `run_cli` strips `BINDFORGE_CATALOG_URL`/`HOME`/`XDG_DATA_HOME`, so
//! every run here is by construction network-less and store-less.

mod common;

use common::{FakeCatalog, Run, SteamRoot, TempDir, candidates_body, cells, ok, run_cli};
use std::path::Path;

const APPID: &str = "900001";
const USER: &str = "76561197960287930";

/// The user config the fixture game ships: native tokens verbatim, one
/// `-Bindings` removal.
const USER_INI: &str = "[/Script/Engine.PlayerInput]\n\
     +Bindings=(Name=\"W\",Command=\"MoveForward\")\n\
     +Bindings=(Name=\"MouseX\",Command=\"TurnRight\")\n\
     +Bindings=(Name=\"E\",Command=\"Interact\")\n\
     -Bindings=(Name=\"E\",Command=\"Interact\")\n";

/// A Steam root holding the game and its Unreal user config under `userdata`.
fn fixture_root(label: &str) -> SteamRoot {
    let root = SteamRoot::new(label);
    root.acf(APPID, Some("Example Game"), "Example Game");
    let ini = format!("userdata/{USER}/{APPID}/Saved/Config/LinuxNoEditor/Engine.ini");
    let path = root.path().join(ini);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("config dir");
    std::fs::write(&path, USER_INI).expect("user ini");
    root
}

fn run_bindings(args: &[&str], root: &SteamRoot) -> Run {
    let mut argv = vec!["bindings", "steam/900001", "--steam-root"];
    let path = root.path().to_str().expect("utf-8");
    argv.push(path);
    argv.extend_from_slice(args);
    run_cli(&argv, &[], "")
}

// proves: Test strategy #13 (§The CLI face step 4) — one table for a fixture
// game: columns `Codec | User | Action | Inputs`, the `Codec` cell populated
// from the snapshot (`unreal-ini`), the file's native tokens verbatim; exit 0
// with no status line on stderr.
#[test]
fn bindings_prints_the_aligned_table_for_a_fixture_game() {
    let root = fixture_root("bind-table");
    let run = run_bindings(&[], &root);
    assert_eq!(
        run.code(),
        Some(0),
        "stdout: {}\nstderr: {}",
        run.stdout,
        run.stderr
    );
    assert!(
        run.stderr.is_empty(),
        "a clean run keeps stderr empty: {}",
        run.stderr
    );
    let mut lines = run.stdout.lines();
    let header = cells(lines.next().expect("header row"));
    assert_eq!(header, ["Codec", "User", "Action", "Inputs"]);
    // The shared table renderer's pinned shape (as `list`'s suites pin it):
    // header, dash separator, then the body rows.
    let separator = lines.next().expect("separator row");
    assert!(
        separator.starts_with('|') && separator.chars().all(|c| matches!(c, '|' | '-' | ' ')),
        "the separator sits under the aligned header: {separator:?}"
    );
    let rows: Vec<Vec<String>> = lines.map(cells).collect();
    assert_eq!(
        rows,
        vec![
            vec![
                "unreal-ini".to_string(),
                USER.to_string(),
                "MoveForward".to_string(),
                "W".to_string()
            ],
            vec![
                "unreal-ini".to_string(),
                USER.to_string(),
                "TurnRight".to_string(),
                "MouseX".to_string()
            ],
        ],
        "entries byte-sorted by action; the removed pair is absent"
    );
}

// proves: Test strategy #13 and #12 (§The CLI face step 4: "`--json` prints
// one compact JSON array of snapshot documents") — one array, the closed
// document keys, byte-identical across repeated runs.
#[test]
fn bindings_json_prints_the_snapshot_documents_byte_identically() {
    let root = fixture_root("bind-json");
    let first = run_bindings(&["--json"], &root);
    let second = run_bindings(&["--json"], &root);
    assert_eq!(first.code(), Some(0), "stderr: {}", first.stderr);
    assert_eq!(
        first.stdout, second.stdout,
        "two --json runs must be byte-identical (017-L4)"
    );
    let docs: serde_json::Value =
        serde_json::from_str(first.stdout.trim()).expect("--json is one parseable JSON document");
    let docs = docs
        .as_array()
        .expect("one JSON array of snapshot documents");
    assert_eq!(docs.len(), 1, "one located user tree means one document");
    let doc = &docs[0];
    let mut keys: Vec<&str> = doc
        .as_object()
        .expect("document object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "codec",
            "entries",
            "format",
            "game",
            "provenance",
            "user",
            "version"
        ],
        "the closed top-level key set"
    );
    assert_eq!(doc["format"], "bindings");
    assert_eq!(doc["version"], 1, "version is the bare integer");
    assert_eq!(doc["codec"], "unreal-ini");
    assert_eq!(doc["game"], "steam/900001", "the game is the lookup key");
    assert_eq!(doc["user"], USER);
    assert_eq!(doc["provenance"]["evidence"], "install-tree-probe");
    assert_eq!(
        doc["provenance"]["files"],
        serde_json::json!(["Saved/Config/LinuxNoEditor/Engine.ini"]),
        "contract files are tree-relative and byte-sorted"
    );
    assert_eq!(doc["entries"][0]["action"], "MoveForward");
    assert_eq!(doc["entries"][1]["inputs"], serde_json::json!(["MouseX"]));
}

// proves: Test strategy #12 (§The CLI face step 4: the table "goes through the
// shared width algorithm") — the aligned table is byte-identical across runs.
#[test]
fn the_table_is_byte_identical_across_runs() {
    let root = fixture_root("bind-table-det");
    let first = run_bindings(&[], &root);
    let second = run_bindings(&[], &root);
    assert_eq!(
        first.stdout, second.stdout,
        "the rendered table must not vary"
    );
}

// proves: Test strategy #13 (§The CLI face step 4: "`--probe` columns are
// `Family | Evidence | User | File` with absolute paths") — a probe run
// prints the located candidates with absolute paths and extracts nothing.
#[test]
fn probe_prints_families_evidence_and_absolute_paths_and_extracts_nothing() {
    let root = fixture_root("bind-probe");
    let run = run_bindings(&["--probe"], &root);
    assert_eq!(run.code(), Some(0), "stderr: {}", run.stderr);
    let mut lines = run.stdout.lines();
    let header = cells(lines.next().expect("header row"));
    assert_eq!(header, ["Family", "Evidence", "User", "File"]);
    // Header, separator, body — the shared renderer's pinned shape.
    let separator = lines.next().expect("separator row");
    assert!(
        separator.starts_with('|') && separator.chars().all(|c| matches!(c, '|' | '-' | ' ')),
        "the separator sits under the aligned header: {separator:?}"
    );
    let rows: Vec<Vec<String>> = lines.map(cells).collect();
    assert_eq!(rows.len(), 1, "one located file, one row");
    assert_eq!(rows[0][0], "unreal-ini", "family");
    assert_eq!(rows[0][1], "install-tree-probe", "evidence");
    assert_eq!(rows[0][2], USER, "user");
    let shown = Path::new(&rows[0][3]);
    assert!(
        shown.is_absolute(),
        "probe shows absolute paths: {}",
        rows[0][3]
    );
    assert!(shown.exists(), "the shown candidate exists");
    assert!(
        !run.stdout.contains("MoveForward"),
        "probe mode extracts nothing: {}",
        run.stdout
    );
}

// proves: Test strategy #13 (§The CLI face step 4: "`--probe --json` prints
// `{\"status\":…,\"claims\":[…]}`") — the probe JSON carries the status and the
// claim list, not the snapshot documents.
#[test]
fn probe_json_prints_the_status_and_claims_document() {
    let root = fixture_root("bind-probe-json");
    let run = run_bindings(&["--probe", "--json"], &root);
    assert_eq!(run.code(), Some(0), "stderr: {}", run.stderr);
    let doc: serde_json::Value =
        serde_json::from_str(run.stdout.trim()).expect("probe --json is one JSON document");
    assert_eq!(doc["status"], "ok");
    let claims = doc["claims"].as_array().expect("claims is an array");
    assert_eq!(claims.len(), 1, "one claiming record, one claim entry");
    let claim = &claims[0];
    assert_eq!(claim["family"], "unreal-ini");
    assert_eq!(claim["evidence"], "install-tree-probe");
    assert_eq!(claim["user"], USER);
    let files = claim["files"].as_array().expect("files is an array");
    assert!(
        files
            .iter()
            .any(|file| file.as_str().is_some_and(|f| f.ends_with("Engine.ini")))
    );
}

// proves: Test strategy #13 (§The CLI face step 1: "No catalog client, no
// probe, no match store, no prompt, no candidate query"; Options' A13
// divergence from `017-C4`'s "same scan") — no network leg exists on this
// face: the test runs with no `--catalog-url`, no `BINDFORGE_CATALOG_URL`
// (stripped by `run_cli`) and a loopback fake nobody talks to, and still
// exits 0.
#[test]
fn the_bindings_face_never_touches_the_catalog() {
    let fake = FakeCatalog::start("bindings-must-not-call", |_| ok(&candidates_body(&[])));
    let root = fixture_root("bind-nonetwork");
    // A configured-but-unreachable endpoint on the command line must not be
    // needed, consulted, or fatal: pass no --catalog-url at all.
    assert!(!fake.url().is_empty());
    let run = run_bindings(&[], &root);
    assert_eq!(
        run.code(),
        Some(0),
        "extraction is local by construction: no service is reachable here\nstdout: {}\nstderr: {}",
        run.stdout,
        run.stderr
    );
    assert_eq!(
        fake.request_count(),
        0,
        "the bindings face made HTTP requests"
    );
    // The match store is not written either (§The CLI face step 1: "no match store").
    let store_home = TempDir::new("bindings-store-home");
    let store_data = TempDir::new("bindings-store-data");
    let mut argv = vec!["bindings", "steam/900001", "--steam-root"];
    let path = root.path().to_str().expect("utf-8");
    argv.push(path);
    let envs = [
        ("HOME", store_home.path().to_str().expect("utf-8")),
        ("XDG_DATA_HOME", store_data.path().to_str().expect("utf-8")),
    ];
    let sandboxed = run_cli(&argv, &envs, "");
    assert_eq!(sandboxed.code(), Some(0), "stderr: {}", sandboxed.stderr);
    assert!(
        !common::store_path(store_data.path()).exists(),
        "the bindings face never touches the match store"
    );
}

// proves: Test strategy #13 and #17 (§Read discipline Never-write, `017-L1`) —
// the fixture tree's file bytes are byte-identical after every face mode, and
// no file is created under the root.
#[test]
fn the_face_never_writes_anything() {
    let root = fixture_root("bind-neverwrite");
    let before = tree_bytes(root.path());
    for args in [
        vec![],
        vec!["--json"],
        vec!["--probe"],
        vec!["--probe", "--json"],
    ] {
        let run = run_bindings(&args, &root);
        assert_eq!(run.code(), Some(0), "stderr: {}", run.stderr);
    }
    let after = tree_bytes(root.path());
    assert_eq!(
        before, after,
        "a bindings run wrote to the game's tree (017-L1: the run writes nothing)"
    );
}

fn tree_bytes(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(dir: &Path, base: &Path, files: &mut Vec<(String, Vec<u8>)>) {
        let entries = std::fs::read_dir(dir).expect("fixture readable");
        for entry in entries {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(&path, base, files);
            } else {
                let name = path
                    .strip_prefix(base)
                    .expect("under base")
                    .display()
                    .to_string();
                files.push((name, std::fs::read(&path).expect("file readable")));
            }
        }
    }
    let mut files = Vec::new();
    walk(root, root, &mut files);
    files.sort();
    files
}

// proves: Test strategy #13 (§The CLI face step 2: "no match is the ordinary
// failure voice with `EXIT_FAILURE`, echoing the escaped slug and no path") —
// an undiscovered slug is exit 1 with the one `error: ` line, the slug echoed,
// no local path echoed.
#[test]
fn an_undiscovered_slug_is_the_ordinary_failure_voice() {
    let root = fixture_root("bind-noslug");
    let run = run_cli(
        &[
            "bindings",
            "steam/999999",
            "--steam-root",
            root.path().to_str().expect("utf-8"),
        ],
        &[],
        "",
    );
    assert_eq!(run.code(), Some(1));
    assert!(
        run.stderr.contains("error: "),
        "the one failure voice: {}",
        run.stderr
    );
    assert!(
        run.stderr.contains("steam/999999"),
        "the slug is echoed: {}",
        run.stderr
    );
    assert!(
        !run.stderr.contains("userdata") && !run.stderr.contains("steamapps"),
        "the refusal echoes no local path: {}",
        run.stderr
    );
}

// proves: Test strategy #14 (§The CLI face step 6, `017-L5`) — a tree the
// bindings face claims a codec for still prints `codec_id: null` on every
// `list --json` row: the row type stays out of the codec story. (The existing
// `cli_output_polish_tests`, `cli_json_mode_tests`, `cli_catalog_gate_tests`
// and `cli_slug_refusal_tests` suites are the rest of this pin and must pass
// unmodified.)
#[test]
fn list_still_prints_a_null_codec_id_for_a_claimed_tree() {
    let root = fixture_root("bind-listnull");
    let binding = run_bindings(&[], &root);
    assert_eq!(
        binding.code(),
        Some(0),
        "bindings claims this tree: {}",
        binding.stderr
    );

    let fake = FakeCatalog::start("list-null", |_| ok(&candidates_body(&[])));
    let data = TempDir::new("listnull-data");
    let home = TempDir::new("listnull-home");
    let envs = [
        ("XDG_DATA_HOME", data.path().to_str().expect("utf-8")),
        ("HOME", home.path().to_str().expect("utf-8")),
    ];
    let list = run_cli(
        &[
            "--json",
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8"),
            "--catalog-url",
            &fake.url(),
        ],
        &envs,
        "",
    );
    assert_eq!(list.code(), Some(0), "stderr: {}", list.stderr);
    let rows: serde_json::Value =
        serde_json::from_str(list.stdout.trim()).expect("list --json is one document");
    let rows = rows.as_array().expect("list --json prints an array");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["slug"], "steam/900001");
    assert!(
        rows[0]["codec_id"].is_null(),
        "`codec_id` keeps spelling null on `list` (017-L5): {}",
        rows[0]
    );
}

// proves: Test strategy #12 (§The CLI face step 3: "More than one record
// matching (one game in two libraries) is not an error: each is processed") —
// one slug installed in two libraries renders two probe groups.
#[test]
fn one_slug_in_two_libraries_prints_two_probe_groups() {
    let root = SteamRoot::new("bind-twolibs");
    root.acf(APPID, Some("Example Game"), "Example Game");
    let second = SteamRoot::new("bind-twolibs-b");
    second.acf(APPID, Some("Example Game"), "Example Game");
    for root in [&root, &second] {
        let ini = format!("userdata/{USER}/{APPID}/Saved/Config/LinuxNoEditor/Engine.ini");
        let path = root.path().join(&ini);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(&path, USER_INI).expect("ini");
    }

    let run = run_cli(
        &[
            "bindings",
            "steam/900001",
            "--probe",
            "--steam-root",
            root.path().to_str().expect("utf-8"),
            "--steam-root",
            second.path().to_str().expect("utf-8"),
        ],
        &[],
        "",
    );
    assert_eq!(run.code(), Some(0), "stderr: {}", run.stderr);
    let rows: Vec<Vec<String>> = run.stdout.lines().skip(2).map(cells).collect(); // header + separator
    assert_eq!(
        rows.len(),
        2,
        "two records, each processed, each stating its own located tree (§Determinism's tree-identity dedup keeps one statement per tree)"
    );
    assert_ne!(
        rows[0][3], rows[1][3],
        "the two groups name different candidate files"
    );
}
