//! The module-mirrored suite for `crates/discovery/src/game.rs` (ADR-004): the row a
//! plugin emits, and the identity half ADR-025 gives it — `Source` with its closed
//! token set, the one lookup-key constructor `lookup_key`, and `friendly_name` as the
//! row's display metadata.
//!
//! proves: design 025's Test strategy #1 — the three tokens spell `steam`, `epic`,
//! `local`; `lookup_key` emits exactly one `/` and composes `steam` + `900001` as
//! `steam/900001`; and the row's JSON carries no `source` and no `namespace` key,
//! because the row carries the composed key, never the token (`025-C1`: `Source` is
//! construction-side only, and the `Namespace` enum and its serde spelling retire with
//! `024-C1`).
//!
//! proves: design 025's Test strategy #2 — `friendly_name` trims, collapses interior
//! whitespace runs to one space, and clamps a multi-byte name to the longest
//! `char`-boundary prefix inside `MAX_NATIVE_NAME_OCTETS`; a 128-octet name is
//! untouched and a 129-octet name loses one octet without panicking or splitting a
//! `char` (`025-C2`: display metadata that joins nothing, which is what makes the
//! clamp admissible here and only here).
//!
//! Unit-level: pure functions and serde over in-memory values, no filesystem, no
//! network (repo practice).

use bindforge_core::{MAX_DISCOVERY_SLUG_OCTETS, MAX_NATIVE_NAME_OCTETS, is_slug_name};
use discovery::game::{DiscoveredGame, Platform, Source, friendly_name};
use std::path::PathBuf;

/// A row in the shape ADR-025 defines: the composed lookup key, the friendly name
/// beside it, and the fields the shipped row already carried. The slug is built
/// through `lookup_key`, the one place the `<source>/<external-id>` spelling exists.
fn row() -> DiscoveredGame {
    DiscoveredGame {
        slug: Source::Steam.lookup_key("900001"),
        friendly_name: friendly_name("Example Game"),
        display_name: "Example Game".to_string(),
        install_path: PathBuf::from("/steamapps/common/Example Game"),
        platform: Platform::SteamLinux,
        codec_id: None,
    }
}

/// proves: design 025's Test strategy #1 (Chosen design: Row identity) — `as_str`
/// spells the closed token set `steam · epic · local`; `epic` and `local` are declared
/// and unpopulated (`025-R2`, no local backend), but the tokens are part of the row
/// surface this run ships.
#[test]
fn the_three_source_tokens_spell_the_closed_set() {
    assert_eq!(Source::Steam.as_str(), "steam");
    assert_eq!(Source::Epic.as_str(), "epic");
    assert_eq!(Source::Local.as_str(), "local");
}

/// proves: design 025's Test strategy #1 (Chosen design: Row identity) —
/// `lookup_key` is the one place the spelling exists: it joins source token and
/// external id with exactly one `/`, composes `steam` + `900001` as `steam/900001`,
/// and the key it builds is exempt from the product slug grammar by design (the
/// retired `024-C1` adoption promise; `025-L5` keeps it out of every id and path).
#[test]
fn lookup_key_composes_source_and_external_id_with_exactly_one_separator() {
    assert_eq!(
        Source::Steam.lookup_key("900001"),
        "steam/900001",
        "`steam` + `900001` compose as the Steam lookup key ADR-025 names"
    );
    assert_eq!(Source::Epic.lookup_key("abc"), "epic/abc");
    assert_eq!(Source::Local.lookup_key("some-game"), "local/some-game");

    let composed = Source::Steam.lookup_key("900001");
    assert_eq!(
        composed.matches('/').count(),
        1,
        "the spelling emits exactly one `/`: `{composed}`"
    );
    assert!(
        composed.len() <= MAX_DISCOVERY_SLUG_OCTETS,
        "`{composed}` is inside `025-B1`'s lookup-key bound"
    );
    assert!(
        !is_slug_name(&composed),
        "`{composed}` fails the shared product-slug grammar — by design: the source \
         key joins nothing and never becomes a product id (`025-L5`)"
    );
}

