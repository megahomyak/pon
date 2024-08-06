pub mod implementations;
pub mod part;

pub trait Input<P, C>: Clone + Iterator<Item = part::Part<P, C>> {}
