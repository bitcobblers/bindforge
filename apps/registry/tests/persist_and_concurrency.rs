//! The persist and concurrency rules the single-process model buys: the fixed-name
//! tmp is the only writer arbitration (a stale one is a hard failure, never a
//! write-through), serving reads come from the locked in-memory model so they can
//! never tear, N concurrent mutations leave the file equal to the model with
//! `revision` at the final value, every request-side budget is refused *before*
//! anything mutates, and connections past the admit ceiling are closed at accept
//! with no status line at all.
//!
//! proves: design 026's Test strategy #11 — "Atomic persist + concurrency: a second
//! save over a stale tmp fails loudly (A7); a reader never sees a torn document
//! (served from the locked in-memory model); N concurrent upserts leave the
//! persisted document equal to the model with `revision` at the final value;
//! `MAX_REQUEST_HEADER_OCTETS`/`MAX_REQUEST_BODY_OCTETS` refusals — a
//! `Transfer-Encoding`, `Content-Length`-less, or over-cap body is refused before
//! any mutation; over-`MAX_CONCURRENT_CONNECTIONS` (64) is closed at accept with no
//! status line."
//!
//! §Concurrency (open Q10) is the unit being tested here, and §Risks records its
//! sharpest consequence: "Two `registry serve` on one corpus fail loudly (by design,
//! A7) … the loud write-time tmp failure is the whole mechanism — there is no
//! startup check or instance marker." So nothing here waits on a race: the refused
//! two-writer state is pinned through the mechanism itself, with a stale tmp already
//! in place, which is exactly the state a second writer's crashed save leaves.
//!
//! §Budgets names the three ceilings, all private to `apps/registry`:
//! `MAX_REQUEST_HEADER_OCTETS` 65,536, `MAX_REQUEST_BODY_OCTETS` = the document cap
//! ("so no per-record key-count bound"), `MAX_CONCURRENT_CONNECTIONS` 64.

mod common;

use common::{
    Registry, TempDir, body_text, build_request, build_request_with_length_claim,
    build_request_without_length, corpus_doc, json_body, read_to_end, record_body, status_code,
    status_line,
};
use std::io::Write;
use std::path::Path;
use std::time::Duration;

/// §Budgets: the request header budget, mirroring the client's own header cap (A1).
const MAX_REQUEST_HEADER_OCTETS: usize = 65_536;
/// §Budgets: the request body budget *is* `MAX_CATALOG_DOCUMENT_OCTETS`, which is why
/// no per-record key-count bound exists.
const MAX_REQUEST_BODY_OCTETS: usize = 16_777_216;
/// §Budgets: one thread per admitted connection, over-cap closed at accept.
const MAX_CONCURRENT_CONNECTIONS: usize = 64;

fn root(label: &str) -> (TempDir, std::path::PathBuf, Registry) {
    let data = TempDir::new(label);
    let corpus = data.write(
        "corpus.json",
        corpus_doc(
            5,
            &[("example-game", "Example Game", &["steam/900001"][..])],
        )
        .as_bytes(),
    );
    let server = Registry::root(label, &corpus, &[]);
    (data, corpus, server)
}

fn document_of(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path).expect("the catalog document is readable");
    serde_json::from_str(&text).expect("the persisted catalog document is JSON")
}

fn sorted_ids(document: &serde_json::Value) -> Vec<String> {
    let mut ids: Vec<String> = document["games"]
        .as_array()
        .expect("games")
        .iter()
        .map(|game| game["game_id"].as_str().expect("a game_id").to_string())
        .collect();
    ids.sort();
    ids
}

fn stats(server: &Registry) -> serde_json::Value {
    let answer = server.get("/v1/admin/stats");
    assert_eq!(
        status_code(&answer),
        Some(200),
        "loopback stats answers: {}",
        status_line(&answer)
    );
    json_body(&answer)
}

