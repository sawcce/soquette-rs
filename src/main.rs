use std::collections::HashMap;

use chumsky::Parser;

pub(crate) mod gen;
pub use gen::*;

mod parser;

fn main() {
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
}
