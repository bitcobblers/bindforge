//! The per-user match store of design 025 / ADR-025 (`025-C3`, `025-L4`): one JSON
//! document at `<data>/bindforge/matches.json`, its path rule, its load-time
//! validation, and save-only-on-confirmation — driven end-to-end against the built
//! CLI with a temp `XDG_DATA_HOME`/`HOME` sandbox and the loopback catalog fake.
//!
//! proves: design 025's Test strategy #5 (store path: an absolute `$XDG_DATA_HOME`
//! wins, a relative value is treated as unset and falls back to `$HOME/.local/share`
//! — the Windows `%APPDATA%` branch is the same rule through the implementation's
//! env-lookup closure and is pinned in-module there, per the Chosen design's env
//! seam; this suite pins the Linux rule through the real environment), #6 (store
//! load: absent is empty; valid resolves without queries; truncated JSON, an unknown
//! `version`, an over-budget file, an out-of-shape or over-bound `slug`, and a
//! `game_id` of `../x` each produce the named refusal naming the path with the file
//! bytes untouched — and a newline in `$XDG_DATA_HOME` still yields one escaped
//! line), and #7 (store save: nothing without a confirmation; records sorted by
//! `slug`; canonical bytes for the same record set; no `.tmp` left behind; the
//! directory and file owner-only on unix).
//!
//! Integration (CLI, subprocess) per repo practice: temp directories cleaned up on
//! drop; the fake lives in the test process; no non-loopback request.

mod common;

use common::{FakeCatalog, SteamRoot, TempDir, candidates_body, cells, ok, run_cli, store_path};
use std::path::Path;

/// The candidate list every store test serves for any queried slug.
fn two_candidates() -> String {
    candidates_body(&[("cand-one", "First Match"), ("cand-two", "Second Match")])
}

fn fake_catalog() -> FakeCatalog {
    let body = two_candidates();
    FakeCatalog::start("store-fake", move |_request| ok(&body))
}

fn fake_no_candidates() -> FakeCatalog {
    FakeCatalog::start("store-fake-empty", |_| ok("[]"))
}

/// One JSON `MatchStore` document as the design spells the file shape:
/// `{ "version": 1, "matches": [ { slug, game_id, friendly_name } ] }`.
fn store_json(records: &[(&str, &str, &str)]) -> String {
    let entries: Vec<String> = records
        .iter()
        .map(|(slug, game_id, friendly_name)| {
            format!(
                "{{\"slug\":\"{slug}\",\"game_id\":\"{game_id}\",\
                 \"friendly_name\":\"{friendly_name}\"}}"
            )
        })
        .collect();
    format!("{{\"version\":1,\"matches\":[{}]}}", entries.join(","))
}

fn run_table(root: &SteamRoot, envs: &[(&str, &str)], url: &str, stdin: &str) -> common::Run {
    let mut args = vec![
        "list",
        "--steam-root",
        root.path().to_str().expect("utf-8 path"),
    ];
    args.push("--catalog-url");
    args.push(url);
    run_cli(&args, envs, stdin)
}

/// A one-game root plus its sandbox pair, shared by the path-rule tests.
fn one_game(label: &str) -> SteamRoot {
    let root = SteamRoot::new(label);
    root.acf("5001", Some("Example Game"), "Example Game");
    root
}

fn env<'a>(xdg: &'a Path, home: &'a Path) -> [(&'a str, &'a str); 2] {
    [
        ("XDG_DATA_HOME", xdg.to_str().expect("utf-8")),
        ("HOME", home.to_str().expect("utf-8")),
    ]
}

