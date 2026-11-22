//! The module suite for `apps/registry/src/steam.rs` (ADR-004's module-mirrored
//! shape): the snapshot document and its loader refusals (`027-C3`), the store pull
//! codec and its budgets (`027-B4`), the production transport's assurance half, and
//! the refresh scheduler's decision — the four things in `027` that have no route
//! through the binary at all.
//!
//! proves: design 027's Test strategy #2 — "valid fixture loads; named refusals for
//! `version ≠ 1`, out-of-order, repeated or non-decimal `appid` in `apps`,
//! over-`013-B4` or control-byte `name`, `fetched_at` outside the plausibility pair,
//! over-`027-B1` (metadata-gated, file never read), and malformed shape"; #6 — "the
//! loopback fake … requests carry `max_results=50000`, `last_appid` continuation, no
//! `include_*` flag, `if_modified_since` on deltas, and nothing derived from any
//! queried key (the `027-L4` assertion); `3xx`, `Transfer-Encoding`,
//! over-`MAX_STEAM_PAGE_OCTETS`, a body that is not the A17 envelope or carries no
//! `apps` list; more than `STEAM_REFRESH_MAX_PAGES` (12) pages while A17's
//! `have_more_results` is true; a non-decimal `appid` each fail by name; a
//! bound-breaking or control-byte `name` is control-stripped then clamped to the
//! `013-B4`-legal form, an all-whitespace or all-control title stores empty, and its
//! record stays in the snapshot; retries stop at `STEAM_PAGE_ATTEMPTS`, the gap
//! between them is the page backoff and never `STEAM_RETRY_SECONDS`, and the deadline
//! turns a stalled pull into `steam pull deadline`; a partial run never publishes; a
//! delta merges by one linear two-way merge preserving order; a full pull replaces;
//! no failure line ever contains the key or the request URI" and its transport half;
//! and #11 — "with injected clock and transport, the pull is due at 86,400 s, a
//! failed pull retries at 3,600 s and keeps the old snapshot published; the thread's
//! seam is a pure decision function — no unit sleeps".
//!
//! # The seam contract this suite pins
//!
//! `apps/registry` is a binary today, so these tests need the crate to grow the
//! library target §The store pull's own wording assumes — "the *production*
//! `StoreTransport`, reached through a test constructor taking the connect address
//! and the name to verify separately (the compile-time-pinned authority stays the
//! production value)" and "the same request/response codec run against a loopback
//! fake (plain TCP)". Concretely, `bindforge_registry::steam` must expose:
//!
//! * `pub const STORE_AUTHORITY: &str` — the compile-time-pinned authority.
//! * `pub fn load_snapshot(path: &Path, now: u64) -> Result<SteamSnapshot, R>` with
//!   `R: Display` naming each loader refusal.
//! * `pub struct SteamSnapshot` with `for_test(fetched_at, &[(appid, name)])`,
//!   `fetched_at()`, and `apps()` over `{ appid: u64, name: String }`.
//! * `pub struct SteamStore` with `for_test(SocketAddr, &str key)` (the codec over a
//!   plaintext loopback), `.with_production_transport(&str verify_name)` (the rustls
//!   client, dialed at the test address, verifying `verify_name`),
//!   `.with_timing(page_retry, deadline)`, `full(now)` and `delta(now, &loaded)`.
//! * `pub fn next_refresh_seconds(now, last_pull_at, PullOutcome::{Published,Failed})
//!   -> u64` — the refresh thread's pure decision.
//!
//! Nothing here dials or resolves anything outside `127.0.0.1`, and no test sleeps
//! longer than the injected timing allows (Test strategy #17).
//!
//! That seam is the crate's library target, which has landed: this suite drives the
//! design's four in-process faces directly, because none of them has a route through
//! the binary at all. The list above is the record of the seam this suite actually
//! exercises, so a rename on either side shows up as a compile failure here rather
//! than as a silently dropped promise.
//!
//! # The #6 transport-assurance pair — TLS, not a gap note
//!
//! #6's pair is driven against loopback **rustls servers** presenting the
//! checked-in self-signed certificates in `tests/fixtures/store/` (their
//! README carries the generation command): one for the very name being
//! verified, one for another name. The client is the production `ClientConfig`
//! — webpki roots only — so both handshakes must die on chain-building, and
//! what the fake decrypts must stay empty: if a verifier were ever relaxed to
//! accept an untrusted chain, the handshake would complete and the fake's
//! *decrypted* stream would receive the real `GET` with the key in it, which
//! is exactly what the two legs below assert against. A plaintext socket
//! cannot make that claim, and until these fixtures landed it didn't.

mod common;

use bindforge_registry::steam::{
    PullOutcome, STORE_AUTHORITY, SteamSnapshot, SteamStore, load_snapshot, next_refresh_seconds,
};
use common::{FakeUpstream, TempDir, now_unix, ok};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

/// `027-B1` / `MAX_STEAM_SNAPSHOT_OCTETS` (§Budgets).
const MAX_STEAM_SNAPSHOT_OCTETS: usize = 33_554_432;
/// `MAX_STEAM_PAGE_OCTETS` — 12 MiB (§Budgets).
const MAX_STEAM_PAGE_OCTETS: usize = 12_582_912;
/// `STEAM_REFRESH_MAX_PAGES` (`027-B4`, against the pinned 50k page).
const STEAM_REFRESH_MAX_PAGES: usize = 12;
/// `STEAM_PAGE_ATTEMPTS` (§Budgets).
const STEAM_PAGE_ATTEMPTS: usize = 3;
/// `STEAM_PAGE_MAX_RESULTS`, pinned explicitly on every page (§The store pull).
const STEAM_PAGE_MAX_RESULTS: usize = 50_000;
/// `013-B4`'s name bound (`MAX_NATIVE_NAME_OCTETS`, A10).
const MAX_NATIVE_NAME_OCTETS: usize = 128;
/// `STEAM_SNAPSHOT_REFRESH_SECONDS` and `STEAM_SNAPSHOT_MAX_AGE_SECONDS` (`027-B2`).
const REFRESH_SECONDS: u64 = 86_400;
const MAX_AGE_SECONDS: u64 = 604_800;
/// `STEAM_RETRY_SECONDS` — the refresh thread's between-pull gap only (§Budgets).
const RETRY_SECONDS: u64 = 3_600;

