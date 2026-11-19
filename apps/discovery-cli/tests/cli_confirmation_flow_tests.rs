//! The interactive confirmation round-trip of design 025 / ADR-025 (`025-C5`,
//! `025-L2`): the `selection` answers, the one-prompt-per-slug flow on stderr, the
//! store consequence of a yes or a no, and the control-byte safety of the rendered
//! table — driven end-to-end against the built CLI with the loopback catalog fake
//! and a temp `XDG_DATA_HOME`/`HOME` sandbox.
//!
//! proves: design 025's Test strategy #8 (Chosen design: "`fn selection(answer:
//! &str, count: usize) -> Option<usize>` (`1`-based; blank, `n`, out-of-range, or
//! EOF is no match") — `1` and `2` confirm their candidates, `" 2 "` is accepted
//! with its padding, and blank, `n`, `0`, `3` over two candidates, and EOF each
//! match nothing and write no store) and #12 (the round-trip against the
//! `TcpListener` fake: scan one answers `1`, writes one record, and prints `-` in
//! `Game Id`; scan two prints the id, makes zero further requests — the fake
//! counts — and rewrites nothing; a `n` answer leaves no record; two fixture
//! slugs confirmed to one `game_id` are two records; and an ACF `name` carrying
//! an ESC byte renders with no raw control byte in the table).
//!
//! The pure `selection` answers per string are in-module pins of
//! `apps/discovery-cli/src/main.rs` (Rollout 2); this suite pins the same
//! behavior through the real stdin of a real subprocess, which is the seam the
//! design says a scripted run actually hits.

mod common;

use common::{FakeCatalog, SteamRoot, TempDir, candidates_body, cells, ok, run_cli, store_path};

/// The two-candidate body the fake serves for any queried slug.
fn two_candidates() -> String {
    candidates_body(&[("cand-one", "First Match"), ("cand-two", "Second Match")])
}

fn fake_catalog() -> FakeCatalog {
    let body = two_candidates();
    FakeCatalog::start("flow-fake", move |_request| ok(&body))
}

fn fake_no_candidates() -> FakeCatalog {
    FakeCatalog::start("flow-fake-empty", |_| ok("[]"))
}

/// A one-game sandbox pair under one label: `XDG_DATA_HOME` and `HOME` both
/// absolute, so the store rule lands on the XDG branch.
fn sandbox(label: &str) -> (TempDir, TempDir) {
    (
        TempDir::new(&format!("flow-{label}-data")),
        TempDir::new(&format!("flow-{label}-home")),
    )
}

fn env<'a>(data: &'a TempDir, home: &'a TempDir) -> [(&'a str, &'a str); 2] {
    [
        ("XDG_DATA_HOME", data.path().to_str().expect("utf-8")),
        ("HOME", home.path().to_str().expect("utf-8")),
    ]
}

fn run_one_game(
    root: &SteamRoot,
    data: &TempDir,
    home: &TempDir,
    url: &str,
    stdin: &str,
) -> common::Run {
    run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8 path"),
            "--catalog-url",
            url,
        ],
        &env(data, home),
        stdin,
    )
}

fn one_game_root(label: &str) -> SteamRoot {
    let root = SteamRoot::new(label);
    root.acf("5001", Some("Example Game"), "Example Game");
    root
}

/// proves: design 025's Test strategy #8 and #12 first half — answering `1`
/// confirms that candidate: the run exits 0, the store gains exactly the record
/// `(steam/5001, cand-one, Example Game)` at `version: 1`, the first scan's own
/// table still prints `-` in `Game Id` (the row rendered before any join), the
/// query is the one pinned GET for the slug, and stdout carries only the table —
/// the prompt lives on stderr, so stdout is exactly header + separator + one row.
#[test]
fn answering_one_writes_its_record_and_the_first_scan_prints_a_dash() {
    let root = one_game_root("confirm-yes");
    let (data, home) = sandbox("confirm-yes");
    let fake = fake_catalog();

    let run = run_one_game(&root, &data, &home, &fake.url(), "1\n");
    assert!(run.success(), "a confirmed run succeeds: {}", run.stderr);

    let lines: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "stdout carries only the table — the prompt never prints there:\n{}",
        run.stdout
    );
    let row = cells(lines[2]);
    assert_eq!(row[0], "steam/5001");
    assert_eq!(
        row[1], "-",
        "the scan that earns a confirmation still renders `-` for the row it printed"
    );

    assert_eq!(
        fake.requests(),
        vec!["GET /v1/candidates?slug=steam%2F5001 HTTP/1.1".to_string()],
        "one GET per distinct unresolved slug"
    );

    let document: serde_json::Value = std::fs::read_to_string(store_path(data.path()))
        .expect("the confirmation writes the store")
        .parse()
        .expect("the store is JSON");
    assert_eq!(document["version"].as_i64(), Some(1));
    let records = document["matches"].as_array().expect("a matches array");
    assert_eq!(
        records
            .iter()
            .map(|record| (
                record["slug"].as_str().unwrap_or_default(),
                record["game_id"].as_str().unwrap_or_default(),
                record["friendly_name"].as_str().unwrap_or_default(),
            ))
            .collect::<Vec<(&str, &str, &str)>>(),
        vec![("steam/5001", "cand-one", "Example Game")],
        "the record is the design's (slug, game_id, friendly name of the first \
         sharing row in (slug, install_path) order)"
    );
}

