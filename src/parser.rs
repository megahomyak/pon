use parco::CollResult;

type ParsingResult<'a, T> = parco::Result<T, parco::PositionedString<'a>, Error>;

pub struct String {
    pub content: std::string::String,
}

pub struct Word {
    pub content: std::string::String,
}

pub struct Filler {
    pub contents: Vec<Name>,
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
}

fn skip_useless(mut s: parco::PositionedString) -> parco::PositionedString {
    use parco::Input;
    loop {
        match s.take_one_part() {
            Some((c, rest)) if c.is_whitespace() && c != '\n' => s = rest,
            _ => return s,
        }
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

mod string {
    use super::*;

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<String> {}
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

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<Name> {

    }
}

pub fn parse(program: parco::PositionedString) -> Result<Program, Error> {
    match parco::collect_repeating(Vec::new(), program, |rest| {
        name::parse(skip_whitespace(*rest))
    }) {
        CollResult::Ok(action_names, rest) => {
            if skip_whitespace(rest).content != "" {
                panic!("something wasn't parsed. Rest: {}", rest.content)
            }
            Ok(Program { action_names })
        }
        CollResult::Fatal(err) => Err(err),
    }
}
