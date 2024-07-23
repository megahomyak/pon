type Input<'a> = std::str::CharIndices<'a>;

struct Pos { index: usize }
enum Result<'a, O, E> {
    Success { output: O, rest: Input<'a> },
    Failure { error: E, span: std::ops::RangeInclusive<Pos> }
}
use Result::*;

mod word_char {
    use super::*;

    pub enum Error {
        Whitespace(char),
        Semicolon(),
        EscapeAtEnd(),

    }

    pub fn parse(s: Input) -> Result<char, Error> {
        use Error::*;

        match s.next() {
            
        }

        match  {
            '\\' => match
            ';' => Err(Semicolon),
            c if c.is_whitespace() => Success(Whitespace(c)),
            c => Success(WordChar(c)),
        }
    }
}
