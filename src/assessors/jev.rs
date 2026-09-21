use kunobi_jev::{Entry, Questions, SystemOneRequest, choice, noul, score};
use secrecy::SecretString;
use serde::Deserialize;
use serde_env::from_env_with_prefix;
use thiserror::Error;

use crate::{assess::PullRequestAssessment, assessors::Assessor};

pub(crate) struct Jev {
    config: JevConfig,
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

        Ok(Self { client, config })
    }
}

impl Assessor for Jev {
    async fn assess(
        &self,
        pr: &PullRequestAssessment,
    ) -> Result<(), crate::assessors::AssessmentError> {
        let mut questions = Questions::new();

        let persistent_state = questions.add(
            "persistent_state",
            noul(
                "Does this change alter the stored representation, constraints, interpretation, lifecycle, mutation semantics, or deletion semantics of persistent application data, rather than only changing how existing data is read or queried?"
            )
        );

        let observable_behavior = questions.add(
            "observable_behavior",
            noul(
                "Could this change alter behavior observable by users, callers, or other components, even if no public API or interface signature changes?"
            ),
        );

        let external_contract = questions.add(
            "external_contract",
            noul(
                "Does this change alter a documented or de facto external interface boundary, such as a public API signature, protocol, message format, event schema, serialized representation, command-line interface, or externally consumed configuration contract?"
            ),
        );

        let security_boundary = questions.add(
            "security_boundary",
            noul(
                "Does this change alter authentication, authorization, identity, permissions, trust boundaries, credential handling, or another security-sensitive boundary?"
            ),
        );

        let category = questions.add(
            "category",
            choice(
                "Which category best describes the primary operational nature of this change?",
                [
                    (
                        "behavior_change",
                        "Changes runtime behavior observable by users, callers, or other systems."
                    ),
                    (
                        "data_model_change",
                        "Changes persistent data structure, representation, constraints, interpretation, or lifecycle."
                    ),
                    (
                        "external_contract_change",
                        "Changes an API, protocol, event, message format, or another externally consumed interface."
                    ),
                    (
                        "infrastructure_change",
                        "Changes deployment, runtime infrastructure, networking, configuration, build, or operational environment."
                    ),
                    (
                        "security_change",
                        "Primarily changes authentication, authorization, identity, permissions, credentials, or trust boundaries."
                    ),
                    (
                        "internal_refactor",
                        "Changes internal implementation or code organization without intentionally changing observable runtime behavior."
                    ),
                    (
                        "observability_change",
                        "Primarily changes logging, metrics, tracing, monitoring, alerting, or diagnostics."
                    ),
                    (
                        "test_only",
                        "Changes only tests, fixtures, test data, or test infrastructure."
                    ),
                    (
                        "documentation_only",
                        "Changes only documentation, comments, examples, or other non-executable material."
                    ),
                ],
            ),
        );

        let blast_radius = questions.add(
            "blast_radius",
            score(
                "If this change contains a serious defect, how broad is the plausible production impact?",
                [
                    "No meaningful production impact.",
                    "Impact is limited to a narrow feature, code path, or small subset of users.",
                    "Impact could materially affect a major feature, service capability, or significant subset of users.",
                    "Impact could broadly affect the service, multiple components, or a large portion of users.",
                    "Impact could cause severe system-wide, security, financial, or data-integrity consequences.",
                ],
            ),
        );

        let restore_complexity = questions.add(
            "restore_complexity",
            score(
                "How operationally difficult would it be to restore the system to its pre-change state after this change has been deployed and exercised?",
                [
                    "Redeploying the previous code is sufficient.",
                    "Minor operational steps may be required, but rollback is routine.",
                    "Rollback requires coordinated steps, migration handling, or careful verification.",
                    "Production state may require repair, transformation, or cross-service coordination.",
                    "The previous state may not be reliably recoverable because information or state may have been irreversibly changed.",
                ],
            ),
        );

        let entry: Entry = pr.try_into().map_err(JevError::EntryConversionError)?;

        let resp = self
            .client
            .system_one(SystemOneRequest::new(entry, questions))
            .await
            .map_err(JevError::ClientError)?;

        // output each answer to the console
        clout::info!("JEV assessment results:");
        for (question_id, answer) in resp.answers.iter() {
            clout::info!("{}: {:?}", question_id, answer);
        }

        Ok(())
    }
}
