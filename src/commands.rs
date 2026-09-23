use anyhow::Result;
use clout::{debug, warn};
use std::{collections::BTreeMap, path::Path};

use crate::{
    assessors::{self, Artifact, Assessor, PullRequestAssessment},
    github::GithubClient,
    policy::{Policy, PolicyEngine},
    queries::Query,
    targets::{Target, parse_target},
    thresholds::Thresholds,
    value::Value,
};

pub async fn assess(
    target: &str,
    policy_path: &Path,
    inputs: Option<&Path>,
    thresholds: Thresholds,
    thresholds_file: Option<&Path>,
) -> Result<()> {
    let target = parse_target(target)?;

    clout::info!("Assessing target: {}", target.get_identifier());
    clout::info!("Using policy: {}", policy_path.display());
    clout::debug!("Using thresholds: {:?}", thresholds);

    let policy = tokio::fs::read_to_string(policy_path)
        .await
        .expect("Failed to read policy file");
    let policy_engine = PolicyEngine::new(
        Policy {
            name: policy_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            contents: policy,
        },
        Some(Value::Object(BTreeMap::from_iter([(
            "target".to_string(),
            Value::Text(target.get_identifier()),
        )]))),
    )?;

    debug!("Assessing target: {}", target.get_identifier());
    let jev = assessors::jev::Jev::new()?;

    match target {
        Target::PullRequest(pr_target) => {
            let mut gh = GithubClient::new()?;
            gh.auth().await?;
            let pr_details = gh.fetch_pr(&pr_target).await?;

            let artifact = Artifact::PullRequest(PullRequestAssessment::Github(
                (pr_details, pr_target).into(),
            ));
            let query = Query::default();
            let assessment = jev.assess(&artifact, &query).await?;
            let assessment_value: Value = assessment.into();
            let policy_result = policy_engine.evaluate(assessment_value)?;

            clout::info!("Policy evaluation result: {:?}", policy_result);
        }
        Target::Path(path_target) => {
            if path_target.is_file {
                let content = tokio::fs::read_to_string(&path_target.path).await?;
                jev.assess(&Artifact::Diff(content), &Query::default())
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
