//! Output-polish pins for `discovery list`: an aligned ADR-022 table (T-003 §1),
//! the header-only empty table, the global flag help — now including ADR-025's
//! `--catalog-url` — and the clean plugin-failure exit (T-003 §3), all migrated
//! behind the catalog gate and the store sandbox per Rollout 3.
//!
//! proves: design 025's Test strategy #12/#13 at the rendering seam and the
//! Rollout 3 migration mandate — these pre-existing subprocess suites run behind
//! a loopback `TcpListener` fake with a temp `XDG_DATA_HOME`/`HOME` sandbox, so
//! the gate and the store never break a rendering pin, and the re-pinned column
//! contract (`Game Id` in, the retired namespace column out) is what alignment is
//! computed over.
//!
//! Every test drives the built binary against a fixture or a temp tree it creates
//! and removes itself, so no test probes the host (repo practice: integration
//! tests use temp directories and clean up after themselves).

mod common;

use common::{FakeCatalog, SteamRoot, TempDir, cells, ok, run_cli};
use std::path::{Path, PathBuf};

/// The crate's checked-in fixture tree, shared with `crates/discovery` tests.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/discovery/tests/fixtures/steam/minimal")
}

/// A fake that serves no candidates to any query.
fn fake_empty() -> FakeCatalog {
    FakeCatalog::start("polish-fake-empty", |_| ok("[]"))
}

/// The sandbox pair that keeps every store byte inside the test.
fn sandbox(label: &str) -> (TempDir, TempDir) {
    (
        TempDir::new(&format!("polish-{label}-data")),
        TempDir::new(&format!("polish-{label}-home")),
    )
}

fn env<'a>(data: &'a TempDir, home: &'a TempDir) -> [(&'a str, &'a str); 2] {
    [
        ("XDG_DATA_HOME", data.path().to_str().expect("utf-8")),
        ("HOME", home.path().to_str().expect("utf-8")),
    ]
}

/// The byte offsets of the `|` delimiters of one rendered table line. T-003 §1
/// makes an aligned table's every line share this list.
fn pipe_offsets(line: &str) -> Vec<usize> {
    line.bytes()
        .enumerate()
        .filter(|(_, byte)| *byte == b'|')
        .map(|(offset, _)| offset)
        .collect()
}

/// The dash run of each column of a separator line, in column order.
fn separator_runs(separator: &str) -> Vec<String> {
    separator
        .split('|')
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect()
}

/// The `gap` between consecutive delimiters of each column, from the offsets.
fn gaps(offsets: &[usize]) -> Vec<usize> {
    offsets
        .windows(2)
        .map(|pair| pair[1] - pair[0] - 1)
        .collect()
}

/// Asserts that `line` is a separator row: delimiters and dashes only.
fn assert_is_separator(line: &str) {
    assert!(
        line.starts_with('|') && line.ends_with('|') && line.len() > 2,
        "the separator must be delimited and dash-filled: {line:?}"
    );
    assert!(
        line.chars().all(|ch| ch == '|' || ch == '-'),
        "the separator may hold nothing but `|` and `-`: {line:?}"
    );
}

