// impl<I: Iterator> Peekable<I> {

use std::iter::Peekable;

pub trait PeekableExt: Iterator {
    fn collect_while<F>(&mut self, f: F) -> Vec<Self::Item>
    where
        F: Fn(&Self::Item) -> bool;
}

impl<I: Iterator> PeekableExt for Peekable<I> {
    fn collect_while<F>(&mut self, f: F) -> Vec<Self::Item>
    where
        F: Fn(&Self::Item) -> bool,
    {
        let mut result = vec![];
        while let Some(t) = self.next_if(&f) {
            result.push(t);
        }
        result
    }
}
