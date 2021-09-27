use proc_macro2::Span;
use quote::ToTokens;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{fs, io};
use syn::token::Else;
use syn::visit_mut::VisitMut;
use syn::{
    BinOp, Block, Expr, ExprIf, File, FnArg, ImplItemMethod, Item, ItemEnum, ItemExternCrate,
    ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemUse, Stmt, Type,
};

use crate::chromosome::{Chromosome, FnInvStmt, MethodItem, Struct, TestCase, ConstructorItem, StaticFnItem, Callable, T, FunctionItem, PrimitiveItem};
use crate::parser::TraceParser;
use crate::util;
use crate::util::type_name;

pub const ROOT_BRANCH: &'static str = "root[{}, {}]";
pub const BRANCH: &'static str = "branch[{}, {}, {}]";
pub const K: u8 = 1;

fn src_to_file(src: &str, path: &str) {
    let mut file = fs::File::create(path).expect("Could not create output source file");
    file.write_all(&src.as_bytes()).unwrap();
}

#[cfg_attr(test, create)]
#[derive(Debug, Clone)]
pub struct SourceFile {
    file_path: String,
    writer: TestWriter,
    runner: TestRunner,
    instrumenter: Instrumenter,
}

impl SourceFile {
    pub fn new(src_path: &str) -> SourceFile {
        SourceFile {
            file_path: src_path.to_owned(),
            writer: TestWriter::new(src_path),
            runner: TestRunner::new(),
            instrumenter: Instrumenter::new(),
        }
    }

    /// Writes the tests as source code into the file.
    pub fn add_tests(&mut self, tests: &[TestCase], instrumented: bool) {
        if instrumented {
            if let Some(ast) = &self.instrumenter.instrumented_ast {
                let mutated_ast = self.writer.add_tests(tests, ast);
                let tokens = mutated_ast.to_token_stream();
                let src_code = tokens.to_string();
                src_to_file(&src_code, &self.file_path);
            }
        } else {
            if let Some(ast) = &self.instrumenter.original_ast {
                let mutated_ast = self.writer.add_tests(tests, ast);
                let tokens = mutated_ast.to_token_stream();
                let src_code = fmt_string(&tokens.to_string()).unwrap();
                src_to_file(&src_code, &self.file_path);
            }
        }
    }

    /// Runs the tests provided they have been added to the source file before.
    pub fn run_tests(&mut self, tests: &mut [TestCase]) {
        self.runner.run().unwrap();
        for test in tests {
            // TODO magic path
            let file = format!("/Users/tim/Documents/master-thesis/testify/src/examples/additions/traces/trace_{}.txt", test.id());
            test.set_results(TraceParser::parse(&file).unwrap());
            match fs::remove_file(&file) {
                Err(err) => {
                    panic!("There was no trace file: {}", err);
                }
                _ => {}
            }
        }
    }

    pub fn instrument(&mut self) {
        self.instrumenter.instrument(&self.file_path);
    }

    pub fn structs(&self) -> &Vec<Struct> {
        self.instrumenter.structs.as_ref()
    }

    pub fn generators(&self, ty: &T) -> Vec<Callable> {
        self.instrumenter.callables
            .iter()
            .filter(|&c| {
                let return_type = c.return_type();
                match return_type {
                    None => false,
                    Some(return_ty) => return_ty == ty
                }
            })
            .cloned()
            .collect()
    }

    pub fn callables(&self) -> &Vec<Callable> {
        &self.instrumenter.callables
    }

    pub fn branches(&self) -> &Vec<Branch> {
        &self.instrumenter.branches
    }

    /// Removes the generated tests from the module.
    pub fn clear_tests(&mut self) {
        if let Some(instrumented_ast) = &self.instrumenter.instrumented_ast {
            let tokens = instrumented_ast.to_token_stream();
            let instrumented_src_code = tokens.to_string();
            src_to_file(&instrumented_src_code, &self.file_path);
        }
    }

    pub fn restore(&self) {
        if let Some(original_ast) = &self.instrumenter.original_ast {
            let tokens = original_ast.to_token_stream();
            let original_src_code = tokens.to_string();
            let formatted_src_code = fmt_string(&original_src_code).unwrap();
            src_to_file(&formatted_src_code, &self.file_path);
        }
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }
}

impl PartialEq for SourceFile {
    fn eq(&self, other: &Self) -> bool {
        self.file_path == other.file_path
    }
}

impl Eq for SourceFile {}

