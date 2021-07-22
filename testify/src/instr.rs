pub mod util {
    use std::path::Path;

    pub fn instrumented_path(original_path: &str) -> String {
        let path = Path::new(original_path);
        let dir = path.parent().expect("No dir available");
        let file_name = path.file_stem().expect("No file name available");

        let new_file_name = format!("{}_instrumented.rs", file_name.to_str().unwrap());
        let new_path = dir.join(Path::new(&new_file_name));

        let str_path = new_path.to_str().unwrap().to_owned();
        str_path
    }


    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_instrumented_path() {
            assert_eq!("/abc/some_file_instrumented.rs", instrumented_path("/abc/some_file.rs"));
        }
    }
}

pub mod data {
    use super::*;
    use serde::{Serialize, Deserialize};

    use syn::ItemFn;
    use crate::instr;
    use std::path;
    use crate::chromosome::TestCase;


    #[derive(Debug, Clone, Hash, Eq, PartialEq, Builder)]
    pub struct Branch {
        id: u64,
        original_file: String,
        target_fn: ItemFn,
        branch_type: BranchType,
    }

    impl Branch {
        pub fn original_file(&self) -> &str {
            &self.original_file
        }
        pub fn instrumented_file(&self) -> String {
            util::instrumented_path(&self.original_file)
        }

        pub fn instrumented_mod(&self) -> String {
            path::Path::new(&self.instrumented_file())
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string()
        }

        // TODO return fitness as enum with ZERO value
        pub fn fitness(&self, test_case: &TestCase) -> f64 {
            test_case.results().get(&self.id).unwrap_or(&f64::MAX).to_owned()
        }

        pub fn target_fn(&self) -> &ItemFn {
            &self.target_fn
        }

        pub fn id(&self) -> &u64 {
            &self.id
        }
    }

    impl Default for Branch {
        fn default() -> Self {
            Branch {
                id: Default::default(),
                original_file: Default::default(),
                target_fn: syn::parse_quote! {fn blank() {}},
                branch_type: BranchType::Root,
            }
        }
    }

    impl Branch {
        pub fn new(id: u64, original_file: &str, target_fn: ItemFn, branch_type: BranchType) -> Self {
            Branch {
                id,
                target_fn,
                branch_type,
                original_file: original_file.to_string(),
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    pub enum BranchType {
        Root,
        Decision,
    }
}

pub mod instr {
    use std::fs;
    use std::io::Write;
    use syn::{Item, ItemFn, Stmt, Expr, Block, Path, ExprIf, File, ItemStruct,
              ItemImpl, Ident, ItemUse, ExprBinary, BinOp};
    use quote::ToTokens;
    use syn::visit_mut::{VisitMut, visit_expr_if_mut, visit_item_fn_mut,
                         visit_file_mut, visit_expr_binary_mut};
    use super::util;
    use super::data::{BranchType, Branch, BranchBuilder};
    use std::borrow::Cow;

    pub const ROOT_BRANCH: &'static str = "root[{}, {}]";
    pub const BRANCH: &'static str = "branch[{}, {}, {}]";
    pub const K: u8 = 1;

    #[derive(Default)]
    pub struct Instrumenter<'a> {
        branch_id: u64,
        branches: Vec<Branch>,
        condition: bool,
        file: Cow<'a, str>,
        current_fn: Option<ItemFn>,
    }

    impl<'a> Instrumenter<'a> {
        const TRACE_FILE: &'static str = "trace.txt";

        pub fn new() -> Instrumenter<'a> {
            Instrumenter {
                branch_id: 0,
                branches: Vec::new(),
                condition: false,
                file: Default::default(),
                current_fn: Default::default(),
            }
        }

        pub fn instrument(&mut self, source_file: &'a str) -> Vec<Branch> {
            self.file = Cow::Borrowed(source_file);
            let content = fs::read_to_string(source_file)
                .expect("Could not read the Rust source file");
            let mut ast = syn::parse_file(&content)
                .expect("Could not parse the contents of the Rust source file with syn");

            fs::write("ast.txt", format!("{:#?}", ast));

            self.visit_file_mut(&mut ast);

            let tokens = ast.to_token_stream();
            let src_code = tokens.to_string();
            self.src_to_file(&src_code, util::instrumented_path(source_file));

            self.branches.clone()
        }

        fn src_to_file(&self, src: &str, path: String) {
            let mut file = fs::File::create(path).expect("Could not create output source file");
            file.write_all(&src.as_bytes());
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
            }
        }

        fn insert_stmt(&mut self, block: &mut Block, stmt: Stmt) {
            let stmts = &mut block.stmts;
            stmts.insert(0, stmt);
        }

        fn create_branch(&mut self, branch_type: BranchType) -> Branch {
            self.branch_id += 1;

            BranchBuilder::default()
                .id(self.branch_id)
                .original_file(self.file.to_string())
                .target_fn(self.current_fn.as_ref().unwrap().clone())
                .branch_type(branch_type)
                .build()
                .unwrap()
        }