const KEY: &str = "bindforge-test-store-key-3d9a";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// The A17 envelope, spelled here so the suite owns the shape it hands the codec.
fn page_body(records: &[(u64, &str)], have_more_results: bool) -> String {
    let apps: Vec<serde_json::Value> = records
        .iter()
        .map(|(appid, name)| {
            serde_json::json!({
                "appid": appid,
                "name": name,
                "last_modified": 1_700_000_000_u64,
                "price_change_number": 0,
            })
        })
        .collect();
    serde_json::to_string(
        &serde_json::json!({ "apps": apps, "have_more_results": have_more_results }),
    )
    .expect("an envelope serialises")
}

fn write_snapshot(path: &Path, text: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the fixture parent exists");
    }
    std::fs::write(path, text.as_bytes()).expect("the snapshot fixture is written");
}

fn snapshot_text(
    version: serde_json::Value,
    fetched_at: serde_json::Value,
    apps: serde_json::Value,
) -> String {
    serde_json::to_string(&serde_json::json!({
        "version": version,
        "fetched_at": fetched_at,
        "apps": apps,
    }))
    .expect("a snapshot document serialises")
}

fn apps_value(apps: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::Value::Array(apps)
}

fn load(text: &str, now: u64, label: &str) -> Result<SteamSnapshot, String> {
    let dir = TempDir::new(label);
    let path = dir.join("steam-snapshot.json");
    write_snapshot(&path, text);
    load_snapshot(&path, now).map_err(|refusal| refusal.to_string())
}

fn app_ids(snapshot: &SteamSnapshot) -> Vec<u64> {
    snapshot.apps().iter().map(|app| app.appid).collect()
}

fn store_at(address: SocketAddr) -> SteamStore {
    SteamStore::for_test(address, KEY)
        .with_timing(Duration::from_millis(50), Duration::from_secs(20))
}

fn loopback(
    responder: impl Fn(&str) -> Vec<u8> + Send + Sync + 'static,
) -> (SocketAddr, FakeUpstream) {
    let fake = FakeUpstream::start(responder);
    let address = fake
        .url()
        .replace("http://", "")
        .parse::<SocketAddr>()
        .expect("a loopback fake address");
    (address, fake)
}

