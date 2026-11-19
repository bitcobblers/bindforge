//! The `026-C7` cache document: its shape, its load refusals, its write-time dual
//! eviction under `026-B1` and `026-B2`, and the clock guard over the eviction key.
//!
//! proves: design 026's Test strategy #4 — "Cache document (`026-C7`): valid load
//! sorted by `slug` byte order; refusals for bad version/shape/`game_id`/`name`,
//! repeated `slug`, and a `slug` failing §The serving face step 1's rule; absent
//! file ⇒ empty cache; a `fetched_at` outside the load bound is refused" — and #5 —
//! "Write-time dual eviction (`026-B1`+`026-B2`): after writes, the cache holds
//! `count ≤ 5_000` **and** a written document `≤ 1_048_576` octets, oldest
//! `fetched_at` evicted first; load over `026-B2` is refused by name" — plus the
//! clock guard named in §Budgets ("a backwards-jumped stamp is treated as expired
//! rather than immortal").
//!
//! The document is pinned through the instance's own load and write seams: a
//! poisoned `<data>/bindforge/registry/cache-v1.json` must stop a caching instance
//! at startup by name, and every eviction is read back off the file the instance
//! renames into place. The only upstream is a loopback fake.

mod common;

use common::{
    FakeUpstream, Registry, TempDir, cache_doc_value, candidates_body, json_body, ok,
    registry_document, respond_to, slug_query, status_code, status_line,
};

const ENTRY_CAP: u64 = 5_000;
const OCTET_CAP: usize = 1_048_576;

/// The answer the octet-budget fixture's upstream gives for the new lookup: legal,
/// and large enough that caching it must evict something.
const FRESH_CANDIDATES: [(&str, &str); 8] = [
    ("cand-0000", "A cached candidate name"),
    ("cand-0001", "A cached candidate name"),
    ("cand-0002", "A cached candidate name"),
    ("cand-0003", "A cached candidate name"),
    ("cand-0004", "A cached candidate name"),
    ("cand-0005", "A cached candidate name"),
    ("cand-0006", "A cached candidate name"),
    ("cand-0007", "A cached candidate name"),
];

/// The seeded cache plus a caching instance reading and writing it.
fn with_cache(
    label: &str,
    document: &str,
    upstream: &str,
    extra: &[&str],
) -> (TempDir, std::path::PathBuf, Registry) {
    let data = TempDir::new(label);
    let cache = registry_document(data.path(), "cache-v1.json");
    std::fs::create_dir_all(cache.parent().expect("the registry directory"))
        .expect("the registry directory is creatable");
    std::fs::write(&cache, document.as_bytes()).expect("the seeded cache lands");
    let server = Registry::caching(label, upstream, &data, extra);
    (data, cache, server)
}

fn refuses_cache(document: &str, label: &str, named: &[&str]) {
    let data = TempDir::new(label);
    let cache = registry_document(data.path(), "cache-v1.json");
    std::fs::create_dir_all(cache.parent().expect("the registry directory"))
        .expect("the registry directory is creatable");
    std::fs::write(&cache, document.as_bytes()).expect("the fixture cache lands");
    let before = std::fs::read(&cache).expect("the fixture cache is readable");

    let upstream = common::unreachable_upstream();
    let run = common::expect_refused_startup(
        &["serve", "--upstream", &upstream],
        &[(
            "XDG_DATA_HOME",
            data.path().to_str().expect("a utf-8 data directory"),
        )],
    );
    let lines: Vec<&str> = run.stderr.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "the `{label}` cache refusal is exactly one named line: {:?}",
        run.stderr
    );
    for needle in named {
        assert!(
            lines[0].contains(needle),
            "the `{label}` refusal must name {needle:?}: {:?}",
            lines[0]
        );
    }
    assert!(
        lines[0].contains("cache"),
        "the refusal names the document it refused: {:?}",
        lines[0]
    );
    assert_eq!(
        std::fs::read(&cache).expect("the fixture cache survives"),
        before,
        "a refused cache document is never rewritten on disk (`026-C7`, A7)"
    );
}

