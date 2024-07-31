use crate::non_empty::{NonEmptyString, NonEmptyVec};

#[derive(Debug)]
pub struct Positioned<T> {
    pub position: parser_input::position::Position,
    pub entity: T,
}

pub mod parser_input {
    pub mod position {
        #[derive(Debug, Clone, Copy)]
        pub struct Position<'a> {
            pub(super) source: &'a str,
            pub(super) index: usize,
        }

        pub struct MarkedLine<'a> {
            pub before_mark: &'a str,
            pub rest: &'a str,
        }

        pub struct Computed<'a> {
            pub row_number: usize,
            pub column_number: usize,
            pub line: MarkedLine<'a>,
        }

        impl<'a> Position<'a> {
            pub fn compute(&self) -> Computed {
                let mut column_number = 0;
                let mut row_number = 1;
                let mut line_beginning_index = 0;
                let mut before_mark = None;
                let mut source = super::Input::new(&self.source);
                let mut rest = loop {

                }
                for part in source {
                    if part.position.index == self.index {
                        before_mark = Some(unsafe {
                            self.source.get_unchecked(line_beginning_index..self.index)
                        });
                    }
                    if part.character == '\n' {
                        if before_mark.is_some() {
                            return 
                        }
                    }
                }
                Computed {
                    column_number,
                    row_number,
                    line,
                }
            }
        }
    }

    pub(super) struct Part<'a> {
        pub position: position::Position<'a>,
        pub character: char,
    }

    #[derive(Clone)]
    pub(super) struct Input<'a> {
        source: &'a str,
        index: usize,
    }

    impl<'a> Input<'a> {
        pub fn new(source: &'a str) -> Self {
            Self { source, index: 0 }
        }
    }

    impl<'a> Iterator for Input<'a> {
        type Item = Part<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            unsafe { self.source.get_unchecked(self.index..) }
                .chars()
                .next()
                .map(|character| {
                    let part = Part {
                        character,
                        position: position::Position {
                            index: self.index,
                            source: &self.source,
                        },
                    };
                    self.index += character.len_utf8();
                    part
                })
        }
    }
}

pub mod word {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct Word {
        pub characters: NonEmptyString,
    }
    pub(super) enum After {
        CommandSeparator(),
        WordSeparator(),
        PonInputOpener(parser_input::Index),
        ParserInputEnd(),
        EscapeAtEndOfInput(),
    }
    pub(super) fn parse(
        parser_input: &mut parser_input::Input,
    ) -> (Option<Positioned<Word>>, After) {
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
                            first: first.character,
                            rest,
                        },
                    },
                }
            }),
            after,
        )
    }
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

    #[derive(Debug)]
    pub struct Input {
        pub content: String,
    }
    pub(super) enum After {
        PonInputTerminator(),
        ParserInputEnd(),
        EscapeAtEndOfInput(),
    }
    pub(super) fn parse(parser_input: &mut parser_input::Input) -> (Input, After) {
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

pub mod command {
    use super::*;

    #[derive(Debug)]
    pub struct Named {
        pub name: Positioned<name::Name>,
        pub inputs: Vec<Positioned<pon_input::Input>>,
    }
    #[derive(Debug)]
    pub struct Unnamed {
        pub inputs: NonEmptyVec<Positioned<pon_input::Input>>,
    }
    #[derive(Debug)]
    pub enum Command {
        Named(Named),
        Unnamed(Unnamed),
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
}

pub mod program {
    use super::*;

    #[derive(Debug)]
    pub struct Program(pub Vec<command::Command>);
    #[derive(Debug)]
    pub enum After {
        EscapeAtEndOfInput(),
        ParserInputEnd(),
        MissingInputTerminator { opener_index: parser_input::Index },
    }
    pub fn parse(parser_input: &mut parser_input::Input) -> (Program, After) {
        let mut commands = Vec::new();
        let after = loop {
            let (command, after_command) = command::parse(parser_input);
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
