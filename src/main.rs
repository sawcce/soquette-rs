use std::collections::HashMap;

use chumsky::Parser;

pub(crate) mod gen;
pub use gen::*;

mod parser;

#[derive(Debug)]
struct Module {
    components: HashMap<String, gen::Component>,
}

impl Module {
    fn new(components: HashMap<String, gen::Component>) -> Self {
        Self { components }
    }
}

#[derive(Debug)]
struct Project {
    id: String,
    modules: HashMap<String, Module>,
}

impl Project {
    fn new(id: String) -> Self {
        Project {
            id,
            modules: HashMap::new(),
        }
    }

    fn add_module_from_file(mut self, module: parser::Module) -> Self {
        self.modules
            .entry(format!("{}.{}", self.id, module.name))
            .and_modify(|m| {})
            .or_insert(Module::new(HashMap::new()));
        self
    }
}

fn main() {
    let p = parser::parser();

    let mut project = Project::new("test_project".into());

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
