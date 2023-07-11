use std::sync::Arc;

use downcast_rs::DowncastSync;

pub type SharedFiller = Arc<dyn Filler>;

pub trait Filler: DowncastSync {
    fn to_string(&self) -> String {
        format!("<{}>", self.content_to_string())
    }
    fn content_to_string(&self) -> String;
}
downcast_rs::impl_downcast!(sync Filler);

pub struct PonString {
    pub content: String,
}

impl Filler for PonString {
    fn content_to_string(&self) -> String {
        self.content.clone()
    }

    fn to_string(&self) -> String {
        format!("{{{}}}", self.content)
    }
}

pub struct Nothing {}

impl Filler for Nothing {
    fn content_to_string(&self) -> String {
        "nothing".to_owned()
    }
}
