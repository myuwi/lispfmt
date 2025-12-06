use chumsky::{
    IterParser, Parser, extra,
    prelude::{Rich, any, choice, group, just, none_of, one_of, recursive},
};

use crate::{
    error::Error,
    kind::SyntaxKind,
    node::{Span, Token},
};

fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Token<'src>>, extra::Err<Rich<'src, char, Span>>> {
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

    let trivia = space.or(newline).or(comment);

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

    let hash_directive = just("#")
        .then(none_of("\n").repeated())
        .to(SyntaxKind::HashDirective)
        .labelled("hash directive");

    // TODO: Error recovery
    let token = delim
        .or(string)
        .or(keyword)
        .or(boolean)
        .or(prefix)
        .or(symbol)
        .or(number);

    group((
        trivia
            .or(hash_directive)
            .map_with(|kind, e| Token::new(kind, e.slice(), e.span()))
            .repeated()
            .collect::<Vec<_>>(),
        trivia
            .or(token)
            .map_with(|kind, e| Token::new(kind, e.slice(), e.span()))
            .repeated()
            .collect::<Vec<_>>(),
    ))
    .map(|(mut directives, mut tokens)| {
        directives.append(&mut tokens);
        directives
    })
}

pub fn lex<'src>(src: &'src str) -> Result<Vec<Token<'src>>, Error<'src>> {
    lexer().parse(src).into_result().map_err(Error::Lex)
}
