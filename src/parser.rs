use crate::non_empty::NonEmptyVec;

#[derive(Debug)]
pub struct First<T>(pub T);
#[derive(Debug)]
pub struct Rest<T>(pub T);

#[derive(Debug)]
pub struct Index(pub usize);

#[derive(Clone)]
struct ParserInput<'a> {
    idx: usize,
    s: &'a str,
}
impl<'a> ParserInput<'a> {
    fn next(&mut self) -> Option<char> {
        unsafe { self.s.get_unchecked(self.idx..) }
            .chars()
            .next()
            .inspect(|c| self.idx += c.len_utf8())
    }
}

enum WordCharResult {
    Valid(char),
    CommandSeparator(),
    WordSeparator(),
    PonInputOpener(Index),
    ParserInputEnd(),
    EscapeAtEndOfInput(),
}
fn word_char(s: &mut ParserInput) -> WordCharResult {
    let index = s.idx;
    match s.next() {
        None => AfterWord::ParserInputEnd(),
        Some(mut c) => {
            match c {
                '(' => AfterWord::PonInputOpener(Index(index)),
                ';' | '\n' => break AfterWord::CommandSeparator(),
                '\\' => match s.next() {
                    None => break AfterWord::EscapeAtEndOfInput(),
                    Some(escaped_c) => c = escaped_c,
                },
                _ if c.is_whitespace() => break AfterWord::WordSeparator(),
                _ => (),
            }
        }
    }
}

#[derive(Debug)]
pub struct Word(pub NonEmptyVec<WordChar>);
enum AfterWord {
    CommandSeparator(),
    WordSeparator(),
    PonInputOpener(Index),
    ParserInputEnd(),
    EscapeAtEndOfInput(),
}
fn word(s: &mut ParserInput) -> (Option<Word>, AfterWord) {
    let mut word = Vec::new();
    let char = |mut s| {
        let index = s.idx;
        match s.next() {
            None => break AfterWord::ParserInputEnd(),
            Some(mut c) => {
                match c {
                    '(' => break AfterWord::PonInputOpener(Index(index)),
                    ';' | '\n' => break AfterWord::CommandSeparator(),
                    '\\' => match s.next() {
                        None => break AfterWord::EscapeAtEndOfInput(),
                        Some(escaped_c) => c = escaped_c,
                    },
                    _ if c.is_whitespace() => break AfterWord::WordSeparator(),
                    _ => (),
                }
                word.push(c);
            }
        }
    };
    let after = loop {};
    let iter = word.into_iter();
    iter.next().map(|first| NonEmptyVec {
        first,
        rest: iter.collect(),
    })
}

#[derive(Debug)]
pub struct Name(pub NonEmpty<Vec<Word>>);
enum AfterName {
    CommandSeparator(),
    PonInputOpener(Index),
    ParserInputEnd(),
    EscapeAtEndOfInput(),
}
fn name(s: &mut ParserInput) -> (Option<Name>, AfterName) {
    let mut name = Vec::new();
    let after = loop {
        let (word, after_word) = word(s);
        if let Some(word) = word {
            name.push(word);
        }
        match after_word {
            AfterWord::EscapeAtEndOfInput() => break AfterName::EscapeAtEndOfInput(),
            AfterWord::CommandSeparator() => break AfterName::CommandSeparator(),
            AfterWord::PonInputOpener(index) => break AfterName::PonInputOpener(index),
            AfterWord::ParserInputEnd() => break AfterName::ParserInputEnd(),
            AfterWord::WordSeparator() => (),
        }
    };
    (NonEmpty::new(name).map(|name| Name(name)), after)
}

#[derive(Debug)]
pub struct PonInput(pub String);
enum AfterPonInput {
    PonInputTerminator(),
    ParserInputEnd(),
    EscapeAtEndOfInput(),
}
fn pon_input(s: &mut ParserInput) -> (PonInput, AfterPonInput) {
    let mut input = String::new();
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
                            input.push(c);
                            c = escaped_c;
                        }
                    },
                    _ => (),
                }
                input.push(c);
            }
        }
    };
    input.shrink_to_fit();
    (PonInput(input), after)
}

#[derive(Debug)]
pub enum CommandKind {
    Named(Name, Vec<PonInput>),
    Unnamed(NonEmpty<Vec<PonInput>>),
}
#[derive(Debug)]
pub struct Command(pub Index, pub CommandKind);
enum AfterCommand {
    CommandSeparator(),
    MissingInputTerminator { opener_index: Index },
    EscapeAtEndOfInput(),
    ParserInputEnd(),
}
fn command(s: &mut ParserInput) -> (Option<Command>, AfterCommand) {
    use name as parse_name;
    let index = Index(s.idx);
    let (name, after_name) = parse_name(s);
    let mut pon_inputs = Vec::new();
    let after = match after_name {
        AfterName::ParserInputEnd() => AfterCommand::ParserInputEnd(),
        AfterName::PonInputOpener(opener_index) => loop {
            let (pon_input, after_pon_input) = pon_input(s);
            match after_pon_input {
                AfterPonInput::ParserInputEnd() => {
                    break AfterCommand::MissingInputTerminator { opener_index }
                }
                AfterPonInput::EscapeAtEndOfInput() => break AfterCommand::EscapeAtEndOfInput(),
                AfterPonInput::PonInputTerminator() => (),
            }
            pon_inputs.push(pon_input);
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
        None => match NonEmpty::new(pon_inputs) {
            None => None,
            Some(pon_inputs) => Some(CommandKind::Unnamed(pon_inputs)),
        },
        Some(name) => {
            pon_inputs.shrink_to_fit();
            Some(CommandKind::Named(name, pon_inputs))
        }
    }
    .map(|kind| Command(index, kind));
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
    let mut s = ParserInput { s, idx: 0 };
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
