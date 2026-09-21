use crate::assess::PullRequestAssessment;
use thiserror::Error;

pub(crate) mod jev;

#[derive(Debug, Error)]
pub enum AssessmentError {
    #[error("JEV error: {0}")]
    JevError(#[from] jev::JevError),
}

pub trait Assessor {
    async fn assess(&self, pr: &PullRequestAssessment) -> Result<(), AssessmentError>;
}
