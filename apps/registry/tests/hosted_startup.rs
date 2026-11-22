//! Hosted startup: the one mode `registry serve` admits, the retired spellings it
//! refuses by name, and the `<data>` write lock that keeps a second writer out.
//!
//! proves: design 027's Test strategy #8 — "`--upstream`/`--refresh-interval`
//! produce the retirement refusal (quoting the flag, naming ADR-027) before any
//! bind; a leftover `cache-v1.json`/`catalog-v1.json` in the registry directory is
//! a named refusal except at the path `--corpus` names; a second process against a
//! `<data>` whose `write.lock` exists refuses before bind and leaves the file
//! alone, and a run removes it on exit; missing key without `--no-steam` refuses;
//! missing `<data>` refuses; `--no-steam` starts with health `steam_snapshot:
//! "none"`" — together with §Hosted mode's startup line ("`registry hosted
//! listening on http://{bound}`") and §Concurrency's `write.lock` clause.
//!
//! Every run here is either `--no-steam` or carries a fresh planted snapshot, so
//! the store pull is never reached: no run in this suite resolves or dials
//! anything outside the process (Test strategy #17).

mod common;

use common::{
    Registry, TempDir, corpus_doc, expect_refused_startup, json_body, registry_document,
    run_registry,
};
use std::path::{Path, PathBuf};

/// The sentinel Web API key. It is not a credential, and `027-L4`'s rule is that
/// nothing like it ever reaches an output or a store request.
const KEY: &str = "bindforge-test-steam-key-0f1e2d3c";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";

fn xdg_of(data: &TempDir) -> String {
    data.path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string()
}

/// One `--root --corpus <path>` hosted invocation, as `026-C1`/`027-C1` spell it.
fn hosted_args<'a>(corpus: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["serve", "--root", "--corpus", corpus];
    args.extend_from_slice(extra);
    args
}

fn corpus_in(data: &TempDir, name: &str) -> PathBuf {
    data.write(
        name,
        corpus_doc(
            3,
            &[("example-game", "Example Game", &["steam/900001"][..])],
        )
        .as_bytes(),
    )
}

fn registry_dir_document(data: &TempDir, file: &str) -> PathBuf {
    registry_document(data.path(), file)
}