/// proves: design 025's Test strategy #5 (Chosen design: Match store — "`<data>` is
/// `$XDG_DATA_HOME` when set and absolute") — an absolute `XDG_DATA_HOME` owns the
/// store: the file lands under it, and the `$HOME/.local/share` fallback stays empty.
#[test]
fn an_absolute_xdg_data_home_owns_the_store_file() {
    let root = one_game("store-xdg");
    let xdg = TempDir::new("store-xdg-xdg");
    let home = TempDir::new("store-xdg-home");
    let fake = fake_catalog();

    let run = run_table(&root, &env(xdg.path(), home.path()), &fake.url(), "1\n");
    assert!(run.success(), "a confirmation run succeeds: {}", run.stderr);

    assert!(
        store_path(xdg.path()).is_file(),
        "the store must land at `<XDG_DATA_HOME>/bindforge/matches.json`: {:?}",
        store_path(xdg.path())
    );
    assert!(
        !home
            .path()
            .join(".local")
            .join("share")
            .join("bindforge")
            .join("matches.json")
            .exists(),
        "an absolute `XDG_DATA_HOME` replaces the `$HOME/.local/share` fallback"
    );
}

/// proves: design 025's Test strategy #5 (Chosen design — "(else `$HOME/.local/
/// share`) … a relative value being unset") — a set-but-relative `XDG_DATA_HOME` is
/// treated as unset: the store falls back to `$HOME/.local/share/bindforge/`.
#[test]
fn a_relative_xdg_data_home_is_treated_as_unset_and_home_wins() {
    let root = one_game("store-rel");
    let home = TempDir::new("store-rel-home");
    let fake = fake_catalog();

    let run = run_table(
        &root,
        &[
            ("XDG_DATA_HOME", "relative-store-dir"),
            ("HOME", home.path().to_str().expect("utf-8")),
        ],
        &fake.url(),
        "1\n",
    );
    assert!(
        run.success(),
        "a relative XDG value is not an error, it is unset: {}",
        run.stderr
    );
    assert!(
        store_path(&home.path().join(".local").join("share")).is_file(),
        "the fallback rule must place the store under `$HOME/.local/share`"
    );
}

/// proves: design 025's Test strategy #5 — with `XDG_DATA_HOME` unset entirely, the
/// same `$HOME/.local/share` fallback places the store.
#[test]
fn the_store_rides_home_when_xdg_data_home_is_unset() {
    let root = one_game("store-home");
    let home = TempDir::new("store-home-dir");
    let fake = fake_catalog();

    let run = run_table(
        &root,
        &[("HOME", home.path().to_str().expect("utf-8"))],
        &fake.url(),
        "1\n",
    );
    assert!(run.success(), "stderr: {}", run.stderr);
    assert!(
        store_path(&home.path().join(".local").join("share")).is_file(),
        "with no XDG value the fallback rule must place the store under `$HOME`"
    );
}

/// proves: design 025's Test strategy #6 (Chosen design: Match store — "an absent
/// file is an empty store") — no file, no confirmation: the run is clean and no
/// store is written (an absent store is a valid initial state; the file appears on
/// the first confirmation only).
#[test]
fn an_absent_store_is_an_empty_store_and_stays_absent() {
    let root = one_game("store-absent");
    let data = TempDir::new("store-absent-data");
    let fake = fake_no_candidates();

    let run = run_table(&root, &env(data.path(), data.path()), &fake.url(), "");
    assert!(run.success(), "stderr: {}", run.stderr);
    assert!(
        !store_path(data.path()).exists(),
        "nothing is stored without a confirmation (`025-L4`)"
    );
    assert!(
        data.files_below("bindforge").is_empty(),
        "the store directory must not be stood up by a run that confirms nothing"
    );
}

/// proves: design 025's Test strategy #6 (Chosen design: "`025-L4` — a slug the
/// store resolves is never queried, and records are re-trusted on every scan") — a
/// valid store file resolves its slug: the `Game Id` cell prints the stored id, the
/// fake records zero candidate requests for it, and the run rewrites nothing.
#[test]
fn a_valid_store_resolves_its_slugs_without_queries_or_rewrites() {
    let root = one_game("store-valid");
    let data = TempDir::new("store-valid-data");
    let bytes = store_json(&[("steam/5001", "example-game", "Example Game")]);
    data.write("bindforge/matches.json", bytes.as_bytes());
    let fake = fake_no_candidates();

    let run = run_table(&root, &env(data.path(), data.path()), &fake.url(), "");
    assert!(run.success(), "stderr: {}", run.stderr);

    let lines: Vec<&str> = run.stdout.lines().collect();
    let row = cells(lines[2]);
    assert_eq!(row[0], "steam/5001");
    assert_eq!(
        row[1], "example-game",
        "the store's confirmed id renders in the `Game Id` column"
    );
    assert_eq!(
        fake.request_count(),
        0,
        "a slug the store resolves is never queried (`025-L4`): {:?}",
        fake.requests()
    );
    let after = std::fs::read(store_path(data.path())).expect("the store survives");
    assert_eq!(
        after,
        bytes.as_bytes(),
        "a run with no new confirmation rewrites nothing"
    );
}

