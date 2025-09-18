use std::fmt::Debug;

pub mod clock;
pub mod code_generator;
pub mod email_sender;
pub mod subscription_repository;
pub mod token_validator;

pub trait Adapter: Clone + Debug + Send + Sync + 'static {}
