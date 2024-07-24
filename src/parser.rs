mod input {
    pub struct Input<'a> {
        current_index: usize,
        content: &'a str,
    }
    impl<'a> Input<'a> {
        pub fn new(s: &'a str) -> Self {
            Self {
                content: s,
                current_index: 0,
            }
        }
        pub fn next(&mut self) -> Option<char> {
            unsafe { self.content.get_unchecked(self.current_index..) }
                .chars()
                .next()
                .inspect(|c| self.current_index += c.len_utf8())
        }
        pub fn content(&self) -> &str {
            self.content
        }
        pub fn current_index(&self) -> usize {
            self.current_index
        }
    }
}
use input::Input;

struct WordSeparator(char);
struct CommandInvocationSeparator(char);
struct InputOpener(char);
struct InputCloser(char);

enum InvalidInput {
    EscapeAtEndOfInput(),
}
use InvalidInput as II_;

enum Result<Success, UnexpectedInput> {
    Success(Success),
    UnexpectedInput(UnexpectedInput),
    InvalidInput(InvalidInput),
}
use Result::InvalidInput as II;
use Result::Success as S;
use Result::UnexpectedInput as UI;

mod program {
    use super::*;
    mod command_invocation {
        use super::*;
        mod command_name {
            use super::*;
            mod word {
                use super::*;
                mod char_ {
                    use super::*;
                    struct Char(char);
                    enum UnexpectedInput {
                        WordSeparator(WordSeparator),
                        CommandInvocationSeparator(CommandInvocationSeparator),
                        InputOpener(InputOpener),
                        InputCloser(InputCloser),
                        InputEnd(),
                    }
                    use UnexpectedInput as UI_;
                    fn parse(s: &mut Input) -> Result<Char, UnexpectedInput> {
                        match s.next() {
                            None => UI(UI_::InputEnd()),
                            Some(c @ ';') => UI(UI_::CommandInvocationSeparator(
                                CommandInvocationSeparator(c),
                            )),
                            Some(c @ '(') => UI(UI_::InputOpener(InputOpener(c))),
                            Some(c @ ')') => UI(UI_::InputCloser(InputCloser(c))),
                            Some('\\') => match s.next() {
                                None => II(InvalidInput::EscapeAtEndOfInput()),
                                Some(c) => S(Char(c)),
                            },
                            Some(c) if c.is_whitespace() => {
                                UI(UI_::WordSeparator(WordSeparator(c)))
                            }
                            Some(c) => S(Char(c)),
                        }
                    }
                }
            }
        }
        use super::*;
        mod command_input {
            use super::*;
            mod contents {
                use super::*;
                mod char_ {
                    use super::*;
                }
            }
        }
    }

    enum ParsingResult {}

    pub fn parse(s: Input) -> ParsingResult {}
}

struct CommandNameWord(Vec<CommandNameWordChar>);
enum CommandNameWordParsingResult {
    // Unexpected input
    Whitespace(Whitespace, Option<CommandNameWord>),
    Semicolon(Semicolon, Option<CommandNameWord>),
    OpeningParen(OpeningParen, Option<CommandNameWord>),
    ClosingParen(ClosingParen, Option<CommandNameWord>),
    Nothing(Option<CommandNameWord>),
    // Invalid input
    EscapeAtEnd(),
}
fn parse_word(s: &mut Input) -> CommandNameWordParsingResult {
    use CommandNameWordCharParsingResult as I;
    use CommandNameWordParsingResult as O;
    let mut word = Vec::new();
    let pack = move |mut word: Vec<CommandNameWordChar>| {
        if word.is_empty() {
            None
        } else {
            word.shrink_to_fit();
            Some(CommandNameWord(word))
        }
    };
    loop {
        match parse_word_char(s) {
            I::Nothing() => return O::Nothing(pack(word)),
            I::EscapeAtEnd() => return O::EscapeAtEnd(),
            I::Valid(c) => word.push(c),
            I::Semicolon(c) => return O::Semicolon(c, pack(word)),
            I::OpeningParen(c) => return O::OpeningParen(c, pack(word)),
            I::Whitespace(c) => return O::Whitespace(c, pack(word)),
            I::ClosingParen(c) => return O::ClosingParen(c, pack(word)),
        }
    }
}

struct CommandName(Vec<CommandNameWord>);
enum NameParsingResult {
    // Unexpected input
    Semicolon(Semicolon, Option<CommandName>),
    OpeningParen(OpeningParen, Option<CommandName>),
    ClosingParen(ClosingParen, Option<CommandName>),
    Nothing(Option<CommandName>),
    // Invalid input
    EscapeAtEnd(),
}
fn parse_name(s: &mut Input) -> NameParsingResult {
    use CommandNameWordParsingResult as I;
    use NameParsingResult as O;
    let mut words = Vec::new();
    let mut maybe_add = |word: Option<CommandNameWord>| {
        if let Some(w) = word {
            words.push(w);
        }
    };
    let pack = |mut words: Vec<CommandNameWord>| {
        if words.is_empty() {
            None
        } else {
            words.shrink_to_fit();
            Some(CommandName(words))
        }
    };
    loop {
        match parse_word(s) {
            I::EscapeAtEnd() => return O::EscapeAtEnd(),
            I::Nothing(w) => {
                maybe_add(w);
                return O::Nothing(pack(words));
            }
            I::ClosingParen(c, w) => {
                maybe_add(w);
                return O::ClosingParen(c, pack(words));
            }
            I::OpeningParen(c, w) => {
                maybe_add(w);
                return O::OpeningParen(c, pack(words));
            }
            I::Semicolon(c, w) => {
                maybe_add(w);
                return O::Semicolon(c, pack(words));
            }
            I::Whitespace(_c, w) => {
                maybe_add(w);
            }
        }
    }
}

enum InputChar {
    // Success
    Valid(char),
    ValidEscaped(char, char),
    // Unexpected input
    OpeningParen(char),
    ClosingParen(char),
    // Invalid input
    EscapeAtEnd,
    Nothing,
}
impl InputChar {
    fn parse(s: &mut Input) -> Self {
        match s.next() {
            None => Self::Nothing,
            Some(c @ '(') => Self::OpeningParen(c),
            Some(c @ ')') => Self::ClosingParen(c),
            Some(c1 @ '\\') => match s.next() {
                None => Self::EscapeAtEnd,
                Some(c2) => Self::ValidEscaped(c1, c2),
            },
            Some(c) => Self::Valid(c),
        }
    }
}

enum InputContents {
    // Unexpected input
    ClosingParen(char, String),
    // Invalid input
    EscapeAtEnd,
    Nothing,
}
impl InputContents {
    fn parse(s: &mut Input) -> Self {
        let mut chars = String::new();
        let mut nesting_level = 0;
        loop {
            match InputChar::parse(s) {
                InputChar::Nothing => return Self::Nothing,
                InputChar::EscapeAtEnd => return Self::EscapeAtEnd,
                InputChar::Valid(c) => chars.push(c),
                InputChar::ValidEscaped(c1, c2) => {
                    chars.push(c1);
                    chars.push(c2);
                }
                InputChar::OpeningParen(c) => {
                    nesting_level += 1;
                    chars.push(c);
                }
                InputChar::ClosingParen(c) => {
                    if nesting_level == 0 {
                        return Self::ClosingParen(c, chars);
                    }
                    nesting_level -= 1;
                    chars.push(c);
                }
            }
        }
    }
}