fn entry(slug: &str, fetched_at: u64) -> serde_json::Value {
    serde_json::json!({
        "slug": slug,
        "candidates": [{"game_id": "cached-game", "name": "Cached Game"}],
        "fetched_at": fetched_at,
    })
}

fn fat_entry(slug: &str, fetched_at: u64, candidates: usize) -> serde_json::Value {
    let list: Vec<serde_json::Value> = (0..candidates)
        .map(|_| serde_json::json!({"game_id": "cand-0000", "name": "A cached candidate name"}))
        .collect();
    serde_json::json!({"slug": slug, "candidates": list, "fetched_at": fetched_at})
}

fn entry_value(
    slug: &str,
    candidates: &[(&str, &str)],
    fetched_at: serde_json::Value,
) -> serde_json::Value {
    let list: Vec<serde_json::Value> = candidates
        .iter()
        .map(|(game_id, name)| serde_json::json!({"game_id": game_id, "name": name}))
        .collect();
    serde_json::json!({"slug": slug, "candidates": list, "fetched_at": fetched_at})
}

fn cached_slugs(document: &serde_json::Value) -> Vec<String> {
    document["entries"]
        .as_array()
        .expect("entries is an array")
        .iter()
        .map(|entry| {
            entry["slug"]
                .as_str()
                .expect("an entry carries a slug")
                .to_string()
        })
        .collect()
}

fn read_cache(cache: &std::path::Path) -> (String, serde_json::Value) {
    let text = std::fs::read_to_string(cache).expect("the cache document is written");
    let document: serde_json::Value = serde_json::from_str(&text).expect("the cache is JSON");
    (text, document)
}

/// proves: Test strategy #4's "absent file ⇒ empty cache" (`026-C7` closes with
/// "An absent cache file is an empty cache, not an error").
#[test]
fn an_absent_cache_document_is_an_empty_cache_not_an_error() {
    let data = TempDir::new("cache-absent");
    let server = Registry::caching("cache-absent", &common::unreachable_upstream(), &data, &[]);

    let health = json_body(&server.health());
    assert_eq!(
        health["cached_slugs"].as_u64(),
        Some(0),
        "no file on disk is an empty cache: {health:?}"
    );
    assert_eq!(
        json_body(&server.get("/v1/admin/stats"))["games"].as_u64(),
        Some(0)
    );
    assert!(
        !registry_document(data.path(), "cache-v1.json").exists(),
        "reading an empty cache does not conjure the file"
    );
}

/// proves: Test strategy #4's "valid load sorted by `slug` byte order" — a write
/// keeps `026-C7`'s stated order, whatever order the lookups arrived in.
#[test]
fn the_cache_document_is_kept_sorted_by_slug_byte_order() {
    let data = TempDir::new("cache-order");
    let upstream = FakeUpstream::start(respond_to(
        vec![
            (
                "slug=steam%2F900003".to_string(),
                ok(&candidates_body(&[("three", "Three")])),
            ),
            (
                "slug=steam%2F900001".to_string(),
                ok(&candidates_body(&[("one", "One")])),
            ),
            (
                "slug=epic%2F5".to_string(),
                ok(&candidates_body(&[("five", "Five")])),
            ),
        ],
        ok("[]"),
    ));
    let server = Registry::caching("cache-order", &upstream.url(), &data, &[]);

    for key in ["steam/900003", "steam/900001", "epic/5"] {
        assert_eq!(status_code(&server.get(&slug_query(key))), Some(200));
    }

    let (_text, document) = read_cache(&registry_document(data.path(), "cache-v1.json"));
    assert_eq!(
        document["version"].as_i64(),
        Some(1),
        "`026-C7` holds `{{version, entries: [{{slug, candidates, fetched_at}}]}}`"
    );
    assert_eq!(
        cached_slugs(&document),
        vec![
            "epic/5".to_string(),
            "steam/900001".to_string(),
            "steam/900003".to_string()
        ],
        "entries are stored sorted by `slug` byte order (`026-C7`), not in arrival order"
    );
    for entry in document["entries"].as_array().expect("entries") {
        assert!(
            entry["fetched_at"].as_u64().is_some(),
            "each entry carries the `fetched_at` the TTL and the eviction key need: \
             {entry:?}"
        );
        assert!(
            entry["candidates"].as_array().is_some(),
            "each entry caches the `{{game_id, name}}` answer: {entry:?}"
        );
    }
}

