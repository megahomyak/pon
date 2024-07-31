#[derive(Debug, PartialEq, Eq)]
pub struct NonEmptyVec<T> {
    pub first: T,
    pub rest: Vec<T>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NonEmptyString {
    pub first: char,
    pub rest: String,
}