/// proves: Test strategy #8's first clause, first spelling — `--upstream` is
/// refused *as a retirement*, quoting the flag and naming ADR-027, distinctly from
/// the unknown-flag refusal (§Hosted mode: "the flags stay in the parser *only* to
/// be refused, so a stale operator invocation explains itself").
#[test]
fn the_retired_upstream_flag_quotes_the_flag_and_names_the_retirement() {
    let data = TempDir::new("start-upstream");
    let xdg = xdg_of(&data);
    let corpus = corpus_in(&data, "corpus.json");
    let corpus_path = corpus.display().to_string();

    let retired = expect_refused_startup(
        &hosted_args(
            &corpus_path,
            &["--upstream", "http://127.0.0.1:9", "--no-steam"],
        ),
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
    );
    let text = format!("{}{}", retired.stdout, retired.stderr).to_ascii_lowercase();
    assert!(
        !text.contains(&KEY.to_ascii_lowercase()),
        "a startup refusal never prints the configured Web API key (§The store pull's \
         diagnostics rule): {:?}",
        retired.stderr
    );
    assert!(
        text.contains("--upstream"),
        "the refusal must quote the flag it refuses (§Hosted mode): {:?}",
        retired.stderr
    );
    assert!(
        text.contains("adr-027") || text.contains("adr 027") || text.contains("027"),
        "the refusal must name the retirement (`caching mode retired with ADR-027`): {:?}",
        retired.stderr
    );
    assert!(
        text.contains("retired") || text.contains("caching"),
        "a stale operator invocation explains itself as a retirement: {:?}",
        retired.stderr
    );

    let unknown = expect_refused_startup(
        &hosted_args(&corpus_path, &["--totally-not-a-flag", "--no-steam"]),
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert_ne!(
        retired.stderr, unknown.stderr,
        "the retirement refusal is distinct from the unknown-flag refusal (§Hosted mode): \
         both said {:?}",
        unknown.stderr
    );
}

/// proves: Test strategy #8's first clause, second spelling —
/// `--refresh-interval` joins the same named-refusal family before any bind.
#[test]
fn the_retired_refresh_interval_flag_quotes_the_flag_and_names_the_retirement() {
    let data = TempDir::new("start-refresh");
    let xdg = xdg_of(&data);
    let corpus = corpus_in(&data, "corpus.json");
    let corpus_path = corpus.display().to_string();

    let retired = expect_refused_startup(
        &hosted_args(&corpus_path, &["--refresh-interval", "900", "--no-steam"]),
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    let text = format!("{}{}", retired.stdout, retired.stderr).to_ascii_lowercase();
    assert!(
        text.contains("--refresh-interval"),
        "the refusal must quote the flag it refuses: {:?}",
        retired.stderr
    );
    assert!(
        text.contains("adr-027") || text.contains("adr 027") || text.contains("027"),
        "the refusal must name ADR-027 as the retirement: {:?}",
        retired.stderr
    );
}

/// proves: Test strategy #8's second clause, first half — the retired
/// cache-document pair is a startup refusal *by presence* in the registry
/// directory, so an upgraded host cleans its data directory rather than carrying
/// inert state (§Hosted mode).
#[test]
fn a_leftover_cache_document_in_the_registry_directory_refuses_startup() {
    for leftover in ["cache-v1.json", "catalog-v1.json"] {
        let data = TempDir::new("start-leftover");
        let xdg = xdg_of(&data);
        let corpus = corpus_in(&data, "corpus.json");
        let corpus_path = corpus.display().to_string();
        data.write(
            &format!("bindforge/registry/{leftover}"),
            b"{\"version\":1,\"entries\":[]}\n",
        );

        let run = expect_refused_startup(
            &hosted_args(&corpus_path, &["--no-steam"]),
            &[("XDG_DATA_HOME", xdg.as_str())],
        );
        let text = format!("{}{}", run.stdout, run.stderr).to_ascii_lowercase();
        assert!(
            text.contains(leftover),
            "`{leftover}` in the registry directory must be refused by name before anything \
             binds (§Hosted mode); the run said {:?}",
            run.stderr
        );
    }
}

/// proves: Test strategy #8's second clause, exception half — the path `--corpus`
/// names is spared, which is how an operator keeps reading their old entries
/// across the upgrade (§Hosted mode).
#[test]
fn the_corpus_flag_path_is_spared_by_the_leftover_document_refusal() {
    let data = TempDir::new("start-corpus-exception");
    let xdg = xdg_of(&data);
    // The old caching document, kept on purpose, and pointed at as the corpus.
    let kept = data.write(
        "bindforge/registry/catalog-v1.json",
        corpus_doc(2, &[("kept-game", "Kept Game", &["steam/900007"][..])]).as_bytes(),
    );
    assert_eq!(
        kept,
        registry_dir_document(&data, "catalog-v1.json"),
        "the spared path is the one inside the registry directory"
    );

    let server = Registry::start(
        "start-corpus-exception",
        &hosted_args(&kept.display().to_string(), &["--no-steam"]),
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    let answer = server.get("/v1/candidates?slug=steam%2F900007");
    assert_eq!(
        common::game_ids(&answer),
        vec!["kept-game".to_string()],
        "the entries an operator copies out keep answering (§Rollout 6)"
    );
}

/// proves: Test strategy #8's third clause — a missing or empty
/// `BINDFORGE_STEAM_WEB_API_KEY` without `--no-steam` is a named startup refusal,
/// required regardless of snapshot freshness because the refresh thread is part of
/// the run (§Hosted mode, `027-C3`'s environment-only key custody).
#[test]
fn a_missing_or_empty_steam_key_without_no_steam_refuses_startup() {
    let data = TempDir::new("start-key");
    let xdg = xdg_of(&data);
    let corpus = corpus_in(&data, "corpus.json");
    let corpus_path = corpus.display().to_string();

    let missing = expect_refused_startup(
        &hosted_args(&corpus_path, &[]),
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    let present_but_empty = expect_refused_startup(
        &hosted_args(&corpus_path, &[]),
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, "")],
    );
    for (label, run) in [("missing", missing), ("empty", present_but_empty)] {
        let text = format!("{}{}", run.stdout, run.stderr).to_ascii_lowercase();
        assert!(
            text.contains("key"),
            "the `{label}` key posture is a named refusal (§Hosted mode): {:?}",
            run.stderr
        );
        assert!(
            text.contains("steam"),
            "the `{label}` key refusal names the steam Web API key: {:?}",
            run.stderr
        );
    }
}

/// proves: Test strategy #8's fourth clause — a resolvable `<data>` directory is
/// now a startup requirement, and its absence is a named refusal: "a posture
/// change from root mode, which needed none" (§Hosted mode).
#[test]
fn an_unresolvable_data_directory_refuses_startup() {
    let data = TempDir::new("start-no-data");
    let corpus = corpus_in(&data, "corpus.json");

    // `expect_refused_startup` clears `XDG_DATA_HOME`, `HOME` and `APPDATA`
    // before applying the pairs below, so nothing here resolves to a `<data>`.
    let run = expect_refused_startup(
        &hosted_args(&corpus.display().to_string(), &["--no-steam"]),
        &[],
    );
    let text = format!("{}{}", run.stdout, run.stderr).to_ascii_lowercase();
    assert!(
        text.contains("data"),
        "an unresolvable `<data>` is a named startup refusal (§Hosted mode): {:?}",
        run.stderr
    );
}

/// proves: Test strategy #8's last clause with §Hosted mode's startup line —
/// `--no-steam` starts with no key and no snapshot, prints the hosted line, and
/// reports health `steam_snapshot: "none"`.
#[test]
fn no_steam_starts_and_reports_the_none_spelling() {
    let data = TempDir::new("start-no-steam");
    let xdg = xdg_of(&data);
    let corpus = corpus_in(&data, "corpus.json");

    let server = Registry::start(
        "start-no-steam",
        &hosted_args(&corpus.display().to_string(), &["--no-steam"]),
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    let started = server.stdout_text().to_ascii_lowercase();
    assert!(
        started.contains("hosted"),
        "the startup line prints the mode (§Hosted mode: `registry hosted listening on \
         http://{{bound}}`): {:?}",
        server.stdout_text()
    );
    assert!(
        started.contains(&format!("http://127.0.0.1:{}", server.port())),
        "the startup line names the bound address the tests then dial: {:?}",
        server.stdout_text()
    );
    assert!(
        !registry_dir_document(&data, "steam-snapshot.json").exists(),
        "`--no-steam` never reads the snapshot file, so it must never write one either \
         (§The steam snapshot resolver)"
    );

    let health = json_body(&server.health());
    assert_eq!(health["mode"].as_str(), Some("hosted"));
    assert_eq!(
        health["steam_snapshot"].as_str(),
        Some("none"),
        "`--no-steam` is the resolver-inactive spelling (§Hosted mode): {health:?}"
    );
}

/// proves: Test strategy #8's fifth clause, first half — a second process against
/// a `<data>` whose `write.lock` exists is the named refusal `registry data
/// directory is locked`, refused before bind, and it leaves the file alone
/// (§Concurrency: "finding it present is the named startup refusal").
#[test]
fn a_held_write_lock_refuses_the_second_process_and_is_left_alone() {
    let data = TempDir::new("start-lock");
    let xdg = xdg_of(&data);
    let corpus = corpus_in(&data, "corpus.json");
    let lock = data.write("bindforge/registry/write.lock", b"");
    let before = std::fs::read(&lock).expect("the planted lock is readable");

    let run = expect_refused_startup(
        &hosted_args(&corpus.display().to_string(), &["--no-steam"]),
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    let text = format!("{}{}", run.stdout, run.stderr).to_ascii_lowercase();
    assert!(
        text.contains("locked") || text.contains("write.lock"),
        "a held `<data>` lock is the named refusal `registry data directory is locked` \
         (§Concurrency): {:?}",
        run.stderr
    );
    assert_eq!(
        std::fs::read(&lock).expect("the lock survives the refused run"),
        before,
        "the refused run must not touch, replace, or take over the lock it refused"
    );
    assert!(
        lock.exists(),
        "the refused run must not remove the live owner's lock"
    );
}

/// proves: Test strategy #8's fifth clause, second half with §Hosted mode's
/// stopped-instance posture and §Concurrency's crashed-run clause — a run takes the
/// lock while it lives and removes it on exit, `query` refuses to start beside a live
/// `serve`, and a run that was *killed* leaves the file behind for the runbook.
///
/// The two exits are different acts and the leg keeps them apart: `common::Registry`
/// is closed by `kill()`, which is the crashed run §Concurrency names — the lock stays
/// and every next instance is refused, because a lock file cannot say who holds the
/// catalog. `registry query` is the exit a test can arrange that a run performs, and it
/// is the one that proves "remove it on exit". Between them sits Rollout 6's runbook
/// act, which the harness performs here and nowhere in the product may: deleting a
/// crashed run's lock is a human deciding the holder is gone.
#[test]
fn a_run_takes_the_write_lock_and_a_clean_exit_removes_it() {
    let data = TempDir::new("start-lock-lifecycle");
    let xdg = xdg_of(&data);
    let corpus = corpus_in(&data, "corpus.json");
    let corpus_path = corpus.display().to_string();

    let server = Registry::start(
        "start-lock-lifecycle",
        &hosted_args(&corpus_path, &["--no-steam"]),
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        registry_dir_document(&data, "write.lock").is_file(),
        "a live hosted run holds `<data>/bindforge/registry/write.lock` (§Concurrency)"
    );

    let beside = run_registry(
        &["query", "steam/900001", "--root", "--corpus", &corpus_path],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        !beside.success(),
        "`query` is a stopped-instance tool: it must refuse to run beside a live `serve` \
         (§Hosted mode, Test strategy #15); it exited {:#?} with stderr {:?}",
        beside.code(),
        beside.stderr
    );
    assert!(
        beside.stderr.to_ascii_lowercase().contains("lock"),
        "the refusal says why: {:?}",
        beside.stderr
    );

    drop(server);

    assert!(
        registry_dir_document(&data, "write.lock").is_file(),
        "a killed run is a crashed run, and §Concurrency says the crashed run leaves the \
         lock for the runbook: a next instance must be refused, not trusted"
    );
    let orphan = run_registry(
        &[
            "query",
            "steam/900001",
            "--root",
            "--corpus",
            &corpus_path,
            "--no-steam",
        ],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        !orphan.success(),
        "presence is the whole rule: a leftover lock refuses the next run whatever its \
         stamped owner says (Rollout 6 leaves the decision to the operator); stderr: {}",
        orphan.stderr
    );

    common::clear_crashed_run_write_lock(data.path());

    let one_shot = run_registry(
        &[
            "query",
            "steam/900001",
            "--root",
            "--corpus",
            &corpus_path,
            "--no-steam",
        ],
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    assert!(
        one_shot.success(),
        "with the runbook's act done and no live instance, the one-shot run starts; stderr: {}",
        one_shot.stderr
    );
    assert!(
        !registry_dir_document(&data, "write.lock").exists(),
        "a run removes `write.lock` on exit (§Concurrency: \"take … and remove it on \
         exit\"); the tree below `<data>` is {:?}",
        data.files_below("bindforge")
    );
}

/// proves: §Hosted mode's startup requirement read against the shipped contract —
/// `--root --corpus <path>` is still required, and a hosted run with no corpus
/// file at that path is a named refusal, never an empty world.
#[test]
fn a_hosted_run_without_a_corpus_file_refuses_startup() {
    let data = TempDir::new("start-no-corpus");
    let xdg = xdg_of(&data);
    let missing: &Path = &data.join("corpus.json");
    assert!(!missing.exists(), "the corpus file is deliberately absent");

    let run = expect_refused_startup(
        &hosted_args(&missing.display().to_string(), &["--no-steam"]),
        &[("XDG_DATA_HOME", xdg.as_str())],
    );
    let text = format!("{}{}", run.stdout, run.stderr).to_ascii_lowercase();
    assert!(
        text.contains("corpus"),
        "the missing corpus is refused by name (`026-C1` survives into hosted mode, \
         §The steam snapshot resolver's startup order): {:?}",
        run.stderr
    );
}
