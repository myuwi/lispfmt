use chumsky::span::SimpleSpan;

use crate::kind::SyntaxKind;

pub type Span = SimpleSpan;

pub enum SyntaxNode<'src> {
    Leaf(LeafNode<'src>),
    Inner(InnerNode<'src>),
}

impl std::fmt::Debug for SyntaxNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxNode::Leaf(node) => {
                write!(
                    f,
                    "{:?}@{:?} \"{}\"",
                    node.kind,
                    node.span,
                    node.text.escape_debug()
                )
            }
            SyntaxNode::Inner(node) => {
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

impl<'src> SyntaxNode<'src> {
    pub fn leaf(kind: SyntaxKind, text: &'src str, span: Span) -> Self {
        Self::Leaf(LeafNode::new(kind, text, span))
    }

    pub fn inner(kind: SyntaxKind, children: Vec<SyntaxNode<'src>>, span: Span) -> Self {
        Self::Inner(InnerNode::new(kind, children, span))
    }
}

impl SyntaxNode<'_> {
    pub fn kind(&self) -> &SyntaxKind {
        match self {
            SyntaxNode::Leaf(leaf) => &leaf.kind,
            SyntaxNode::Inner(inner) => &inner.kind,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            SyntaxNode::Leaf(leaf) => leaf.text,
            SyntaxNode::Inner(_) => "",
        }
    }

    pub fn children(&self) -> std::slice::Iter<'_, SyntaxNode<'_>> {
        match self {
            SyntaxNode::Leaf(_) => [].iter(),
            SyntaxNode::Inner(inner) => inner.children.iter(),
        }
    }
}

#[derive(Debug)]
pub struct LeafNode<'src> {
    pub kind: SyntaxKind,
    pub span: Span,
    pub text: &'src str,
}

impl<'src> LeafNode<'src> {
    pub fn new(kind: SyntaxKind, text: &'src str, span: Span) -> Self {
        Self { kind, span, text }
    }
}

#[derive(Debug)]
pub struct InnerNode<'src> {
    pub kind: SyntaxKind,
    pub span: Span,
    pub children: Vec<SyntaxNode<'src>>,
}

impl<'src> InnerNode<'src> {
    pub fn new(kind: SyntaxKind, children: Vec<SyntaxNode<'src>>, span: Span) -> Self {
        Self {
            kind,
            span,
            children,
        }
    }
}
