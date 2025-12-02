use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Rich;

#[derive(Debug)]
pub enum Error<'src> {
    Lex(Vec<Rich<'src, char>>),
}

impl<'src> Error<'src> {
    pub fn print(self, src: &str) {
        match self {
            Error::Lex(errs) => errs.into_iter().for_each(|e| {
                Report::build(ReportKind::Error, ((), e.span().into_range()))
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_message(e.to_string())
                    .with_label(
                        Label::new(((), e.span().into_range()))
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(&src))
                    .unwrap()
            }),
        }
    }
}
