use std::{fs, path};

use thiserror::Error;

/// TargetType represents the type of target that can be used in a sentinal assessment.
pub enum TargetType {
    Path,
    PullRequest,
    Diff,
    Other,
}

/// Target represents a target that can be used in a sentinal assessment.
pub struct Target2 {
    pub target_type: TargetType,
    pub identifier: String,
}

struct Path {
    pub path: path::PathBuf,
    pub is_file: bool,
}

impl Target for Path {
    fn get_identifier(&self) -> String {
        self.path.display().to_string()
    }
}

pub(crate) struct PullRequest {
    pub provider: String, // todo: enum
    pub owner: String,
    pub repo: String,
    pub pr_number: u32,
}

impl Target for PullRequest {
    fn get_identifier(&self) -> String {
        format!(
            "{}-{}/{}#{}",
            self.provider, self.owner, self.repo, self.pr_number
        )
    }
}

struct Other {
    pub identifier: String,
}

impl Target for Other {
    fn get_identifier(&self) -> String {
        self.identifier.clone()
    }
}

pub(crate) trait Target {
    fn get_identifier(&self) -> String;
}

#[derive(Debug, Error)]
pub enum TargetError {
    #[error("Invalid target: {0}")]
    InvalidTarget(String),
}

pub fn parse_target(target: &str) -> Result<Box<dyn Target>, TargetError> {
    if target.is_empty() {
        return Err(TargetError::InvalidTarget(
            "Target cannot be empty".to_string(),
        ));
    }

    Ok(if target.contains("#") && target.contains("/") {
        let (owner, repo, pr_number): (String, String, u32) = {
            let parts: Vec<&str> = target.split(&['/', '#'][..]).collect();
            if parts.len() < 3 {
                return Err(TargetError::InvalidTarget(format!(
                    "Invalid pull request target: {}",
                    target
                )));
            }

            (
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].parse::<u32>().map_err(|_| {
                    TargetError::InvalidTarget(format!("Invalid pull request number: {}", parts[2]))
                })?,
            )
        };

        Box::new(PullRequest {
            provider: "github".to_string(),
            owner,
            repo,
            pr_number,
        }) as Box<dyn Target>
    } else if std::path::Path::new(target).exists() {
        Box::new(Path {
            path: std::path::PathBuf::from(target),
            is_file: fs::metadata(target).map(|m| m.is_file()).unwrap_or(false),
        }) as Box<dyn Target>
    } else {
        Box::new(Other {
            identifier: target.to_string(),
        }) as Box<dyn Target>
    })
}
