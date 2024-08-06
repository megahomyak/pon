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
pub(super) fn parse<I: parser_input::Input>(
    mut parser_input: I,
) -> (Option<Positioned<I::Part, Word<I::Part::Content>>>, After<I::Part::Position>) {
    let mut first = None;
    let mut rest = I::Part::Container::default();
    let mut push = |part| match first {
        None => first = Some(part),
        Some(_) => rest.extend(part.content),
    };
    let after = loop {
        match parser_input.next() {
            None => break After::ParserInputEnd(),
            Some(mut part) => {
                match part.kind {
                    parser_input::part::Kind::PonInputOpener() => {
                        break After::PonInputOpener {
                            position: part.position(),
                        }
                    }
                    parser_input::part::Kind::CommandSeparator() => {
                        break After::CommandSeparator()
                    }
                    parser_input::part::Kind::NextCharacterEscaper() => match parser_input.next() {
                        None => (),
                        Some(escaped_part) => {
                            match escaped_part.kind {
                                parser_input::part::Kind::Literal() => push(part),
                                _ => (),
                            }
                            part = escaped_part
                        }
                    },
                    parser_input::part::Kind::WordSeparator() => break After::WordSeparator(),
                    _ => (),
                }
                push(part);
            }
        }
    };
    (
        first.map(|first| {
            rest.shrink_to_fit();
            Positioned {
                position: first.position,
                entity: Word {
                    characters: NonEmptyVec {
                        first: first.content,
                        rest,
                    },
                },
            }
        }),
        after,
    )
}
