use crate::parser::parser_input::Input;

#[derive(Eq, PartialEq, Debug)]
pub struct NonEmpty<I: Input> {
    pub first: I::PartContent,
    pub rest: I::PartContainer,
}
