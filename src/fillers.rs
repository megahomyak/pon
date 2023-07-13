pub trait Filler: downcast_rs::DowncastSync + std::fmt::Display {}
downcast_rs::impl_downcast!(sync Filler);

pub struct PonString {
    pub content: String,
}
impl std::fmt::Display for PonString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.content)
    }
}
impl Filler for PonString {}

pub struct Nothing {}
impl std::fmt::Display for Nothing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("nothing")
    }
}
impl Filler for Nothing {}

pub struct Error {
    pub text: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {{ {} }}", self.text)
    }
}
impl Filler for Error {}
