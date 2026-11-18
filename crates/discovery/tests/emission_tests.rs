//! The plugin emission seam for ADR-024 game identity: every row that leaves a
//! discovery plugin carries a slug that satisfies the shared grammar and the
//! shared length bound, and a row from which no such slug can be built never
//! reaches the output.
//!
//! proves: design 024's Test strategy #3 (fallback and refusal counting) and the
//! plugin half of #8 (the bound is enforced at the emission seam too). Plugin-side
//! refusals are count-only in this design: the row is dropped, its file is counted
//! into the plugin's `skipped` counter, and nothing is echoed per row — the named
//! per-game refusal line lives at the CLI boundary (`024-L1`, see
//! `apps/discovery-cli/tests/cli_slug_refusal_tests.rs`). The counter is observable
//! at this seam through the existing diagnostic that fires when a plugin emitted
//! nothing at all while files were skipped.
//!
//! The rows are built in a scratch Steam root so the boundary cases can be made
//! exactly, and the plugin is driven through `SteamLinuxPlugin::from_roots`, the
//! same seam the production path uses. The derivation rule itself is pinned in
//! `slug_tests.rs`.

use bindforge_core::{MAX_SLUG_OCTETS, is_slug_name};
use discovery::game::{DiscoveredGame, Namespace, Platform};
use discovery::plugin::DiscoveryPlugin;
use discovery::registry::DiscoveryError;
use discovery::slug::derive_slug;
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

/// The install directory of an over-bound game: a single path component whose
/// derived slug is exactly one octet over the shared bound.
fn over_bound_install_dir() -> String {
    let dir = common::letters(64, 64);
    let slug = derive_slug(&dir).expect("a run of letters is slug-shaped");
    assert_eq!(
        slug.len(),
        MAX_SLUG_OCTETS + 1,
        "the fixture must sit exactly one octet over the bound"
    );
    dir
}

/// A name, and the install directory spelling of it, that yield no slug at all.
const NO_SLUG_SOURCE: &str = "\u{5e83}\u{304f}\u{30ed}\u{30c8}\u{3068}\u{3072}\u{3068}\u{307f}";

// proves: design 024's Test strategy #3 — a game whose manifest carries no usable
// name gets its slug from its install directory, through the same algorithm, which
// is what makes the install directory the identity's second source.
#[test]
fn a_game_without_a_name_takes_its_slug_from_its_install_directory() {
    let root = SteamRoot::new("emission-nameless", false);
    root.acf("2201", "", None, "S.T.A.L.K.E.R. Shadow of Chornobyl");

    let games = discover(&root).expect("a nameless install is discoverable");
    assert_eq!(
        slugs(&games),
        vec!["s-t-a-l-k-e-r-shadow-of-chornobyl"],
        "the install directory is the slug's source when the manifest names nothing"
    );
    assert_eq!(
        games[0].display_name, "S.T.A.L.K.E.R. Shadow of Chornobyl",
        "the display name keeps its own install-directory fallback"
    );
}

// proves: design 024's Test strategy #2, #3 + Consequences (the emission
// invariant) — every slug the Linux plugin emits passes `is_slug_name` and the
// shared bound, carries no path separator, and repeats no namespace prefix.
#[test]
fn every_slug_the_linux_plugin_emits_passes_the_shared_grammar_and_bound() {
    let root = SteamRoot::new("emission-grammar", true);
    root.acf("2301", "", Some("Example Game"), "Example Game");
    root.acf("2302", "", Some("Zażółć Gęślą Jaźń"), "Zazolc Gesla Jazn");
    root.acf("2303", "", Some("DOOM (1993)"), "DOOM (1993)");
    root.acf(
        "2304",
        "",
        Some("Final Fantasy XIV - Dawntrail"),
        "ffxiv_dawntrail",
    );
    root.acf("2305", "secondary", Some("Hades"), "Hades II");

    let games = discover(&root).expect("a root of ordinary installs is discoverable");
    assert_eq!(games.len(), 5, "every install in the root is emitted");
    for game in &games {
        assert!(
            is_slug_name(&game.slug),
            "`{}` — the slug emitted for `{}` must satisfy `is_slug_name`: ADR-024 makes it \
             the plugin's output contract",
            game.slug,
            game.display_name
        );
        assert!(
            game.slug.len() <= MAX_SLUG_OCTETS,
            "`{}` is {} octets, past the shared bound of {MAX_SLUG_OCTETS}",
            game.slug,
            game.slug.len()
        );
        assert!(
            !game.slug.contains('/') && !game.slug.contains('\\') && !game.slug.contains(':'),
            "`{}` must not be able to address a path or an id",
            game.slug
        );
        assert!(
            game.slug != "unknown" && !game.slug.starts_with("unknown/"),
            "`{}` must not repeat as a prefix the namespace the `namespace` field now carries",
            game.slug
        );
    }
}

