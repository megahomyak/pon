pub enum Kind {
    CommandSeparator(),
    WordSeparator(),
    PonInputOpener(),
    PonInputCloser(),
    NextCharacterEscaper(),
    Literal(),
}

pub trait Part {
    type Content;
    type Position;
    type Container: Extend<Self::Content> + Default;

    fn position(&self) -> Self::Position;
    fn content(&self) -> Self::Content;
    fn kind(&self) -> Kind;
}