/// Sends one request and reports *nothing at all* as `None`, rather than panicking on
/// the write or the read: for the framing and admit-ceiling refusals, "the connection
/// was closed with no status line" is itself a passing answer (§Concurrency, #11).
fn attempt(server: &Registry, raw: &[u8]) -> Option<Vec<u8>> {
    let Ok(mut stream) = server.try_connection() else {
        return None;
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .expect("setting the read timeout failed");
    if stream.write_all(raw).is_err() {
        return None;
    }
    let _ = stream.flush();
    let answer = read_to_end(&mut stream);
    (!answer.is_empty()).then_some(answer)
}

/// The two shapes a budget refusal legitimately takes: a `4xx`/`5xx` status line, or
/// the connection being closed with no status line (`#11`'s "closed at accept with no
/// status line"). A `2xx` is the only answer that can never be right.
fn refused(answer: Option<Vec<u8>>, label: &str) {
    match answer {
        None => {}
        Some(raw) => {
            let code = status_code(&raw);
            assert!(
                !matches!(code, Some(200..=299)),
                "`{label}` is never answered with a success: {}",
                status_line(&raw)
            );
            assert!(
                matches!(code, Some(400..=599)) || status_line(&raw).is_empty(),
                "`{label}` is refused by status or by a closed connection, not answered \
                 some other way: {:?}",
                String::from_utf8_lossy(&raw)
            );
        }
    }
}

/// proves: `026-C6`'s write seam as §Concurrency applies it to a second writer (Test
/// strategy #11, first clause) — the fixed tmp name is opened `create_new(true)`, so a
/// stale tmp entry (what a crashed or second writer leaves behind) is a hard failure
/// for the *next* mutation: the mutation is refused loudly and nothing is written
/// through it. A save that silently overwrote the stale tmp is the failure this pins.
#[test]
fn a_stale_tmp_makes_the_next_mutation_fail_loudly_and_writes_through_nothing() {
    let (data, corpus, server) = root("conc-stale-tmp");
    // A7's persist discipline names the tmp after the document it is renaming into
    // place — `apps/discovery-cli/tests/cli_match_store_tests.rs` pins exactly
    // `matches.json.tmp` for the copy this design re-uses.
    let stale_tmp = data.join("corpus.json.tmp");
    std::fs::write(&stale_tmp, "a save that never finished its rename").expect("plant the tmp");
    let before = std::fs::read(&corpus).expect("the corpus is readable");
    let before_counts = stats(&server);

    let answer = attempt(
        &server,
        &build_request(
            "POST",
            "/v1/admin/games",
            &server.host_header(),
            &[],
            &record_body("blocked", "Blocked Game", &["steam/900500"]),
        ),
    );
    match answer {
        None => {
            // Dying over a refused persist is loud. Silently answering `2xx` is not.
        }
        Some(raw) => refused(Some(raw), "a mutation over a stale tmp"),
    }

    assert_eq!(
        std::fs::read(&corpus).expect("the corpus survives"),
        before,
        "a refused persist never writes through the stale tmp into the document"
    );
    assert_eq!(
        std::fs::read(&stale_tmp).expect("the stale tmp is still there"),
        b"a save that never finished its rename".to_vec(),
        "the stale tmp is not adopted, replaced, or renamed into place (A7)"
    );
    assert_eq!(
        stats(&server),
        before_counts,
        "the model and `revision` are untouched by a refused persist"
    );
}

/// proves: Test strategy #11's "a reader never sees a torn document (served from the
/// locked in-memory model)" — with mutations landing under the write lock while
/// readers hammer the shared lock, every lookup answer is a complete, parseable
/// candidate array in one of the two states the document can actually be in: never a
/// truncated body, never a half-applied record set.
#[test]
fn a_reader_never_sees_a_torn_document_while_mutations_land() {
    let (_data, _corpus, server) = root("conc-torn");
    // The subprocess handle is `!Clone` and every request method takes `&self`,
    // so the racing threads share one handle through an `Arc`.
    let server = std::sync::Arc::new(server);

    std::thread::scope(|scope| {
        let writers = (0..4)
            .map(|index| {
                let server = std::sync::Arc::clone(&server);
                scope.spawn(move || {
                    let answer = server.request(
                        "POST",
                        "/v1/admin/games",
                        &[],
                        &record_body(
                            &format!("racer-{index}"),
                            &format!("Racer {index}"),
                            &[&format!("steam/95010{index}")],
                        ),
                    );
                    assert!(
                        matches!(status_code(&answer), Some(200..=299)),
                        "mutation {index} is accepted: {}",
                        status_line(&answer)
                    );
                })
            })
            .collect::<Vec<_>>();

        let readers = (0..6)
            .map(|_| {
                let server = std::sync::Arc::clone(&server);
                scope.spawn(move || {
                    let mut torn = Vec::new();
                    for round in 0..25 {
                        for index in 0..4 {
                            let target = format!("/v1/candidates?slug=steam%2F95010{index}");
                            let answer = server.get(&target);
                            let expected = serde_json::json!({
                                "game_id": format!("racer-{index}"),
                                "name": format!("Racer {index}"),
                            });
                            let body = body_text(&answer);
                            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body);
                            match parsed {
                                // The only two complete states this lookup can have.
                                Ok(value)
                                    if value == serde_json::json!([])
                                        || value == serde_json::json!([expected]) => {}
                                Ok(value) => torn.push(format!(
                                    "round {round}, slug {index}: an answer that is neither \
                                     the empty array nor the one complete record: {value}"
                                )),
                                Err(err) => torn.push(format!(
                                    "round {round}, slug {index}: an unparseable body ({err}): \
                                     {body:?}"
                                )),
                            }
                        }
                    }
                    torn
                })
            })
            .collect::<Vec<_>>();

        for writer in writers {
            writer.join().expect("a mutation thread panicked");
        }
        for reader in readers {
            let torn = reader.join().expect("a lookup thread panicked");
            assert!(torn.is_empty(), "torn reads observed: {torn:?}");
        }
    });
}