/// proves: Test strategy #4's refusal set (`026-C7`'s "untrusted input at load and
/// refused by name outside its shape, bounds, and the `game_id` grammar") — every
/// poisoned document stops the instance with one named line and is not rewritten.
#[test]
fn every_out_of_shape_cache_document_is_refused_by_name() {
    let now = common::now_unix();

    refuses_cache(
        &cache_doc_value(serde_json::json!(2), vec![entry("steam/900001", now)]),
        "cache-version",
        &["version"],
    );
    refuses_cache("{\"version\": 1}", "cache-missing-entries", &["entries"]);
    refuses_cache(
        "{\"version\": 1, \"entries\": {\"steam/900001\": []}}",
        "cache-entries-not-an-array",
        &["entries"],
    );
    refuses_cache(
        &cache_doc_value(
            serde_json::json!(1),
            vec![serde_json::json!("steam/900001")],
        ),
        "cache-entry-not-an-object",
        &["entries"],
    );
    refuses_cache(
        &cache_doc_value(
            serde_json::json!(1),
            vec![serde_json::json!({"slug": "steam/900001", "fetched_at": now})],
        ),
        "cache-missing-candidates",
        &["candidates"],
    );
    refuses_cache(
        &cache_doc_value(
            serde_json::json!(1),
            vec![entry_value(
                "steam/900001",
                &[("../x", "Traversal")],
                serde_json::json!(now),
            )],
        ),
        "cache-bad-game-id",
        &["game_id"],
    );
    let long_name = "n".repeat(129);
    refuses_cache(
        &cache_doc_value(
            serde_json::json!(1),
            vec![entry_value(
                "steam/900001",
                &[("ok-game", long_name.as_str())],
                serde_json::json!(now),
            )],
        ),
        "cache-long-name",
        &["name"],
    );
    refuses_cache(
        &cache_doc_value(
            serde_json::json!(1),
            vec![entry("steam/900001", now), entry("steam/900001", now - 5)],
        ),
        "cache-repeated-slug",
        &["steam/900001"],
    );

    // The §serving-face step-1 rule, applied to a cached `slug`.
    let over_cap = format!("steam/{}", "9".repeat(130));
    for (label, slug) in [
        ("cache-slug-unknown-token", "gog/900001"),
        ("cache-slug-no-token", "steam-900001"),
        ("cache-slug-second-separator", "steam/900001/eu"),
        ("cache-slug-control-byte", "steam/900\n001"),
        ("cache-slug-over-cap", over_cap.as_str()),
    ] {
        refuses_cache(
            &cache_doc_value(serde_json::json!(1), vec![entry(slug, now)]),
            label,
            &[],
        );
    }

    // `fetched_at` is untrusted input, so the load refuses a stamp outside
    // `[now − 31_536_000, now + 86_400]` (§Budgets) — and a non-number.
    for (label, stamp) in [
        ("cache-future-stamp", serde_json::json!(now + 100_000)),
        ("cache-ancient-stamp", serde_json::json!(now - 40_000_000)),
        ("cache-zero-stamp", serde_json::json!(0)),
        ("cache-string-stamp", serde_json::json!("yesterday")),
        ("cache-negative-stamp", serde_json::json!(-1_i64)),
    ] {
        refuses_cache(
            &cache_doc_value(
                serde_json::json!(1),
                vec![entry_value(
                    "steam/900001",
                    &[("ok-game", "Ok")],
                    stamp.clone(),
                )],
            ),
            label,
            &["fetched_at"],
        );
    }
}

