mod branch;

use std::fs;
use std::io::Write;
use syn::{Item, ItemFn, Stmt, Expr, Block, Path, ExprIf, File, ItemStruct,
          ItemImpl, Ident, ItemUse, ExprBinary, BinOp};
use quote::ToTokens;
use syn::visit_mut::{VisitMut, visit_expr_if_mut, visit_item_fn_mut,
                     visit_file_mut, visit_expr_binary_mut};
use crate::branch::{Branch};

const ROOT_BRANCH: &'static str = "root({}, {})";
const BRANCH: &'static str = "branch({}, {}, [{}])";
const K: u64 = 1;

pub fn instrument(file: String) {
    let content = fs::read_to_string(file)
        .expect("Could not read the Rust source file");
    let mut ast = syn::parse_file(&content)
        .expect("Could not parse the contents of the Rust source file with syn");

    fs::write("ast.txt", format!("{:#?}", ast));

    let mut visitor = Visitor::new("trace.txt".to_string());
    visitor.visit_file_mut(&mut ast);
    visitor.finalize();

    let tokens = ast.to_token_stream();
    let src_code = tokens.to_string();
    src_to_file(&src_code, "src/examples/additions/src/instrumented-main.rs".into());
}

fn src_to_file(src: &str, path: String) {
    let mut file = fs::File::create(path).expect("Could not create output source file");
    file.write_all(&src.as_bytes());
}


struct Visitor {
    branch_id: u64,
    trace_file: String,
    branches: Vec<Branch>,
    condition: bool
}

impl Visitor {
    fn new(trace_file: String) -> Visitor {
        Visitor {
            branch_id: 0,
            trace_file,
            branches: Vec::new(),
            condition: false
        }
    }

    fn finalize(&self) {
        let serialized_branches = serde_json::to_string(&self.branches).unwrap();
        fs::write("branches.json", serialized_branches).unwrap();
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

    fn instrument_condition(&mut self, i: &mut ExprIf) -> (Stmt, Stmt) {
        self.branch_id += 1;
        let true_branch_id = self.branch_id;
        let true_branch = Branch::Decision(true_branch_id);

        self.branch_id += 1;
        let false_branch_id = self.branch_id;
        let false_branch = Branch::Decision(false_branch_id);


        self.branches.push(true_branch);
        self.branches.push(false_branch);

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

        (true_trace, false_trace)
    }

    fn instrument_fn(&mut self, block: &mut Block, ident: &Ident) {
        self.branch_id += 1;
        let branch_id = self.branch_id;
        let branch = Branch::Root(branch_id);
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
        let trace_file = &self.trace_file;
        let monitor: ItemStruct = syn::parse_quote! {
            struct TestifyMonitor {

            }
        };

        let trace_file = &self.trace_file;
        let monitor_impl = syn::parse_quote! {
            impl TestifyMonitor {
                const TRACE_FILE: &'static str = #trace_file;

                fn trace_branch(visited_branch: u64, other_branch: u64, distance: f64) {
                    TestifyMonitor::write(format!(#BRANCH, visited_branch, other_branch, distance));
                }

                fn trace_fn(name: String, id: u64) {
                    TestifyMonitor::write(format!(#ROOT_BRANCH, name, id));
                }

                fn write(output: String) {
                    let mut trace_file = std::fs::OpenOptions::new()
                                            .write(true)
                                            .append(true)
                                            .open(TestifyMonitor::TRACE_FILE)
                                            .unwrap();
                    let mut trace_file = std::io::LineWriter::new(trace_file);
                    trace_file.write_all(&output.as_bytes());
                    trace_file.write_all(b"\n");
                }
            }
        };

        (monitor, monitor_impl)
    }
}


impl VisitMut for Visitor {
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
        for at in &mut i.attrs {
            VisitMut::visit_attribute_mut(self, at);
        }

        VisitMut::visit_visibility_mut(self, &mut i.vis);
        VisitMut::visit_signature_mut(self, &mut i.sig);
        VisitMut::visit_block_mut(self, &mut i.block);

        self.instrument_fn(&mut i.block, &i.sig.ident);
    }

}

