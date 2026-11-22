//! The issued document as a write path and as a contract document: mint ordering,
//! revision, the cap, and the `026-C3` loader refusals applied verbatim.
//!
//! proves: design 027's Test strategy #4 — "persist-before-serve (an injected
//! persist failure answers the named `503` with model, `revision`, and file
//! untouched, and nothing served); dedup answers the existing record with zero
//! writes; at `MAX_ISSUED_ENTRIES` a novel key is `503 mint cap reached` while a
//! held key still answers; the octet cap refuses the same way; `revision` bumps per
//! mint; two concurrent mints of one key leave one record and one id" — and Test
//! strategy #7 — "`026-C3` refusals apply verbatim; absent file = empty document;
//! the `MAX_ISSUED_DOCUMENT_OCTETS` load refusal is metadata-gated" — read against
//! §Issued entries' Ordering, §Concurrency's lock scope, and §Budgets.
//!
//! The persist failure is injected the only way a black-box suite can: the
//! registry directory is made unwritable while the instance runs, so the
//! `create_new` tmp and the rename both fail (§Assumption register A2's discipline
//! is the thing being broken). The cap tests plant real records because there is
//! no budget knob: `027-B3`'s 200,000 is a constant, not a flag.

mod common;

use common::{
    Registry, TempDir, body_text, corpus_doc, json_body, now_unix, status_code, status_line,
};
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::sync::Barrier;

const KEY: &str = "bindforge-test-steam-key-0f1e2d3c";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";
const ISSUED: &str = "bindforge/registry/issued-v1.json";

/// `027-B3`'s cap, as §Budgets spells the constant.
const MAX_ISSUED_ENTRIES: usize = 200_000;
/// `MAX_ISSUED_DOCUMENT_OCTETS`, §Budgets.
const MAX_ISSUED_DOCUMENT_OCTETS: usize = 67_108_864;

fn xdg_of(data: &TempDir) -> String {
    data.path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string()
}

fn plant_snapshot(data: &TempDir, apps: &[(u64, &str)]) {
    let mut ordered: Vec<(u64, &str)> = apps.to_vec();
    ordered.sort_by_key(|(appid, _)| *appid);
    assert_eq!(
        ordered,
        apps.to_vec(),
        "the fixture must be in `appid` order"
    );
    let entries: Vec<serde_json::Value> = ordered
        .iter()
        .map(|(appid, name)| serde_json::json!({ "appid": appid, "name": name }))
        .collect();
    let document = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "fetched_at": now_unix(),
        "apps": entries,
    }))
    .expect("a snapshot document serialises");
    data.write(
        "bindforge/registry/steam-snapshot.json",
        document.as_bytes(),
    );
}

fn corpus_in(data: &TempDir, games: &[(&str, &str, &[&str])]) -> std::path::PathBuf {
    data.write("corpus.json", corpus_doc(3, games).as_bytes())
}

fn serve(label: &str, data: &TempDir, corpus: &Path, extra: &[&str]) -> Registry {
    let corpus_path = corpus.display().to_string();
    let mut args = vec!["serve", "--root", "--corpus", corpus_path.as_str()];
    args.extend_from_slice(extra);
    Registry::start(
        label,
        &args,
        &[("XDG_DATA_HOME", xdg_of(data).as_str()), (KEY_VAR, KEY)],
    )
}

fn issued_document(data: &TempDir) -> serde_json::Value {
    let path = data.join(ISSUED);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
    serde_json::from_str(&text).expect("the issued document is a catalog document")
}

fn issued_records(document: &serde_json::Value) -> Vec<String> {
    document["games"]
        .as_array()
        .expect("a games list")
        .iter()
        .map(|game| game["game_id"].as_str().expect("a game_id").to_string())
        .collect()
}

