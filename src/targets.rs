use std::{fs, path};

use thiserror::Error;

pub(crate) struct Path {
    pub path: path::PathBuf,
    pub is_file: bool,
}

pub(crate) struct PullRequest {
    pub provider: String, // todo: enum
    pub owner: String,
    pub repo: String,
    pub pr_number: u32,
}

pub(crate) struct Other {
    pub identifier: String,
}

pub enum Target {
    Path(Path),
    PullRequest(PullRequest),
    Other(Other),
}

impl Target {
    pub fn get_identifier(&self) -> String {
        match self {
            Target::Path(t) => t.path.display().to_string(),
            Target::PullRequest(t) => {
                format!("{}:{}/{}#{}", t.provider, t.owner, t.repo, t.pr_number)
            }
            Target::Other(t) => t.identifier.clone(),
        }
    }

    pub fn as_pull_request(&self) -> Option<&PullRequest> {
        match self {
            Target::PullRequest(pr) => Some(pr),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum TargetError {
    #[error("Invalid target: {0}")]
    InvalidTarget(String),
}

pub fn parse_target(target: &str) -> Result<Target, TargetError> {
    if target.is_empty() {
        return Err(TargetError::InvalidTarget(
            "Target cannot be empty".to_string(),
        ));
    }

    Ok(
        if target.contains(":") && target.contains("#") && target.contains("/") {
            let (provider, owner, repo, pr_number): (String, String, String, u32) = {
                let parts: Vec<&str> = target.split(&[':', '/', '#'][..]).collect();
                if parts.len() < 4 {
                    return Err(TargetError::InvalidTarget(format!(
                        "Invalid pull request target: {}",
                        target
                    )));
                }

                (
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                    parts[3].parse::<u32>().map_err(|_| {
                        TargetError::InvalidTarget(format!(
                            "Invalid pull request number: {}",
                            parts[2]
                        ))
                    })?,
                )
            };

            Target::PullRequest(PullRequest {
                provider,
                owner,
                repo,
                pr_number,
            })
        } else if std::path::Path::new(target).exists() {
            Target::Path(Path {
                path: std::path::PathBuf::from(target),
                is_file: fs::metadata(target).map(|m| m.is_file()).unwrap_or(false),
            })
        } else {
            Target::Other(Other {
                identifier: target.to_string(),
            })
        },
    )
}
