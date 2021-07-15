use syn::ItemFn;
use instrument::util;

#[derive(Debug, Clone)]
pub struct Target {
    original_file: String,
    instrumented_file: String,
    target_fn: ItemFn,
}

impl Target {
    pub fn original_file(&self) -> &str {
        &self.original_file
    }
    pub fn instrumented_file(&self) -> &str {
        &self.instrumented_file
    }
    pub fn target_fn(&self) -> &ItemFn {
        &self.target_fn
    }
}

impl Target {
    pub fn new(original_file: &str, target_fn: ItemFn) -> Self {
        let instrumented_file = util::instrumented_path(&original_file);
        Target {
            original_file: original_file.to_string(),
            instrumented_file,
            target_fn,
        }
    }
}