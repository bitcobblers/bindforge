//! The `--json` contract of design 025 / ADR-025 (`025-C6`, `025-L2`): one
//! parseable JSON document with the deliberately re-pinned row key set —
//! `slug`, `friendly_name`, `display_name`, `install_path`, `platform`,
//! `codec_id`, `game_id`, `candidates` — and none of the interactive flow's
//! store consequences.
//!
//! proves: design 025's Test strategy #13 (Chosen design: "JSON mode: no prompt,
//! no prompt text, no store writes; candidates are emitted instead, per row", and
//! the re-pinned key sample with `game_id: null` and a nested `candidates` list;
//! "`candidates` is `[]` for a slug the store resolved") — the strict parse of the
//! whole stdout doubles as the no-prompt-text proof, the retired row key is
//! asserted absent by name, and the store stays untouched in both directions: no
//! file is created by a JSON run, and a pre-existing store is neither consulted
//! for writes nor rewritten while resolving its slugs without queries.
//!
//! Integration (CLI, subprocess): loopback fake, temp `XDG_DATA_HOME`/`HOME`
//! sandbox, piped stdin closed immediately (a JSON run that prompted would be a
//! design break observed here as a missing candidate passthrough).

mod common;

use common::{FakeCatalog, SteamRoot, TempDir, candidates_body, ok, run_cli, store_path};

fn fake_two_candidates() -> FakeCatalog {
    let body = candidates_body(&[("cand-one", "First Match"), ("cand-two", "Second Match")]);
    FakeCatalog::start("json-fake", move |_request| ok(&body))
}

fn fake_no_candidates() -> FakeCatalog {
    FakeCatalog::start("json-fake-empty", |_| ok("[]"))
}

fn env<'a>(data: &'a TempDir, home: &'a TempDir) -> [(&'a str, &'a str); 2] {
    [
        ("XDG_DATA_HOME", data.path().to_str().expect("utf-8")),
        ("HOME", home.path().to_str().expect("utf-8")),
    ]
}

/// A sandbox pair under one label.
fn sandbox(label: &str) -> (TempDir, TempDir) {
    (
        TempDir::new(&format!("json-{label}-data")),
        TempDir::new(&format!("json-{label}-home")),
    )
}

/// proves: design 025's Test strategy #13 (Chosen design: "The JSON row is one
/// document per run … with the key set re-pinned deliberately") — a JSON run over
/// one unresolved game prints exactly one JSON document whose single row carries
/// exactly the eight pinned keys (the retired namespace key absent by name), a
/// `null` `game_id` as the unmatched state, the plugin's `friendly_name`, and the
/// catalog's candidates as the nested `candidates` list in response order; and no
/// store file or directory is written (`025-L2`: JSON mode never confirms).
#[test]
fn a_json_run_prints_the_pinned_key_set_with_null_id_and_nested_candidates() {
    let root = SteamRoot::new("json-keys");
    root.acf("5001", Some("Example Game"), "Example Game");
    let (data, home) = sandbox("keys");
    let fake = fake_two_candidates();

    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
            "--json",
        ],
        &env(&data, &home),
        "",
    );
    assert!(run.success(), "stderr: {}", run.stderr);

    // A strict parse of the whole stdout: any prompt text interleaved on stdout
    // breaks this line, which is the strategy's "no prompt text on stdout" half.
    let rows: Vec<serde_json::Value> = serde_json::from_str(run.stdout.trim())
        .expect("`list --json` must print one JSON document and nothing else");
    assert_eq!(rows.len(), 1, "exactly the one fixture row");
    let row = rows[0].as_object().expect("the row is a JSON object");

    let mut keys: Vec<&str> = row.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "candidates",
            "codec_id",
            "display_name",
            "friendly_name",
            "game_id",
            "install_path",
            "platform",
            "slug",
        ],
        "the JSON row carries exactly the eight keys the design re-pinned"
    );
    assert!(
        row.get("namespace").is_none(),
        "the retired `namespace` key must be absent, got: {}",
        run.stdout
    );

    assert_eq!(row["slug"], "steam/5001");
    assert_eq!(row["friendly_name"], "Example Game");
    assert_eq!(row["display_name"], "Example Game");
    assert_eq!(row["platform"], "SteamLinux");
    assert!(
        row["codec_id"].is_null(),
        "codec_id stays the nullable field it was"
    );
    assert!(
        row["game_id"].is_null(),
        "`null` game_id is the unmatched state (Chosen design)"
    );

    let candidates = row["candidates"].as_array().expect("a candidates array");
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| (
                candidate["game_id"].as_str().unwrap_or_default(),
                candidate["name"].as_str().unwrap_or_default(),
            ))
            .collect::<Vec<(&str, &str)>>(),
        vec![("cand-one", "First Match"), ("cand-two", "Second Match")],
        "the candidates are emitted per row, in the response's order"
    );
    for candidate in candidates {
        assert_eq!(
            candidate.as_object().expect("a candidate object").len(),
            2,
            "a candidate is `{{game_id, name}}` — the design's `Candidate`"
        );
    }

    assert_eq!(
        fake.request_count(),
        1,
        "one query for the one unresolved slug: {:?}",
        fake.requests()
    );
    assert!(
        !store_path(data.path()).exists() && !data.join("bindforge").exists(),
        "a JSON run writes no store: no file, not even a directory"
    );
}

