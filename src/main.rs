use serde::Deserialize;
use std::collections::HashMap;

use chumsky::Parser;

pub(crate) mod gen;
pub use gen::*;

mod parser;

#[derive(Debug, Clone)]
struct Method {
    name: String,
    statements: Vec<Statement>,
}

#[derive(Debug)]
struct Module {
    functions: Vec<Method>,
    components: HashMap<String, gen::Component>,
}

impl Module {
    fn new(components: HashMap<String, gen::Component>, functions: Vec<Method>) -> Self {
        Self {
            components,
            functions,
        }
    }
}

#[derive(Debug)]
struct Project {
    id: String,
    modules: HashMap<String, Module>,
}

impl Project {
    fn new(config: ProjectConfig) -> Self {
        Project {
            id: format!("{}@{}", config.name, config.version),
            modules: HashMap::new(),
        }
    }

    fn add_module_from_file(mut self, module: parser::Module) -> Self {
        self.modules
            .entry(format!("{}.{}", self.id, module.name))
            .and_modify(|m| {})
            .or_insert(Module::new(HashMap::new(), Vec::new()));
        self
    }
}

#[derive(Deserialize)]
struct ProjectConfig {
    root: String,
    name: String,
    version: String,
}

fn main() {
    let dir = std::env::args().skip(1).next().unwrap_or("./".into());
    println!("{dir:?}");
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(format!("{dir}project.yaml")).unwrap())
            .unwrap();

    let mut project = Project::new(config);

    let p = parser::parser();

    let code = r#"module main
component Counter() {
   state count = 0
   <div>
    <p>Counter!</p>
    <button class="btn btn-primary" click="">
        {"Count: $count"}
    </button>
   </div>
}"#;
    let result = p.parse(code);
    println!("{result:#?}");

    let project = project.add_module_from_file(result.unwrap());
    println!("{:?}", project);
}
