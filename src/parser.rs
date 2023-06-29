type ParsingResult<'a, T> = parco::Result<T, parco::PositionedString<'a>, Error>;
type CollResult<'a, T> = parco::CollResult<T, parco::PositionedString<'a>, Error>;
type StdString = std::string::String;

pub struct String {
    pub content: StdString,
}

pub struct Word {
    pub content: StdString,
}

pub struct Filler {
    pub contents: Program,
}

pub enum NamePart {
    Word(Word),
    String(String),
    Filler(Filler),
}

pub struct Name {
    pub parts: Vec<NamePart>,
}

pub struct Program {
    pub action_names: Vec<Name>,
}

pub enum Error {
    UnclosedFiller { opening_position: parco::Position },
    UnclosedString { opening_position: parco::Position },
    UnexpectedFillerClosure { position: parco::Position },
    UnexpectedStringClosure { position: parco::Position },
}

impl<T> From<Error> for ParsingResult<'_, T> {
    fn from(err: Error) -> Self {
        ParsingResult::Fatal(err)
    }
}

fn skip_whitespace(mut s: parco::PositionedString) -> parco::PositionedString {
    use parco::Input;
    loop {
        match s.take_one_part() {
            Some((c, rest)) if c.is_whitespace() => s = rest,
            _ => return s,
        }
    }
}

fn shrink_string(mut s: StdString) -> StdString {
    s.shrink_to_fit();
    s
}

fn shrink_vec<T>(mut v: Vec<T>) -> Vec<T> {
    v.shrink_to_fit();
    v
}

mod string {
    use super::*;

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<String> {
        let opening_position = rest.position;
        parco::one_matching_part(rest, |c| *c == '{')
            .and(|_, rest| {
                parco::collect_repeating(StdString::new(), rest, |rest| {
                    parco::one_matching_part(*rest, |c| !"{}".contains(*c))
                        .map(|c| StdString::from(c))
                        .or(|| parse(*rest).map(|string| string.content))
                })
                .norm()
            })
            .and(|content, rest| {
                parco::one_matching_part(rest, |c| *c == '}')
                    .or(|| Error::UnclosedString { opening_position }.into())
                    .map(|_| String { content })
            })
    }
}

mod word {
    use super::*;

    fn part(rest: parco::PositionedString) -> ParsingResult<char> {
        parco::one_matching_part(rest, |c| !"{}()".contains(*c) && !c.is_whitespace())
    }

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<Word> {
        part(rest)
            .and(|c, rest| {
                parco::collect_repeating(std::string::String::from(c), rest, |rest| part(*rest))
                    .norm()
            })
            .map(|content| Word { content })
    }
}

mod filler {
    use super::*;

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<Filler> {}
}

mod name_part {
    use super::*;

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<NamePart> {}
}

mod name {
    use super::*;

    fn skip_useless(mut s: parco::PositionedString) -> parco::PositionedString {
        use parco::Input;
        loop {
            match s.take_one_part() {
                Some((c, rest)) if c.is_whitespace() && c != '\n' => s = rest,
                _ => return s,
            }
        }
    }

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<Name> {
        name_part::parse(rest)
            .and(|part, rest| {
                parco::collect_repeating(Vec::from([part]), rest, |rest| {
                    name_part::parse(skip_useless(*rest))
                })
                .norm()
            })
            .map(|parts| Name {
                parts: shrink_vec(parts),
            })
    }
}

mod program {
    use super::*;

    pub fn parse(rest: parco::PositionedString) -> CollResult<Program> {
        parco::collect_repeating(Vec::new(), rest, |rest| name::parse(skip_whitespace(*rest))).map(
            |action_invocations| Program {
                action_names: action_invocations,
            },
        )
    }
}

pub fn parse(program: parco::PositionedString) -> Result<Program, Error> {
    match program::parse(program) {
        CollResult::Ok(program, rest) => {
            use parco::Input;
            if let Some((c, _)) = skip_whitespace(rest).take_one_part() {
                return Err(match c {
                    '}' => Error::UnexpectedStringClosure {
                        position: rest.position,
                    },
                    ')' => Error::UnexpectedFillerClosure {
                        position: rest.position,
                    },
                    _ => panic!("something wasn't parsed. Rest: {}", rest.content),
                });
            }
            Ok(program)
        }
        CollResult::Fatal(err) => Err(err),
    }
}
