use std::collections::BTreeMap;

use regorus::{CompiledPolicy, Engine};
use thiserror::Error;

use crate::{thresholds::default_thresholds, value::Value};

const POLICY_ENTRYPOINT: &str = "data.sentinel.result";

fn default_data() -> Value {
    Value::Object(BTreeMap::from_iter([
        (
            "dispositions".to_string(),
            Value::Array(vec![
                Value::Text("proceed".to_string()),
                Value::Text("canary".to_string()),
                Value::Text("human-review".to_string()),
            ]),
        ),
        (
            "flags".to_string(),
            Value::Array(vec![
                Value::Text("security-review".to_string()),
                Value::Text("data-review".to_string()),
                Value::Text("api-review".to_string()),
                Value::Text("rollback-plan-required".to_string()),
            ]),
        ),
    ]))
}

pub struct PolicyEngine {
    policy: CompiledPolicy,
}

pub struct Policy {
    pub name: String,
    pub contents: String,
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Failed to load policy: {0}")]
    PolicyLoad(#[source] anyhow::Error),

    #[error("Failed to add data")]
    DataLoad(#[source] anyhow::Error),

    #[error("Failed to compile policy: {0}")]
    PolicyEngine(#[source] anyhow::Error),

    #[error("Failed to evaluate policy: {0}")]
    Evaluation(#[source] anyhow::Error),
}

impl PolicyEngine {
    pub fn new(policy: Policy, data: Option<Value>) -> Result<Self, PolicyError> {
        let mut engine = Engine::new();
        engine
            .add_policy(policy.name, policy.contents)
            .map_err(PolicyError::PolicyLoad)?;
        if let Some(provided_data) = data {
            engine
                .add_data(provided_data.into())
                .map_err(PolicyError::DataLoad)?;
        }
        engine
            .add_data(default_data().into())
            .map_err(PolicyError::DataLoad)?;
        engine
            .add_data(default_thresholds().into())
            .map_err(PolicyError::DataLoad)?;

        let policy = engine
            .compile_with_entrypoint(&POLICY_ENTRYPOINT.into())
            .map_err(PolicyError::PolicyEngine)?;
        Ok(Self { policy })
    }

    pub fn evaluate(&self, input: Value) -> Result<Value, PolicyError> {
        self.policy
            .eval_with_input(input.into())
            .map_err(PolicyError::Evaluation)
            .map(|v| v.into())
    }
}

impl From<Value> for regorus::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => regorus::Value::Null,
            Value::Text(text) => regorus::Value::from(text),
            Value::Number(num) => regorus::Value::from(num),
            Value::Bool(b) => regorus::Value::Bool(b),
            Value::Object(obj) => {
                let map: BTreeMap<regorus::Value, regorus::Value> = obj
                    .into_iter()
                    .map(|(k, v)| (regorus::Value::from(k), regorus::Value::from(v)))
                    .collect();
                regorus::Value::from(map)
            }
            Value::Array(arr) => {
                let vec: Vec<regorus::Value> = arr.into_iter().map(regorus::Value::from).collect();
                regorus::Value::from(vec)
            }
        }
    }
}

impl From<regorus::Value> for Value {
    fn from(value: regorus::Value) -> Self {
        match value {
            regorus::Value::Null | regorus::Value::Undefined => Value::Null,
            regorus::Value::String(text) => Value::Text(text.clone().to_string()),
            regorus::Value::Number(num) => Value::Number(num.as_f64().unwrap_or_default()),
            regorus::Value::Bool(b) => Value::Bool(b),
            regorus::Value::Object(map) => {
                let obj: BTreeMap<String, Value> = map
                    .clone()
                    .iter()
                    .map(|(k, v)| (k.clone().to_string(), Value::from(v.clone())))
                    .collect();
                Value::Object(obj)
            }
            regorus::Value::Array(arr) => {
                let vec: Vec<Value> = arr.iter().cloned().map(Value::from).collect();
                Value::Array(vec)
            }
            regorus::Value::Set(set) => {
                let vec: Vec<Value> = set.iter().cloned().map(Value::from).collect();
                Value::Array(vec)
            }
        }
    }
}
