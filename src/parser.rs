struct S<'a> {
    idx: usize,
    s: &'a str,
}
impl S<'_> {
    fn next(&mut self) -> Option<char> {
        unsafe { self.s.get_unchecked(self.idx..) }
            .chars()
            .next()
            .inspect(|c| self.idx += c.len_utf8())
    }
}

pub enum WordChar {
    // Success
    Valid { c: char },
    // Unexpected input
    Whitespace { c: char },
    Semicolon { c: char },
    OpeningParen { c: char },
    ClosingParen { c: char },
    // Invalid input
    EscapeAtEnd,
    Nothing,
}
impl WordChar {
    pub fn parse(s: &mut S) -> Self {
        match s.next() {
            None => Self::Nothing,
            Some(c @ ';') => Self::Semicolon { c },
            Some(c @ '(') => Self::OpeningParen { c },
            Some(c @ ')') => Self::ClosingParen { c },
            Some('\\') => match s.next() {
                None => Self::EscapeAtEnd,
                Some(c) => Self::Valid { c },
            },
            Some(c) if c.is_whitespace() => Self::Whitespace { c },
            Some(c) => Self::Valid { c },
        }
    }
}

pub enum Word {
    // Unexpected input
    Whitespace { c: char, content: String },
    Semicolon { c: char, content: String },
    OpeningParen { c: char, content: String },
    ClosingParen { c: char, content: String },
    Nothing { content: String },
    // Invalid input
    EscapeAtEnd,
}
impl Word {
    pub fn parse(s: &mut S) -> Self {
        let mut content = String::new();
        loop {
            match WordChar::parse(s) {
                WordChar::Nothing => return Self::Nothing { content },
                WordChar::EscapeAtEnd => return Self::EscapeAtEnd,
                WordChar::Valid { c } => content.push(c),
                WordChar::Semicolon { c } => return Self::Semicolon { c, content },
                WordChar::OpeningParen { c } => return Self::OpeningParen { c, content },
                WordChar::Whitespace { c } => return Self::Whitespace { c, content },
                WordChar::ClosingParen { c } => return Self::ClosingParen { c, content },
            }
        }
    }
}

pub enum Name {
    // Unexpected input
    Semicolon(char, Vec<String>),
    OpeningParen(char, Vec<String>),
    ClosingParen(char, Vec<String>),
    Nothing(Vec<String>),
    // Invalid input
    EscapeAtEnd,
}
impl Name {
    pub fn parse(s: &mut S) -> Self {
        let mut words = Vec::new();
        loop {
            match Word::parse(s) {
                Word::EscapeAtEnd => return Self::EscapeAtEnd,
                Word::Nothing { content } => {
                    if !content.is_empty() {
                        words.push(content);
                    }
                    return Self::Nothing(words);
                }
                Word::ClosingParen(c, w) => {
                    if !w.is_empty() {
                        words.push(w);
                    }
                    return Self::ClosingParen(c, words);
                }
                Word::OpeningParen(c, w) => {
                    if !w.is_empty() {
                        words.push(w);
                    }
                    return Self::OpeningParen(c, words);
                }
                Word::Semicolon(c, w) => {
                    if !w.is_empty() {
                        words.push(w);
                    }
                    return Self::Semicolon(c, words);
                }
                Word::Whitespace(_c, w) => {
                    if !w.is_empty() {
                        words.push(w);
                    }
                }
            }
        }
    }
}

pub enum InputChar {
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
    pub fn parse(s: &mut S) -> Self {
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

pub enum InputContents {
    // Unexpected input
    ClosingParen(char, String),
    // Invalid input
    EscapeAtEnd,
    Nothing,
}
impl InputContents {
    pub fn parse(s: &mut S) -> Self {
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

pub enum Invocation {}
impl Invocation {
    pub fn parse(s: &mut S) -> Self {
        let name = match Name::parse(s) {
            Name::Nothing(words) => {}
        };
    }
}
