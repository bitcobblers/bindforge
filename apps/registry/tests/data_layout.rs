//! Where a hosted instance's documents actually live: `<data>/bindforge/registry/`
//! `{steam-snapshot.json, issued-v1.json, retired-ids-v1.json}` (§Hosted mode's
//! owner paths), with `<data>` resolved by the `027`-carried A2 rule — absolute
//! `$XDG_DATA_HOME`, else absolute `$HOME/.local/share`, absolute `%APPDATA%` on
//! Windows, and a *relative* value treated as unset — plus the owner-only
//! `0700`/`0600` modes A7/A8 give the directory and its documents on unix
//! (§Compatibility matrix's Linux row: "owner-only `0700`/`0600` apply").
//!
//! proves: design 027's Test strategy #16 — "**Windows** is covered by the shipped
//! `cargo check --target x86_64-pc-windows-msvc` leg … plus the **`DataLayout`/env-
//! injection seams** for the Windows `%APPDATA%` branch" — read with #12's hosted
//! end-to-end, whose mint is what puts a document on disk at all in the one mode
//! that has one.
//!
//! # What ADR-027 retired out of this file
//!
//! Every leg here used to run a caching instance and count `cache-v1.json` beside
//! `catalog-v1.json`. ADR-027 retires the caching mode and both of those names
//! (A15 lists this file among the suites that re-pin; §Hosted mode makes a leftover
//! copy of either document in the registry directory a *startup refusal* rather than
//! state, which `hosted_startup.rs` pins). The resolution rule and the mode
//! discipline never belonged to caching in the first place — A2 and A7/A8 are
//! `<data>` contracts — so they re-pin here against hosted mode's own documents.
//! The successor of "the document at the pinned path is the next instance's cache"
//! is #4's persist-before-serve and #7's document contract, pinned in
//! `issued_entries.rs`, and it is not restated here.
//!
//! The suite is the subprocess face of the seam: `apps/registry` is a bin-only
//! crate (`026-C1`), so the only relocation knob a subprocess has is its own
//! environment, which the harness scrubs of `XDG_DATA_HOME`/`HOME`/`APPDATA` before
//! any test sets them. No path knob is invented. Every run gets a planted snapshot —
//! planted under the `<data>` its own branch resolves to, which is what `data_of` is
//! for — so no run has anything to pull and no test dials or resolves a non-loopback
//! host (Test strategy #17); the key it is given is a throwaway that stays unused. A
//! leg that planted anywhere else would not be measuring a resolution rule at all: the
//! instance would find nothing to load, pull, and the harness would report a stall.

mod common;

#[cfg(unix)]
use common::mode_of;
use common::{
    Registry, TempDir, corpus_doc, game_ids, now_unix, registry_document, status_code, status_line,
};
use std::path::{Path, PathBuf};

const KEY: &str = "bindforge-test-steam-key-4d5e6f70";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";

/// The store snapshot's pinned path inside `<data>` (§Hosted mode; the name the
/// suites plant and the loader reads).
const SNAPSHOT: &str = "steam-snapshot.json";

/// The one document a lookup-driven mint writes: `027-C4`'s issued entries.
const ISSUED: &str = "issued-v1.json";

/// The one directory the contracts fix inside `<data>`, created by the instance.
fn registry_dir(data: &Path) -> PathBuf {
    data.join("bindforge").join("registry")
}

fn document_at(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("{} is JSON ({err})", path.display()))
}

fn corpus_in(data: &TempDir, games: &[(&str, &str, &[&str])]) -> std::path::PathBuf {
    data.write("corpus.json", corpus_doc(3, games).as_bytes())
}

