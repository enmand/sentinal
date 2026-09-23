use std::{collections::BTreeMap, str::FromStr};

use crate::value::Value;

#[derive(Debug, Clone)]
pub(crate) struct Threshold {
    /// The name of the threshold to evaluate.
    pub name: String,

    /// The value of the threshold to evaluate.
    pub value: f64,
}

impl FromStr for Threshold {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid threshold format: {}", s));
        }

        let name = parts[0].to_string();
        let value = parts[1]
            .parse::<f64>()
            .map_err(|_| format!("Invalid threshold value: {}", parts[1]))?;

        Ok(Threshold { name, value })
    }
}

#[derive(Debug, Clone)]
pub struct Thresholds {
    pub thresholds: BTreeMap<String, f64>,
}

impl<const N: usize> From<[(&str, f64); N]> for Thresholds {
    fn from(thresholds: [(&str, f64); N]) -> Self {
        let thresholds = thresholds
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect();
        Self { thresholds }
    }
}

impl From<Thresholds> for Value {
    fn from(thresholds: Thresholds) -> Self {
        Value::Object(BTreeMap::from_iter([(
            "thresholds".to_string(),
            Value::Object(BTreeMap::from_iter(
                thresholds
                    .thresholds
                    .into_iter()
                    .map(|(name, value)| (name, Value::Number(value))),
            )),
        )]))
    }
}

impl From<Vec<Threshold>> for Thresholds {
    fn from(thresholds: Vec<Threshold>) -> Self {
        let thresholds = thresholds
            .into_iter()
            .map(|t| (t.name, t.value))
            .collect::<BTreeMap<String, f64>>();
        Self { thresholds }
    }
}

pub(crate) fn default_thresholds() -> Value {
    Value::Object(BTreeMap::from_iter([(
        "thresholds".to_string(),
        Thresholds::from([
            ("security-review", 0.7),
            ("data-review", 0.75),
            ("api-review", 0.75),
            ("rollback-plan-required", 2.0),
        ])
        .into(),
    )]))
}
