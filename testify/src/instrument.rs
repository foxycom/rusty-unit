use crate::chromosome::{
    Callable, ConstructorItem, FunctionItem, MethodItem, StaticFnItem, StructType,
};
use crate::source::{Branch, BranchBuilder, BranchType, FileType, Project, SourceFile};
use crate::util;
use proc_macro2::Span;
use quote::ToTokens;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use syn::token::Else;
use syn::visit_mut::VisitMut;
use syn::*;

pub const K: u8 = 1;
pub const ROOT_BRANCH: &'static str = "root[{}, {}]";
pub const BRANCH: &'static str = "branch[{}, {}, {}]";

#[derive(Default, Debug, Clone)]
struct InstrumentState {
    current_fn: Option<Item>,
    condition: bool,
    branch_id: u64,
    file_type: Option<FileType>
}

impl InstrumentState {
    fn new() -> Self {
        InstrumentState { current_fn: None, condition: false, branch_id: Default::default(), file_type: None }
    }

    fn set_file_type(&mut self, file_type: &FileType) {
        self.file_type = Some(file_type.clone());
    }

    fn increment_branch_id(&mut self) {
        self.branch_id += 1;
    }

    fn set_condition(&mut self, condition: bool) {
        self.condition = condition;
    }

    fn condition(&self) -> bool {
        self.condition
    }

    fn set_current_fn(&mut self, item: Item) {
        self.current_fn = Some(item);
    }

    fn reset_current_fn(&mut self) {
        self.current_fn = None;
    }


