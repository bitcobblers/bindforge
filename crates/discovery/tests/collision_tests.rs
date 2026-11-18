//! The per-plugin collision pass ADR-024 adds to discovery: one discovered game,
//! one slug.
//!
//! proves: design 024's Test strategy #4. The rule the suite pins, from the
//! proposal's "Emission, refusal and collisions" section: the pass runs once per
//! plugin over all rows that plugin collected; it walks the distinct display names
//! in byte order keeping an assigned-slug map; the first name to claim a base slug
//! keeps it; a later distinct name whose derivation is already claimed takes the
//! first free candidate `base-2`, `base-3`, …, skipping candidates another name
//! already holds; a name with no candidate inside the shared bound is refused and
//! counted; rows that share a display name share its assigned slug; and the
//! `Vec` is ordered by `(slug, install_path)` before it is returned.
//!
//! The refusals the pass makes are count-only, so this suite pins them by the
//! absent row; the counter's delta is pinned beside the pass itself, in the
//! `#[cfg(test)]` module of `crates/discovery/src/steam_common.rs` (the r1 F2
//! resolution): unit tests over hand-built rows show `skipped` growing by exactly
//! the refused row count, while suffix and shared-name rows count nothing.
//! `emission_tests.rs` pins the construction seam's refusals.

use bindforge_core::{MAX_SLUG_OCTETS, is_slug_name};
use discovery::game::DiscoveredGame;
use discovery::plugin::DiscoveryPlugin;
use discovery::registry::DiscoveryError;
use discovery::steam_linux::SteamLinuxPlugin;
use std::path::PathBuf;

mod common;

use common::SteamRoot;

/// The slugs of discovered games, in the order the plugin returned them.
fn slugs(games: &[DiscoveredGame]) -> Vec<&str> {
    games.iter().map(|g| g.slug.as_str()).collect()
}

/// Pairs of `(slug, display name)` so a suffix assertion says who got it.
fn identities(games: &[DiscoveredGame]) -> Vec<(&str, &str)> {
    games
        .iter()
        .map(|g| (g.slug.as_str(), g.display_name.as_str()))
        .collect()
}

/// Several roots, run through one Linux plugin instance — the scope the design
/// gives the collision pass.
fn discover_all(roots: &[&SteamRoot]) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
    let paths: Vec<PathBuf> = roots.iter().map(|r| r.path().to_path_buf()).collect();
    SteamLinuxPlugin::from_roots(paths).discover()
}

// proves: design 024's Test strategy #4 (Chosen design: the collision pass) — the
// surveyed `Ys IX: Monstrum Nox` / `Ys IX -Monstrum NOX-` pair derives one base
// slug, and the two rows keep two identities: the base goes to the first name in
// byte order and `-2` to the second, with the rows ordered by slug.
#[test]
fn a_pair_of_titles_that_derive_one_slug_is_split_on_the_base_and_a_numeric_suffix() {
    let root = SteamRoot::new("collision-ys", false);
    root.acf(
        "4001",
        "",
        Some("Ys IX: Monstrum Nox"),
        "Ys IX Monstrum Nox",
    );
    root.acf(
        "4002",
        "",
        Some("Ys IX -Monstrum NOX-"),
        "Ys IX Monstrum NOX",
    );

    let games = discover_all(&[&root]).expect("both installs are discoverable");
    assert_eq!(
        identities(&games),
        vec![
            ("ys-ix-monstrum-nox", "Ys IX -Monstrum NOX-"),
            ("ys-ix-monstrum-nox-2", "Ys IX: Monstrum Nox"),
        ],
        "` ` sorts before `:` in byte order, so the hyphenated title keeps the base slug"
    );
    assert_ne!(
        games[0].install_path, games[1].install_path,
        "the two rows are the two installs, not one row duplicated"
    );
}

// proves: design 024's Test strategy #4 (Chosen design: "rows sharing a display
// name share its assigned slug") — the same game seen in two libraries of one
// plugin keeps one slug, so ADR-024's identity survives the duplicate row instead
// of inventing a second id for one installation.
#[test]
fn one_display_name_in_two_libraries_of_one_plugin_keeps_one_slug() {
    let root = SteamRoot::new("collision-two-libraries", true);
    root.acf("4101", "", Some("Hades"), "Hades");
    root.acf("4102", "secondary", Some("Hades"), "Hades");

    let games = discover_all(&[&root]).expect("both installs are discoverable");
    assert_eq!(
        identities(&games),
        vec![("hades", "Hades"), ("hades", "Hades")],
        "the duplicate keeps the assigned slug rather than taking a `-2`"
    );
    assert_ne!(
        games[0].install_path, games[1].install_path,
        "the two rows are distinguished by their install paths, the shipped ordering key"
    );
    assert!(
        games[0].install_path < games[1].install_path,
        "rows sharing a slug stay ordered by install path: {:?} then {:?}",
        games[0].install_path,
        games[1].install_path
    );
}

// proves: design 024's Test strategy #4 (Chosen design: "the pass re-applies
// `(slug, install_path)` ordering before returning") — one install directory under
// two roots of one plugin is two rows with one slug, ordered by install path.
#[test]
fn one_display_name_in_two_roots_of_one_plugin_keeps_one_slug_too() {
    let first = SteamRoot::new("collision-roots-a", false);
    let second = SteamRoot::new("collision-roots-b", false);
    first.acf("4201", "", Some("Hades"), "Hades");
    second.acf("4201", "", Some("Hades"), "Hades");

    let games = discover_all(&[&first, &second]).expect("both roots are discoverable");
    assert_eq!(
        slugs(&games),
        vec!["hades", "hades"],
        "a second copy of the same title is not a second identity"
    );
}

