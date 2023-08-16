use std::collections::HashMap;
use std::fmt::Display;

use crate::Node;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    String(String),
}

impl Value {
    pub(crate) fn js(&self) -> String {
        match self {
            Value::String(v) => format!("\"{v}\""),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Expression {
    Literal(Value),
    FormatString(Vec<Expression>),
    Variable(String),
}

impl Expression {
    pub(crate) fn js(&self) -> String {
        match self {
            Expression::Literal(ref value) => value.js(),
            Expression::Variable(ref key) => format!("this.{key}"),
            Expression::FormatString(ref parts) => {
                let mut result = vec!["`".into()];

                for part in parts {
                    result.push(match part {
                        Expression::Literal(ref lit) => format!("{lit}"),
                        x => format!("${{{}}}", x.js()),
                    });
                }

                result.push("`".into());

                result.join("")
            }
        }
    }

    pub(crate) fn evaluate(&self, context: &HashMap<String, Value>) -> String {
        match self {
            &Expression::Literal(ref value) => format!("{value}"),
            &Expression::Variable(ref key) => format!("{}", context.get(&key.clone()).unwrap()),
            &Expression::FormatString(ref parts) => {
                let mut result = vec![];

                for part in parts {
                    result.push(part.evaluate(context));
                }

                result.join("")
            }
        }
    }
}

impl Into<Node> for Expression {
    fn into(self) -> Node {
        Node::Expression(self)
    }
}
