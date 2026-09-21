mod cli;
mod commands;
mod github;
mod targets;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    clout::init()
        .with_verbose(4)
        .with_quiet(false)
        .with_silent(false)
        .with_use_color(clout::UseColor::Auto)
        .done()
        .expect("clout failed to init");

    match cli.command {
        Command::Assess { target, policy } => commands::assess(&target, policy.as_deref()).await,
    }
}
