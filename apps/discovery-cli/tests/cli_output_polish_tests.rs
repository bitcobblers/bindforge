//! T-003 output-polish pins for `discovery list`: an aligned ADR-022 table, the
//! header-only empty table, and the clean plugin-failure exit (T-003 Chosen
//! design §1/§3, Test plan).
//!
//! Every test drives the built binary through
//! `env!("CARGO_BIN_EXE_discovery-cli")` against a fixture or a temp tree it
//! creates and removes itself, so no test probes the host (repo practice:
//! integration tests use temp directories and clean up after themselves).

use std::path::{Path, PathBuf};
use std::process::Command;

/// The crate's checked-in fixture tree, shared with `crates/discovery` tests.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/discovery/tests/fixtures/steam/minimal")
}

/// Runs the built `discovery-cli` with `args` and returns its raw output.
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_discovery-cli"))
        .args(args)
        .output()
        .expect("failed to execute discovery-cli")
}

/// The stdout of a successful run, as text.
fn stdout_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The stderr of a run, as text.
fn stderr_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
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

/// The cells of one rendered table line, split at its `|` delimiters. Each cell
/// keeps the padding spaces the renderer wrote for it.
fn cells(line: &str) -> Vec<String> {
    let inner = line
        .strip_prefix('|')
        .and_then(|rest| rest.strip_suffix('|'))
        .unwrap_or(line);
    inner.split('|').map(str::to_string).collect()
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

// proves: T-003 Test plan — for `list --steam-root <fixture>`, every stdout line
// has the identical list of `|` byte offsets; line 2 is the header separator and
// its dash runs fill the same columns as the rows around it; each column is as
// wide as its widest cell in characters; and the table is exactly the header, the
// separator and the four fixture rows.
#[test]
fn test_cli_list_table_columns_align_under_a_separator() {
    let root = fixture_root();
    let output = run(&["list", "--steam-root", root.to_str().expect("utf-8 path")]);
    assert!(
        output.status.success(),
        "discovery-cli must exit successfully, stderr: {}",
        stderr_of(&output)
    );
    let stdout = stdout_of(&output);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        6,
        "the table is exactly header + separator + four fixture rows:\n{stdout}"
    );

    let offsets = pipe_offsets(lines[0]);
    assert_eq!(
        offsets.len(),
        5,
        "four columns render five delimiters: {:?}",
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
    assert_eq!(runs.len(), 4, "one dash run per column: {:?}", lines[1]);
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
    let rows: Vec<Vec<String>> = lines[2..].iter().map(|line| cells(line)).collect();
    for row in &rows {
        assert_eq!(row.len(), 4, "every row has exactly four cells: {row:?}");
        assert!(
            row.iter()
                .all(|cell| cell.starts_with(' ') && cell.ends_with(' ')),
            "every cell must be framed by a space on each side: {row:?}"
        );
    }
    for cell in &header {
        assert!(
            cell.starts_with(' ') && cell.ends_with(' '),
            "every header cell must be framed by a space on each side: {header:?}"
        );
    }
    let column_gaps: Vec<usize> = gaps(&offsets);
    for index in 0..4 {
        let widest = rows
            .iter()
            .map(|row| row[index].trim().chars().count())
            .max()
            .unwrap_or(0)
            .max(header[index].trim().chars().count());
        assert_eq!(
            column_gaps[index],
            widest + 2,
            "column {index} must be as wide as its widest cell plus one space on each side"
        );
    }

    // And those four rows are the four fixture games.
    assert_eq!(
        stdout.matches("unknown/").count(),
        4,
        "the table must contain exactly the four fixture rows"
    );
    for line in &lines[2..] {
        assert!(
            line.contains("unknown/"),
            "every body row is a fixture row: {line:?}"
        );
    }
}

// proves: T-003 Test plan — `list --steam-root /bindforge-no-such-steam-root`
// prints exactly the header block (header + separator) with no body row and no
// "(no games found)" copy, and nothing on stderr.
#[test]
fn test_cli_list_table_is_the_header_block_when_no_games_are_found() {
    let output = run(&["list", "--steam-root", "/bindforge-no-such-steam-root"]);
    assert!(
        output.status.success(),
        "a missing root is an empty result, not a failure, stderr: {}",
        stderr_of(&output)
    );
    assert!(
        stderr_of(&output).is_empty(),
        "an empty result writes nothing on stderr, got: {}",
        stderr_of(&output)
    );

    let stdout = stdout_of(&output);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "the empty table is the header and the separator only:\n{stdout}"
    );
    let header = cells(lines[0]);
    assert_eq!(
        header.iter().map(|cell| cell.trim()).collect::<Vec<&str>>(),
        vec!["Slug", "Display Name", "Install Path", "Platform"],
        "the header names the four ADR-022 columns"
    );
    assert_is_separator(lines[1]);
    assert_eq!(
        pipe_offsets(lines[1]),
        pipe_offsets(lines[0]),
        "the separator sits under the aligned header"
    );
    assert!(
        !stdout.contains("unknown/"),
        "no body row may be printed:\n{stdout}"
    );
    assert!(
        !stdout.contains("no games") && stdout.lines().all(|line| line.starts_with('|')),
        "an empty table is still only the table, never prose copy:\n{stdout}"
    );
}