/// proves: §Issued entries' Ordering with `027-C4`'s unit text — the whole issued
/// record is on disk before the answer is written out, so a record a client was
/// served cannot be lost by a crash or a restart, and `revision` moved with it.
///
/// The restart is a second instance over the same `<data>`, and the one act between
/// them is Rollout 6's runbook line: `common::Registry` is closed by `kill()`, which
/// makes the first run §Concurrency's crashed run, and its `write.lock` refuses the
/// next one until the operator deletes it. §Concurrency's single-writer rule is what
/// makes "persisted before served" mean something across a restart, so the leg keeps
/// the refusal in view rather than arranging around it invisibly.
#[test]
fn an_issued_record_is_on_disk_before_the_answer_and_survives_a_restart() {
    let data = TempDir::new("issued-persist");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_300_001, "Tunic")]);
    let server = serve("issued-persist", &data, &corpus, &[]);

    let answer = server.get("/v1/candidates?slug=steam%2F9300001");
    let served = common::game_ids(&answer);
    assert_eq!(served, vec!["tunic".to_string()]);
    assert_eq!(
        issued_records(&issued_document(&data)),
        vec!["tunic".to_string()],
        "the record was persisted before it was served (§Issued entries' Ordering: \
         \"persist the whole issued record, swap the in-memory model, and only then write \
         the response out\")"
    );
    assert_eq!(
        issued_document(&data)["revision"].as_u64(),
        Some(1),
        "an absent issued document is an empty document at revision 0, and a mint bumps it"
    );

    drop(server);
    assert!(
        common::write_lock(data.path()).is_file(),
        "the killed run left its `<data>` lock behind, exactly as §Concurrency says a \
         crashed run does"
    );
    common::clear_crashed_run_write_lock(data.path());

    let restarted = serve("issued-persist-2", &data, &corpus, &[]);
    assert_eq!(
        common::game_ids(&restarted.get("/v1/candidates?slug=steam%2F9300001")),
        vec!["tunic".to_string()],
        "an identity a client was served can never be minted twice: the next instance \
         reads the document, not the snapshot"
    );
    assert_eq!(
        issued_document(&data)["revision"].as_u64(),
        Some(1),
        "and the restart does not mint it again"
    );
}

/// proves: Test strategy #4's first clause — an injected persist failure is the
/// named `503` with the model, the `revision`, and the file untouched, and nothing
/// served: a served-but-unsaved identity is the one double-mint path `027-L2`
/// cannot survive, so the write happens first and its failure ends the request.
#[cfg(unix)]
#[test]
fn an_injected_persist_failure_answers_the_named_503_and_touches_nothing() {
    use std::os::unix::fs::PermissionsExt;

    let data = TempDir::new("issued-persist-fail");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(
        &data,
        &[(9_310_001, "Outer Wilds"), (9_310_002, "Returnal")],
    );
    data.write(ISSUED, corpus_doc(4, &[]).as_bytes());
    let registry_directory = data.join("bindforge/registry");
    let issued_path = data.join(ISSUED);
    let before = std::fs::read(&issued_path).expect("the planted issued document is readable");
    let server = serve("issued-persist-fail", &data, &corpus, &[]);
    let before_stats = json_body(&server.stats(None));

    // The injection: no tmp may be created and no rename may happen.
    std::fs::set_permissions(&registry_directory, std::fs::Permissions::from_mode(0o500))
        .expect("the registry directory becomes unwritable");

    let answer = server.get("/v1/candidates?slug=steam%2F9310001");
    let line = status_line(&answer);
    let code = status_code(&answer);
    std::fs::set_permissions(&registry_directory, std::fs::Permissions::from_mode(0o700))
        .expect("the injection is undone before the temp directory is cleaned");

    assert_eq!(
        code,
        Some(503),
        "an identity that cannot be persisted is not an identity (§The hosted lookup policy \
         step 5, `027-B3`): {line}"
    );
    assert!(
        !body_text(&answer).contains('['),
        "nothing is served: the body carries no candidate list at all: {:?}",
        body_text(&answer)
    );
    assert_eq!(
        std::fs::read(&issued_path).expect("the issued document survives"),
        before,
        "a failed persist leaves the file untouched"
    );
    assert_eq!(
        json_body(&server.stats(None)),
        before_stats,
        "and leaves the model and `revision` untouched"
    );
    assert_eq!(
        data.files_below("bindforge/registry")
            .into_iter()
            .filter(|path| path.display().to_string().ends_with(".tmp"))
            .count(),
        0,
        "no half-written tmp is left behind (§Assumption register A2's discipline)"
    );

    // And the same key still works once the write path is healthy again.
    let recovered = server.get("/v1/candidates?slug=steam%2F9310001");
    assert_eq!(
        status_code(&recovered),
        Some(200),
        "the failure was the filesystem's, not the key's: {}",
        status_line(&recovered)
    );
}

