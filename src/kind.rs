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
    /// A Lua-compatible number: `10`, `3.1415`, `10e-3`, `0xFFFFFF`.
    Number,
    /// A string, either quoted: `"foo"` or starting with a colon: `:bar`.
    String,
    /// A boolean: `true`, `false`.
    Boolean,
    Prefix,
    /// End of input
    End,

    // Trivia
    /// A Newline
    Newline,
    /// Spaces
    Space,
    /// A comment: `; ...`.
    Comment,

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
    /// A node prefixed by another node: `#(...)`.
    Prefixed,
}

impl SyntaxKind {
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            SyntaxKind::Newline | SyntaxKind::Space | SyntaxKind::Comment
        )
    }

    pub fn is_space(&self) -> bool {
        matches!(self, SyntaxKind::Space)
    }

    pub fn is_comment(&self) -> bool {
        matches!(self, SyntaxKind::Comment)
    }
}
