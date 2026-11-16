use clap::{Parser, Subcommand};
use discovery::engine::DiscoveryEngine;
use discovery::game::DiscoveredGame;
use discovery::plugin::DiscoveryPlugin;
use discovery::registry::DiscoveryError;

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
    },
}

struct DummyPlugin;
impl DiscoveryPlugin for DummyPlugin {
    fn name(&self) -> &'static str {
        "dummy-plugin"
    }
    fn discover(&self) -> Result<Vec<DiscoveredGame>, DiscoveryError> {
        Ok(vec![])
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { json } => {
            let mut engine = DiscoveryEngine::new();
            engine.register_plugin(Box::new(DummyPlugin));
            
            let all_games = engine.discover_all().unwrap();

            if *json {
                println!("{}", serde_json::to_string(&all_games).unwrap());
            } else {
                println!("| Slug | Display Name | Install Path | Platform |");
                for game in all_games {
                    println!(
                        "| {} | {} | {} | {:?} |",
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
