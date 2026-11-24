//! The `discovery bindings` statuses and exit codes (`017-C4`, `017-L2`) —
//! four named statuses on four distinct codes, each reached by a real fixture
//! route, each printing exactly one `status: <name>` line on stderr, all above
//! clap's own usage-error exit (§The CLI face step 5).
//!
//! proves: Test strategy #13 (the status half).

mod common;

use common::{Run, SteamRoot, run_cli};

const USER: &str = "76561197960287930";

fn bindings(appid: &str, roots: &[&SteamRoot], extra: &[&str]) -> Run {
    let slug = format!("steam/{appid}");
    let mut argv: Vec<&str> = vec!["bindings", slug.as_str()];
    for root in roots {
        argv.push("--steam-root");
        argv.push(root.path().to_str().expect("utf-8"));
    }
    argv.extend_from_slice(extra);
    run_cli(&argv, &[], "")
}

fn game(root: &SteamRoot, appid: &str) {
    root.acf(appid, Some("Example Game"), "Example Game");
}

fn user_ini(root: &SteamRoot, appid: &str, body: &str) {
    let ini = format!("userdata/{USER}/{appid}/Saved/Config/LinuxNoEditor/Engine.ini");
    let path = root.path().join(ini);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("config dir");
    std::fs::write(&path, body).expect("user ini");
}

/// Every stderr `status:` lines a run printed.
fn status_lines(run: &Run) -> Vec<String> {
    run.stderr
        .lines()
        .filter(|line| line.starts_with("status: "))
        .map(str::to_string)
        .collect()
}

// proves: Test strategy #13 (§The CLI face step 5: `ok` 0) — a clean
// extraction is status `ok`, exit 0, and prints no status line (only
// non-ok statuses do).
#[test]
fn a_clean_extraction_is_ok_exit_zero_with_no_status_line() {
    let root = SteamRoot::new("status-ok");
    game(&root, "800001");
    user_ini(
        &root,
        "800001",
        "[/Script/Engine.PlayerInput]\n+Bindings=(Name=\"W\",Command=\"MoveForward\")\n",
    );
    let run = bindings("800001", &[&root], &[]);
    assert_eq!(run.code(), Some(0), "stderr: {}", run.stderr);
    assert!(
        status_lines(&run).is_empty(),
        "ok prints no status line: {}",
        run.stderr
    );
    assert!(run.stdout.contains("MoveForward"));
}

// proves: Test strategy #13 (§The CLI face step 5: `claim-refused` 3;
// §Compatibility matrix: a game no shipped grammar locates is `claim-refused`)
// — a discovered steam row with no locatable config grammar exits 3 with
// exactly one named status line and no table.
#[test]
fn a_row_no_family_claims_is_claim_refused_exit_three() {
    let root = SteamRoot::new("status-claim-refused");
    game(&root, "800002"); // installed, but no unreal config anywhere under the root
    let run = bindings("800002", &[&root], &[]);
    assert_eq!(
        run.code(),
        Some(3),
        "stdout: {}\nstderr: {}",
        run.stdout,
        run.stderr
    );
    assert_eq!(
        status_lines(&run),
        vec!["status: claim-refused".to_string()],
        "exactly one named status line on stderr: {:?}",
        run.stderr_lines()
    );
    assert!(
        run.stdout.trim().is_empty(),
        "stdout stays the table or the document: {:?}",
        run.stdout
    );
}

// proves: Test strategy #13 (§The CLI face step 5: `family-confirmed-files-absent`
// 4; §Compatibility matrix: a degraded launcher context "reads as
// `family-confirmed-files-absent` (exit 4), never a hang or an out-of-tree
// read"; §Options: the visible `017-L2` case is family-confirmed-with-nothing-to-read)
// — the family claims the game and the located store states nothing readable:
// its own status, exit 4.
#[test]
fn a_claimed_family_with_nothing_to_read_is_exit_four() {
    let root = SteamRoot::new("status-files-absent");
    game(&root, "800003");
    // A located unreal config whose every pair the file itself removes: the
    // family is confirmed, and there is nothing to read.
    user_ini(
        &root,
        "800003",
        "[/Script/Engine.PlayerInput]\n\
         +Bindings=(Name=\"E\",Command=\"Interact\")\n\
         -Bindings=(Name=\"E\",Command=\"Interact\")\n",
    );
    let run = bindings("800003", &[&root], &[]);
    assert_eq!(
        run.code(),
        Some(4),
        "family-confirmed, nothing extracted, must reach its own exit code\nstdout: {}\nstderr: {}",
        run.stdout,
        run.stderr
    );
    assert_eq!(
        status_lines(&run),
        vec!["status: family-confirmed-files-absent".to_string()]
    );
}

