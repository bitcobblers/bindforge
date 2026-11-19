//! The plugin emission seam for ADR-025 discovery identity: every row that leaves a
//! Steam plugin carries the `steam/<appid>` lookup key built from the `AppState.appid`
//! the scan now reads, and a manifest whose appid is missing or not a numeric id
//! never reaches the output — refused through the same count-only skip path an
//! unusable `installdir` already takes.
//!
//! proves: design 025's Test strategy #3 (appid extraction: `"appid" "900001"` yields
//! the slug `steam/900001`; key casing accepted; a missing key, an empty value, a
//! non-numeric value, an embedded `/`, and a 21-digit value each yield no row, with
//! the skip counted) and the plugin half of #4 (the identity gate is bound-only: the
//! grammar-invalid `steam/…` spelling survives, and an over-bound row is the
//! defense-in-depth skip the plugin's construction path owns). Plugin-side refusals
//! stay count-only in this design: the row is dropped, its file is counted into the
//! plugin's `skipped` counter, and nothing is echoed per row — the named per-row
//! refusal lines live at the CLI boundary.
//!
//! The rows are built in a scratch Steam root so the boundary cases can be made
//! exactly, and the plugin is driven through `SteamLinuxPlugin::from_roots`, the
//! same seam the production path uses. The row-shape rule itself is pinned in
//! `game_tests.rs`; the `steam_windows` parity in `steam_windows_tests.rs`.

use bindforge_core::{MAX_DISCOVERY_SLUG_OCTETS, is_slug_name};
use discovery::game::{DiscoveredGame, Platform};
use discovery::plugin::DiscoveryPlugin;
use discovery::registry::DiscoveryError;
use discovery::steam_linux::SteamLinuxPlugin;

mod common;

use common::SteamRoot;

/// The one scratch root, run through the Linux plugin.
fn discover(root: &SteamRoot) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
    SteamLinuxPlugin::from_roots(vec![root.path().to_path_buf()]).discover()
}

/// The slugs of discovered games, in the order the plugin returned them.
fn slugs(games: &[DiscoveredGame]) -> Vec<&str> {
    games.iter().map(|g| g.slug.as_str()).collect()
}

/// The twenty-one-digit value one past the accepted 1–20 ASCII-digit appid set.
const TWENTY_ONE_DIGITS: &str = "123456789012345678901";

// proves: design 025's Test strategy #3 (Chosen design: Emission — `parse_manifest`
// reads `app_state.get_str("appid")`, the plugin composes the slug) — a quoted
// `appid` becomes the slug `steam/<appid>`, and the row's `friendly_name` is the
// display name's normalization.
#[test]
fn an_appid_becomes_the_steam_lookup_key_on_the_emitted_row() {
    let root = SteamRoot::new("emission-appid", false);
    root.acf("900001", "", Some("Example Game"), "Example Game");

    let games = discover(&root).expect("an ordinary install is discoverable");
    assert_eq!(
        slugs(&games),
        vec!["steam/900001"],
        "`\"appid\" \"900001\"` yields the slug `steam/900001` — the identity is read \
         out of the ACF, never derived from the name"
    );
    assert_eq!(games[0].display_name, "Example Game");
    assert_eq!(
        games[0].friendly_name, "Example Game",
        "the row's friendly name is the display name's normalization"
    );
    assert_eq!(
        games[0].platform,
        Platform::SteamLinux,
        "the row stays attributed to the plugin that found it"
    );
}

// proves: design 025's Test strategy #3 + 01-research.md Open question 9 — the appid
// read is case-insensitive, like every other key the shipped reader looks up: `AppId`
// and `APPID` spellings resolve and emit `steam/<appid>`.
#[test]
fn appid_key_casing_is_accepted() {
    for (key, appid) in [
        ("AppId", "777001"),
        ("APPID", "777002"),
        ("appid", "777003"),
    ] {
        let root = SteamRoot::new(&format!("emission-appid-case-{appid}"), false);
        root.acf_keyed(
            appid,
            "",
            Some("Mixed Case Key Game"),
            "Mixed Case Key Game",
            Some((key, appid)),
        );

        let games = discover(&root).unwrap_or_else(|err| panic!("`{key}` must resolve: {err}"));
        let want = format!("steam/{appid}");
        assert_eq!(
            slugs(&games),
            vec![want.as_str()],
            "the `{key}` spelling of the appid key must compose the same lookup key"
        );
    }
}

