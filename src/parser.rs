use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar, "/parser/grammar.rs");

mod ast;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue() {
        let src =
r#"
title: test
---
blah
===
"#;
        let dialogue = grammar::DialogueParser::new()
            .parse(src)
            .unwrap();
        dbg!(dialogue);
    }
}
