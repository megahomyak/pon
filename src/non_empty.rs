#[derive(Debug)]
pub struct First<T>(pub T);
#[derive(Debug)]
pub struct Rest<T>(pub T);

#[derive(Debug)]
pub struct NonEmptyVec<T>(pub First<T>, pub Rest<Vec<T>>);
impl<T> NonEmptyVec<T> {
    pub(crate) fn from(mut v: Vec<T>) -> Option<Self> {
        v.shrink_to_fit(); // Nice to do
        let drained = v.drain(..=1).next();
        drained.map(|first| Self(First(first), Rest(v)))
    }
}

#[derive(Debug)]
pub struct NonEmptyString(pub First<char>, pub Rest<String>);
impl NonEmptyString {
    pub(crate) fn from(mut s: String) -> Option<Self> {
        s.shrink_to_fit(); // Nice to do
        let drained = s.drain(..=1).next();
        drained.map(|first| Self(First(first), Rest(s)))
    }
}
