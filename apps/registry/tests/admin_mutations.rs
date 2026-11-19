//! The admin mutations: `POST /v1/admin/games` upserts a whole record, `DELETE
//! /v1/admin/games/<game_id>` removes one, `GET /v1/admin/stats` answers three
//! integer counts — every accepted one bumps `revision` and lands atomically in
//! that instance's own catalog document, every refused one touches nothing.
//!
//! proves: design 026's Test strategy #10 — "Admin mutations (`026-C4`): upsert
//! validates + bumps `revision` + atomic rename, and an upsert whose serialized
//! corpus with its bumped `revision` would exceed `MAX_CATALOG_DOCUMENT_OCTETS` is
//! that named `422` with model, `revision` and file untouched and no tmp written;
//! delete decodes + validates + bumps `revision` + atomic rename; `405` on a method
//! a route lacks; `stats` is the three-field integer shape `{games, source_keys,
//! revision}` from the instance document" (with the r4 revision-log note that the
//! cap check rides the post-`revision`-bump buffer that is renamed).
//!
//! Loopback binds, so no credential is involved (the auth rule is `admin_auth.rs`).
//! Every document lives in a temp directory, and every refusal asserts the file it
//! refused came back byte-for-byte with no tmp beside it.

mod common;

use common::{
    Registry, TempDir, body_text, cache_doc, corpus_doc, json_body, record_body, registry_document,
    status_code, status_line,
};

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

fn document_of(path: &std::path::Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path).expect("the catalog document is readable");
    serde_json::from_str(&text).expect("the persisted catalog document is JSON")
}

fn game_ids_in(document: &serde_json::Value) -> Vec<String> {
    document["games"]
        .as_array()
        .expect("games")
        .iter()
        .map(|game| game["game_id"].as_str().expect("a game_id").to_string())
        .collect()
}

/// proves: `026-C4` (Test strategy #10, first clause) — an upsert that passes the
/// `026-C3` checks is persisted whole, with `revision` bumped, through the
/// tmp-and-rename discipline: the directory holds the document and nothing else.
#[test]
fn an_upsert_validates_bumps_revision_and_persists_atomically() {
    let (data, corpus, server) = root("mut-upsert");

    let answer = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("new-game", "New Game", &["steam/900002", "epic/42"]),
    );
    assert!(
        matches!(status_code(&answer), Some(200..=299)),
        "a valid upsert is accepted: {}",
        status_line(&answer)
    );

    let document = document_of(&corpus);
    assert_eq!(
        document["revision"].as_u64(),
        Some(6),
        "every accepted mutation bumps `revision` (`026-C4`)"
    );
    assert_eq!(
        game_ids_in(&document),
        vec!["example-game".to_string(), "new-game".to_string()],
        "the record lands whole in that instance's own document"
    );
    let added = document["games"][1].as_object().expect("a record object");
    assert_eq!(added["name"].as_str(), Some("New Game"));
    assert_eq!(
        added["source_keys"].as_array().expect("source_keys").len(),
        2,
        "an upsert carries the whole record, keys included"
    );

    let counts = stats(&server);
    assert_eq!(counts["games"].as_u64(), Some(2));
    assert_eq!(counts["source_keys"].as_u64(), Some(3));
    assert_eq!(counts["revision"].as_u64(), Some(6));

    assert_eq!(
        data.files_below("."),
        vec![corpus.clone()],
        "the tmp-and-rename discipline leaves no tmp behind (A7)"
    );

    // And the join sees the new key immediately, from the in-memory model.
    let answer = server.get("/v1/candidates?slug=steam%2F900002");
    assert_eq!(common::game_ids(&answer), vec!["new-game".to_string()]);
}

