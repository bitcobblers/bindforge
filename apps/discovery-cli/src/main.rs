//! `discovery list` — the discovery CLI of ADR-025's shape.
//!
//! The scan flow is design 025's fixed order: the catalog gate first (a probe
//! of the configured endpoint before any plugin is constructed, `025-L1` with
//! `026-L1`'s amended host rule), then
//! the per-user match store load, then enumeration through the crate's
//! plugins, then the `025-B1` bound-only emit boundary, then one candidate
//! query per distinct unresolved slug (`025-L4`), then — in table mode only —
//! the numbered confirmation prompt and a save when at least one candidate was
//! confirmed (`025-L2`). The table and JSON renderings carry the five ADR-025
//! columns / eight row keys; this scan's own renderings never echo a just-
//! confirmed id — the confirmation lands in the store file and the row prints
//! `-` until a later scan re-reads it.
//!
//! Every failure path is exactly one `error: ` line naming the seam that
//! refused; the plugin's failure quotes the `DiscoveryError` Debug shape and
//! never a row's contents.

mod bindings;
mod catalog;
mod matches;

use catalog::{Candidate, CatalogClient, CatalogError, HttpCatalog, partition_candidates};
use clap::{Parser, Subcommand};
use discovery::game::DiscoveredGame;
use discovery::plugin::DiscoveryPlugin;
use discovery::steam_linux::SteamLinuxPlugin;
use discovery::steam_windows::SteamWindowsPlugin;
use matches::{MatchStore, echo_path, host_layout, store_path};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "discovery")]
#[command(about = "Discovery CLI", long_about = None)]
struct Cli {
    /// Outputs the findings as a JSON array of rows instead of the table.
    /// Accepted before or after the `list` subcommand. Never prompts.
    #[arg(long, global = true)]
    json: bool,
    /// The catalog endpoint: a literal `http://` URL, any host (`026-L1`
    /// retires the loopback-only allowlist, so a remote registry is admitted and
    /// every queried source key is sent to it in clear text). Falls back to
    /// `BINDFORGE_CATALOG_URL`.
    #[arg(long = "catalog-url", global = true)]
    catalog_url: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs all enabled plugins and prints a table of discovered games.
    List {
        /// Steam root(s) to scan instead of probing this host. Repeatable.
        #[arg(long = "steam-root")]
        steam_root: Vec<PathBuf>,
    },
    /// Prints the key bindings the located user config files state for one
    /// discovered game (ADR-017, `017-C4`). Local only: no catalog, no store.
    Bindings {
        /// The discovery lookup key, exactly as `list` printed it (`steam/…`).
        slug: String,
        /// Steam root(s) to scan instead of probing this host. Repeatable.
        #[arg(long = "steam-root")]
        steam_root: Vec<PathBuf>,
        /// Prints the codec claim — family, evidence, user and the absolute
        /// paths it located — and extracts nothing.
        #[arg(long)]
        probe: bool,
    },
}

/// The table's columns, left to right (ADR-025: the lookup key beside the
/// confirmed game id; the retired namespace column is gone).
const COLUMNS: [&str; 5] = [
    "Slug",
    "Game Id",
    "Display Name",
    "Install Path",
    "Platform",
];

fn main() -> ExitCode {
    let cli = Cli::parse();
    match &cli.command {
        Commands::List { steam_root } => list(cli.json, cli.catalog_url.as_deref(), steam_root),
        // ADR-017 (`017-C4`): the bindings face shares this binary's failure
        // voice and table renderer and nothing else — no catalog client, no
        // gate, no match store, no prompt (`017-L1`, and Options' A13
        // divergence from "the same scan").
        Commands::Bindings {
            slug,
            steam_root,
            probe,
        } => bindings::run(cli.json, slug, steam_root, *probe),
    }
}

/// The one failure voice: a single named `error: ` line, `ExitCode::FAILURE`.
pub(crate) fn refused(message: String) -> ExitCode {
    eprintln!("error: {message}");
    ExitCode::FAILURE
}

