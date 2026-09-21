use chrono::TimeZone;
use std::sync::Arc;

use keyring::Entry;
use kunobi_jev::reqwest::header::ACCEPT;
use octocrab::{Octocrab, auth::OAuth, models::pulls};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_env::from_env_with_prefix;
use thiserror::Error;

use crate::targets::PullRequest;

const KEYRING_SERVICE: &str = "sentinal-cli";

// scopes needed to review PR details, and repo context
const SCOPES: &[&str] = &["repo", "read:org"];

#[derive(Clone, Deserialize)]
struct GithubClientConfig {
    pat: Option<SecretString>,
    uat: Option<SecretString>,
    #[serde(rename = "client_id")]
    client_id: Option<SecretString>,
}

pub(crate) struct GithubClient {
    config: GithubClientConfig,
    octocrab: Option<Arc<Octocrab>>,
}

#[derive(Debug, Error)]
pub(crate) enum GithubClientError {
    #[error("Missing Github token")]
    MissingGithubToken,

    #[error("None, or multiple Github tokens provided")]
    MultipleGithubTokens,

    #[error("Missing Octocrab instance")]
    MissingOctocrab,

    #[error("Client ID is not supported")]
    ClientIdNotSupported,

    #[error("Unable to load config: {0}")]
    ConfigLoadError(#[from] serde_env::Error),

    #[error("Octocrab error: {0}")]
    OctocrabError(#[from] octocrab::Error),

    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),
}

impl GithubClient {
    pub fn new() -> Result<Self, GithubClientError> {
        let config = from_env_with_prefix("GITHUB")?;

        Ok(Self {
            config,
            octocrab: None,
        })
    }

    pub async fn auth(&mut self) -> Result<(), GithubClientError> {
        let config = self.config.clone();
        let mut builder = Octocrab::builder();
        builder = match (config.pat, config.uat, &config.client_id) {
            (Some(token), None, None) => builder.personal_token(token),
            (None, Some(token), None) => builder.user_access_token(token),
            (None, None, Some(client_id)) => {
                if let Some(auth) = fetch_cached_github_auth() {
                    builder.oauth(auth)
                } else {
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
                    cache_github_auth(&auth)?;

                    Octocrab::builder().oauth(auth)
                }
            }
            _ => {
                if let Some(auth) = fetch_cached_github_auth() {
                    builder.oauth(auth)
                } else {
                    return Err(GithubClientError::MultipleGithubTokens);
                }
            }
        };

        let octocrab = Some(Arc::new(builder.build()?));

        self.octocrab = octocrab;
        Ok(())
    }

    pub(crate) async fn fetch_pr(
        &self,
        pr: &PullRequest,
    ) -> Result<pulls::PullRequest, GithubClientError> {
        // Implement the logic to fetch the pull request details from Github API
        // For now, we will just return a placeholder string
        self.octocrab
            .as_ref()
            .ok_or(GithubClientError::MissingOctocrab)?
            .pulls(pr.owner.clone(), pr.repo.clone())
            .get(pr.pr_number as u64)
            .await
            .map_err(GithubClientError::OctocrabError)
    }
}

fn cache_github_auth(auth: &OAuth) -> Result<(), keyring::Error> {
    let at_entry =
        Entry::new(KEYRING_SERVICE, "access-token").expect("Failed to create keyring entry");
    at_entry.set_secret(auth.access_token.expose_secret().as_bytes())?;

    let tt_entry =
        Entry::new(KEYRING_SERVICE, "token-type").expect("Failed to create keyring entry");
    tt_entry.set_secret(auth.token_type.as_bytes())?;

    let scope_entry = Entry::new(KEYRING_SERVICE, "scope").expect("Failed to create keyring entry");
    scope_entry.set_secret(auth.scope.join(",").as_bytes())?;

    if let Some(ref refresh_token) = auth.refresh_token {
        let rt_entry =
            Entry::new(KEYRING_SERVICE, "refresh-token").expect("Failed to create keyring entry");
        rt_entry.set_secret(refresh_token.expose_secret().as_bytes())?;
    }

    if let Some(expires_in) = auth.expires_in {
        let expiry_entry = Entry::new(KEYRING_SERVICE, "expires-at")?;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);
        expiry_entry.set_secret(expires_at.to_rfc3339().as_bytes())?;
    }

    if let Some(refresh_token_expires_in) = auth.refresh_token_expires_in {
        let rt_expiry_entry = Entry::new(KEYRING_SERVICE, "refresh-token-expires-at")?;
        let rt_expires_at =
            chrono::Utc::now() + chrono::Duration::seconds(refresh_token_expires_in as i64);
        rt_expiry_entry.set_secret(rt_expires_at.to_string().as_bytes())?;
    }

    Ok(())
}

fn fetch_cached_github_auth() -> Option<OAuth> {
    let at_entry = Entry::new(KEYRING_SERVICE, "access-token").ok()?;
    let access_token = at_entry.get_secret().ok()?;

    let tt_entry = Entry::new(KEYRING_SERVICE, "token-type").ok()?;
    let token_type = tt_entry.get_secret().ok()?;

    let scope_entry = Entry::new(KEYRING_SERVICE, "scope").ok()?;
    let scope = scope_entry.get_secret().ok()?;

    let refresh_token = Entry::new(KEYRING_SERVICE, "refresh-token")
        .ok()
        .and_then(|entry| entry.get_secret().ok());

    let expires_at = Entry::new(KEYRING_SERVICE, "expires-at")
        .ok()
        .and_then(|entry| entry.get_secret().ok())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&String::from_utf8_lossy(&s)).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let refresh_token_expires_at = Entry::new(KEYRING_SERVICE, "refresh-token-expires-at")
        .ok()
        .and_then(|entry| entry.get_secret().ok())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&String::from_utf8_lossy(&s)).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    Some(OAuth {
        access_token: SecretString::new(
            String::from_utf8_lossy(&access_token)
                .to_string()
                .into_boxed_str(),
        ),
        token_type: String::from_utf8_lossy(&token_type).to_string(),
        scope: String::from_utf8_lossy(&scope)
            .split(',')
            .map(|s| s.to_string())
            .collect(),
        refresh_token: refresh_token
            .map(|rt| SecretString::new(String::from_utf8_lossy(&rt).to_string().into_boxed_str())),
        expires_in: expires_at.map(|dt| (dt - chrono::Utc::now()).num_seconds() as usize),
        refresh_token_expires_in: refresh_token_expires_at
            .map(|dt| (dt - chrono::Utc::now()).num_seconds() as usize),
    })
}
