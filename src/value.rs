use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Default, Debug, Serialize)]
pub enum Value {
    #[default]
    Null,
    Text(String),
    Number(f64),
    Bool(bool),
    Object(BTreeMap<String, Value>),
    Array(Vec<Value>),
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::String(text) => Value::Text(text),
            serde_json::Value::Object(object) => Value::Object(
                object
                    .into_iter()
                    .map(|(k, v)| (k, Value::from(v)))
                    .collect(),
            ),
            serde_json::Value::Array(array) => {
                Value::Array(array.into_iter().map(Value::from).collect())
            }
            serde_json::Value::Number(num) => Value::Number(num.as_f64().unwrap_or_default()),
            serde_json::Value::Bool(b) => Value::Bool(b),
        }
    }
}