/// proves: design 025's Test strategy #8 (Chosen design: "`1`-based") — `" 2 "`
/// is accepted despite its padding and confirms the second candidate.
#[test]
fn a_padded_answer_confirms_the_candidate_it_numbers() {
    let root = one_game_root("confirm-padded");
    let (data, home) = sandbox("confirm-padded");
    let fake = fake_catalog();

    let run = run_one_game(&root, &data, &home, &fake.url(), " 2 \n");
    assert!(run.success(), "stderr: {}", run.stderr);

    let document: serde_json::Value = std::fs::read_to_string(store_path(data.path()))
        .expect("the confirmation writes the store")
        .parse()
        .expect("the store is JSON");
    let records = document["matches"].as_array().expect("a matches array");
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0]["game_id"].as_str(),
        Some("cand-two"),
        "`\" 2 \"` numbers the second candidate, padding ignored"
    );
}

/// proves: design 025's Test strategy #8 (Chosen design: "blank, `n`,
/// out-of-range, or EOF is no match") — EOF, a blank line, `n`, `0`, and `3` over
/// two candidates each decline: the run still exits 0 with the table, and nothing
/// is ever stored without the user's choice (`025-L2`) — no store file, not even
/// a store directory.
#[test]
fn blank_decline_out_of_range_and_eof_answers_store_nothing() {
    for (label, answer) in [
        ("eof", ""),
        ("blank", "\n"),
        ("letter-n", "n\n"),
        ("zero", "0\n"),
        ("past-count", "3\n"),
    ] {
        let root = one_game_root(&format!("confirm-{label}"));
        let (data, home) = sandbox(&format!("confirm-{label}"));
        let fake = fake_catalog();

        let run = run_one_game(&root, &data, &home, &fake.url(), answer);
        assert!(
            run.success(),
            "the `{label}` answer declines, which is a clean run: {}",
            run.stderr
        );
        assert_eq!(
            run.stdout.lines().count(),
            3,
            "the `{label}` run still prints the table:\n{}",
            run.stdout
        );
        assert_eq!(
            cells(run.stdout.lines().nth(2).expect("one row"))[1],
            "-",
            "a declined slug renders `-` in `Game Id`"
        );
        assert!(
            !store_path(data.path()).exists(),
            "the `{label}` answer matches nothing, so no store file is written"
        );
        assert!(
            !data.join("bindforge").exists(),
            "the `{label}` answer must not even stand up the store directory"
        );
    }
}

/// proves: design 025's Test strategy #12, second half (Chosen design: "`025-L4`
/// — records are re-trusted on every scan… A slug the store resolves is never
/// queried") — scan one confirms; scan two over the same sandbox and fake prints
/// the stored id in `Game Id`, asks the fake nothing new (the recorded request
/// count is unchanged), and rewrites no store byte.
#[test]
fn the_rescan_prints_the_stored_id_and_asks_the_catalog_nothing() {
    let root = one_game_root("rescan");
    let (data, home) = sandbox("rescan");
    let fake = fake_catalog();

    let scan_one = run_one_game(&root, &data, &home, &fake.url(), "1\n");
    assert!(scan_one.success(), "stderr 1: {}", scan_one.stderr);
    let stored = std::fs::read(store_path(data.path())).expect("store after scan 1");
    assert_eq!(
        fake.request_count(),
        1,
        "scan one made exactly one query: {:?}",
        fake.requests()
    );

    let scan_two = run_one_game(&root, &data, &home, &fake.url(), "");
    assert!(
        scan_two.success(),
        "a resolved run needs no answer and succeeds: {}",
        scan_two.stderr
    );
    let lines: Vec<&str> = scan_two.stdout.lines().collect();
    assert_eq!(
        cells(lines[2])[1],
        "cand-one",
        "the `Game Id` column prints the store's confirmed id"
    );
    assert_eq!(
        fake.request_count(),
        1,
        "a slug the store resolves is never queried (`025-L4`) — scan two added no \
         request: {:?}",
        fake.requests()
    );
    assert_eq!(
        std::fs::read(store_path(data.path())).expect("store after scan 2"),
        stored,
        "a scan with nothing new to confirm rewrites nothing"
    );
}

