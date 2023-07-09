use std::sync::Arc;

use downcast_rs::DowncastSync;

pub type SharedFiller = Arc<dyn Filler>;

pub trait Filler: DowncastSync {
    fn to_string(&self) -> String {
        format!("<{}>", self.content_to_string())
    }
    fn content_to_string(&self) -> String;
}
downcast_rs::impl_downcast!(sync FillerOps);
