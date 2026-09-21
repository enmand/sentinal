/// TargetType represents the type of target that can be used in a sentinal assessment.
pub enum TargetType {
    Path,
    PullRequest,
    Diff,
    Other
}

/// Target represents a target that can be used in a sentinal assessment.
pub struct Target {
    pub target_type: TargetType,
    pub identifier: String,
}

fn parse_target(target: &str) -> Target {
    if target.starts_with("pr/") {
        Target {
            target_type: TargetType::PullRequest,
            identifier: target[3..].to_string(),
        }
    } else if target.starts_with("diff/") {
        Target {
            target_type: TargetType::Diff,
            identifier: target[5..].to_string(),
        }
    } else if std::path::Path::new(target).exists() {
        Target {
            target_type: TargetType::Path,
            identifier: target.to_string(),
        }
    } else {
        Target {
            target_type: TargetType::Other,
            identifier: target.to_string(),
        }
    }
}
