use std::{str::CharIndices};

use crate::parser::token::Token;

fn is_identifier_start(c: char) -> bool {
    match c {
        '_' | 'a'..='z' | 'A'..='Z' => true,
        _ => false,
    }
}

fn is_identifier_rest(c: char) -> bool {
    match c {
        '0'..='9' => true,
        c => is_identifier_start(c),
    }
}

pub struct Location {
    
}

pub struct Span {
    start: usize,
    end: usize,
}

pub type SpannedToken<'input> = (usize, Token<'input>, usize);

#[derive(Debug)]
pub enum LookaheadKind {
    Char(char),
    EndOfFile,
}

#[derive(Debug)]
pub struct Lookahead {
    kind: LookaheadKind,
    loc: usize,
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char),
}

#[derive(Debug)]
enum LexerMode {
    Header,
    HeaderValue,
    Body,
}

pub struct Lexer<'input> {
    input: &'input str,
    chars: CharIndices<'input>,
    mode: LexerMode,
    lookahead: Option<(usize, char)>,
    /// Whether the current line is all whitespace or comments.
    is_empty_line: bool,
    eof_location: isize,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        let mut chars = input.char_indices();
        Self {
            input,
            mode: LexerMode::Header,
            lookahead: chars.next(),
            chars,
            is_empty_line: true,
            eof_location: -1,
        }
    }

    pub fn bump(&mut self) -> Option<(usize, char)> {
        match self.lookahead {
            Some((loc, c)) => {
                self.lookahead = self.chars.next();
                if self.lookahead.is_none() {
                    self.eof_location = (loc + 1) as isize;
                }
                Some((loc, c))
            }
            None => None,
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> &'input str {
        &self.input[start..end]
    }

    pub fn test_lookahead<F>(&self, mut test: F) -> bool
    where
        F: FnMut(char) -> bool,
    {
        self.lookahead.map_or(false, |(_, c)| test(c))
    }

    pub fn line_comment(&mut self) {
        // Skip this line since it's just a comment.
        while let Some((loc, c)) = self.lookahead {
            if c == '\n' {
                return;
            }
            self.bump();
        }
    }

    pub fn rest_of_line(&mut self, start: usize, trim_whitespace: bool) -> (usize, &'input str) {
        while let Some((loc, c)) = self.lookahead {
            if c == '\n' {
                return (loc, self.slice(start, loc));
            }
            self.bump();
        }

        (self.eof_location as usize, self.slice(start, self.eof_location as usize))
    }

    pub fn identifier(&mut self, start: usize) -> (usize, &'input str) {
        while let Some((loc, c)) = self.lookahead {
            if !is_identifier_rest(c) {
                return (loc, self.slice(start, loc));
            }
            self.bump();
        }

        (self.eof_location as usize, self.slice(start, self.eof_location as usize))
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<SpannedToken<'input>, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((start, c)) = self.bump() {
            if c == '\n' {
                if self.is_empty_line {
                    // If we have a line of just whitespace, then don't emit a newline token.
                    continue;
                } else {
                    // We're going to the start of a new line, so reset the empty line flag.
                    self.is_empty_line = true;
                    return Some(Ok((start, Token::NewLine, start + 1)));
                }
            }

            // If we see a comment, ignore the rest of the line.
            if c == '/' && self.test_lookahead(|la| la == '/') {
                self.line_comment();
                continue;
            }

            // The current character is not whitespace, a new line, or a comment, so clear the empty line flag.
            if !c.is_whitespace() {
                self.is_empty_line = false;
            }

            match self.mode {
                LexerMode::Header => {
                    if c.is_whitespace() {
                        continue;
                    }

                    // Check for "---"
                    if c == '-' {
                        // TODO: Actually try to lex '---' instead of assuming it...
                        let next = self.bump();
                        let is_dash = matches!(next, Some((_, '-')));
                        assert!(is_dash, "Expected a '-', got {:?}", next);
                        let next = self.bump();
                        let is_dash = matches!(next, Some((_, '-')));
                        assert!(is_dash, "Expected a '-', got {:?}", next);

                        self.mode = LexerMode::Body;
                        return Some(Ok((start, Token::BodyStart, start + 3)));
                    }

                    // Try match an identifier.
                    if is_identifier_start(c) {
                        let (end, identifier) = self.identifier(start);
                        let tok = Token::Identifier(identifier);
                        return Some(Ok((start, tok, end)));
                    }

                    if c == ':' {
                        self.mode = LexerMode::HeaderValue;
                        return Some(Ok((start, Token::HeaderDelimiter, start + 1)));
                    }

                    return Some(Err(LexerError::UnexpectedCharacter(c)));
                }
                LexerMode::HeaderValue => {
                    // Get a string until the end of the line and trim it.
                    let (end, value) = self.rest_of_line(start, true);
                    let tok = Token::HeaderValue(value);
                    self.mode = LexerMode::Header;
                    return Some(Ok((start, tok, end)));
                }
                LexerMode::Body => {
                    // TODO: Actually try to lex '===' instead of assuming it...
                    assert_eq!(c, '=', "Expected a '=', got {:?}", c);
                    let next = self.bump();
                    let is_equals = matches!(next, Some((_, '=')));
                    assert!(is_equals, "Expected a '=', got {:?}", next);
                    let next = self.bump();
                    let is_equals = matches!(next, Some((_, '=')));
                    assert!(is_equals, "Expected a '=', got {:?}", next);
                    self.mode = LexerMode::Body;
                    return Some(Ok((start, Token::BodyEnd, start + 3)));
                }
            }
        }

        // TODO: Consider creating and returning an EOF token?
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let src =
r#"
title: test
---
===
"#;
        let mut lex = Lexer::new(src);
        for res in lex {
            dbg!(res);
        }
    }
}
