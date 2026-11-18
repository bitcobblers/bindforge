use clap::{Parser, Subcommand};
use discovery::engine::DiscoveryEngine;
use discovery::steam_linux::SteamLinuxPlugin;
use discovery::steam_windows::SteamWindowsPlugin;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "discovery")]
#[command(about = "Discovery CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs all enabled plugins and prints a table of discovered games.
    List {
        /// Outputs a JSON array of DiscoveredGame objects.
        #[arg(long)]
        json: bool,
        /// Steam root(s) to scan instead of probing this host. Repeatable.
        #[arg(long = "steam-root")]
        steam_root: Vec<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { json, steam_root } => {
            let mut engine = DiscoveryEngine::new();
            if steam_root.is_empty() {
                // Host probing: each plugin probes its own platform (§8, §3).
                engine.register_plugin(Box::new(SteamLinuxPlugin::from_host()));
                engine.register_plugin(Box::new(SteamWindowsPlugin::from_host()));
            } else {
                // `--steam-root` feeds the plugin whose host can produce tagged
                // rows; the row's platform tag is the plugin's, as on T-001's
                // seam (§8).
                #[cfg(not(windows))]
                engine.register_plugin(Box::new(SteamLinuxPlugin::from_roots(
                    steam_root.iter().cloned(),
                )));
                #[cfg(windows)]
                engine.register_plugin(Box::new(SteamWindowsPlugin::from_roots(
                    steam_root.iter().cloned(),
                )));
            }

            let all_games = engine.discover_all().unwrap();

            if *json {
                println!("{}", serde_json::to_string(&all_games).unwrap());
            } else {
                println!("| Slug | Display Name | Install Path | Platform |");
                for game in all_games {
                    println!(
                        "| {} | {} | {} | {} |",
                        game.slug,
                        game.display_name,
                        game.install_path.display(),
                        game.platform
                    );
                }
            }
        }
    }
}
