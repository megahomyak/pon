type ParsingResult<'a, T> = parco::Result<T, parco::PositionedString<'a>, Error>;
type CollResult<'a, T> = parco::CollResult<T, parco::PositionedString<'a>, Error>;

#[derive(Debug, PartialEq, Eq)]
pub struct Filler {
    pub content: Program,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NamePart {
    Word(String),
    String(String),
    Filler(Filler),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Name {
    pub parts: Vec<NamePart>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    pub names: Vec<Name>,
}

#[derive(Debug, PartialEq, Eq)]
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

fn shrink_string(mut s: String) -> String {
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
        let mut indentation_level: usize = 0;
        parco::one_matching_part(rest, |c| *c == '{')
            .and(|_, rest| {
                parco::collect_repeating(String::new(), rest, |rest| {
                    use parco::Input;
                    match rest.take_one_part() {
                        None => Error::UnclosedString { opening_position }.into(),
                        Some((c, rest)) => {
                            if c == '}' {
                                match indentation_level.checked_sub(1) {
                                    Some(new_level) => indentation_level = new_level,
                                    None => return ParsingResult::Err,
                                }
                            } else if c == '{' {
                                indentation_level += 1;
                            }
                            ParsingResult::Ok(c, rest)
                        }
                    }
                })
                .norm()
            })
            .and(|content, rest| {
                use parco::Input;
                let Some(('}', rest)) = rest.take_one_part() else {
                    panic!("string must've ended with `}}`, in reality it ended with {:?}", rest.take_one_part())
                };
                ParsingResult::Ok(content, rest)
            })
            .map(|content| shrink_string(content))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nothing() {
            assert_eq!(parse(parco::PositionedString::from("")), ParsingResult::Err);
        }

        #[test]
        fn whitespace() {
            assert_eq!(
                parse(parco::PositionedString::from(" \t\n ")),
                ParsingResult::Err
            );
        }

        #[test]
        fn word() {
            assert_eq!(
                parse(parco::PositionedString::from("blahblah")),
                ParsingResult::Err
            );
        }

        #[test]
        fn closure() {
            assert_eq!(
                parse(parco::PositionedString::from("}")),
                ParsingResult::Err
            );
        }

        #[test]
        fn not_closed() {
            assert_eq!(
                parse(parco::PositionedString::from("{blah")),
                Error::UnclosedString {
                    opening_position: parco::Position { row: 1, column: 1 }
                }
                .into()
            );
        }

        #[test]
        fn not_closed_2() {
            assert_eq!(
                parse(parco::PositionedString::from("{")),
                Error::UnclosedString {
                    opening_position: parco::Position { row: 1, column: 1 }
                }
                .into()
            );
        }

        #[test]
        fn empty() {
            assert_eq!(
                parse(parco::PositionedString::from("{}}")),
                ParsingResult::Ok(
                    "".to_owned(),
                    parco::PositionedString {
                        content: "}",
                        position: parco::Position { column: 3, row: 1 }
                    }
                )
            );
        }

        #[test]
        fn filled() {
            assert_eq!(
                parse(parco::PositionedString::from("{abc}rest")),
                ParsingResult::Ok(
                    "abc".to_owned(),
                    parco::PositionedString {
                        content: "rest",
                        position: parco::Position { column: 6, row: 1 }
                    }
                )
            );
        }

        #[test]
        fn nested() {
            assert_eq!(
                parse(parco::PositionedString::from("{a{b}c}rest")),
                ParsingResult::Ok(
                    "a{b}c".to_owned(),
                    parco::PositionedString {
                        content: "rest",
                        position: parco::Position { column: 8, row: 1 }
                    }
                )
            );
        }

        #[test]
        fn nested_and_not_closed() {
            assert_eq!(
                parse(parco::PositionedString::from("{a{b}c")),
                Error::UnclosedString {
                    opening_position: parco::Position { row: 1, column: 1 }
                }
                .into()
            );
        }

        #[test]
        fn nested_and_not_closed_2() {
            assert_eq!(
                parse(parco::PositionedString::from("{a{bc")),
                Error::UnclosedString {
                    opening_position: parco::Position { row: 1, column: 1 }
                }
                .into()
            );
        }
    }
}

mod word {
    use super::*;