impl Hash for SourceFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_path.hash(state);
    }
}

fn fmt_path() -> io::Result<PathBuf> {
    match which::which("rustfmt") {
        Ok(p) => Ok(p),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
    }
}

fn fmt_string(source: &str) -> io::Result<String> {
    let rustfmt = fmt_path()?;
    let mut cmd = Command::new(&*rustfmt);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    let source = source.to_owned();
    let stdin_handle = std::thread::spawn(move || {
        let _ = child_stdin.write_all(source.as_bytes());
        source
    });

    let mut output = vec![];
    io::copy(&mut child_stdout, &mut output)?;
    let status = child.wait()?;
    let source = stdin_handle.join().unwrap();

    match String::from_utf8(output) {
        Ok(source) => match status.code() {
            Some(0) => Ok(source),
            Some(2) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Rustfmt parsing errors".to_string(),
            )),
            Some(3) => Ok(source),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal rustfmt error".to_string(),
            )),
        },
        Err(_) => Ok(source),
    }
}

#[derive(Debug, Clone)]
struct TestClearer {
    src_path: String,
}

impl VisitMut for TestClearer {
    fn visit_item_mod_mut(&mut self, i: &mut ItemMod) {
        // TODO default implementation?

        let ident = &i.ident;
        if ident == "testify_tests" {
            if let Some((_, items)) = &mut i.content {
                items.clear();
            }
        }
    }
}

#[derive(Debug, Clone)]
struct TestWriter {
    use_all_star: Item,
    test_cases: Option<Vec<TestCase>>,
    src_path: String,
}

impl TestWriter {
    const TESTS_MODULE: &'static str = "testify_tests";
    pub fn new(src_path: &str) -> Self {
        TestWriter {
            use_all_star: syn::parse_quote! { use super::*; },
            test_cases: None,
            src_path: src_path.to_string(),
        }
    }

    pub fn add_tests(&mut self, test_cases: &[TestCase], ast: &File) -> File {
        self.test_cases = Some(test_cases.to_vec());
        let mut ast = ast.clone();
        self.visit_file_mut(&mut ast);
        ast
    }

    fn contains_use_super_star(&self, items: &[Item]) -> bool {
        items.iter().any(|i| *i == self.use_all_star)
    }
}

impl VisitMut for TestWriter {
    fn visit_item_mut(&mut self, i: &mut Item) {
        if let Item::Mod(item_mod) = i {
            let mod_name = &item_mod.ident;
            if mod_name.to_string() == TestWriter::TESTS_MODULE {
                if let Some((_, items)) = &mut item_mod.content {
                    if !self.contains_use_super_star(items) {
                        items.insert(0, self.use_all_star.clone());
                    }

                    if let Some(ref mut tests) = self.test_cases {
                        let mut code: Vec<Item> =
                            tests.iter_mut().map(|t| t.to_syn(true)).collect();
                        items.append(&mut code);
                    }
                } else {
                    todo!()
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct TestRunner {}

impl TestRunner {
    pub fn new() -> Self {
        TestRunner {}
    }

    pub fn run(&self) -> io::Result<()> {
        let cargo = self.cargo_path()?;
        let mut cmd = Command::new(&*cargo);
        let log_file = fs::File::create("out.log")?;
        let err_file = fs::File::create("err.log")?;
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(err_file));

        // TODO extract package and bin files
        cmd.args(&["test", "--package", "additions", "testify_tests"])
            .current_dir("/Users/tim/Documents/master-thesis/testify/src/examples/additions");
        match cmd.status() {
            Ok(_) => {
                //println!("Test {}: OK", test_case.name());
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn cargo_path(&self) -> io::Result<PathBuf> {
        match which::which("cargo") {
            Ok(p) => Ok(p),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
        }
    }
}

#[derive(Clone, Builder)]
pub struct Branch {
    id: u64,
    branch_type: BranchType,
    span: Span,
}

impl Debug for Branch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Branch (id: {}, line: {}:{})",
            self.id,
            self.span.start().line,
            self.span.start().column
        ))
    }
}

impl Hash for Branch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.branch_type.hash(state);
    }
}

impl PartialEq for Branch {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.branch_type == other.branch_type
    }
}

impl Eq for Branch {}

impl Branch {
    pub fn new(id: u64, branch_type: BranchType, span: Span) -> Self {
        Branch {
            id,
            branch_type,
            span,
        }
    }

