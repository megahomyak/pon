#[derive(Clone)]
pub struct ParserInput<'a> {
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

pub struct Word(pub String);
enum AfterWord {
    CommandSeparator(),
    WordSeparator(),
    PonInputOpener(),
    ParserInputEnd(),
    EscapeAtEndOfInput(),
}
fn word(s: &mut ParserInput) -> (Option<Word>, AfterWord) {
    let mut word = String::new();
    let after = loop {
        match s.next() {
            None => break AfterWord::ParserInputEnd(),
            Some(mut c) => {
                match c {
                    '(' => break AfterWord::PonInputOpener(),
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
    let word = if word.is_empty() {
        None
    } else {
        word.shrink_to_fit();
        Some(Word(word))
    };
    (word, after)
}

pub struct Name(pub Vec<Word>);
enum AfterName {
    CommandSeparator(),
    PonInputOpener(),
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
            AfterWord::PonInputOpener() => break AfterName::PonInputOpener(),
            AfterWord::ParserInputEnd() => break AfterName::ParserInputEnd(),
            AfterWord::WordSeparator() => (),
        }
    };
    let name = if name.is_empty() {
        None
    } else {
        name.shrink_to_fit();
        Some(Name(name))
    };
    (name, after)
}

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

pub struct Command(pub Option<Name>, pub Vec<PonInput>);
enum AfterCommand {
    CommandSeparator(),
    MissingInputTerminator(),
    EscapeAtEndOfInput(),
    ParserInputEnd(),
}
fn command(s: &mut ParserInput) -> (Option<Command>, AfterCommand) {
    use name as parse_name;
    let (name, after_name) = parse_name(s);
    let mut pon_inputs = Vec::new();
    let after = loop {
        match after_name {
            AfterName::ParserInputEnd() => break AfterCommand::ParserInputEnd(),
            AfterName::PonInputOpener() => {
                break loop {
                    let (pon_input, after_pon_input) = pon_input(s);
                    match after_pon_input {
                        AfterPonInput::ParserInputEnd() => break AfterCommand::MissingInputTerminator(),
                        AfterPonInput::EscapeAtEndOfInput() => {
                            break AfterCommand::EscapeAtEndOfInput()
                        }
                        AfterPonInput::PonInputTerminator() => (),
                    }
                    pon_inputs.push(pon_input);
                    let mut new_s = s.clone();
                    let (name, after_name) = parse_name(&mut new_s);
                    if name.is_some() {
                        break AfterCommand::CommandSeparator();
                    }
                    match after_name {
                        AfterName::EscapeAtEndOfInput() => {
                            break AfterCommand::EscapeAtEndOfInput()
                        }
                        AfterName::PonInputOpener() => (),
                        AfterName::ParserInputEnd() => break AfterCommand::ParserInputEnd(),
                        AfterName::CommandSeparator() => break AfterCommand::CommandSeparator(),
                    }
                    *s = new_s;
                }
            }
            AfterName::CommandSeparator() => break AfterCommand::CommandSeparator(),
            AfterName::EscapeAtEndOfInput() => break AfterCommand::EscapeAtEndOfInput(),
        }
    };
    let command = if name.is_none() && pon_inputs.is_empty() {
        None
    } else {
        pon_inputs.shrink_to_fit();
        Some(Command(name, pon_inputs))
    };
    (command, after)
}

pub struct Program(Vec<Command>);
pub enum AfterProgram {
    EscapeAtEndOfInput(),
    ParserInputEnd(),
    MissingInputTerminator(),
}
pub fn program(s: &mut ParserInput) -> (Program, AfterProgram) {
    let mut commands = Vec::new();
    let after = loop {
        let (command, after_command) = command(s);
        if let Some(command) = command {
            commands.push(command);
        }
        match after_command {
            AfterCommand::CommandSeparator() => (),
            AfterCommand::EscapeAtEndOfInput() => break AfterProgram::EscapeAtEndOfInput(),
            AfterCommand::ParserInputEnd() => break AfterProgram::ParserInputEnd(),
            AfterCommand::MissingInputTerminator() => break AfterProgram::MissingInputTerminator(),
        }
    };
    commands.shrink_to_fit();
    (Program(commands), after)
}
