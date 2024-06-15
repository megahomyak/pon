pub enum TrimmingResult<P, S> {
    NothingToTrim,
    Trimmed {
        part: P, rest: S,
    }
}

pub trait Trimmable: Sized {
    type Part;

    fn trim(&self) -> TrimmingResult<Self::Part, Self>;
}