    // TODO return fitness as enum with ZERO value
    pub fn fitness(&self, test_case: &TestCase) -> f64 {
        test_case
            .results()
            .get(&self.id)
            .unwrap_or(&f64::MAX)
            .to_owned()
    }

    pub fn id(&self) -> &u64 {
        &self.id
    }

    pub fn branch_type(&self) -> &BranchType {
        &self.branch_type
    }
}

impl Default for Branch {
    fn default() -> Self {
        Branch {
            id: Default::default(),
            branch_type: BranchType::Root,
            span: Span::call_site(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BranchType {
    Root,
    Decision,
}

#[derive(Default, Debug, Clone)]
pub struct Instrumenter {
    branch_id: u64,
    branches: Vec<Branch>,
    original_ast: Option<File>,
    instrumented_ast: Option<File>,
    structs: Vec<Struct>,
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
            original_ast: None,
            instrumented_ast: None,
            condition: false,
            current_fn: Default::default(),
        }
    }

    pub fn instrument(&mut self, source_file: &str) {
        let content = fs::read_to_string(source_file).expect("Could not read the Rust source file");
        self.original_ast = Some(
            syn::parse_file(&content)
                .expect("Could not parse the contents of the Rust source file with syn"),
        );

        fs::write("ast.txt", format!("{:#?}", &self.original_ast)).unwrap();

        if let Some(original_ast) = &self.original_ast {
            let mut instrumented_ast = original_ast.clone();
            self.visit_file_mut(&mut instrumented_ast);

            let tokens = instrumented_ast.to_token_stream();
            let instrumented_src_code = tokens.to_string();
            src_to_file(&instrumented_src_code, source_file);
            self.instrumented_ast = Some(instrumented_ast);
        } else {
            panic!()
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
            if let Item::Struct(item_struct) = it {
                self.structs.push(Struct::new(item_struct.ident.clone()));
            }
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
        let ident = self.structs.last_mut().unwrap().ident();
        let ty: Box<Type> = Box::new(syn::parse_quote! {#ident});

        // FIXME here two copies of the same thing are created
        if util::is_constructor(i) {
            let constructor = ConstructorItem::new(i.clone(), ty);
            self.structs
                .last_mut()
                .unwrap()
                .set_constructor(constructor.clone());

            self.callables.push(Callable::Constructor(constructor));
        } else if util::is_method(i) {
            let method = MethodItem::new(i.clone(), ty);
            self.structs
                .last_mut()
                .unwrap()
                .add_method(method.clone());
            self.callables.push(Callable::Method(method));
        } else {
            let func = StaticFnItem::new(i.clone(), ty);
            self.structs
                .last_mut()
                .unwrap()
                .add_static_method(func.clone());
            self.callables.push(Callable::StaticFunction(func));
        }

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

        let func = FunctionItem::new(i.clone());
        self.callables.push(Callable::Function(func));

        VisitMut::visit_visibility_mut(self, &mut i.vis);
        VisitMut::visit_signature_mut(self, &mut i.sig);
        VisitMut::visit_block_mut(self, &mut i.block);

        // TODO don't instrument test functions
        self.instrument_fn(i);
        self.current_fn = None;
    }
}

#[derive(Debug, Clone)]
pub struct BranchManager {
    branches: Vec<Branch>,
    uncovered_branches: Vec<Branch>,
}

impl BranchManager {
    pub fn new(branches: &[Branch]) -> Self {
        BranchManager {
            branches: branches.to_vec(),
            uncovered_branches: branches.to_vec(),
        }
    }

    pub fn branches(&self) -> &Vec<Branch> {
        &self.branches
    }
    pub fn uncovered_branches(&self) -> &Vec<Branch> {
        &self.uncovered_branches
    }

    pub fn set_branches(&mut self, branches: &[Branch]) {
        self.branches = branches.to_vec();
    }

    pub fn set_current_population(&mut self, population: &[TestCase]) {
        let uncovered_branches = self.compute_uncovered_branches(population);
        self.uncovered_branches = uncovered_branches;
    }

    fn compute_uncovered_branches(&self, population: &[TestCase]) -> Vec<Branch> {
        let mut uncovered_branches = vec![];
        for branch in &self.branches {
            let mut covered = false;
            for individual in population {
                if individual.fitness(branch) == 0.0 {
                    covered = true;
                    break;
                }
            }

            if !covered {
                uncovered_branches.push(branch.clone());
            }
        }

        uncovered_branches
    }
}
