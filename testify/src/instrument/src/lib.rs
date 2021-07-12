use std::fs;
use std::io::Write;
use syn::{Item, ItemFn, Stmt, Expr, Block, Path, ExprIf, File, ItemStruct, ItemImpl, Ident, ItemUse};
use quote::ToTokens;
use syn::visit_mut::{VisitMut, visit_expr_if_mut, visit_item_fn_mut, visit_file_mut};


pub fn instrument(file: String) {
    let content = fs::read_to_string(file)
        .expect("Could not read the Rust source file");
    let mut ast = syn::parse_file(&content)
        .expect("Could not parse the contents of the Rust source file with syn");

    fs::write("ast.txt", format!("{:#?}", ast));

    let mut visitor = Visitor::new("trace.txt".to_string());
    visitor.visit_file_mut(&mut ast);
    /*let items = &ast.items;

    // Gets top level free-standing functions
    let mut top_fns: Vec<ItemFn> = items.iter().filter_map(|i| {
        if let Item::Fn(item_fn) = i {
            Some(item_fn.clone())
        } else {
            None
        }
    }).collect();

    top_fns.iter_mut().for_each(|f| instrument_fn(f));*/

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
}

impl Visitor {
    fn new(trace_file: String) -> Visitor {
        Visitor {
            branch_id: 0,
            trace_file,
        }
    }

    fn instrument_branch(&mut self, branch: &mut Block) {
        println!("Instrumenting branch: {:?}", branch);
        let branch_id = self.branch_id;
        let trace_stmt: Stmt = syn::parse_quote! {
            TestifyMonitor::trace_branch(#branch_id);
        };
        self.branch_id += 1;
        let stmts = &mut branch.stmts;
        stmts.insert(0, trace_stmt);
    }

    fn instrument_fn(&mut self, block: &mut Block, ident: &Ident) {
        let name = ident.to_string();
        let trace_stmt = syn::parse_quote! {
            TestifyMonitor::trace_fn(String::from(#name));
        };

        let stmts = &mut block.stmts;
        stmts.insert(0, trace_stmt);
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

                fn trace_branch(id: u64) {
                    TestifyMonitor::write(format!("b[{}]", id));
                }

                fn trace_fn(name: String) {
                    TestifyMonitor::write(format!(">[{}]", name));
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
        VisitMut::visit_block_mut(self, &mut i.then_branch);
        self.instrument_branch(&mut i.then_branch);

        if let Some((_, branch)) = &mut i.else_branch {
            VisitMut::visit_expr_mut(self, branch.as_mut());
            if let Expr::Block(expr_block) = branch.as_mut() {
                let mut else_branch = &mut expr_block.block;
                self.instrument_branch(else_branch);
            }
        }
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

