pub mod implementations;
pub mod part;

pub struct Next<T: Input> {
    pub part: T::Part,
    pub rest: T,
}

pub trait Input: Clone {
    type PartContent;
    type PartPosition;
    type PartContainer: Extend<Self::PartContent> + Default;
    type Part: part::Part<
        Content = Self::PartContent,
        Position = Self::PartPosition,
        Container = Self::PartContainer,
    >;

    fn next(self) -> Option<Next<Self>>;
}
