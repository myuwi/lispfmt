use pretty::RcDoc;

use crate::{kind::SyntaxKind, node::SyntaxElement};

// const INDENT: isize = 2;

pub trait DocExt {
    fn to_doc(&self) -> RcDoc<'_>;
}

impl DocExt for SyntaxElement<'_> {
    fn to_doc(&self) -> RcDoc<'_> {
        match self.kind() {
            SyntaxKind::Root => {
                let mut lines: Vec<RcDoc> = vec![];

                for node in self.children() {
                    let (leading_trivia, trailing_trivia) = node.trivia();

                    let mut ignore_newlines = lines.is_empty();
                    let mut consecutive_newlines = 0;

                    let mut doc = RcDoc::nil();

                    for trivia in leading_trivia {
                        match trivia.kind {
                            SyntaxKind::Newline if !ignore_newlines => {
                                consecutive_newlines += 1;
                            }
                            SyntaxKind::Comment => {
                                if consecutive_newlines >= 2 {
                                    doc = doc.append(RcDoc::hardline());
                                }

                                doc = doc.append(trivia.text).append(RcDoc::hardline());
                                ignore_newlines = false;
                                consecutive_newlines = 0;
                            }
                            _ => (),
                        }
                    }

                    if consecutive_newlines >= 2 && *node.kind() != SyntaxKind::End {
                        doc = doc.append(RcDoc::hardline());
                    }
                    doc = doc.append(node.to_doc());

                    for trivia in trailing_trivia {
                        if trivia.kind.is_comment() {
                            doc = doc.append(RcDoc::space()).append(trivia.text)
                        }
                    }

                    lines.push(doc);
                }

                RcDoc::intersperse(lines, RcDoc::hardline())
            }

            // FIXME: Handle trivia within container elements
            SyntaxKind::List
            | SyntaxKind::Sequence
            | SyntaxKind::Table
            | SyntaxKind::Pair
            | SyntaxKind::Prefixed => self
                .children()
                .fold(RcDoc::nil(), |doc, node| doc.append(node.to_doc())),

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
            | SyntaxKind::Prefix
            | SyntaxKind::End => RcDoc::text(self.text()),

            SyntaxKind::Newline | SyntaxKind::Space | SyntaxKind::Comment => {
                unreachable!("Trivia should not appear as a SyntaxElement")
            }
        }
    }
}
