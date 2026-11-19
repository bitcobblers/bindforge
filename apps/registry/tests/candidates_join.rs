//! The corpus join: `GET /v1/candidates?slug=…` answers the games whose
//! `source_keys` hold the queried key, ordered, projected, and truncated — and
//! answers it identically in both modes.
//!
//! proves: design 026's Test strategy #2 — "Corpus join: `candidates(slug)`
//! probes the load-built index and returns the games whose `source_keys` contain
//! the decoded slug, postings already ordered by `game_id`, projected to
//! `{game_id, name}` only, truncated to `MAX_CANDIDATES` (A1); a root-mode empty
//! join answers `200` with `[]`" — read together with `026-C1` (the serving face
//! is byte-identical between modes), `026-C2` (the response shape) and `026-L6`
//! (only the queried key and `{game_id, name}` answers ever leave).
//!
//! Integration over a loopback socket: every corpus is a temp file, every
//! instance binds an ephemeral `127.0.0.1` port.

mod common;

use common::{
    Registry, TempDir, body_of, body_text, corpus_doc, game_ids, header_names, header_values,
    json_body, slug_query, status_code, status_line,
};

/// A1: the shipped client caps the response body it will read at
/// `MAX_RESPONSE_OCTETS = 65_536`; the answer seam must never emit more.
const MAX_RESPONSE_OCTETS: usize = 65_536;

/// A corpus with one key claimed by several games, in deliberately mixed id order.
fn shared_key_corpus() -> Vec<(&'static str, &'static str, &'static [&'static str])> {
    vec![
        ("alphabet", "Alphabet", &["steam/900100"][..]),
        ("alpha", "Alpha", &["steam/900100"][..]),
        ("alpha-x", "Alpha X", &["steam/900100"][..]),
        ("alpha-2", "Alpha Two", &["steam/900100"][..]),
        ("alone", "Alone", &["steam/900200"][..]),
        ("epic-only", "Epic Only", &["epic/55"][..]),
        ("local-only", "Local Only", &["local/Steam.SE"][..]),
    ]
}

/// proves: Test strategy #2 (the join itself) — the answer is exactly the games
/// carrying the queried key, nothing else from the corpus, whatever the key's
/// source token is.
#[test]
fn candidates_answers_the_games_carrying_the_queried_key() {
    let data = TempDir::new("join-keys");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(7, &shared_key_corpus()).as_bytes(),
    );
    let server = Registry::root("join-keys", &corpus, &[]);

    for (key, expected) in [
        (
            "steam/900100",
            vec!["alpha", "alpha-2", "alpha-x", "alphabet"],
        ),
        ("steam/900200", vec!["alone"]),
        ("epic/55", vec!["epic-only"]),
        ("local/Steam.SE", vec!["local-only"]),
    ] {
        let answer = server.get(&slug_query(key));
        assert_eq!(status_code(&answer), Some(200), "`{key}` answers");
        assert_eq!(
            game_ids(&answer),
            expected,
            "the join for `{key}` is the postings for that key, in `game_id` order"
        );
    }
}

/// proves: Test strategy #2 (postings built already ordered by `game_id` byte
/// order at load, Chosen design: The serving face step 2) — byte order, not
/// case-folded or component-wise order, and stable across requests because the
/// order is fixed at load rather than computed per request.
#[test]
fn candidate_postings_are_ordered_by_game_id_byte_order() {
    let data = TempDir::new("join-order");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(0, &shared_key_corpus()).as_bytes(),
    );
    let server = Registry::root("join-order", &corpus, &[]);

    let first = game_ids(&server.get(&slug_query("steam/900100")));
    let second = game_ids(&server.get(&slug_query("steam/900100")));
    assert_eq!(
        first,
        vec!["alpha", "alpha-2", "alpha-x", "alphabet"],
        "postings are in `game_id` byte order: the bare prefix first, then `-` (0x2D) \
         before a letter"
    );
    assert_eq!(first, second, "the order is the same on every request");
}

/// proves: Test strategy #2 / `026-L6` (the projection) — a candidate carries
/// `{game_id, name}` and nothing else: no `source_keys`, no other key a corpus
/// record holds.
#[test]
fn candidates_are_projected_to_game_id_and_name_only() {
    let data = TempDir::new("join-projection");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(0, &shared_key_corpus()).as_bytes(),
    );
    let server = Registry::root("join-projection", &corpus, &[]);

    let answer = server.get(&slug_query("steam/900100"));
    let body = body_text(&answer);
    assert!(
        !body.contains("source_key"),
        "`source_keys` never leave the process (`026-C2`'s projection, `026-L6`): {body}"
    );
    assert!(
        !body.contains("steam/900100"),
        "the corpus' join facts are not echoed back (`026-L6`): {body}"
    );
    let candidates = json_body(&answer)
        .as_array()
        .expect("a candidate answer is an array")
        .clone();
    assert!(!candidates.is_empty(), "the answer is not vacuously empty");
    for candidate in &candidates {
        let object = candidate.as_object().expect("a candidate object");
        let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["game_id", "name"],
            "every candidate is exactly `{{game_id, name}}`: {candidate}"
        );
    }
}

