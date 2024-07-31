use crate::non_empty::{self, NonEmptyString, NonEmptyVec};

#[derive(Debug, Clone, Copy)]
pub struct Index(pub usize);

#[derive(Debug)]
pub struct Positioned<T> {
    pub position: Index,
    pub entity: T,
}

mod parser_input {
    use super::*;

    pub struct Part {
        pub position: Index,
        pub character: char,
    }

    #[derive(Clone)]
    pub struct Input<'a>(std::str::CharIndices<'a>);

    impl<'a> Input<'a> {
        pub fn new(input: &'a str) -> Self {
            Self(input.char_indices())
        }
    }

    impl<'a> Iterator for Input<'a> {
        type Item = Part;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next().map(|(index, content)| Part {
                position: Index(index),
                character: content,
            })
        }
    }
}
use parser_input::Input as ParserInput;

mod word {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct Word {
        pub characters: NonEmptyString,
    }
    pub enum After {
        CommandSeparator(),
        WordSeparator(),
        PonInputOpener(Index),
        ParserInputEnd(),
        EscapeAtEndOfInput(),
    }
    pub fn parse(parser_input: &mut ParserInput) -> (Option<Positioned<Word>>, After) {
        let mut first = None;
        let mut rest = String::new();
        let after = loop {
            match parser_input.next() {
                None => break After::ParserInputEnd(),
                Some(mut part) => {
                    match part.character {
                        '(' => break After::PonInputOpener(part.position),
                        ';' | '\n' => break After::CommandSeparator(),
                        '\\' => match parser_input.next() {
                            None => break After::EscapeAtEndOfInput(),
                            Some(escaped_part) => part = escaped_part,
                        },
                        character if character.is_whitespace() => break After::WordSeparator(),
                        _ => (),
                    }
                    match first {
                        None => first = Some(part),
                        Some(_) => rest.push(part.character),
                    }
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
                            first: non_empty::First(first.character),
                            rest: non_empty::Rest(rest),
                        },
                    },
                }
            }),
            after,
        )
    }
}

mod name {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct Name {
        pub words: NonEmptyVec<word::Word>,
    }
    pub enum After {
        CommandSeparator(),
        PonInputOpener(Index),
        ParserInputEnd(),
        EscapeAtEndOfInput(),
    }
    pub fn parse(parser_input: &mut ParserInput) -> (Option<Positioned<Name>>, After) {
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
                            first: non_empty::First(first.entity),
                            rest: non_empty::Rest(rest),
                        },
                    },
                }
            }),
            after,
        )
    }
}

mod pon_input {
    use super::*;

    #[derive(Debug)]
    pub struct Input {
        pub content: String,
    }
    pub enum After {
        PonInputTerminator(),
        ParserInputEnd(),
        EscapeAtEndOfInput(),
    }
    pub fn parse(parser_input: &mut ParserInput) -> (Input, After) {
        let mut content = String::new();
        let mut nesting_level = 0;
        let after = loop {
            match parser_input.next() {
                None => break After::ParserInputEnd(),
                Some(mut part) => {
                    match part.character {
                        ')' => {
                            if nesting_level == 0 {
                                break After::PonInputTerminator();
                            }
                            nesting_level -= 1;
                        }
                        '(' => nesting_level += 1,
                        escape_character @ '\\' => match parser_input.next() {
                            None => break After::EscapeAtEndOfInput(),
                            Some(escaped_part) => {
                                content.push(escape_character);
                                part = escaped_part;
                            }
                        },
                        _ => (),
                    }
                    content.push(part.character);
                }
            }
        };
        content.shrink_to_fit();
        (Input { content }, after)
    }
}

mod command {
    use super::*;

    #[derive(Debug)]
    pub struct Named {
        name: Positioned<name::Name>,
        inputs: Vec<Positioned<pon_input::Input>>,
    }
    #[derive(Debug)]
    pub struct Unnamed {
        inputs: NonEmptyVec<Positioned<pon_input::Input>>,
    }
    #[derive(Debug)]
    pub enum Command {
        Named(Named),
        Unnamed(Unnamed),
    }
    pub enum After {
        CommandSeparator(),
        MissingInputTerminator { opener_index: Index },
        EscapeAtEndOfInput(),
        ParserInputEnd(),
    }
    pub fn parse(parser_input: &mut ParserInput) -> (Option<Command>, After) {
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
                inputs.next().map(|input| {
                    let mut rest: Vec<_> = inputs.collect();
                    rest.shrink_to_fit();
                    Command::Unnamed(Unnamed {
                        inputs: NonEmptyVec {
                            first: non_empty::First(input),
                            rest: non_empty::Rest(rest),
                        },
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
}

pub mod program {
    use super::*;

    #[derive(Debug)]
    pub struct Program(pub Vec<command::Command>);
    #[derive(Debug)]
    pub enum After {
        EscapeAtEndOfInput(),
        ParserInputEnd(),
        MissingInputTerminator { opener_index: Index },
    }
    pub fn parse(parser_input: &str) -> (Program, After) {
        let mut parser_input = ParserInput::new(parser_input);
        let mut commands = Vec::new();
        let after = loop {
            let (command, after_command) = command::parse(&mut parser_input);
            if let Some(command) = command {
                commands.push(command);
            }
            match after_command {
                command::After::CommandSeparator() => (),
                command::After::EscapeAtEndOfInput() => break After::EscapeAtEndOfInput(),
                command::After::ParserInputEnd() => break After::ParserInputEnd(),
                command::After::MissingInputTerminator { opener_index } => {
                    break After::MissingInputTerminator { opener_index }
                }
            }
        };
        commands.shrink_to_fit();
        (Program(commands), after)
    }
}
