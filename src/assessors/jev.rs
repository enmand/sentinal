use std::collections::BTreeMap;

use kunobi_jev::{
    Entry, Question, Questions, SystemOneRequest, SystemOneResult, choice, noul, score,
};
use secrecy::SecretString;
use serde::Deserialize;
use serde_env::from_env_with_prefix;
use thiserror::Error;

use crate::{
    assessors::{Artifact, Assessment, AssessmentError, Assessor},
    queries::{Query, Statement},
    value::Value,
};

pub(crate) struct Jev {
    client: kunobi_jev::Client,
}

#[derive(Clone, Deserialize)]
pub struct JevConfig {
    #[serde(rename = "api_key")]
    pub api_key: Option<SecretString>,
}

#[derive(Debug, Error)]
pub enum JevError {
    #[error("Missing JEV API key")]
    MissingJevApiKey,

    #[error("JEV client error: {0}")]
    ClientError(#[from] kunobi_jev::Error),

    #[error("Unable to load config: {0}")]
    ConfigLoadError(#[from] serde_env::Error),

    #[error("unable to convert to Jev entry: {0}")]
    EntryConversionError(#[from] serde_json::Error),
}

impl Jev {
    pub fn new() -> Result<Self, JevError> {
        let config: JevConfig = from_env_with_prefix("JEV").map_err(JevError::ConfigLoadError)?;

        if config.api_key.is_none() {
            return Err(JevError::MissingJevApiKey);
        }

        let client = if let Some(api_key) = &config.api_key {
            kunobi_jev::Client::builder()
                .api_key(api_key.clone())
                .build()?
        } else {
            kunobi_jev::Client::new()?
        };

        Ok(Self { client })
    }
}

impl Assessor for Jev {
    async fn assess<'a>(
        &self,
        artifact: &'a Artifact,
        query: &'a Query,
    ) -> Result<Assessment<'a>, AssessmentError> {
        let entry: Entry = artifact
            .try_into()
            .map_err(JevError::EntryConversionError)?;

        let resp = self
            .client
            .system_one(SystemOneRequest::new(entry, query.into()))
            .await
            .map_err(JevError::ClientError)?;

        let assessment = Assessment::from((query, resp, artifact));

        Ok(assessment)
    }
}

impl<'a> From<(&'a Query, SystemOneResult, &'a Artifact)> for Assessment<'a> {
    fn from(value: (&'a Query, SystemOneResult, &'a Artifact)) -> Self {
        let (query, result, artifact) = value;
        Assessment {
            artifact,
            verdicts: BTreeMap::from_iter(
                result
                    .answers
                    .iter()
                    .zip(query.statements().iter())
                    .map(|(k, s)| (s.0.clone(), (s.1, k.1.into()))),
            ),
        }
    }
}

impl From<&Query> for Questions {
    fn from(query: &Query) -> Self {
        query
            .statements()
            .iter()
            .map(|s| {
                (
                    s.0,
                    match s.1 {
                        Statement::Question(q) => Question::Noul(noul(q)),
                        Statement::Choice(q, choices) => Question::Choice(choice(
                            q,
                            choices
                                .iter()
                                .map(|(label, value)| (label.clone(), Into::<Entry>::into(value))),
                        )),
                        Statement::Score(q, scores) => {
                            Question::Score(score(q, scores.iter().map(Into::<Entry>::into)))
                        }
                    },
                )
            })
            .map(|(k, v)| (k.clone(), v))
            .collect()
    }
}

impl From<&Value> for Entry {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Entry::Null,
            Value::Text(text) => Entry::Text(text.clone()),
            Value::Object(object) => {
                let value = serde_json::to_value(object).unwrap_or_default();
                let map = value.as_object().cloned().unwrap_or_default();
                Entry::Object(map)
            }
            Value::Array(array) => Entry::Array(
                array
                    .iter()
                    .map(serde_json::to_value)
                    .map(|r| r.unwrap_or_default())
                    .collect(),
            ),
            _ => Entry::Null, // Fallback for unsupported types
        }
    }
}

impl From<Entry> for Value {
    fn from(entry: Entry) -> Self {
        match entry {
            Entry::Null => Value::Null,
            Entry::Text(text) => Value::Text(text),
            Entry::Object(object) => Value::Object(
                object
                    .into_iter()
                    .map(|(k, v)| (k, Value::from(v)))
                    .collect(),
            ),
            Entry::Array(array) => Value::Array(array.into_iter().map(Value::from).collect()),
        }
    }
}