// proves: design 025's Test strategy #3 (Chosen design: Emission — the appid must be
// 1–20 ASCII digits after trimming; anything else is the same skip-and-count outcome
// as an unusable `installdir`) — a missing `appid` key yields no row while the valid
// row of the same root survives, and the skip is reported through the plugin's
// count-only `skipped` diagnostic.
#[test]
fn a_manifest_without_an_appid_key_yields_no_row_and_counts_into_skipped() {
    let root = SteamRoot::new("emission-no-appid", false);
    root.acf_keyed(
        "900101",
        "",
        Some("Nameless Identity"),
        "Nameless Identity",
        None,
    );
    root.acf("900102", "", Some("Example Game"), "Example Game");

    let games =
        discover(&root).expect("one refused row must not fail the run while another row is fine");
    assert_eq!(
        slugs(&games),
        vec!["steam/900102"],
        "the appid-less row is dropped; no row carries the file name's number as a \
         fallback identity, and none carries a name-derived slug"
    );
    assert!(
        !games.iter().any(|g| g.slug.contains("900101")),
        "the `appmanifest_900101.acf` file name is never an identity source: {:?}",
        slugs(&games)
    );

    let solo = SteamRoot::new("emission-no-appid-solo", false);
    solo.acf_keyed(
        "900103",
        "",
        Some("Nameless Identity"),
        "Nameless Identity",
        None,
    );
    let outcome = discover(&solo);
    let Err(error) = outcome else {
        panic!(
            "a root whose only manifest has no appid must report the skip through the \
             plugin's error diagnostic; emitted: {:?}",
            slugs(&outcome.expect("checked"))
        );
    };
    let message = format!("{error:?}");
    assert!(
        message.contains("skipped"),
        "the refusal counts into the plugin's `skipped` counter: `{message}`"
    );
    assert!(
        message.contains("emission-no-appid-solo"),
        "the diagnostic names the selected root, as the shipped one does: `{message}`"
    );
}

// proves: design 025's Test strategy #3 (Chosen design: Emission — "trims it, and
// requires 1–20 ASCII digits") — an empty value, a whitespace-only value, a
// non-numeric value, an embedded `/`, and a 21-digit value each yield no row; a
// 20-digit value is the accepted extreme. Every refused value names nothing: the file
// name is spelled away from the value so a filename fallback would show up as a row.
#[test]
fn a_refused_appid_value_yields_no_row() {
    let refused: [(&str, &str); 5] = [
        ("empty", ""),
        ("whitespace", "   "),
        ("non-numeric", "900o1"),
        ("embedded-slash", "900/1"),
        ("twenty-one-digits", TWENTY_ONE_DIGITS),
    ];
    for (label, value) in refused {
        let root = SteamRoot::new(&format!("emission-bad-appid-{label}"), false);
        root.acf_keyed(
            "900201",
            "",
            Some("Bad Appid Game"),
            "Bad Appid Game",
            Some(("appid", value)),
        );
        root.acf("900202", "", Some("Example Game"), "Example Game");

        let games = discover(&root).unwrap_or_else(|err| {
            panic!("the `{label}` case must still emit the valid row: {err}")
        });
        assert_eq!(
            slugs(&games),
            vec!["steam/900202"],
            "appid value `{value}` (the {label} case) must refuse its row through the \
             skip path while the valid row survives"
        );
    }

    let twenty_digits = "1".repeat(20);
    let root = SteamRoot::new("emission-twenty-digit-appid", false);
    root.acf(&twenty_digits, "", Some("Long Id Game"), "Long Id Game");
    let games = discover(&root).expect("a 20-digit appid is the accepted extreme");
    let want = format!("steam/{twenty_digits}");
    assert_eq!(
        slugs(&games),
        vec![want.as_str()],
        "1–20 ASCII digits are the accepted set — the twentieth digit still composes"
    );
    assert!(
        games[0].slug.len() <= MAX_DISCOVERY_SLUG_OCTETS,
        "even the widest accepted appid sits inside `025-B1`"
    );
}