/// The query parameters of the request target the fake recorded.
fn query_params(request: &str) -> Vec<(String, String)> {
    let Some(target) = request.split_whitespace().nth(1) else {
        return Vec::new();
    };
    match target.split_once('?') {
        Some((_, query)) => query
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| match pair.split_once('=') {
                Some((name, value)) => (name.to_string(), value.to_string()),
                None => (pair.to_string(), String::new()),
            })
            .collect(),
        None => Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Test strategy #2 — the snapshot document and its loader
// ---------------------------------------------------------------------------

/// proves: Test strategy #2's first clause — a valid fixture loads, in the
/// ascending `appid` order `027-C3` fixes, with its own `fetched_at`.
#[test]
fn a_valid_snapshot_loads_in_appid_order() {
    let now = now_unix();
    let text = snapshot_text(
        serde_json::json!(1),
        serde_json::json!(now),
        apps_value(vec![
            serde_json::json!({"appid": 730, "name": "Counter-Strike 2"}),
            serde_json::json!({"appid": 57_100, "name": "Tunic"}),
            serde_json::json!({"appid": 9_600_003, "name": "Outer Wilds"}),
        ]),
    );
    let snapshot = load(&text, now, "steam-load-valid").expect("a valid snapshot loads");
    assert_eq!(snapshot.fetched_at(), now);
    assert_eq!(app_ids(&snapshot), vec![730, 57_100, 9_600_003]);
    assert_eq!(snapshot.apps()[1].name, "Tunic");
}

/// proves: Test strategy #2 with §The steam snapshot resolver — freshness is a
/// policy question, not a load question: a snapshot inside the plausibility pair
/// but past `027-B2`'s age bound still *loads* (the policy is what turns its misses
/// into the named `503`).
#[test]
fn a_stale_but_plausible_snapshot_loads() {
    let now = now_unix();
    let fetched_at = now - MAX_AGE_SECONDS - 3_600;
    let text = snapshot_text(
        serde_json::json!(1),
        serde_json::json!(fetched_at),
        apps_value(vec![
            serde_json::json!({"appid": 730, "name": "Counter-Strike 2"}),
        ]),
    );
    let snapshot = load(&text, now, "steam-load-stale").expect("a plausible stamp loads");
    assert_eq!(snapshot.fetched_at(), fetched_at);
}

/// proves: Test strategy #2's `version ≠ 1` clause.
#[test]
fn a_snapshot_of_the_wrong_version_is_refused_by_name() {
    for version in [
        serde_json::json!(0),
        serde_json::json!(2),
        serde_json::json!("1"),
    ] {
        let now = now_unix();
        let text = snapshot_text(
            version.clone(),
            serde_json::json!(now),
            apps_value(vec![
                serde_json::json!({"appid": 730, "name": "Counter-Strike 2"}),
            ]),
        );
        let refusal = load(&text, now, "steam-load-version")
            .err()
            .unwrap_or_else(|| {
                panic!("`version: {version}` is refused (§The steam snapshot resolver)")
            });
        assert!(
            refusal.to_ascii_lowercase().contains("version"),
            "the refusal names the condition it refused: {refusal}"
        );
    }
}

/// proves: Test strategy #2's order clause with §The steam snapshot resolver —
/// "an out-of-order file is refused, not resorted (`026-C7`'s sort-rule
/// precedent)".
#[test]
fn an_out_of_order_apps_list_is_refused_not_resorted() {
    let now = now_unix();
    let text = snapshot_text(
        serde_json::json!(1),
        serde_json::json!(now),
        apps_value(vec![
            serde_json::json!({"appid": 900, "name": "Later"}),
            serde_json::json!({"appid": 730, "name": "Earlier"}),
        ]),
    );
    let refusal =
        load(&text, now, "steam-load-order").expect_err("an unordered `apps` list is refused");
    assert!(
        refusal.to_ascii_lowercase().contains("order"),
        "the refusal says the order is the fault: {refusal}"
    );
}

/// proves: Test strategy #2's `appid` clauses — a repeated `appid` and a
/// non-decimal `appid` are both refusals of the whole document.
#[test]
fn a_repeated_or_non_decimal_appid_is_refused() {
    let now = now_unix();
    let repeated = snapshot_text(
        serde_json::json!(1),
        serde_json::json!(now),
        apps_value(vec![
            serde_json::json!({"appid": 730, "name": "Once"}),
            serde_json::json!({"appid": 730, "name": "Twice"}),
        ]),
    );
    let refusal =
        load(&repeated, now, "steam-load-repeat").expect_err("a repeated appid is refused");
    assert!(
        refusal.to_ascii_lowercase().contains("appid"),
        "the refusal names `appid`: {refusal}"
    );

    for bogus in ["\"730\"", "null", "\"abc\"", "-7", "1.5"] {
        let text = format!(
            "{{\"version\":1,\"fetched_at\":{now},\"apps\":[{{\"appid\":{bogus},\"name\":\"X\"}}]}}"
        );
        let refusal = load(&text, now, "steam-load-decimal")
            .err()
            .unwrap_or_else(|| panic!("`appid: {bogus}` is not a decimal integer and is refused"));
        assert!(
            refusal.to_ascii_lowercase().contains("appid"),
            "the refusal names `appid` for `{bogus}`: {refusal}"
        );
    }
}

/// proves: Test strategy #2's `name` clauses — the `013-B4` bound and the
/// control-byte rule are loader refusals, because the pull normalizes before it
/// writes and a document that carries either is not one this instance serves.
#[test]
fn an_over_bound_or_control_byte_name_is_refused() {
    let now = now_unix();
    let over_bound = "k".repeat(MAX_NATIVE_NAME_OCTETS + 1);
    for (label, name) in [
        ("over-bound", over_bound.clone()),
        ("nul", "Game\0Name".to_string()),
        ("escape", "Game\u{1b}Name".to_string()),
        ("delete", "Game\u{7f}Name".to_string()),
        ("c1-range", "Game\u{9c}Name".to_string()),
        ("tab", "Game\tName".to_string()),
    ] {
        let text = snapshot_text(
            serde_json::json!(1),
            serde_json::json!(now),
            apps_value(vec![serde_json::json!({"appid": 730, "name": name})]),
        );
        let refusal = load(&text, now, "steam-load-name")
            .err()
            .unwrap_or_else(|| panic!("a `{label}` name is refused by the loader"));
        assert!(
            refusal.to_ascii_lowercase().contains("name"),
            "the `{label}` refusal names `name`: {refusal}"
        );
    }

    // And the bound itself is admitted: exactly `013-B4` is legal, one over is not.
    let at_bound = "k".repeat(MAX_NATIVE_NAME_OCTETS);
    let text = snapshot_text(
        serde_json::json!(1),
        serde_json::json!(now),
        apps_value(vec![
            serde_json::json!({"appid": 730, "name": at_bound.clone()}),
        ]),
    );
    let snapshot = load(&text, now, "steam-load-name-bound").expect("a name at the bound loads");
    assert_eq!(snapshot.apps()[0].name, at_bound);
}

/// proves: Test strategy #2's plausibility clause with §Assumption register A12 —
/// `fetched_at` is validated at load against the shipped `026-C7` tolerance pair,
/// because `SystemTime` is not monotonic and a future-dated stamp would re-arm
/// `027-L3`'s `200 []`.
#[test]
fn a_fetched_at_outside_the_plausibility_pair_is_refused() {
    let now = now_unix();
    for (label, stamp) in [
        ("a decade of clock drift", now - 10 * 365 * REFRESH_SECONDS),
        ("two days ahead", now + 2 * REFRESH_SECONDS + 600),
    ] {
        let text = snapshot_text(
            serde_json::json!(1),
            serde_json::json!(stamp),
            apps_value(vec![
                serde_json::json!({"appid": 730, "name": "Counter-Strike 2"}),
            ]),
        );
        let refusal = load(&text, now, "steam-load-stamp")
            .err()
            .unwrap_or_else(|| panic!("`{label}` is outside the plausibility pair"));
        assert!(
            refusal.to_ascii_lowercase().contains("fetched"),
            "the `{label}` refusal names `fetched_at`: {refusal}"
        );
    }

    // A stamp one day ahead of the clock is still inside the future tolerance the
    // §Risks row bounds at 86,400 s — the pair is what refuses, not the age bound.
    let text = snapshot_text(
        serde_json::json!(1),
        serde_json::json!(now + 3_600),
        apps_value(vec![
            serde_json::json!({"appid": 730, "name": "Counter-Strike 2"}),
        ]),
    );
    assert!(
        load(&text, now, "steam-load-plausible").is_ok(),
        "an hour of forward drift is inside the tolerance the design bought"
    );
}

/// proves: Test strategy #2's over-`027-B1` clause — the bound is a metadata gate
/// decided *before* the file is read, so a shape-valid document is refused on its
/// size alone: only a size check can produce this refusal.
#[test]
fn an_over_bound_snapshot_is_refused_on_its_metadata_alone() {
    let now = now_unix();
    let small = snapshot_text(
        serde_json::json!(1),
        serde_json::json!(now),
        apps_value(vec![
            serde_json::json!({"appid": 730, "name": "Counter-Strike 2"}),
        ]),
    );
    let mut padded = " ".repeat(MAX_STEAM_SNAPSHOT_OCTETS - small.len() + 1);
    padded.push_str(&small);
    assert!(padded.len() > MAX_STEAM_SNAPSHOT_OCTETS);

    let refusal =
        load(&padded, now, "steam-load-octets").expect_err("an over-`027-B1` file is refused");
    let lower = refusal.to_ascii_lowercase();
    assert!(
        lower.contains("octet") || lower.contains("bound") || lower.contains("size"),
        "the refusal names the size bound: {refusal}"
    );
    assert!(
        !lower.contains("version") && !lower.contains("appid"),
        "the file was never parsed, so no shape condition can appear: {refusal}"
    );
}

/// proves: Test strategy #2's last clause — malformed shape is a named refusal,
/// including a missing `apps`, a non-object record, and a document that is not JSON
/// at all.
#[test]
fn a_malformed_snapshot_is_refused() {
    let now = now_unix();
    let cases: Vec<(&str, String)> = vec![
        (
            "no-apps",
            serde_json::to_string(&serde_json::json!({"version": 1, "fetched_at": now}))
                .expect("serialises"),
        ),
        (
            "apps-not-a-list",
            snapshot_text(
                serde_json::json!(1),
                serde_json::json!(now),
                serde_json::json!({"appid": 730}),
            ),
        ),
        (
            "record-without-name",
            snapshot_text(
                serde_json::json!(1),
                serde_json::json!(now),
                apps_value(vec![serde_json::json!({"appid": 730})]),
            ),
        ),
        (
            "fetched-at-not-decimal",
            snapshot_text(
                serde_json::json!(1),
                serde_json::json!("yesterday"),
                apps_value(Vec::new()),
            ),
        ),
        ("not-json", "this is not a snapshot".to_string()),
    ];
    for (label, text) in cases {
        assert!(
            load(&text, now, "steam-load-malformed").is_err(),
            "`{label}` is a malformed document and must be refused (§The steam snapshot \
             resolver's contract-document discipline)"
        );
    }
}

// ---------------------------------------------------------------------------
// Test strategy #6 — the pull codec against a loopback fake
// ---------------------------------------------------------------------------

/// proves: Test strategy #6's request clauses — `max_results=50000` pinned
/// explicitly on every page (the research's Q5 finding), `last_appid` continuation,
/// the `key` from the environment seam, no `include_*` flag ever, and no
/// `if_modified_since` on a full pull (§The store pull's request rules).
#[test]
fn a_full_pull_pins_the_page_size_and_continues_by_last_appid() {
    let second = page_body(&[(900, "Second Game")], false);
    let first = page_body(&[(730, "Counter-Strike 2")], true);
    let (address, fake) = loopback(move |request| {
        if request.contains("last_appid=") {
            ok(&second)
        } else {
            ok(&first)
        }
    });
    let store = store_at(address);

    let snapshot = store
        .full(now_unix())
        .unwrap_or_else(|refusal| panic!("a two-page pull publishes: {refusal}"));
    assert_eq!(app_ids(&snapshot), vec![730, 900]);

    let lines = fake.request_lines();
    assert_eq!(lines.len(), 2, "one request per page: {lines:?}");
    for line in &lines {
        let params = query_params(line);
        assert!(
            line.starts_with("GET ") && line.contains("/IStoreService/GetAppList/v1"),
            "the pull is the keyed `IStoreService/GetAppList` request (§The store pull): {line}"
        );
        assert!(
            params
                .iter()
                .any(|(name, value)| name == "key" && value == KEY),
            "the key arrives from the environment seam on every page: {line}"
        );
        assert!(
            params.iter().any(|(name, value)| name == "max_results"
                && value == &STEAM_PAGE_MAX_RESULTS.to_string()),
            "`max_results={STEAM_PAGE_MAX_RESULTS}` is pinned explicitly on *every* page — \
             the documented 10k default would trip `027-B4`'s 12-page budget against a \
             ~200k catalog (open Q5): {line}"
        );
        assert!(
            !params.iter().any(|(name, _)| name.starts_with("include_")),
            "no `include_*` flag is ever sent: the pull asks for the endpoint's default \
             games-only set, which is the class `200 []` certifies against: {line}"
        );
        assert!(
            !params.iter().any(|(name, _)| name == "if_modified_since"),
            "a full pull carries no delta cursor: {line}"
        );
    }
    assert!(
        !query_params(&lines[0])
            .iter()
            .any(|(name, _)| name == "last_appid"),
        "the first page has no cursor: {:?}",
        lines[0]
    );
    assert!(
        query_params(&lines[1])
            .iter()
            .any(|(name, value)| name == "last_appid" && value == "730"),
        "the second page continues from the last appid the previous page returned (A3): {:?}",
        lines[1]
    );
}

/// proves: Test strategy #6's delta clause — `if_modified_since` is the loaded
/// snapshot's own `fetched_at`, and nothing else (§The store pull).
#[test]
fn a_delta_pull_asks_for_what_changed_since_the_loaded_stamp() {
    let loaded_at = now_unix() - 4_000;
    let loaded = SteamSnapshot::for_test(
        loaded_at,
        &[(730, "Counter-Strike 2"), (900, "An Older Name")],
    );
    let (address, fake) = loopback(|_request| ok(&page_body(&[(900, "A Newer Name")], false)));
    let store = store_at(address);

    let snapshot = store
        .delta(now_unix(), &loaded)
        .unwrap_or_else(|refusal| panic!("a one-page delta publishes: {refusal}"));
    let lines = fake.request_lines();
    assert_eq!(lines.len(), 1);
    assert!(
        query_params(&lines[0])
            .iter()
            .any(|(name, value)| name == "if_modified_since" && value == &loaded_at.to_string()),
        "`if_modified_since` is the loaded snapshot's `fetched_at`: {:?}",
        lines[0]
    );
    assert_eq!(
        app_ids(&snapshot),
        vec![730, 900],
        "a delta publishes by one linear two-way merge with the loaded vector, preserving \
         `appid` order — never one insert per changed appid, and never a dropped record the \
         store did not mention"
    );
    assert_eq!(
        snapshot.apps()[1].name,
        "A Newer Name",
        "the changed record's name is the store's new one"
    );
}

/// proves: Test strategy #6's `027-L4` clause, mechanically — across a whole
/// two-page pull, the only query parameters the registry ever sends are the four
/// pinned ones: no queried key, user or otherwise, ever appears in a store request.
#[test]
fn a_store_request_carries_only_the_pinned_parameters() {
    let first = page_body(&[(730, "Counter-Strike 2")], true);
    let second = page_body(&[(900, "Outer Wilds")], false);
    let (address, fake) = loopback(move |request| {
        if request.contains("last_appid=") {
            ok(&second)
        } else {
            ok(&first)
        }
    });
    let store = store_at(address);
    store.full(now_unix()).expect("the pull publishes");

    let pinned = ["key", "max_results", "last_appid", "if_modified_since"];
    for raw in fake.raw_requests() {
        for (name, _value) in query_params(&raw) {
            assert!(
                pinned.contains(&name.as_str()),
                "`{name}` is not one of the fixed request lines §The store pull allows; the \
                 only bytes this registry sends a store are those four: {raw:?}"
            );
        }
        for needle in [
            "slug",
            "candidate",
            "query",
            "token",
            "user",
            "local",
            "epic",
        ] {
            assert!(
                !raw.to_ascii_lowercase().contains(needle),
                "a store request mentions `{needle}`, which nothing in the fixed request \
                 lines may: {raw:?}"
            );
        }
    }
}

/// proves: Test strategy #6's response-rule clauses — a non-`200`, a `3xx`, a
/// `Transfer-Encoding`, a body that is not the A17 envelope, an envelope with no
/// `apps` list, and a non-decimal `appid` each fail the page and therefore the pull,
/// by name, and publish nothing.
#[test]
fn every_unhealthy_store_answer_fails_the_pull_by_name() {
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("non-200", b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()),
        ("rate-limited", b"HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()),
        ("redirect", b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:1/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec()),
        (
            "transfer-encoding",
            format!(
                "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{}",
                page_body(&[(730, "Counter-Strike 2")], false)
            )
            .into_bytes(),
        ),
        ("not-json", b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nnope!".to_vec()),
        (
            "wrong-envelope",
            ok(&serde_json::to_string(&serde_json::json!({"response": {"items": []}})).expect("serialises")),
        ),
        (
            "no-apps-list",
            ok(&serde_json::to_string(&serde_json::json!({"have_more_results": false})).expect("serialises")),
        ),
        (
            "non-decimal-appid",
            ok("{\"apps\":[{\"appid\":\"seven\",\"name\":\"X\"}],\"have_more_results\":false}"),
        ),
    ];

    for (label, response) in cases {
        let (address, _fake) = loopback(move |_| response.clone());
        let store = store_at(address);
        let outcome = store.full(now_unix());
        let refusal = match outcome {
            Ok(snapshot) => panic!(
                "`{label}` fails the page and therefore the pull (§The store pull's response \
                 rules); the pull published {} records instead",
                snapshot.apps().len()
            ),
            Err(refusal) => refusal.to_string(),
        };
        assert!(
            !refusal.trim().is_empty(),
            "`{label}` is a *named* refusal, never a silent failure"
        );
        assert!(
            !refusal.to_ascii_lowercase().contains("deadline"),
            "`{label}` is refused on its own condition, not mistaken for the deadline: \
             {refusal}"
        );
    }
}

/// proves: Test strategy #6's over-`MAX_STEAM_PAGE_OCTETS` clause — a well-formed
/// page whose body is over the 12 MiB read budget is the named refusal that fails
/// the pull (§Budgets' page read budget, §Risks' residual).
#[test]
fn an_over_bound_page_fails_the_pull() {
    // §Budgets sizes the 12 MiB read budget *above* what a pinned
    // `STEAM_PAGE_MAX_RESULTS` page of records at `013-B4`'s name bound can return,
    // so the only legal store answer that breaches it is the one §Risks names as the
    // residual: over-`013-B4`, attacker-chosen names, which make a page's true size
    // unbounded. A fixture of ordinary names can never reach the refusal, and a
    // fixture under the bound proves nothing.
    let mut records: Vec<(u64, String)> = Vec::new();
    for index in 0..STEAM_PAGE_MAX_RESULTS as u64 {
        records.push((
            1_000_000 + index,
            // Past `MAX_NATIVE_NAME_OCTETS` on purpose: that is the content the
            // residual is about, and the page-size refusal is decided on the read
            // budget before any normalization is reached.
            format!("{:0<200}", format!("game {index}")),
        ));
    }
    let borrowed: Vec<(u64, &str)> = records
        .iter()
        .map(|(appid, name)| (*appid, name.as_str()))
        .collect();
    let body = page_body(&borrowed, false);
    assert!(
        body.len() > MAX_STEAM_PAGE_OCTETS,
        "the fixture page must be over the read budget: {} octets",
        body.len()
    );

    let (address, fake) = loopback(move |_| ok(&body));
    let store = store_at(address);
    let refusal = store
        .full(now_unix())
        .expect_err("an over-`MAX_STEAM_PAGE_OCTETS` page fails the pull");
    let lower = refusal.to_string().to_ascii_lowercase();
    assert!(
        lower.contains("octet") || lower.contains("bound") || lower.contains("size"),
        "the refusal names the page bound: {refusal}"
    );
    assert!(
        fake.request_count() <= STEAM_PAGE_ATTEMPTS,
        "a page-size refusal is decided on the read budget, not by draining the world: {:?}",
        fake.request_lines()
    );
}

/// proves: Test strategy #6's page-budget clause with §Options' partial-serving
/// refusal — more than `STEAM_REFRESH_MAX_PAGES` pages while `have_more_results` is
/// true fails the pull by name, and a partial worldlist is never published.
#[test]
fn an_endless_listing_exhausts_the_page_budget_and_publishes_nothing() {
    let (address, fake) = loopback(|request| {
        let cursor: u64 = request
            .split("last_appid=")
            .nth(1)
            .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|digits| digits.parse().ok())
            .unwrap_or(0);
        let appid = cursor + 1;
        ok(&page_body(&[(appid, "Endless Game")], true))
    });
    let store = store_at(address);
    let refusal = store
        .full(now_unix())
        .expect_err("a listing that never ends is the page-budget refusal (`027-B4`)");
    let lower = refusal.to_string().to_ascii_lowercase();
    assert!(
        !lower.contains("deadline"),
        "the page-budget refusal is *distinct* from `steam pull deadline` (§The store pull): \
         {refusal}"
    );
    assert!(
        lower.contains("page"),
        "the refusal names the page budget: {refusal}"
    );
    assert!(
        fake.request_count() <= STEAM_REFRESH_MAX_PAGES,
        "at most `STEAM_REFRESH_MAX_PAGES` ({STEAM_REFRESH_MAX_PAGES}) pages are asked for \
         against the pinned {}-record page: {:?}",
        STEAM_PAGE_MAX_RESULTS,
        fake.request_lines()
    );
    assert_eq!(
        fake.request_count(),
        STEAM_REFRESH_MAX_PAGES,
        "the budget is spent before the pull gives up — 12 pages of {} is the 600k-slot \
         world `027-B4` measured against A5's band",
        STEAM_PAGE_MAX_RESULTS
    );
}

/// proves: Test strategy #6's "a partial run never publishes" read against §Options'
/// partial-serving refusal, in the only case where "partial" has content: the pull
/// already has a page in hand when a *later* page fails all its attempts. Half a
/// worldlist is the knowingly incomplete document `027-L3` would certify "never heard
/// of it" against, so a first page bought by a second page's failure publishes nothing.
#[test]
fn a_pull_that_loses_a_later_page_publishes_nothing() {
    let (address, fake) = loopback(move |request| {
        if request.contains("last_appid=") {
            b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_vec()
        } else {
            ok(&page_body(&[(730, "Counter-Strike 2")], true))
        }
    });
    let store = store_at(address);

    let refusal = store
        .full(now_unix())
        .expect_err("a page already in hand never buys a partial publish");
    let lines = fake.request_lines();
    assert!(
        lines.iter().any(|line| line.contains("last_appid=")),
        "the pull got past its first page, so this really is the partial case: {lines:?}"
    );
    assert!(
        !refusal
            .to_string()
            .to_ascii_lowercase()
            .contains("deadline"),
        "the pull is refused on the page's condition, not mistaken for the deadline: \
         {refusal}"
    );
}

/// proves: Test strategy #6's normalization clauses with §The store pull's "owns
/// the rule" — a store name is control-stripped and then clamped to the
/// `013-B4`-legal form, an all-control or all-whitespace title stores empty, and the
/// record and its appid stay in the snapshot: nothing is dropped, because dropping
/// one is a remote, permanent denial of the supply.
#[test]
fn a_store_name_is_normalized_and_the_record_stays() {
    let control_laden = "Helld\u{1b}ivers \u{7f}2".to_string();
    let over_bound = "\u{1}".repeat(40) + &"x".repeat(300);
    let all_control = "\u{0}\u{7}\u{1f}";
    let all_whitespace = "   \t\n ";

    let body = page_body(
        &[
            (1000, &control_laden),
            (1001, &over_bound),
            (1002, all_control),
            (1003, all_whitespace),
        ],
        false,
    );
    let (address, _fake) = loopback(move |_| ok(&body));
    let store = store_at(address);
    let snapshot = store
        .full(now_unix())
        .unwrap_or_else(|refusal| panic!("a bound-breaking name never fails the pull: {refusal}"));

    assert_eq!(
        app_ids(&snapshot),
        vec![1000, 1001, 1002, 1003],
        "every record the pull was handed is in the snapshot: the appid misses stay \
         certifiable against it (`027-L3`)"
    );
    for app in snapshot.apps() {
        assert!(
            !app.name.chars().any(char::is_control),
            "`{}` still carries control bytes: {:?}",
            app.appid,
            app.name
        );
        assert!(
            app.name.len() <= MAX_NATIVE_NAME_OCTETS,
            "`{}`'s name is {} octets, past `013-B4`'s bound: {:?}",
            app.appid,
            app.name.len(),
            app.name
        );
    }
    assert!(
        snapshot.apps()[0].name.contains("ivers"),
        "the strip removes the control characters and keeps the letters: {:?}",
        snapshot.apps()[0].name
    );
    assert_eq!(
        snapshot.apps()[1].name.trim(),
        snapshot.apps()[1].name,
        "the clamp is the shipped `friendly_name` rule, so a stripped-to-nothing label is \
         not left padded: {:?}",
        snapshot.apps()[1].name
    );
    assert!(
        snapshot.apps()[2].name.is_empty() || snapshot.apps()[2].name.trim().is_empty(),
        "an all-control title stores empty: {:?}",
        snapshot.apps()[2].name
    );
    assert!(
        snapshot.apps()[3].name.is_empty() || snapshot.apps()[3].name.trim().is_empty(),
        "an all-whitespace title stores empty: {:?}",
        snapshot.apps()[3].name
    );
}

/// proves: Test strategy #6's retry clauses — a page is attempted
/// `STEAM_PAGE_ATTEMPTS` times and no further, the gap between attempts is the page
/// backoff (never the refresh thread's `STEAM_RETRY_SECONDS`), and a page failing
/// all attempts fails the pull.
#[test]
fn a_failing_page_is_attempted_the_pinned_number_of_times() {
    let (address, fake) = loopback(|_| {
        b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            .to_vec()
    });
    let page_backoff = Duration::from_millis(200);
    let store =
        SteamStore::for_test(address, KEY).with_timing(page_backoff, Duration::from_secs(30));

    let started = std::time::Instant::now();
    let refusal = store
        .full(now_unix())
        .expect_err("a page failing all its attempts fails the pull");
    let elapsed = started.elapsed();

    assert_eq!(
        fake.request_count(),
        STEAM_PAGE_ATTEMPTS,
        "retries stop at `STEAM_PAGE_ATTEMPTS` ({STEAM_PAGE_ATTEMPTS}): {:?}",
        fake.request_lines()
    );
    let two_backoffs = page_backoff * (STEAM_PAGE_ATTEMPTS as u32 - 1);
    assert!(
        elapsed >= two_backoffs / 2 && elapsed < Duration::from_secs(120),
        "the {} gaps between the attempts are spaced by the page backoff \
         ({page_backoff:?}) and never by `STEAM_RETRY_SECONDS` ({RETRY_SECONDS} s), which is \
         the refresh thread's between-pull gap only: the pull took {elapsed:?}",
        STEAM_PAGE_ATTEMPTS - 1
    );
    assert!(
        !refusal
            .to_string()
            .to_ascii_lowercase()
            .contains("deadline"),
        "a page that fails all its attempts is the page refusal, not the deadline: {refusal}"
    );
}

/// proves: Test strategy #6's deadline clause with §Budgets' `STEAM_PULL_DEADLINE_SECONDS`
/// — a stalled store costs a named `steam pull deadline`, distinct from the page
/// refusal, and publishes nothing.
#[test]
fn a_stalled_pull_is_cut_off_by_its_own_deadline() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback stall listener");
    let address = listener.local_addr().expect("the stall address");
    std::thread::spawn(move || {
        // Accepts and answers nothing: the pull's own deadline is the only way out.
        let mut held: Vec<TcpStream> = Vec::new();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => held.push(stream),
                Err(_) => break,
            }
        }
    });

    let store = SteamStore::for_test(address, KEY)
        .with_timing(Duration::from_millis(50), Duration::from_millis(400));
    let started = std::time::Instant::now();
    let refusal = store
        .full(now_unix())
        .expect_err("a pull that cannot publish inside its deadline fails by name");
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(30),
        "the deadline, not a socket timeout grid, is what cuts the pull off: {elapsed:?}"
    );
    assert!(
        refusal
            .to_string()
            .to_ascii_lowercase()
            .contains("deadline"),
        "the refusal is the named `steam pull deadline`: {refusal}"
    );
}