/// The `<data>` the A2 rule resolves for the branch `envs` spells: absolute
/// `$XDG_DATA_HOME`, else absolute `%APPDATA%` (the Windows branch), else absolute
/// `$HOME/.local/share`, and a *relative* spelling of either skipped exactly as an
/// unset one. The suite has to know which one a leg is on, because a snapshot planted
/// anywhere else is a snapshot the instance never finds — and an instance with nothing
/// to load pulls the store at startup, which is the 20 s stall the harness reports and
/// the non-loopback dial Test strategy #17 rules out.
fn data_of(envs: &[(&str, &str)]) -> PathBuf {
    let absolute = |name: &str| {
        envs.iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| PathBuf::from(value))
            .filter(|path| path.is_absolute())
    };
    if let Some(xdg) = absolute("XDG_DATA_HOME") {
        return xdg;
    }
    if let Some(appdata) = absolute("APPDATA") {
        return appdata;
    }
    absolute("HOME")
        .expect("a data-layout leg spells the branch it is pinning")
        .join(".local")
        .join("share")
}

/// A fresh, plausible snapshot that already knows `appid`, so the lookup below is a
/// mint rather than a `200 []` — and, because the file is there to be *loaded*, so
/// the instance has no reason to reach the store at all (§The steam snapshot
/// resolver's startup order). It is planted under the `<data>` the leg's own branch
/// resolves to, which is the whole point of a leg about where documents live.
fn plant_fresh_snapshot(data: &Path, appid: u64, name: &str) {
    let document = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "fetched_at": now_unix(),
        "apps": [{ "appid": appid, "name": name }],
    }))
    .expect("a snapshot document serialises");
    let path = registry_document(data, SNAPSHOT);
    std::fs::create_dir_all(path.parent().expect("the registry directory"))
        .unwrap_or_else(|err| panic!("creating {}: {err}", data.display()));
    std::fs::write(&path, document.as_bytes())
        .unwrap_or_else(|err| panic!("writing {}: {err}", path.display()));
}

/// A hosted instance whose `<data>` is whichever branch `envs` spells — the
/// relocation seam this whole suite exists to exercise, spelled out per leg rather
/// than through the harness's default data home.
fn serve(envs: &[(&str, &str)], corpus: &Path) -> Registry {
    let xdg_present = envs.iter().any(|(name, _)| *name == "XDG_DATA_HOME");
    let home_present = envs.iter().any(|(name, _)| *name == "HOME");
    let appdata_present = envs.iter().any(|(name, _)| *name == "APPDATA");
    assert!(
        xdg_present || home_present || appdata_present,
        "a data-layout leg has to set the branch it is pinning"
    );
    let corpus_path = corpus.display().to_string();
    let args = vec!["serve", "--root", "--corpus", corpus_path.as_str()];
    let mut all: Vec<(&str, &str)> = envs.to_vec();
    all.push((KEY_VAR, KEY));
    Registry::start("layout", &args, &all)
}

/// The mint this suite uses as its document-writing event: a snapshot-known
/// `steam/<appid>` key, answered once, and the minted identity it printed — plus the
/// corpus path, for the legs that need a second run over the same two documents. The
/// identity is *read* rather than hard-coded so this suite stays about paths and
/// modes, not about the fold table `mint_rule.rs` owns.
fn mint_one(
    data: &TempDir,
    envs: &[(&str, &str)],
    appid: u64,
) -> (Registry, String, std::path::PathBuf) {
    let corpus = corpus_in(
        data,
        &[("curated-fixture", "Curated Fixture", &["steam/900001"][..])],
    );
    plant_fresh_snapshot(&data_of(envs), appid, "Layout Probe");
    let server = serve(envs, &corpus);
    let target = format!("/v1/candidates?slug=steam%2F{appid}");
    let answer = server.get(&target);
    assert_eq!(
        status_code(&answer),
        Some(200),
        "a snapshot-known key mints, so the document this suite pins has a writer: {}",
        status_line(&answer)
    );
    let minted = game_ids(&answer)
        .first()
        .cloned()
        .expect("the mint path answers exactly one candidate");
    (server, minted, corpus)
}

fn issued_at(data: &Path) -> PathBuf {
    registry_document(data, ISSUED)
}

