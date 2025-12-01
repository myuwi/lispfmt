use chumsky::{
    IterParser, Parser,
    extra::{self, SimpleState},
    prelude::{Rich, any, choice, end, group, just, none_of, one_of, recursive},
};

use crate::{
    kind::SyntaxKind,
    node::{Span, Token, TriviaPiece},
};

#[derive(Clone, Default)]
struct LexState {
    end_consumed: bool,
}

// TODO: Shebang, #lang directive
fn lexer<'src>() -> impl Parser<
    'src,
    &'src str,
    Vec<Token<'src>>,
    extra::Full<Rich<'src, char, Span>, extra::SimpleState<LexState>, ()>,
> {
    let number = {
        let sign = one_of("+-");
        let decimal = {
            let dec_digit = any().filter(move |c: &char| c.is_ascii_digit());
            let dec_literal = dec_digit.then(dec_digit.or(just('_')).repeated());

            let frac = just(".").then(dec_literal.or_not());
            let exp = one_of("eE").then(sign.or_not()).then(dec_literal);

            choice((
                dec_literal.then(frac.or_not()).ignored(),
                just(".").then(dec_literal).ignored(),
            ))
            .then(exp.or_not())
            .to_slice()
        };

        let hexadecimal = {
            let hex_prefix = just("0").then(one_of("xX"));
            let hex_digit = any().filter(move |c: &char| c.is_ascii_hexdigit());
            let hex_literal = hex_digit.then(hex_digit.or(just('_')).repeated());

            let frac = just(".").then(hex_literal.or_not());
            let exp = one_of("pP").then(sign.or_not()).then(hex_literal);

            hex_prefix
                .then(choice((
                    hex_literal.then(frac.or_not()).ignored(),
                    just(".").then(hex_literal).ignored(),
                )))
                .then(exp.or_not())
                .to_slice()
        };

        sign.or_not()
            .then(hexadecimal.or(decimal).or(just(".inf")).or(just(".nan")))
    }
    .to(SyntaxKind::Number)
    .labelled("number");

    let boolean = just("true")
        .or(just("false"))
        .to(SyntaxKind::Boolean)
        .labelled("boolean");

    fn is_newline(c: char) -> bool {
        c == '\n' || c == '\r'
    }

    let opening_delim = one_of("({[")
        .map(|c| match c {
            '(' => SyntaxKind::LParen,
            '{' => SyntaxKind::LBrace,
            '[' => SyntaxKind::LBracket,
            _ => unreachable!(),
        })
        .labelled("opening delimiter");

    let closing_delim = one_of(")}]")
        .map(|c| match c {
            ')' => SyntaxKind::RParen,
            '}' => SyntaxKind::RBrace,
            ']' => SyntaxKind::RBracket,
            _ => unreachable!(),
        })
        .labelled("closing delimiter");

    let delim = opening_delim.or(closing_delim).labelled("delimiter");

    // TODO: Make this cleaner
    let symbol_impl = recursive(|symbol_impl| {
        none_of("\"`',~;@")
            .and_is(delim.not())
            .filter(|c: &char| !c.is_control() && !c.is_whitespace())
            .repeated()
            .at_least(1)
            .and_is(number.then(symbol_impl.not()).not())
            .to_slice()
    });

    let symbol = just("~=")
        .or(symbol_impl.clone())
        .to(SyntaxKind::Symbol)
        .labelled("symbol");

    let string = none_of("\\\"")
        .ignored()
        .or(just('\\').then(any()).ignored())
        .repeated()
        .delimited_by(just('"'), just('"'))
        .to(SyntaxKind::String)
        .labelled("string");

    let keyword = just(":")
        .then(symbol_impl)
        .to(SyntaxKind::Keyword)
        .labelled("keyword");

    // Trivia

    let space = any()
        .filter(|c: &char| c.is_whitespace() && !is_newline(*c))
        .repeated()
        .at_least(1)
        .to(SyntaxKind::Space)
        .labelled("whitespace");

    let newline = any()
        .filter(|c: &char| is_newline(*c))
        .to(SyntaxKind::Newline)
        .labelled("newline");

    let comment = just(";")
        .then(none_of("\n").repeated())
        .to(SyntaxKind::Comment)
        .labelled("comment");

    let leading_trivia = space
        .or(newline)
        .or(comment)
        .map_with(|kind, e| TriviaPiece::new(kind, e.slice(), e.span()))
        .repeated()
        .collect();

    let trailing_trivia = space
        .or(comment)
        .map_with(|kind, e| TriviaPiece::new(kind, e.slice(), e.span()))
        .repeated()
        .collect();

    let end_once = end()
        .try_map_with(|_, e| {
            let state: &mut SimpleState<LexState> = e.state();
            if state.end_consumed {
                return Err(Rich::custom(e.span(), "End of input already consumed"));
            }
            state.end_consumed = true;

            Ok(SyntaxKind::End)
        })
        .labelled("end of input");

    let expression_start = opening_delim
        .or(string)
        .or(keyword.clone())
        .or(boolean)
        .or(symbol.clone())
        .or(number)
        .labelled("expression");

    // TODO: A comma can also be a whitespace character in Clojure
    let prefix = recursive(|prefix| {
        one_of("#@?~^'`,")
            .and_is(just("~=").not())
            .then_ignore(expression_start.or(prefix).rewind())
            .to(SyntaxKind::Prefix)
            .labelled("prefix")
    });

    // TODO: Error recovery
    let token = delim
        .or(string)
        .or(keyword)
        .or(boolean)
        .or(prefix)
        .or(symbol)
        .or(number)
        .or(end_once)
        .map_with(|kind, e| (kind, e.slice(), e.span()));

    group((leading_trivia, token, trailing_trivia))
        .map(|(leading, (kind, slice, span), trailing)| {
            Token::new(kind, slice, span, leading, trailing)
        })
        .repeated()
        .collect()
}

pub fn lex(src: &str) -> Result<Vec<Token<'_>>, Vec<Rich<'_, char>>> {
    lexer().parse(src).into_result()
}
