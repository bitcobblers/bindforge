//! Decode-then-validate: `025-B1`'s cap and the shipped lookup-key rule are
//! applied to the **decoded** `slug` (and to the decoded `DELETE` path segment),
//! with the path-safety refusals at their own scope.
//!
//! proves: design 026's Test strategy #3 — "Decode-then-validate: `025-B1`'s cap
//! and §The serving face step 1's rule apply to **decoded** `slug` and decoded
//! `DELETE` path segment; over-cap, unspellable, separator- or dot-dot-bearing
//! (`local/%2e%2e%2f…`), or control-byte-bearing (`steam/%0a730`) ⇒ `404`; a
//! dotted external id (`local/Steam.SE`) is admitted; encoded length stays within
//! the request budget" (Chosen design: The serving face step 1; Open question 13
//! resolved as decode-first).

mod common;

use common::{
    Registry, TempDir, build_request, corpus_doc, game_ids, percent_encode, slug_query,
    status_code, status_line,
};
use std::io::Write;

/// A corpus with one record per spelling the tests ask about.
fn corpus() -> Vec<(&'static str, &'static str, &'static [&'static str])> {
    vec![
        ("dotted-game", "Dotted Game", &["local/Steam.SE"][..]),
        ("percent-game", "Percent Game", &["local/100%"][..]),
        ("spaced-game", "Spaced Game", &["local/two words"][..]),
        ("plain-game", "Plain Game", &["steam/900001"][..]),
    ]
}

/// One temp corpus and a root instance over it.
fn serving(label: &str) -> (TempDir, std::path::PathBuf, Registry) {
    let data = TempDir::new(label);
    let corpus = data.write("corpus.json", corpus_doc(0, &corpus()).as_bytes());
    let server = Registry::root(label, &corpus, &[]);
    (data, corpus, server)
}

/// proves: Test strategy #3 (decode first, then measure) — a query whose
/// *encoded* form is many times the cap but whose *decoded* key is exactly
/// `025-B1`'s 128 octets is admitted, while one octet more on the decoded side is
/// the `404` `026-C2` names. Both keys are `local/…` keys: §The steam snapshot
/// resolver keeps them out of the resolver's hands, so what answers is the decode
/// plus the join — the thing this leg measures — rather than a hosted-mode
/// resolver verdict on a steam key the corpus does not hold.
#[test]
fn the_octet_cap_and_key_rule_apply_to_the_decoded_slug() {
    let (_data, _corpus, server) = serving("decode-cap");

    // The 128-octet key, spelled out as `%XX` escapes on every octet: the
    // *encoded* query is several times the cap while the decoded key sits
    // exactly on it — admitted, because the cap rides the decoded bytes.
    let decoded = format!("local/{}", "a".repeat(122));
    assert_eq!(decoded.len(), 128, "the fixture key is exactly on the cap");
    let encoded_key: String = decoded.bytes().map(|byte| format!("%{byte:02X}")).collect();
    let encoded_query = format!("/v1/candidates?slug={encoded_key}");
    assert!(
        encoded_query.len() > 3 * decoded.len(),
        "the fixture's encoded form is several times the cap: {}",
        encoded_query.len()
    );
    let answer = server.get(&encoded_query);
    assert_eq!(
        status_code(&answer),
        Some(200),
        "an over-long *encoded* query that decodes inside the cap is admitted: {}",
        status_line(&answer)
    );

    // One octet over the cap once decoded: the named 404.
    let over = format!("local/{}", "a".repeat(123));
    let answer = server.get(&slug_query(&over));
    assert_eq!(
        status_code(&answer),
        Some(404),
        "a decoded slug over `025-B1`'s 128 octets is a `404` (`026-C2`): {}",
        status_line(&answer)
    );
}

/// proves: Test strategy #3 (the refusals) — every spelling the design names is a
/// `404` and never a corpus probe, a refusal line carrying the key, or a `200`.
#[test]
fn every_slug_outside_the_key_rule_is_a_404() {
    let (_data, _corpus, server) = serving("decode-refusals");

    // Each entry is the *decoded* key the registry must refuse after decoding.
    let refused = [
        ("unknown-source-token", "gog/900001"),
        ("missing-separator", "steam-900001"),
        ("empty-external-id", "steam/"),
        ("empty-source", "/900001"),
        ("empty-slug", ""),
        ("second-separator", "steam/900001/eu"),
        ("backslash-separator", "steam\\900001"),
        ("dot-dot-component", "local/../etc"),
        ("dot-component", "local/./x"),
        ("leading-dot-dot", "../steam/900001"),
        ("control-byte", "steam/\n730"),
        ("tab-byte", "steam/\t730"),
    ];
    for (label, decoded) in refused {
        let answer = server.get(&slug_query(decoded));
        assert_eq!(
            status_code(&answer),
            Some(404),
            "`{label}` (`{decoded:?}` decoded, `{}` on the wire) is the `404` `026-C2` \
             names for a malformed slug: {}",
            percent_encode(decoded),
            status_line(&answer)
        );
        let body = common::body_of(&answer);
        assert!(
            !body
                .iter()
                .any(|byte| *byte < 0x20 && *byte != b'\r' && *byte != b'\n'),
            "`{label}` answers without putting a raw control byte on the wire: {body:?}"
        );
    }
}

