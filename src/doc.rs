use pretty::{Arena, DocAllocator, DocBuilder};

use crate::{
    doc_ext::DocExt,
    kind::SyntaxKind,
    node::{SyntaxElement, TriviaPiece},
};

pub type ArenaDoc<'a> = DocBuilder<'a, Arena<'a>>;

impl<'src> SyntaxElement<'src> {
    pub fn to_doc(&'src self, arena: &'src Arena<'src>) -> ArenaDoc<'src> {
        match self.kind() {
            SyntaxKind::Root => convert_root(arena, self),
            SyntaxKind::List => convert_container(arena, self, 2, true),
            // TODO: Heterogeneous sequences
            SyntaxKind::Sequence => convert_container(arena, self, 1, false).group(),
            SyntaxKind::Table => convert_container(arena, self, 1, false).group(),

            // FIXME: Handle trivia between pair
            SyntaxKind::Pair => arena.intersperse(
                self.children().map(|expr| expr.to_doc(arena)),
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

            SyntaxKind::Newline | SyntaxKind::Space | SyntaxKind::Comment => {
                unreachable!("Trivia should not appear as a SyntaxElement.")
            }
        }
    }
}

fn convert_root<'src>(arena: &'src Arena<'src>, root: &'src SyntaxElement<'src>) -> ArenaDoc<'src> {
    arena.concat(root.children().enumerate().map(|(i, expr)| {
        let (leading_trivia, trailing_trivia) = expr.trivia();

        let ignore_initial_newlines = i == 0;

        let mut doc = convert_leading_trivia(
            arena,
            leading_trivia,
            !ignore_initial_newlines,
            *expr.kind() != SyntaxKind::End,
        );

        doc = doc.append(expr.to_doc(arena));

        for trivia in trailing_trivia {
            if trivia.kind == SyntaxKind::Comment {
                doc = doc.append(arena.space()).append(trivia.text.trim_end());
            }
        }

        if *expr.kind() != SyntaxKind::End {
            doc = doc.append(arena.hardline());
        }

        doc
    }))
}

fn convert_container<'src>(
    arena: &'src Arena<'src>,
    elem: &'src SyntaxElement<'src>,
    indent: isize,
    keep_original_linebreaks: bool,
) -> ArenaDoc<'src> {
    let [open, exprs @ .., close] = &elem.children().collect::<Vec<_>>()[..] else {
        panic!("Container is missing an opening or closing delimiter.");
    };

    let mut doc = open.to_doc(arena);
    for trivia in open.trivia().1 {
        if trivia.kind == SyntaxKind::Comment {
            doc = doc
                .append(arena.space())
                .append(trivia.text.trim_end())
                .append(arena.hardline());
        }
    }

    doc = doc.append(arena.concat(exprs.iter().enumerate().map(|(i, expr)| {
        let (leading_trivia, trailing_trivia) = expr.trivia();

        let first_expr = i == 0;

        let add_linebreak = !keep_original_linebreaks
            || leading_trivia
                .first()
                .map(|t| t.kind == SyntaxKind::Newline)
                .unwrap_or(false);

        // Handle expr spacing
        let mut doc = if first_expr {
            arena.nil()
        } else if add_linebreak {
            arena.line()
        } else {
            arena.space()
        };

        doc = doc.append(convert_leading_trivia(
            arena,
            leading_trivia,
            !first_expr,
            true,
        ));
        doc = doc.append(expr.to_doc(arena));

        for trivia in trailing_trivia {
            if trivia.kind == SyntaxKind::Comment {
                doc = doc
                    .append(arena.space())
                    .append(trivia.text.trim_end())
                    .append(arena.break_group());
            }
        }

        doc
    })));

    let last_expr_has_trailing_comment = exprs
        .last()
        .map(|n| n.trivia().1.iter().any(|t| t.kind == SyntaxKind::Comment))
        .unwrap_or(false);

    let closing_has_leading_comment = close
        .trivia()
        .0
        .iter()
        .any(|t| t.kind == SyntaxKind::Comment);

    if last_expr_has_trailing_comment || closing_has_leading_comment {
        doc = doc.append(arena.line());
    }

    doc = doc.append(convert_leading_trivia(
        arena,
        close.trivia().0,
        !exprs.is_empty(),
        false,
    ));

    doc = doc.append(close.to_doc(arena)).hang(indent);

    doc
}

fn convert_leading_trivia<'src>(
    arena: &'src Arena<'src>,
    leading_trivia: &'src Vec<TriviaPiece<'src>>,
    allow_leading_newline: bool,
    allow_trailing_newline: bool,
) -> ArenaDoc<'src> {
    let mut track_newlines = allow_leading_newline;
    let mut consecutive_newlines = 0;
    let mut doc = arena.nil();

    for trivia in leading_trivia {
        match trivia.kind {
            SyntaxKind::Newline if track_newlines => {
                consecutive_newlines += 1;
            }
            SyntaxKind::Comment => {
                if consecutive_newlines >= 2 {
                    doc = doc.append(arena.hardline());
                }

                doc = doc.append(trivia.text.trim_end()).append(arena.hardline());
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
