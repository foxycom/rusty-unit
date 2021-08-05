use std::process::{Command, Stdio, Output, ExitStatus};
use std::path::{PathBuf, Path};
use std::{io, fs};
use crate::chromosome::TestCase;
use std::io::{Write, Error};
use syn::{Item, ItemFn, File};
use syn::visit_mut::VisitMut;
use quote::ToTokens;
use crate::instr::data::Branch;
use crate::instr::util::instrumented_path;
use std::hash::{Hash, Hasher};
use crate::parser::TraceParser;

#[derive(Debug, Clone)]
pub struct SourceFile {
    original_path: String,
    instrumented_path: String,
    writer: TestWriter,
    runner: TestRunner,
    registrar: ModuleRegistrar
}

impl SourceFile {
    pub fn new(original_path: &str) -> SourceFile {
        let instrumented_path = instrumented_path(original_path);
        SourceFile {
            original_path: original_path.to_owned(),
            instrumented_path: instrumented_path.to_owned(),
            writer: TestWriter::new(&instrumented_path),
            runner: TestRunner::new(),
            registrar: ModuleRegistrar::new()
        }
    }

    /// Writes the tests as source code into the file.
    pub fn add_tests(&mut self, tests: &[TestCase]) {
        self.writer.add(tests);
    }

    /// Runs the tests provided they have been added to the source file before.
    pub fn run_tests(&mut self, tests: &mut [TestCase]) {
        self.registrar.register();
        self.runner.run();
        for test in tests {
            let file = format!("/Users/tim/Documents/master-thesis/testify/src/examples/additions/trace_{}.txt", test.id());
            test.set_results(TraceParser::parse(&file).unwrap());
            match fs::remove_file(&file) {
                Err(err) => {
                    panic!("There was no trace file: {}", err);
                }
                _ => {}
            }
        }
        /*for test in tests {
            test.set_results(TraceParser::parse("/Users/tim/Documents/master-thesis/testify/src/examples/additions/trace.txt").unwrap());
            match fs::remove_file("/Users/tim/Documents/master-thesis/testify/src/examples/additions/trace.txt") {
                Err(err) => {
                    panic!("There was no trace file: {}", err);
                }
                _ => {}
            }
        }
*/        self.registrar.unregister();
    }

    /// Removes the generated tests from the module.
    pub fn clear_tests(&mut self) {
        self.writer.clear();
    }

    pub fn instrumented_path(&self) -> &str {
        &self.instrumented_path
    }

    pub fn original_pth(&self) -> &str {
        &self.original_path
    }

    pub fn instr_module_name(&self) -> &str {
        Path::new(&self.instrumented_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
    }
}

impl PartialEq for SourceFile {
    fn eq(&self, other: &Self) -> bool {
        self.instrumented_path == other.instrumented_path
        && self.original_path == other.original_path
    }
}

impl Eq for SourceFile {}

impl Hash for SourceFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.original_path.hash(state);
        self.instrumented_path.hash(state);
    }
}

impl Default for SourceFile {
    fn default() -> Self {
        SourceFile {
            original_path: Default::default(),
            instrumented_path: Default::default(),
            writer: TestWriter::new(Default::default()),
            runner: TestRunner::new(),
            registrar: ModuleRegistrar::new()
        }
    }
}

fn fmt_path() -> io::Result<PathBuf> {
    match which::which("rustfmt") {
        Ok(p) => Ok(p),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e)))
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
            ))
        }
        Err(_) => Ok(source)
    }
}

#[derive(Debug, Clone)]
struct TestWriter {
    use_all_star: Item,
    test_cases: Option<Vec<TestCase>>,
    original_ast: File,
    instrumented_path: String,
}

impl TestWriter {
    const TESTS_MODULE: &'static str = "testify_tests";
    pub fn new(instrumented_path: &str) -> Self {
        let content = fs::read_to_string(&instrumented_path)
            .expect("Could not read the Rust source file");
        let ast = syn::parse_file(&content)
            .expect("Could not parse the contents of the Rust source file with syn");

        TestWriter {
            use_all_star: syn::parse_quote! { use super::*; },
            test_cases: None,
            original_ast: ast,
            instrumented_path: instrumented_path.to_owned(),
        }
    }

    pub fn add(&mut self, test_cases: &[TestCase]) -> io::Result<()> {
        self.test_cases = Some(test_cases.to_vec());
        let mut ast = self.original_ast.clone();
        self.visit_file_mut(&mut ast);

        let tokens = ast.to_token_stream();
        let modified_src_code = fmt_string(&tokens.to_string())?;
        let mut instrumented_file = fs::File::create(&self.instrumented_path)?;
        instrumented_file.write_all(&modified_src_code.as_bytes())?;

        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        let tokens = self.original_ast.to_token_stream();
        let original_source_code = fmt_string(&tokens.to_string())?;
        let mut instrumented_file = fs::File::create(&self.instrumented_path)?;
        instrumented_file.write_all(&original_source_code.as_bytes())?;

        Ok(())
    }

    fn contains_use_super_star(&self, items: &[Item]) -> bool {
        items.iter().any(|i| {
            *i == self.use_all_star
        })
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
                        let mut code: Vec<Item> = tests.iter_mut().map(|t| t.to_syn()).collect();
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
struct ModuleRegistrar {
    ast: File,
}

impl ModuleRegistrar {
    pub fn new() -> Self {
        let main_file = "/Users/tim/Documents/master-thesis/testify/src/examples/additions/src/main.rs";
        let content = fs::read_to_string(main_file)
            .expect("Could not read the Rust source file");
        let ast = syn::parse_file(&content)
            .expect("Could not parse the contents of the Rust source file with syn");

        ModuleRegistrar {
            ast,
        }
    }

    pub fn register(&mut self) {
        let mut ast = self.ast.clone();
        self.visit_file_mut(&mut ast);

        let tokens = ast.to_token_stream();
        let src = fmt_string(&tokens.to_string()).unwrap();

        let mut file = fs::File::create("/Users/tim/Documents/master-thesis/testify/src/examples/additions/src/main.rs").expect("Could not create output source file");
        file.write_all(&src.as_bytes()).unwrap();
    }

    pub fn unregister(&self) {
        let tokens = self.ast.to_token_stream();
        let src = fmt_string(&tokens.to_string()).unwrap();
        let mut file = fs::File::create("/Users/tim/Documents/master-thesis/testify/src/examples/additions/src/main.rs").expect("Could not create output source file");
        file.write_all(&src.as_bytes()).unwrap();
    }
}

impl VisitMut for ModuleRegistrar {
    fn visit_file_mut(&mut self, i: &mut File) {
        //let instrumented_mod = self.target.instrumented_mod();
        let instrumented_mod = "hei";
        let use_instrumented_module: Item = syn::parse_quote! {
            mod main_instrumented;
        };

        let uses_instr_module = i.items.iter().any(|i| {
            *i == use_instrumented_module
        });

        if !uses_instr_module {
            i.items.insert(0, use_instrumented_module);
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
        cmd.args(&["test",
            "--package",
            "additions",
            "testify_tests"])
            .current_dir("/Users/tim/Documents/master-thesis/testify/src/examples/additions");
        match cmd.status() {
            Ok(_) => {
                //println!("Test {}: OK", test_case.name());
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    fn cargo_path(&self) -> io::Result<PathBuf> {
        match which::which("cargo") {
            Ok(p) => Ok(p),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e)))
        }
    }
}
