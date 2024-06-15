pub trait Trimmable: Sized {
    type Part;

    fn trim(&self) -> Option<(Self::Part, Self)>;
}