/// proves: Test strategy #6's publish clauses with §The store pull — `fetched_at`
/// is the wall clock read *before the pull's first page*, so a next delta's
/// `if_modified_since` can never sit above appids the pull never saw.
#[test]
fn fetched_at_is_the_clock_read_before_the_first_page() {
    let (address, _fake) = loopback(|_| {
        std::thread::sleep(Duration::from_millis(1_500));
        ok(&page_body(&[(730, "Counter-Strike 2")], false))
    });
    let store = store_at(address);

    let before = now_unix();
    let snapshot = store
        .full(before)
        .unwrap_or_else(|refusal| panic!("a slow page still publishes: {refusal}"));
    assert!(
        snapshot.fetched_at() <= before,
        "`fetched_at` is the clock read before the pull's first page request, never the \
         publish-time stamp (§The store pull): the stamp is {} and the request went out at {before}",
        snapshot.fetched_at()
    );
}

/// proves: Test strategy #6's merge clauses — a full pull *replaces* the snapshot,
/// carrying exactly what the store returned, while a delta merges into the loaded
/// vector without dropping what it did not mention (A3's "changed or added"
/// semantics).
#[test]
fn a_full_pull_replaces_where_a_delta_merges() {
    let loaded =
        SteamSnapshot::for_test(now_unix() - 100, &[(730, "Counter-Strike 2"), (900, "Old")]);

    let (replace_address, _fake) = loopback(|_| ok(&page_body(&[(500, "Brand New")], false)));
    let replaced = store_at(replace_address)
        .full(now_unix())
        .expect("a full pull publishes");
    assert_eq!(
        app_ids(&replaced),
        vec![500],
        "a full pull replaces the snapshot rather than adding to it"
    );

    let (delta_address, _fake) = loopback(|_| ok(&page_body(&[(1_000, "Added")], false)));
    let merged = store_at(delta_address)
        .delta(now_unix(), &loaded)
        .expect("a delta publishes");
    assert_eq!(
        app_ids(&merged),
        vec![730, 900, 1_000],
        "a delta never sees removals (A3), so the loaded records stay and the added one \
         lands in `appid` order"
    );
}

