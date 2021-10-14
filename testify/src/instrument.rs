use crate::chromosome::{
    Callable, ConstructorItem, FunctionItem, MethodItem, StaticFnItem, StructType,
};
use crate::source::{Branch, BranchBuilder, BranchType, SourceFile};
use crate::util;
use proc_macro2::Span;
use quote::ToTokens;
use std::fs;
use std::io::Write;
use syn::token::Else;
use syn::visit_mut::VisitMut;
use syn::{
    BinOp, Block, Expr, ExprIf, File, ImplItemMethod, Item, ItemEnum, ItemExternCrate, ItemFn,
    ItemImpl, ItemMacro, ItemStruct, ItemUse, Stmt, Type,
};

pub const K: u8 = 1;
pub const ROOT_BRANCH: &'static str = "root[{}, {}]";
pub const BRANCH: &'static str = "branch[{}, {}, {}]";

#[derive(Default, Debug, Clone)]
pub struct Instrumenter {
    branch_id: u64,
    branches: Vec<Branch>,
    structs: Vec<StructType>,
    callables: Vec<Callable>,
    condition: bool,
    current_fn: Option<Item>,
}

impl Instrumenter {
    const TRACE_FILE: &'static str = "trace.txt";

    pub fn new() -> Instrumenter {
        Instrumenter {
            branch_id: 0,
            branches: Vec::new(),
            structs: Vec::new(),
            callables: Vec::new(),
            condition: false,
            current_fn: Default::default(),
        }
    }