    fn part(rest: parco::PositionedString) -> ParsingResult<char> {
        parco::one_matching_part(rest, |c| !"{}()".contains(*c) && !c.is_whitespace())
    }

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<String> {
        part(rest)
            .and(|c, rest| {
                parco::collect_repeating(std::string::String::from(c), rest, |rest| part(*rest))
                    .norm()
            })
            .map(|content| shrink_string(content))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nothing() {
            assert_eq!(parse(parco::PositionedString::from("")), ParsingResult::Err);
        }

        #[test]
        fn whitespace() {
            assert_eq!(
                parse(parco::PositionedString::from("\n\t  ")),
                ParsingResult::Err
            );
        }

        #[test]
        fn special_character() {
            assert_eq!(
                parse(parco::PositionedString::from("{")),
                ParsingResult::Err
            );
        }

        #[test]
        fn special_character_2() {
            assert_eq!(
                parse(parco::PositionedString::from("(")),
                ParsingResult::Err
            );
        }

        #[test]
        fn special_character_3() {
            assert_eq!(
                parse(parco::PositionedString::from("}")),
                ParsingResult::Err
            );
        }

        #[test]
        fn special_character_4() {
            assert_eq!(
                parse(parco::PositionedString::from(")")),
                ParsingResult::Err
            );
        }

        #[test]
        fn correct_word() {
            assert_eq!(
                parse(parco::PositionedString::from("blah rest")),
                ParsingResult::Ok(
                    "blah".to_owned(),
                    parco::PositionedString {
                        content: " rest",
                        position: parco::Position { column: 5, row: 1 }
                    }
                )
            );
        }
    }
}

mod filler {
    use super::*;

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<Filler> {
        let opening_position = rest.position;
        parco::one_matching_part(rest, |c| *c == '(')
            .and(|_, rest| program::parse(rest).norm())
            .and(|contents, rest| {
                parco::one_matching_part(rest, |c| *c == ')')
                    .or(|| Error::UnclosedFiller { opening_position }.into())
                    .and(|_, rest| ParsingResult::Ok(Filler { content: contents }, rest))
            })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nothing() {
            assert_eq!(parse(parco::PositionedString::from("")), ParsingResult::Err);
        }

        #[test]
        fn not_closed() {
            assert_eq!(
                parse(parco::PositionedString::from("(")),
                Error::UnclosedFiller {
                    opening_position: parco::Position { row: 1, column: 1 }
                }
                .into()
            );
        }

        #[test]
        fn not_closed_2() {
            assert_eq!(
                parse(parco::PositionedString::from("(abc")),
                Error::UnclosedFiller {
                    opening_position: parco::Position { row: 1, column: 1 }
                }
                .into()
            );
        }

        #[test]
        fn correct() {
            assert_eq!(
                parse(parco::PositionedString::from("(abc)rest")),
                ParsingResult::Ok(
                    Filler {
                        content: Program {
                            names: vec![Name {
                                parts: vec![NamePart::Word("abc".to_owned())]
                            }]
                        }
                    },
                    parco::PositionedString {
                        content: "rest",
                        position: parco::Position { column: 6, row: 1 }
                    }
                )
            );
        }
    }
}

mod name_part {
    use super::*;