/// proves: Test strategy #3's admission clause and revision-log r3/r4 — the
/// registry keeps the shipped pair's widened alphabet: a dot in the external id
/// (`local/Steam.SE`, the ADR's `local/<dir>` case) and, by the same rule, a `%`
/// or a space in it are admitted, because `is_slug_name`'s dot ban is
/// `game_id`-only (A4) and a space traverses nothing.
#[test]
fn a_dotted_or_punctuation_bearing_external_id_is_admitted() {
    let (_data, _corpus, server) = serving("decode-admit");

    for (key, expected) in [
        ("local/Steam.SE", "dotted-game"),
        ("local/100%", "percent-game"),
        ("local/two words", "spaced-game"),
        ("steam/900001", "plain-game"),
    ] {
        let answer = server.get(&slug_query(key));
        assert_eq!(
            status_code(&answer),
            Some(200),
            "`{key:?}` is inside the shipped pair's alphabet and must answer: {}",
            status_line(&answer)
        );
        assert_eq!(
            game_ids(&answer),
            vec![expected.to_string()],
            "`{key:?}` joins to its curated record"
        );
    }
}

/// proves: Test strategy #3 (a missing `slug` parameter is a malformed slug) and
/// `026-C2`'s "any other route … is a `404`".
#[test]
fn a_candidate_request_without_a_slug_and_an_unknown_route_are_404() {
    let (_data, _corpus, server) = serving("decode-missing");

    for target in [
        "/v1/candidates",
        "/v1/candidates?slug=",
        "/v1/candidates?other=steam%2F900001",
        "/v1/candidate?slug=steam%2F900001",
        "/",
        "/v1",
    ] {
        let answer = server.get(target);
        assert_eq!(
            status_code(&answer),
            Some(404),
            "`{target}` is the named `404` (`026-C2`): {}",
            status_line(&answer)
        );
    }
}

/// proves: Test strategy #3's last clause and §Budgets' request side — a query
/// line big enough to blow `MAX_REQUEST_HEADER_OCTETS` (65,536, mirroring the
/// client's own header cap) is refused outright rather than decoded and joined.
/// Test strategy #11 fixes the posture a framing refusal takes: refused *before*
/// any mutation, either with a `4xx`/`5xx` status line or with the connection
/// closed and no status line at all — only a served `2xx` is impossible, and no
/// document may move.
#[test]
fn an_encoded_query_over_the_request_header_budget_is_refused() {
    let (_data, corpus, server) = serving("decode-budget");
    let before = std::fs::read(&corpus).expect("the corpus is readable");

    let padded = format!("/v1/candidates?slug={}p", "a".repeat(70_000));
    let raw = build_request("GET", &padded, &server.host_header(), &[], "");
    let mut stream = server.connection();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .ok();
    // A framing refusal may close the door mid-write; the write is not what is
    // under test.
    let _ = stream.write_all(&raw);
    stream.flush().ok();
    let answer = common::read_to_end(&mut stream);
    if let Some(code) = status_code(&answer) {
        assert!(
            !matches!(code, 200..=299),
            "a header block over `MAX_REQUEST_HEADER_OCTETS` is refused, never \
             served: {:?}",
            code
        );
    }
    assert_eq!(
        std::fs::read(&corpus).expect("the corpus survives"),
        before,
        "a framing refusal touches no document"
    );
}

/// proves: Test strategy #3 applied to the admin side (the decoded `DELETE` path
/// segment takes the same rule, Chosen design: The serving face step 1 "governs
/// candidate queries, admin upserts, and cache loads alike") — an undecodable or
/// out-of-grammar `<game_id>` segment is refused and mutates nothing.
#[test]
fn a_delete_segment_outside_the_game_id_grammar_is_refused_and_changes_nothing() {
    let data = TempDir::new("decode-delete");
    let corpus = data.write(
        "corpus.json",
        corpus_doc(
            3,
            &[("example-game", "Example Game", &["steam/900001"][..])],
        )
        .as_bytes(),
    );
    let before = std::fs::read(&corpus).expect("the corpus is readable");
    let server = Registry::root("decode-delete", &corpus, &[]);

    let long_id = "a".repeat(129);
    let refused_segments = [
        ("dot-dot", "../../etc/passwd"),
        ("uppercase", "Example-Game"),
        ("underscore", "example_game"),
        ("empty", ""),
        ("over-bound", long_id.as_str()),
    ];
    for (label, segment) in refused_segments {
        let target = format!("/v1/admin/games/{}", percent_encode(segment));
        let answer = server.request("DELETE", &target, &[], "");
        assert!(
            !matches!(status_code(&answer), Some(200) | Some(204)),
            "`{label}` (`{segment:?}`) is refused before any mutation: {}",
            status_line(&answer)
        );
        assert_eq!(
            std::fs::read(&corpus).expect("the corpus survives"),
            before,
            "`{label}` writes nothing"
        );
    }
}

/// proves: Test strategy #3's wire-shape premise (A19) — the registry's answers
/// are keyed by the decoded bytes it validated, so the corpus' own dotted key
/// resolves through the same percent-encoded spelling the shipped client sends.
#[test]
fn the_wire_spelling_of_a_query_is_the_encoded_form_of_the_decoded_key() {
    let (_data, _corpus, server) = serving("decode-wire");

    let answer = server.get("/v1/candidates?slug=local%2FSteam.SE");
    assert_eq!(status_code(&answer), Some(200));
    assert_eq!(game_ids(&answer), vec!["dotted-game".to_string()]);
    assert!(
        !percent_encode("local/Steam.SE").contains("%2E"),
        "`.` is in the RFC 3986 unreserved set, so the client never escapes it (A19)"
    );
}