/// proves: design 025's Test strategy #1 (Chosen design: Row identity) — the row's
/// compact serialisation is the six-field shape `slug`, `friendly_name`,
/// `display_name`, `install_path`, `platform`, `codec_id`: `friendly_name` sits after
/// `slug`, and there is no `source` and no `namespace` key anywhere in the row JSON —
/// `Source` rides no serde and the `Namespace` enum is deleted.
#[test]
fn a_discovered_game_serialises_friendly_name_after_slug_and_no_source_or_namespace() {
    let game = row();
    let json = serde_json::to_string(&game).expect("a row serialises");

    assert!(
        json.starts_with(
            r#"{"slug":"steam/900001","friendly_name":"Example Game","display_name":"Example Game""#
        ),
        "the row's field order is `slug`, `friendly_name`, `display_name`, … and the slug \
         carries the composed lookup key: {json}"
    );

    let value: serde_json::Value = serde_json::from_str(&json).expect("the row is JSON");
    let object = value.as_object().expect("the row is a JSON object");
    assert_eq!(
        object.len(),
        6,
        "`DiscoveredGame` carries exactly the six fields ADR-025 states: keys {:?}",
        object.keys().collect::<Vec<_>>()
    );
    assert!(
        !object.contains_key("source"),
        "`Source` is construction-side only — the row carries the composed key, never the \
         token: keys {:?}",
        object.keys().collect::<Vec<_>>()
    );
    assert!(
        !object.contains_key("namespace"),
        "the `Namespace` field and its serde spelling retire with `024-C1`: keys {:?}",
        object.keys().collect::<Vec<_>>()
    );
    assert_eq!(object["slug"], "steam/900001");
    assert_eq!(object["friendly_name"], "Example Game");
    assert_eq!(object["display_name"], "Example Game");
    assert_eq!(
        object["platform"], "SteamLinux",
        "`Platform` keeps serde's default variant-name spelling, unchanged by ADR-025"
    );
    assert!(object["codec_id"].is_null(), "`codec_id` stays nullable");
    assert_eq!(object["install_path"], "/steamapps/common/Example Game");
}

/// proves: design 025's Test strategy #1 + #14 (the field-set re-pin) — a row built
/// by hand reads back every field of the ADR-025 shape; the retired `namespace` field
/// is simply not part of the type, which is why its absence is a compile fact this
/// literal pins.
#[test]
fn a_row_carries_the_composed_lookup_key_and_its_friendly_name() {
    let game = row();

    assert_eq!(game.slug, "steam/900001", "the slug carries the lookup key");
    assert_eq!(game.friendly_name, "Example Game");
    assert_eq!(game.display_name, "Example Game");
    assert_eq!(
        game.install_path,
        PathBuf::from("/steamapps/common/Example Game")
    );
    assert_eq!(game.platform, Platform::SteamLinux);
    assert_eq!(game.codec_id, None);
}

/// proves: design 025's Test strategy #2 (Chosen design: Row identity — the only
/// remaining normalization in this crate) — `friendly_name` trims leading and
/// trailing whitespace and collapses interior whitespace runs to one space. It is
/// metadata, not a derivation law: no kebabing, no grammar.
#[test]
fn friendly_name_trims_and_collapses_interior_whitespace_runs() {
    assert_eq!(friendly_name("  Example Game  "), "Example Game");
    assert_eq!(friendly_name("Example   Game"), "Example Game");
    assert_eq!(friendly_name("Example\t\n  Game "), "Example Game");
    assert_eq!(
        friendly_name("Example Game"),
        "Example Game",
        "an already-normal name passes through untouched"
    );
}

/// proves: design 025's Test strategy #2 (Chosen design: Options — a friendly name
/// past its bound; clamp, never refuse, because `friendly_name` joins nothing) — the
/// clamp keeps the longest prefix that is both inside `MAX_NATIVE_NAME_OCTETS` and on
/// a `char` boundary.
#[test]
fn friendly_name_clamps_a_multibyte_name_to_a_char_boundary_inside_the_bound() {
    // 127 ASCII octets plus one two-octet `é` is 129 octets: the 128-octet cut would
    // split `é`, so the answer is the 127-octet prefix.
    let name = format!("{}é", "k".repeat(127));
    assert_eq!(name.len(), MAX_NATIVE_NAME_OCTETS + 1);
    let clamped = friendly_name(&name);
    assert_eq!(
        clamped,
        "k".repeat(127),
        "the clamp keeps the longest `char`-boundary prefix inside the bound"
    );
    assert!(clamped.len() <= MAX_NATIVE_NAME_OCTETS);
    assert!(
        std::str::from_utf8(clamped.as_bytes()).is_ok(),
        "the clamped value is a whole `String`, never a split `char`"
    );
}

/// proves: design 025's Test strategy #2 — the bound sits exactly where
/// `MAX_NATIVE_NAME_OCTETS` states it: a 128-octet name is untouched and a
/// 129-octet name loses exactly one octet, without panicking.
#[test]
fn a_name_at_the_bound_survives_and_one_octet_past_it_loses_one() {
    let at_bound = "k".repeat(MAX_NATIVE_NAME_OCTETS);
    assert_eq!(
        friendly_name(&at_bound),
        at_bound,
        "a name of exactly {MAX_NATIVE_NAME_OCTETS} octets is untouched"
    );

    let past_bound = "k".repeat(MAX_NATIVE_NAME_OCTETS + 1);
    let clamped = friendly_name(&past_bound);
    assert_eq!(
        clamped.len(),
        MAX_NATIVE_NAME_OCTETS,
        "a name one octet over the bound loses exactly that one octet — this is the only \
         place ADR-025 truncates, and only because the value joins nothing"
    );
    assert!(past_bound.starts_with(&clamped));
}