    pub fn parse(rest: parco::PositionedString) -> ParsingResult<NamePart> {
        filler::parse(rest)
            .map(|filler| NamePart::Filler(filler))
            .or(|| string::parse(rest).map(|string| NamePart::String(string)))
            .or(|| word::parse(rest).map(|word| NamePart::Word(word)))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nothing() {
            assert_eq!(parse(parco::PositionedString::from("")), ParsingResult::Err);
        }

        #[test]
        fn whitespace() {
            assert_eq!(
                parse(parco::PositionedString::from("\n\t  ")),
                ParsingResult::Err,
            );
        }

        #[test]
        fn filler() {
            assert_eq!(
                parse(parco::PositionedString::from("() rest")),
                ParsingResult::Ok(
                    NamePart::Filler(Filler {
                        content: Program { names: vec![] }
                    }),
                    parco::PositionedString {
                        content: " rest",
                        position: parco::Position { row: 1, column: 3 }
                    }
                )
            );
        }

        #[test]
        fn string() {
            assert_eq!(
                parse(parco::PositionedString::from("{abc}rest")),
                ParsingResult::Ok(
                    NamePart::String("abc".to_owned()),
                    parco::PositionedString {
                        content: "rest",
                        position: parco::Position { column: 6, row: 1 }
                    }
                )
            );
        }

        #[test]
        fn word() {
            assert_eq!(
                parse(parco::PositionedString::from("abc\nrest")),
                ParsingResult::Ok(
                    NamePart::Word("abc".to_owned()),
                    parco::PositionedString {
                        content: "\nrest",
                        position: parco::Position { column: 4, row: 1 }
                    }
                )
            );
        }
    }
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nothing() {
            assert_eq!(parse(parco::PositionedString::from("")), ParsingResult::Err);
        }

        #[test]
        fn whitespace() {
            assert_eq!(
                parse(parco::PositionedString::from("\n\t  ")),
                ParsingResult::Err,
            );
        }

        #[test]
        fn correct() {
            assert_eq!(
                parse(parco::PositionedString::from("()   blah  \nrest")),
                ParsingResult::Ok(
                    Name {
                        parts: vec![
                            NamePart::Filler(Filler {
                                content: Program { names: vec![] }
                            }),
                            NamePart::Word("blah".to_owned())
                        ]
                    },
                    parco::PositionedString {
                        content: "  \nrest",
                        position: parco::Position { row: 1, column: 10 }
                    }
                )
            );
        }

        #[test]
        fn correct_2() {
            assert_eq!(
                parse(parco::PositionedString::from("{\n}\nrest")),
                ParsingResult::Ok(
                    Name {
                        parts: vec![NamePart::String("\n".to_owned())]
                    },
                    parco::PositionedString {
                        content: "\nrest",
                        position: parco::Position { row: 2, column: 2 }
                    }
                )
            );
        }
    }
}

mod program {
    use super::*;