/// Runs `discovery list` end to end; every `ExitCode` below carries its own
/// single named stderr line or is the success path.
fn list(json: bool, catalog_url: Option<&str>, steam_root: &[PathBuf]) -> ExitCode {
    // Step 1 — the gate, before any plugin exists (`025-L1`): the endpoint
    // comes from the flag or the env var and from nothing else.
    let endpoint = catalog_url
        .map(str::to_string)
        .or_else(|| {
            std::env::var("BINDFORGE_CATALOG_URL")
                .ok()
                .map(|value| value.trim().to_string())
        })
        .filter(|value| !value.is_empty());
    let Some(endpoint) = endpoint else {
        return refused(
            CatalogError::transport_unavailable("no catalog endpoint is configured").line(),
        );
    };
    let catalog = match HttpCatalog::new(&endpoint) {
        Ok(catalog) => catalog,
        Err(error) => return refused(error.line()),
    };
    if let Err(error) = catalog.probe() {
        return refused(error.line());
    }

    // Step 2 — the store, before enumerating: a file this build cannot trust
    // refuses the run by name and is rewritten by nothing.
    let store_file = store_path(host_layout(), &|key: &str| std::env::var(key).ok());
    let mut store = match MatchStore::load(store_file) {
        Ok(store) => store,
        Err(error) => {
            return refused(format!(
                "match store unreadable: {}: {}",
                echo_path(&error.path),
                error.reason
            ));
        }
    };

    // The CLI owns its plugin list rather than borrowing the engine's:
    // `DiscoveryEngine::discover_all` reports a plugin failure on stderr and
    // still returns `Ok`, which leaves the CLI no way to exit non-zero in its
    // own voice (T-003 Chosen design §3).
    let mut plugins: Vec<Box<dyn DiscoveryPlugin>> = Vec::new();
    if steam_root.is_empty() {
        plugins.push(Box::new(SteamLinuxPlugin::from_host()));
        plugins.push(Box::new(SteamWindowsPlugin::from_host()));
    } else {
        #[cfg(not(windows))]
        plugins.push(Box::new(SteamLinuxPlugin::from_roots(
            steam_root.iter().cloned(),
        )));
        #[cfg(windows)]
        plugins.push(Box::new(SteamWindowsPlugin::from_roots(
            steam_root.iter().cloned(),
        )));
    }

    let mut all_games: Vec<DiscoveredGame> = Vec::new();
    for plugin in &plugins {
        match plugin.discover() {
            Ok(mut games) => all_games.append(&mut games),
            // Debug, not Display: the line carries `PluginError` and the roots
            // the plugin named, and a plugin message never carries row content.
            Err(error) => {
                return refused(format!("plugin {} failed: {error:?}", plugin.name()));
            }
        }
    }

    // Step 3 — the emit boundary: the `025-B1` bound alone. A refused row is
    // one named line, the valid rows of the run still render.
    for line in refuse_invalid_slugs(&mut all_games) {
        eprintln!("{line}");
    }

    // Step 4 — one query per distinct unresolved slug, byte order (`025-L4`).
    // This scan's resolutions are snapshotted here: a confirmation earned
    // below renders as `-` in this run and as the id only in a later scan.
    let resolved: BTreeMap<String, String> = all_games
        .iter()
        .filter_map(|game| {
            store
                .game_id(&game.slug)
                .map(|game_id| (game.slug.clone(), game_id.to_string()))
        })
        .collect();
    let unresolved: BTreeSet<String> = all_games
        .iter()
        .map(|game| game.slug.clone())
        .filter(|slug| !store.resolves(slug))
        .collect();

    // The friendly name a confirmation stores: the first row sharing the slug
    // in the plugin's `(slug, install_path)` order.
    let friendly_name_of = |slug: &str| {
        all_games
            .iter()
            .find(|game| game.slug == slug)
            .map(|game| game.friendly_name.clone())
            .unwrap_or_default()
    };

    let mut candidates_by_slug: BTreeMap<String, Vec<Candidate>> = BTreeMap::new();
    let mut confirmed_any = false;
    let stdin = std::io::stdin();
    let mut answers = stdin.lock();
    let mut answers_open = true;
    for slug in &unresolved {
        let fetched = match catalog.candidates(slug) {
            Ok(candidates) => candidates,
            // A mid-scan failure fails closed: no partial table (`025-C4`).
            Err(error) => return refused(error.line()),
        };
        let (kept, dropped) = partition_candidates(slug, &fetched);
        for line in dropped {
            eprintln!("{line}");
        }
        if kept.is_empty() {
            continue;
        }
        candidates_by_slug.insert(slug.clone(), kept.clone());
        // Step 5 — the prompt, table mode only (`025-L2`): stderr, one per
        // distinct unresolved slug, in byte order. A store with no data
        // directory can keep no confirmation, so it earns no prompt.
        if !json && store.writable() {
            print_prompt(slug, &kept);
            let answer = if answers_open {
                let mut line = String::new();
                match answers.read_line(&mut line) {
                    Ok(0) => {
                        answers_open = false;
                        String::new()
                    }
                    Ok(_) => line,
                    Err(_) => {
                        answers_open = false;
                        String::new()
                    }
                }
            } else {
                String::new()
            };
            if let Some(index) = selection(&answer, kept.len()) {
                let chosen = &kept[index];
                if store.confirm(slug, &chosen.game_id, &friendly_name_of(slug)) {
                    confirmed_any = true;
                }
            }
        }
    }

    // Step 6 — save only after at least one confirmation; the file itself
    // stands up the owner-only directory, so a run that confirms nothing
    // creates nothing.
    if !json
        && confirmed_any
        && let Err(error) = store.save()
    {
        return refused(format!(
            "match store unwritable: {}: {}",
            echo_path(&error.path),
            error.reason
        ));
    }

    // Step 7 — the rendering.
    if json {
        let rows: Vec<serde_json::Value> = all_games
            .iter()
            .map(|game| json_row(game, &resolved, &candidates_by_slug))
            .collect();
        match serde_json::to_string(&rows) {
            // Compact, byte-stable output; a serialization failure is a clean
            // error path, never a panic.
            Ok(document) => println!("{document}"),
            Err(error) => return refused(format!("failed to serialize games: {error}")),
        }
    } else {
        println!("{}", render_table(&all_games, &resolved));
    }
    ExitCode::SUCCESS
}

