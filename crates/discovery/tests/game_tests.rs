//! The module-mirrored suite for `crates/discovery/src/game.rs` (ADR-004): the row a
//! plugin emits, and the namespace field ADR-024 gives it.
//!
//! proves: design 024's Test strategy #5 — `Namespace` serde round-trips as
//! `"unknown"`, its `Display` renders `unknown`, and a `DiscoveredGame` serialises
//! with the new `namespace` field between `slug` and `display_name` (the JSON shape
//! pin). The `"unknown"` spelling is the kebab-case value the design chose for this
//! closed enum; `Platform` keeps serde's default variant-name spelling (`"SteamLinux"`),
//! a divergence the design states as deliberate.
//!
//! This file also carries the ADR-024 re-pin of the shipped row test: a discovered
//! game's `slug` is a single-segment product id, so the old `unknown/test-game`
//! spelling is gone and the namespace it used to smuggle is now its own field.
//!
//! Note for the build stage (Rollout 4): these are the crate's first serde assertions,
//! so `crates/discovery` needs `serde_json` as a dev-dependency (`Cargo.toml`
//! `[dev-dependencies]`). The `bindforge-core` edge the design adds is what lets the
//! row-level grammar assertions below read the shared predicate.
//!
//! Unit-level: serde over in-memory values, no filesystem, no network (repo practice).

use discovery::game::{DiscoveredGame, Namespace, Platform};
use std::path::PathBuf;

/// A row in the shape ADR-024 defines: prefix-free slug, its own namespace field, and
/// the fields the shipped row already carried.
fn row(slug: &str, display_name: &str) -> DiscoveredGame {
    DiscoveredGame {
        slug: slug.to_string(),
        namespace: Namespace::Unknown,
        display_name: display_name.to_string(),
        install_path: PathBuf::from("/steamapps/common/Game"),
        platform: Platform::SteamLinux,
        codec_id: None,
    }
}

/// proves: design 024's Test strategy #5 + Chosen design (Identity shape) — the
/// namespace's serde spelling is the kebab-case `"unknown"` in both directions, and
/// `Display` renders the same string the table's `Namespace` column shows.
#[test]
fn the_namespace_round_trips_as_unknown_in_serde_and_renders_unknown() {
    assert_eq!(
        serde_json::to_string(&Namespace::Unknown).expect("a namespace serialises"),
        "\"unknown\"",
        "`Namespace::Unknown` serialises as the kebab-case `\"unknown\"` the design names"
    );
    assert_eq!(
        serde_json::from_str::<Namespace>("\"unknown\"").expect("`\"unknown\"` deserialises"),
        Namespace::Unknown,
        "the row JSON's `\"unknown\"` comes back as the v1 namespace"
    );
    assert_eq!(
        Namespace::Unknown.to_string(),
        "unknown",
        "`Display` renders the same spelling the CLI's `Namespace` column prints"
    );
}

/// proves: design 024's Test strategy #5 + Goals (a separate *closed* field) —
/// `unknown` is the only v1 value: `publisher` and the PascalCase spelling are outside
/// the closed set, so a row cannot claim a namespace the product has not defined
/// (ADR-025 owns any second value).
#[test]
fn the_namespace_field_is_closed_to_the_one_v1_value() {
    for spelling in [
        "\"publisher\"",
        "\"Unknown\"",
        "\"UNKNOWN\"",
        "\"\"",
        "\"unknown-x\"",
    ] {
        assert!(
            serde_json::from_str::<Namespace>(spelling).is_err(),
            "`{spelling}` is not the closed v1 namespace set, so it must not deserialise"
        );
    }
}

/// proves: design 024's Test strategy #5 + Rollout item 4+5 (one commit: the row's
/// field set and the CLI's rendering of it) — the compact serialisation of a
/// `DiscoveredGame` is the six-field row with `"namespace":"unknown"` sitting between
/// `slug` and `display_name`, and `platform` keeping its default variant spelling.
#[test]
fn a_discovered_game_serialises_its_namespace_between_slug_and_display_name() {
    let game = row("ys-ix-monstrum-nox", "Ys IX: Monstrum Nox");
    let json = serde_json::to_string(&game).expect("a row serialises");

    assert!(
        json.contains(r#"{"slug":"ys-ix-monstrum-nox","namespace":"unknown","display_name":"Ys IX: Monstrum Nox""#),
        "the row's field order is `slug`, `namespace`, `display_name`, … and the namespace \
         spells `unknown`: {json}"
    );

    let value: serde_json::Value = serde_json::from_str(&json).expect("the row is JSON");
    let object = value.as_object().expect("the row is a JSON object");
    assert_eq!(
        object.len(),
        6,
        "`DiscoveredGame` gains exactly one field over the shipped five: keys {object:?}"
    );
    assert_eq!(object["slug"], "ys-ix-monstrum-nox");
    assert_eq!(object["namespace"], "unknown");
    assert_eq!(object["display_name"], "Ys IX: Monstrum Nox");
    assert_eq!(
        object["platform"], "SteamLinux",
        "`Platform` keeps its default variant-name spelling: the divergence between the two \
         closed enums is the design's stated decision"
    );
    assert!(object["codec_id"].is_null(), "`codec_id` stays nullable");
    assert_eq!(object["install_path"], "/steamapps/common/Game");
}

/// proves: design 024's Test strategy #5 + #8 (re-pin of the shipped row test) — a
/// row built by hand carries a single-segment, grammar-valid slug in its own namespace,
/// which is the shape every plugin must emit before the CLI renders it.
#[test]
fn a_row_carries_a_single_segment_slug_beside_its_namespace() {
    let game = row("test-game", "Test Game");

    assert_eq!(
        game.slug, "test-game",
        "the slug carries no namespace prefix"
    );
    assert_eq!(game.namespace, Namespace::Unknown);
    assert_eq!(game.display_name, "Test Game");
    assert_eq!(game.install_path, PathBuf::from("/steamapps/common/Game"));
    assert_eq!(game.platform, Platform::SteamLinux);
    assert_eq!(game.codec_id, None);

    assert!(
        bindforge_core::is_slug_name(&game.slug),
        "a hand-built row's slug is a product id: `{}` must pass the shared grammar",
        game.slug
    );
    assert!(
        game.slug.len() <= bindforge_core::MAX_SLUG_OCTETS,
        "a hand-built row's slug is inside the shared bound: {}",
        game.slug.len()
    );
}
