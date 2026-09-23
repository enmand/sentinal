mod general;

use std::collections::BTreeMap;

use crate::value::Value;

#[derive(Debug)]
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