/// ADR-025's emission boundary (`025-B1`): the emit-side check is the bound
/// alone. The shared product-slug grammar is deliberately *not* applied — a
/// plugin row's lookup key `steam/<appid>` fails that grammar by design, and
/// the grammar is where a lookup key joins the catalog, not the store
/// (`025-L3`). An over-bound row is dropped with one named refusal line
/// quoting the escaped and truncated slug and no install path, while the
/// valid rows of the same run still render.
///
/// Returns the exact stderr lines (unprefixed by any newline) so the refusal
/// text is one named, testable value rather than an inline `eprintln`.
fn refuse_invalid_slugs(games: &mut Vec<DiscoveredGame>) -> Vec<String> {
    let mut refusals: Vec<String> = Vec::new();
    games.retain(|game| {
        let valid = game.slug.len() <= bindforge_core::MAX_DISCOVERY_SLUG_OCTETS;
        if !valid {
            refusals.push(format!(
                "error: refused game with invalid slug: {}",
                bindforge_core::escape_and_truncate(&game.slug)
            ));
        }
        valid
    });
    refusals
}

/// The prompt, on stderr — stdout is the table alone. `1`-based; blank, `n`,
/// out-of-range or EOF is no match.
fn print_prompt(slug: &str, candidates: &[Candidate]) {
    eprintln!("Discovered game `{slug}` has catalog candidates:");
    for (index, candidate) in candidates.iter().enumerate() {
        eprintln!(
            "  {}) {} — {}",
            index + 1,
            cell(candidate.game_id.clone()),
            cell(candidate.name.clone())
        );
    }
    eprint!("Enter the candidate's number, or blank for none: ");
    // The prompt is a question, not a log line: it must actually appear.
    let _ = std::io::stderr().flush();
}

/// Interprets one prompt answer against `count` candidates: a `1`-based
/// number inside the list, else `None` (blank, `n`, `0`, out of range, and
/// anything unparseable all decline).
fn selection(answer: &str, count: usize) -> Option<usize> {
    let trimmed = answer.trim();
    if trimmed.is_empty() {
        return None;
    }
    let number: usize = trimmed.parse().ok()?;
    (number >= 1 && number <= count).then(|| number - 1)
}

/// One rendered cell: values this build did not compose (ACF names, install
/// paths) pass the shared control-byte escape before reaching the terminal;
/// clean values render as themselves (design 025: hostile ACF bytes never
/// emit a raw control sequence).
pub(crate) fn cell(value: String) -> String {
    if value.chars().any(char::is_control) {
        bindforge_core::escape_and_truncate(&value)
    } else {
        value
    }
}

