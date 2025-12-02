use std::{iter::Peekable, vec::IntoIter};

use crate::{
    error::Error,
    kind::SyntaxKind,
    lexer::lex,
    node::{SyntaxElement, Token},
};

// TODO: Error recovery
pub fn parse<'src>(src: &'src str) -> Result<SyntaxElement<'src>, Error<'src>> {
    let mut p = Parser::new(src)?;

    exprs(&mut p, SyntaxKind::End);
    p.assert(SyntaxKind::End);
    let root = SyntaxElement::node(SyntaxKind::Root, p.finish());

    Ok(root)
}

struct Marker(usize);

struct Parser<'src> {
    lexer: Peekable<IntoIter<Token<'src>>>,
    nodes: Vec<SyntaxElement<'src>>,
}

impl<'src> Parser<'src> {
    fn new(src: &'src str) -> Result<Self, Error<'src>> {
        let lexer = lex(src)?.into_iter().peekable();
        Ok(Parser {
            lexer,
            nodes: vec![],
        })
    }

    fn finish(self) -> Vec<SyntaxElement<'src>> {
        self.nodes
    }

    fn marker(&self) -> Marker {
        Marker(self.nodes.len())
    }

    fn peek(&mut self) -> Option<&Token<'src>> {
        self.lexer.peek()
    }

    fn peek_kind(&mut self) -> SyntaxKind {
        self.peek().map(|t| t.kind).unwrap_or(SyntaxKind::End)
    }

    fn wrap(&mut self, m: Marker, kind: SyntaxKind) {
        let to = self.marker().0;
        let from = m.0.min(to);
        let children = self.nodes.drain(from..to).collect();

        self.nodes.push(SyntaxElement::node(kind, children));
    }

    fn at(&mut self, kind: SyntaxKind) -> bool {
        self.peek().map(|t| t.kind == kind).unwrap_or(false)
    }

    fn assert(&mut self, kind: SyntaxKind) {
        assert!(self.at(kind));
        self.eat();
    }

    fn eat(&mut self) {
        // TODO: make this safe
        let token = self.lexer.next().unwrap();
        self.nodes.push(SyntaxElement::token(token));
    }
}

fn exprs(p: &mut Parser, stop_kind: SyntaxKind) {
    while !p.at(stop_kind) {
        expr(p);
    }
}

fn expr(p: &mut Parser) {
    match p.peek_kind() {
        SyntaxKind::LParen => list(p),
        SyntaxKind::LBracket => sequence(p),
        SyntaxKind::LBrace => table(p),
        SyntaxKind::Prefix => prefixed(p),
        _ => p.eat(),
    };
}

fn list(p: &mut Parser) {
    let m = p.marker();
    p.assert(SyntaxKind::LParen);
    exprs(p, SyntaxKind::RParen);
    p.assert(SyntaxKind::RParen);
    p.wrap(m, SyntaxKind::List);
}

fn sequence(p: &mut Parser) {
    let m = p.marker();
    p.assert(SyntaxKind::LBracket);
    exprs(p, SyntaxKind::RBracket);
    p.assert(SyntaxKind::RBracket);
    p.wrap(m, SyntaxKind::Sequence);
}

fn table(p: &mut Parser) {
    let m = p.marker();

    p.assert(SyntaxKind::LBrace);
    while !p.at(SyntaxKind::RBrace) {
        pair(p)
    }
    p.assert(SyntaxKind::RBrace);
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
