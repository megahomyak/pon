use crate::result::Result;
use crate::trimmable::Trimmable;

pub trait Parser<I: Trimmable, O, E> {
    fn parse(&self, input: I) -> Result<I, O, E>;
}

impl<T, I, O, E> Parser<I, O, E> for T
where
    I: Trimmable,
    T: Fn(I) -> Result<I, O, E>,
{
    fn parse(&self, input: I) -> Result<I, O, E> {
        self(input)
    }
}