/// proves: Test strategy #6's last clause with §The store pull's key-custody rule —
/// no failure line ever carries the key or the request URI.
#[test]
fn no_pull_failure_line_carries_the_key_or_the_request_uri() {
    let cases: Vec<(&str, Vec<u8>)> = vec![
        (
            "http-500",
            b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_vec(),
        ),
        ("bad-envelope", ok("{\"unexpected\":true}")),
    ];
    for (label, response) in cases {
        let (address, _fake) = loopback(move |_| response.clone());
        let store = store_at(address);
        let refusal = store
            .full(now_unix())
            .err()
            .unwrap_or_else(|| panic!("`{label}` fails the pull"));
        let text = refusal.to_string();
        assert!(
            !text.contains(KEY),
            "the `{label}` refusal leaks the Web API key (A14, CWE-319/526): {text}"
        );
        assert!(
            !text.contains("http://") && !text.contains("https://"),
            "the `{label}` refusal carries a request URI; every pull-failure line names the \
             condition and status, never the URL: {text}"
        );
        assert!(
            !text.contains("GetAppList"),
            "the `{label}` refusal carries the request path: {text}"
        );
    }
}

/// proves: Test strategy #6's transport-assurance pair — the *production*
/// transport (webpki roots only), dialed at the test address with the
/// verification name a separate parameter, against a loopback rustls listener
/// presenting a self-signed certificate: one for the very name being verified
/// (whose handshake must die on chain-building, not on the hostname), one for
/// another name. The fake watches the *decrypted* stream, so a verifier
/// relaxed to accept an untrusted chain would complete the handshake and see
/// the real `GET` with the key delivered to it — the empty-decrypted-view
/// assertion is the mutation catcher this clause exists to have.
#[test]
fn the_production_transport_refuses_self_signed_store_certificates() {
    assert_eq!(
        STORE_AUTHORITY, "partner.steam-api.com",
        "`027-C3` pins the store at the partner host; there is no operator-configurable \
         store URL (§Options' plaintext rejection)"
    );

    for verify_name in [STORE_AUTHORITY, "some-other-name.example"] {
        let fixture = |suffix: &str| {
            let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("tests/fixtures/store");
            path.push(format!("{verify_name}.{suffix}"));
            path
        };
        let certificate = std::fs::read(fixture("crt.der")).expect("the fixture certificate");
        let key = std::fs::read(fixture("key.pkcs8.der")).expect("the fixture key");
        let config = std::sync::Arc::new(
            rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(
                    vec![rustls::pki_types::CertificateDer::from(certificate)],
                    rustls::pki_types::PrivateKeyDer::try_from(key)
                        .expect("the fixture key is a PKCS#8 DER"),
                )
                .expect("a server config for the fixture certificate"),
        );

        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
        let address = listener.local_addr().expect("the listener address");
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        {
            let seen = std::sync::Arc::clone(&seen);
            std::thread::spawn(move || {
                // The bind above is the ordering: connections to a bound,
                // listening socket queue regardless of when this accepts.
                // Every dial gets its answer — the pull may retry pages, and
                // a fixture that accepts once and stops would strand the
                // retry in a backlog that answers nothing.
                let mut held: Vec<std::thread::JoinHandle<()>> = Vec::new();
                for stream in listener.incoming() {
                    let Ok(stream) = stream else { break };
                    // Bound every read so an abandoned handshake cannot
                    // strand a thread on the fixture forever.
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    let config = std::sync::Arc::clone(&config);
                    let seen = std::sync::Arc::clone(&seen);
                    held.push(std::thread::spawn(move || {
                        serve_tls_fixture(stream, config, seen);
                    }));
                }
            });
        }
        let store = SteamStore::for_test(address, KEY)
            .with_timing(Duration::from_millis(20), Duration::from_secs(5))
            .with_production_transport(verify_name);
        let refusal = store.full(now_unix()).expect_err(
            "a self-signed certificate is refused against the production roots, in both \
             name cases: a verifier relaxed to accept the fake would pass the whole \
             suite with the key on an unauthenticated link",
        );
        let text = refusal.to_string();
        assert!(
            !text.contains(KEY),
            "the handshake refusal carries the key: {text}"
        );
        if verify_name == STORE_AUTHORITY {
            // The very name being verified must fail on the *chain* — the
            // hostname fits. That is the property the self-signed-for-the-
            // verified-name fixture buys: a refusal by name instead would
            // leave a chain-only relaxation invisible.
            assert!(
                text.contains("UnknownIssuer"),
                "the handshake for a certificate of the very verified name must refuse on \
                 chain-building (UnknownIssuer), not on the name: {text}"
            );
        }

        // The mutation catcher: if a handshake ever completes here, its
        // decrypted stream *is* the request. Nothing may reach the fake.
        std::thread::sleep(Duration::from_millis(150));
        let decrypted = seen.lock().expect("record").clone();
        assert!(
            decrypted.is_empty(),
            "the self-signed fake decrypted {} bytes of request — a handshake completed \
             on an untrusted chain (`{verify_name}`): {:?}",
            decrypted.len(),
            String::from_utf8_lossy(&decrypted)
        );
    }
}

