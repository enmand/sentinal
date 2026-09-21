use std::sync::Arc;

use kunobi_jev::reqwest::header::ACCEPT;
use octocrab::Octocrab;
use secrecy::SecretString;
use serde::Deserialize;
use serde_env::from_env;
use thiserror::Error;

use crate::targets::{PullRequest, TargetError};

// scopes needed to review PR details, and repo context
const SCOPES: &[&str] = &["repo", "read:org", "read:user"];

#[derive(Debug, Clone, Deserialize)]
struct GitHubClientConfig {
    personal_token: Option<SecretString>,
    user_access_token: Option<SecretString>,
    client_id: Option<SecretString>,
}

struct GitHubClient {
    config: GitHubClientConfig,
    octocrab: Option<Arc<Octocrab>>,
}

#[derive(Debug, Error)]
enum GitHubClientError {
    #[error("Missing GitHub token")]
    MissingGitHubToken,

    #[error("Multiple GitHub tokens provided")]
    MultipleGitHubTokens,

    #[error("Client ID is not supported")]
    ClientIdNotSupported,

    #[error("Unable to load config: {0}")]
    ConfigLoadError(#[from] serde_env::Error),

    #[error("Octocrab error: {0}")]
    OctocrabError(#[from] octocrab::Error),
}

impl GitHubClient {
    fn new() -> Result<Self, GitHubClientError> {
        let config = from_env()?;

        Ok(Self {
            config,
            octocrab: None,
        })
    }

    async fn auth(&mut self) -> Result<(), GitHubClientError> {
        let config = self.config.clone();
        let mut builder = Octocrab::builder();
        builder = match (
            config.personal_token,
            config.user_access_token,
            &config.client_id,
        ) {
            (Some(token), None, None) => builder.personal_token(token),
            (None, Some(token), None) => builder.user_access_token(token),
            (None, None, Some(client_id)) => {
                let crab = Octocrab::builder()
                    .base_uri("https://github.com")?
                    .add_header(ACCEPT, "application/json".to_string())
                    .build()?;
                let codes = crab.authenticate_as_device(client_id, SCOPES).await?;

                clout::info!(
                    "Please visit {} and enter the code: {}",
                    codes.verification_uri,
                    codes.user_code
                );
                let auth = codes.poll_until_available(&crab, client_id).await?;

                Octocrab::builder().oauth(auth)
            }
            _ => {
                return Err(GitHubClientError::MultipleGitHubTokens);
            }
        };

        let octocrab = Some(Arc::new(builder.build()?));

        self.octocrab = octocrab;
        Ok(())
    }

    fn fetch_pr(&self, pr: PullRequest) -> Result<String, TargetError> {
        // Implement the logic to fetch the pull request details from GitHub API
        // For now, we will just return a placeholder string
        Ok(format!(
            "Fetched PR: {}-{}/{}#{}",
            pr.provider, pr.owner, pr.repo, pr.pr_number
        ))
    }
}
