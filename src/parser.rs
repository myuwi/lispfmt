use std::{iter::Peekable, vec::IntoIter};

use crate::{
    error::Error,
    kind::SyntaxKind,
    lexer::lex,
    node::{Span, SyntaxElement, Token},
};

// TODO: Error recovery
pub fn parse<'src>(src: &'src str) -> Result<SyntaxElement<'src>, Error<'src>> {
    let mut p = Parser::new(src)?;

    exprs(&mut p, &[SyntaxKind::End]);
    let root = SyntaxElement::node(SyntaxKind::Root, p.finish()?);

    Ok(root)
}

struct Marker(usize);

struct Parser<'src> {
    lexer: Peekable<IntoIter<Token<'src>>>,
    n_trivia: usize,
    nodes: Vec<SyntaxElement<'src>>,
    errors: Vec<(String, Span)>,
}

impl<'src> Parser<'src> {
    fn new(src: &'src str) -> Result<Self, Error<'src>> {
        let lexer = lex(src)?.into_iter().peekable();

        let mut p = Parser {
            lexer,
            n_trivia: 0,
            nodes: vec![],
            errors: vec![],
        };
        p.consume_trivia();

        Ok(p)
    }

    fn finish(self) -> Result<Vec<SyntaxElement<'src>>, Error<'src>> {
        if self.errors.is_empty() {
            Ok(self.nodes)
        } else {
            Err(Error::Parse(self.errors))
        }
    }

    fn marker(&self) -> Marker {
        Marker(self.nodes.len())
    }

    fn before_trivia(&self) -> Marker {
        Marker(self.nodes.len() - self.n_trivia)
    }

    fn peek(&mut self) -> Option<&Token<'src>> {
        self.lexer.peek()
    }

    fn peek_kind(&mut self) -> SyntaxKind {
        self.peek().map(|t| t.kind).unwrap_or(SyntaxKind::End)
    }

    fn wrap(&mut self, m: Marker, kind: SyntaxKind) {
        let to = self.before_trivia().0;
        let from = m.0.min(to);
        let children = self.nodes.drain(from..to).collect();

        self.nodes.insert(from, SyntaxElement::node(kind, children));
    }

    fn at(&mut self, kind: SyntaxKind) -> bool {
        self.peek_kind() == kind
    }

    fn at_one_of(&mut self, kinds: &[SyntaxKind]) -> bool {
        kinds.contains(&self.peek_kind())
    }

    fn assert(&mut self, kind: SyntaxKind) {
        assert_eq!(kind, self.peek_kind());
        self.eat();
    }

    fn consume_trivia(&mut self) {
        while let Some(trivia) = self.lexer.next_if(|t| t.kind.is_trivia()) {
            self.nodes.push(SyntaxElement::token(trivia));
            self.n_trivia += 1;
        }
    }

    fn eat(&mut self) {
        if let Some(token) = self.lexer.next() {
            self.nodes.push(SyntaxElement::token(token));
        }
        self.n_trivia = 0;
        self.consume_trivia();
    }

    fn eat_if(&mut self, kind: SyntaxKind) -> bool {
        let is_kind = self.at(kind);
        if is_kind {
            self.eat();
        }
        is_kind
    }

    fn expect(&mut self, kind: SyntaxKind) {
        if !self.eat_if(kind) {
            let m = self.marker();
            let pos = self.nodes.get(m.0 - 1).map(|n| n.span().end).unwrap_or(0);
            self.errors
                .push((format!("expected {}", kind.name()), (pos..pos).into()))
        }
    }

    fn unexpected(&mut self) {
        let m = self.marker();
        // TODO: Produce a SyntaxElement::Error?
        self.eat();
        let node = &self.nodes[m.0];
        self.errors
            .push((format!("unexpected {}", node.kind().name()), node.span()))
    }
}

fn exprs(p: &mut Parser, stop_kinds: &[SyntaxKind]) {
    while !p.at_one_of(stop_kinds) {
        expr(p);
    }
}

fn expr(p: &mut Parser) {
    match p.peek_kind() {
        SyntaxKind::LParen => list(p),
        SyntaxKind::LBracket => sequence(p),
        SyntaxKind::LBrace => table(p),
        SyntaxKind::Prefix => prefixed(p),

        SyntaxKind::Symbol
        | SyntaxKind::Number
        | SyntaxKind::String
        | SyntaxKind::Keyword
        | SyntaxKind::Boolean
        | SyntaxKind::HashDirective => p.eat(),

        _ => p.unexpected(),
    };
}

fn list(p: &mut Parser) {
    let m = p.marker();
    p.assert(SyntaxKind::LParen);
    exprs(p, &[SyntaxKind::RParen, SyntaxKind::End]);
    p.expect(SyntaxKind::RParen);
    p.wrap(m, SyntaxKind::List);
}

fn sequence(p: &mut Parser) {
    let m = p.marker();
    p.assert(SyntaxKind::LBracket);
    exprs(p, &[SyntaxKind::RBracket, SyntaxKind::End]);
    p.expect(SyntaxKind::RBracket);
    p.wrap(m, SyntaxKind::Sequence);
}

fn table(p: &mut Parser) {
    let m = p.marker();
    p.assert(SyntaxKind::LBrace);

    // TODO: Improve error message when table contains an odd number of exprs and/or unexpected tokens
    while !p.at_one_of(&[SyntaxKind::RBrace, SyntaxKind::End]) {
        pair(p)
    }
    p.expect(SyntaxKind::RBrace);
    p.wrap(m, SyntaxKind::Table);
}

fn pair(p: &mut Parser) {
    let m = p.marker();
    expr(p);
    expr(p);
    p.wrap(m, SyntaxKind::Pair);
}

fn prefixed(p: &mut Parser) {
    let m = p.marker();
    p.assert(SyntaxKind::Prefix);
    expr(p);
    p.wrap(m, SyntaxKind::Prefixed);
}
