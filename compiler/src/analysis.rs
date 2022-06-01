use std::path::Path;
use crate::types::RuCallable;
use serde::Serialize;

#[derive(Serialize)]
pub struct Analysis {
  callables: Vec<RuCallable>,
}

impl Analysis {
  pub fn new(callables: Vec<RuCallable>) -> Self {
    Self {
      callables: callables.clone()
    }
  }

  pub fn to_file<P: AsRef<Path>>(&self, path: P) {
    let content = serde_json::to_string(&self.callables).unwrap();
    std::fs::write(path, content).unwrap();
  }
}