use pretty::{Arena, Doc, DocAllocator, DocBuilder};

use crate::{doc_ext::DocExt, kind::SyntaxKind, node::SyntaxElement, peekable_ext::PeekableExt};

pub type ArenaDoc<'a> = DocBuilder<'a, Arena<'a>>;

impl<'src> SyntaxElement<'src> {
    pub fn to_doc(&'src self, arena: &'src Arena<'src>) -> ArenaDoc<'src> {
        match self.kind() {
            SyntaxKind::Root => convert_root(arena, self),
            SyntaxKind::List => convert_list_like(arena, self, 2, true),
            SyntaxKind::Sequence => {
                let [_open, exprs @ .., _close] = &self.children().collect::<Vec<_>>()[..] else {
                    panic!("Container is missing an opening or closing delimiter.");
                };

                let mut non_trivia = exprs.iter().filter(|e| !e.kind().is_trivia());
                let heterogeneous = match non_trivia.next() {
                    Some(first) => !non_trivia.all(|e| e.kind() == first.kind()),
                    None => false,
                };

                let doc = convert_list_like(arena, self, 1, heterogeneous);
                if heterogeneous { doc } else { doc.group() }
            }
            SyntaxKind::Table => convert_list_like(arena, self, 1, false).group(),

            // FIXME: Handle trivia between pair
            SyntaxKind::Pair => arena.intersperse(
                self.children()
                    .filter(|e| !e.kind().is_trivia())
                    .map(|expr| expr.to_doc(arena)),
                arena.space(),
            ),

            // A prefixed cannot contain any trivia between its children
            SyntaxKind::Prefixed => self
                .children()
                .fold(arena.nil(), |doc, expr| doc.append(expr.to_doc(arena))),

            SyntaxKind::LParen
            | SyntaxKind::RParen
            | SyntaxKind::LBrace
            | SyntaxKind::RBrace
            | SyntaxKind::LBracket
            | SyntaxKind::RBracket
            | SyntaxKind::Symbol
            | SyntaxKind::Number
            | SyntaxKind::String
            | SyntaxKind::Keyword
            | SyntaxKind::Boolean
            | SyntaxKind::Prefix
            | SyntaxKind::End => arena.text(self.text()),

            SyntaxKind::HashDirective => arena.text(self.text().trim_end()),

            SyntaxKind::Newline | SyntaxKind::Space | SyntaxKind::Comment => {
                unreachable!("Trivia should not be handled through `to_doc`.")
            }
        }
    }
}

fn is_leading_trivia(kind: &SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Space | SyntaxKind::Newline | SyntaxKind::Comment
    )
}

fn is_trailing_trivia(kind: &SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::Space | SyntaxKind::Comment)
}

fn is_ignore_comment<'a>(trivia: &'a SyntaxElement<'a>) -> bool {
    trivia.kind() == &SyntaxKind::Comment
        && trivia.text().trim_start_matches(";").trim() == "lispfmt-ignore"
}

fn convert_root<'src>(arena: &'src Arena<'src>, root: &'src SyntaxElement<'src>) -> ArenaDoc<'src> {
    let mut iter = root.children().peekable();
    let mut doc = arena.nil();

    while iter.peek().is_some() {
        let ignore_leading_newlines = matches!(*doc, Doc::Nil);

        let leading_trivia = iter.collect_while(|e| is_leading_trivia(e.kind()));
        let expr = iter.next();
        let trailing_trivia = iter.collect_while(|e| is_trailing_trivia(e.kind()));

        let ignored = leading_trivia
            .iter()
            .chain(trailing_trivia.iter())
            .cloned()
            .any(|t| is_ignore_comment(t));

        doc = doc.append(convert_leading_trivia(
            arena,
            &leading_trivia,
            !ignore_leading_newlines,
            expr.is_some(),
        ));

        if let Some(expr) = expr {
            if ignored {
                doc = doc.append(expr.content());
            } else {
                doc = doc.append(expr.to_doc(arena));
            }
        }

        for trivia in trailing_trivia {
            if *trivia.kind() == SyntaxKind::Comment {
                doc = doc.append(arena.space()).append(trivia.text().trim_end());
            }
        }

        if iter.peek().is_some() {
            doc = doc.append(arena.hardline());
        }
    }

    doc
}

