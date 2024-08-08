pub mod parser_input {
    pub struct Next<P, R> {
        pub part: P,
        pub rest: R,
    }
    pub trait ParserInput: Sized {
        type Part: part::Part;

        fn next(self) -> Option<Next<Self::Part, Self>>;
    }
    pub mod part {
        use super::*;

        pub mod container {
            use super::*;

            pub trait Container: Sized {
                type NonEmpty;
                type Item;

                fn push(&mut self, item: Self::Item);
                fn as_non_empty(self) -> Result<Self::NonEmpty, Self>;
            }
        }
        pub enum Kind {
            WordSeparator(),
            InvocationInputOpener(),
            InvocationInputCloser(),
            Escape(),
            Literal(),
        }
        pub trait Part {
            type Container: container::Container<Item = Self>;
            type Content;
            type Position;

            fn content(&self) -> Self::Content;
            fn position(&self) -> Self::Position;
            fn kind(&self) -> Kind;
        }
        // The lines below were added for short types
        pub type Position<I: ParserInput> = <I::Part as Part>::Position;
        pub type Container<I: ParserInput> = <I::Part as Part>::Container;
        pub type Content<I: ParserInput> = <I::Part as Part>::Content;
    }
}

pub struct Positioned<I: parser_input::ParserInput, T> {
    pub position: parser_input::part::Position<I>,
    pub entity: T,
}

pub mod invocation_input {
    use super::*;

    pub struct InvocationInput<I: parser_input::ParserInput> {
        pub content: parser_input::part::Container<I>,
    }
}

pub mod object_name {
    use super::*;

    pub struct ObjectName<I: parser_input::ParserInput> {
        pub content: parser_input::part::,
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
