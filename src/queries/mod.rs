mod general;

use std::collections::BTreeMap;

use kunobi_jev::Entry;
use serde::Serialize;

#[derive(Default, Serialize)]
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

impl From<&Value> for Entry {
    fn from(value: &Value) -> Self {
        match value {
            Value::Null => Entry::Null,
            Value::Text(text) => Entry::Text(text.clone()),
            Value::Object(object) => {
                let value = serde_json::to_value(object).unwrap_or_default();
                let map = value.as_object().cloned().unwrap_or_default();
                Entry::Object(map)
            }
            Value::Array(array) => Entry::Array(
                array
                    .iter()
                    .map(serde_json::to_value)
                    .map(|r| r.unwrap_or_default())
                    .collect(),
            ),
            _ => Entry::Null, // Fallback for unsupported types
        }
    }
}

impl From<Entry> for Value {
    fn from(entry: Entry) -> Self {
        match entry {
            Entry::Null => Value::Null,
            Entry::Text(text) => Value::Text(text),
            Entry::Object(object) => Value::Object(
                object
                    .into_iter()
                    .map(|(k, v)| (k, Value::from(v)))
                    .collect(),
            ),
            Entry::Array(array) => Value::Array(array.into_iter().map(Value::from).collect()),
        }
    }
}

pub enum Statement {
    Question(String),
    Choice(String, Vec<(String, Value)>),
    Score(String, Vec<Value>),
}

impl From<&str> for Statement {
    fn from(s: &str) -> Self {
        Statement::Question(s.to_string())
    }
}

impl<const N: usize> From<(&str, [(&str, &str); N])> for Statement {
    fn from((question, choices): (&str, [(&str, &str); N])) -> Self {
        let choices = choices
            .iter()
            .map(|(label, value)| (label.to_string(), Value::Text(value.to_string())))
            .collect();
        Statement::Choice(question.to_string(), choices)
    }
}

impl<const N: usize> From<(&str, [&str; N])> for Statement {
    fn from((question, scores): (&str, [&str; N])) -> Self {
        let scores = scores
            .iter()
            .map(|score| Value::Text(score.to_string()))
            .collect();
        Statement::Score(question.to_string(), scores)
    }
}

pub struct Query {
    statements: BTreeMap<String, Statement>,
}

impl Query {
    pub fn new<S, T, I>(statements: S) -> Self
    where
        S: IntoIterator<Item = (T, I)>,
        I: Into<Statement>,
        T: Into<String>,
    {
        let statements = statements
            .into_iter()
            .map(|(key, statement)| (key.into(), statement.into()))
            .collect();
        Self { statements }
    }

    pub fn statements(&self) -> &BTreeMap<String, Statement> {
        &self.statements
    }
}

impl Default for Query {
    fn default() -> Self {
        general::general_queries()
    }
}

impl<S, T, I> From<S> for Query
where
    S: IntoIterator<Item = (T, I)>,
    I: Into<Statement>,
    T: Into<String>,
{
    fn from(statements: S) -> Self {
        Self::new(statements)
    }
}