    pub fn parse(rest: parco::PositionedString) -> CollResult<Program> {
        parco::collect_repeating(Vec::new(), rest, |rest| name::parse(skip_whitespace(*rest))).map(
            |action_invocations| Program {
                names: shrink_vec(action_invocations),
            },
        )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nothing() {
            assert_eq!(
                parse(parco::PositionedString::from("")),
                CollResult::Ok(
                    Program { names: vec![] },
                    parco::PositionedString {
                        content: "",
                        position: parco::Position { row: 1, column: 1 }
                    }
                ),
            );
        }

        #[test]
        fn whitespace() {
            assert_eq!(
                parse(parco::PositionedString::from("\n\t  ")),
                CollResult::Ok(
                    Program { names: vec![] },
                    parco::PositionedString {
                        content: "\n\t  ",
                        position: parco::Position { row: 1, column: 1 }
                    }
                ),
            );
        }

        #[test]
        fn filled() {
            assert_eq!(
                parse(parco::PositionedString::from("one \n two \n three \n\t")),
                CollResult::Ok(
                    Program {
                        names: vec![
                            Name {
                                parts: vec![NamePart::Word("one".to_owned())]
                            },
                            Name {
                                parts: vec![NamePart::Word("two".to_owned())]
                            },
                            Name {
                                parts: vec![NamePart::Word("three".to_owned())]
                            },
                        ]
                    },
                    parco::PositionedString {
                        content: " \n\t",
                        position: parco::Position { row: 3, column: 7 }
                    }
                ),
            );
        }

        #[test]
        fn unexpected_character() {
            assert_eq!(
                parse(parco::PositionedString::from("blah}")),
                CollResult::Ok(
                    Program {
                        names: vec![Name {
                            parts: vec![NamePart::Word("blah".to_owned())]
                        }]
                    },
                    parco::PositionedString {
                        content: "}",
                        position: parco::Position { row: 1, column: 5 }
                    }
                ),
            );
        }

        #[test]
        fn unexpected_character_2() {
            assert_eq!(
                parse(parco::PositionedString::from("blah)")),
                CollResult::Ok(
                    Program {
                        names: vec![Name {
                            parts: vec![NamePart::Word("blah".to_owned())]
                        }]
                    },
                    parco::PositionedString {
                        content: ")",
                        position: parco::Position { row: 1, column: 5 }
                    }
                ),
            );
        }

        #[test]
        fn unexpected_character_3() {
            assert_eq!(
                parse(parco::PositionedString::from(")")),
                CollResult::Ok(
                    Program { names: vec![] },
                    parco::PositionedString {
                        content: ")",
                        position: parco::Position { row: 1, column: 1 }
                    }
                ),
            );
        }

        #[test]
        fn unexpected_character_4() {
            assert_eq!(
                parse(parco::PositionedString::from("blah\n  \n }")),
                CollResult::Ok(
                    Program { names: vec![] },
                    parco::PositionedString {
                        content: "\n  \n }",
                        position: parco::Position { row: 1, column: 5 }
                    }
                ),
            );
        }
    }
}

mod complete_program {
    use super::*;

    pub fn parse(program: parco::PositionedString) -> Result<Program, Error> {
        match program::parse(program) {
            CollResult::Ok(program, rest) => {
                use parco::Input;
                let rest = skip_whitespace(rest);
                if let Some((c, _)) = rest.take_one_part() {
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn nothing() {
            assert_eq!(
                parse(parco::PositionedString::from("")),
                Ok(Program { names: vec![] }),
            );
        }

        #[test]
        fn whitespace() {
            assert_eq!(
                parse(parco::PositionedString::from("\n\t  ")),
                Ok(Program { names: vec![] }),
            );
        }

        #[test]
        fn filled() {
            assert_eq!(
                parse(parco::PositionedString::from("one \n two \n three \n\t")),
                Ok(Program {
                    names: vec![
                        Name {
                            parts: vec![NamePart::Word("one".to_owned())]
                        },
                        Name {
                            parts: vec![NamePart::Word("two".to_owned())]
                        },
                        Name {
                            parts: vec![NamePart::Word("three".to_owned())]
                        },
                    ]
                }),
            );
        }

        #[test]
        fn unexpected_character() {
            assert_eq!(
                parse(parco::PositionedString::from("blah}")),
                Err(Error::UnexpectedStringClosure {
                    position: parco::Position { row: 1, column: 5 }
                })
            );
        }

        #[test]
        fn unexpected_character_2() {
            assert_eq!(
                parse(parco::PositionedString::from("blah)")),
                Err(Error::UnexpectedFillerClosure {
                    position: parco::Position { row: 1, column: 5 }
                })
            );
        }

        #[test]
        fn unexpected_character_3() {
            assert_eq!(
                parse(parco::PositionedString::from(")")),
                Err(Error::UnexpectedFillerClosure {
                    position: parco::Position { row: 1, column: 1 }
                })
            );
        }

        #[test]
        fn unexpected_character_4() {
            assert_eq!(
                parse(parco::PositionedString::from("blah\n  \n }")),
                Err(Error::UnexpectedStringClosure {
                    position: parco::Position { row: 3, column: 2 }
                })
            );
        }
    }
}

pub fn parse(program: parco::PositionedString) -> Result<Program, Error> {
    complete_program::parse(program)
}
