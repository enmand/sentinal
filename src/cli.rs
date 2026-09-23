use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::thresholds::Threshold;

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
        policy: PathBuf,

        /// Inputs to use for the policy evaluation
        #[arg(short, long)]
        inputs: Option<PathBuf>,

        /// Thresholds to use for the policy evaluation, in the format `name=value`. Can be specified multiple times.
        #[arg(short, long)]
        thresholds: Vec<Threshold>,

        /// Path to a file containing thresholds to use for the policy evaluation.
        #[arg(short('s'), long)]
        thresholds_file: Option<PathBuf>,
    },
}
