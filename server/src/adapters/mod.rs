pub mod clock;

pub trait Adapter: Clone + Send + Sync + 'static {}