/// proves: Test strategy #5's load half (`026-B2`: "the cache document's load
/// ceiling, past which the file is refused by name like the match store's") — the
/// cap gates the read (A22's metadata discipline) and the file is left alone.
#[test]
fn a_cache_document_over_the_octet_budget_is_refused_by_name() {
    let now = common::now_unix();
    let mut entries: Vec<serde_json::Value> = Vec::new();
    let mut index = 0usize;
    while cache_doc_value(serde_json::json!(1), entries.clone()).len() <= OCTET_CAP {
        let slug = format!("steam/{:06}", 400_000 + entries.len());
        entries.push(fat_entry(&slug, now - 10_000 + entries.len() as u64, 200));
        index += 1;
        assert!(index < 500, "the fixture generator terminates");
    }
    let document = cache_doc_value(serde_json::json!(1), entries);
    assert!(
        document.len() > OCTET_CAP,
        "the fixture is over `026-B2` by construction: {}",
        document.len()
    );
    refuses_cache(&document, "cache-over-budget", &[]);
}

/// proves: Test strategy #5 (`026-B1`) — with the cache full at `5_000` slugs, the
/// next write evicts the oldest `fetched_at` (that budget's own key), so the count
/// never goes over the cap.
#[test]
fn a_write_past_the_entry_cap_evicts_the_oldest_fetched_at() {
    let now = common::now_unix();
    let entries: Vec<serde_json::Value> = (0..ENTRY_CAP)
        .map(|index| entry(&format!("steam/{}", 700_000 + index), now - 6_000 + index))
        .collect();
    let document = cache_doc_value(serde_json::json!(1), entries);
    assert!(
        document.len() < OCTET_CAP,
        "this fixture exercises the count cap, not the octet cap: {}",
        document.len()
    );

    let upstream = FakeUpstream::start(respond_to(
        vec![(
            "slug=steam%2F900999".to_string(),
            ok(&candidates_body(&[("fresh", "Fresh Game")])),
        )],
        ok("[]"),
    ));
    let (_data, cache, server) = with_cache("cache-count-cap", &document, &upstream.url(), &[]);
    assert_eq!(
        json_body(&server.health())["cached_slugs"].as_u64(),
        Some(ENTRY_CAP),
        "the seeded cache loaded whole"
    );

    let answer = server.get(&slug_query("steam/900999"));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "the fresh lookup is served: {}",
        status_line(&answer)
    );

    let (text, persisted) = read_cache(&cache);
    let slugs = cached_slugs(&persisted);
    assert_eq!(
        slugs.len(),
        ENTRY_CAP as usize,
        "`026-B1` holds after the write: exactly the cap, never cap + 1"
    );
    assert!(
        !slugs.contains(&"steam/700000".to_string()),
        "the oldest `fetched_at` is evicted first (`026-B1`'s own key)"
    );
    assert!(
        slugs.contains(&"steam/900999".to_string()),
        "the entry the lookup asked for is cached"
    );
    assert!(
        text.len() <= OCTET_CAP,
        "and the written document is inside `026-B2`: {}",
        text.len()
    );
    assert_eq!(
        json_body(&server.health())["cached_slugs"].as_u64(),
        Some(ENTRY_CAP),
        "health's `cached_slugs` agrees with the file"
    );
}

