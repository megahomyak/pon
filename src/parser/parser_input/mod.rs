pub mod implementations;
pub mod part;

pub struct Next<T: Input> {
    pub part: T::Part,
    pub rest: T,
}

pub trait Input: Clone + Iterator<Item = Self::Part> {
    type Part: part::Part;

    fn next(self) -> Option<Next<Self>>;
}