/// proves: design 025's Test strategy #6 (Chosen design: load-time refusals — "the
/// named refusal `error: match store unreadable: <path>` before enumeration, and
/// nothing on disk is rewritten") — a truncated JSON body, `version: 2`, a file
/// over the `65_536`-octet read budget, a slug past `025-B1`, a slug spelled outside
/// `<token>/<external-id>`, and a `game_id` of `../x` each refuse by name, name the
/// path, exit non-zero, print no table, and leave the file's bytes untouched.
#[test]
fn a_broken_store_refuses_by_name_before_enumerating_and_unchanged() {
    // "steam/" + 130 digits: one octet past the `025-B1` ceiling.
    let over_bound_slug = format!("steam/{}", "9".repeat(130));
    assert!(over_bound_slug.len() > bindforge_core::MAX_DISCOVERY_SLUG_OCTETS);
    let padded = format!(
        "{{\"version\":1,\"matches\":[],\"pad\":\"{}\"}}",
        "p".repeat(70_000)
    );

    let broken: [(&str, Vec<u8>); 6] = [
        (
            "truncated-json",
            b"{\"version\":1,\"matches\":[{\"slug\":".to_vec(),
        ),
        ("version-two", b"{\"version\":2,\"matches\":[]}".to_vec()),
        ("over-budget", padded.into_bytes()),
        (
            "slug-past-bound",
            store_json(&[(&over_bound_slug, "example-game", "Example Game")]).into_bytes(),
        ),
        (
            "slug-unspelled",
            store_json(&[("example-game", "example-game", "Example Game")]).into_bytes(),
        ),
        (
            "game-id-traversal",
            store_json(&[("steam/5001", "../x", "Example Game")]).into_bytes(),
        ),
    ];

    for (label, bytes) in broken {
        let root = one_game(&format!("store-broken-{label}"));
        let data = TempDir::new(&format!("store-broken-{label}-data"));
        data.write("bindforge/matches.json", &bytes);
        let fake = fake_no_candidates();

        let run = run_table(&root, &env(data.path(), data.path()), &fake.url(), "");
        assert!(
            !run.success(),
            "the {label} store must fail the run, stdout: {}",
            run.stdout
        );
        let lines = run.stderr_lines();
        assert_eq!(
            lines.len(),
            1,
            "the {label} refusal is exactly one named line: {:?}",
            run.stderr
        );
        assert!(
            lines[0].starts_with("error: match store unreadable:"),
            "the {label} store speaks `error: match store unreadable: <path>`, got: {:?}",
            lines[0]
        );
        assert!(
            lines[0].contains("matches.json"),
            "the refusal names the store path the user must fix or remove: {:?}",
            lines[0]
        );
        assert!(
            run.stdout.is_empty(),
            "a broken store refuses before enumeration — no table is printed for \
             `{label}`: {:?}",
            run.stdout
        );
        assert_eq!(
            std::fs::read(store_path(data.path())).expect("the file is still there"),
            bytes,
            "the refusal rewrites nothing on disk (`{label}`): the user's remedy is to \
             remove the file, not to have it re-mangled"
        );
        assert_eq!(
            fake.request_count(),
            0,
            "the refusal lands before enumeration — no catalog query for `{label}`"
        );
    }
}

