use anyhow::Result;
use clout::{debug, error, info, success, warn};
use std::path::Path;

use crate::{
    assess::{self, PullRequestAssessment},
    assessors,
    github::GithubClient,
    targets::parse_target,
};

pub async fn assess(target: &str, policy: Option<&Path>) -> Result<()> {
    let target = parse_target(target)?;

    debug!("Assessing target: {}", target.get_identifier());

    let mut gh = GithubClient::new()?;
    gh.auth().await?;

    let jev = assessors::jev::Jev::new()?;

    match target {
        crate::targets::Target::PullRequest(pr_target) => {
            let pr_details = gh.fetch_pr(&pr_target).await?;
            assess::pull_request(
                jev,
                PullRequestAssessment::Github((pr_details, pr_target).into()),
            )
            .await?;
        }
        _ => {
            warn!("Target is not a pull request, skipping GitHub fetch");
        }
    }

    Ok(())
}