    pub fn instrument(&mut self, source_file: &SourceFile) {
        let content = fs::read_to_string(source_file.file_path()).expect("Could not read the Rust source file");
        let mut ast = syn::parse_file(&content)
            .expect("Could not parse the contents of the Rust source file with syn");

        self.visit_file_mut(&mut ast);

        let tokens = ast.to_token_stream();
        let instrumented_src_code = tokens.to_string();

        let mut file =
            fs::File::create(source_file.file_path()).expect("Could not create output source file");
        file.write_all(&instrumented_src_code.as_bytes()).unwrap();
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
        self.branch_id += 1;

        BranchBuilder::default()
            .id(self.branch_id)
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
                        LOGGER.lock().unwrap().trace_branch(#true_branch_id, #false_branch_id, (#left - #right) as f64);
                        LOGGER.with(|l| l.borrow().trace_branch(#true_branch_id, #false_branch_id, (#left - #right) as f64));
                    };
                    // left <= right
                    false_trace = syn::parse_quote! {
                        LOGGER.lock().unwrap().trace_branch(#false_branch_id, #true_branch_id, (#right - #left + #K) as f64);
                        LOGGER.with(|l| l.borrow().trace_branch(#false_branch_id, #true_branch_id, (#right - #left + #K) as f64));
                    };
                }
                BinOp::Ge(_) => {
                    // left >= right
                    true_trace = syn::parse_quote! {
                        LOGGER.with(|l| l.borrow().trace_branch(#true_branch_id, #false_branch_id, (#left - #right + #K) as f64));
                    };
                    // left < right
                    false_trace = syn::parse_quote! {
                        LOGGER.with(|l| l.borrow().trace_branch(#false_branch_id, #true_branch_id, (#right - #left) as f64));
                    };
                }
                BinOp::Lt(_) => {
                    // left < right
                    true_trace = syn::parse_quote! {
                        LOGGER.with(|l| l.borrow().trace_branch(#true_branch_id, #false_branch_id, (#right - #left) as f64));
                    };
                    // left >= right
                    false_trace = syn::parse_quote! {
                        LOGGER.with(|l| l.borrow().trace_branch(#false_branch_id, #true_branch_id, (#left - #right + #K) as f64));
                    };
                }
                BinOp::Le(_) => {
                    // left <= right
                    true_trace = syn::parse_quote! {
                        LOGGER.with(|l| l.borrow().trace_branch(#true_branch_id, #false_branch_id, (#right - #left + #K) as f64));
                    };
                    // left > right
                    false_trace = syn::parse_quote! {
                        LOGGER.with(|l| l.borrow().trace_branch(#false_branch_id, #true_branch_id, (#left - #right) as f64));
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
                        LOGGER.with(|l| l.borrow().trace_branch(#true_branch_id, #false_branch_id, 1.0));
                    };
                    false_trace = syn::parse_quote! {
                        LOGGER.with(|l| l.borrow().trace_branch(#false_branch_id, #true_branch_id, ((#left - #right) as f64).abs()));
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
            LOGGER.with(|l| l.borrow().trace_fn(#name, #branch_id));
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
            LOGGER.with(|l| l.borrow().trace_fn(#name, #branch_id));
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
        let io_write_use = syn::parse_quote! {
            use std::io::Write;
        };

        let fmt_write_use = syn::parse_quote! {
            use std::fmt::Write as FmtWrite;
        };

        vec![io_write_use, fmt_write_use]
    }

    fn macros(&self) -> Vec<ItemMacro> {
        let test_id_macro: ItemMacro = syn::parse_quote! {
            thread_local! {
                pub static TEST_ID: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
                static LOGGER: std::cell::RefCell<TestifyMonitor> = std::cell::RefCell::new(TestifyMonitor::new());
            }
        };

        vec![test_id_macro]
    }

    fn message_enum(&self) -> ItemEnum {
        let testify_message: ItemEnum = syn::parse_quote! {
            enum TestifyMessage {
                Stop,
                Line(String)
            }
        };
        testify_message
    }

    fn monitor_struct(&mut self) -> (ItemStruct, ItemImpl) {
        let trace_file = Instrumenter::TRACE_FILE;
        let monitor: ItemStruct = syn::parse_quote! {
            struct TestifyMonitor {
                sender: Option<std::sync::mpsc::Sender<TestifyMessage>>,
                thread: Option<std::thread::JoinHandle<()>>
            }
        };

        let monitor_impl = syn::parse_quote! {
            impl TestifyMonitor {
                const TRACE_FILE: &'static str = #trace_file;

                fn new() -> Self {
                    TestifyMonitor {
                        sender: None,
                        thread: None
                    }
                }

                fn set_test_id(&mut self, id: u64) {
                    let file = format!("traces/trace_{}.txt", id);
                    let (tx, rx) = std::sync::mpsc::channel();
                    let thread_handle = std::thread::spawn(move || {
                        let trace_file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(file)
                            .unwrap();
                        let mut trace_file = std::io::LineWriter::new(trace_file);
                        while let Ok(msg) = rx.recv() {
                            match msg {
                                TestifyMessage::Stop => {
                                    break;
                                }
                                TestifyMessage::Line(line) => {
                                    let mut s = String::new();
                                    writeln!(s, "{}", line);
                                    trace_file.write_all(s.as_bytes()).unwrap();
                                }
                            }
                        }
                    });

                    self.sender = Some(tx);
                    self.thread = Some(thread_handle);
                }

                fn trace_branch(&self, visited_branch: u64, other_branch: u64, distance: f64) {
                    self.write(format!(#BRANCH, visited_branch, other_branch, distance));
                }

                fn trace_fn(&self, name: &'static str, id: u64) {
                    self.write(format!(#ROOT_BRANCH, name, id));
                }

                fn write(&self, message: String) {
                    if let Some(sender) = &self.sender {
                        sender.send(TestifyMessage::Line(message)).unwrap();
                    }
                }

                fn wait(&mut self) {
                    if let Some(sender) = &self.sender {
                        sender.send(TestifyMessage::Stop).unwrap();
                    }
                    self.thread.take().unwrap().join().unwrap();
                }
            }
        };

        (monitor, monitor_impl)
    }
}

impl VisitMut for Instrumenter {
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
        let testify_message = self.message_enum();

        i.items.insert(0, Item::Impl(monitor_impl));
        i.items.insert(0, Item::Struct(monitor));
        i.items.insert(0, Item::Enum(testify_message));

        for mcro in self.macros() {
            i.items.insert(0, Item::Macro(mcro));
        }

        for u in self.uses() {
            // TODO check if this import already exists
            i.items.insert(0, Item::Use(u));
        }

        for crte in self.extern_crates() {
            i.items.insert(0, Item::ExternCrate(crte));
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
        self.current_fn = Some(Item::Fn(i.clone()));
        for at in &mut i.attrs {
            VisitMut::visit_attribute_mut(self, at);
        }

        VisitMut::visit_visibility_mut(self, &mut i.vis);
        VisitMut::visit_signature_mut(self, &mut i.sig);
        VisitMut::visit_block_mut(self, &mut i.block);

        // TODO don't instrument test functions
        self.instrument_fn(i);
        self.current_fn = None;
    }
}
