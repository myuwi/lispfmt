use chumsky::span::SimpleSpan;

use crate::kind::SyntaxKind;

pub type Span = SimpleSpan;

pub enum SyntaxElement<'src> {
    Token(Token<'src>),
    Node(Node<'src>),
}

impl std::fmt::Debug for SyntaxElement<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxElement::Token(node) => {
                write!(
                    f,
                    "{:?}@{:?} \"{}\"",
                    node.kind,
                    node.span,
                    node.text.escape_debug()
                )
            }
            SyntaxElement::Node(node) => {
                writeln!(f, "{:?}@{:?}", node.kind, node.span)?;

                let children = node
                    .children
                    .iter()
                    .map(|child| {
                        format!("{:?}", child)
                            .lines()
                            .map(|s| format!("  {}", s))
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                write!(f, "{}", children)
            }
        }
    }
}

impl<'src> SyntaxElement<'src> {
    pub fn token(t: Token<'src>) -> Self {
        Self::Token(t)
    }

    pub fn node(kind: SyntaxKind, children: Vec<SyntaxElement<'src>>) -> Self {
        let span = match &children[..] {
            [first, .., last] => {
                let span_start = first.span().start;
                let span_end = last.span().end;
                (span_start..span_end).into()
            }
            [node] => node.span(),
            [] => (0..0).into(),
        };

        Self::Node(Node::new(kind, children, span))
    }
}

impl SyntaxElement<'_> {
    pub fn kind(&self) -> &SyntaxKind {
        match self {
            SyntaxElement::Token(token) => &token.kind,
            SyntaxElement::Node(node) => &node.kind,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            SyntaxElement::Token(token) => token.text,
            SyntaxElement::Node(_) => "",
        }
    }

    pub fn content(&self) -> String {
        match self {
            SyntaxElement::Token(token) => token.text.to_string(),
            SyntaxElement::Node(node) => node.children.iter().map(|c| c.content()).collect(),
        }
    }

    pub fn children(&self) -> std::slice::Iter<'_, SyntaxElement<'_>> {
        match self {
            SyntaxElement::Token(_) => [].iter(),
            SyntaxElement::Node(node) => node.children.iter(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            SyntaxElement::Token(t) => t.span,
            SyntaxElement::Node(node) => node.span,
        }
    }
}

#[derive(Debug)]
pub struct Token<'src> {
    pub kind: SyntaxKind,
    pub span: Span,
    pub text: &'src str,
}

impl<'src> Token<'src> {
    pub fn new(kind: SyntaxKind, text: &'src str, span: Span) -> Self {
        Self { kind, span, text }
    }
}

#[derive(Debug)]
pub struct Node<'src> {
    pub kind: SyntaxKind,
    pub span: Span,
    pub children: Vec<SyntaxElement<'src>>,
}

impl<'src> Node<'src> {
    pub fn new(kind: SyntaxKind, children: Vec<SyntaxElement<'src>>, span: Span) -> Self {
        Self {
            kind,
            span,
            children,
        }
    }
}
