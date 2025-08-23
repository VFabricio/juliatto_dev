use crate::adapters::{Adapter, clock::Clock};
use crate::cross_cutting::time::DateTime;

#[derive(Clone)]
pub struct SystemClock {}

impl SystemClock {
    pub fn new() -> Self {
        Self {}
    }
}

impl Adapter for SystemClock {}

impl Clock for SystemClock {
    fn now(&self) -> DateTime
    where
        Self: Sized,
    {
        DateTime::now()
    }
}
