use uuid::Uuid;

use crate::*;

use super::{Component, Tag};

#[derive(Debug, Clone)]
pub(crate) enum Node {
    Tag(Tag),
    Fragment(Vec<Node>),
    Empty,
    Text(String),
    Expression(Expression),
    ComponentInvocation(Box<Component>, String, Option<HashMap<String, Value>>),
}

impl Node {
    pub(crate) fn component_invocation(component: &Component) -> Node {
        Node::ComponentInvocation(
            Box::new(component.clone()),
            format!("_{}", Uuid::new_v4().to_string().replace("-", "")),
            None,
        )
    }
    pub(crate) fn visit<T>(
        &self,
        predicate: impl Fn(&Node, Option<(Tag, usize)>) -> Vec<T>,
    ) -> Vec<T> {
        self.visitor(&predicate, None)
    }

    pub(crate) fn visitor<T>(
        &self,
        predicate: &impl Fn(&Node, Option<(Tag, usize)>) -> Vec<T>,
        parent: Option<(Tag, usize)>,
    ) -> Vec<T> {
        match self {
            Node::Tag(tag) => {
                let mut result = predicate(self, None);

                let mut n = match *tag.children.clone() {
                    Node::Fragment(ref children) => {
                        let mut result = vec![];

                        for (index, child) in children.iter().enumerate() {
                            println!("Child {child:?}");
                            result
                                .append(&mut child.visitor(predicate, Some((tag.clone(), index))));
                        }

                        result
                    }
                    _ => tag.children.visitor(predicate, Some((tag.clone(), 0))),
                };

                result.append(&mut n);

                result
            }
            _ => predicate(self, parent),
        }
    }
}
