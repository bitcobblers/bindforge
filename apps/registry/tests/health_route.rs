//! `GET /v1/health` — always a `200`, never a readiness gate, and never gated.
//!
//! # What ADR-027 retired out of this file
//!
//! Design 026's Test strategy #12 pinned health as the reporting surface of the
//! caching mode: `upstream: down`, `last_upstream_fetch: 0` on a fresh caching
//! instance, and "a health read never fires the TTL probe". ADR-027 retires the
//! caching mode, and §Hosted mode names the three fields it takes with it —
//! `upstream`, `cached_slugs`, `last_upstream_fetch` — so those legs went with the
//! `--upstream` flag they described (A15: this file is one of the suites that
//! re-pins). The *field list* re-pin is `hosted_health_stats.rs`'s Test strategy #9
//! (exactly `{status, mode: "hosted", corpus_entries, steam_snapshot}`, the
//! `{games, fetched_at}` and `"none"` spellings, none of the three retired facts);
//! it is not restated here.
//!
//! What survives this file is the route's **identity**, which the design carries
//! across the mode change: "it never answers non-`200`, so a degraded supply is
//! reported in the body, not in the code" (§Chosen design, and the reason
//! `steam_snapshot.fetched_at` is named as the retired `upstream` verdict's
//! successor). Re-pinned here, that claim is about hosted mode's own degraded
//! postures — Test strategy #13's stale snapshot and #8's `--no-steam` — plus
//! `026-C5`'s admin surface standing unchanged for a hosted instance's health read.
//!
//! The two degraded legs take different routes, because the design leaves them
//! different routes. `--no-steam` is a posture a hosted process really does boot in,
//! so its health read is a socket read. A *stale* snapshot is not: §The steam snapshot
//! resolver's startup order loads a snapshot only when it is "present, loadable, and
//! fresh, else pull in full", so no instance can be planted on an aging one by a test
//! that may not dial the compile-time-pinned authority (#17), and the only instance
//! that serves an aging snapshot is one whose refresh thread has failed — a thread
//! with no loopback seam in the binary either. That leg reads `state.rs`'s own two
//! faces, the health body and the policy, over one loaded snapshot. The `state` face
//! is the crate's library target, which is also what `tests/steam.rs` and the cold
//! rows of `tests/hosted_lookup_policy.rs` drive for the same reason.
//!
//! One retired clause has **no route** in hosted mode and is recorded rather than
//! papered over: "a health read never fires the probe". The resolver's pulls belong
//! to the startup order and the refresh thread, and the binary has no seam that
//! points the store at a loopback fake, so a subprocess leg cannot observe a pull
//! without dialling the real authority — which Test strategy #17 forbids. The
//! pull's own assurance half is pinned in-process, against a loopback fake
//! transport, in `tests/steam.rs`.

mod common;

use common::{Registry, TempDir, corpus_doc, json_body, now_unix, status_code, status_line};
use std::path::Path;

const KEY: &str = "bindforge-test-steam-key-9a8b7c6d";
const KEY_VAR: &str = "BINDFORGE_STEAM_WEB_API_KEY";

/// `027-B2`'s age bound: freshness is `now − fetched_at ≤` this, and past it a
/// snapshot may certify nothing (`027-L3`).
const MAX_AGE_SECONDS: u64 = 604_800;

fn xdg_of(data: &TempDir) -> String {
    data.path()
        .to_str()
        .expect("a utf-8 temp data directory")
        .to_string()
}

fn corpus_in(data: &TempDir, games: &[(&str, &str, &[&str])]) -> std::path::PathBuf {
    data.write("corpus.json", corpus_doc(3, games).as_bytes())
}

fn serve(data: &TempDir, corpus: &Path, extra: &[&str]) -> Registry {
    let xdg = xdg_of(data);
    let mut args: Vec<String> = vec![
        "serve".to_string(),
        "--root".to_string(),
        "--corpus".to_string(),
        corpus.display().to_string(),
    ];
    args.extend(extra.iter().map(|arg| (*arg).to_string()));
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    Registry::start(
        "health",
        &borrowed,
        &[("XDG_DATA_HOME", xdg.as_str()), (KEY_VAR, KEY)],
    )
}

