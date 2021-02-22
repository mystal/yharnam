use lalrpop_util::lalrpop_mod;

mod ast;
lalrpop_mod!(
    #[allow(unused)]
    grammar,
    "/parser/grammar.rs"
);
mod lexer;
mod token;

pub fn parse_string(src: &str) -> ast::Dialogue {
    let lex = lexer::Lexer::new(src);
    grammar::DialogueParser::new()
        .parse(lex)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
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