/// proves: `026-C4`'s "every mutation validates against `026-C3` and refuses by
/// name" (Test strategy #10) — a record the catalog contract refuses is a
/// well-framed refusal naming the offending field, and nothing on disk moves.
#[test]
fn an_upsert_the_catalog_contract_refuses_names_the_condition_and_touches_nothing() {
    let long_name = "n".repeat(129);
    let long_id = "a".repeat(129);
    let cases: Vec<(&str, String)> = vec![
        (
            "uppercase-game-id",
            record_body("Example-Game", "X", &["steam/1"]),
        ),
        (
            "dotted-game-id",
            record_body("example.game", "X", &["steam/1"]),
        ),
        (
            "over-long-game-id",
            record_body(&long_id, "X", &["steam/1"]),
        ),
        (
            "over-long-name",
            record_body("ok-id", &long_name, &["steam/1"]),
        ),
        ("unknown-source-key", record_body("ok-id", "X", &["gog/1"])),
        (
            "unspaced-source-key",
            record_body("ok-id", "X", &["steam-1"]),
        ),
        ("empty-external-id", record_body("ok-id", "X", &["steam/"])),
        (
            "duplicate-source-key-in-record",
            record_body("ok-id", "X", &["steam/1", "steam/1"]),
        ),
        (
            "missing-name",
            "{\"game_id\":\"ok-id\",\"source_keys\":[]}".to_string(),
        ),
        ("not-an-object", "\"just a string\"".to_string()),
        ("not-json", "this is not a record".to_string()),
    ];

    for (label, body) in cases {
        let (data, corpus, server) = root(&format!("mut-refuse-{label}"));
        let before = std::fs::read(&corpus).expect("the corpus is readable");
        let before_counts = stats(&server);

        let answer = server.request("POST", "/v1/admin/games", &[], &body);
        let code = status_code(&answer);
        assert!(
            matches!(code, Some(400..=499)),
            "`{label}` is refused with a `4xx`, never accepted: {}",
            status_line(&answer)
        );
        assert_eq!(
            std::fs::read(&corpus).expect("the corpus survives"),
            before,
            "`{label}` writes nothing to the document"
        );
        assert_eq!(
            stats(&server),
            before_counts,
            "`{label}` leaves the model and `revision` untouched"
        );
        assert_eq!(
            data.files_below("."),
            vec![corpus.clone()],
            "`{label}` writes no tmp either (Chosen design: The admin face and auth)"
        );
    }
}

/// proves: the `Content-Type` half of the round-1 security finding (the accepted
/// browser-forgery closure recorded in the build notes' *Deviations → Admin face
/// and auth*) — an upsert whose body is *declared* as a media type this face never
/// parses (`text/plain`, `application/x-www-form-urlencoded`,
/// `multipart/form-data` — exactly what an HTML form forges — or
/// `application/xml`) is the named `415` whose body carries
/// `error: unsupported_media_type`, decided on the header block: the corpus is
/// byte-identical afterwards, the model and `revision` do not move, and no tmp is
/// written.
#[test]
fn an_upsert_declared_as_a_non_json_media_type_is_415_and_writes_nothing() {
    let (data, corpus, server) = root("mut-media-type");
    let before = std::fs::read(&corpus).expect("the corpus is readable");
    let before_counts = stats(&server);
    let body = record_body("form-forged", "Form Forged", &["steam/900400"]);

    for declared in [
        "text/plain",
        "application/x-www-form-urlencoded",
        "multipart/form-data",
        "application/xml",
    ] {
        let answer = server.request(
            "POST",
            "/v1/admin/games",
            &[&format!("Content-Type: {declared}")],
            &body,
        );
        assert_eq!(
            status_code(&answer),
            Some(415),
            "an upsert declared `{declared}` is the named `415`: {}",
            status_line(&answer)
        );
        assert!(
            body_text(&answer).contains("unsupported_media_type"),
            "the refusal names the condition: {:?}",
            body_text(&answer)
        );
        assert_eq!(
            std::fs::read(&corpus).expect("the corpus survives"),
            before,
            "`{declared}` refused on the head — the document never moved"
        );
        assert_eq!(
            stats(&server),
            before_counts,
            "`{declared}` left the model and `revision` untouched"
        );
        assert_eq!(
            data.files_below("."),
            vec![corpus.clone()],
            "`{declared}` wrote no tmp either"
        );
    }
}

