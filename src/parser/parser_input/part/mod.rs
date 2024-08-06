pub enum Kind {
    CommandSeparator(),
    WordSeparator(),
    PonInputOpener(),
    PonInputCloser(),
    NextCharacterEscaper(),
    Literal(),
}

pub struct Part<P, C> {
    pub position: P,
    pub content: C,
    pub kind: Kind,
}