/// proves: T-003 Test plan re-pinned to the ADR-025 columns — for `list
/// --steam-root <fixture>` behind the gate, every stdout line has the identical
/// list of `|` byte offsets; line 2 is the header separator and its dash runs
/// fill the same columns as the rows around it; each column is as wide as its
/// widest cell in characters; and the table is exactly the header, the separator
/// and the four fixture rows with their `steam/<appid>` lookup keys beside `-`
/// `Game Id` cells.
#[test]
fn test_cli_list_table_columns_align_under_a_separator() {
    let root = fixture_root();
    let (data, home) = sandbox("align");
    let fake = fake_empty();
    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root.to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
        ],
        &env(&data, &home),
        "",
    );
    assert!(run.success(), "stderr: {}", run.stderr);
    let stdout = run.stdout.clone();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        6,
        "the table is exactly header + separator + four fixture rows:\n{stdout}"
    );

    let offsets = pipe_offsets(lines[0]);
    assert_eq!(
        offsets.len(),
        6,
        "five columns render six delimiters — ADR-025 swapped the second column, \
         it did not add one: {:?}",
        lines[0]
    );
    for line in &lines {
        assert_eq!(
            pipe_offsets(line),
            offsets,
            "every rendered line must share the header's `|` offsets: {line:?}"
        );
    }

    // Line 2 is the separator, and it fills exactly the columns the header and
    // the body rows occupy.
    assert_is_separator(lines[1]);
    let runs = separator_runs(lines[1]);
    assert_eq!(runs.len(), 5, "one dash run per column: {:?}", lines[1]);
    for (run, gap) in runs.iter().zip(gaps(&offsets)) {
        assert_eq!(
            run.chars().count(),
            gap,
            "each separator run must fill its column: {:?}",
            lines[1]
        );
    }

    // T-003 §1: column width = max of `chars().count()` over the header cell and
    // that column's body cells, and each rendered cell is framed by one space
    // on each side.
    let header = cells(lines[0]);
    // `cells` from the harness trims; re-split untrimmed for the framing check.
    let untrimmed: Vec<Vec<String>> = lines
        .iter()
        .map(|line| {
            let inner = line
                .strip_prefix('|')
                .and_then(|rest| rest.strip_suffix('|'))
                .unwrap_or(line);
            inner.split('|').map(str::to_string).collect()
        })
        .collect();
    let rows: Vec<Vec<String>> = untrimmed[2..].to_vec();
    for row in &rows {
        assert_eq!(row.len(), 5, "every row has exactly five cells: {row:?}");
        assert!(
            row.iter()
                .all(|cell| cell.starts_with(' ') && cell.ends_with(' ')),
            "every cell must be framed by a space on each side: {row:?}"
        );
    }
    for cell in &untrimmed[0] {
        assert!(
            cell.starts_with(' ') && cell.ends_with(' '),
            "every header cell must be framed by a space on each side: {:?}",
            untrimmed[0]
        );
    }
    let column_gaps: Vec<usize> = gaps(&offsets);
    assert_eq!(
        column_gaps.len(),
        5,
        "the six delimiters enclose five columns"
    );
    for index in 0..5 {
        let widest = rows
            .iter()
            .map(|row| row[index].trim().chars().count())
            .max()
            .unwrap_or(0)
            .max(header[index].chars().count());
        assert_eq!(
            column_gaps[index],
            widest + 2,
            "column {index} must be as wide as its widest cell plus one space on \
             each side (widest cell: {:?})",
            index
        );
    }

    // And those four rows are the four fixture games, rendered with the ADR-025
    // `steam/<appid>` lookup key beside an unresolved `-` `Game Id`.
    assert_eq!(
        rows.iter()
            .map(|row| (row[0].trim(), row[1].trim()))
            .collect::<Vec<(&str, &str)>>(),
        vec![
            ("steam/900001", "-"),
            ("steam/900002", "-"),
            ("steam/900003", "-"),
            ("steam/900100", "-"),
        ],
        "the table must contain exactly the four fixture rows, each lookup key \
         beside its unresolved id"
    );
    for line in &lines[2..] {
        assert!(
            !line.contains("unknown"),
            "no body row may carry the retired namespace value: {line:?}"
        );
    }
}

/// proves: T-003 Test plan re-pinned — `list --steam-root
/// /bindforge-no-such-steam-root` behind the gate prints exactly the header
/// block (header + separator) with no body row and no "(no games found)" copy,
/// and nothing on stderr; the header names the five ADR-025 columns.
#[test]
fn test_cli_list_table_is_the_header_block_when_no_games_are_found() {
    let (data, home) = sandbox("empty");
    let fake = fake_empty();
    let run = run_cli(
        &[
            "list",
            "--steam-root",
            "/bindforge-no-such-steam-root",
            "--catalog-url",
            &fake.url(),
        ],
        &env(&data, &home),
        "",
    );
    assert!(
        run.success(),
        "a missing root is an empty result, not a failure, stderr: {}",
        run.stderr
    );
    assert!(
        run.stderr.is_empty(),
        "an empty result writes nothing on stderr, got: {}",
        run.stderr
    );

    let stdout = run.stdout.clone();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "the empty table is the header and the separator only:\n{stdout}"
    );
    let header = cells(lines[0]);
    assert_eq!(
        header,
        vec![
            "Slug",
            "Game Id",
            "Display Name",
            "Install Path",
            "Platform"
        ],
        "the header names the five columns ADR-025 renders: the lookup key beside \
         the confirmed game id"
    );
    assert_is_separator(lines[1]);
    assert_eq!(
        pipe_offsets(lines[1]),
        pipe_offsets(lines[0]),
        "the separator sits under the aligned header"
    );
    assert!(
        !stdout.contains("unknown"),
        "no body row may be printed:\n{stdout}"
    );
    assert!(
        !stdout.contains("no games") && stdout.lines().all(|line| line.starts_with('|')),
        "an empty table is still only the table, never prose copy:\n{stdout}"
    );
}

