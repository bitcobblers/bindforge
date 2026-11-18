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

/// The table's columns, left to right (ADR-022).
const COLUMNS: [&str; 4] = ["Slug", "Display Name", "Install Path", "Platform"];

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

/// One rendered table row: the four column cells, in [`COLUMNS`] order.
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

/// The four cells of one game's row.
fn row_cells(game: &DiscoveredGame) -> Row {
    [
        game.slug.clone(),
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