/// proves: the admitted spelling of the same rule, pinned at the subprocess seam —
/// an upsert declared `Content-Type: application/json` lands, and the parameters
/// after the `;` are ignored (`application/json; charset=utf-8` lands too). The
/// rule guards only the one route that parses a body: `stats` and a `DELETE` that
/// carry a `Content-Type` are not asked about and answer as they always did.
#[test]
fn an_upsert_declared_as_json_lands_and_other_routes_ignore_the_header() {
    let (_data, corpus, server) = root("mut-media-type-ok");

    let plain = server.request(
        "POST",
        "/v1/admin/games",
        &["Content-Type: application/json"],
        &record_body("plain-json", "Plain Json", &["steam/900401"]),
    );
    assert!(
        matches!(status_code(&plain), Some(200..=299)),
        "an upsert declared `application/json` is accepted: {}",
        status_line(&plain)
    );
    assert!(game_ids_in(&document_of(&corpus)).contains(&"plain-json".to_string()));

    let parameterised = server.request(
        "POST",
        "/v1/admin/games",
        &["Content-Type: application/json; charset=utf-8"],
        &record_body("json-charset", "Json Charset", &["steam/900402"]),
    );
    assert!(
        matches!(status_code(&parameterised), Some(200..=299)),
        "parameters after the `;` are ignored: {}",
        status_line(&parameterised)
    );
    assert!(game_ids_in(&document_of(&corpus)).contains(&"json-charset".to_string()));

    let stats_answer = server.request(
        "GET",
        "/v1/admin/stats",
        &["Content-Type: application/json"],
        "",
    );
    assert_eq!(
        status_code(&stats_answer),
        Some(200),
        "a `Content-Type` on a route that parses no body is not asked about: {}",
        status_line(&stats_answer)
    );
    let delete_answer = server.request(
        "DELETE",
        "/v1/admin/games/json-charset",
        &["Content-Type: application/json"],
        "",
    );
    assert!(
        matches!(status_code(&delete_answer), Some(200..=299)),
        "nor is it asked about on the delete route: {}",
        status_line(&delete_answer)
    );
}

/// proves: Test strategy #10's cap clause (revision-log r4) — an upsert whose
/// serialized corpus *with its bumped `revision`* would exceed
/// `MAX_CATALOG_DOCUMENT_OCTETS` is the named `422` (A21: well-formed but
/// unusable), with model, `revision`, and file untouched and no tmp written, so no
/// accepted mutation can persist a corpus its own loader refuses.
#[test]
fn an_upsert_past_the_catalog_document_cap_is_422_with_nothing_touched() {
    const CAP: usize = 16_777_216;
    let data = TempDir::new("mut-cap");

    // A corpus that is loadable and sits just under the cap.
    let mut games: Vec<serde_json::Value> = Vec::new();
    let mut size = 0usize;
    loop {
        let index = games.len();
        let game_id = format!("game-{index:06}");
        let source_key = format!("steam/{index}000000");
        let record = serde_json::json!({
            "game_id": game_id,
            "name": "A padded game",
            "source_keys": [source_key],
        });
        let extra = serde_json::to_string(&record).expect("serialises").len() + 1;
        if size + extra > CAP - 40 {
            break;
        }
        size += extra;
        games.push(record);
        assert!(games.len() < 400_000, "the fixture generator terminates");
    }
    let revision = 41;
    let document = serde_json::to_string(&serde_json::json!({
        "version": 1,
        "revision": revision,
        "games": games,
    }))
    .expect("the padded corpus serialises");
    assert!(
        document.len() < CAP,
        "the fixture corpus is loadable: {} octets",
        document.len()
    );
    let straw = record_body(
        "the-last-straw",
        &"n".repeat(128),
        &["steam/7777777", "epic/7777777", "local/7777777"],
    );
    assert!(
        document.len() + straw.len() > CAP,
        "the fixture leaves less room than the record needs: {} + {}",
        document.len(),
        straw.len()
    );
    let corpus = data.write("corpus.json", document.as_bytes());
    let before = std::fs::read(&corpus).expect("the corpus is readable");
    let server = Registry::root("mut-cap", &corpus, &[]);
    let before_counts = stats(&server);

    let answer = server.request("POST", "/v1/admin/games", &[], &straw);
    assert_eq!(
        status_code(&answer),
        Some(422),
        "an upsert whose serialized corpus with its bumped `revision` would breach \
         `MAX_CATALOG_DOCUMENT_OCTETS` is the named `422`: {}",
        status_line(&answer)
    );
    assert_eq!(
        std::fs::read(&corpus).expect("the corpus survives"),
        before,
        "the file is untouched — the over-cap buffer is never renamed into place"
    );
    assert_eq!(
        stats(&server),
        before_counts,
        "the in-memory model and `revision` are untouched too"
    );
    assert_eq!(
        data.files_below("."),
        vec![corpus.clone()],
        "and no tmp was written at all"
    );
}