/// proves: design 025's Chosen design (`--catalog-url` is "a global clap flag
/// beside `--json`") and the operator follow-up on T-003 — `discovery --help` and
/// `discovery list --help` list `--json` and the new `--catalog-url`, and the
/// global position is accepted: `--json` before the subcommand prints the same
/// JSON array (behind the fake and the sandbox).
#[test]
fn test_help_lists_the_json_and_catalog_flags_at_every_level() {
    let top = run_cli(&["--help"], &[], "");
    assert!(
        top.success(),
        "`--help` must succeed, stderr: {}",
        top.stderr
    );
    for flag in ["--json", "--catalog-url"] {
        assert!(
            top.stdout.contains(flag),
            "the top-level help must list {flag}:\n{}",
            top.stdout
        );
    }

    let sub = run_cli(&["list", "--help"], &[], "");
    assert!(
        sub.success(),
        "`list --help` must succeed, stderr: {}",
        sub.stderr
    );
    for flag in ["--json", "--catalog-url"] {
        assert!(
            sub.stdout.contains(flag),
            "the `list` help must list {flag}:\n{}",
            sub.stdout
        );
    }

    // Global position: accepted before the subcommand, same JSON array.
    let root = fixture_root();
    let (data, home) = sandbox("global");
    let fake = fake_empty();
    let run = run_cli(
        &[
            "--json",
            "list",
            "--steam-root",
            root.to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
        ],
        &env(&data, &home),
        "",
    );
    assert!(
        run.success(),
        "`--json` before the subcommand must parse, stderr: {}",
        run.stderr
    );
    let games: Vec<serde_json::Value> = serde_json::from_str(run.stdout.trim())
        .expect("`--json list` must print a JSON array of DiscoveredGame objects");
    assert_eq!(games.len(), 4, "the fixture yields exactly four games");
}

/// proves: T-003 §3 Test plan, behind the migrated gate — a `--steam-root` whose
/// `steamapps/libraryfolders.vdf` is non-UTF-8 makes `discover()` return
/// `PluginError` (the §7 error path); the gate passed (the fake is listening), so
/// the failure is the plugin's: the CLI exits non-zero with exactly one stderr
/// line naming the plugin, leaves stdout empty, and never panics.
#[test]
fn test_cli_list_reports_a_plugin_failure_on_one_stderr_line() {
    let root = SteamRoot::new("polish-unreadable-root");
    // One selected root, no games and one file skipped as unreadable is the §7
    // error path, so `discover()` answers `PluginError`.
    std::fs::write(
        root.path().join("steamapps").join("libraryfolders.vdf"),
        [0xff_u8, 0xfe, 0x00, 0x01],
    )
    .expect("temp libraryfolders.vdf");
    let (data, home) = sandbox("pluginfail");
    let fake = fake_empty();

    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
        ],
        &env(&data, &home),
        "",
    );

    assert!(
        !run.success(),
        "a plugin failure must exit non-zero, stdout: {}",
        run.stdout
    );
    assert_eq!(
        run.code(),
        Some(1),
        "a plugin failure exits with `ExitCode::FAILURE`"
    );

    assert!(
        run.stdout.is_empty(),
        "a failed run writes no table on stdout, got: {}",
        run.stdout
    );

    assert!(
        !run.stderr.contains("panicked at"),
        "the failure must be a clean error path, never a panic: {}",
        run.stderr
    );
    assert!(
        !run.stderr.contains("Error from plugin"),
        "the CLI must not speak the engine's swallowed-error voice: {}",
        run.stderr
    );
    let lines = run.stderr_lines();
    assert_eq!(
        lines.len(),
        1,
        "exactly one stderr line names the failure: {}",
        run.stderr
    );
    assert!(
        lines[0].starts_with("error: plugin steam-linux failed: "),
        "the line must name the failed plugin in the CLI's own voice, got: {:?}",
        lines[0]
    );
    assert!(
        lines[0].contains("PluginError"),
        "the `DiscoveryError`'s Debug shape is quoted, got: {:?}",
        lines[0]
    );
}
