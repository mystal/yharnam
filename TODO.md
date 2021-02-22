# TODO

* [ ] Split out compiler and parser into a separate `yharnam-compiler` crate.
  * And add a feature flag to `yharnam` that allows loading and compiling `.yarn` files using the compiler.

## Parser
* [x] Set up a Token enum.
* [ ] Whitespace-aware lexer
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

## Compiler
* [ ] Figure out how to walk the AST to spit out protobufs