/// proves: Test strategy #4's third clause with §Concurrency — `revision` is
/// bumped by every mint and by nothing else, and a repeated key pays no mint.
#[test]
fn the_issued_revision_bumps_once_per_mint_and_not_on_a_hit() {
    let data = TempDir::new("issued-revision");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(
        &data,
        &[
            (9_320_001, "Hades"),
            (9_320_002, "Hades II"),
            (9_320_003, "Hades III"),
        ],
    );
    let server = serve("issued-revision", &data, &corpus, &[]);

    let mut seen = Vec::new();
    for appid in [9_320_001_u64, 9_320_002, 9_320_003] {
        let answer = server.get(&common::slug_query(&format!("steam/{appid}")));
        assert_eq!(status_code(&answer), Some(200), "{}", status_line(&answer));
        seen.push(issued_document(&data)["revision"].as_u64());
    }
    assert_eq!(
        seen,
        vec![Some(1), Some(2), Some(3)],
        "every mint bumps the issued document's own `revision`"
    );

    let before = std::fs::read(data.join(ISSUED)).expect("the issued document is readable");
    for _ in 0..3 {
        server.get("/v1/candidates?slug=steam%2F9320001");
    }
    assert_eq!(
        std::fs::read(data.join(ISSUED)).expect("the issued document survives"),
        before,
        "keys already held answer from the issued document forever, with no mint and no \
         `revision` move (§The hosted lookup policy step 2)"
    );
}

/// proves: Test strategy #4's second clause — two concurrent mints of one key,
/// serialised by the one mutation mutex, leave exactly one record and one id.
#[test]
fn two_concurrent_mints_of_one_key_leave_one_record_and_one_id() {
    let data = TempDir::new("issued-race");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_330_001, "Celeste")]);
    let server = serve("issued-race", &data, &corpus, &[]);

    let target = common::slug_query("steam/9330001");
    let host = server.host_header();
    let streams: Vec<TcpStream> = (0..8).map(|_| server.connection()).collect();
    let barrier = Arc::new(Barrier::new(streams.len()));
    let handles: Vec<_> = streams
        .into_iter()
        .map(|mut stream| {
            let target = target.clone();
            let host = host.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                stream
                    .write_all(&common::build_request("GET", &target, &host, &[], ""))
                    .expect("the request goes out");
                stream.flush().ok();
                common::read_to_end(&mut stream)
            })
        })
        .collect();
    let answers: Vec<Vec<u8>> = handles
        .into_iter()
        .map(|handle| handle.join().expect("the request thread finishes"))
        .collect();

    let mut ids = Vec::new();
    for answer in &answers {
        assert_eq!(
            status_code(answer),
            Some(200),
            "every concurrent request for the one key is answered: {}",
            status_line(answer)
        );
        ids.extend(common::game_ids(answer));
    }
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids,
        vec!["celeste".to_string()],
        "one key mints exactly one identity, however many requests race it (§Concurrency: \
         the mutation mutex \"is also what stops two concurrent mints of one key from both \
         passing the dedup check\")"
    );
    assert_eq!(
        issued_records(&issued_document(&data)),
        vec!["celeste".to_string()],
        "and the document holds exactly one record"
    );
    assert_eq!(
        issued_document(&data)["revision"].as_u64(),
        Some(1),
        "one mint, one `revision` bump"
    );
    assert!(
        !body_text(&answers[0]).contains(KEY),
        "the store key is nowhere in an answer (`027-L4`)"
    );
}

