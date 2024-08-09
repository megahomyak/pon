pub mod parser_input {
    pub mod part {
        pub enum Kind {
            WordSeparator(),
            InvocationInputCloser(),
            InvocationInputOpener(),
            Escape(),
            Literal(),
        }

        pub struct Part<C, P> {
            pub content: C,
            pub position: P,
            pub kind: Kind,
        }
    }

    pub trait ParserInput: Sized {
        type PartContent;
        type PartPosition;

        fn next(self) -> Option<(part::Part<Self::PartContent, Self::PartPosition>, Self)>;
    }
}
pub use parser_input::ParserInput;

pub mod non_empty_vec {
    pub struct NonEmptyVec<T> {
        pub first: T,
        pub rest: Vec<T>,
    }

    pub struct Incomplete<T> {
        pub first: Option<T>,
        pub rest: Vec<T>,
    }

    impl<T> Incomplete<T> {
        pub fn new() -> Self {
            Self {
                first: None,
                rest: Vec::new(),
            }
        }
        pub fn push(&mut self, t: T) {
            match self.first {
                None => self.first = Some(t),
                Some(_) => self.rest.push(t),
            }
        }
        pub fn finalize(self) -> Option<NonEmptyVec<T>> {
            self.first.map(|first| NonEmptyVec {
                first,
                rest: self.rest,
            })
        }
    }
}
pub use non_empty_vec::NonEmptyVec;

pub mod positioned {
    pub struct Positioned<T> {
        pub position: usize,
        pub entity: T,
    }
}
pub use positioned::Positioned;

pub mod program {
    use super::*;

    pub mod command {
        use super::*;

        pub mod object_name {
            use super::*;

            pub mod word {
                use super::*;

                pub struct Word<I: ParserInput> {
                    pub content: NonEmptyVec<I::PartContent>,
                }

                pub enum After<I: ParserInput> {
                    WordSeparator(),
                    InvocationInputCloser(),
                    InvocationInputOpener(I::PartPosition),
                    Literal(),
                    ParserInputEnd(),
                }

                pub fn parse<I: ParserInput>(mut i: I) -> (Option<Word<I>>, After<I>, Option<I>) {
                    let mut content = non_empty_vec::Incomplete::new();
                    let (after, i) = loop {
                        match i.next() {
                            None => break After::ParserInputEnd(),
                            Some((mut part, new_i)) => {
                                i = new_i;
                                use parser_input::part;
                                match part.kind {
                                    part::Kind::Escape() => {
                                        if let Some((new_part, new_i)) = i.next() {
                                            i = new_i;
                                            if let part::Kind::Literal() = new_part.kind {
                                                content.push(part.content);
                                            }
                                            part = new_part;
                                        } else {

                                        }
                                    }
                                    part::Kind::Literal() => (),
                                    part::Kind::WordSeparator() => break After::WordSeparator(),
                                    part::Kind::InvocationInputCloser() => {
                                        break After::InvocationInputCloser()
                                    }
                                    part::Kind::InvocationInputOpener() => {
                                        break After::InvocationInputOpener(part.position)
                                    }
                                }
                                content.push(part.content);
                            }
                        }
                    };
                    (content.finalize().map(|content| Word { content }), after, i)
                }
            }
            pub use word::Word;

            pub struct ObjectName<I: ParserInput> {
                pub content: NonEmptyVec<Word<I>>,
            }
        }
        pub use object_name::ObjectName;

        pub mod invocation_input {
            use super::*;

            pub struct InvocationInput<I: ParserInput> {
                pub content: NonEmptyVec<I::PartContent>,
            }
        }
        pub use invocation_input::InvocationInput;

        pub enum Command<I: ParserInput> {
            LoadObject(ObjectName<I>),
            InvokeLoadedObject(InvocationInput<I>),
        }
    }
    pub use command::Command;

    pub struct Program<I: ParserInput> {
        pub commands: Vec<Positioned<Command<I>>>,
    }
}
