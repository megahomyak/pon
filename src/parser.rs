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

struct WordChar(char);
enum WordCharParsingResult {
    // Success
    Valid(WordChar),
    // Unexpected input
    Whitespace(Whitespace),
    Semicolon(Semicolon),
    OpeningParen(OpeningParen),
    ClosingParen(ClosingParen),
    // Invalid input
    EscapeAtEnd(),
    Nothing(),
}
fn parse_word_char(s: &mut S) -> WordCharParsingResult {
    use WordCharParsingResult as O;
    match s.next() {
        None => O::Nothing(),
        Some(c @ ';') => O::Semicolon(Semicolon(c)),
        Some(c @ '(') => O::OpeningParen(OpeningParen(c)),
        Some(c @ ')') => O::ClosingParen(ClosingParen(c)),
        Some('\\') => match s.next() {
            None => O::EscapeAtEnd(),
            Some(c) => O::Valid(WordChar(c)),
        },
        Some(c) if c.is_whitespace() => O::Whitespace(Whitespace(c)),
        Some(c) => O::Valid(WordChar(c)),
    }
}

struct Word(Vec<WordChar>);
enum WordParsingResult {
    // Unexpected input
    Whitespace(Whitespace, Word),
    Semicolon(Semicolon, Word),
    OpeningParen(OpeningParen, Word),
    ClosingParen(ClosingParen, Word),
    Nothing(Word),
    // Invalid input
    EscapeAtEnd,
}
fn parse_word(s: &mut S) -> WordParsingResult {
    use WordCharParsingResult as I;
    use WordParsingResult as O;
    let mut word = Vec::new();
    loop {
        match parse_word_char(s) {
            I::Nothing() => return O::Nothing(Word(word)),
            I::EscapeAtEnd() => return O::EscapeAtEnd,
            I::Valid(c) => word.push(c),
            I::Semicolon(c) => return O::Semicolon(c, Word(word)),
            I::OpeningParen(c) => return O::OpeningParen(c, Word(word)),
            I::Whitespace(c) => return O::Whitespace(c, Word(word)),
            I::ClosingParen(c) => return O::ClosingParen(c, Word(word)),
        }
    }
}

struct Name(Vec<Word>);
enum NameParsingResult {
    // Unexpected input
    Semicolon(Semicolon, Name),
    OpeningParen(OpeningParen, Name),
    ClosingParen(ClosingParen, Name),
    Nothing(Name),
    // Invalid input
    EscapeAtEnd,
}
fn parse_name(s: &mut S) -> NameParsingResult {
    use WordParsingResult as I;
    use NameParsingResult as O;
    let mut words = Vec::new();
    loop {
        match parse_word(s) {
            I::EscapeAtEnd => return O::EscapeAtEnd,
            I::Nothing(w) => {
                if !w.is_empty() {
                    words.push(w);
                }
                return O::Nothing(words);
            }
            I::ClosingParen(c, w) => {
                if !w.is_empty() {
                    words.push(w);
                }
                return Self::ClosingParen(c, words);
            }
            I::OpeningParen(c, w) => {
                if !w.is_empty() {
                    words.push(w);
                }
                return Self::OpeningParen(c, words);
            }
            I::Semicolon(c, w) => {
                if !w.is_empty() {
                    words.push(w);
                }
                return Self::Semicolon(c, words);
            }
            I::Whitespace(_c, w) => {
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
