use std::path::Path;
use crate::types::Callable;
use serde::Serialize;

#[derive(Serialize)]
pub struct Analysis {
    callables: Vec<Callable>
}

impl Analysis {
    pub fn new(callables: Vec<Callable>) -> Self {
        Self {
            callables: callables.clone()
        }
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) {
        let content = serde_json::to_string(&self.callables).unwrap();
        std::fs::write(path, content).unwrap();
    }
}