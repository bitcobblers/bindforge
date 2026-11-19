//! The public surface ADR-025 gives `crates/discovery`: `Source` and its closed
//! token spelling, the one lookup-key constructor, the `friendly_name` normalizer,
//! and the row's new field set — plus the lookup-key bound this run adds beside the
//! shared slug bound at the `bindforge-core` root.
//!
//! proves: design 025's Test strategy #14 (Chosen design: Bound — "`crates/
//! discovery/tests/zz_api_probe.rs` is re-pinned to `Source`, `lookup_key`, and the
//! new field set") — this target resolves every promised name in type or call
//! position, so a rename of `Source` or `lookup_key`, or a field moved out of
//! `DiscoveredGame`, is a compile failure here rather than a silent break for
//! `apps/discovery-cli`, the one in-workspace consumer of the row.
//!
//! The `024` surface retires with this probe's re-pin: `derive_slug`, the
//! `discovery::slug` module, and the `Namespace` enum resolve nowhere (Test
//! strategy #14, Rollout 1), so no name here asks for them; the retirement sweep is
//! `retirement_tests.rs`. The behavior behind each name belongs to `game_tests.rs`
//! (#1, #2) and `emission_tests.rs` (#3, #4); this file adds no rule of its own.

use bindforge_core::{MAX_DISCOVERY_SLUG_OCTETS, is_slug_name};
use discovery::game::{DiscoveredGame, Platform, Source, friendly_name};
use std::path::PathBuf;

/// `Source::as_str` in signature position: the closed token set, spelled.
fn spelled(source: &Source) -> &'static str {
    source.as_str()
}

/// `Source::lookup_key` in signature position: the one place the
/// `<source>/<external-id>` spelling exists (`025-C1`).
fn lookup_key(source: &Source, external_id: &str) -> String {
    source.lookup_key(external_id)
}

/// `friendly_name` in signature position: the only remaining normalization in this
/// crate (`025-C2`).
fn normalized(display_name: &str) -> String {
    friendly_name(display_name)
}

/// proves: design 025's Test strategy #1 + #14 (Chosen design: Row identity) —
/// `Source` resolves as the closed three-variant enum with the `as_str` spelling,
/// `lookup_key` resolves in call position answering `String`, `friendly_name`
/// resolves in call position answering `String`, and a `DiscoveredGame` literal built
/// with `friendly_name` (and no `namespace`) compiles and reads back.
#[test]
fn the_adr_025_discovery_names_resolve_at_the_crate_root() {
    // `Source`, in type and call position: the three shipped tokens, in the closed
    // set `steam · epic · local` (`025-C1`).
    let sources = [Source::Steam, Source::Epic, Source::Local];
    let spellings: Vec<&'static str> = sources.iter().map(spelled).collect();
    assert_eq!(
        spellings,
        ["steam", "epic", "local"],
        "`as_str` spells the closed token set and nothing else"
    );

    // `lookup_key`, in call position: `steam` + `900001` compose as `steam/900001`.
    assert_eq!(lookup_key(&Source::Steam, "900001"), "steam/900001");

    // `friendly_name`, in call position: whitespace normalization, not derivation.
    assert_eq!(normalized("  Fine   Game "), "Fine Game");

    // `DiscoveredGame`'s field set, in field position: `friendly_name` after `slug`,
    // no `namespace`, and everything else the shipped row carried still there.
    let game = DiscoveredGame {
        slug: lookup_key(&Source::Steam, "900001"),
        friendly_name: normalized("Example Game"),
        display_name: "Example Game".to_string(),
        install_path: PathBuf::from("/steamapps/common/Example Game"),
        platform: Platform::SteamLinux,
        codec_id: None,
    };
    assert_eq!(game.slug, "steam/900001", "`slug` is still a String field");
    assert_eq!(
        game.friendly_name, "Example Game",
        "`friendly_name` is a String field of the row, sitting after `slug`"
    );
    assert_eq!(game.display_name, "Example Game");
    assert_eq!(
        game.install_path,
        PathBuf::from("/steamapps/common/Example Game"),
        "`install_path` keeps the type the shipped row used"
    );
    assert_eq!(game.platform, Platform::SteamLinux);
    assert!(
        game.codec_id.is_none(),
        "`codec_id` stays the nullable field it was"
    );
}

/// proves: design 025's Test strategy #14 (Chosen design: Bound — "`lib.rs`
/// re-exports `MAX_DISCOVERY_SLUG_OCTETS` as the thirteenth exported constant") —
/// the lookup-key bound resolves at the `bindforge-core` root at the value `025-B1`
/// states, equal to the two sibling bounds so the family reads as one, and it is the
/// bound a composed lookup key rides — explicitly not the grammar, from which the key
/// is exempt by design.
#[test]
fn the_shared_discovery_bound_resolves_at_the_bindforge_core_root() {
    assert_eq!(
        MAX_DISCOVERY_SLUG_OCTETS, 128,
        "`025-B1`'s cap is set equal to the two 128 bounds so the family reads as one"
    );
    assert_eq!(
        MAX_DISCOVERY_SLUG_OCTETS,
        bindforge_core::MAX_SLUG_OCTETS,
        "the lookup-key bound is the sibling of the shared slug bound it stands beside"
    );

    let key = Source::Steam.lookup_key("900001");
    assert!(
        key.len() <= MAX_DISCOVERY_SLUG_OCTETS,
        "`{key}` is inside the only bound the emit boundary checks (`025-L3`)"
    );
    assert!(
        !is_slug_name(&key),
        "`{key}` fails the shared grammar and rides the discovery bound instead — the \
         exemption `025-B1` states, replacing the `024-B1` rider check at this seam"
    );
}