// proves: design 025's Test strategy #4 (Chosen design: Emission — "the identity gate
// becomes bound-only … `steam/1` — grammar-invalid by design — survives, which the
// retired grammar check refused") — a minimal, grammar-invalid lookup key is emitted
// unchanged: only `025-B1`'s length bound refuses, and nothing truncates.
#[test]
fn a_grammar_invalid_but_in_bound_lookup_key_is_emitted() {
    let root = SteamRoot::new("emission-bound-only", false);
    root.acf("1", "", Some("Minimal Appid Game"), "Minimal Appid Game");

    let games = discover(&root)
        .expect("`steam/1` is grammar-invalid by design and must survive the bound-only gate");
    assert_eq!(slugs(&games), vec!["steam/1"]);
    assert!(
        !is_slug_name(&games[0].slug),
        "`{}` fails the shared product-slug grammar — that is the point: the retired \
         `024` grammar check refused exactly this row",
        games[0].slug
    );
    assert!(
        games[0].slug.len() <= MAX_DISCOVERY_SLUG_OCTETS,
        "`{}` is inside the only bound the emission seam checks",
        games[0].slug
    );
}

// proves: design 025's Test strategy #2 + Chosen design (Emission — the row's
// `friendly_name` comes from `display_name`, whose absent/empty-name fallback to
// `installdir` is unchanged) — the identity move does not disturb the display chain.
#[test]
fn the_display_name_and_friendly_name_keep_their_install_directory_fallback() {
    let root = SteamRoot::new("emission-nameless-display", false);
    root.acf("900301", "", None, "S.T.A.L.K.E.R. Shadow of Chornobyl");

    let games = discover(&root).expect("a nameless install is discoverable");
    assert_eq!(
        slugs(&games),
        vec!["steam/900301"],
        "identity comes from the appid even when the name can derive nothing"
    );
    assert_eq!(
        games[0].display_name, "S.T.A.L.K.E.R. Shadow of Chornobyl",
        "the display name keeps its own install-directory fallback"
    );
    assert_eq!(
        games[0].friendly_name, "S.T.A.L.K.E.R. Shadow of Chornobyl",
        "the friendly name normalizes the same fallback value"
    );
}

// proves: design 025's Test strategy #3 + Chosen design (Emission — "`resolve_collisions`
// … deleted … The `(slug, install_path)` sort stays, which keeps the same game in two
// libraries as two rows and makes the row order the byte order of `steam/<appid>`") —
// rows leave the plugin in the byte order of the composed keys.
#[test]
fn rows_leave_the_plugin_in_the_byte_order_of_their_lookup_keys() {
    let root = SteamRoot::new("emission-order", true);
    root.acf(
        "900401",
        "",
        Some("Primary Library Game"),
        "Primary Library Game",
    );
    root.acf(
        "900400",
        "secondary",
        Some("Secondary Library Game"),
        "Secondary Title",
    );

    let games = discover(&root).expect("both libraries are discoverable");
    assert_eq!(
        slugs(&games),
        vec!["steam/900400", "steam/900401"],
        "the row order is the byte order of `steam/<appid>`, across libraries, with no \
         collision pass in between"
    );
    for game in &games {
        assert!(
            game.slug.len() <= MAX_DISCOVERY_SLUG_OCTETS
                && !game.slug.contains('\\')
                && !game.slug.contains(':'),
            "every emitted key `{}` is inside the bound and carries no separator beyond \
             its one `/`",
            game.slug
        );
    }
}

// proves: design 025's Rollout 1 (the emission seam is otherwise unchanged) — the
// install path keeps its shipped shape, `<canonical library>/steamapps/common/\
// <installdir>`, built from the install directory and never from the identity.
#[test]
fn a_rows_install_path_is_still_the_library_plus_the_install_directory() {
    let root = SteamRoot::new("emission-install-path", true);
    root.acf("900501", "secondary", Some("Hades"), "Hades II");

    let games = discover(&root).expect("the secondary library install is discoverable");
    let expected = root
        .canonical()
        .join("secondary")
        .join("steamapps")
        .join("common")
        .join("Hades II");
    assert_eq!(
        games[0].install_path, expected,
        "the row's install path is the install directory inside the library that holds it, \
         canonicalized the way the shipped rule canonicalizes it"
    );
    assert_eq!(
        games[0].slug, "steam/900501",
        "the identity rides beside the path, never inside it"
    );
}
