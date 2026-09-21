use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "sentinel",
    version,
    about = "Semantic predicates for software delivery"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Assess a target against a policy.
    Assess {
        /// Target to assess (path, repo, or other identifier).
        target: String,

        /// Path to the policy to evaluate against.
        #[arg(short, long)]
        policy: Option<PathBuf>,
    },
}
