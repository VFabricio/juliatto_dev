use super::Adapter;

pub trait CodeGenerator: Adapter {
    fn generate(&self) -> String;
}
