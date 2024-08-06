mod name;
mod pon_input;

use crate::parser::parser_input;
use crate::parser::Positioned;

#[derive(Debug)]
pub struct Named<P> {
    pub name: Positioned<P, name::Name>,
    pub inputs: Vec<Positioned<P, pon_input::Input>>,
}
#[derive(Debug)]
pub struct Unnamed<P> {
    pub inputs: NonEmptyVec<Positioned<P, pon_input::Input>>,
}
#[derive(Debug)]
pub enum Command<P> {
    Named(Named<P>),
    Unnamed(Unnamed<P>),
}
pub(super) enum After {
    CommandSeparator(),
    MissingInputTerminator { opener_index: parser_input::Index },
    EscapeAtEndOfInput(),
    ParserInputEnd(),
}
pub(super) fn parse(parser_input: &mut parser_input::Input) -> (Option<Command>, After) {
    let (name, after_name) = name::parse(parser_input);
    let mut inputs = Vec::new();
    let after = match after_name {
        name::After::ParserInputEnd() => After::ParserInputEnd(),
        name::After::PonInputOpener(opener_index) => loop {
            let (input, after_input) = pon_input::parse(parser_input);
            match after_input {
                pon_input::After::ParserInputEnd() => {
                    break After::MissingInputTerminator { opener_index }
                }
                pon_input::After::EscapeAtEndOfInput() => break After::EscapeAtEndOfInput(),
                pon_input::After::PonInputTerminator() => (),
            }
            inputs.push(Positioned {
                position: opener_index,
                entity: input,
            });
            let mut new_parser_input = parser_input.clone();
            let (name, after_name) = name::parse(&mut new_parser_input);
            if name.is_some() {
                break After::CommandSeparator();
            }
            match after_name {
                name::After::EscapeAtEndOfInput() => break After::EscapeAtEndOfInput(),
                name::After::PonInputOpener(_index) => (),
                name::After::ParserInputEnd() => break After::ParserInputEnd(),
                name::After::CommandSeparator() => break After::CommandSeparator(),
            }
            *parser_input = new_parser_input;
        },
        name::After::CommandSeparator() => After::CommandSeparator(),
        name::After::EscapeAtEndOfInput() => After::EscapeAtEndOfInput(),
    };
    let command = match name {
        None => {
            let mut inputs = inputs.into_iter();
            inputs.next().map(|first| {
                let mut rest: Vec<_> = inputs.collect();
                rest.shrink_to_fit();
                Command::Unnamed(Unnamed {
                    inputs: NonEmptyVec { first, rest },
                })
            })
        }
        Some(name) => {
            inputs.shrink_to_fit();
            Some(Command::Named(Named { inputs, name }))
        }
    };
    (command, after)
}
