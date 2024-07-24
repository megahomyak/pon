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

mod non_empty {
    #[derive(Debug)]
    pub struct NonEmpty<T>(T);
    impl<T> AsRef<T> for NonEmpty<T> {
        fn as_ref(&self) -> &T {
            &self.0
        }
    }
    impl<T: Container> NonEmpty<T> {
        pub fn new(mut t: T) -> Option<Self> {
            if t.is_empty() {
                None
            } else {
                t.shrink_to_fit();
                Some(Self(t))
            }
        }
    }
    pub trait Container {
        fn is_empty(&self) -> bool;
        /// Shrink the container to occupy the minimal amount of memory possible, preserving
        /// the container's contents
        fn shrink_to_fit(&mut self);
    }
    impl Container for String {
        fn is_empty(&self) -> bool {
            String::is_empty(self)
        }
        fn shrink_to_fit(&mut self) {
            String::shrink_to_fit(self);
        }
    }
    impl<T> Container for Vec<T> {
        fn is_empty(&self) -> bool {
            Vec::is_empty(self)
        }
        fn shrink_to_fit(&mut self) {
            Vec::shrink_to_fit(self);
        }
    }
}
pub use non_empty::NonEmpty;

#[derive(Debug)]
pub struct Word(pub NonEmpty<String>);
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
    (NonEmpty::new(word).map(|word| Word(word)), after)
}

#[derive(Debug)]
pub struct Name(pub NonEmpty<Vec<Word>>);
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
                        AfterPonInput::ParserInputEnd() => {
                            break AfterCommand::MissingInputTerminator()
                        }
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

#[derive(Debug)]
pub struct Program(pub Vec<Command>);
#[derive(Debug)]
pub enum AfterProgram {
    EscapeAtEndOfInput(),
    ParserInputEnd(),
    MissingInputTerminator(),
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
            AfterCommand::MissingInputTerminator() => break AfterProgram::MissingInputTerminator(),
        }
    };
    commands.shrink_to_fit();
    (Program(commands), after)
}
