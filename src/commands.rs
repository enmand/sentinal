use anyhow::Result;
use clout::{debug, warn};
use std::path::Path;

use crate::{
    assessors::{self, Assessment, Assessor, PullRequestAssessment},
    github::GithubClient,
    queries::Query,
    targets::{Target, parse_target},
};

pub async fn assess(target: &str, policy: Option<&Path>) -> Result<()> {
    let target = parse_target(target)?;

    debug!("Assessing target: {}", target.get_identifier());

    let jev = assessors::jev::Jev::new()?;

    match target {
        Target::PullRequest(pr_target) => {
            let mut gh = GithubClient::new()?;
            gh.auth().await?;
            let pr_details = gh.fetch_pr(&pr_target).await?;
            jev.assess(
                &Assessment::PullRequest(PullRequestAssessment::Github(
                    (pr_details, pr_target).into(),
                )),
                &Query::default(),
            )
            .await?;
        }
        Target::Path(path_target) => {
            if path_target.is_file {
                let content = tokio::fs::read_to_string(&path_target.path).await?;
                jev.assess(&Assessment::Diff(content), &Query::default())
                    .await?;
            }

            warn!(
                "Path target assessment is not implemented yet for a directory: {}",
                path_target.path.display()
            );
        }
        Target::Other(other_target) => {
            warn!(
                "Other target assessment is not implemented yet: {}",
                other_target.identifier
            );
        }
    }

    Ok(())
}
