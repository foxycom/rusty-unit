use crate::chromosome::{Arg, Param};
use syn::{Expr, FnArg, Lit, Type};

#[derive(Debug, Default, Clone)]
pub struct TestIdGenerator {
    id: u64,
}

impl TestIdGenerator {
    pub fn new() -> TestIdGenerator {
        TestIdGenerator {
            id: Default::default(),
        }
    }

    pub fn next_id(&mut self) -> u64 {
        self.id += 1;
        self.id
    }

    pub fn reset(&mut self) {
        self.id = Default::default()
    }
}