/// proves: Test strategy #2 (`MAX_CANDIDATES = 256`, A1) — a key claimed by more
/// games than the shipped client will accept is truncated to the cap, and the
/// truncation is the *stable* first 256 in `game_id` order, not an arbitrary
/// window (Open question 15 resolved: the answer is ordered).
#[test]
fn a_key_claimed_by_more_games_than_the_cap_is_truncated_to_256_in_order() {
    let games: Vec<(String, String, Vec<String>)> = (0..300_u32)
        .map(|index| {
            (
                format!("game-{index:04}"),
                format!("Game {index}"),
                vec!["steam/900900".to_string()],
            )
        })
        .collect();
    let records: Vec<serde_json::Value> = games
        .iter()
        .map(|(game_id, name, keys)| {
            serde_json::json!({"game_id": game_id, "name": name, "source_keys": keys})
        })
        .collect();
    let document = serde_json::to_string_pretty(&serde_json::json!({
        "version": 1,
        "revision": 0,
        "games": records,
    }))
    .expect("the wide corpus serialises");

    let data = TempDir::new("join-cap");
    let corpus = data.write("corpus.json", document.as_bytes());
    let server = Registry::root("join-cap", &corpus, &[]);

    let answer = server.get(&slug_query("steam/900900"));
    assert_eq!(status_code(&answer), Some(200), "a wide join still answers");
    let ids = game_ids(&answer);
    assert_eq!(
        ids.len(),
        256,
        "exactly `MAX_CANDIDATES` candidates are served, never the 300 the corpus holds"
    );
    let again = game_ids(&server.get(&slug_query("steam/900900")));
    assert_eq!(ids, again, "truncation is stable across requests");
    assert_eq!(
        ids.first().map(String::as_str),
        Some("game-0000"),
        "the window is the first 256 in `game_id` byte order"
    );
    assert_eq!(ids.last().map(String::as_str), Some("game-0255"));
}

/// proves: Test strategy #2's cap clause read together with A1's *octet* bound, and
/// the round-1 correctness/security findings that the answer seam must bound the
/// answer in octets as well as in count (build notes: "an octet-ceiling leg").
/// 256 postings at the `024-B1`/`013-B4` field bounds serialize to ≈72,000 octets —
/// over `MAX_RESPONSE_OCTETS` while still under `MAX_CANDIDATES` — so the octet cap,
/// not the count cap, must bind: the answer stays a `200`, its body is at most
/// 65,536 octets, it is the `game_id`-ordered stable prefix (never an arbitrary
/// window and never a socket-level truncation of a longer body), and a second
/// request answers the identical bytes.
#[test]
fn the_octet_cap_trims_a_wide_join_to_a_stable_prefix_under_the_response_budget() {
    // Every posting sits at both field bounds: a 128-octet `game_id` and a
    // 128-octet `name`, so each projected candidate is ≈280 octets and the full
    // 256-posting join is ≈72,000 — over the budget before any count cap applies.
    let records: Vec<serde_json::Value> = (0..256_u32)
        .map(|index| {
            // "g" + 4 digits + "-" is 6 octets; 122 padding octets make 128.
            let game_id = format!("g{index:04}-{}", "x".repeat(122));
            assert_eq!(game_id.len(), 128, "a grammar-valid id at `024-B1`'s bound");
            serde_json::json!({
                "game_id": game_id,
                "name": "n".repeat(128),
                "source_keys": ["steam/900950"],
            })
        })
        .collect();
    let document = serde_json::to_string_pretty(&serde_json::json!({
        "version": 1,
        "revision": 0,
        "games": records,
    }))
    .expect("the wide-field corpus serialises");

    let data = TempDir::new("join-octets");
    let corpus = data.write("corpus.json", document.as_bytes());
    let server = Registry::root("join-octets", &corpus, &[]);

    let answer = server.get(&slug_query("steam/900950"));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "a join over the octet budget is trimmed, never refused: {}",
        status_line(&answer)
    );

    let body = body_of(&answer);
    assert!(
        body.len() <= MAX_RESPONSE_OCTETS,
        "the answer body is at most `MAX_RESPONSE_OCTETS` — the shipped client (A1) \
         refuses anything larger: {} octets",
        body.len()
    );
    let lengths = header_values(&answer, "Content-Length");
    assert_eq!(
        lengths.len(),
        1,
        "exactly one `Content-Length`, whatever the trim: {lengths:?}"
    );
    let claimed: usize = lengths[0].parse().expect("a decimal Content-Length");
    assert_eq!(
        claimed,
        body.len(),
        "the promise names the bytes actually written — trimming happens before \
         serialization, not as a socket-level truncation"
    );

    let ids = game_ids(&answer);
    assert!(
        ids.len() < 256,
        "the octet cap binds below `MAX_CANDIDATES` at these field bounds, so the \
         served count is below the count cap: {}",
        ids.len()
    );
    let expected: Vec<String> = (0..256_u32)
        .map(|index| format!("g{index:04}-{}", "x".repeat(122)))
        .collect();
    assert_eq!(
        ids,
        expected[..ids.len()],
        "the served window is the first postings in `game_id` byte order — a stable \
         prefix, not an arbitrary selection"
    );

    let again = server.get(&slug_query("steam/900950"));
    assert_eq!(
        body_of(&again),
        body,
        "the same question answers the same bytes on every request"
    );
}

