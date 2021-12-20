use serde::{Deserialize, Serialize};
use syn::Item;

#[derive(Serialize, Deserialize, Debug)]
pub struct TestSuite {
    pub tests: Vec<String>
}

impl TestSuite {
    pub fn to_items() -> Vec<Item> {
        todo!()
    }
}