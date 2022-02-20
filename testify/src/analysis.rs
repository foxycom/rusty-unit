use crate::types::Callable;
use serde::Serialize;

#[derive(Serialize)]
pub struct Analysis {
    callables: Vec<Callable>
}

impl Analysis {
    pub fn new(callables: &Vec<Callable>) -> Self {
        Self {
            callables: callables.clone()
        }
    }
}