/// proves: design 025's Test strategy #6, last clause — "a newline in
/// `$XDG_DATA_HOME` still yields one escaped line": every value a refusal
/// interpolates goes through `escape_and_truncate`, so an outside path cannot forge
/// a second stderr line.
#[test]
fn a_newline_in_the_data_home_path_still_yields_one_escaped_line() {
    let root = one_game("store-newline");
    let outer = TempDir::new("store-newline-outer");
    // One sandbox directory whose name carries a newline: the refusal must echo it
    // escaped, on a single line.
    let data_home = outer.join("xdg\nhome");
    std::fs::create_dir_all(data_home.join("bindforge")).expect("store dir");
    std::fs::write(
        data_home.join("bindforge").join("matches.json"),
        b"{ broken",
    )
    .expect("broken store");
    let home = outer.join("home");
    std::fs::create_dir_all(&home).expect("home dir");
    let fake = fake_no_candidates();

    let xdg = data_home.to_str().expect("utf-8").to_string();
    let home_s = home.to_str().expect("utf-8").to_string();
    let run = run_table(
        &root,
        &[("XDG_DATA_HOME", &xdg), ("HOME", &home_s)],
        &fake.url(),
        "",
    );
    assert!(!run.success(), "a broken store is a refused run");
    let lines = run.stderr_lines();
    assert_eq!(
        lines.len(),
        1,
        "the path's newline must be escaped, never emitted raw to forge a line: {:?}",
        run.stderr
    );
    assert!(
        lines[0].starts_with("error: match store unreadable:"),
        "the named refusal still speaks: {:?}",
        lines[0]
    );
}

/// proves: design 025's Test strategy #7 (Chosen design: "Save: only after at least
/// one confirmation") — a declined prompt writes nothing: no store file and no
/// store directory.
#[test]
fn a_declined_confirmation_writes_no_store() {
    let root = one_game("store-decline");
    let data = TempDir::new("store-decline-data");
    let fake = fake_catalog();

    let run = run_table(&root, &env(data.path(), data.path()), &fake.url(), "n\n");
    assert!(run.success(), "declining is a clean run: {}", run.stderr);
    assert!(
        !store_path(data.path()).exists(),
        "`n` is no match: nothing is ever stored without the user's choice (`025-L2`)"
    );
}

/// proves: design 025's Test strategy #7 (Chosen design: file shape — "records
/// sorted by `slug` byte order so a re-save is byte-stable; the store holds one
/// record per distinct slug — two installs of one game are two records sharing one
/// `game_id`") — one confirmation per distinct slug writes one record each, in byte
/// order of `slug`, with the design's three fields and `version: 1`.
#[test]
fn confirmations_write_one_record_per_slug_sorted_in_byte_order() {
    let root = SteamRoot::new("store-sort");
    root.acf("5002", Some("Second Game"), "Second Game");
    root.acf("5001", Some("First Game"), "First Game");
    let data = TempDir::new("store-sort-data");
    let fake = fake_catalog();

    let run = run_table(&root, &env(data.path(), data.path()), &fake.url(), "1\n1\n");
    assert!(run.success(), "stderr: {}", run.stderr);

    let document: serde_json::Value = std::fs::read_to_string(store_path(data.path()))
        .expect("the store lands")
        .parse()
        .expect("the store is JSON");
    assert_eq!(
        document["version"].as_i64(),
        Some(1),
        "the store carries the design's `version: 1`"
    );
    let records = document["matches"].as_array().expect("a matches array");
    assert_eq!(
        records
            .iter()
            .map(|record| record["slug"].as_str().unwrap_or_default())
            .collect::<Vec<&str>>(),
        vec!["steam/5001", "steam/5002"],
        "records are sorted by `slug` byte order, not confirmation order"
    );
    for record in records {
        assert_eq!(
            record["game_id"].as_str(),
            Some("cand-one"),
            "each slug got its own confirmation of the same candidate"
        );
    }
    assert_eq!(
        records[0]["friendly_name"].as_str(),
        Some("First Game"),
        "the record's friendly name is the friendly name of the first row sharing the \
         slug, in (slug, install_path) order"
    );
}

