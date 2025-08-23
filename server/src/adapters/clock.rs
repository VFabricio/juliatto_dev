use super::Adapter;
use crate::cross_cutting::time::DateTime;

pub trait Clock: Adapter {
    fn now(&self) -> DateTime;
}
