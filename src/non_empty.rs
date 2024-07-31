#[derive(Debug)]
pub struct First<T>(pub T);
#[derive(Debug)]
pub struct Rest<T>(pub T);

#[derive(Debug)]
pub struct NonEmptyVec<T>(pub First<T>, pub Rest<Vec<T>>);
impl<T> NonEmptyVec<T> {
    pub(crate) fn from(v: Vec<T>) -> Option<Self> {
        let mut v = v.into_iter();
        v.next().map(|first| Self(First(first), Rest(v.collect())))
    }
}

#[derive(Debug)]
pub struct NonEmptyString(pub First<char>, pub Rest<String>);
impl NonEmptyString {
    pub(crate) fn from(s: String) -> Option<Self> {
        let mut s = s.chars();
        s.next().map(|first| Self(First(first), Rest(s.collect())))
    }
}