// proves: operator follow-up on T-003 — `--json` is a global flag, so both
// `discovery --help` and `discovery list --help` list it, and it is accepted
// before or after the `list` subcommand.
#[test]
fn test_help_lists_the_json_flag_at_every_level() {
    let top = run(&["--help"]);
    assert!(
        top.status.success(),
        "`--help` must succeed, stderr: {}",
        stderr_of(&top)
    );
    assert!(
        stdout_of(&top).contains("--json"),
        "the top-level help must list --json:\n{}",
        stdout_of(&top)
    );

    let sub = run(&["list", "--help"]);
    assert!(
        sub.status.success(),
        "`list --help` must succeed, stderr: {}",
        stderr_of(&sub)
    );
    assert!(
        stdout_of(&sub).contains("--json"),
        "the `list` help must list --json:\n{}",
        stdout_of(&sub)
    );

    // Global position: accepted before the subcommand, same JSON array.
    let root = fixture_root();
    let output = run(&[
        "--json",
        "list",
        "--steam-root",
        root.to_str().expect("utf-8 path"),
    ]);
    assert!(
        output.status.success(),
        "`--json` before the subcommand must parse, stderr: {}",
        stderr_of(&output)
    );
    let games: Vec<serde_json::Value> = serde_json::from_str(stdout_of(&output).trim())
        .expect("`--json list` must print a JSON array of DiscoveredGame objects");
    assert_eq!(games.len(), 4, "the fixture yields exactly four games");
}

// proves: T-003 §3 Test plan — a `--steam-root` whose
// `steamapps/libraryfolders.vdf` is non-UTF-8 makes `discover()` return
// `PluginError` (the §7 error path); the CLI exits non-zero with exactly one
// stderr line naming the plugin, leaves stdout empty, and never panics.
#[test]
fn test_cli_list_reports_a_plugin_failure_on_one_stderr_line() {
    let root = std::env::temp_dir().join(format!(
        "bindforge-t003-unreadable-root-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("steamapps")).expect("temp steamapps directory");
    // One selected root, no games and one file skipped as unreadable is the §7
    // error path, so `discover()` answers `PluginError`.
    std::fs::write(
        root.join("steamapps").join("libraryfolders.vdf"),
        [0xff_u8, 0xfe, 0x00, 0x01],
    )
    .expect("temp libraryfolders.vdf");

    let output = run(&["list", "--steam-root", root.to_str().expect("utf-8 path")]);
    match std::fs::remove_dir_all(&root) {
        Ok(()) => {}
        Err(error) => panic!("failed to clean up {}: {error}", root.display()),
    }

    assert!(
        !output.status.success(),
        "a plugin failure must exit non-zero, stdout: {}",
        stdout_of(&output)
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "a plugin failure exits with `ExitCode::FAILURE`"
    );

    let stdout = stdout_of(&output);
    assert!(
        stdout.is_empty(),
        "a failed run writes no table on stdout, got: {stdout}"
    );

    let stderr = stderr_of(&output);
    assert!(
        !stderr.contains("panicked at"),
        "the failure must be a clean error path, never a panic: {stderr}"
    );
    assert!(
        !stderr.contains("Error from plugin"),
        "the CLI must not speak the engine's swallowed-error voice: {stderr}"
    );
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "exactly one stderr line names the failure: {stderr}"
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
