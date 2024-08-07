use crate::parser::parser_input::part::Part;
use crate::parser::parser_input::Input;

#[derive(Eq, PartialEq)]
pub struct NonEmpty<I: Input> {
    pub first: I::PartContent,
    pub rest: I::PartContainer,
}

impl<I: Input> PartialEq for NonEmpty<I> {
    fn eq(&self, other: &Self) -> bool {
        self.first == other.first && self.rest == other.rest
    }
}
impl<I: Input> Eq for NonEmpty<I> {}
