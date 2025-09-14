use std::fmt::Debug;

pub mod clock;
pub mod token_validator;

pub trait Adapter: Clone + Debug + Send + Sync + 'static {}
