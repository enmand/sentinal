use crate::{github::PullRequestDetails, queries::Query, targets::PullRequest};
use kunobi_jev::Entry;
use serde::Serialize;
use thiserror::Error;

pub(crate) mod jev;

#[derive(Debug, Error)]
pub enum AssessmentError {
    #[error("JEV error: {0}")]
    JevError(#[from] jev::JevError),
}

pub enum Assessment {
    PullRequest(PullRequestAssessment),
    Diff(String),
}

pub trait Assessor {
    async fn assess(&self, artifact: &Assessment, query: &Query) -> Result<(), AssessmentError>;
}

pub enum PullRequestAssessment {
    Github(GithubPullRequestAssessment),
}

#[derive(Serialize)]
pub struct GithubPullRequestAssessment {
    owner: String,
    repo: String,
    pr_number: u32,
    title: Option<String>,
    description: Option<String>,
    changed_files: Vec<String>,
    diff: String,
}

impl From<(PullRequestDetails, PullRequest)> for GithubPullRequestAssessment {
    fn from(((pull, diff), request): (PullRequestDetails, PullRequest)) -> Self {
        Self {
            owner: request.owner,
            repo: request.repo,
            pr_number: request.pr_number,
            title: pull.title,
            description: pull.body,
            changed_files: diff
                .clone()
                .into_iter()
                .map(|entry| entry.filename)
                .collect(),
            diff: diff
                .into_iter()
                .map(|entry| entry.patch.unwrap_or_default())
                .collect::<Vec<String>>()
                .join("\n"),
        }
    }
}

impl TryFrom<&Assessment> for Entry {
    type Error = serde_json::Error;

    fn try_from(value: &Assessment) -> Result<Self, Self::Error> {
        match value {
            Assessment::PullRequest(PullRequestAssessment::Github(assessment)) => {
                let obj: serde_json::Map<String, serde_json::Value> =
                    serde_json::to_value(assessment)?
                        .as_object()
                        .cloned()
                        .unwrap_or_default();

                Ok(Entry::Object(obj))
            }
            Assessment::Diff(content) => Ok(content.into()),
        }
    }
}
