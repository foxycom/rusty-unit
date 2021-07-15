use std::fs;
use syn::visit_mut::{VisitMut, visit_item_fn_mut};
use syn::ItemFn;
use crate::data::Target;

pub fn analyze_src(path: &str) -> Vec<Target> {
    let content = fs::read_to_string(path)
        .expect("Could not read the Rust source file");
    let mut ast = syn::parse_file(&content)
        .expect("Could not parse the contents of the Rust source file with syn");

    let mut visitor = Visitor::new(path);
    visitor.visit_file_mut(&mut ast);
    visitor.targets
}


pub struct Visitor {
    targets: Vec<Target>,
    file_path: String
}

impl Visitor {
    pub fn new(file_path: &str) -> Self {
        Visitor {
            targets: Vec::new(),
            file_path: file_path.to_string()
        }
    }
}

impl VisitMut for Visitor {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        for it in &mut node.attrs {
            VisitMut::visit_attribute_mut(self, it);
        }
        VisitMut::visit_visibility_mut(self, &mut node.vis);
        VisitMut::visit_signature_mut(self, &mut node.sig);
        VisitMut::visit_block_mut(self, &mut *node.block);

        let target = Target::new(&self.file_path, node.clone());
        self.targets.push(target);
    }
}