/// Answers one connection as a rustls server with the given prebuilt config,
/// appending every decrypted byte it reads to `seen`, and returns when the
/// peer errors, times out, or closes.
fn serve_tls_fixture(
    stream: TcpStream,
    config: std::sync::Arc<rustls::ServerConfig>,
    seen: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
) {
    let Ok(connection) = rustls::ServerConnection::new(config) else {
        return;
    };
    let mut io = rustls::StreamOwned::new(connection, stream);
    // The started io pumps both directions over the blocking socket: peer
    // records in, handshake flights out, failures as errors. The client's
    // chain check happens on *its* side; what arrives here is an alert or the
    // plaintext.
    let mut buffer = [0_u8; 4096];
    loop {
        match std::io::Read::read(&mut io, &mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => seen
                .lock()
                .expect("record")
                .extend_from_slice(&buffer[..read]),
        }
    }
}

/// proves: Test strategy #6's separate-names clause — the connect address and the
/// name verified against it are distinct parameters, so the fake can be dialed at
/// `127.0.0.1` while the client verifies the authority (#17's no-outbound rule).
#[test]
fn the_verify_name_is_independent_of_the_connect_address() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
    let address = listener.local_addr().expect("the listener address");
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    {
        let seen = std::sync::Arc::clone(&seen);
        std::thread::spawn(move || {
            let mut held: Vec<TcpStream> = Vec::new();
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut buffer = [0_u8; 4_096];
                        if let Ok(read) = std::io::Read::read(&mut stream, &mut buffer) {
                            seen.lock()
                                .expect("record")
                                .extend_from_slice(&buffer[..read]);
                        }
                        held.push(stream);
                    }
                    Err(_) => break,
                }
            }
        });
    }

    let store = SteamStore::for_test(address, KEY)
        .with_timing(Duration::from_millis(20), Duration::from_secs(5))
        .with_production_transport(STORE_AUTHORITY);
    let outcome = store.full(now_unix());
    assert!(
        outcome.is_err(),
        "nothing is served over an unverified link"
    );

    std::thread::sleep(Duration::from_millis(50));
    let bytes = seen.lock().expect("record").clone();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    assert!(
        text.contains(STORE_AUTHORITY),
        "the client dialed `127.0.0.1:{}` but verifies the pinned authority — that name is \
        what its handshake carries, and its absence would mean the seam ignores the name: \
        {text:?}",
        address.port()
    );
    let address_spelling = format!("127.0.0.1:{}", address.port());
    assert!(
        !text.contains(address_spelling.as_str()),
        "the client put the *connect address* in the position the authority belongs (`Host`/SNI): \
         {text:?}"
    );
}

