use lalrpop_util::lalrpop_mod;

mod ast;
lalrpop_mod!(
    #[allow(unused)]
    grammar,
    "/parser/grammar.rs"
);
mod lexer;
mod token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue() {
        let src =
r#"
title: test
---
//blah
===
"#;
        let lex = lexer::Lexer::new(src);
        let dialogue = grammar::DialogueParser::new()
            .parse(lex)
            .unwrap();
        dbg!(dialogue);
    }
}
