use std::fmt::Debug;

pub mod clock;

pub trait Adapter: Clone + Debug + Send + Sync + 'static {}