    pub fn current_fn(&self) -> Option<&Item> {
        self.current_fn.as_ref()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Instrumenter {
    branches: Vec<Branch>,
    structs: Vec<StructType>,
    callables: Vec<Callable>,
    state: InstrumentState
}

impl Instrumenter {
    const TRACE_FILE: &'static str = "trace.txt";

    pub fn new() -> Instrumenter {
        Instrumenter {
            branches: Vec::new(),
            structs: Vec::new(),
            callables: Vec::new(),
            state: InstrumentState::new()
        }
    }

    pub fn instrument(&mut self, project: &mut Project) {
        for file in project.source_files_mut() {
            let content = fs::read_to_string(file.file_path())
                .expect("Could not read the Rust source file");
            let mut ast = syn::parse_file(&content)
                .expect("Could not parse the contents of the Rust source file with syn");


            self.state.set_file_type(file.file_type());
            self.visit_file_mut(&mut ast);

            file.set_ast(ast);
        }
    }

    fn instrument_if(&mut self, i: &mut ExprIf) {
        let (true_trace, false_trace) = self.instrument_condition(i);

        self.insert_stmt(&mut i.then_branch, true_trace);

        if let Some((_, branch)) = &mut i.else_branch {
            VisitMut::visit_expr_mut(self, branch.as_mut());
            if let Expr::Block(expr_block) = branch.as_mut() {
                let mut else_branch = &mut expr_block.block;
                self.insert_stmt(else_branch, false_trace);
            }
        } else {
            // There was no else branch before, so create an artificial ones
            let else_expr: Else = syn::parse_quote! {else};
            let mut block: Block = syn::parse_quote! {{}};
            self.insert_stmt(&mut block, false_trace);
            let expr = syn::parse_quote! {
                #block
            };
            i.else_branch = Some((else_expr, Box::new(expr)));
        }
    }

    fn insert_stmt(&mut self, block: &mut Block, stmt: Stmt) {
        let stmts = &mut block.stmts;
        stmts.insert(0, stmt);
    }

    fn create_branch(&mut self, branch_type: BranchType, span: Span) -> Branch {
        self.state.increment_branch_id();

        BranchBuilder::default()
            .id(self.state.branch_id)
            //.source_file(source_file)
            .branch_type(branch_type)
            .span(span)
            .build()
            .unwrap()
    }

    fn instrument_condition(&mut self, i: &mut ExprIf) -> (Stmt, Stmt) {
        let span = &i.if_token.span;
        let true_branch = self.create_branch(BranchType::Decision, span.clone());
        let false_branch = self.create_branch(BranchType::Decision, span.clone());

        let true_branch_id = true_branch.id();
        let false_branch_id = false_branch.id();

        let cond = i.cond.as_mut();
        let mut true_trace: Stmt;
        let mut false_trace: Stmt;

        // TODO unary OP

        if let Expr::Binary(expr_binary) = cond {
            let left = expr_binary.left.as_mut();
            let right = expr_binary.right.as_mut();
            match expr_binary.op {
                BinOp::Gt(_) => {
                    // left > right
                    true_trace = syn::parse_quote! {
                        //LOGGER.lock().unwrap().trace_branch(#true_branch_id, #false_branch_id, (#left - #right) as f64);
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#true_branch_id, #false_branch_id, (#left - #right) as f64));
                    };
                    // left <= right
                    false_trace = syn::parse_quote! {
                        //LOGGER.lock().unwrap().trace_branch(#false_branch_id, #true_branch_id, (#right - #left + #K) as f64);
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#false_branch_id, #true_branch_id, (#right - #left + #K) as f64));
                    };
                }
                BinOp::Ge(_) => {
                    // left >= right
                    true_trace = syn::parse_quote! {
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#true_branch_id, #false_branch_id, (#left - #right + #K) as f64));
                    };
                    // left < right
                    false_trace = syn::parse_quote! {
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#false_branch_id, #true_branch_id, (#right - #left) as f64));
                    };
                }
                BinOp::Lt(_) => {
                    // left < right
                    true_trace = syn::parse_quote! {
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#true_branch_id, #false_branch_id, (#right - #left) as f64));
                    };
                    // left >= right
                    false_trace = syn::parse_quote! {
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#false_branch_id, #true_branch_id, (#left - #right + #K) as f64));
                    };
                }
                BinOp::Le(_) => {
                    // left <= right
                    true_trace = syn::parse_quote! {
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#true_branch_id, #false_branch_id, (#right - #left + #K) as f64));
                    };
                    // left > right
                    false_trace = syn::parse_quote! {
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#false_branch_id, #true_branch_id, (#left - #right) as f64));
                    };
                }
                BinOp::And(_) => {
                    // TODO this is useless
                    true_trace = syn::parse_quote! {println!();};
                    false_trace = syn::parse_quote! {println!();};
                }
                BinOp::Eq(_) => {
                    // left == right
                    true_trace = syn::parse_quote! {
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#true_branch_id, #false_branch_id, 1.0));
                    };
                    false_trace = syn::parse_quote! {
                        testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_branch(#false_branch_id, #true_branch_id, ((#left - #right) as f64).abs()));
                    }
                }
                // TODO all other ops
                _ => {
                    unimplemented!();
                }
            }
        } else if let Expr::Unary(expr_unary) = cond {
            unimplemented!()
        } else {
            println!("{}", cond.to_token_stream().to_string());
            unimplemented!()
        }
        self.branches.push(true_branch);
        self.branches.push(false_branch);
        (true_trace, false_trace)
    }

    fn instrument_method(&mut self, item_method: &mut ImplItemMethod) {
        let block = &mut item_method.block;
        let ident = &item_method.sig.ident;
        let branch = self.create_branch(BranchType::Root, ident.span());
        let branch_id = branch.id();

        let name = ident.to_string();

        let trace_stmt = syn::parse_quote! {
            testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_fn(#name, #branch_id));
        };

        let stmts = &mut block.stmts;

        stmts.insert(0, trace_stmt);
        self.branches.push(branch);
    }

    fn instrument_fn(&mut self, item_fn: &mut ItemFn) {
        let block = &mut item_fn.block;
        let ident = &item_fn.sig.ident;
        let branch = self.create_branch(BranchType::Root, ident.span());
        let branch_id = branch.id();

        let name = ident.to_string();

        let trace_stmt = syn::parse_quote! {
            testify_monitor::MONITOR.with(|m| m.borrow_mut().trace_fn(#name, #branch_id));
        };

        let stmts = &mut block.stmts;

        stmts.insert(0, trace_stmt);
        self.branches.push(branch);
    }

    fn extern_crates(&self) -> Vec<ItemExternCrate> {
        /*let lazy_static_crate = syn::parse_quote! {
            #[macro_use]
            extern crate lazy_static;
        };*/

        vec![]
    }

    fn uses(&self) -> Vec<ItemUse> {
        /*let io_write_use = syn::parse_quote! {
            use std::io::Write;
        };

        let fmt_write_use = syn::parse_quote! {
            use std::fmt::Write as FmtWrite;
        };*/

       /* let use_client = syn::parse_quote! {
            use testify_monitor::MONITOR;
        };*/

        vec![]
    }

    fn mods(&self) -> Vec<ItemMod> {
        let mod_testify_monitor = syn::parse_quote! {
            mod testify_monitor;
        };

        vec![mod_testify_monitor]
    }

    fn macros(&self) -> Vec<ItemMacro> {
        let test_id_macro: ItemMacro = syn::parse_quote! {
            thread_local! {
                pub static TEST_ID: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
            }
        };

        vec![test_id_macro]
    }
}

impl VisitMut for Instrumenter {
    fn visit_expr_if_mut(&mut self, i: &mut ExprIf) {
        for it in &mut i.attrs {
            VisitMut::visit_attribute_mut(self, it);
        }

        self.state.set_condition(true);
        VisitMut::visit_expr_mut(self, &mut *i.cond);
        self.state.set_condition(false);

        VisitMut::visit_block_mut(self, &mut i.then_branch);

        self.instrument_if(i);
    }

    fn visit_file_mut(&mut self, i: &mut File) {
        for at in &mut i.attrs {
            VisitMut::visit_attribute_mut(self, at);
        }

        for mcro in self.macros() {
            i.items.insert(0, Item::Macro(mcro));
        }

        for u in self.uses() {
            // TODO check if this import already exists
            i.items.insert(0, Item::Use(u));
        }

        if let FileType::Executable(..) = self.state.file_type.as_ref().unwrap() {
            for m in self.mods() {
                i.items.insert(0, Item::Mod(m));
            }
        }

        if let FileType::Library(..) = self.state.file_type.as_ref().unwrap() {
            for m in self.mods() {
                i.items.insert(0, Item::Mod(m));
            }
        }

        for crte in self.extern_crates() {
            i.items.insert(0, Item::ExternCrate(crte));
        }

        for it in &mut i.items {
            VisitMut::visit_item_mut(self, it);
        }
    }

    fn visit_impl_item_method_mut(&mut self, i: &mut ImplItemMethod) {
        for it in &mut i.attrs {
            VisitMut::visit_attribute_mut(self, it);
        }

        VisitMut::visit_visibility_mut(self, &mut i.vis);
        VisitMut::visit_signature_mut(self, &mut i.sig);
        VisitMut::visit_block_mut(self, &mut i.block);

        self.instrument_method(i);
    }



    fn visit_item_fn_mut(&mut self, i: &mut ItemFn) {
        self.state.set_current_fn(Item::Fn(i.clone()));

        for at in &mut i.attrs {
            VisitMut::visit_attribute_mut(self, at);
        }

        VisitMut::visit_visibility_mut(self, &mut i.vis);
        VisitMut::visit_signature_mut(self, &mut i.sig);
        VisitMut::visit_block_mut(self, &mut i.block);

        self.instrument_fn(i);

        self.state.reset_current_fn();
    }

    fn visit_item_mod_mut(&mut self, i: &mut ItemMod) {

        for it in &i.attrs {
            if is_test_cfg_attribute(it) {
                // Ignore a mod if it has #[cfg(test)] attribute
                return;
            }
        }

        for it in &mut i.attrs {
            VisitMut::visit_attribute_mut(self, it);
        }
        VisitMut::visit_visibility_mut(self, &mut i.vis);
        VisitMut::visit_ident_mut(self, &mut i.ident);
        if let Some(it) = &mut i.content {
            for it in &mut (it).1 {
                VisitMut::visit_item_mut(self, it);
            }
        };
    }
}

fn is_test_cfg_attribute(attribute: &Attribute) -> bool {
    let cfg_attribute: Attribute = syn::parse_quote! {#[cfg(test)]};
    &cfg_attribute == attribute
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cfg_attribute() {
        let cfg_test: Attribute = syn::parse_quote!{
            #[cfg(test)]
        };

        let is_cfg_test = is_test_cfg_attribute(&cfg_test);

        assert!(is_cfg_test);
    }
}