/// proves: `026-C4` as design 026 pins it for `DELETE` (revision-log: "delete now
/// states its `revision` bump and tmp-and-rename") — the `<game_id>` path segment
/// is percent-decoded, the record is removed, `revision` bumps, and the document is
/// persisted through the same rename.
#[test]
fn a_delete_removes_the_record_bumps_revision_and_persists_atomically() {
    let (data, corpus, server) = root("mut-delete");
    let upsert = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("to-remove", "To Remove", &["steam/900003"]),
    );
    assert!(matches!(status_code(&upsert), Some(200..=299)));
    let after_upsert = document_of(&corpus);

    let answer = server.request("DELETE", "/v1/admin/games/to-remove", &[], "");
    assert!(
        matches!(status_code(&answer), Some(200..=299)),
        "the delete is accepted: {}",
        status_line(&answer)
    );

    let document = document_of(&corpus);
    assert!(!game_ids_in(&document).contains(&"to-remove".to_string()));
    assert_eq!(
        document["revision"].as_u64(),
        Some(after_upsert["revision"].as_u64().expect("a revision") + 1),
        "a delete bumps `revision` like every other mutation"
    );
    assert_eq!(
        data.files_below("."),
        vec![corpus.clone()],
        "no tmp survives the delete's rename"
    );
    let counts = stats(&server);
    assert_eq!(
        counts["games"].as_u64(),
        Some(1),
        "stats agrees with the persisted document: {counts:?}"
    );
    assert_eq!(counts["revision"], document["revision"]);

    let answer = server.get("/v1/candidates?slug=steam%2F900003");
    assert_eq!(
        common::game_ids(&answer),
        Vec::<String>::new(),
        "the removed record stops answering"
    );
}

/// proves: Test strategy #10's `stats` clause and `026-L3` — `{games,
/// source_keys, revision}` are three integer counts from the instance's own
/// catalog document, never key lists.
#[test]
fn stats_is_three_integer_counts_from_the_instance_document() {
    let (_data, _corpus, server) = root("mut-stats");

    let counts = stats(&server);
    let object = counts.as_object().expect("stats is an object");
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys,
        vec!["games", "revision", "source_keys"],
        "`026-C4` fixes the stats body exactly"
    );
    assert_eq!(counts["games"].as_u64(), Some(1));
    assert_eq!(counts["source_keys"].as_u64(), Some(1));
    assert_eq!(counts["revision"].as_u64(), Some(5));

    server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("second", "Second Game", &["epic/9", "local/Some.Dir"]),
    );
    let counts = stats(&server);
    assert_eq!(counts["games"].as_u64(), Some(2));
    assert_eq!(
        counts["source_keys"].as_u64(),
        Some(3),
        "the count is the sum of the records' keys, not a list of them"
    );
    assert!(
        !body_text(&server.get("/v1/admin/stats")).contains("steam/900001"),
        "no key list ever appears in stats (`026-L3`)"
    );
}

/// proves: `026-C4`'s "mutating that instance's own catalog document" for caching
/// mode (Chosen design: CLI surface — the local-entries document defaults to
/// `<data>/bindforge/registry/catalog-v1.json`, "a `026-C3` instance, created empty
/// if absent, mutated by the admin face").
#[test]
fn in_caching_mode_the_admin_face_mutates_the_local_entries_document() {
    let data = TempDir::new("mut-caching");
    let local_entries = registry_document(data.path(), "catalog-v1.json");
    let upstream = common::unreachable_upstream();
    let server = Registry::caching("mut-caching", &upstream, &data, &[]);

    assert_eq!(
        json_body(&server.get("/v1/admin/stats"))["games"].as_u64(),
        Some(0),
        "an absent local-entries document is an empty catalog, not an error"
    );

    let answer = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("mine", "Mine", &["steam/900010"]),
    );
    assert!(
        matches!(status_code(&answer), Some(200..=299)),
        "the caching instance's admin face works: {}",
        status_line(&answer)
    );
    assert!(
        local_entries.is_file(),
        "the local-entries document lives at `<data>/bindforge/registry/catalog-v1.json`"
    );
    let document = document_of(&local_entries);
    assert_eq!(game_ids_in(&document), vec!["mine".to_string()]);
    assert_eq!(
        document["version"].as_i64(),
        Some(1),
        "it is a `026-C3` instance"
    );
    assert_eq!(
        json_body(&server.get("/v1/admin/stats"))["revision"].as_u64(),
        document["revision"].as_u64(),
        "stats reads the same document the admin face wrote"
    );
    assert_eq!(
        json_body(&server.health())["local_entries"].as_u64(),
        Some(1),
        "health's `local_entries` counts it (`026-C2`)"
    );
}

