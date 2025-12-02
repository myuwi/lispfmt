use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Rich;

use crate::node::Span;

// TODO: Combine errors
#[derive(Debug)]
pub enum Error<'src> {
    Lex(Vec<Rich<'src, char>>),
    Parse(Vec<(String, Span)>),
}

impl<'src> Error<'src> {
    pub fn print(self, src: &str) {
        match self {
            Error::Lex(errs) => errs.into_iter().for_each(|e| {
                build_report(&e.to_string(), e.span())
                    .eprint(Source::from(&src))
                    .unwrap()
            }),
            Error::Parse(errs) => errs.into_iter().for_each(|(ref reason, span)| {
                build_report(reason, &span)
                    .eprint(Source::from(&src))
                    .unwrap()
            }),
        }
    }
}

fn build_report<'a>(reason: &'a str, span: &'a Span) -> Report<'a> {
    Report::build(ReportKind::Error, span.into_range())
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
        .with_message(reason)
        .with_label(
            Label::new(span.into_range())
                .with_message(reason.to_string())
                .with_color(Color::Red),
        )
        .finish()
}