/// proves: design 025's Test strategy #13 with `025-L4` (Chosen design:
/// "`candidates` is `[]` for a slug the store resolved") — a slug the store
/// resolves renders its confirmed `game_id` and an empty `candidates` list, makes
/// zero catalog queries, and leaves the store bytes untouched even in JSON mode.
#[test]
fn a_store_resolved_slug_prints_its_id_and_empty_candidates() {
    let root = SteamRoot::new("json-resolved");
    root.acf("5001", Some("Example Game"), "Example Game");
    let (data, home) = sandbox("resolved");
    let stored = b"{\"version\":1,\"matches\":[{\"slug\":\"steam/5001\",\"\
                   game_id\":\"example-game\",\"friendly_name\":\"Example Game\"}]}";
    data.write("bindforge/matches.json", stored);
    let fake = fake_two_candidates();

    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
            "--json",
        ],
        &env(&data, &home),
        "",
    );
    assert!(run.success(), "stderr: {}", run.stderr);

    let rows: Vec<serde_json::Value> =
        serde_json::from_str(run.stdout.trim()).expect("one JSON document");
    let row = rows[0].as_object().expect("the row is a JSON object");
    assert_eq!(
        row["game_id"], "example-game",
        "the store's confirmed id prints instead of `null`"
    );
    assert_eq!(
        row["candidates"],
        serde_json::json!([]),
        "a store-resolved slug carries no candidates"
    );
    assert_eq!(
        fake.request_count(),
        0,
        "a slug the store resolves is never queried, JSON mode included: {:?}",
        fake.requests()
    );
    assert_eq!(
        std::fs::read(store_path(data.path())).expect("the store survives"),
        stored,
        "a JSON run rewrites nothing"
    );
}

/// proves: design 025's Test strategy #13, "no prompt … no store writes" under
/// pressure — a JSON run whose slugs have zero candidates is plainly clean: exit
/// 0, one JSON document whose row still carries the pinned key set with `null`
/// `game_id` and `[]` candidates, and no store directory. (Together with the
/// candidates-present test above, this pins that JSON mode has no interactive
/// step at all — `025-L2`.)
#[test]
fn a_json_run_with_no_candidates_still_prints_the_pinned_row() {
    let root = SteamRoot::new("json-empty-candidates");
    root.acf("5001", Some("Example Game"), "Example Game");
    let (data, home) = sandbox("empty-cands");
    let fake = fake_no_candidates();

    let run = run_cli(
        &[
            "list",
            "--steam-root",
            root.path().to_str().expect("utf-8 path"),
            "--catalog-url",
            &fake.url(),
            "--json",
        ],
        &env(&data, &home),
        "",
    );
    assert!(run.success(), "stderr: {}", run.stderr);

    let rows: Vec<serde_json::Value> =
        serde_json::from_str(run.stdout.trim()).expect("one JSON document");
    let row = rows[0].as_object().expect("the row is a JSON object");
    assert_eq!(row.keys().count(), 8);
    assert!(row["game_id"].is_null());
    assert_eq!(row["candidates"], serde_json::json!([]));
    assert!(
        !data.join("bindforge").exists(),
        "a zero-candidate JSON run writes no store"
    );
}
