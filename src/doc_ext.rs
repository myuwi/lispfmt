use pretty::{DocAllocator, DocBuilder};

pub trait DocExt<'a, A>
where
    A: 'a,
    Self: DocAllocator<'a, A>,
{
    /// Forces the current group to break onto multiple lines
    fn break_group(&'a self) -> DocBuilder<'a, Self, A>;
}

impl<'a, A, T> DocExt<'a, A> for T
where
    A: 'a,
    T: DocAllocator<'a, A>,
{
    fn break_group(&'a self) -> DocBuilder<'a, Self, A> {
        self.nil().flat_alt(self.hardline())
    }
}
