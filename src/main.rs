use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

mod value;
use value::*;
mod node;
use node::*;
mod functions;
use functions::*;

use uuid::Uuid;

struct Renderer {
    context: HashMap<String, Value>,
    instance_id: Option<String>,
    codegen: String,
}

impl Renderer {
    fn new(context: HashMap<String, Value>) -> Self {
        Self {
            context,
            instance_id: None,
            codegen: "".into(),
        }
    }

    fn new_instance(context: HashMap<String, Value>, instance_id: String) -> Self {
        Self {
            context,
            instance_id: Some(instance_id),
            codegen: "".into(),
        }
    }

    fn render(&mut self, root: &Node) -> String {
        match root {
            &Node::Empty => "".into(),
            &Node::Text(ref text) => text.clone(),
            &Node::Tag(ref tag) => {
                let children = self.render(&tag.children);
                format!(
                    "<{} id=\"{}\"{}{}>{}</{}>",
                    tag.name,
                    tag.id,
                    if self.instance_id.is_some() {
                        format!(" data-instance=\"{}\"", self.instance_id.clone().unwrap())
                    } else {
                        "".into()
                    },
                    if tag.properties.is_empty() {
                        "".into()
                    } else {
                        tag.properties
                            .clone()
                            .iter()
                            .map(|(k, v)| format!(" {k}=\"{v}\""))
                            .collect::<Vec<_>>()
                            .join("")
                    },
                    children,
                    tag.name
                )
            }
            &Node::Fragment(ref children) => {
                let mut result = vec![];

                for child in children {
                    result.push(self.render(&child));
                }

                result.join("\n")
            }
            &Node::Expression(ref expression) => expression.evaluate(&self.context.clone()),
            &Node::ComponentInvocation(ref component, ref id, ..) => {
                self.codegen = format!(
                    "{}\ndocument.__soquette__.static.push(new {}(\"{}\"))",
                    self.codegen,
                    component.id.clone(),
                    id.clone()
                );
                component.render(id.clone())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum EventType {
    Click,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        match self {
            EventType::Click => write!(f, "click"),
        }?;
        write!(f, "\"")
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EventListener {
    event_type: EventType,
    listener: Function,
}

impl EventListener {
    pub(crate) fn new(et: EventType, listener: Function) -> Self {
        Self {
            event_type: et,
            listener,
        }
    }
}

#[derive(Debug, Clone)]
struct Tag {
    id: String,
    name: String,
    properties: HashMap<String, String>,
    children: Box<Node>,
    listeners: Vec<EventListener>,
}

impl Tag {
    fn new(name: impl ToString, children: Node) -> Self {
        Self {
            name: name.to_string(),
            children: Box::new(children),
            id: format!("_{}", Uuid::new_v4().to_string().replace("-", "")),
            properties: HashMap::new(),
            listeners: Vec::new(),
        }
    }

    fn property(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    fn listener(mut self, et: EventType, function: Function) -> Self {
        self.listeners.push(EventListener::new(et, function));
        self
    }

    fn node(self) -> Node {
        Node::Tag(self)
    }
}

#[derive(Debug, Clone)]
struct Component {
    state: HashMap<String, Value>,
    tree: Node,
    id: String,
    refs: HashSet<String>,
    exprs: Vec<(String, usize, String)>,
    listeners: Vec<(String, EventListener)>,
}

impl Component {
    fn new(state: HashMap<String, Value>, tree: Node) -> Self {
        let mut refs = HashSet::new();

        let exprs = tree.visit(|n, parent| {
            println!("{n:?}");
            match n {
                Node::Expression(ref expr) => {
                    if let Some((parent, index)) = parent {
                        vec![(parent.id, index, expr.js())]
                    } else {
                        println!("Empty!");
                        vec![]
                    }
                }
                _ => Vec::new(),
            }
        });

        for expr in exprs.clone() {
            refs.insert(expr.0);
        }

        let listeners = tree.visit(|n, _| match n {
            Node::Tag(ref tag) => tag
                .listeners
                .iter()
                .map(|listener| (tag.id.clone(), listener.clone()))
                .collect(),
            _ => {
                vec![]
            }
        });

        for listener in listeners.clone() {
            refs.insert(listener.0);
        }

        Self {
            state,
            tree,
            id: format!("Soquette_{}", Uuid::new_v4().to_string().replace("-", "")),
            refs,
            exprs,
            listeners,
        }
    }

    fn generate_class(&self) -> String {
        let states = self
            .state
            .iter()
            .map(|(key, value)| format!("_{key} = {};", value.js()))
            .collect::<Vec<String>>()
            .join("\n");

        let ref_init = self
            .refs
            .iter()
            .map(|id| {
                format!(
                    "this.{} = document.querySelector(`#{}[data-instance=${{this.instanceID}}]`)",
                    id, id
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let expr_refresh = self
            .exprs
            .iter()
            .map(|(id, index, expr)| {
                format!("this.{}.childNodes[{}].textContent = {}", id, index, expr)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let listeners = self
            .listeners
            .iter()
            .map(|(id, listener)| {
                format!(
                    "{id}.addEventListener({}, {})",
                    listener.event_type,
                    listener.listener.js()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let setters = self
            .state
            .clone()
            .iter()
            .map(|(k, v)| {
                format!(
                    "set {k}(value) {{this._{k} = value; this.refresh(); }}
                 get {k}() {{ return this._{k}; }}"
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let constructor = format!(
            "
        constructor(instanceID) {{
            this.instanceID = instanceID;
            // Refs init
            {}
            {listeners}
        }}

        refresh() {{
            {}
        }}

        {setters}
        ",
            ref_init, expr_refresh
        );

        println!("Ref init {ref_init}");

        format!(
            include_str!("./class_template.js"),
            self.id, states, "", constructor
        )
    }

    fn render(&self, instance_id: String) -> String {
        let mut renderer = Renderer::new_instance(self.state.clone(), instance_id);

        renderer.render(&self.tree)
    }
}

fn main() {
    let mut greet_model = HashMap::new();
    greet_model.insert("name".into(), Value::String("world".into()));

    let greet = Component::new(
        greet_model,
        Tag::new(
            "div",
            Node::Fragment(vec![
                Tag::new(
                    "h1",
                    Expression::FormatString(vec![
                        Expression::Literal(Value::String("Hello, ".into())),
                        Expression::Variable("name".into()),
                        Expression::Literal(Value::String("!".into())),
                    ])
                    .into(),
                )
                .node(),
                Tag::new("button", Node::Text("Set name".into()))
                    .listener(
                        EventType::Click,
                        Function::new(vec![Statement::Assignment(
                            "name".into(),
                            Expression::Literal(Value::String("Sawcce".into())),
                        )]),
                    )
                    .node(),
            ]),
        )
        .node(),
    );

    let generated = greet.generate_class();
    println!("{generated:#}");

    let script = format!(
        "
    document.__soquette__ = {{version: \"0.0.1\", static: []}};
    {}",
        generated
    );

    let mut renderer = Renderer::new(HashMap::new());

    let tree = Tag::new(
        "html",
        Node::Fragment(vec![
            Tag::new(
                "head",
                Node::Fragment(vec![
                    Tag::new("title", Node::Text("Soquette".into())).node(),
                    Tag::new("script", Node::Text(script)).node(),
                    Tag::new("script", Node::Empty)
                        .property("src", "./sq.js")
                        .property("defer", "true")
                        .node(),
                ]),
            )
            .node(),
            Tag::new("body", Node::component_invocation(&greet)).node(),
        ]),
    )
    .node();

    println!("{tree:#?}");

    let result = renderer.render(&tree);
    std::fs::write("./sq.js", renderer.codegen.clone()).unwrap();
    println!("{result:#}");

    std::fs::write("./index.html", result).unwrap();
}