/// proves: Test strategy #16's `DataLayout` seam on the branch Linux and CI
/// actually run (`027`'s A2: `<data>` is absolute `$XDG_DATA_HOME`) — a hosted
/// instance that mints puts `issued-v1.json` at
/// `<data>/bindforge/registry/issued-v1.json`, reads the snapshot from that same
/// directory, and writes nothing else there: no cache document, no catalogue copy.
/// The listing is taken after a `registry query` one-shot has run over the same
/// `<data>` and exited of its own accord, because that is the only exit a test can
/// arrange in which §Concurrency's `write.lock` is removed — `common::Registry::drop`
/// is `child.kill()`, and a killed run is exactly the crashed run §Concurrency says
/// leaves the file behind for the runbook to delete (`hosted_startup.rs` pins both
/// halves of that, and the deletion between the two runs below is Rollout 6's act, not
/// one a run performs). The one-shot asks the key the serve run already minted, so it
/// answers from the issued document with nothing new to write (#12).
#[cfg(unix)]
#[test]
fn a_mint_lands_under_the_xdg_data_home_spelling_and_nothing_else_lands_beside_it() {
    let data = TempDir::new("layout-xdg");
    let xdg = data
        .path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string();
    let appid = 9_300_001;
    let (server, minted, corpus) = mint_one(&data, &[("XDG_DATA_HOME", xdg.as_str())], appid);
    drop(server);

    // The crashed run's lock is the runbook's to delete; until it is, §Concurrency
    // refuses the next instance over this `<data>`.
    common::clear_crashed_run_write_lock(data.path());

    let issued = issued_at(data.path());
    assert!(
        issued.is_file(),
        "`027-C4` puts the issued document at `<data>/bindforge/registry/{ISSUED}`, not \
         {:?}",
        data.files_below("bindforge")
    );
    let ids: Vec<String> = document_at(&issued)["games"]
        .as_array()
        .expect("a games list")
        .iter()
        .map(|game| game["game_id"].as_str().expect("a game_id").to_string())
        .collect();
    assert_eq!(
        ids,
        vec![minted],
        "the mint is in the document at the pinned path"
    );

    let one_shot = common::run_registry(
        &[
            "query",
            &format!("steam/{appid}"),
            "--root",
            "--corpus",
            corpus.to_str().expect("a utf-8 temp corpus path"),
        ],
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
    );
    assert!(
        one_shot.success(),
        "the one-shot that closes the run answers the key the serve run minted: {}",
        one_shot.stderr
    );

    let mut written = data.files_below("bindforge");
    written.sort();
    let mut wanted = vec![registry_document(data.path(), SNAPSHOT), issued.clone()];
    wanted.sort();
    assert_eq!(
        written,
        wanted,
        "the instance owns exactly the snapshot it read and the document it wrote: the \
         caching mode's `cache-v1.json`/`catalog-v1.json` are retired names that must not \
         reappear, and a run that exits by itself leaves no `write.lock` behind \
         (§Concurrency, Test strategy #8) — found {:?} beside {:?}",
        data.files_below("bindforge"),
        issued
    );
}

/// proves: Test strategy #16's seam on the `HOME` fallback branch (A2, unchanged by
/// ADR-027) — with `XDG_DATA_HOME` unset, `<data>` is absolute
/// `$HOME/.local/share`, and the hosted instance creates that directory for itself
/// and puts its issued document in it rather than anywhere near the process's
/// working directory.
#[cfg(unix)]
#[test]
fn an_unset_xdg_data_home_falls_back_to_home_local_share() {
    let home = TempDir::new("layout-home");
    let spell = home.path().to_str().expect("a utf-8 temp home").to_string();
    let (server, _, _) = mint_one(&home, &[("HOME", spell.as_str())], 9_300_002);
    drop(server);

    let data = home.join(".local/share");
    assert!(
        issued_at(&data).is_file(),
        "with no `XDG_DATA_HOME`, `<data>` is `$HOME/.local/share`: the issued document is \
         not written there ({:?})",
        home.files_below(".")
    );
    let listed: Vec<std::ffi::OsString> = std::fs::read_dir(registry_dir(&data))
        .expect("the instance created `<data>/bindforge/registry` for itself")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name())
        .collect();
    assert!(
        listed.iter().any(|name| name == ISSUED),
        "the directory it created holds the pinned file name, found {listed:?}"
    );
}