// proves: Test strategy #13 (§The CLI face step 5: `extraction-refused` 5,
// "its `error: ` line carries the `ExtractionRefusal` name") — a located
// config the codec refuses to parse exits 5, naming `UnparsableEntry`.
#[test]
fn an_extraction_refusal_is_exit_five_naming_the_variant() {
    let root = SteamRoot::new("status-extraction-refused");
    game(&root, "800004");
    user_ini(
        &root,
        "800004",
        "[/Script/Engine.PlayerInput]\n+Bindings=(Name=W,Command=NoQuotes)\n",
    );
    let run = bindings("800004", &[&root], &[]);
    assert_eq!(run.code(), Some(5), "stderr: {}", run.stderr);
    assert_eq!(
        status_lines(&run),
        vec!["status: extraction-refused".to_string()]
    );
    assert!(
        run.stderr.contains("error: ") && run.stderr.contains("UnparsableEntry"),
        "the error line carries the ExtractionRefusal name: {:?}",
        run.stderr_lines()
    );
}

// proves: Test strategy #13 (§The CLI face step 5: "`--json` printing the
// status document") — with `--json`, a non-ok status prints one JSON status
// document instead of a snapshot array.
#[test]
fn json_mode_prints_the_status_document_for_a_failing_status() {
    let root = SteamRoot::new("status-json");
    game(&root, "800005"); // nothing to claim
    let run = bindings("800005", &[&root], &["--json"]);
    assert_eq!(run.code(), Some(3), "stderr: {}", run.stderr);
    let doc: serde_json::Value =
        serde_json::from_str(run.stdout.trim()).expect("the status document is one JSON object");
    assert_eq!(doc["status"], "claim-refused", "the status is named: {doc}");
}

// proves: Test strategy #13 (§The CLI face step 5: "a run over several records
// exits the highest-ranked status it reported (`extraction-refused` >
// `family-confirmed-files-absent` > `claim-refused` > `ok`)") — a game that
// refuses extraction in one root and is claim-refused in another exits 5.
#[test]
fn several_records_exit_the_highest_ranked_status_reported() {
    let refused = SteamRoot::new("status-rank-refused");
    game(&refused, "800006"); // claim-refused record (route to 3)
    let broken = SteamRoot::new("status-rank-broken");
    game(&broken, "800006"); // same slug, second install, refusing config (route to 5)
    user_ini(
        &broken,
        "800006",
        "[/Script/Engine.PlayerInput]\n+Bindings=(Name=W,Command=NoQuotes)\n",
    );
    let run = bindings("800006", &[&refused, &broken], &[]);
    assert_eq!(
        run.code(),
        Some(5),
        "extraction-refused outranks claim-refused\nstdout: {}\nstderr: {}",
        run.stdout,
        run.stderr
    );
}

// proves: Test strategy #13 (§The CLI face step 5: statuses sit "above the
// code clap's parser returns for a usage error before `main` runs, so no
// status can be confused with one") — a missing slug is clap's own usage
// exit (2) with no status line; no named status shares code 2.
#[test]
fn a_usage_error_keeps_claps_own_code_and_no_status_line() {
    let run = run_cli(&["bindings"], &[], "");
    assert_eq!(
        run.code(),
        Some(2),
        "clap's parser exits before `main`; statuses are 3/4/5 (stdout: {:?}, stderr: {:?})",
        run.stdout,
        run.stderr
    );
    assert!(
        status_lines(&run).is_empty(),
        "a usage error is not a status: {}",
        run.stderr
    );
}
