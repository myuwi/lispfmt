use chumsky::{
    IterParser, Parser, extra,
    prelude::{Rich, any, choice, group, just, none_of, one_of, recursive},
};

use crate::{
    kind::SyntaxKind,
    node::{Span, SyntaxNode},
};

// TODO: Error recovery

fn parser<'src>()
-> impl Parser<'src, &'src str, SyntaxNode<'src>, extra::Err<Rich<'src, char, Span>>> {
    let node = recursive(|node| {
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
        .map_with(|s, e| SyntaxNode::leaf(SyntaxKind::Number, s, e.span()));

        let quoted_string = none_of("\\\"")
            .ignored()
            .or(just('\\').then(any()).ignored())
            .repeated()
            .delimited_by(just('"'), just('"'))
            .to_slice();

        let boolean = just("true")
            .or(just("false"))
            .map_with(|s, e| SyntaxNode::leaf(SyntaxKind::Boolean, s, e.span()));

        fn is_newline(c: char) -> bool {
            c == '\n' || c == '\r'
        }

        let whitespace = choice((
            any()
                .filter(|c: &char| c.is_whitespace() && !is_newline(*c))
                .repeated()
                .then(any().filter(|c: &char| is_newline(*c)))
                .ignored(),
            any()
                .filter(|c: &char| c.is_whitespace() && !is_newline(*c))
                .repeated()
                .at_least(1)
                .ignored(),
        ))
        .to_slice()
        .map_with(|s, e| SyntaxNode::leaf(SyntaxKind::Whitespace, s, e.span()));

        let symbol = just(":")
            .or(just("~="))
            .or(none_of("(){}[]\"`',~;@")
                .filter(|c: &char| !c.is_control())
                .and_is(whitespace.not())
                .repeated()
                .at_least(1)
                .to_slice())
            .map_with(|s, e| SyntaxNode::leaf(SyntaxKind::Symbol, s, e.span()));

        let colon_string = just(":").then(symbol).to_slice();

        let string = quoted_string
            .or(colon_string)
            .map_with(|s, e| SyntaxNode::leaf(SyntaxKind::String, s, e.span()));

        let comment = just(";")
            .then(none_of("\n").repeated())
            .to_slice()
            .map_with(|s, e| SyntaxNode::leaf(SyntaxKind::Comment, s, e.span()));

        let trivia = whitespace.or(comment);
        let non_trivia = node.clone().and_is(trivia.not());

        let list = group((
            just("(").map_with(|s, e| SyntaxNode::leaf(SyntaxKind::LParen, s, e.span())),
            node.clone().repeated().collect::<Vec<SyntaxNode>>(),
            just(")").map_with(|s, e| SyntaxNode::leaf(SyntaxKind::RParen, s, e.span())),
        ))
        .map_with(|s, e| {
            let mut children = vec![s.0];
            for elem in s.1 {
                children.push(elem);
            }
            children.push(s.2);

            SyntaxNode::inner(SyntaxKind::List, children, e.span())
        });

        let sequence = group((
            just("[").map_with(|s, e| SyntaxNode::leaf(SyntaxKind::LBracket, s, e.span())),
            node.clone().repeated().collect::<Vec<SyntaxNode>>(),
            just("]").map_with(|s, e| SyntaxNode::leaf(SyntaxKind::RBracket, s, e.span())),
        ))
        .map_with(|s, e| {
            let mut children = vec![s.0];
            for elem in s.1 {
                children.push(elem);
            }
            children.push(s.2);

            SyntaxNode::inner(SyntaxKind::Sequence, children, e.span())
        });

        let table_pair = group((
            non_trivia.clone(),
            trivia.repeated().collect::<Vec<SyntaxNode>>(),
            non_trivia.clone(),
        ))
        .map_with(|s, e| {
            let mut children = vec![s.0];
            for elem in s.1 {
                children.push(elem);
            }
            children.push(s.2);

            SyntaxNode::inner(SyntaxKind::Pair, children, e.span())
        });

        let table = group((
            just("{").map_with(|s, e| SyntaxNode::leaf(SyntaxKind::LBrace, s, e.span())),
            trivia.repeated().collect::<Vec<SyntaxNode>>(),
            table_pair
                .then(trivia.repeated().collect::<Vec<SyntaxNode>>())
                .repeated()
                .collect::<Vec<(SyntaxNode, Vec<SyntaxNode>)>>(),
            just("}").map_with(|s, e| SyntaxNode::leaf(SyntaxKind::RBrace, s, e.span())),
        ))
        .map_with(|s, e| {
            let mut children = vec![s.0];
            for elem in s.1 {
                children.push(elem);
            }
            for elem in s.2 {
                children.push(elem.0);

                for trivia in elem.1 {
                    children.push(trivia);
                }
            }
            children.push(s.3);

            SyntaxNode::inner(SyntaxKind::Table, children, e.span())
        });

        let prefixed = group((
            one_of("#'`,")
                .to_slice()
                .map_with(|s, e| SyntaxNode::leaf(SyntaxKind::Symbol, s, e.span())),
            non_trivia.clone(),
        ))
        .map_with(|s, e| SyntaxNode::inner(SyntaxKind::Prefixed, vec![s.0, s.1], e.span()));

        // TODO: Error recovery
        list.or(sequence)
            .or(table)
            .or(prefixed)
            .or(number)
            .or(string)
            .or(boolean)
            .or(comment)
            .or(whitespace)
            .or(symbol)
    });

    node.repeated()
        .collect()
        .map_with(|s, e| SyntaxNode::inner(SyntaxKind::Root, s, e.span()))
}

pub fn parse(src: &str) -> (Option<SyntaxNode<'_>>, Vec<Rich<'_, char>>) {
    parser().parse(src).into_output_errors()
}
