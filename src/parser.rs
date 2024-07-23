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

pub enum Error {
    EscapeAtEndOfInput(),
}
pub type Result<T> = std::result::Result<T, Error>;

struct WordSeparator(char);
struct CommandInvocationSeparator(char);
struct InputContentsOpener(char);
struct InputContentsCloser(char);

struct WordChar(char);
enum WordCharParsingResultUnexpectedInput {
    WordSeparator(WordSeparator),
    CommandInvocationSeparator(CommandInvocationSeparator),
    InputContentsOpener(InputContentsOpener),
    InputContentsCloser(InputContentsCloser),
    EndOfInput(),
}
enum WordCharParsingResult {
    Valid(WordChar),
    UnexpectedInput(WordCharParsingResultUnexpectedInput),
}
fn parse_word_char(s: &mut S) -> Result<WordCharParsingResult> {
    use WordCharParsingResult as O;
    use O::UnexpectedInput as UI;
    use WordCharParsingResultUnexpectedInput as UIK;
    Ok(match s.next() {
        None => UI(UIK::EndOfInput()),
        Some(c @ ';') => O::UnexpectedSemicolon(CommandInvocationSeparator(c)),
        Some(c @ '(') => O::UnexpectedOpeningParen(InputContentsOpener(c)),
        Some(c @ ')') => O::UnexpectedClosingParen(InputContentsCloser(c)),
        Some('\\') => match s.next() {
            None => return Err(Error::EscapeAtEndOfInput()),
            Some(c) => O::Valid(WordChar(c)),
        },
        Some(c) if c.is_whitespace() => O::UnexpectedWhitespace(WordSeparator(c)),
        Some(c) => O::Valid(WordChar(c)),
    })
}

struct Word(Vec<WordChar>);
enum WordParsingResultUnexpectedInput {
    Whitespace(WordSeparator),
    Semicolon(CommandInvocationSeparator),
    OpeningParen(InputContentsOpener),
    ClosingParen(InputContentsCloser),
    EndOfInput(),
}
struct WordParsingResult {
    word: Option<Word>,

    UnexpectedWhitespace(WordSeparator, Option<Word>),
    UnexpectedSemicolon(CommandInvocationSeparator, Option<Word>),
    UnexpectedOpeningParen(InputContentsOpener, Option<Word>),
    UnexpectedClosingParen(InputContentsCloser, Option<Word>),
    UnexpectedEndOfInput(Option<Word>),
}
fn parse_word(s: &mut S) -> Result<WordParsingResult> {
    use WordCharParsingResult as I;
    use WordParsingResult as O;
    let mut word = Vec::new();
    let pack = move |mut word: Vec<WordChar>| if word.is_empty() {
        None
    } else {
        word.shrink_to_fit();
        Some(Word(word))
    };
    loop {
        match parse_word_char(s)? {
            I::UnexpectedEndOfInput() => return Ok(O::UnexpectedEndOfInput(pack(word))),
            I::Valid(c) => word.push(c),
            I::UnexpectedSemicolon(c) => return Ok(O::UnexpectedSemicolon(c, pack(word))),
            I::UnexpectedOpeningParen(c) => return Ok(O::UnexpectedOpeningParen(c, pack(word))),
            I::UnexpectedWhitespace(c) => return O::UnexpectedWhitespace(c, pack(word)),
            I::UnexpectedClosingParen(c) => return O::UnexpectedClosingParen(c, pack(word)),
        }
    }
}

struct Name(Vec<Word>);
enum NameParsingResult {
    // Unexpected input
    UnexpectedSemicolon(CommandInvocationSeparator, Option<Name>),
    UnexpectedOpeningParen(InputContentsOpener, Option<Name>),
    UnexpectedClosingParen(InputContentsCloser, Option<Name>),
    UnexpectedEndOfInput(Option<Name>),
    // Invalid input
    EscapeAtEndOfInput(),
}
fn parse_name(s: &mut S) -> NameParsingResult {
    use WordParsingResult as I;
    use NameParsingResult as O;
    let mut words = Vec::new();
    loop {
        match parse_word(s) {
            I::EscapeAtEndOfInput() => return O::EscapeAtEndOfInput(),
            I::UnexpectedEndOfInput(w) => {
                if !w.is_empty() {
                    words.push(w);
                }
                return O::UnexpectedEndOfInput(words);
            }
            I::UnexpectedClosingParen(c, w) => {
                if !w.is_empty() {
                    words.push(w);
                }
                return Self::ClosingParen(c, words);
            }
            I::UnexpectedOpeningParen(c, w) => {
                if !w.is_empty() {
                    words.push(w);
                }
                return Self::OpeningParen(c, words);
            }
            I::UnexpectedSemicolon(c, w) => {
                if !w.is_empty() {
                    words.push(w);
                }
                return Self::Semicolon(c, words);
            }
            I::UnexpectedWhitespace(_c, w) => {
                if !w.is_empty() {
                    words.push(w);
                }
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

enum Invocation {}
impl Invocation {
    fn parse(s: &mut S) -> Self {
        let name = match Name::parse(s) {
            Name::Nothing(words) => {}
        };
    }
}
