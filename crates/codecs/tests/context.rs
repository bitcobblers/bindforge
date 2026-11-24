//! `GameContext::from_record` — the gate that turns a discovery row into a
//! codec context (`017-C2`, §GameContext).
//!
//! proves: Test strategy #2.

mod common;

use bindforge_codecs::context::GameContext;
use discovery::game::{DiscoveredGame, Platform, ScanRecord};
use std::path::PathBuf;

fn record(slug: &str, library: &str) -> ScanRecord {
    ScanRecord {
        game: DiscoveredGame {
            slug: slug.to_string(),
            friendly_name: "Example Game".to_string(),
            display_name: "Example Game".to_string(),
            install_path: PathBuf::from(library)
                .join("steamapps")
                .join("common")
                .join("Example Game"),
            platform: Platform::SteamLinux,
            codec_id: None,
        },
        root: PathBuf::from("/steam/root"),
        library: PathBuf::from(library),
    }
}

// proves: Test strategy #2 (§GameContext) — `steam/900001` yields the digit
// appid, the lookup key byte-for-byte, every resolved root in the byte order
// handed in, and the *owning* library from the record.
#[test]
fn a_steam_row_with_a_digit_appid_yields_the_full_context() {
    let roots = vec![
        PathBuf::from("/steam/root-a"),
        PathBuf::from("/steam/root-b"),
    ];
    let ctx = GameContext::from_record(&record("steam/900001", "/steam/root-a/lib-2"), &roots)
        .expect("a steam row with a digit appid must produce a context");
    assert_eq!(
        ctx.lookup_key, "steam/900001",
        "the lookup key is the row's slug verbatim"
    );
    assert_eq!(ctx.appid, "900001", "the appid is the external id");
    assert_eq!(
        ctx.roots, roots,
        "017-C2's whole resolved root set rides through unchanged, in order"
    );
    assert_eq!(
        ctx.library,
        PathBuf::from("/steam/root-a/lib-2"),
        "the library is the owning one"
    );
    assert_eq!(ctx.platform, Platform::SteamLinux);
    assert_eq!(
        ctx.install_path,
        PathBuf::from("/steam/root-a/lib-2/steamapps/common/Example Game")
    );
}

// proves: Test strategy #2 (§GameContext: "`None` unless the slug is a `steam`
// lookup key with a digit appid") — a `local/x` key, a `steam/` empty id, a
// non-digit appid and a 21-digit appid each yield `None`; the row is never
// half-parsed.
#[test]
fn rows_outside_the_steam_digit_shape_yield_no_context() {
    let roots = [PathBuf::from("/steam/root")];
    for slug in [
        "local/x",                     // not a steam row
        "epic/900001",                 // the other closed source token
        "steam/",                      // empty external id
        "steam/900o1",                 // non-digit appid
        "steam/900001000000000000000", // 21 digits: past the 20-digit appid bound
    ] {
        assert!(
            GameContext::from_record(&record(slug, "/steam/root"), &roots).is_none(),
            "`{slug}` must yield no context — the face answers claim-refused, never a half-parsed identity"
        );
    }
}

// proves: Test strategy #2 (§GameContext: "1..=20 ASCII digits", mirroring the
// scan-seam appid rule) — the accepted extremes of the appid bound.
#[test]
fn the_appid_bound_accepts_one_to_twenty_digits() {
    let roots = [PathBuf::from("/steam/root")];
    assert!(
        GameContext::from_record(&record("steam/1", "/steam/root"), &roots).is_some(),
        "a single-digit appid is inside the bound"
    );
    assert!(
        GameContext::from_record(&record("steam/12345678901234567890", "/steam/root"), &roots)
            .is_some(),
        "a 20-digit appid is the accepted extreme"
    );
}
