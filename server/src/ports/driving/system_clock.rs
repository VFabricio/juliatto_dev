use crate::adapters::{Adapter, clock::Clock};
use crate::cross_cutting::time::DateTime;

#[derive(Clone, Debug)]
pub struct SystemClock {}

impl SystemClock {
    pub fn new() -> Self {
        Self {}
    }
}

impl Adapter for SystemClock {}

impl Clock for SystemClock {
    fn now(&self) -> DateTime {
        DateTime::now()
    }
}
