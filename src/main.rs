mod cli;
mod commands;
mod targets;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Assess { target, policy } => commands::assess(&target, policy.as_deref()),
    }
}
