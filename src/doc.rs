use pretty::RcDoc;

use crate::{kind::SyntaxKind, node::SyntaxNode};

// const INDENT: isize = 2;

pub trait DocExt {
    fn to_doc(&self) -> RcDoc<'_>;
}

impl DocExt for SyntaxNode<'_> {
    fn to_doc(&self) -> RcDoc<'_> {
        match self.kind() {
            SyntaxKind::LParen
            | SyntaxKind::RParen
            | SyntaxKind::LBrace
            | SyntaxKind::RBrace
            | SyntaxKind::LBracket
            | SyntaxKind::RBracket
            | SyntaxKind::Symbol
            | SyntaxKind::Number
            | SyntaxKind::String
            | SyntaxKind::Boolean
            | SyntaxKind::Newline
            | SyntaxKind::Space
            | SyntaxKind::Comment => RcDoc::text(self.text()),

            SyntaxKind::Root => {
                let mut lines: Vec<RcDoc> = vec![];
                let mut iter = self.children().peekable();

                while iter.peek().is_some() {
                    let mut ignore_newlines = lines.is_empty();
                    let mut consecutive_newlines = 0;

                    let mut doc = RcDoc::nil();

                    while let Some(trivia) = iter.next_if(|s| s.kind().is_trivia()) {
                        match trivia.kind() {
                            SyntaxKind::Newline if !ignore_newlines => {
                                consecutive_newlines += 1;
                            }
                            SyntaxKind::Comment => {
                                if consecutive_newlines >= 2 {
                                    doc = doc.append(RcDoc::hardline());
                                }

                                doc = doc.append(trivia.text()).append(RcDoc::hardline());
                                ignore_newlines = false;
                                consecutive_newlines = 0;
                            }
                            _ => (),
                        }
                    }

                    if let Some(node) = iter.next() {
                        if consecutive_newlines >= 2 {
                            doc = doc.append(RcDoc::hardline());
                        }
                        doc = doc.append(node.to_doc());
                    }

                    while let Some(trivia) =
                        iter.next_if(|s| s.kind().is_space() || s.kind().is_comment())
                    {
                        if trivia.kind().is_comment() {
                            doc = doc.append(RcDoc::space()).append(trivia.text())
                        }
                    }

                    lines.push(doc);
                }

                RcDoc::intersperse(lines, RcDoc::hardline())
            }
            // SyntaxKind::List => todo!(),
            // SyntaxKind::Sequence => todo!(),
            // SyntaxKind::Table => todo!(),
            // SyntaxKind::Pair => todo!(),
            // SyntaxKind::Prefixed => todo!(),
            _ => self
                .children()
                .fold(RcDoc::nil(), |doc, node| doc.append(node.to_doc())),
        }
    }
}
