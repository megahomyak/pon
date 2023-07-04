use downcast_rs::DowncastSync;

mod string;

trait Filler: DowncastSync {
    fn to_string(&self) -> String {
        format!("<{}>", self.content_to_string())
    }
    fn content_to_string(&self) -> String;
}
downcast_rs::impl_downcast!(sync Filler);
