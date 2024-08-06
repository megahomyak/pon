use crate::non_empty_vec::{self, NonEmptyVec};
use crate::parser::parser_input;
use crate::parser::Positioned;

#[derive(Debug, PartialEq)]
pub struct Word<C> {
    pub characters: NonEmptyVec<C>,
}
pub(super) enum After<P> {
    CommandSeparator(),
    WordSeparator(),
    PonInputOpener { position: P },
    ParserInputEnd(),
}
pub(super) fn parse<P, C>(
    mut parser_input: impl parser_input::Input<P, C>,
) -> (Option<Positioned<P, Word<C>>>, After<P>) {
    let characters = non_empty_vec::Incomplete::new();
    let after = loop {
        match parser_input.next() {
            None => break After::ParserInputEnd(),
            Some(mut part) => {
                match part.kind {
                    parser_input::part::Kind::PonInputOpener() => {
                        break After::PonInputOpener {
                            position: part.position,
                        }
                    }
                    parser_input::part::Kind::CommandSeparator() => {
                        break After::CommandSeparator()
                    }
                    parser_input::part::Kind::NextCharacterEscaper() => match parser_input.next() {
                        None => (),
                        Some(escaped_part) => {
                            match escaped_part.kind {
                                parser_input::part::Kind::Literal() => {
                                    characters.push(Positioned {
                                        position: part.position,
                                        entity: part.content,
                                    })
                                }
                                _ => (),
                            }
                            part = escaped_part
                        }
                    },
                    parser_input::part::Kind::WordSeparator() => break After::WordSeparator(),
                    _ => (),
                }
                characters.push(Positioned {  });
            }
        }
    };
    (
        first.map(|first| {
            rest.shrink_to_fit();
            Positioned {
                position: first.position,
                entity: Word {
                    characters: NonEmptyString {
                        first: first.character,
                        rest,
                    },
                },
            }
        }),
        after,
    )
}
