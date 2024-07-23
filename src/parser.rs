mod s {
    pub struct S<'a> {
        idx: usize,
        s: &'a str,
    }
    impl<'a> S<'a> {
        pub fn new(s: &'a str) -> Self {
            Self { s, idx: 0 }
        }
        pub fn next(&mut self) -> Option<char> {
            unsafe { self.s.get_unchecked(self.idx..) }
                .chars()
                .next()
                .inspect(|c| self.idx += c.len_utf8())
        }
        pub fn s(&self) -> &str {
            self.s
        }
        pub fn idx(&self) -> usize {
            self.idx
        }
    }
}
use s::S;

struct Whitespace(char);
struct Semicolon(char);
struct OpeningParen(char);
struct ClosingParen(char);

mod program {
    mod command_invocation {
        mod command_name {
            mod word {
                mod char_ {

                }
            }
        }
        mod command_input {
            mod contents {
                mod char_ {

                }
            }
        }
    }
}

struct CommandNameWordChar(char);
enum CommandNameWordCharParsingResult {
    // Success
    Valid(CommandNameWordChar),
    // Unexpected input
    Whitespace(Whitespace),
    Semicolon(Semicolon),
    OpeningParen(OpeningParen),
    ClosingParen(ClosingParen),
    // Invalid input
    EscapeAtEnd(),
    Nothing(),
}
fn parse_word_char(s: &mut S) -> CommandNameWordCharParsingResult {
    use CommandNameWordCharParsingResult as O;
    match s.next() {
        None => O::Nothing(),
        Some(c @ ';') => O::Semicolon(Semicolon(c)),
        Some(c @ '(') => O::OpeningParen(OpeningParen(c)),
        Some(c @ ')') => O::ClosingParen(ClosingParen(c)),
        Some('\\') => match s.next() {
            None => O::EscapeAtEnd(),
            Some(c) => O::Valid(CommandNameWordChar(c)),
        },
        Some(c) if c.is_whitespace() => O::Whitespace(Whitespace(c)),
        Some(c) => O::Valid(CommandNameWordChar(c)),
    }
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
fn parse_word(s: &mut S) -> CommandNameWordParsingResult {
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
fn parse_name(s: &mut S) -> NameParsingResult {
    use NameParsingResult as O;
    use CommandNameWordParsingResult as I;
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
    fn parse(s: &mut S) -> Self {
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
    fn parse(s: &mut S) -> Self {
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