/// proves: §Concurrency's central claim (Test strategy #11) — "a mutation serializes
/// the new model, writes the fixed tmp name, renames it into place, and swaps the
/// model in, **all under the one write lock**, so persist order can never diverge
/// from mutation order": 12 concurrent upserts all apply, the persisted document
/// equals the model exactly, and `revision` sits at the final `5 + 12`, not at some
/// lost-update value below it.
#[test]
fn concurrent_upserts_leave_the_document_equal_to_the_model_at_the_final_revision() {
    let (data, corpus, server) = root("conc-upserts");
    // Distinct ids *and* distinct source keys, so a lost write shows up as a missing
    // row rather than as an ambiguous posting in the slug index.
    let records: Vec<(String, String)> = (0..12)
        .map(|index| (format!("concurrent-{index}"), format!("steam/96000{index}")))
        .collect();

    let answers = std::thread::scope(|scope| {
        let joining = records
            .iter()
            .map(|(game_id, source_key)| {
                scope.spawn(|| {
                    let (game_id, source_key) = (game_id.clone(), source_key.clone());
                    let answer = server.request(
                        "POST",
                        "/v1/admin/games",
                        &[],
                        &record_body(&game_id, "Concurrent Game", &[source_key.as_str()]),
                    );
                    (game_id, status_code(&answer))
                })
            })
            .collect::<Vec<_>>();
        joining
            .into_iter()
            .map(|handle| handle.join().expect("an upsert thread panicked"))
            .collect::<Vec<_>>()
    });

    for (game_id, code) in answers {
        assert!(
            matches!(code, Some(200..=299)),
            "upserting `{game_id}` under contention is accepted, not lost: {code:?}"
        );
    }

    let mut expected = vec!["example-game".to_string()];
    expected.extend(records.iter().map(|(game_id, _)| game_id.clone()));
    expected.sort();

    let document = document_of(&corpus);
    assert_eq!(
        sorted_ids(&document),
        expected,
        "every concurrent upsert is in the persisted document — none was overwritten"
    );
    assert_eq!(
        document["revision"].as_u64(),
        Some(5 + records.len() as u64),
        "`revision` sits at the final value: every accepted mutation bumped it exactly once"
    );
    let counts = stats(&server);
    assert_eq!(
        counts["games"].as_u64(),
        Some(1 + records.len() as u64),
        "stats reads the same model the file holds"
    );
    assert_eq!(
        counts["revision"], document["revision"],
        "the model and the persisted document report the same `revision`"
    );
    assert_eq!(
        data.files_below("."),
        vec![corpus.clone()],
        "twelve serialized saves leave no tmp behind"
    );
}

/// proves: Test strategy #11's atomic-persist clause as §Concurrency orders it, and
/// the round-1 correctness finding (accepted) that `persist::save` now documents:
/// `write_all` → `sync_all` → rename, all under the one write lock, before the
/// mutation is answered. The observable half from the subprocess seam is that a
/// mutation's `2xx` and the document on disk are never separated — after *each*
/// accepted upsert returns, the renamed document already holds the record that
/// answer spoke for, at the bumped `revision`, with no tmp beside it. A refactor
/// that answers before the document is durable (or drops the `sync_all`-then-rename
/// ordering this documents) is what this leg stops from silently surviving.
#[test]
fn every_accepted_mutation_is_answered_only_after_the_document_holds_it() {
    let (data, corpus, server) = root("conc-durable-answer");

    for index in 0..5_u32 {
        let game_id = format!("durable-{index}");
        let answer = server.request(
            "POST",
            "/v1/admin/games",
            &[],
            &record_body(&game_id, "Durable Game", &[&format!("steam/97000{index}")]),
        );
        assert!(
            matches!(status_code(&answer), Some(200..=299)),
            "upsert {index} is accepted: {}",
            status_line(&answer)
        );

        // The answer has returned; the rename that carried this record is already
        // in place — not a pending tmp, not a torn file, not the previous revision.
        let document = document_of(&corpus);
        assert!(
            sorted_ids(&document).contains(&game_id),
            "upsert {index} answered `2xx` but the renamed document does not hold \
             the answered record: {document:?}"
        );
        assert_eq!(
            document["revision"].as_u64(),
            Some(5 + u64::from(index) + 1),
            "the answered document carries exactly the revisions answered so far"
        );
        assert_eq!(
            data.files_below("."),
            vec![corpus.clone()],
            "upsert {index}: the rename consumed its tmp"
        );
    }

    let counts = stats(&server);
    assert_eq!(
        counts["revision"].as_u64(),
        Some(10),
        "the model sits at the last revision every answer already spoke for"
    );
}

