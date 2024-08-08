pub mod parser_input {}

pub mod non_empty_string {
    pub struct NonEmptyString {
        content: String,
    }

    impl NonEmptyString {
        pub fn new(content: String) -> Result<Self, String> {
            if content.is_empty() {
                Err(content)
            } else {
                Ok(Self { content })
            }
        }

        pub fn first(&self) -> char {
            return unsafe { self.content.chars().next().unwrap_unchecked() };
        }

        pub fn rest(&self) -> &str {
            return unsafe { self.content.get_unchecked(self.first().len_utf8()..).chars().next().unwrap_unchecked() };
        }
    }
}

pub struct Positioned<T> {
    pub position: usize,
    pub entity: T,
}

pub mod invocation_input {
    use super::*;

    pub struct InvocationInput {
        pub content: String,
    }
}

pub mod object_name {
    use super::*;

    pub struct ObjectName {
        pub content: NonEmptyString,
    }
}

pub mod command {
    use super::*;

    pub enum Kind<I: parser_input::ParserInput> {
        LoadObject(object_name::ObjectName<I>),
        InvokeLoadedObject(invocation_input::InvocationInput<I>),
    }
    pub struct Command<I: parser_input::ParserInput> {
        pub position: I::PartPosition,
        pub kind: Kind<I>,
    }
}

pub mod program {
    use super::*;

    pub struct Program {
        pub commands: Vec<Command>,
    }
}
