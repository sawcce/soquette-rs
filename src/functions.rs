use crate::value::{Expression, Value};

#[derive(Clone, Debug)]
pub(crate) enum Statement {
    Assignment(String, Expression),
}

impl Statement {
    pub(crate) fn js(&self) -> String {
        use Statement::*;

        match self {
            Assignment(key, value) => format!("this.{key} = {};", value.js()),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Function {
    statements: Vec<Statement>,
}

impl Function {
    pub(crate) fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }

    pub(crate) fn js(&self) -> String {
        format!(
            "() => {{{}}}",
            self.statements
                .iter()
                .map(|statement| statement.js())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
