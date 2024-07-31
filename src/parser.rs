use crate::non_empty::{self, NonEmptyString, NonEmptyVec};

#[derive(Debug, Clone, Copy)]
pub struct Index(pub usize);

#[derive(Clone)]
struct ParserInput<'a> {
    index: Index,
    s: &'a str,
}
impl<'a> ParserInput<'a> {
    fn next(&mut self) -> Option<char> {
        unsafe { self.s.get_unchecked(self.index.0..) }
            .chars()
            .next()
            .inspect(|c| self.index.0 += c.len_utf8())
    }
}

#[derive(Debug)]
pub struct Word {
    pub characters: NonEmptyString,
}
impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.characters == other.characters
    }
}
enum AfterWord {
    CommandSeparator(),
    WordSeparator(),
    PonInputOpener(Index),
    ParserInputEnd(),
    EscapeAtEndOfInput(),
}
fn word(s: &mut ParserInput) -> (Option<Word>, AfterWord) {
    struct First(Index, char);
    let mut first = None;
    let mut rest = String::new();
    let after = loop {
        let index = s.index;
        match s.next() {
            None => break AfterWord::ParserInputEnd(),
            Some(mut c) => {
                match c {
                    '(' => break AfterWord::PonInputOpener(index),
                    ';' | '\n' => break AfterWord::CommandSeparator(),
                    '\\' => match s.next() {
                        None => break AfterWord::EscapeAtEndOfInput(),
                        Some(escaped_c) => c = escaped_c,
                    },
                    _ if c.is_whitespace() => break AfterWord::WordSeparator(),
                    _ => (),
                }
                match first {
                    None => first = Some(First(index, c)),
                    Some(_) => rest.push(c),
                }
            }
        }
    };
    (
        first.map(|First(position, c)| {
            rest.shrink_to_fit();
            Word {
                position,
                characters: NonEmptyString {
                    first: non_empty::First(c),
                    rest: non_empty::Rest(rest),
                },
            }
        }),
        after,
    )
}

#[derive(Debug)]
pub struct Name {
    pub position: Index,
    pub words: NonEmptyVec<Word>,
}
impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        other.words == self.words
    }
}
enum AfterName {
    CommandSeparator(),
    PonInputOpener(Index),
    ParserInputEnd(),
    EscapeAtEndOfInput(),
}
fn name(s: &mut ParserInput) -> (Option<Name>, AfterName) {
    struct First(Index, Word);
    let mut first = None;
    let mut rest = Vec::new();
    let after = loop {
        let (word, after_word) = word(s);
        if let Some(word) = word {
            match first {
                None => first = Some(First(word.position, word)),
                Some(_) => rest.push(word),
            }
        }
        match after_word {
            AfterWord::EscapeAtEndOfInput() => break AfterName::EscapeAtEndOfInput(),
            AfterWord::CommandSeparator() => break AfterName::CommandSeparator(),
            AfterWord::PonInputOpener(index) => break AfterName::PonInputOpener(index),
            AfterWord::ParserInputEnd() => break AfterName::ParserInputEnd(),
            AfterWord::WordSeparator() => (),
        }
    };
    (
        first.map(|First(position, word)| {
            rest.shrink_to_fit();
            Name {
                position,
                words: NonEmptyVec {
                    first: non_empty::First(word),
                    rest: non_empty::Rest(rest),
                },
            }
        }),
        after,
    )
}

#[derive(Debug)]
pub struct Positioned<T> {
    pub position: Index,
    pub entity: T,
}

#[derive(Debug)]
pub struct PonInput {
    pub content: String,
}
enum AfterPonInput {
    PonInputTerminator(),
    ParserInputEnd(),
    EscapeAtEndOfInput(),
}
fn input(s: &mut ParserInput) -> (PonInput, AfterPonInput) {
    let mut content = String::new();
    let mut nesting_level = 0;
    let after = loop {
        match s.next() {
            None => break AfterPonInput::ParserInputEnd(),
            Some(mut c) => {
                match c {
                    ')' => {
                        if nesting_level == 0 {
                            break AfterPonInput::PonInputTerminator();
                        }
                        nesting_level -= 1;
                    }
                    '(' => nesting_level += 1,
                    '\\' => match s.next() {
                        None => break AfterPonInput::EscapeAtEndOfInput(),
                        Some(escaped_c) => {
                            content.push(c);
                            c = escaped_c;
                        }
                    },
                    _ => (),
                }
                content.push(c);
            }
        }
    };
    content.shrink_to_fit();
    (PonInput { content }, after)
}