// proves: design 024's Test strategy #2, #8 + Chosen design (Derivation) — length
// is never truncated anywhere: a game whose only slug source is one octet over
// `MAX_SLUG_OCTETS` is refused rather than cut, so no 65–128-octet title is ever
// silently shortened into a different identity.
#[test]
fn a_slug_one_octet_over_the_bound_is_refused_rather_than_truncated() {
    let install_dir = over_bound_install_dir();
    let over_bound_slug = derive_slug(&install_dir).expect("a run of letters is slug-shaped");
    let root = SteamRoot::new("emission-over-bound", false);
    root.acf("2401", "", None, &install_dir);

    let outcome = discover(&root);
    let Err(error) = outcome else {
        let games = outcome.expect("checked");
        let emitted: Vec<usize> = games.iter().map(|g| g.slug.len()).collect();
        assert!(
            !games.iter().any(|g| g.slug == over_bound_slug),
            "a slug of {} octets must not be emitted; the row must be refused",
            over_bound_slug.len()
        );
        panic!(
            "an over-bound slug must be refused, and a run whose only game is refused reports \
             it through the skip path; emitted slug lengths: {emitted:?}"
        );
    };

    let message = format!("{error:?}");
    assert!(
        message.contains("skipped"),
        "the refusal counts into the plugin's `skipped` counter, whose diagnostic is the only \
         thing a plugin-side refusal says: `{message}`"
    );
    assert!(
        message.contains("emission-over-bound"),
        "the diagnostic names the selected root, as the shipped one does: `{message}`"
    );
    assert!(
        !message.contains(over_bound_slug.as_str()),
        "a plugin-side refusal is count-only and never echoes the row it dropped: `{message}`"
    );
}

// proves: design 024's Test strategy #3 + Chosen design (Emission, refusal and
// collisions) — a refused row is removed from the plugin's output while the rows
// that are fine still come out: the bound is per row, not per root.
#[test]
fn an_over_bound_row_is_dropped_while_the_valid_rows_of_the_same_root_survive() {
    let install_dir = over_bound_install_dir();
    let root = SteamRoot::new("emission-over-bound-mixed", false);
    root.acf("2501", "", Some("Example Game"), "Example Game");
    root.acf("2502", "", None, &install_dir);

    let games =
        discover(&root).expect("one refused row must not fail the run while another row is fine");
    assert_eq!(
        slugs(&games),
        vec!["example-game"],
        "only the in-bound row is emitted, and the refused row leaves no trace in it"
    );
}