/// proves: §Chosen design's "it never answers non-`200`" carried into hosted mode,
/// read with §Hosted mode's "`steam_snapshot.fetched_at` is their successor" — the
/// degraded supply is a body fact. Here the degraded supply is the one a hosted
/// instance can actually be caught in from outside: `--no-steam`, the operator's
/// degraded option (§The steam snapshot resolver). `/v1/health` stays a `200` that
/// says `"none"` while the very same instance refuses the steam keys the missing
/// supply makes uncertifiable. The bound CLI fails closed on that `503`, which is why
/// the design makes the health document the operator's availability dial (§Compatibility
/// matrix, §Risks): a readiness-flavoured code here would replace that dial with a
/// load balancer's blind retry. The stale-snapshot half of the same posture — the body
/// that carries an age the policy will not certify against — is
/// `a_stale_snapshot_shows_its_age_in_the_health_body_never_in_a_code`, because no
/// booting instance is ever planted on a stale snapshot.
#[test]
fn health_stays_a_200_that_reports_the_supply_it_is_refusing_on() {
    let data = TempDir::new("health-degraded");
    let corpus = corpus_in(
        &data,
        &[("curated-name", "Curated Name", &["steam/900001"][..])],
    );
    let server = serve(&data, &corpus, &["--no-steam"]);

    let miss = server.get("/v1/candidates?slug=steam%2F9199998");
    assert_eq!(
        status_code(&miss),
        Some(503),
        "the instance really is in the degraded posture this test is about: {}",
        status_line(&miss)
    );

    let answer = server.health();
    assert_eq!(
        status_line(&answer),
        "HTTP/1.1 200 OK",
        "health is never a readiness gate: the degraded supply is reported in the body, \
         and the `503` stays on the lookup route that earned it"
    );
    let document = json_body(&answer);
    assert_eq!(
        document["mode"].as_str(),
        Some("hosted"),
        "`027-C1`'s one mode: {document}"
    );
    assert_eq!(
        document["steam_snapshot"].as_str(),
        Some("none"),
        "the missing supply is a body fact, spelled the way §Hosted mode spells it: \
         {document}"
    );
    assert!(
        document["status"].is_string(),
        "`status` keeps the shipped health document's own spelling: {document}"
    );
    for retired in ["upstream", "cached_slugs", "last_upstream_fetch"] {
        assert!(
            document[retired].is_null(),
            "`{retired}` retired with the mode and must not reappear on the degraded \
             path, which is the one path it used to describe: {document}"
        );
    }
}

