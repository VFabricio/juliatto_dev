use uuid::Uuid;

use crate::adapters::{Adapter, code_generator::CodeGenerator};

#[derive(Clone, Debug)]
pub struct RandomCodeGenerator;

impl Adapter for RandomCodeGenerator {}

impl CodeGenerator for RandomCodeGenerator {
    fn generate(&self) -> String {
        Uuid::new_v4().to_string()
    }
}
