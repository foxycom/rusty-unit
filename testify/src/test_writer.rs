use std::fs;
use quote::ToTokens;
use crate::chromosome::TestCase;
use syn::visit_mut::{VisitMut, visit_item_mut};
use syn::Item;
use std::io::Write;

pub fn write(test_case: &TestCase) {
    let target = test_case.target();
    let path = target.instrumented_file();

    let content = fs::read_to_string(path)
        .expect("Could not read the Rust source file");
    let mut ast = syn::parse_file(&content)
        .expect("Could not parse the contents of the Rust source file with syn");

    let mut visitor = Visitor::new(&test_case);
    visitor.visit_file_mut(&mut ast);

    let tokens = ast.to_token_stream();
    let src = tokens.to_string();

    let mut file = fs::File::create(&path).expect("Could not create output source file");
    file.write_all(&src.as_bytes());
}

struct Visitor<'a> {
    test_case: &'a TestCase
}

impl<'a> Visitor<'a> {
    pub fn new(test_case: &'a TestCase) -> Self {
        Visitor { test_case }
    }
}

impl<'a> VisitMut for Visitor<'a> {
    fn visit_item_mut(&mut self, i: &mut Item) {
        if let Item::Mod(item_mod) = i {
            let ident = &item_mod.ident;
            if ident.to_string() == "tests" {
                if let Some((_, items)) = &mut item_mod.content {

                    items.insert(0, self.test_case.to_syn());
                } else {
                    todo!()
                }
                return;
            }
        }

        match i {
            Item::Const(_binding_0) => {
                VisitMut::visit_item_const_mut(self, _binding_0);
            }
            Item::Enum(_binding_0) => {
                VisitMut::visit_item_enum_mut(self, _binding_0);
            }
            Item::ExternCrate(_binding_0) => {
                VisitMut::visit_item_extern_crate_mut(self, _binding_0);
            }
            Item::Fn(_binding_0) => {
                VisitMut::visit_item_fn_mut(self, _binding_0);
            }
            Item::ForeignMod(_binding_0) => {
                VisitMut::visit_item_foreign_mod_mut(self, _binding_0);
            }
            Item::Impl(_binding_0) => {
                VisitMut::visit_item_impl_mut(self, _binding_0);
            }
            Item::Macro(_binding_0) => {
                VisitMut::visit_item_macro_mut(self, _binding_0);
            }
            Item::Macro2(_binding_0) => {
                VisitMut::visit_item_macro2_mut(self, _binding_0);
            }
            Item::Mod(_binding_0) => {
                VisitMut::visit_item_mod_mut(self, _binding_0);
            }
            Item::Static(_binding_0) => {
                VisitMut::visit_item_static_mut(self, _binding_0);
            }
            Item::Struct(_binding_0) => {
                VisitMut::visit_item_struct_mut(self, _binding_0);
            }
            Item::Trait(_binding_0) => {
                VisitMut::visit_item_trait_mut(self, _binding_0);
            }
            Item::TraitAlias(_binding_0) => {
                VisitMut::visit_item_trait_alias_mut(self, _binding_0);
            }
            Item::Type(_binding_0) => {
                VisitMut::visit_item_type_mut(self, _binding_0);
            }
            Item::Union(_binding_0) => {
                VisitMut::visit_item_union_mut(self, _binding_0);
            }
            Item::Use(_binding_0) => {
                VisitMut::visit_item_use_mut(self, _binding_0);
            }
            _ => unreachable!(),
        }
    }
}