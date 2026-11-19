//! Where the two caching documents actually live: `<data>/bindforge/registry/`
//! `{cache-v1.json, catalog-v1.json}`, with `<data>` resolved by the copied
//! `026-C7`/A6 rule — absolute `$XDG_DATA_HOME`, else absolute
//! `$HOME/.local/share`, absolute `%APPDATA%` on Windows, and a *relative* value
//! treated as unset — plus the owner-only `0700`/`0600` modes A7/A8 give the
//! directory and its documents on unix.
//!
//! proves: design 026's Test strategy #13 — "Data-dir / cache path (`026-C7`, A6):
//! both Unix and Windows branches through the `StoreLayout` env-closure seam
//! (XDG/HOME vs `%APPDATA%`), relative = unset; this seam is how caching tests
//! relocate the local-entries document — the subprocess cases (#17) set
//! `XDG_DATA_HOME`/`APPDATA` in the child environment, since there is no path knob."
//!
//! The suite is the subprocess face of that item: `apps/registry` is a bin-only
//! crate (`026-C1`), so the seam is exercised the way the design says a subprocess
//! reaches it — the environment of the spawned `registry`, which the harness scrubs
//! of `XDG_DATA_HOME`/`HOME`/`APPDATA` before any test sets them. No path knob is
//! invented, and no test dials anything: the only upstream is a loopback fake.

mod common;

#[cfg(unix)]
use common::mode_of;
use common::{
    FakeUpstream, Registry, TempDir, candidates_body, game_ids, ok, record_body, registry_document,
    respond_to, slug_query, status_code, status_line,
};
use std::path::{Path, PathBuf};

/// The one directory `026-C7` fixes inside `<data>`, created by the instance itself.
fn registry_dir(data: &Path) -> PathBuf {
    data.join("bindforge").join("registry")
}

fn documents(data: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    (
        registry_document(data, "cache-v1.json"),
        registry_document(data, "catalog-v1.json"),
    )
}

fn document_at(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{} is JSON ({err})", path.display()))
}

/// A fake upstream that answers every candidate request with the same one-game
/// answer, so a cold lookup is guaranteed to produce a cache entry.
fn one_answer_upstream(slug: &str, game_id: &str) -> FakeUpstream {
    let needle = format!("slug={}", common::percent_encode(slug));
    let body = candidates_body(&[(game_id, game_id)]);
    FakeUpstream::start(respond_to(
        vec![(needle, ok(&body))],
        ok(&candidates_body(&[])),
    ))
}

/// proves: Test strategy #13's Unix/`XDG` branch (`026-C7`) — a caching instance
/// whose `XDG_DATA_HOME` is set writes both documents under
/// `<data>/bindforge/registry/`, at exactly those two paths, and creates nothing
/// else in the data directory.
#[cfg(unix)]
#[test]
fn the_documents_land_at_the_xdg_data_home_spelling() {
    let data = TempDir::new("layout-xdg");
    let upstream = one_answer_upstream("steam/900001", "fetched-game");
    let server = Registry::caching("layout-xdg", &upstream.url(), &data, &[]);

    let answer = server.get(&slug_query("steam/900001"));
    assert_eq!(status_code(&answer), Some(200), "the cold fetch answers");
    assert_eq!(game_ids(&answer), vec!["fetched-game".to_string()]);

    let upsert = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("mine", "Mine", &["steam/900002"]),
    );
    assert!(
        matches!(status_code(&upsert), Some(200..=299)),
        "the local-entries document takes an upsert: {}",
        status_line(&upsert)
    );
    drop(server);

    let (cache, local_entries) = documents(data.path());
    assert!(
        cache.is_file(),
        "`026-C7` puts the cache at `<data>/bindforge/registry/cache-v1.json`, not {:?}",
        data.files_below(".")
    );
    assert!(
        local_entries.is_file(),
        "the local-entries document sits beside it at `catalog-v1.json`"
    );
    assert_eq!(
        document_at(&cache)["entries"][0]["slug"].as_str(),
        Some("steam/900001"),
        "the cached slug is the looked-up key"
    );
    assert_eq!(
        document_at(&local_entries)["version"].as_i64(),
        Some(1),
        "the local-entries document is a `026-C3` instance"
    );
    let mut written = data.files_below(".");
    written.sort();
    let mut wanted = vec![cache, local_entries];
    wanted.sort();
    assert_eq!(
        written, wanted,
        "the data directory holds the two documents and nothing else"
    );
}

/// proves: Test strategy #13's `HOME` fallback (`026-C7`) — with `XDG_DATA_HOME`
/// unset, `<data>` is absolute `$HOME/.local/share`, so both documents land under
/// that spelling rather than anywhere near the process's working directory.
#[cfg(unix)]
#[test]
fn an_unset_xdg_data_home_falls_back_to_home_local_share() {
    let home = TempDir::new("layout-home");
    let upstream = one_answer_upstream("steam/900003", "fetched-home");
    let spell = home.path().to_str().expect("a utf-8 temp home").to_string();
    let url = upstream.url();
    let server = Registry::start(
        "layout-home",
        &["serve", "--upstream", url.as_str()],
        &[("HOME", spell.as_str())],
    );

    let answer = server.get(&slug_query("steam/900003"));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "the caching instance answers from the fallback data directory"
    );
    assert_eq!(game_ids(&answer), vec!["fetched-home".to_string()]);
    drop(server);

    let data = home.join(".local/share");
    assert!(
        registry_document(&data, "cache-v1.json").is_file(),
        "with no `XDG_DATA_HOME`, `<data>` is `$HOME/.local/share`: the cache document is \
         not written there"
    );
    let listed: Vec<std::ffi::OsString> = std::fs::read_dir(registry_dir(&data))
        .expect("the instance created `<data>/bindforge/registry` for itself")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name())
        .collect();
    assert!(
        listed.iter().any(|name| name == "cache-v1.json"),
        "the directory it created holds the pinned file name, found {listed:?}"
    );
}