/// proves: Test strategy #16's seam on the "relative = unset" clause (A2/A6, which
/// §Compatibility matrix keeps for every platform) — a *relative* `XDG_DATA_HOME` is
/// not a data directory: it is skipped exactly as an unset one would be, the
/// absolute fallback owns the documents, and no directory is created for the
/// relative spelling. A hosted instance writes real user data, so this branch is the
/// difference between a misconfigured service and a service that litters.
#[cfg(unix)]
#[test]
fn a_relative_xdg_data_home_is_treated_as_unset() {
    let home = TempDir::new("layout-relative");
    let home_spelling = home.path().to_str().expect("a utf-8 temp home").to_string();
    let (server, _, _) = mint_one(
        &home,
        &[
            ("XDG_DATA_HOME", "relative-registry-data"),
            ("HOME", home_spelling.as_str()),
        ],
        9_300_003,
    );
    drop(server);

    assert!(
        !Path::new("relative-registry-data").exists(),
        "a relative data directory is never created, not even beside the binary"
    );
    assert!(
        issued_at(&home.join(".local/share")).is_file(),
        "a relative `XDG_DATA_HOME` is unset, so the absolute `$HOME/.local/share` \
         fallback owns the documents"
    );
}

/// proves: Test strategy #16's `DataLayout`/env-injection leg — on Windows `<data>`
/// is absolute `%APPDATA%` (§Compatibility matrix's Windows row spells the owner
/// paths as `%APPDATA%\bindforge\registry\{steam-snapshot,issued-v1,
/// retired-ids-v1}.json`), so a mint lands there. Not run in CI, which is exactly
/// why the design names this seam for the branch instead of a Windows test job; it
/// is kept here so the branch is pinned wherever it can run.
#[cfg(windows)]
#[test]
fn a_mint_lands_under_the_appdata_spelling_on_windows() {
    let data = TempDir::new("layout-appdata");
    let spell = data
        .path()
        .to_str()
        .expect("a utf-8 temp appdata")
        .to_string();
    let (server, _, _) = mint_one(&data, &[("APPDATA", spell.as_str())], 9_300_004);
    drop(server);

    assert!(
        registry_dir(data.path()).is_dir(),
        "`<data>` is `%APPDATA%`, so the instance creates the `bindforge/registry` \
         directory under it"
    );
    assert!(
        issued_at(data.path()).is_file(),
        "and `{ISSUED}` lands in that directory"
    );
}

/// proves: the owner-only consequence the design keeps unchanged on unix (A7/A8,
/// §Compatibility matrix's Linux row: "owner-only `0700`/`0600` apply") — the
/// registry directory a hosted instance creates for itself is `0700` and each
/// document it renames into place is `0600`, so another account on the hosting
/// machine cannot read a curated corpus's derived records. The issued document is
/// the one this mode's own writes produce, and it is the one whose contents are
/// remote-influenced (a minted name comes from the store snapshot).
#[cfg(unix)]
#[test]
fn the_registry_directory_and_the_issued_document_are_owner_only() {
    let data = TempDir::new("layout-modes");
    let xdg = data
        .path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string();
    let (server, _, _) = mint_one(&data, &[("XDG_DATA_HOME", xdg.as_str())], 9_300_005);

    // The rename is already in place by the time the mint's answer was served
    // (§Concurrency's persist-before-serve); the run is closed here so the modes are
    // read the way the next instance against the same `<data>` will find them.
    drop(server);

    assert_eq!(
        mode_of(&registry_dir(data.path())),
        0o700,
        "the directory A7's save creates is owner-only"
    );
    let issued = issued_at(data.path());
    assert!(issued.is_file(), "{} is written", issued.display());
    assert_eq!(
        mode_of(&issued) & 0o777,
        0o600,
        "{} is `0600`: the tmp's mode is re-applied after the rename (A7)",
        issued.display()
    );
}