fn convert_list_like<'src>(
    arena: &'src Arena<'src>,
    elem: &'src SyntaxElement<'src>,
    indent: isize,
    keep_original_linebreaks: bool,
) -> ArenaDoc<'src> {
    let [open, exprs @ .., close] = &elem.children().collect::<Vec<_>>()[..] else {
        panic!("Container is missing an opening or closing delimiter.");
    };

    let mut iter = exprs.iter().cloned().peekable();
    let mut doc = open.to_doc(arena);
    let mut has_leading_ignore_comment = false;

    // Skip trivia until the first comment
    while let Some(trivia) = iter.next_if(|t| t.kind().is_trivia()) {
        if *trivia.kind() == SyntaxKind::Comment {
            doc = doc
                .append(trivia.text().trim_end())
                .append(arena.hardline());
            has_leading_ignore_comment = is_ignore_comment(trivia);
            break;
        }
    }

    let allow_leading_empty_line_after_first_newline = indent <= 1;

    // TODO: Avoid mutating state?
    let mut first_expr = true;
    let mut first_newline_found = false;
    let mut last_expr_has_trailing_comment = false;

    let leading_trivia = loop {
        let leading_trivia = iter.collect_while(|e| is_leading_trivia(e.kind()));

        // If there is no expr, this trivia belongs to the closing delimiter
        let Some(expr) = iter.next() else {
            break leading_trivia;
        };

        let trailing_trivia = iter.collect_while(|e| is_trailing_trivia(e.kind()));

        let has_leading_newline = leading_trivia
            .first()
            .map(|t| *t.kind() == SyntaxKind::Newline)
            .unwrap_or(false);

        let add_linebreak = if keep_original_linebreaks {
            has_leading_newline
        } else {
            true
        };

        // Handle expr spacing
        let mut expr_doc = if first_expr {
            arena.nil()
        } else if add_linebreak {
            arena.line()
        } else {
            arena.space()
        };

        let allow_leading_empty_newline =
            !(first_expr || !allow_leading_empty_line_after_first_newline && !first_newline_found);

        expr_doc = expr_doc.append(convert_leading_trivia(
            arena,
            &leading_trivia,
            allow_leading_empty_newline,
            true,
        ));

        first_newline_found = first_newline_found || has_leading_newline;

        let ignored = first_expr && has_leading_ignore_comment
            || leading_trivia
                .iter()
                .chain(trailing_trivia.iter())
                .cloned()
                .any(|t| is_ignore_comment(t));

        // TODO: Break group when ignored is multiline
        if ignored {
            expr_doc = expr_doc.append(expr.content());
        } else {
            expr_doc = expr_doc.append(expr.to_doc(arena));
        }

        last_expr_has_trailing_comment = false;
        for trivia in trailing_trivia {
            if *trivia.kind() == SyntaxKind::Comment {
                // TODO: This should add a hardline
                expr_doc = expr_doc
                    .append(arena.space())
                    .append(trivia.text().trim_end())
                    .append(arena.break_group());

                last_expr_has_trailing_comment = true;
            }
        }

        doc = doc.append(expr_doc);
        first_expr = false;
    };

    let closing_has_leading_comment = leading_trivia
        .iter()
        .any(|t| *t.kind() == SyntaxKind::Comment);

    if last_expr_has_trailing_comment || closing_has_leading_comment {
        doc = doc.append(arena.hardline());
    }

    doc = doc.append(convert_leading_trivia(
        arena,
        &leading_trivia,
        !exprs.iter().all(|e| e.kind().is_trivia()),
        false,
    ));

    doc = doc.append(close.to_doc(arena)).hang(indent);

    // Make sure line breaks are kept even when inside a grouped container
    if keep_original_linebreaks {
        doc = doc.append(arena.break_group());
    }

    doc
}

fn convert_leading_trivia<'src>(
    arena: &'src Arena<'src>,
    leading_trivia: &Vec<&'src SyntaxElement<'src>>,
    allow_leading_newline: bool,
    allow_trailing_newline: bool,
) -> ArenaDoc<'src> {
    let mut track_newlines = allow_leading_newline;
    let mut consecutive_newlines = 0;
    let mut doc = arena.nil();

    for trivia in leading_trivia {
        match trivia.kind() {
            SyntaxKind::Newline if track_newlines => {
                consecutive_newlines += 1;
            }
            SyntaxKind::Comment => {
                if consecutive_newlines >= 2 {
                    doc = doc.append(arena.hardline());
                }

                doc = doc
                    .append(trivia.text().trim_end())
                    .append(arena.hardline());
                track_newlines = true;
                consecutive_newlines = 0;
            }
            _ => (),
        }
    }

    if allow_trailing_newline && consecutive_newlines >= 2 {
        doc = doc.append(arena.hardline());
    }

    doc
}
