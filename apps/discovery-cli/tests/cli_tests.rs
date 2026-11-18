//! Integration tests for `discovery list`.
//!
//! Both tests drive the built binary through `env!("CARGO_BIN_EXE_discovery-cli")`
//! with `--steam-root` at the crate's shared read-only fixture tree, so no CLI
//! test probes the host or depends on the runner having Steam installed
//! (Test strategy #9, Chosen design §8/§9).

use std::path::{Path, PathBuf};
use std::process::Command;

/// The crate's checked-in fixture tree, shared with `crates/discovery` tests.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/discovery/tests/fixtures/steam/minimal")
}

/// The plugin emits canonical paths; expected paths are built the same way.
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).expect("fixture root must exist")
}

/// Runs the built `discovery-cli` and returns its stdout.
fn run_list(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_discovery-cli"))
        .args(args)
        .output()
        .expect("failed to execute discovery-cli");
    assert!(
        output.status.success(),
        "discovery-cli must exit successfully, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The cells of one rendered table line, split at its `|` delimiters. Each
/// cell keeps the padding spaces the renderer wrote for it (T-003 §1).
fn cells(line: &str) -> Vec<String> {
    let inner = line
        .strip_prefix('|')
        .and_then(|rest| rest.strip_suffix('|'))
        .unwrap_or(line);
    inner.split('|').map(str::to_string).collect()
}

/// The byte offsets of the `|` delimiters of one rendered table line (T-003 §1:
/// an aligned table gives every line the same list of offsets).
fn pipe_offsets(line: &str) -> Vec<usize> {
    line.bytes()
        .enumerate()
        .filter(|(_, byte)| *byte == b'|')
        .map(|(offset, _)| offset)
        .collect()
}

// proves: Test strategy #9 (Chosen design §8), as polished by T-003 §1 —
// `list --steam-root <fixture>` prints the padded table header and exactly the
// four expected fixture rows, with every header cell padded right to its
// column's width; the compatdata decoys never appear.
#[test]
fn test_cli_list_format() {
    let root = fixture_root();
    let stdout = run_list(&["list", "--steam-root", root.to_str().expect("utf-8 path")]);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        6,
        "the table is the header, the separator, and exactly four rows:\n{stdout}"
    );

    // T-003 §1: cells are padded, so the old unpadded header string is gone and
    // every header cell carries padding beyond its single leading space.
    assert!(
        !stdout.contains("| Slug | Display Name | Install Path | Platform |"),
        "the header must be padded to its columns' widths:\n{}",
        lines[0]
    );
    let header_cells = cells(lines[0]);
    assert_eq!(
        header_cells.len(),
        4,
        "the header names exactly four columns"
    );
    for (cell, column) in
        header_cells
            .iter()
            .zip(["Slug", "Display Name", "Install Path", "Platform"])
    {
        assert_eq!(
            cell.trim(),
            column,
            "the header must name the {column} column"
        );
        assert!(
            cell.starts_with(' ') && cell.chars().count() > column.chars().count() + 1,
            "the {column} header cell must be padded to its column width, got {cell:?}"
        );
    }
    // T-003 §1: the padded cells line up, so every rendered line has its `|`
    // characters at the same byte offsets.
    for line in &lines[1..] {
        assert_eq!(
            pipe_offsets(line),
            pipe_offsets(lines[0]),
            "every rendered line must share the header's `|` offsets: {line:?}"
        );
    }

    for slug in [
        "unknown/example-game",
        "unknown/mixed-case-game",
        "unknown/nameless-install",
        "unknown/secondary-title",
    ] {
        assert!(stdout.contains(slug), "table must contain the {slug} row");
    }
    for name in [
        "Example Game",
        "Mixed Case Game",
        "Nameless Install",
        "Secondary Title",
    ] {
        assert!(
            stdout.contains(name),
            "table must contain the {name} display name"
        );
    }
    assert!(
        !stdout.contains("prefix-decoy"),
        "compatdata decoys must never appear in the table"
    );
    assert_eq!(
        stdout.matches("unknown/").count(),
        4,
        "the table must contain exactly the four fixture rows"
    );
}

// proves: Test strategy #9 (Chosen design §8) — `list --steam-root <fixture> --json`
// prints exactly the four DiscoveredGame objects, in (slug, install_path) order,
// with the unchanged five-field JSON shape.
#[test]
fn test_cli_list_json_format() {
    let root_arg = fixture_root();
    let root = canonical(&root_arg);
    let stdout = run_list(&[
        "list",
        "--steam-root",
        root_arg.to_str().expect("utf-8 path"),
        "--json",
    ]);

    let games: Vec<serde_json::Value> = serde_json::from_str(stdout.trim())
        .expect("`list --json` must print a JSON array of DiscoveredGame objects");
    assert_eq!(games.len(), 4, "exactly the four fixture games");

    // §5/A2: each row's `install_path` composes from the library that holds its
    // manifest, so the secondary library's game carries the `<root>/secondary` prefix.
    let expected = [
        ("unknown/example-game", "Example Game", "Example Game", ""),
        (
            "unknown/mixed-case-game",
            "Mixed Case Game",
            "Mixed Case Game",
            "",
        ),
        (
            "unknown/nameless-install",
            "Nameless Install",
            "Nameless Install",
            "",
        ),
        (
            "unknown/secondary-title",
            "Secondary Title",
            "Secondary Title",
            "secondary",
        ),
    ];
    for (game, (slug, display_name, install_dir, library)) in games.iter().zip(expected) {
        let obj = game.as_object().expect("each game is a JSON object");
        assert_eq!(obj.len(), 5, "DiscoveredGame has exactly five fields");
        assert_eq!(obj["slug"], slug);
        assert_eq!(obj["display_name"], display_name);
        // Chosen design §8 / Context: T-001 changes no serialized shape, so
        // `platform` keeps serde's default variant-name spelling (the r1 major
        // `#[serde(rename_all = "kebab-case")]` was reverted as unauthorized).
        assert_eq!(obj["platform"], "SteamLinux");
        assert!(obj["codec_id"].is_null(), "codec_id must be null");
        let library_root = if library.is_empty() {
            root.clone()
        } else {
            canonical(&root.join(library))
        };
        let want = library_root
            .join("steamapps")
            .join("common")
            .join(install_dir);
        assert_eq!(
            obj["install_path"],
            want.to_string_lossy().as_ref(),
            "install_path must be the composed steamapps/common path under the canonicalized library that holds the manifest"
        );
    }
}

// proves: Test strategy #9 (Chosen design §7/§8) — `list --json` prints `[]`
// for a `--steam-root` that does not exist.
#[test]
fn test_cli_list_json_empty_for_missing_steam_root() {
    let stdout = run_list(&[
        "list",
        "--json",
        "--steam-root",
        "/bindforge-no-such-steam-root",
    ]);
    assert_eq!(stdout.trim(), "[]");
}
