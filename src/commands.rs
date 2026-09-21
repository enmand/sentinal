use anyhow::Result;
use clout::{debug, error, info, success, warn};
use std::path::Path;

use crate::{github::GithubClient, targets::parse_target};

pub async fn assess(target: &str, policy: Option<&Path>) -> Result<()> {
    let target = parse_target(target)?;

    debug!("Assessing target: {}", target.get_identifier());

    let mut gh = GithubClient::new()?;
    gh.auth().await?;

    match target {
        crate::targets::Target::PullRequest(ref pr_target) => {
            let pr = gh.fetch_pr(pr_target).await?;
        }
        _ => {
            warn!("Target is not a pull request, skipping GitHub fetch");
        }
    }

    Ok(())
}
