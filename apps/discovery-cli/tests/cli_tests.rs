//! Integration tests for `discovery list` against the checked-in fixture tree —
//! the Linux-regression gate (design 024 Test strategy #9, re-pinned by design
//! 025's Test strategy #12/#13 and Rollout 3).
//!
//! ADR-025 moved the CLI behind two seams, so every run here is migrated to the
//! shared harness: a loopback `TcpListener` fake answers the gate and the
//! candidate queries, and a temp `XDG_DATA_HOME`/`HOME` sandbox keeps every store
//! byte inside the test. The fake serves no candidates, so the runs are pure
//! rendering: every `Game Id` cell is `-` and no prompt is earned.

mod common;

use common::{FakeCatalog, TempDir, run_cli};
use std::path::{Path, PathBuf};

/// The crate's checked-in fixture tree, shared with `crates/discovery` tests.
fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/discovery/tests/fixtures/steam/minimal")
}

/// The plugin emits canonical paths; expected paths are built the same way.
fn canonical(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).expect("fixture root must exist")
}

/// A fake that serves no candidates to any query.
fn fake_empty() -> FakeCatalog {
    FakeCatalog::start("cli-fake-empty", |_| common::ok("[]"))
}

/// The sandbox pair every fixture run gets, so the developer's real store is
/// never read or written (Rollout 3 migration).
fn sandbox(label: &str) -> (TempDir, TempDir) {
    (
        TempDir::new(&format!("cli-{label}-data")),
        TempDir::new(&format!("cli-{label}-home")),
    )
}

fn env<'a>(data: &'a TempDir, home: &'a TempDir) -> [(&'a str, &'a str); 2] {
    [
        ("XDG_DATA_HOME", data.path().to_str().expect("utf-8")),
        ("HOME", home.path().to_str().expect("utf-8")),
    ]
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

/// proves: design 025's Test strategy #12 and the re-pinned output contract
/// (Rollout 3: the retired namespace column out, `Game Id` in) — `list --steam-root
/// <fixture>` behind the gate prints the padded five-column table, now `Slug |
/// Game Id | Display Name | Install Path | Platform`, with the four fixture rows
/// in slug order rendering `steam/<appid>` lookup keys and `-` `Game Id` cells;
/// the namespace token and the compatdata decoys appear nowhere.
#[test]
fn test_cli_list_format() {
    let root = fixture_root();
    let (data, home) = sandbox("fmt");
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
        "the table is the header, the separator, and exactly four rows:\n{stdout}"
    );

    // T-003 §1: cells are padded to their columns' widths, so the unpadded
    // header string is gone, and every line shares the header's `|` offsets.
    assert!(
        !stdout.contains("| Slug | Game Id | Display Name | Install Path | Platform |"),
        "the header must be padded to its columns' widths:\n{}",
        lines[0]
    );
    let header_cells = common::cells(lines[0]);
    assert_eq!(
        header_cells
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>(),
        vec![
            "Slug",
            "Game Id",
            "Display Name",
            "Install Path",
            "Platform",
        ],
        "the header names exactly the five ADR-025 columns: the lookup key beside \
         the confirmed id, the retired namespace column gone"
    );
    for line in &lines[1..] {
        assert_eq!(
            pipe_offsets(line),
            pipe_offsets(lines[0]),
            "every rendered line must share the header's `|` offsets: {line:?}"
        );
    }

    let rows: Vec<Vec<String>> = lines[2..].iter().map(|line| common::cells(line)).collect();
    assert_eq!(rows.len(), 4, "exactly the four fixture rows are rendered");
    let rendered: Vec<(&str, &str, &str)> = rows
        .iter()
        .map(|row| (row[0].as_str(), row[1].as_str(), row[2].as_str()))
        .collect();
    assert_eq!(
        rendered,
        vec![
            ("steam/900001", "-", "Example Game"),
            ("steam/900002", "-", "Nameless Install"),
            ("steam/900003", "-", "Mixed Case Game"),
            ("steam/900100", "-", "Secondary Title"),
        ],
        "the slug, game id and display name cells, in (slug, install_path) order:\n{stdout}"
    );
    for row in &rows {
        assert!(
            row[0].starts_with("steam/"),
            "the `Slug` cell is the `<source>/<external-id>` lookup key, got {:?}",
            row[0]
        );
        assert!(
            !row[3].contains("unknown/"),
            "the install path carries no namespace segment, got {:?}",
            row[3]
        );
    }

    // The retired namespace spelling is gone from the whole rendering, table and
    // paths alike — the column that carried it no longer exists.
    assert!(
        !stdout.contains("unknown"),
        "the retired namespace value appears nowhere in the rendering:\n{stdout}"
    );
    assert!(
        !stdout.contains("prefix-decoy"),
        "compatdata decoys must never appear in the table"
    );
}

