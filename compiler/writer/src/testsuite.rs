use serde::{Deserialize, Serialize};
use syn::Item;

#[derive(Serialize, Deserialize, Debug)]
pub struct TestSuite {
    pub tests: Vec<String>
}

impl TestSuite {
    pub fn to_items(&self) -> Vec<Item> {
        self.tests.iter().map(|s| {
            syn::parse_str::<Item>(s).unwrap()
        }).collect::<Vec<_>>()
    }
}