// ---------------------------------------------------------------------------
// Test strategy #11 — the refresh thread's decision, with no sleeps
// ---------------------------------------------------------------------------

/// proves: Test strategy #11's first clause with `027-B2` — a published snapshot is
/// due for its next delta exactly `STEAM_SNAPSHOT_REFRESH_SECONDS` later.
#[test]
fn a_published_snapshot_is_due_again_after_the_refresh_interval() {
    let published_at = 1_800_000_000_u64;
    assert_eq!(
        next_refresh_seconds(published_at, published_at, PullOutcome::Published),
        REFRESH_SECONDS,
        "the refresh cadence is `STEAM_SNAPSHOT_REFRESH_SECONDS` ({REFRESH_SECONDS})"
    );
    assert_eq!(
        next_refresh_seconds(published_at + 10, published_at, PullOutcome::Published),
        REFRESH_SECONDS - 10,
        "the decision is a function of the clock, not of a counter"
    );
    assert_eq!(
        next_refresh_seconds(
            published_at + REFRESH_SECONDS,
            published_at,
            PullOutcome::Published
        ),
        0,
        "at the interval the pull is due now"
    );
    assert_eq!(
        next_refresh_seconds(
            published_at + REFRESH_SECONDS + 5_000,
            published_at,
            PullOutcome::Published
        ),
        0,
        "and overdue means due now, never a negative wait"
    );
}

