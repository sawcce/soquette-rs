use std::collections::HashMap;
use std::fmt::Display;

use crate::Node;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Variable(String),
}

impl Value {
    pub(crate) fn js(&self) -> String {
        match self {
            Value::String(v) => format!("\"{v}\""),
            Value::Number(number) => format!("{number}"),
            Value::Variable(ref key) => format!("this.{key}"),
        }
    }

    pub(crate) fn evaluate(&self, context: &HashMap<String, Value>) -> String {
        match self {
            Value::String(v) => format!("{v}"),
            Value::Number(number) => format!("{number}"),
            Value::Variable(key) => format!("{}", context.get(key).unwrap().evaluate(context)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
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
pub enum Expression {
    Literal(Value),
    FormatString(Vec<Expression>),
    Operation(Box<Expression>, Operation, Box<Expression>),
}

impl Expression {
    pub(crate) fn js(&self) -> String {
        match self {
            Expression::Literal(ref value) => value.js(),
            Expression::Operation(ref lhs, ref operation, ref rhs) => {
                format!("{} {operation} {}", lhs.js(), rhs.js())
            }
            Expression::FormatString(ref parts) => {
                let mut result = vec!["`".into()];

                for part in parts {
                    result.push(match part {
                        Expression::Literal(ref lit) => format!("{}", lit.js()),
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
            &Expression::Literal(ref value) => value.evaluate(context),
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
