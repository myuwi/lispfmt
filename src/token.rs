#[derive(Debug)]
pub enum TokenKind {
    // Delims
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // Primitives
    Prefix,
    Symbol,
    Number,
    String,
    Boolean,

    // Trivia
    Whitespace,
    Comment,
}

#[derive(Debug)]
pub struct Token<'src> {
    pub kind: TokenKind,
    pub text: &'src str,
}

impl<'src> Token<'src> {
    pub fn new(kind: TokenKind, text: &'src str) -> Self {
        Self { kind, text }
    }
}
