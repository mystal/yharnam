# TODO

* [ ] Split out compiler and parser into a separate `yharnam-compiler` crate.
  * And add a feature flag to `yharnam` that allows loading and compiling `.yarn` files using the compiler.

## Parser
* [x] Set up a Token enum.
* [ ] Implement expressions
  * Operator precedence: https://github.com/lalrpop/lalrpop/pull/555
* Implement statements
  * [ ] line_statement
  * [ ] if_statement
  * [ ] set_statement
  * [ ] option_statement
  * [ ] shortcut_option_statement
  * [ ] call_statement
  * [ ] command_statement

### Lexer
* [ ] Try out [logos](https://docs.rs/logos/)
* [ ] Whitespace-aware, aka indents and dedents
* Modes from the ANTLR lexer
  * Root
    * Skip whitespace
    * If see "---", push BodyMode
    * If see ":", push HeaderMode
    * If see "#", push HashtagMode
  * Header
    * Return the rest of the current line as a token REST_OF_LINE and pop modes
  * Body
    * Ignore all whitespace and comments
    * If see "===", pop modes
    * If see "<<", push CommandMode
    * If see "{", push TextMode/ExpressionMode (an inline expression)
      * Starts lexing an Expression, and will pop out into TextMode
    * If we see anything else, push TextMode
  * Text
  * TextCommandOrHashtag
  * HashTag
  * Expression
    * If we see "}", pop mode
    * If we see ">>", pop mode twice to leave expression and command modes
  * Command
    * Has some keywords that are command-only.
  * CommandText
  * CommandID

## Compiler
* [ ] Figure out how to walk the AST to spit out protobufs