/// proves: Test strategy #10's "from the instance document" qualifier, in the mode
/// where the two documents differ — `stats` counts local entries, never cache
/// entries (Chosen design: "`GET /v1/admin/stats` … from the instance's own catalog
/// document (not the cache)").
#[test]
fn stats_counts_the_local_entries_document_and_never_the_cache() {
    let data = TempDir::new("mut-stats-cache");
    let now = common::now_unix();
    let seeded = cache_doc(&[
        (
            "steam/800001",
            &[("cached-one", "Cached One")][..],
            now - 10,
        ),
        (
            "steam/800002",
            &[("cached-two", "Cached Two")][..],
            now - 10,
        ),
    ]);
    let cache_path = registry_document(data.path(), "cache-v1.json");
    std::fs::create_dir_all(cache_path.parent().expect("the registry directory"))
        .expect("the registry directory is creatable");
    std::fs::write(&cache_path, seeded.as_bytes()).expect("the seeded cache lands");

    let upstream = common::unreachable_upstream();
    let server = Registry::caching("mut-stats-cache", &upstream, &data, &[]);

    let counts = json_body(&server.get("/v1/admin/stats"));
    assert_eq!(
        counts["games"].as_u64(),
        Some(0),
        "cached lookups are not curated entries: {counts:?}"
    );
    assert_eq!(
        json_body(&server.health())["cached_slugs"].as_u64(),
        Some(2),
        "the cache is counted where `026-C2` says it is"
    );
}

/// proves: `026-C4`'s refusal discipline on the delete route — a `<game_id>` the
/// instance does not hold is refused and bumps nothing.
#[test]
fn deleting_a_record_that_is_not_there_bumps_nothing() {
    let (data, corpus, server) = root("mut-delete-absent");
    let before = std::fs::read(&corpus).expect("the corpus is readable");
    let before_counts = stats(&server);

    let answer = server.request("DELETE", "/v1/admin/games/not-here", &[], "");
    assert!(
        matches!(status_code(&answer), Some(400..=499)),
        "a delete of an absent record is refused, not silently accepted: {}",
        status_line(&answer)
    );
    assert_eq!(
        std::fs::read(&corpus).expect("the corpus survives"),
        before,
        "a refused delete writes nothing"
    );
    assert_eq!(stats(&server), before_counts, "and bumps no `revision`");
    assert_eq!(data.files_below("."), vec![corpus.clone()]);
}

/// proves: `026-C4`'s persist promise, seen from the other side of a restart — an
/// accepted mutation is on disk before the answer, so a restart cannot lose it
/// (Chosen design: Concurrency, "a restart cannot lose an accepted mutation").
#[test]
fn an_accepted_mutation_survives_a_restart() {
    let (_data, corpus, server) = root("mut-restart");
    let answer = server.request(
        "POST",
        "/v1/admin/games",
        &[],
        &record_body("kept", "Kept Game", &["steam/900011"]),
    );
    assert!(matches!(status_code(&answer), Some(200..=299)));
    drop(server);

    let restarted = Registry::root("mut-restart-2", &corpus, &[]);
    let document = document_of(&corpus);
    assert!(game_ids_in(&document).contains(&"kept".to_string()));
    assert_eq!(
        json_body(&restarted.get("/v1/candidates?slug=steam%2F900011")).as_array(),
        Some(&vec![
            serde_json::json!({"game_id": "kept", "name": "Kept Game"})
        ]),
        "the record answers exactly as it was written"
    );
    assert_eq!(
        json_body(&restarted.get("/v1/admin/stats"))["revision"],
        document["revision"],
        "stats after the restart reads the persisted revision, not a fresh counter"
    );
}
