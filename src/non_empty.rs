#[derive(Debug, PartialEq, Eq)]
pub struct First<T>(pub T);
#[derive(Debug, PartialEq, Eq)]
pub struct Rest<T>(pub T);

#[derive(Debug, PartialEq, Eq)]
pub struct NonEmptyVec<T> {
    pub first: First<T>,
    pub rest: Rest<Vec<T>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NonEmptyString {
    pub first: First<char>,
    pub rest: Rest<String>,
}