/// proves: Test strategy #4's third clause with `027-B3` — at the cap a *novel*
/// key is the named `503 mint cap reached`, literally even when the snapshot knows
/// it, while every key already held keeps answering: the cap degrades only novel
/// keys.
#[test]
fn at_the_issued_cap_a_novel_key_is_refused_while_held_keys_answer() {
    let data = TempDir::new("issued-cap");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_340_001, "A Brand New Game")]);

    // `027-B3` is a constant, so the only honest way to be at it is to be at it.
    let mut held: Vec<serde_json::Value> = Vec::with_capacity(MAX_ISSUED_ENTRIES);
    for index in 0..MAX_ISSUED_ENTRIES {
        held.push(serde_json::json!({
            "game_id": format!("issued-{index}"),
            "name": format!("Issued Game {index}"),
            "source_keys": [format!("steam/{}", 700_000_000_u64 + index as u64)],
        }));
    }
    let document = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "revision": MAX_ISSUED_ENTRIES as u64,
        "games": held,
    }))
    .expect("a large issued document serialises");
    data.write(ISSUED, document.as_bytes());

    let server = serve("issued-cap", &data, &corpus, &[]);
    let novel = server.get("/v1/candidates?slug=steam%2F9340001");
    assert_eq!(
        status_code(&novel),
        Some(503),
        "an identity that cannot be persisted is not an identity, even for a key the \
         snapshot knows (`027-B3`): {}",
        status_line(&novel)
    );
    assert!(
        status_line(&novel)
            .to_ascii_lowercase()
            .replace('-', " ")
            .contains("cap"),
        "the refusal names the cap (`mint cap reached`), so an operator who hits steady \
         state reads it as such: {}",
        status_line(&novel)
    );

    let held_answer = server.get("/v1/candidates?slug=steam%2F700000042");
    assert_eq!(
        common::game_ids(&held_answer),
        vec!["issued-42".to_string()],
        "keys already held answer from the issued document forever: the cap check is behind \
         the issued join (§The hosted lookup policy step 2 precedes the cap check)"
    );
    assert_eq!(
        json_body(&server.stats(None))["issued"].as_u64(),
        Some(MAX_ISSUED_ENTRIES as u64),
        "and `stats.issued` is where the approach is visible (§Risks)"
    );
}

