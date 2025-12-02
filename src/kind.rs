#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SyntaxKind {
    // Tokens
    /// An opening parenthesis: `(`.
    LParen,
    /// A closing parenthesis: `)`.
    RParen,
    /// An opening curly brace: `{`.
    LBrace,
    /// A closing curly brace: `}`.
    RBrace,
    /// An opening bracket: `[`.
    LBracket,
    /// A closing bracket: `]`.
    RBracket,
    /// A symbol: `foo`, `bar`, `baz`.
    Symbol,
    /// A number: `10`, `3.1415`, `10e-3`, `0xFFFFFF`.
    Number,
    /// A quoted string: `"foo"`.
    String,
    /// A keyword: `:foo`.
    Keyword,
    /// A boolean: `true`, `false`.
    Boolean,
    /// A prefix: `'`, `#`.
    Prefix,
    /// A hash directive: `#!/usr/bin/env fennel`, `#lang racket`.
    HashDirective,
    /// End of input
    End,

    // Trivia
    /// A Newline
    Newline,
    /// Spaces
    Space,
    /// A comment: `; ...`.
    Comment,
    // TODO: Block comment

    // Nodes
    /// The root of a syntax tree
    Root,
    /// A list: `(print "hello")`.
    List,
    /// A sequence: `[1 2 3]`.
    Sequence,
    /// A table: `{:hello :world}`.
    Table,
    /// A key-value pair: `:hello :world`.
    Pair,
    /// An expression preceded by a prefix: `#(...)`.
    Prefixed,
}
