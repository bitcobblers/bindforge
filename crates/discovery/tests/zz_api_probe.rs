//! The public surface ADR-024 gives `crates/discovery`: the canonical derivation,
//! the namespace the slug used to smuggle, and the row's new field — plus the two
//! shared slug names the crate now compiles against from `bindforge-core`.
//!
//! proves: design 024's Test strategy #12 + its Delta inventory (Rollout item 4:
//! "`slug.rs` rewritten to the canonical derivation; `Namespace` field; … the new
//! API probe"; and Risks: "the only in-workspace consumer [of the removed public
//! `normalize_slug`] is `apps/discovery-cli`, re-pinned in the same run"). This
//! target resolves every promised name in type or call position, so a rename of
//! `derive_slug` or `Namespace`, or a field moved out of `DiscoveredGame`, is a
//! compile failure here rather than a silent break for the CLI and for the profile
//! writers that adopt a discovered slug as a manifest `id`.
//!
//! `normalize_slug` is deleted by this design (A4's public-surface note), so no
//! name here asks for it: the probe resolves its replacement, and the shipped
//! suites that called it are re-pinned in the same run. The behavior behind each
//! name belongs to `slug_tests.rs` (#1, #2) and `game_tests.rs` (#5); this file
//! adds no rule of its own.

use bindforge_core::{MAX_SLUG_OCTETS, is_slug_name};
use discovery::game::{DiscoveredGame, Namespace, Platform};
use discovery::slug::derive_slug;
use std::path::PathBuf;

/// The derivation in a signature: what the plugin's construction path calls, and
/// what an outside consumer can call to check a slug before it becomes an id.
fn derived(name: &str) -> Option<String> {
    derive_slug(name)
}

/// proves: design 024's Test strategy #12 (Chosen design: Derivation, Identity
/// shape) — `derive_slug` resolves in call position answering `Option<String>`,
/// `Namespace` resolves as a closed enum whose `Display` is the kebab-case
/// spelling, and a `DiscoveredGame` literal built with `namespace` compiles and
/// reads back.
#[test]
fn the_adr_024_discovery_names_resolve_at_the_crate_root() {
    // `derive_slug`, in call position: the derivation is total and answers `None`
    // rather than inventing a slug.
    let slug: Option<String> = derived("NieR:Automata\u{2122}");
    assert_eq!(
        slug.as_deref(),
        Some("nier-automata"),
        "`derive_slug` is the canonical spelling of the old `normalize_slug` name"
    );
    assert_eq!(
        derived("\u{5e83}\u{304f}\u{30ed}\u{30c8}").as_deref(),
        None,
        "a name with no Latin base has no slug, which is the refusal the plugin's \
         construction path returns"
    );

    // `Namespace`, in type position on a row, and `Display` in the spelling the
    // CLI's `Namespace` column prints.
    let namespace: Namespace = Namespace::Unknown;
    assert_eq!(
        namespace.to_string(),
        "unknown",
        "`Display` renders the one v1 namespace, in the spelling the table prints"
    );

    // `DiscoveredGame`'s field set, in field position: `namespace` beside `slug`,
    // and everything the shipped row carried still there.
    let game = DiscoveredGame {
        slug: "nier-automata".to_string(),
        namespace,
        display_name: "NieR:Automata\u{2122}".to_string(),
        install_path: PathBuf::from("/steamapps/common/NieR Automata"),
        platform: Platform::SteamLinux,
        codec_id: None,
    };
    assert_eq!(game.slug, "nier-automata", "`slug` is still a String field");
    assert_eq!(
        game.namespace,
        Namespace::Unknown,
        "`namespace` is a field of the row, not a prefix inside the slug"
    );
    assert_eq!(game.display_name, "NieR:Automata\u{2122}");
    assert_eq!(
        game.install_path,
        PathBuf::from("/steamapps/common/NieR Automata"),
        "`install_path` keeps the type the shipped row used"
    );
    assert_eq!(game.platform, Platform::SteamLinux);
    assert!(
        game.codec_id.is_none(),
        "`codec_id` stays the nullable field it was"
    );
    assert!(
        is_slug_name(&game.slug) && game.slug.len() <= MAX_SLUG_OCTETS,
        "a hand-built row's slug is in the shape core's parsers will accept as a manifest id"
    );
}

/// proves: design 024's Test strategy #12 (Chosen design: Core — "`lib.rs`: `pub
/// use placement::{is_slug_name, MAX_SLUG_OCTETS}`", and Discovery: "`Cargo.toml`
/// gains `bindforge-core`") — the shared grammar predicate and the twelfth budget
/// constant resolve at the `bindforge-core` root and compose with the crate's own
/// derivation, which is the edge the row-level suites are written through.
#[test]
fn the_shared_slug_names_resolve_at_the_bindforge_core_root() {
    assert_eq!(
        MAX_SLUG_OCTETS, 128,
        "the shared bound is ADR-024's raised value, at one stated home"
    );

    let derived =
        derive_slug("Zażółć Gęślą Jaźń").expect("the Polish survey title has Latin bases");
    assert!(
        is_slug_name(&derived),
        "`{derived}` — a derived slug is a manifest-id-shaped token by construction, which is \
         the invariant this crate's emission seam hands to core's parsers"
    );
    assert!(
        derived.len() <= MAX_SLUG_OCTETS,
        "`{derived}` is inside the bound the same constant states"
    );
    assert!(
        !is_slug_name("unknown/nier-automata"),
        "and the namespaced spelling ADR-024 retires is not a slug name: `{}` must fail the \
         shared grammar",
        "unknown/nier-automata"
    );
}