/// proves: Test strategy #13's "relative = unset" clause (A6) — a *relative*
/// `XDG_DATA_HOME` is not a data directory: it is skipped exactly as an unset one
/// would be, the fallback takes over, and no directory is created for the relative
/// spelling.
#[cfg(unix)]
#[test]
fn a_relative_xdg_data_home_is_treated_as_unset() {
    let home = TempDir::new("layout-relative");
    let upstream = one_answer_upstream("steam/900004", "fetched-relative");
    let home_spelling = home.path().to_str().expect("a utf-8 temp home").to_string();

    let url = upstream.url();
    let server = Registry::start(
        "layout-relative",
        &["serve", "--upstream", url.as_str()],
        &[
            ("XDG_DATA_HOME", "relative-registry-data"),
            ("HOME", home_spelling.as_str()),
        ],
    );
    let answer = server.get(&slug_query("steam/900004"));
    assert_eq!(status_code(&answer), Some(200));
    drop(server);

    assert!(
        !Path::new("relative-registry-data").exists(),
        "a relative data directory is never created, not even beside the binary"
    );
    assert!(
        home.join(".local/share/bindforge/registry/cache-v1.json")
            .is_file(),
        "a relative `XDG_DATA_HOME` is unset, so the absolute `$HOME/.local/share` \
         fallback owns the documents"
    );
}

/// proves: Test strategy #13's Windows branch (A6, §Cross-platform notes) — on
/// Windows `<data>` is absolute `%APPDATA%` and the same two documents land under
/// it. Not run in CI, which is why the design names the env-closure seam for this
/// branch; it is kept here so the branch is pinned wherever it can run.
#[cfg(windows)]
#[test]
fn the_documents_land_at_the_appdata_spelling_on_windows() {
    let data = TempDir::new("layout-appdata");
    let upstream = one_answer_upstream("steam/900001", "fetched-game");
    let spell = data
        .path()
        .to_str()
        .expect("a utf-8 temp appdata")
        .to_string();
    let server = Registry::start(
        "layout-appdata",
        &["serve", "--upstream", &upstream.url()],
        &[("APPDATA", spell.as_str())],
    );

    let answer = server.get(&slug_query("steam/900001"));
    assert_eq!(status_code(&answer), Some(200));
    drop(server);

    assert!(
        registry_document(data.path(), "cache-v1.json").is_file(),
        "`<data>` = `%APPDATA%`, so the cache lands at `<appdata>\\bindforge\\registry\\\
         cache-v1.json`"
    );
}

/// proves: Test strategy #13's owner-only consequence on unix (A7, A8, and the
/// §Cross-platform row "owner-only `0700`/`0600` modes apply") — the registry
/// directory the instance creates for itself is `0700` and each document it renames
/// into place is `0600`, so another account on the machine cannot read a curated
/// corpus or a cache.
#[cfg(unix)]
#[test]
fn the_registry_directory_and_its_documents_are_owner_only() {
    let data = TempDir::new("layout-modes");
    let upstream = one_answer_upstream("steam/900006", "fetched-modes");
    let server = Registry::caching("layout-modes", &upstream.url(), &data, &[]);

    let answer = server.get(&slug_query("steam/900006"));
    assert_eq!(status_code(&answer), Some(200));
    let upsert = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("mine", "Mine", &["steam/900007"]),
    );
    assert!(matches!(status_code(&upsert), Some(200..=299)));
    drop(server);

    assert_eq!(
        mode_of(&registry_dir(data.path())),
        0o700,
        "the directory A7's save creates is owner-only"
    );
    for document in [
        registry_document(data.path(), "cache-v1.json"),
        registry_document(data.path(), "catalog-v1.json"),
    ] {
        assert!(document.is_file(), "{} is written", document.display());
        assert_eq!(
            mode_of(&document) & 0o777,
            0o600,
            "{} is `0600`: the tmp's mode is re-applied after the rename (A7)",
            document.display()
        );
    }
}

/// proves: Test strategy #13 read together with `026-C7` — the pinned path *is* the
/// persistence contract: a second caching instance pointed at the same data
/// directory finds the cache the first one wrote, without asking the upstream
/// anything.
#[test]
fn a_document_written_at_the_pinned_path_is_the_next_instances_cache() {
    let data = TempDir::new("layout-restart");
    let upstream = one_answer_upstream("steam/900008", "fetched-once");

    {
        let first = Registry::caching("layout-restart-a", &upstream.url(), &data, &[]);
        let answer = first.get(&slug_query("steam/900008"));
        assert_eq!(status_code(&answer), Some(200));
        assert_eq!(game_ids(&answer), vec!["fetched-once".to_string()]);
    }
    let after_first = upstream.lines_for("GET /v1/candidates").len();
    assert_eq!(after_first, 1, "the first instance fetched it once");

    let second = Registry::caching("layout-restart-b", &upstream.url(), &data, &[]);
    let answer = second.get(&slug_query("steam/900008"));
    assert_eq!(status_code(&answer), Some(200));
    assert_eq!(
        game_ids(&answer),
        vec!["fetched-once".to_string()],
        "the persisted document at the pinned path answers for the new instance"
    );
    assert_eq!(
        upstream.lines_for("GET /v1/candidates").len(),
        after_first,
        "and it cost no upstream lookup: the fake saw {:?}",
        upstream.lines_for("GET /v1/candidates")
    );
    assert_eq!(
        common::json_body(&second.health())["cached_slugs"].as_u64(),
        Some(1),
        "`026-C2` counts what came off the disk"
    );
}