/// proves: Test strategy #2's last clause and `026-L4`'s boundary — an empty join
/// in root mode is a `200` with `[]` ("the catalog has never heard of this key");
/// the cold `503` belongs to caching mode only.
#[test]
fn an_empty_join_in_root_mode_is_200_with_an_empty_array() {
    let data = TempDir::new("join-empty");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(0, &shared_key_corpus()).as_bytes(),
    );
    let server = Registry::root("join-empty", &corpus, &[]);

    let answer = server.get(&slug_query("steam/999999"));
    assert_eq!(
        status_line(&answer),
        "HTTP/1.1 200 OK",
        "a root-mode empty join is a plain 200: {}",
        status_line(&answer)
    );
    assert_eq!(
        body_text(&answer),
        "[]",
        "the body is exactly an empty array"
    );
    assert_eq!(json_body(&answer).as_array().expect("[]").len(), 0);
}

/// proves: `026-C1` (named in Test strategy #2's byte-identical posture) — the
/// same answered question asked of a root and of a caching instance holding the
/// same records as local entries gets the same status line, the same body, and
/// the same header names: the client cannot tell which mode answered. The
/// absent slug is the one exception the design itself names ("the caching-only
/// cold `503`" — Chosen design: The serving face; `026-L4`): root answers the
/// empty join with `200 []`, and a caching instance with an unreachable
/// upstream answers the named `503` — never `[]`.
#[test]
fn the_serving_face_is_byte_identical_between_the_two_modes() {
    let root_home = TempDir::new("identical-root");
    let document = corpus_doc(0, &shared_key_corpus());
    let corpus = root_home.write("corpus.json", document.as_bytes());
    let root = Registry::root("identical-root", &corpus, &[]);

    // The same document as a caching instance's local-entries document, which is
    // "a `026-C3` instance" at `<data>/bindforge/registry/catalog-v1.json`.
    let caching_home = TempDir::new("identical-caching");
    let local_entries = common::registry_document(caching_home.path(), "catalog-v1.json");
    std::fs::create_dir_all(local_entries.parent().expect("the registry directory"))
        .expect("the registry directory is creatable");
    std::fs::write(&local_entries, document.as_bytes()).expect("the local entries land");
    let caching = Registry::caching(
        "identical-caching",
        &common::unreachable_upstream(),
        &caching_home,
        &[],
    );

    for key in ["steam/900100", "epic/55", "local/Steam.SE"] {
        let from_root = root.get(&slug_query(key));
        let from_caching = caching.get(&slug_query(key));
        assert_eq!(
            status_line(&from_root),
            status_line(&from_caching),
            "`{key}` gets the same status line from both modes"
        );
        assert_eq!(
            body_text(&from_root),
            body_text(&from_caching),
            "`{key}` gets the same body bytes from both modes"
        );
        let mut root_headers = header_names(&from_root);
        let mut caching_headers = header_names(&from_caching);
        root_headers.sort();
        caching_headers.sort();
        assert_eq!(
            root_headers, caching_headers,
            "`{key}` gets the same set of header names from both modes"
        );
    }

    // The named mode-revealing exception: an empty join. Root mode answers the
    // `200 []` the design pins; the cold `503` is reachable only from caching
    // mode (`026-L4`), so the two are never confused with each other.
    let from_root = root.get(&slug_query("steam/999999"));
    assert_eq!(
        status_code(&from_root),
        Some(200),
        "a root-mode empty join is `200 []`: {}",
        status_line(&from_root)
    );
    assert_eq!(body_text(&from_root), "[]");
    let from_caching = caching.get(&slug_query("steam/999999"));
    assert_eq!(
        status_code(&from_caching),
        Some(503),
        "the caching-mode cold case is the named `503`, never `[]`: {}",
        status_line(&from_caching)
    );
}

/// proves: Test strategy #2 through the projection law's other half — a
/// corpus-local query never leaks a *different* key's game: `026-L6` permits only
/// the queried key outbound.
#[test]
fn a_queried_key_never_carries_another_keys_game() {
    let data = TempDir::new("join-boundary");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(0, &shared_key_corpus()).as_bytes(),
    );
    let server = Registry::root("join-boundary", &corpus, &[]);

    let answer = server.get(&slug_query("steam/900200"));
    let ids = game_ids(&answer);
    assert_eq!(ids, vec!["alone".to_string()]);
    assert!(
        !ids.contains(&"alpha".to_string()) && !ids.contains(&"local-only".to_string()),
        "no other key's game rides along: {ids:?}"
    );
}