/// proves: Test strategy #7's first two clauses — an absent issued document is an
/// empty document (unlike the corpus, which must exist), and the `026-C3` loader
/// refusals apply verbatim to it: a document the catalog contract refuses is a
/// named startup refusal that binds nothing.
#[test]
fn the_issued_document_loads_under_the_catalog_contracts_own_refusals() {
    let clean = TempDir::new("issued-absent");
    let clean_corpus = corpus_in(
        &clean,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    let clean_server = serve("issued-absent", &clean, &clean_corpus, &["--no-steam"]);
    assert_eq!(
        json_body(&clean_server.stats(None))["issued"].as_u64(),
        Some(0),
        "an absent issued file is an empty document, not an error"
    );
    assert!(
        !clean.join(ISSUED).exists(),
        "and starting up does not create one: {:?}",
        clean.files_below("bindforge")
    );
    drop(clean_server);

    let cases: Vec<(&str, String)> = vec![
        (
            "wrong-version",
            serde_json::to_string(&serde_json::json!({
                "version": 2, "revision": 1, "games": [],
            }))
            .expect("a document serialises"),
        ),
        (
            "uppercase-game-id",
            corpus_doc(
                1,
                &[("Example-Game", "Example Game", &["steam/900002"][..])],
            ),
        ),
        (
            "over-long-name",
            corpus_doc(1, &[("ok-id", &"n".repeat(129), &["steam/900002"][..])]),
        ),
        (
            "unknown-source-key",
            corpus_doc(1, &[("ok-id", "Ok", &["gog/1"][..])]),
        ),
        (
            // The refusal the shared `026-C3` contract actually carries is one key
            // repeated *inside a single record*. Two records holding one key is not a
            // refusal at all — it is what a two-candidate answer is (`catalog_document.rs`
            // pins both halves for the corpus), so this is the spelling that has to be
            // refused here too.
            "source-key-repeated-within-one-record",
            serde_json::to_string(&serde_json::json!({
                "version": 1,
                "revision": 1,
                "games": [
                    {
                        "game_id": "one",
                        "name": "One",
                        "source_keys": ["steam/500", "steam/500"],
                    },
                ],
            }))
            .expect("a document serialises"),
        ),
        ("not-json", "this is not a catalog document".to_string()),
    ];

    for (label, document) in cases {
        let data = TempDir::new("issued-refusal");
        let corpus = corpus_in(
            &data,
            &[("example-game", "Example Game", &["steam/900001"][..])],
        );
        data.write(ISSUED, document.as_bytes());
        let run = common::expect_refused_startup(
            &[
                "serve",
                "--root",
                "--corpus",
                &corpus.display().to_string(),
                "--no-steam",
            ],
            &[("XDG_DATA_HOME", xdg_of(&data).as_str())],
        );
        let text = format!("{}{}", run.stdout, run.stderr).to_ascii_lowercase();
        assert!(
            text.contains("issued") || text.contains("catalog") || text.contains("document"),
            "`{label}` must be refused by name as a contract document (§Issued entries' \
             \"same loader refusals\"); the run said {:?}",
            run.stderr
        );
    }
}

/// proves: Test strategy #7's last clause — the `MAX_ISSUED_DOCUMENT_OCTETS` load
/// refusal is metadata-gated: a shape-valid document is refused on its size alone,
/// so the file is never read (A2's load discipline, as the shipped corpus read
/// already applies it).
#[test]
fn an_over_bound_issued_document_is_refused_on_its_size_alone() {
    let data = TempDir::new("issued-octets");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    // Valid `026-C3` shape, one record, and enough leading JSON whitespace to put
    // `fs::metadata` over the bound: only a size check can refuse this, because
    // every shape check would parse it first.
    let small = corpus_doc(1, &[("ok-id", "Ok", &["steam/900002"][..])]);
    let path = data.join(ISSUED);
    // The padding is streamed, so this fixture cannot go through the temp
    // directory's one-shot writer — and that writer is what creates parents, so the
    // registry directory is made here instead.
    std::fs::create_dir_all(path.parent().expect("a registry directory as the parent"))
        .expect("the registry directory exists");
    let mut file = std::fs::File::create(&path).expect("the issued document is creatable");
    file.write_all(vec![b' '; MAX_ISSUED_DOCUMENT_OCTETS - small.len() + 1].as_slice())
        .expect("the padding is written");
    file.write_all(small.as_bytes())
        .expect("the document follows the padding");
    drop(file);
    assert!(
        std::fs::metadata(&path).expect("metadata").len() > MAX_ISSUED_DOCUMENT_OCTETS as u64,
        "the fixture is over the bound the loader must gate on"
    );

    let run = common::expect_refused_startup(
        &[
            "serve",
            "--root",
            "--corpus",
            &corpus.display().to_string(),
            "--no-steam",
        ],
        &[("XDG_DATA_HOME", xdg_of(&data).as_str())],
    );
    let text = format!("{}{}", run.stdout, run.stderr).to_ascii_lowercase();
    assert!(
        text.contains("octet") || text.contains("bound") || text.contains("large"),
        "an over-`MAX_ISSUED_DOCUMENT_OCTETS` issued document is refused by name on its \
         size, before it is read (§Issued entries): {:?}",
        run.stderr
    );
    assert!(
        !text.contains("version"),
        "the refusal is the size gate, not a shape refusal — the file was never parsed: \
         {:?}",
        run.stderr
    );
}

/// proves: §Issued entries' file posture read together with §Concurrency — the
/// issued document is owner-only after a mint, the same discipline as every other
/// registry document, and no tmp survives.
#[cfg(unix)]
#[test]
fn a_minted_issued_document_is_owner_only_and_leaves_no_tmp() {
    let data = TempDir::new("issued-modes");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900001"][..])],
    );
    plant_snapshot(&data, &[(9_350_001, "Slay The Spire")]);
    let server = serve("issued-modes", &data, &corpus, &[]);
    server.get("/v1/candidates?slug=steam%2F9350001");

    let path = data.join(ISSUED);
    assert!(path.is_file(), "the mint persisted the record");
    assert_eq!(
        common::mode_of(&path) & 0o777,
        0o600,
        "issued records are the instance's own state: owner-only, like every document it \
         writes (A2)"
    );
    let mut survivors: Vec<String> = data
        .files_below("bindforge/registry")
        .iter()
        .map(|path| {
            path.file_name()
                .expect("a file name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    survivors.sort();
    assert_eq!(
        survivors,
        vec![
            "issued-v1.json".to_string(),
            "steam-snapshot.json".to_string(),
            "write.lock".to_string(),
        ],
        "the fixed-tmp-and-rename discipline leaves nothing else in the directory"
    );
}
