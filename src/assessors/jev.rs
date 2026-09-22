use kunobi_jev::{Entry, Question, Questions, SystemOneRequest, choice, noul, score};
use secrecy::SecretString;
use serde::Deserialize;
use serde_env::from_env_with_prefix;
use thiserror::Error;

use crate::{
    assessors::{Assessment, Assessor},
    queries::{Query, Statement},
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
    async fn assess(
        &self,
        artifact: &Assessment,
        query: &Query,
    ) -> Result<(), crate::assessors::AssessmentError> {
        let entry: Entry = artifact
            .try_into()
            .map_err(JevError::EntryConversionError)?;

        let resp = self
            .client
            .system_one(SystemOneRequest::new(entry, query.into()))
            .await
            .map_err(JevError::ClientError)?;

        clout::info!("JEV response: {:#?}", resp);

        Ok(())
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