        fn instrument_condition(&mut self, i: &mut ExprIf) -> (Stmt, Stmt) {
            let true_branch = self.create_branch(BranchType::Decision);
            let false_branch = self.create_branch(BranchType::Decision);

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
                        TestifyMonitor::trace_branch(#true_branch_id, #false_branch_id, (#left - #right) as f64);
                    };
                        // left <= right
                        false_trace = syn::parse_quote! {
                        TestifyMonitor::trace_branch(#false_branch_id, #true_branch_id, (#right - #left + #K) as f64);
                    };
                    }
                    BinOp::Ge(_) => {
                        // left >= right
                        true_trace = syn::parse_quote! {
                        TestifyMonitor::trace_branch(#true_branch_id, #false_branch_id, (#left - #right + #K) as f64);
                    };
                        // left < right
                        false_trace = syn::parse_quote! {
                        TestifyMonitor::trace_branch(#false_branch_id, #true_branch_id, (#right - #left) as f64);
                    };
                    }
                    BinOp::Lt(_) => {
                        // left < right
                        true_trace = syn::parse_quote! {
                        TestifyMonitor::trace_branch(#true_branch_id, #false_branch_id, (#right - #left) as f64);
                    };
                        // left >= right
                        false_trace = syn::parse_quote! {
                        TestifyMonitor::trace_branch(#false_branch_id, #true_branch_id, (#left - #right + #K) as f64);
                    };
                    }
                    BinOp::Le(_) => {
                        // left <= right
                        true_trace = syn::parse_quote! {
                        TestifyMonitor::trace_branch(#true_branch_id, #false_branch_id, (#right - #left + #K) as f64);
                    };
                        // left > right
                        false_trace = syn::parse_quote! {
                        TestifyMonitor::trace_branch(#false_branch_id, #true_branch_id, (#left - #right) as f64);
                    };
                    }
                    // TODO all other ops
                    _ => {
                        unimplemented!();
                    }
                }
            } else {
                unimplemented!();
            }
            self.branches.push(true_branch);
            self.branches.push(false_branch);
            (true_trace, false_trace)
        }

        fn instrument_fn(&mut self, block: &mut Block, ident: &Ident) {
            let branch = self.create_branch(BranchType::Root);
            let branch_id = branch.id();

            let name = ident.to_string();
            let trace_stmt = syn::parse_quote! {
                TestifyMonitor::trace_fn(String::from(#name), #branch_id);
            };

            let stmts = &mut block.stmts;
            stmts.insert(0, trace_stmt);
            self.branches.push(branch);
        }

        fn uses(&mut self) -> Vec<ItemUse> {
            let write_import: ItemUse = syn::parse_quote! {
                use std::io::Write;
            };

            /*let line_writer_import: ItemUse = syn::parse_quote! {
                use std::io::LineWriter;
            };*/

            vec![write_import]
        }

        fn monitor_struct(&mut self) -> (ItemStruct, ItemImpl) {
            let trace_file = Instrumenter::TRACE_FILE;
            let monitor: ItemStruct = syn::parse_quote! {
                struct TestifyMonitor {}
            };

            let monitor_impl = syn::parse_quote! {
                impl TestifyMonitor {
                    const TRACE_FILE: &'static str = #trace_file;

                    fn trace_branch(visited_branch: u64, other_branch: u64, distance: f64) {
                        TestifyMonitor::write(format!(#BRANCH, visited_branch, other_branch, distance));
                    }

                    fn trace_fn(name: String, id: u64) {
                        println!("HELLLO");
                        TestifyMonitor::write(format!(#ROOT_BRANCH, name, id));
                    }

                    fn write(output: String) {
                        let trace_file = std::fs::OpenOptions::new()
                                                .write(true)
                                                .append(true)
                                                .create(true)

                                                .open(TestifyMonitor::TRACE_FILE)
                                                .unwrap();
                        let mut trace_file = std::io::LineWriter::new(trace_file);
                        trace_file.write_all(&output.as_bytes()).unwrap();
                        trace_file.write_all(b"\n").unwrap();
                    }
                }
            };

            (monitor, monitor_impl)
        }
    }


    impl<'a> VisitMut for Instrumenter<'a> {
        // TODO use also other visitors
        fn visit_expr_if_mut(&mut self, i: &mut ExprIf) {
            for it in &mut i.attrs {
                VisitMut::visit_attribute_mut(self, it);
            }

            self.condition = true;
            VisitMut::visit_expr_mut(self, &mut *i.cond);
            self.condition = true;

            VisitMut::visit_block_mut(self, &mut i.then_branch);

            self.instrument_if(i);
        }


        fn visit_file_mut(&mut self, i: &mut File) {
            for at in &mut i.attrs {
                VisitMut::visit_attribute_mut(self, at);
            }

            for it in &mut i.items {
                VisitMut::visit_item_mut(self, it);
            }


            // TODO skip imports first
            let (monitor, monitor_impl) = self.monitor_struct();
            i.items.insert(0, Item::Impl(monitor_impl));
            i.items.insert(0, Item::Struct(monitor));

            let uses = self.uses();
            for u in uses {
                // TODO check if this import already exists
                i.items.insert(0, Item::Use(u));
            }
        }

        fn visit_item_fn_mut(&mut self, i: &mut ItemFn) {
            self.current_fn = Some(i.clone());
            for at in &mut i.attrs {
                VisitMut::visit_attribute_mut(self, at);
            }

            VisitMut::visit_visibility_mut(self, &mut i.vis);
            VisitMut::visit_signature_mut(self, &mut i.sig);
            VisitMut::visit_block_mut(self, &mut i.block);

            // TODO don't instrument test functions
            self.instrument_fn(&mut i.block, &i.sig.ident);
            self.current_fn = None;
        }
    }
}