/// proves: design 025's Test strategy #12 (Chosen design: "the store holds one
/// record per distinct slug — two installs of one game are two records sharing one
/// `game_id`") — two fixture slugs confirmed to the same `game_id` are two
/// records, each with its own friendly name.
#[test]
fn two_slugs_confirmed_to_one_game_id_are_two_records() {
    let root = SteamRoot::new("flow-two-slugs");
    root.acf("5001", Some("First Game"), "First Game");
    root.acf("5002", Some("Second Game"), "Second Game");
    let (data, home) = sandbox("two-slugs");
    let fake = fake_catalog();

    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
        ],
        &env(&data, &home),
        "1\n1\n",
    );
    assert!(run.success(), "stderr: {}", run.stderr);
    assert_eq!(
        fake.request_count(),
        2,
        "one query per distinct slug, none for the second row of a shared slug: {:?}",
        fake.requests()
    );

    let document: serde_json::Value = std::fs::read_to_string(store_path(data.path()))
        .expect("the confirmations write the store")
        .parse()
        .expect("the store is JSON");
    let records = document["matches"].as_array().expect("a matches array");
    assert_eq!(
        records
            .iter()
            .map(|record| (
                record["slug"].as_str().unwrap_or_default(),
                record["game_id"].as_str().unwrap_or_default(),
                record["friendly_name"].as_str().unwrap_or_default(),
            ))
            .collect::<Vec<(&str, &str, &str)>>(),
        vec![
            ("steam/5001", "cand-one", "First Game"),
            ("steam/5002", "cand-one", "Second Game"),
        ],
        "one `game_id`, two slugs, two records"
    );
}

/// proves: design 025's Test strategy #12, last clause (Chosen design: the two
/// ACF-sourced cells "render through `escape_and_truncate` in place of today's raw
/// `row_cells` pass-through, so a hostile ACF byte cannot emit a terminal control
/// sequence") — an ACF `name` carrying a raw ESC byte renders in the table with no
/// raw control byte anywhere in either output stream, and the run stays clean.
#[test]
fn an_esc_byte_in_an_acf_name_renders_without_a_raw_control_byte() {
    let root = SteamRoot::new("flow-esc");
    let mut acf: Vec<u8> = Vec::new();
    acf.extend_from_slice(b"\"AppState\"\n{\n\t\"appid\"\t\t\"5001\"\n");
    acf.extend_from_slice(b"\t\"Universe\"\t\t\"1\"\n\t\"name\"\t\t\"Esc");
    acf.push(0x1b);
    acf.extend_from_slice(b"ape Game\"\n\t\"StateFlags\"\t\t\"4\"\n");
    acf.extend_from_slice(
        b"\t\"installdir\"\t\t\"Esc Game\"\n\t\"SizeOnDisk\"\t\t\"1000000\"\n}\n",
    );
    root.acf_raw("5001", &acf);
    let (data, home) = sandbox("esc");
    let fake = fake_no_candidates();

    let run = run_one_game(&root, &data, &home, &fake.url(), "");
    assert!(
        run.success(),
        "a hostile name is rendered, not fatal: {}",
        run.stderr
    );

    let table: Vec<&str> = run.stdout.lines().collect();
    assert_eq!(table.len(), 3, "the row still renders:\n{}", run.stdout);
    for (stream, text) in [("stdout", &run.stdout), ("stderr", &run.stderr)] {
        for byte in text.as_bytes() {
            assert!(
                *byte >= 0x20 || *byte == b'\n',
                "the {stream} carries no raw control byte (found {byte:#04x}) — ACF bytes \
                 are escaped before rendering: {text:?}"
            );
        }
    }
    let row = cells(table[2]);
    assert_eq!(row[0], "steam/5001");
    assert!(
        !row[2].trim().is_empty(),
        "the hostile Display Name cell still renders its name, escaped: {:?}",
        row[2]
    );
}