/// proves: Test strategy #11's second clause with §The store pull — a failed pull
/// is retried after `STEAM_RETRY_SECONDS`, the only consumer of that constant, while
/// the old snapshot stays published until `027-B2`'s age bound converts its misses
/// to `503`.
#[test]
fn a_failed_pull_retries_at_the_between_pull_gap() {
    let attempted_at = 1_800_000_000_u64;
    assert_eq!(
        next_refresh_seconds(attempted_at, attempted_at, PullOutcome::Failed),
        RETRY_SECONDS,
        "a failed pull keeps the old snapshot and retries after `STEAM_RETRY_SECONDS` \
         ({RETRY_SECONDS}), not after the full refresh interval"
    );
    assert_eq!(
        next_refresh_seconds(
            attempted_at + RETRY_SECONDS,
            attempted_at,
            PullOutcome::Failed
        ),
        0,
        "and the retry is due when the gap has passed, until it succeeds"
    );
    assert_ne!(
        next_refresh_seconds(attempted_at, attempted_at, PullOutcome::Failed),
        next_refresh_seconds(attempted_at, attempted_at, PullOutcome::Published),
        "failure and success have different tempos — one retry constant for two tempos is \
         the split §Revision log r1→r2 records"
    );
}

/// proves: §The store pull's refresh-thread rule read against `027-B2` — a failure
/// gap that outlives the age bound leaves the pull overdue: the thread keeps
/// retrying, and the age bound is what the serving face reads, not the scheduler.
#[test]
fn a_long_failure_streak_leaves_the_pull_overdue_rather_than_abandoned() {
    let last_published = 1_800_000_000_u64;
    let much_later = last_published + MAX_AGE_SECONDS * 2;
    assert_eq!(
        next_refresh_seconds(much_later, last_published, PullOutcome::Failed),
        0,
        "a thread whose last attempt failed long ago pulls again immediately: nothing here \
         gives up on the supply"
    );
}