/// proves: design 025's Test strategy #7 (Chosen design: "written to a temp name
/// and renamed"; "no `.tmp` file remains") — after a confirmed save the store
/// directory holds exactly `matches.json`: the temp name is renamed away, never
/// left behind. (The exclusive open of the temp name is an in-module pin of
/// `apps/discovery-cli/src/matches.rs`, Rollout 2.)
#[test]
fn the_save_leaves_no_tmp_file_behind() {
    let root = one_game("store-tmp");
    let data = TempDir::new("store-tmp-data");
    let fake = fake_catalog();

    let run = run_table(&root, &env(data.path(), data.path()), &fake.url(), "1\n");
    assert!(run.success(), "stderr: {}", run.stderr);

    let files = data.files_below("bindforge");
    let names: Vec<String> = files
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    assert_eq!(
        names,
        vec!["matches.json".to_string()],
        "the save leaves no `matches.json.tmp` behind"
    );
}

/// proves: design 025's Test strategy #7 ("re-saving the same store is
/// byte-identical") — the same set of confirmed records lands as identical bytes
/// however the confirmations are ordered across runs: sorted, canonical output (and
/// Test strategy #12's "rewrites nothing" half — a run with zero new confirmations
/// leaves the bytes untouched, pinned above in the valid-store test).
#[test]
fn the_same_record_set_saves_as_identical_bytes_whatever_the_order() {
    // Path A: confirm 5001 first, then 5002.
    let root_a = SteamRoot::new("store-stable-a");
    root_a.acf("5001", Some("First Game"), "First Game");
    root_a.acf("5002", Some("Second Game"), "Second Game");
    let data_a = TempDir::new("store-stable-a-data");
    let fake_a = fake_catalog();
    let run_a1 = run_table(
        &root_a,
        &env(data_a.path(), data_a.path()),
        &fake_a.url(),
        "1\nn\n",
    );
    assert!(run_a1.success(), "stderr A1: {}", run_a1.stderr);
    let run_a2 = run_table(
        &root_a,
        &env(data_a.path(), data_a.path()),
        &fake_a.url(),
        "1\n",
    );
    assert!(run_a2.success(), "stderr A2: {}", run_a2.stderr);

    // Path B: confirm 5002 first (its own root), then 5001.
    let root_b1 = SteamRoot::new("store-stable-b1");
    root_b1.acf("5002", Some("Second Game"), "Second Game");
    let data_b = TempDir::new("store-stable-b-data");
    let fake_b = fake_catalog();
    let run_b1 = run_table(
        &root_b1,
        &env(data_b.path(), data_b.path()),
        &fake_b.url(),
        "1\n",
    );
    assert!(run_b1.success(), "stderr B1: {}", run_b1.stderr);
    let root_b2 = SteamRoot::new("store-stable-b2");
    root_b2.acf("5001", Some("First Game"), "First Game");
    root_b2.acf("5002", Some("Second Game"), "Second Game");
    let run_b2 = run_table(
        &root_b2,
        &env(data_b.path(), data_b.path()),
        &fake_b.url(),
        "1\n",
    );
    assert!(run_b2.success(), "stderr B2: {}", run_b2.stderr);

    let bytes_a = std::fs::read(store_path(data_a.path())).expect("store A");
    let bytes_b = std::fs::read(store_path(data_b.path())).expect("store B");
    assert_eq!(
        bytes_a, bytes_b,
        "one record set, two confirmation orders: the stored bytes are byte-identical \
         (sorted by `slug`, canonical)"
    );
}

/// proves: design 025's Test strategy #7, unix half (Chosen design: "the `bindforge/`
/// directory and both files are created owner-only — 0700/0600 where unix modes
/// apply — so the confirmed inventory is never a `umask` accident").
#[cfg(unix)]
#[test]
fn the_store_directory_and_file_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;

    let root = one_game("store-modes");
    let data = TempDir::new("store-modes-data");
    let fake = fake_catalog();

    let run = run_table(&root, &env(data.path(), data.path()), &fake.url(), "1\n");
    assert!(run.success(), "stderr: {}", run.stderr);

    let dir_mode = std::fs::metadata(data.join("bindforge"))
        .expect("the store directory")
        .permissions()
        .mode();
    assert_eq!(
        dir_mode & 0o777,
        0o700,
        "the `bindforge/` directory is created owner-only"
    );
    let file_mode = std::fs::metadata(store_path(data.path()))
        .expect("the store file")
        .permissions()
        .mode();
    assert_eq!(
        file_mode & 0o777,
        0o600,
        "matches.json is created owner-only"
    );
}