/// The eight-key JSON row (design 025 `025-C6`): the plugin's six fields plus
/// the store-resolved `game_id` (`null` when unmatched, never a confirmation
/// this run earned) and the per-row `candidates` (`[]` for a resolved slug).
fn json_row(
    game: &DiscoveredGame,
    resolved: &BTreeMap<String, String>,
    candidates_by_slug: &BTreeMap<String, Vec<Candidate>>,
) -> serde_json::Value {
    let mut row = serde_json::to_value(game).unwrap_or(serde_json::Value::Null);
    if let Some(object) = row.as_object_mut() {
        object.insert(
            "game_id".to_string(),
            match resolved.get(&game.slug) {
                Some(game_id) => serde_json::Value::String(game_id.clone()),
                None => serde_json::Value::Null,
            },
        );
        object.insert(
            "candidates".to_string(),
            serde_json::Value::Array(
                candidates_by_slug
                    .get(&game.slug)
                    .map(|candidates| {
                        candidates
                            .iter()
                            .map(|candidate| {
                                serde_json::json!({
                                    "game_id": candidate.game_id,
                                    "name": candidate.name,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            ),
        );
    }
    row
}

/// One rendered table row: one cell per column, in header order.
type Row = Vec<String>;

/// Renders `list`'s table: the five pinned columns through the shared width
/// algorithm in [`render_grid`].
///
/// A column is as wide as its widest cell measured in characters, every cell is
/// padded right to that width and framed by one space on each side, and the
/// separator fills the same columns, so the `|` characters of every rendered
/// line sit at identical byte offsets. An empty result renders the header
/// block only. This scan's confirmations are never rendered: `Game Id` is the
/// store as it was loaded, `-` when the store names no id for the row.
fn render_table(games: &[DiscoveredGame], resolved: &BTreeMap<String, String>) -> String {
    let rows: Vec<Row> = games.iter().map(|game| row_cells(game, resolved)).collect();
    render_grid(&COLUMNS.map(str::to_string), &rows)
}

/// The shared width algorithm, generalized from `list`'s fixed
/// `[String; COLUMNS.len()]` rows to any header and any number of columns
/// (design 017, `017-C4`): `list` renders through it byte-identically, and the
/// `bindings` and `--probe` faces render the same way without a second
/// renderer.
pub(crate) fn render_grid(header: &[String], rows: &[Row]) -> String {
    let widths = column_widths(header, rows);

    let mut lines: Vec<String> = Vec::with_capacity(rows.len() + 2);
    lines.push(render_row(header, &widths));
    lines.push(render_separator(&widths));
    lines.extend(rows.iter().map(|row| render_row(row, &widths)));
    lines.join("\n")
}

/// The cells of one game's row, in [`COLUMNS`] order.
fn row_cells(game: &DiscoveredGame, resolved: &BTreeMap<String, String>) -> Row {
    vec![
        cell(game.slug.clone()),
        resolved
            .get(&game.slug)
            .cloned()
            .or(Some("-".to_string()))
            .expect("the unmatched spelling is a constant"),
        cell(game.display_name.clone()),
        cell(game.install_path.display().to_string()),
        cell(game.platform.to_string()),
    ]
}

/// The width of each column: the greatest `chars().count()` of the header cell
/// and that column's body cells.
fn column_widths(header: &[String], rows: &[Row]) -> Vec<usize> {
    let mut widths: Vec<usize> = header.iter().map(|column| column.chars().count()).collect();
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }
    widths
}

/// The row under the header: per column, its `|` then dashes filling the whole
/// rendered gap (one space's worth on each side of the column), closed by a
/// final `|` — which is what keeps the separator's delimiters at the same
/// offsets as every row's.
fn render_separator(widths: &[usize]) -> String {
    widths
        .iter()
        .map(|width| format!("|{}", "-".repeat(width + 2)))
        .collect::<Vec<String>>()
        .join("")
        + "|"
}

/// One rendered row: per column, `"| "`, then the cell padded right to the
/// column's width, then one trailing space, closed by a final `|`. Every cell
/// is framed by one space on each side, so no content sits flush against a
/// delimiter.
fn render_row(cells: &[String], widths: &[usize]) -> String {
    cells
        .iter()
        .zip(widths)
        .map(|(cell, width)| format!("| {:width$} ", cell, width = *width))
        .collect::<Vec<String>>()
        .join("")
        + "|"
}

#[cfg(test)]
mod tests {
    use super::{DiscoveredGame, refuse_invalid_slugs, selection};
    use discovery::game::Platform;
    use std::path::PathBuf;

    /// One hand-built row for the emit-boundary tests. The install path is a
    /// distinctive constant so a refusal line can be asserted clean of it.
    fn row(slug: &str) -> DiscoveredGame {
        DiscoveredGame {
            friendly_name: "Some Game".to_string(),
            slug: slug.to_string(),
            display_name: "Some Game".to_string(),
            install_path: PathBuf::from("/install/paths-must-not-leak-here/game"),
            platform: Platform::SteamLinux,
            codec_id: None,
        }
    }

    const PREFIX: &str = "error: refused game with invalid slug: ";
    const LEAK_CANARY: &str = "paths-must-not-leak-here";

    #[test]
    fn a_lookup_key_row_survives_and_draws_no_refusal() {
        // ADR-025's reversal: `steam/7` fails the shared grammar and is emitted
        // all the same — the boundary is the bound alone (`025-B1`, `025-L3`).
        let mut games = vec![row("steam/7"), row("steam/900001")];
        let refusals = refuse_invalid_slugs(&mut games);
        assert!(refusals.is_empty(), "in-bound keys must not be refused");
        assert_eq!(
            games.iter().map(|g| g.slug.as_str()).collect::<Vec<_>>(),
            ["steam/7", "steam/900001"]
        );
    }

    #[test]
    fn a_slug_one_octet_past_the_bound_is_refused_not_truncated_into_a_row() {
        let past_cap = "a".repeat(bindforge_core::MAX_DISCOVERY_SLUG_OCTETS + 1);
        let mut games = vec![row(&past_cap)];
        let refusals = refuse_invalid_slugs(&mut games);
        assert!(games.is_empty(), "a past-bound slug never renders");
        // The bound itself is referenced, not retyped; the echo is exactly the
        // shared renderer's output for the slug, prefixed by the one name.
        assert_eq!(
            refusals,
            [format!(
                "{PREFIX}{}",
                bindforge_core::escape_and_truncate(&past_cap)
            )]
        );
        assert!(refusals[0].starts_with(PREFIX));
    }

    #[test]
    fn a_refusal_echo_is_escaped_and_never_carries_a_second_line_or_path() {
        // A newline inside a slug must reach stderr as a `\uXXXX` escape,
        // never as a second line, and no line may name the install path.
        let past_cap = format!(
            "{}\0x",
            "a".repeat(bindforge_core::MAX_DISCOVERY_SLUG_OCTETS)
        );
        let mut games = vec![row(&past_cap)];
        let refusals = refuse_invalid_slugs(&mut games);
        assert_eq!(games.len(), 0, "the over-bound row is dropped");
        for line in &refusals {
            assert!(!line.contains('\n'), "one refusal is one line: {line:?}");
            assert!(
                !line.contains(LEAK_CANARY),
                "a refusal names no path: {line:?}"
            );
        }
        assert_eq!(
            refusals,
            [format!(
                "{PREFIX}{}",
                bindforge_core::escape_and_truncate(&past_cap)
            )]
        );
    }

    #[test]
    fn in_bound_rows_survive_while_the_over_bound_ones_refused() {
        let past_cap = "a".repeat(bindforge_core::MAX_DISCOVERY_SLUG_OCTETS + 1);
        let mut games = vec![
            row("steam/7"),
            row("valid-game"),
            row(&past_cap),
            row("steam/900100"),
        ];
        let refusals = refuse_invalid_slugs(&mut games);
        assert_eq!(
            games.iter().map(|g| g.slug.as_str()).collect::<Vec<_>>(),
            ["steam/7", "valid-game", "steam/900100"]
        );
        assert_eq!(refusals.len(), 1);
        assert!(refusals[0].starts_with(PREFIX));
    }

    #[test]
    fn the_selection_is_one_based_and_in_range_only() {
        assert_eq!(selection("1\n", 2), Some(0));
        assert_eq!(selection(" 2 ", 2), Some(1), "padding is ignored");
        assert_eq!(selection("", 2), None, "EOF/blank is no match");
        assert_eq!(selection("\n", 2), None);
        assert_eq!(selection("n\n", 2), None, "`n` is no match");
        assert_eq!(selection("0\n", 2), None, "the list is 1-based");
        assert_eq!(selection("3\n", 2), None, "past the count is no match");
        assert_eq!(selection("two\n", 2), None, "unparseable is no match");
        assert_eq!(selection("1\n", 0), None, "nothing to choose from");
    }
}
