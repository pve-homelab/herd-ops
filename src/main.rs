//! Dev Team — Herdr plugin for Master-gated multi-agent coding loops.

mod app;
mod herdr;
mod plugins;
mod registry;
mod roles;
mod session;
mod storage;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "dev-team", version, about = "Dev Team Herdr plugin")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the interactive TUI (default when no subcommand).
    Tui,
    /// Print plugin id / version for smoke tests.
    Version,
    /// Run doctor checks non-interactively (JSON to stdout).
    Doctor {
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => app::run(),
        Commands::Version => {
            println!("dev-team {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Doctor { json } => plugins::doctor::run_cli(json),
    }
}
