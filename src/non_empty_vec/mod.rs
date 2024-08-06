#[derive(Debug, PartialEq, Eq)]
pub struct NonEmptyVec<T> {
    pub first: T,
    pub rest: Vec<T>,
}