// proves: design 024's Test strategy #4 (Chosen design: "skipping candidates
// already claimed, which keeps a suffixed slug from colliding with a different
// name's base slug") — the pass hands out free candidates in order, never one a
// different name already holds.
#[test]
fn a_third_name_that_would_take_an_already_claimed_candidate_moves_to_the_next_free_one() {
    let root = SteamRoot::new("collision-candidates", false);
    root.acf("4301", "", Some("Alpha Bet"), "Alpha Bet");
    root.acf("4302", "", Some("Alpha Bet 2"), "Alpha Bet 2");
    root.acf("4303", "", Some("Alpha Bet!"), "Alpha Bet Bang");

    let games = discover_all(&[&root]).expect("all three installs are discoverable");
    assert_eq!(
        identities(&games),
        vec![
            ("alpha-bet", "Alpha Bet"),
            ("alpha-bet-2", "Alpha Bet 2"),
            ("alpha-bet-3", "Alpha Bet!"),
        ],
        "`Alpha Bet!` derives the claimed `alpha-bet`, and its first free candidate is \
         `alpha-bet-3` because `alpha-bet-2` is another name's base slug"
    );
}

// proves: design 024's Test strategy #4 + Risks (a derivation past the bound) — a
// name whose base slug is already at `MAX_SLUG_OCTETS` has no in-bound suffixed
// candidate, so the second name is refused and counted instead of being emitted
// with an over-bound or truncated slug.
#[test]
fn a_name_with_no_in_bound_candidate_is_refused_rather_than_pushed_past_the_bound() {
    let base_name = common::letters(64, 63);
    let base_slug = base_name.replace(' ', "-");
    assert_eq!(
        base_slug.len(),
        MAX_SLUG_OCTETS,
        "the fixture's base slug must sit exactly on the bound for its `-2` to fall outside it"
    );
    let other_name = common::letters_exclaim(64, 63);

    let root = SteamRoot::new("collision-no-candidate", false);
    root.acf("4401", "", Some(&base_name), "Boundary A");
    root.acf("4402", "", Some(&other_name), "Boundary B");
    root.acf("4403", "", Some("Example Game"), "Example Game");

    let games = discover_all(&[&root]).expect("the in-bound rows are still discoverable");
    assert_eq!(
        slugs(&games),
        vec!["example-game", base_slug.as_str()],
        "the byte-order-later name has no candidate inside the bound and is refused; the \
         other two rows are unaffected"
    );
    for game in &games {
        assert!(
            game.slug.len() <= MAX_SLUG_OCTETS && is_slug_name(&game.slug),
            "`{}` must stay a grammar-valid, in-bound id",
            game.slug
        );
    }
}

// proves: design 024's Test strategy #4 (Chosen design: "run once per plugin over
// all rows that plugin's `discover()` collected") — the scope is the plugin, not
// the process: two plugins that each find the same title assign it the same base
// slug, and neither one's suffixing decides the other's.
#[test]
fn the_collision_scope_is_one_plugin_so_two_plugins_assign_the_same_base_slug() {
    let first = SteamRoot::new("collision-plugin-scope-a", false);
    let second = SteamRoot::new("collision-plugin-scope-b", false);
    first.acf("4501", "", Some("Hades"), "Hades");
    second.acf("4501", "", Some("Hades"), "Hades");

    let in_first = SteamLinuxPlugin::from_roots(vec![first.path().to_path_buf()])
        .discover()
        .expect("the first root is discoverable");
    let in_second = SteamLinuxPlugin::from_roots(vec![second.path().to_path_buf()])
        .discover()
        .expect("the second root is discoverable");

    assert_eq!(
        slugs(&in_first),
        vec!["hades"],
        "a plugin that holds one title gives it the bare base slug"
    );
    assert_eq!(
        slugs(&in_second),
        vec!["hades"],
        "a second plugin instance is not disturbed by the first one's assignment: the pass is \
         scoped to one plugin's own rows"
    );
}

// proves: design 024's Test strategy #4 + Goals (one game, one slug) — suffixing
// is bounded by grammar before it leaves the plugin: every assigned slug, suffixed
// or not, still passes the shared predicate that makes it a manifest id.
#[test]
fn every_slug_the_collision_pass_hands_out_is_a_valid_product_id() {
    let root = SteamRoot::new("collision-validity", false);
    root.acf("4601", "", Some("Polo"), "Polo");
    root.acf("4602", "", Some("Polo#"), "Polo Hash");
    root.acf("4603", "", Some("Polo$"), "Polo Dollar");

    let games = discover_all(&[&root]).expect("all three installs are discoverable");
    assert_eq!(
        slugs(&games),
        vec!["polo", "polo-2", "polo-3"],
        "the candidates are handed out in byte order of the names, skipping claimed ones"
    );
    for game in &games {
        assert!(
            is_slug_name(&game.slug),
            "`{}` — a suffixed slug must be as valid a product id as a base one",
            game.slug
        );
    }
}