/// proves: the same never-a-readiness-gate identity on the *stale* row of Test
/// strategy #13 — a snapshot whose `fetched_at` is past `027-B2`'s age bound keeps
/// reporting that stamp verbatim in the health body, `games` and all, while the
/// policy refuses every steam lookup on it. §The steam snapshot resolver's startup
/// order ("load the snapshot if present, loadable, **and** fresh, else pull in full")
/// means a hosted process is never planted on a stale snapshot — the boot would pull
/// the compile-time-pinned authority, which Test strategy #17 rules out — and §The
/// store pull keeps an aging snapshot in service only after a refresh that has no
/// loopback seam in the binary. So this is `state.rs`'s own two faces, driven from the
/// loaded models: the body the operator reads and the policy that reads the same stamp.
#[test]
fn a_stale_snapshot_shows_its_age_in_the_health_body_never_in_a_code() {
    use bindforge_registry::constants::MAX_CATALOG_DOCUMENT_OCTETS;
    use bindforge_registry::corpus::Catalog;
    use bindforge_registry::retired::RetiredIds;
    use bindforge_registry::state::{Answer, State, SteamPosture};
    use bindforge_registry::steam::SteamSnapshot;

    let data = TempDir::new("health-stale");
    let corpus = data.write(
        "health-corpus.json",
        corpus_doc(
            3,
            &[("curated-name", "Curated Name", &["steam/900001"][..])],
        )
        .as_bytes(),
    );
    let issued = data.join("health-issued.json");
    let stale = now_unix().saturating_sub(MAX_AGE_SECONDS + 3_600);
    let state = State::hosted(
        Catalog::load(&corpus, MAX_CATALOG_DOCUMENT_OCTETS).expect("the corpus fixture loads"),
        Catalog::empty(issued),
        RetiredIds::load(&data.join("health-retired-ids.json"))
            .expect("an absent retired list is the empty set"),
        Some(SteamSnapshot::for_test(
            stale,
            &[(9_100_801_u64, "Aging Title")],
        )),
        SteamPosture::Active,
    );

    let document: serde_json::Value = state
        .health()
        .into_iter()
        .map(|(field, value)| (field.to_string(), value))
        .collect();
    assert_eq!(
        document["mode"].as_str(),
        Some("hosted"),
        "the degraded instance is still the one mode: {document}"
    );
    assert_eq!(
        document["steam_snapshot"]["fetched_at"].as_u64(),
        Some(stale),
        "`steam_snapshot.fetched_at` is the retired `upstream` verdict's successor, and it \
         reports the stale stamp verbatim rather than hiding it: {document}"
    );
    assert_eq!(
        document["steam_snapshot"]["games"].as_u64(),
        Some(1),
        "the count is the loaded snapshot's own: {document}"
    );
    assert!(
        document["status"].is_string(),
        "health's own `status` word does not degrade into a refusal code: {document}"
    );
    for retired in ["upstream", "cached_slugs", "last_upstream_fetch"] {
        assert!(
            document[retired].is_null(),
            "`{retired}` retired with the mode and must not reappear on the degraded path: \
             {document}"
        );
    }

    let refusal = |answer: &Answer| match answer {
        Answer::Refusal(named) => (named.reason(), named.code()),
        Answer::Candidates {
            candidates,
            provenance,
        } => panic!(
            "an aging snapshot certifies nothing; the policy answered {provenance:?} with \
             {candidates:?}"
        ),
    };
    let miss = state.candidates("steam/9199998", now_unix());
    assert_eq!(
        refusal(&miss).0,
        "steam snapshot stale",
        "the age the body shows is the age the policy refuses on"
    );
    let known = state.candidates("steam/9100801", now_unix());
    assert_eq!(
        refusal(&known).1,
        "steam_snapshot_stale",
        "and the refusal carries its own name, never an empty list"
    );
}

/// proves: the never-gated half of the route's identity, re-pinned against a hosted
/// instance (§Hosted mode retires nothing of `026-C5`'s admin posture, and Test
/// strategy #9 enumerates the health fields as public): with a token configured and
/// a bind that disables the admin face, the admin route still refuses an
/// uncredentialled call while `/v1/health` answers `200` with the resolver-inactive
/// `"none"` spelling (#8) — the two halves pinned side by side so "never gated"
/// cannot quietly become "nothing is gated".
#[test]
fn health_is_never_gated_while_the_admin_face_keeps_its_requirement() {
    let data = TempDir::new("health-ungated");
    let corpus = corpus_in(
        &data,
        &[("example-game", "Example Game", &["steam/900002"][..])],
    );
    let server = serve(
        &data,
        &corpus,
        &[
            "--no-steam",
            "--host",
            "0.0.0.0",
            "--admin-token",
            "s3cret-health-token",
        ],
    );

    let answer = server.health();
    assert_eq!(
        status_code(&answer),
        Some(200),
        "no credential is asked of `/v1/health`, whatever the bind: {}",
        status_line(&answer)
    );
    let document = json_body(&answer);
    assert_eq!(
        document["steam_snapshot"].as_str(),
        Some("none"),
        "the resolver-inactive instance spells its supply `none` in the body (#8): {document}"
    );

    let uncredentialled = server.request("GET", "/v1/admin/stats", &[], "");
    assert_eq!(
        status_code(&uncredentialled),
        Some(401),
        "the admin route beside it still requires the token, so this file pins that \
         health is ungated rather than that nothing is: {}",
        status_line(&uncredentialled)
    );
}