/// proves: Test strategy #11's request-side clause — a `Transfer-Encoding`, a
/// `Content-Length`-less, a non-decimal-length, or an over-`MAX_REQUEST_BODY_OCTETS`
/// body is "refused before any mutation", and §Budgets' header side: an
/// over-`MAX_REQUEST_HEADER_OCTETS` header block is refused the same way. Each row
/// asserts the document and the model byte-for-byte unchanged, so no refusal can have
/// been half-applied.
#[test]
fn every_request_budget_is_refused_before_any_mutation() {
    let body = record_body("never-lands", "Never Lands", &["steam/900600"]);
    let oversized_header = format!("X-Pad: {}", "p".repeat(MAX_REQUEST_HEADER_OCTETS + 1_000));

    let cases: Vec<(&str, Vec<u8>)> = vec![
        (
            "transfer-encoding",
            build_request(
                "POST",
                "/v1/admin/games",
                "127.0.0.1:0",
                &["Transfer-Encoding: chunked"],
                "0\r\n\r\n",
            ),
        ),
        (
            "no-content-length",
            build_request_without_length("POST", "/v1/admin/games", "127.0.0.1:0", &body),
        ),
        (
            "non-decimal-content-length",
            build_request_with_length_claim(
                "POST",
                "/v1/admin/games",
                "127.0.0.1:0",
                "twelve",
                &body,
            ),
        ),
        (
            "over-cap-content-length",
            build_request_with_length_claim(
                "POST",
                "/v1/admin/games",
                "127.0.0.1:0",
                &format!("{}", MAX_REQUEST_BODY_OCTETS + 1),
                &body,
            ),
        ),
        (
            "over-cap-header-block",
            build_request(
                "POST",
                "/v1/admin/games",
                "127.0.0.1:0",
                &[oversized_header.as_str()],
                &body,
            ),
        ),
    ];

    for (label, raw) in cases {
        let (data, corpus, server) = root(&format!("conc-budget-{label}"));
        let before = std::fs::read(&corpus).expect("the corpus is readable");
        let before_counts = stats(&server);

        // The `Host` line the builders were handed is wrong on purpose (the port is
        // ephemeral); this design's serving face does not route on it, so the framing
        // is the only variable in the row.
        refused(attempt(&server, &raw), label);

        assert_eq!(
            std::fs::read(&corpus).expect("the corpus survives"),
            before,
            "`{label}` is refused before anything is written to the document"
        );
        assert_eq!(
            stats(&server),
            before_counts,
            "`{label}` is refused before the model or `revision` moved"
        );
        assert_eq!(
            data.files_below("."),
            vec![corpus.clone()],
            "`{label}` wrote no tmp either — the refusal precedes the persist"
        );
    }
}

/// proves: §Budgets' admit ceiling (Test strategy #11's last clause) — "over-cap is
/// closed at accept with no status line": with `MAX_CONCURRENT_CONNECTIONS` threads
/// already admitted (and one thread per admitted connection, so an idle connection
/// holds its slot), the next connection gets no HTTP answer at all, and releasing a
/// connection lets a request through again.
#[test]
fn connections_past_the_admit_ceiling_are_closed_with_no_status_line() {
    let (_data, _corpus, server) = root("conc-connections");
    let held = server.hold_connections(MAX_CONCURRENT_CONNECTIONS);

    let raw = build_request("GET", "/v1/health", &server.host_header(), &[], "");
    let answer = attempt(&server, &raw);
    match answer {
        None => {}
        Some(answer) => assert!(
            status_code(&answer).is_none(),
            "a connection over the ceiling is closed at accept, so it never sees a status \
             line: {}",
            status_line(&answer)
        ),
    }

    drop(held);
    let health = server.health();
    assert_eq!(
        status_code(&health),
        Some(200),
        "the ceiling is a bound on concurrent connections, not a poisoned listener: {}",
        status_line(&health)
    );
}