// proves: design 024's Test strategy #3 + Chosen design (Derivation) — the
// fallback chain ends in a refusal: when neither the display name nor the install
// directory yields a slug, the game is not emitted, so no invented slug such as
// today's bare `unknown/` ever becomes an identity.
#[test]
fn a_game_with_no_slug_source_in_either_field_is_not_emitted() {
    assert!(
        derive_slug(NO_SLUG_SOURCE).is_none(),
        "the fixture's two fields must both be slug-less for this pin to mean anything"
    );

    let root = SteamRoot::new("emission-no-slug-source", false);
    root.acf("2601", "", Some(NO_SLUG_SOURCE), NO_SLUG_SOURCE);

    let outcome = discover(&root);
    let Err(error) = outcome else {
        panic!(
            "a game with no derivable slug must not be emitted; the plugin invents nothing. \
             Emitted: {:?}",
            slugs(&outcome.expect("checked"))
        );
    };
    let message = format!("{error:?}");
    assert!(
        message.contains("skipped"),
        "the refusal is counted into `skipped`, which is how a run reports it: `{message}`"
    );
    assert!(
        message.contains("emission-no-slug-source"),
        "the diagnostic names the root the game came from, so the run can be traced without a \
         second lookup: `{message}`"
    );
    assert!(
        !message.contains(NO_SLUG_SOURCE),
        "the refusal is count-only: it names roots and counts, never the games it skipped: \
         `{message}`"
    );
}

// proves: design 024's Test strategy #3 + Risks (the named refusal through the
// shared skip path) — an un-derivable game is a per-row loss: it is counted and
// dropped, and the rest of the catalogue is discovered normally.
#[test]
fn an_un_derivable_game_does_not_disturb_the_other_games_of_its_root() {
    let root = SteamRoot::new("emission-no-slug-source-mixed", false);
    root.acf("2701", "", Some(NO_SLUG_SOURCE), NO_SLUG_SOURCE);
    root.acf("2702", "", Some("Hades"), "Hades II");

    let games = discover(&root).expect("the derivable row must still be discovered");
    assert_eq!(slugs(&games), vec!["hades"]);
    assert_eq!(games[0].display_name, "Hades");
}

// proves: design 024's Test strategy #1, #11 + Deferred (reserved device names
// stay deferred) — a name that is already a slug is emitted verbatim, including a
// Windows reserved-device spelling: the `g-` rule applies to a leading digit only,
// and this row is the emission-side paper trail for that deferral.
#[test]
fn a_name_that_is_already_a_slug_is_emitted_unchanged() {
    let root = SteamRoot::new("emission-verbatim", false);
    root.acf("2801", "", Some("hades-ii"), "hades-ii");
    root.acf("2802", "", Some("con"), "con");

    let games = discover(&root).expect("both installs are discoverable");
    assert_eq!(
        slugs(&games),
        vec!["con", "hades-ii"],
        "the slug-ordered pair, unchanged by derivation"
    );
}

// proves: design 024's Test strategy #5, #8 + Chosen design (Identity shape) — the
// identity change moves the namespace out of the slug into its own field and
// leaves the row's other attributions alone: the platform still says which plugin
// found the game.
#[test]
fn a_row_still_carries_the_platform_of_the_plugin_that_found_it() {
    let root = SteamRoot::new("emission-platform", false);
    root.acf("2901", "", Some("Example Game"), "Example Game");

    let games = discover(&root).expect("the install is discoverable");
    assert_eq!(
        games[0].platform,
        Platform::SteamLinux,
        "the row stays attributed to the plugin that found it: ADR-024's `namespace` is a \
         different axis, and its serialized spelling is pinned in `game_tests.rs`"
    );
    assert_eq!(
        games[0].namespace,
        Namespace::Unknown,
        "and the row's namespace is the v1 namespace, not something the platform stands in for"
    );
}

// proves: design 024's Test strategy #3 (Chosen design: only the slug is new) —
// the install path keeps its shipped shape, `<canonical library>/steamapps/common/\
// <installdir>`, built from the install directory and never from the slug.
#[test]
fn a_rows_install_path_is_still_the_library_plus_the_install_directory() {
    let root = SteamRoot::new("emission-install-path", true);
    root.acf("3001", "secondary", Some("Hades"), "Hades II");

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
    assert!(
        !games[0].install_path.to_string_lossy().contains("hades-ii"),
        "the install path is built from the install directory, not from the slug: {:?}",
        games[0].install_path
    );
}
