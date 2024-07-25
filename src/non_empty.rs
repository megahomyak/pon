#[derive(Debug)]
pub struct NonEmpty<T>(T);
impl<T> NonEmpty<T> {
    pub fn unpack(self) -> T {
        self.0
    }
    pub fn read(&self) -> &T {
        &self.0
    }
}
impl<T: Container> NonEmpty<T> {
    pub fn new(mut t: T) -> Option<Self> {
        if t.is_empty() {
            None
        } else {
            t.shrink_to_fit();
            Some(Self(t))
        }
    }
}
pub trait Container {
    fn is_empty(&self) -> bool;
    /// Shrink the container to occupy the minimal amount of memory possible, preserving
    /// the container's contents
    fn shrink_to_fit(&mut self);
}
impl Container for String {
    fn is_empty(&self) -> bool {
        String::is_empty(self)
    }
    fn shrink_to_fit(&mut self) {
        String::shrink_to_fit(self);
    }
}
impl<T> Container for Vec<T> {
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
    fn shrink_to_fit(&mut self) {
        Vec::shrink_to_fit(self);
    }
}
