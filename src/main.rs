mod cli;
mod collectors;
mod config;
mod display;
mod error;
mod models;
mod renderer;
mod state;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "chronicle")]
#[command(about = "Generate daily chronicles from Git, TODOs, and notes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configuration commands
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// State management commands
    State {
        #[command(subcommand)]
        command: StateCommands,
    },
    /// Generate a daily chronicle
    Gen {
        /// Path to config file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Date for the chronicle (defaults to today)
        #[arg(long)]
        date: Option<String>,

        /// Custom since timestamp (defaults to last run or 24h ago)
        #[arg(long)]
        since: Option<String>,

        /// Only collect from specific sources (git, todos, notes)
        #[arg(long)]
        only: Option<String>,

        /// Dry run - print to stdout instead of writing file
        #[arg(long)]
        dry_run: bool,
    },
    /// Show commands
    Show {
        #[command(subcommand)]
        command: ShowCommands,
    },
}

#[derive(Subcommand)]
enum ShowCommands {
    /// Display the most recent chronicle
    Latest {
        /// Path to config file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Initialize chronicle.toml configuration file
    Init {
        /// Path where to create the config file
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum StateCommands {
    /// Reset state tracking (clears all incremental update tracking)
    Reset {
        /// Path to the config file (defaults to chronicle.toml)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Config { command } => match command {
            ConfigCommands::Init { path } => cli::config::init(path),
        },
        Commands::State { command } => match command {
            StateCommands::Reset { config } => cli::state::reset(config),
        },
        Commands::Gen {
            config,
            date,
            since,
            only,
            dry_run,
        } => cli::gen::run(config, date, since, only, dry_run),
        Commands::Show { command } => match command {
            ShowCommands::Latest { config } => cli::show::latest(config),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
