pub mod parser_input;
pub mod program;

use crate::non_empty_vec::{NonEmptyString, NonEmptyVec};

#[derive(Debug)]
pub struct Positioned<P, T> {
    pub position: P,
    pub entity: T,
}

pub mod name {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct Name {
        pub words: NonEmptyVec<word::Word>,
    }
    pub(super) enum After {
        CommandSeparator(),
        PonInputOpener(parser_input::Index),
        ParserInputEnd(),
        EscapeAtEndOfInput(),
    }
    pub(super) fn parse(
        parser_input: &mut parser_input::Input,
    ) -> (Option<Positioned<Name>>, After) {
        let mut first = None;
        let mut rest = Vec::new();
        let after = loop {
            let (word, after_word) = word::parse(parser_input);
            if let Some(word) = word {
                match first {
                    None => first = Some(word),
                    Some(_) => rest.push(word.entity),
                }
            }
            match after_word {
                word::After::EscapeAtEndOfInput() => break After::EscapeAtEndOfInput(),
                word::After::CommandSeparator() => break After::CommandSeparator(),
                word::After::PonInputOpener(index) => break After::PonInputOpener(index),
                word::After::ParserInputEnd() => break After::ParserInputEnd(),
                word::After::WordSeparator() => (),
            }
        };
        (
            first.map(|first| {
                rest.shrink_to_fit();
                Positioned {
                    position: first.position,
                    entity: Name {
                        words: NonEmptyVec {
                            first: first.entity,
                            rest,
                        },
                    },
                }
            }),
            after,
        )
    }
}

pub mod pon_input {
    use super::*;

}
