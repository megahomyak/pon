pub struct PonString {
    content: String,
}

impl super::Filler for PonString {
    fn content_to_string(&self) -> String {
        self.content.clone()
    }

    fn to_string(&self) -> String {
        format!("{{{}}}", self.content_to_string())
    }
}
