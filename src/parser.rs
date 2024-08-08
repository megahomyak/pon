pub mod parser_input {
    pub struct Next<I: ParserInput> {
        pub part: part::Part<I>,
        pub rest: I,
    }
    pub trait ParserInput: Sized {
        type PartContent;
        type PartContainer: Extend<Self::PartContent> + Default;
        type PartContainerNonEmpty;
        type PartPosition;
        type Part: part::Part<Self>;

        fn next(self) -> Option<Next<Self>>;
    }
    pub mod part {
        use super::*;

        pub mod container {
            use super::*;

            pub struct NonEmpty<T>(pub T);

            pub trait Container<I: ParserInput> {
                fn push(&mut self, part_content: I::PartContent);
                fn as_non_empty(self) -> Result<I::PartContainerNonEmpty, Self>;
            }
        }
        pub enum Kind {
            WordSeparator,
        }
        pub trait Part<I: ParserInput> {
            fn content(&self) -> I::PartContent;
            fn position(&self) -> I::PartPosition;
            fn kind(&self) -> Kind;
        }
    }
}

pub struct Positioned<I: parser_input::ParserInput, T> {
    pub position: I::PartPosition,
    pub entity: T,
}

pub mod invocation_input {
    use super::*;

    pub struct InvocationInput<I: parser_input::ParserInput> {
        pub content: I::PartContainer,
    }
}

pub mod object_name {
    use super::*;

    pub struct ObjectName<I: parser_input::ParserInput> {
        pub content: I::PartContainer,
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