/// proves: design 025's Test strategy #13 over the fixture tree — `list
/// --steam-root <fixture> --json` behind the gate prints exactly the four
/// DiscoveredGame objects in (slug, install_path) order with the re-pinned
/// eight-field shape: `friendly_name` sits beside `slug`, `game_id` is `null` and
/// `candidates` `[]` with no candidates served, and the retired `namespace` key
/// is absent.
#[test]
fn test_cli_list_json_format() {
    let root_arg = fixture_root();
    let root = canonical(&root_arg);
    let (data, home) = sandbox("json");
    let fake = fake_empty();
    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root_arg.to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
            "--json",
        ],
        &env(&data, &home),
        "",
    );
    assert!(run.success(), "stderr: {}", run.stderr);

    let games: Vec<serde_json::Value> = serde_json::from_str(run.stdout.trim())
        .expect("`list --json` must print a JSON array of DiscoveredGame objects");
    assert_eq!(games.len(), 4, "exactly the four fixture games");

    // §5/A2: each row's `install_path` composes from the library that holds its
    // manifest, so the secondary library's game carries the `<root>/secondary` prefix.
    let expected = [
        ("steam/900001", "Example Game", "Example Game", ""),
        ("steam/900002", "Nameless Install", "Nameless Install", ""),
        ("steam/900003", "Mixed Case Game", "Mixed Case Game", ""),
        (
            "steam/900100",
            "Secondary Title",
            "Secondary Title",
            "secondary",
        ),
    ];
    for (game, (slug, friendly_name, display_name, library)) in games.iter().zip(expected) {
        let obj = game.as_object().expect("each game is a JSON object");
        assert_eq!(
            obj.len(),
            8,
            "DiscoveredGame has exactly eight fields: ADR-025 adds `friendly_name` \
             and `game_id`/`candidates` and drops `namespace`"
        );
        assert!(
            obj.get("namespace").is_none(),
            "the retired `namespace` key must be absent: {}",
            game
        );
        assert_eq!(obj["slug"], slug, "the JSON slug is the lookup key");
        assert_eq!(
            obj["friendly_name"], friendly_name,
            "`friendly_name` is discovery output beside the slug"
        );
        assert_eq!(obj["display_name"], display_name);
        // Chosen design §8 / Context: ADR-025 changes the row's field *set*, not
        // `Platform`'s spelling, so `platform` keeps serde's default variant-name
        // spelling; the mixed spelling of the two closed enums is by design.
        assert_eq!(obj["platform"], "SteamLinux");
        assert!(obj["codec_id"].is_null(), "codec_id must be null");
        assert!(
            obj["game_id"].is_null(),
            "an unmatched row serializes `game_id: null`"
        );
        assert_eq!(
            obj["candidates"],
            serde_json::json!([]),
            "the fake served no candidates"
        );
        let library_root = if library.is_empty() {
            root.clone()
        } else {
            canonical(&root.join(library))
        };
        let want = library_root
            .join("steamapps")
            .join("common")
            .join(display_name);
        assert_eq!(
            obj["install_path"],
            want.to_string_lossy().as_ref(),
            "install_path must be the composed steamapps/common path under the \
             canonicalized library that holds the manifest"
        );
        assert!(
            !obj["install_path"]
                .as_str()
                .unwrap_or_default()
                .contains("unknown"),
            "an install path carries no namespace segment: {}",
            obj["install_path"]
        );
    }
}

/// proves: design 025 keeps the pre-ADR contract that `list --json` prints `[]`
/// for a `--steam-root` that does not exist — the gate is satisfied by the fake,
/// the store is the empty sandbox default, and the document is the empty array.
#[test]
fn test_cli_list_json_empty_for_missing_steam_root() {
    let (data, home) = sandbox("missing");
    let fake = fake_empty();
    let run = run_cli(
        &[
            "list",
            "--json",
            "--steam-root",
            "/bindforge-no-such-steam-root",
            "--catalog-url",
            &fake.url(),
        ],
        &env(&data, &home),
        "",
    );
    assert!(run.success(), "stderr: {}", run.stderr);
    assert_eq!(run.stdout.trim(), "[]");
}