#[derive(Debug)]
pub struct NamedCommand {
    name: Name,
    inputs: Vec<Positioned<PonInput>>,
}
#[derive(Debug)]
pub struct UnnamedCommand {
    inputs: NonEmptyVec<Positioned<PonInput>>,
}
#[derive(Debug)]
pub enum CommandKind {
    Named(NamedCommand),
    Unnamed(UnnamedCommand),
}
#[derive(Debug)]
pub struct Command {
    pub position: Index,
    pub kind: CommandKind,
}
enum AfterCommand {
    CommandSeparator(),
    MissingInputTerminator { opener_index: Index },
    EscapeAtEndOfInput(),
    ParserInputEnd(),
}
fn command(s: &mut ParserInput) -> (Option<Command>, AfterCommand) {
    use name as parse_name;
    let (name, after_name) = parse_name(s);
    let mut inputs = Vec::new();
    let after = match after_name {
        AfterName::ParserInputEnd() => AfterCommand::ParserInputEnd(),
        AfterName::PonInputOpener(opener_index) => loop {
            let (input, after_input) = input(s);
            match after_input {
                AfterPonInput::ParserInputEnd() => {
                    break AfterCommand::MissingInputTerminator { opener_index }
                }
                AfterPonInput::EscapeAtEndOfInput() => break AfterCommand::EscapeAtEndOfInput(),
                AfterPonInput::PonInputTerminator() => (),
            }
            inputs.push(Positioned {
                position: opener_index,
                entity: input,
            });
            let mut new_s = s.clone();
            let (name, after_name) = parse_name(&mut new_s);
            if name.is_some() {
                break AfterCommand::CommandSeparator();
            }
            match after_name {
                AfterName::EscapeAtEndOfInput() => break AfterCommand::EscapeAtEndOfInput(),
                AfterName::PonInputOpener(_index) => (),
                AfterName::ParserInputEnd() => break AfterCommand::ParserInputEnd(),
                AfterName::CommandSeparator() => break AfterCommand::CommandSeparator(),
            }
            *s = new_s;
        },
        AfterName::CommandSeparator() => AfterCommand::CommandSeparator(),
        AfterName::EscapeAtEndOfInput() => AfterCommand::EscapeAtEndOfInput(),
    };
    let command = match name {
        None => {
            let mut inputs = inputs.into_iter();
            inputs.next().map(|input| {
                let mut rest: Vec<_> = inputs.collect();
                rest.shrink_to_fit();
                let position = input.position;
                Command {
                    kind: CommandKind::Unnamed(UnnamedCommand {
                        inputs: NonEmptyVec {
                            first: non_empty::First(input),
                            rest: non_empty::Rest(rest),
                        },
                    }),
                    position,
                }
            })
        }
        Some(name) => {
            inputs.shrink_to_fit();
            Some(Command {
                position: name.position,
                kind: CommandKind::Named(NamedCommand { name, inputs }),
            })
        }
    };
    (command, after)
}

#[derive(Debug)]
pub struct Program(pub Vec<Command>);
#[derive(Debug)]
pub enum AfterProgram {
    EscapeAtEndOfInput(),
    ParserInputEnd(),
    MissingInputTerminator { opener_index: Index },
}
pub fn program(s: &str) -> (Program, AfterProgram) {
    let mut s = ParserInput { s, index: Index(0) };
    let mut commands = Vec::new();
    let after = loop {
        let (command, after_command) = command(&mut s);
        if let Some(command) = command {
            commands.push(command);
        }
        match after_command {
            AfterCommand::CommandSeparator() => (),
            AfterCommand::EscapeAtEndOfInput() => break AfterProgram::EscapeAtEndOfInput(),
            AfterCommand::ParserInputEnd() => break AfterProgram::ParserInputEnd(),
            AfterCommand::MissingInputTerminator { opener_index } => {
                break AfterProgram::MissingInputTerminator { opener_index }
            }
        }
    };
    commands.shrink_to_fit();
    (Program(commands), after)
}
