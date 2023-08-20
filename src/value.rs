use std::collections::HashMap;
use std::fmt::Display;

use crate::Node;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
}

impl Value {
    pub(crate) fn js(&self) -> String {
        match self {
            Value::String(v) => format!("\"{v}\""),
            Value::Number(number) => format!("{number}"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(v) => write!(f, "{v}"),
            Value::Number(number) => write!(f, "{number}"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Add,
    Substract,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operation::Add => "+",
                Operation::Substract => "-",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Expression {
    Literal(Value),
    FormatString(Vec<Expression>),
    Variable(String),
    Operation(Box<Expression>, Operation, Box<Expression>),
}

impl Expression {
    pub(crate) fn js(&self) -> String {
        match self {
            Expression::Literal(ref value) => value.js(),
            Expression::Variable(ref key) => format!("this.{key}"),
            Expression::Operation(ref lhs, ref operation, ref rhs) => {
                format!("{} {operation} {}", lhs.js(), rhs.js())
            }
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
            &Expression::Operation(..) => todo!("Operation evaluation not supported yet!"),
        }
    }
}

impl Into<Node> for Expression {
    fn into(self) -> Node {
        Node::Expression(self)
    }
}
