#[derive(Debug)]
pub struct NonEmptyVec<T> {
    pub first: T,
    pub rest: Vec<T>,
}
