use std::collections::HashMap;

use crate::{functions::Statement, value::Expression, Value};
use chumsky::error::Simple;
use chumsky::prelude::*;

#[derive(Debug)]
pub struct Module {
    name: String,
    body: Vec<ComponentDeclaration>,
}

#[derive(Debug)]
pub struct ComponentDeclaration {
    name: String,
    statements: Vec<Statement>,
    tree: Node,
}

#[derive(Debug, Clone)]
pub enum Node {
    Tag(HTMLTag),
    Expression(Expression),
    Empty,
}

#[derive(Debug, Clone)]
pub struct HTMLTag {
    name: String,
    children: Vec<Node>,
    properties: HashMap<String, String>,
}

impl HTMLTag {
    pub fn new(name: String, children: Vec<Node>, properties: HashMap<String, String>) -> Self {
        Self {
            name,
            children,
            properties,
        }
    }
}

fn expression() -> impl Parser<char, Expression, Error = Simple<char>> {
    let var_seg = just('$')
        .then(text::ident())
        .map(|(_, ident)| Expression::Literal(Value::Variable(ident)));

    let segment = filter(|c: &char| *c != '"' && *c != '$')
        .repeated()
        .at_least(1)
        .map(|str| Expression::Literal(Value::String(str.iter().collect())))
        .or(var_seg);

    let format_str = segment
        .repeated()
        .map(|segs| Expression::FormatString(segs))
        .delimited_by(just('"'), just('"'))
        .labelled("Format string");

    value()
        .map(|value| Expression::Literal(value))
        .or(format_str)
        .labelled("Expression")
}

fn mustache_expr() -> impl Parser<char, Node, Error = Simple<char>> {
    expression()
        .delimited_by(just('{'), just('}'))
        .map(|expr| Node::Expression(expr))
        .labelled("Mustache expression")
}

// TODO: Handle when opening tag doesn't match closing tag and vice-versa
fn tag() -> impl Parser<char, HTMLTag, Error = Simple<char>> {
    recursive(|tag_r| {
        let identifier = text::ident().padded();

        let property = identifier
            .then(just('='))
            .then(
                filter(|c: &char| *c != '"')
                    .repeated()
                    .delimited_by(just('"'), just('"'))
                    .map(|s| s.iter().collect::<String>()),
            )
            .map(|((key, _), value)| (key, value));

        let opening_tag = just('<')
            .then(identifier)
            .then(property.repeated())
            .then(just('>'))
            .map(|(((_, name), properties), _)| (name, properties));
        let closing_tag = just("</")
            .then(identifier)
            .then(just('>'))
            .map(|_| ())
            .labelled("Closing tag");

        opening_tag
            .padded()
            .then(
                tag_r
                    .map(|tag| Node::Tag(tag))
                    .or(mustache_expr())
                    .padded()
                    .repeated()
                    .or(empty().padded().to(Vec::new())),
            )
            .then(closing_tag.padded())
            .map(|(((name, properties), children), _)| {
                HTMLTag::new(name, children, HashMap::from_iter(properties))
            })
            .labelled("HTML Tag")
    })
}

fn value() -> impl Parser<char, Value, Error = Simple<char>> {
    text::int(10)
        .map(|number: String| Value::Number(number.parse::<i64>().unwrap() as f64))
        .or(text::ident().map(|ident| Value::Variable(ident)))
        .labelled("Value")
}

pub fn parser() -> impl Parser<char, Module, Error = Simple<char>> {
    let module_ident = just("module ").padded();
    let identifier = text::ident().padded();

    let component_ident = just("component ").padded();

    let args = || just('(').padded().then(just(')').padded());
    let state_declaration = just("state ")
        .padded()
        .then(identifier)
        .then(just('='))
        .then(value().padded())
        .map(|(((_, name), _), value)| Statement::Assignment(name, Expression::Literal(value)));

    let fn_body = state_declaration.padded().repeated().then(
        tag()
            .map(|tag| Node::Tag(tag))
            .or(empty().padded().to(Node::Empty)),
    );

    let component = component_ident
        .then(identifier)
        .then(args().to(()))
        .then(fn_body.delimited_by(just('{'), just('}')))
        .map(|(((_, name), _), (statements, tree))| {
            return ComponentDeclaration {
                name,
                statements,
                tree,
            };
        });

    let module = module_ident
        .then(identifier)
        .then(component.padded().repeated())
        .map(|((_, name), body)| Module { name, body });

    module.then_ignore(end())
}
