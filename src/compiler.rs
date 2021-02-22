use std::collections::HashMap;

use crate::{
    LineInfo,
    parser,
    yarn_proto as proto,
};

pub fn compile_string(src: &str) -> (proto::Program, HashMap<String, LineInfo>) {
    let ast = parser::parse_string(src);
    let mut program = proto::Program::default();
    let mut string_table = HashMap::new();
    for node in &ast.nodes {
        let mut proto_node = proto::Node::default();
        for header in &node.headers {
            if header.key == "title" {
                proto_node.name = header.value.to_string();
            }
        }
        program.nodes.insert(proto_node.name.clone(), proto_node);
    }
    (program, string_table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile() {
        let src =
r#"
title: test
---
//blah
===
"#;
        let (program, string_table) = compile_string(src);
        dbg!(program, string_table);
    }
}
