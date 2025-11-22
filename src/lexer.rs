use chumsky::{
    IterParser, Parser, extra,
    prelude::{Rich, SimpleSpan, any, choice, end, just, none_of, one_of, skip_then_retry_until},
};

use crate::token::{Token, TokenKind};

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char, Span>>> {
    let delim = one_of("(){}[]").to_slice().map(|delim: &str| {
        let kind = match delim {
            "(" => TokenKind::LParen,
            ")" => TokenKind::RParen,
            "{" => TokenKind::LBrace,
            "}" => TokenKind::RBrace,
            "[" => TokenKind::LBracket,
            "]" => TokenKind::RBracket,
            _ => unreachable!(),
        };
        Token::new(kind, delim)
    });

    let prefix = one_of("#'`,")
        .to_slice()
        .map(|s| Token::new(TokenKind::Prefix, s));

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
    .to_slice()
    .map(|s| Token::new(TokenKind::Number, s));

    let quoted_string = none_of("\\\"")
        .ignored()
        .or(just('\\').then(any()).ignored())
        .repeated()
        .delimited_by(just('"'), just('"'))
        .to_slice();

    let boolean = just("true")
        .or(just("false"))
        .map(|s| Token::new(TokenKind::Boolean, s));

    let whitespace = any()
        .filter(|c: &char| c.is_whitespace())
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s| Token::new(TokenKind::Whitespace, s));

    let symbol = just(":")
        .or(just("~="))
        .or(none_of("\"'~;@`")
            .filter(|c: &char| !c.is_control())
            .and_is(whitespace.not())
            .and_is(delim.not())
            .repeated()
            .at_least(1)
            .to_slice())
        .map(|s| Token::new(TokenKind::Symbol, s));

    let colon_string = just(":").then(symbol).to_slice();

    let string = quoted_string
        .or(colon_string)
        .map(|s| Token::new(TokenKind::String, s));

    let comment = just(";")
        .then(none_of("\n").repeated())
        .to_slice()
        .map(|s| Token::new(TokenKind::Comment, s));

    let token = delim
        .or(prefix)
        .or(number)
        .or(string)
        .or(boolean)
        .or(comment)
        .or(whitespace)
        .or(symbol);

    token
        .map_with(|tok, e| (tok, e.span()))
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

pub fn tokenize(src: &str) -> (Option<Vec<Spanned<Token<'_>>>>, Vec<Rich<'_, char>>) {
    lexer().parse(src).into_output_errors()
}
