use std::collections::BTreeMap;

use crate::{
    github::PullRequestDetails,
    queries::{Query, Statement},
    targets::PullRequest,
    value::Value,
};
use kunobi_jev::{Answer, ChoiceAnswer, Entry, NoulAnswer, ScoreAnswer};
use serde::Serialize;
use thiserror::Error;

pub(crate) mod jev;

#[derive(Debug, Error)]
pub enum AssessmentError {
    #[error("JEV error: {0}")]
    JevError(#[from] jev::JevError),
}

#[derive(Debug)]
pub enum Artifact {
    PullRequest(PullRequestAssessment),
    Diff(String),
}

pub trait Assessor {
    async fn assess<'a>(
        &self,
        artifact: &'a Artifact,
        query: &'a Query,
    ) -> Result<Assessment<'a>, AssessmentError>;
}

#[derive(Debug)]
pub enum PullRequestAssessment {
    Github(GithubPullRequestAssessment),
}

#[derive(Debug, Serialize)]
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

impl TryFrom<&Artifact> for Entry {
    type Error = serde_json::Error;

    fn try_from(value: &Artifact) -> Result<Self, Self::Error> {
        match value {
            Artifact::PullRequest(PullRequestAssessment::Github(assessment)) => {
                let obj: serde_json::Map<String, serde_json::Value> =
                    serde_json::to_value(assessment)?
                        .as_object()
                        .cloned()
                        .unwrap_or_default();

                Ok(Entry::Object(obj))
            }
            Artifact::Diff(content) => Ok(content.into()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Verdict {
    Question(f64),
    Choice(String, f64, Vec<(String, f64)>),
    Score(f64, f64, BTreeMap<usize, (Value, f64)>),
    Unknown(Value),
}

impl From<&Answer> for Verdict {
    fn from(answer: &Answer) -> Self {
        match answer {
            Answer::Noul(NoulAnswer { noul: score }) => Verdict::Question(*score),
            Answer::Choice(ChoiceAnswer {
                choice: label,
                confidence: score,
                probabilities: choices,
            }) => {
                let choices = choices
                    .iter()
                    .map(|(label, score)| (label.clone(), *score))
                    .collect();
                Verdict::Choice(label.clone(), *score, choices)
            }
            Answer::Score(ScoreAnswer {
                score,
                confidence,
                legend,
                probabilities,
            }) => {
                let scores = probabilities
                    .iter()
                    .zip(legend.iter())
                    .map(|(score, label)| (*label.0 as usize, (label.1.clone().into(), *score.1)))
                    .collect();
                Verdict::Score(*score, *confidence, scores)
            }
            Answer::Unknown(v) => Verdict::Unknown(v.clone().into()),
        }
    }
}

#[derive(Debug)]
pub struct Assessment<'a> {
    verdicts: BTreeMap<String, (&'a Statement, Verdict)>,
    artifact: &'a Artifact,
}

impl<'a> Assessment<'a> {
    pub fn verdicts(&self) -> &BTreeMap<String, (&'a Statement, Verdict)> {
        &self.verdicts
    }

    pub fn artifact(&self) -> &'a Artifact {
        self.artifact
    }
}