/// proves: Test strategy #5's octet half — a cache of legal-but-large entries is
/// brought back inside `1_048_576` octets of **the document that gets renamed**
/// (§Budgets: "the octet test is the document's exact length and not a sum of
/// per-entry totals"), evicting the oldest `fetched_at` first.
#[test]
fn a_write_past_the_octet_budget_evicts_oldest_until_the_document_fits() {
    let now = common::now_unix();
    let mut entries: Vec<serde_json::Value> = Vec::new();
    // Stage 1: fat entries carry the seeded cache to within 100 KiB of the cap.
    while cache_doc_value(serde_json::json!(1), entries.clone()).len() < OCTET_CAP - 100_000 {
        let slug = format!("steam/{:06}", 400_000 + entries.len());
        entries.push(fat_entry(&slug, now - 10_000 + entries.len() as u64, 200));
        assert!(entries.len() < 500, "the fixture generator terminates");
    }
    // Stage 2: one-candidate rows close the remaining gap a row at a time, so
    // the leftover room provably is smaller than the fresh answer needs.
    loop {
        let slug = format!("steam/{:06}", 400_000 + entries.len());
        let candidate = entry(&slug, now - 10_000 + entries.len() as u64);
        let probe = {
            let mut trial = entries.clone();
            trial.push(candidate.clone());
            cache_doc_value(serde_json::json!(1), trial)
        };
        if probe.len() > OCTET_CAP - 100 {
            break;
        }
        entries.push(candidate);
        assert!(entries.len() < 2_000, "the fixture generator terminates");
    }
    assert!(entries.len() >= 3, "the fixture needs several entries");
    let document = cache_doc_value(serde_json::json!(1), entries.clone());
    assert!(
        document.len() < OCTET_CAP,
        "the seeded cache is loadable: {}",
        document.len()
    );
    assert!(
        document.len() + 400 > OCTET_CAP,
        "the fixture leaves less room than the new entry needs: {}",
        document.len()
    );
    let oldest = "steam/400000".to_string();

    let fresh: Vec<(&str, &str)> = FRESH_CANDIDATES.to_vec();
    let upstream = FakeUpstream::start(respond_to(
        vec![(
            "slug=steam%2F900888".to_string(),
            ok(&candidates_body(&fresh)),
        )],
        ok("[]"),
    ));
    let (_data, cache, server) = with_cache("cache-octet-cap", &document, &upstream.url(), &[]);

    let answer = server.get(&slug_query("steam/900888"));
    assert_eq!(
        status_code(&answer),
        Some(200),
        "the lookup is served from the upstream: {}",
        status_line(&answer)
    );

    let (text, persisted) = read_cache(&cache);
    assert!(
        text.len() <= OCTET_CAP,
        "the document the instance renames into place is inside `026-B2`: {}",
        text.len()
    );
    let slugs = cached_slugs(&persisted);
    assert!(
        !slugs.contains(&oldest),
        "the oldest `fetched_at` went first to buy the room (`026-B1` is the eviction \
         key for both caps)"
    );
    assert!(
        slugs.contains(&"steam/900888".to_string()),
        "the entry the lookup asked for survived its own write"
    );
}

/// proves: §Budgets' clock guard — a stamp ahead of the wall clock is a
/// backwards-jumped clock as far as TTL arithmetic can see, and the design treats
/// it as **expired**, never immortal: with `--refresh-interval 1` a future-dated
/// entry is refreshed rather than served forever.
#[test]
fn a_future_dated_entry_is_expired_rather_than_immortal() {
    let _data = TempDir::new("cache-future-guard");
    let now = common::now_unix();
    let stamped = cache_doc_value(
        serde_json::json!(1),
        vec![entry_value(
            "steam/900777",
            &[("immortal", "Immortal Game")],
            serde_json::json!(now + 3_600),
        )],
    );
    let upstream = FakeUpstream::start(respond_to(
        vec![(
            "slug=steam%2F900777".to_string(),
            ok(&candidates_body(&[("refreshed", "Refreshed Game")])),
        )],
        ok("[]"),
    ));
    let (_data, _cache, server) = with_cache(
        "cache-future-guard",
        &stamped,
        &upstream.url(),
        &["--refresh-interval", "1"],
    );

    let answer = server.get(&slug_query("steam/900777"));
    assert_eq!(
        common::game_ids(&answer),
        vec!["refreshed".to_string()],
        "a stamp the clock has jumped back past is expired, not immortal"
    );
    assert_eq!(
        upstream.lines_for("GET /v1/candidates").len(),
        1,
        "and the instance really refreshed it: {:?}",
        upstream.request_lines()
    );
}
