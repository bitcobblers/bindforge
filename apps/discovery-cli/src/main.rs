use clap::{Parser, Subcommand};
use discovery::game::DiscoveredGame;
use discovery::plugin::DiscoveryPlugin;
use discovery::steam_linux::SteamLinuxPlugin;
use discovery::steam_windows::SteamWindowsPlugin;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "discovery")]
#[command(about = "Discovery CLI", long_about = None)]
struct Cli {
    /// Outputs the findings as a JSON array of DiscoveredGame objects
    /// instead of the table. Accepted before or after the `list` subcommand.
    #[arg(long, global = true)]
    json: bool,
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
}

/// The table's columns, left to right (ADR-022, widened by ADR-024's namespace).
const COLUMNS: [&str; 5] = [
    "Slug",
    "Namespace",
    "Display Name",
    "Install Path",
    "Platform",
];

fn main() -> ExitCode {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { steam_root } => {
            // The CLI owns its plugin list rather than borrowing the engine's:
            // `DiscoveryEngine::discover_all` reports a plugin failure on
            // stderr and still returns `Ok`, which leaves the CLI no way to
            // exit non-zero in its own voice (T-003 Chosen design §3).
            let mut plugins: Vec<Box<dyn DiscoveryPlugin>> = Vec::new();
            if steam_root.is_empty() {
                // Host probing: each plugin probes its own platform (§8, §3).
                plugins.push(Box::new(SteamLinuxPlugin::from_host()));
                plugins.push(Box::new(SteamWindowsPlugin::from_host()));
            } else {
                // `--steam-root` feeds the plugin whose host can produce tagged
                // rows; the row's platform tag is the plugin's, as on T-001's
                // seam (§8).
                #[cfg(not(windows))]
                plugins.push(Box::new(SteamLinuxPlugin::from_roots(
                    steam_root.iter().cloned(),
                )));
                #[cfg(windows)]
                plugins.push(Box::new(SteamWindowsPlugin::from_roots(
                    steam_root.iter().cloned(),
                )));
            }

            // Aggregation appends in registration order, matching the engine;
            // the first failure is the run's failure.
            let mut all_games: Vec<DiscoveredGame> = Vec::new();
            for plugin in &plugins {
                match plugin.discover() {
                    Ok(mut games) => all_games.append(&mut games),
                    Err(error) => {
                        eprintln!("error: plugin {} failed: {:?}", plugin.name(), error);
                        return ExitCode::FAILURE;
                    }
                }
            }

            // ADR-024's emission boundary: see `refuse_invalid_slugs`.
            for line in refuse_invalid_slugs(&mut all_games) {
                eprintln!("{line}");
            }

            if cli.json {
                // Compact, byte-stable output; a serialization failure is a
                // clean error path, never a panic.
                match serde_json::to_string(&all_games) {
                    Ok(document) => println!("{document}"),
                    Err(error) => {
                        eprintln!("error: failed to serialize games: {error}");
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                println!("{}", render_table(&all_games));
            }

            ExitCode::SUCCESS
        }
    }
}

/// ADR-024's emission boundary (`024-L1`): every row that reaches a rendering —
/// from any `dyn DiscoveryPlugin`, including a future third-party one — carries
/// a slug in the shared grammar and inside the shared bound. An invalid row is
/// dropped from `games` and answered by one named refusal line quoting the
/// escaped and truncated slug and no install path, while the valid rows of the
/// same run still render. The shipped plugins refuse such a row one seam
/// earlier, so this check is the last line, not the only one.
///
/// Returns the exact stderr lines (unprefixed by any newline) so the refusal
/// text is one named, testable value rather than an inline `eprintln`.
fn refuse_invalid_slugs(games: &mut Vec<DiscoveredGame>) -> Vec<String> {
    let mut refusals: Vec<String> = Vec::new();
    games.retain(|game| {
        let valid = bindforge_core::is_slug_name(&game.slug)
            && game.slug.len() <= bindforge_core::MAX_SLUG_OCTETS;
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

/// One rendered table row: the five column cells, in [`COLUMNS`] order.
type Row = [String; COLUMNS.len()];

/// The rendered widths of each column, in [`COLUMNS`] order.
type Widths = [usize; COLUMNS.len()];

/// Renders the ADR-022 table: a header row, a separator row, then one row per
/// game (T-003 Chosen design §1).
///
/// A column is as wide as its widest cell measured in characters, every cell is
/// padded right to that width and framed by one space on each side, and the
/// separator fills the same columns, so the
/// `|` characters of every rendered line sit at identical byte offsets. An empty
/// result renders the header block only.
fn render_table(games: &[DiscoveredGame]) -> String {
    let rows: Vec<Row> = games.iter().map(row_cells).collect();
    let widths = column_widths(&rows);

    let mut lines: Vec<String> = Vec::with_capacity(rows.len() + 2);
    lines.push(render_row(&COLUMNS.map(str::to_string), &widths));
    lines.push(render_separator(&widths));
    lines.extend(rows.iter().map(|row| render_row(row, &widths)));
    lines.join("\n")
}

/// The cells of one game's row, in [`COLUMNS`] order.
fn row_cells(game: &DiscoveredGame) -> Row {
    [
        game.slug.clone(),
        game.namespace.to_string(),
        game.display_name.clone(),
        game.install_path.display().to_string(),
        game.platform.to_string(),
    ]
}

/// The width of each column: the greatest `chars().count()` of the header cell
/// and that column's body cells.
fn column_widths(rows: &[Row]) -> Widths {
    let mut widths = COLUMNS.map(|column| column.chars().count());
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
fn render_separator(widths: &Widths) -> String {
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
fn render_row(cells: &Row, widths: &Widths) -> String {
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
    use super::{DiscoveredGame, refuse_invalid_slugs};
    use discovery::game::{Namespace, Platform};
    use std::path::PathBuf;

    /// One hand-built row for the emit-boundary tests. The install path is a
    /// distinctive constant so a refusal line can be asserted clean of it.
    fn row(slug: &str) -> DiscoveredGame {
        DiscoveredGame {
            slug: slug.to_string(),
            namespace: Namespace::Unknown,
            display_name: "Some Game".to_string(),
            install_path: PathBuf::from("/install/paths-must-not-leak-here/game"),
            platform: Platform::SteamLinux,
            codec_id: None,
        }
    }

    const PREFIX: &str = "error: refused game with invalid slug: ";
    const LEAK_CANARY: &str = "paths-must-not-leak-here";

    #[test]
    fn a_valid_row_survives_and_draws_no_refusal() {
        let mut games = vec![row("valid-game")];
        let refusals = refuse_invalid_slugs(&mut games);
        assert!(refusals.is_empty(), "a valid row must not be refused");
        assert_eq!(
            games.iter().map(|g| g.slug.as_str()).collect::<Vec<_>>(),
            ["valid-game"]
        );
    }

    #[test]
    fn a_grammar_invalid_row_is_dropped_with_the_named_escaped_line() {
        // Uppercase and a space are outside the slug grammar; the renderer
        // passes them through unchanged, so the exact line is pinned whole.
        let mut games = vec![row("Bad Slug")];
        let refusals = refuse_invalid_slugs(&mut games);
        assert!(games.is_empty(), "the invalid row must be dropped");
        assert_eq!(refusals, [format!("{PREFIX}Bad Slug")]);
    }

    #[test]
    fn a_refusal_echo_is_escaped_and_never_carries_a_second_line_or_path() {
        // A newline inside the slug must reach stderr as a `\uXXXX` escape,
        // never as a second line, and no line may name the install path.
        let mut games = vec![row("evil\nslug")];
        let refusals = refuse_invalid_slugs(&mut games);
        assert_eq!(refusals, [format!("{PREFIX}evil\\u000aslug")]);
        for line in &refusals {
            assert!(!line.contains('\n'), "one refusal is one line: {line:?}");
            assert!(
                !line.contains(LEAK_CANARY),
                "a refusal names no path: {line:?}"
            );
        }
    }

    #[test]
    fn a_slug_one_octet_past_the_shared_bound_is_refused_not_truncated_into_a_row() {
        let past_cap = "a".repeat(bindforge_core::MAX_SLUG_OCTETS + 1);
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
    fn valid_rows_survive_while_the_invalid_ones_of_the_same_run_refused() {
        let past_cap = "a".repeat(bindforge_core::MAX_SLUG_OCTETS + 1);
        let mut games = vec![
            row("Bad Slug"),
            row("valid-game"),
            row("evil\nslug"),
            row("another-valid"),
            row(&past_cap),
        ];
        let refusals = refuse_invalid_slugs(&mut games);
        assert_eq!(
            games.iter().map(|g| g.slug.as_str()).collect::<Vec<_>>(),
            ["valid-game", "another-valid"]
        );
        assert_eq!(refusals.len(), 3);
        assert_eq!(refusals[0], format!("{PREFIX}Bad Slug"));
        assert_eq!(refusals[1], format!("{PREFIX}evil\\u000aslug"));
        assert_eq!(
            refusals[2],
            format!("{PREFIX}{}", bindforge_core::escape_and_truncate(&past_cap))
        );
        for line in &refusals {
            assert!(!line.contains('\n'), "one refusal is one line: {line:?}");
            assert!(
                !line.contains(LEAK_CANARY),
                "a refusal names no path: {line:?}"
            );
        }
    }
}
