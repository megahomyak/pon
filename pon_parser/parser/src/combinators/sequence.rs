use crate::trimmable::TrimmingResult;
use crate::{parser::Parser, trimmable::Trimmable};
use crate::result::Result;

pub trait IntoIter {
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::Iter;
}

impl<'a> IntoIter for &'a str {
    type Item = char;
    type Iter = std::str::Chars<'a>;

    fn into_iter(self) -> Self::Iter {
        self.chars()
    }
}

/// Parses the provided sequence
pub fn sequence<P: PartialEq, I: Trimmable<Part = P>, E>(s: impl IntoIterator<Item = I::Part>) -> impl Parser<I, (), E> {
    let iter = s.into_iter();
    move |mut input: I| {
        for i in iter {
            match input.trim() {
                TrimmingResult::Trimmed { part, rest } => {
                    if i != part {
                        return Result::RecoverableError;
                    }
                    input = rest;
                }
                TrimmingResult::NothingToTrim => {
                    return Result::RecoverableError;
                }
            }
        }
        return Result::Ok { output: (), rest_of_input: input };
    }
}
