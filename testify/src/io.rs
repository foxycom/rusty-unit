use std::process::{Command, Stdio, Output, ExitStatus};
use std::path::PathBuf;
use std::io;
use crate::chromosome::TestCase;

pub struct SourceFile<'a> {
    path: &'a str
}

impl<'a> SourceFile<'a> {
    pub fn new(path: &'a str) -> SourceFile<'a> {
        SourceFile {
            path
        }
    }

    pub fn path(&self) -> &'a str {
        self.path
    }
}

pub mod writer {
    use super::*;

    use std::fs;
    use quote::ToTokens;
    use syn::visit_mut::{VisitMut, visit_item_mut, visit_file_mut};
    use syn::{Item, ItemUse, File, ItemFn};
    use std::io::{Write, Stdout, stdin};
    use std::string::FromUtf8Error;
    use crate::instr::data::Branch;
    use std::rc::Rc;

    fn rustfmt_path() -> io::Result<PathBuf> {
        match which::which("rustfmt") {
            Ok(p) => Ok(p),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e)))
        }
    }

    fn rustfmt_string(source: &str) -> io::Result<String> {
        let rustfmt = rustfmt_path()?;
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

    #[derive(Debug)]
    pub struct TestWriter {
        use_all_star: Item,
        test_cases: Vec<TestCase>,
        // TODO this is ugly
        inserting: bool,
        current_test: usize,
    }

    impl TestWriter {
        pub fn new() -> Self {
            TestWriter {
                use_all_star: syn::parse_quote! {
                    use super::*;
                },
                test_cases: vec![],
                inserting: true,
                current_test: 0,
            }
        }

        pub fn write(&mut self, test_cases: &[TestCase]) -> io::Result<()> {
            self.test_cases = test_cases.to_vec();
            self.inserting = true;
            self.start()
        }

        pub fn unwrite(&mut self, test_cases: &[TestCase]) -> io::Result<()> {
            self.test_cases = test_cases.to_vec();
            self.inserting = false;
            self.start()
        }

        fn start(&mut self) -> io::Result<()> {
            self.current_test = 0;

            let targets: Vec<Branch> = self.test_cases
                .iter()
                .map(|t| t.target().to_owned())
                .collect();

            for target in &targets {
                let path = target.instrumented_file();

                let content = fs::read_to_string(&path)
                    .expect("Could not read the Rust source file");
                let mut ast = syn::parse_file(&content)
                    .expect("Could not parse the contents of the Rust source file with syn");

                self.visit_file_mut(&mut ast);

                let tokens = ast.to_token_stream();
                let src = rustfmt_string(&tokens.to_string()).unwrap();

                let mut file = fs::File::create(path).expect("Could not create output source file");
                file.write_all(&src.as_bytes()).unwrap();
                self.current_test += 1;
            }

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
                let ident = &item_mod.ident;
                if ident.to_string() == "tests" {
                    if let Some((_, items)) = &mut item_mod.content {
                        if !self.contains_use_super_star(items) {
                            items.insert(0, self.use_all_star.clone());
                        }

                        if self.inserting {
                            let code = self.test_cases
                                .get(self.current_test)
                                .unwrap()
                                .to_syn();

                            // Insert at the end
                            items.insert(items.len(), code);
                        } else {
                            items.retain(|i| {
                                if let Item::Fn(ItemFn { sig, .. }) = i {
                                    !sig.ident.to_string().starts_with(TestCase::TEST_FN_PREFIX)
                                } else {
                                    false
                                }
                            });
                        }
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

    pub struct ModuleRegistrar<'a> {
        target: &'a Branch,
        ast: File,
    }

    impl<'a> ModuleRegistrar<'a> {
        pub fn new(target: &'a Branch) -> Self {
            let main_file = "/Users/tim/Documents/master-thesis/testify/src/examples/additions/src/main.rs";
            let content = fs::read_to_string(main_file)
                .expect("Could not read the Rust source file");
            let ast = syn::parse_file(&content)
                .expect("Could not parse the contents of the Rust source file with syn");

            ModuleRegistrar {
                target,
                ast,
            }
        }

        pub fn register(&mut self) {
            let mut ast = self.ast.clone();
            self.visit_file_mut(&mut ast);

            let tokens = ast.to_token_stream();
            let src = rustfmt_string(&tokens.to_string()).unwrap();

            let mut file = fs::File::create("/Users/tim/Documents/master-thesis/testify/src/examples/additions/src/main.rs").expect("Could not create output source file");
            file.write_all(&src.as_bytes()).unwrap();
        }

        pub fn unregister(&self) {
            let tokens = self.ast.to_token_stream();
            let src = rustfmt_string(&tokens.to_string()).unwrap();
            let mut file = fs::File::create("/Users/tim/Documents/master-thesis/testify/src/examples/additions/src/main.rs").expect("Could not create output source file");
            file.write_all(&src.as_bytes()).unwrap();
        }
    }

    impl<'a> VisitMut for ModuleRegistrar<'a> {
        fn visit_file_mut(&mut self, i: &mut File) {
            let instrumented_mod = self.target.instrumented_mod();
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
}

pub mod runner {
    use super::*;

    use std::io::Error;
    use std::fs;

    pub struct TestRunner {}

    impl TestRunner {
        pub fn new() -> Self {
            TestRunner {}
        }

        pub fn run(&self, test_case: &TestCase) -> io::Result<()> {
            let cargo = self.cargo_path()?;
            let mut cmd = Command::new(&*cargo);
            let log_file = fs::File::create("out.log")?;
            cmd.stdin(Stdio::piped()).stdout(Stdio::from(log_file));

            // TODO extract package and bin files
            cmd.args(&["test",
                "--package",
                "additions",
                &format!("tests::{}", test_case.name())])
                .current_dir("/Users/tim/Documents/master-thesis/testify/src/examples/additions");
            match cmd.status() {
                Ok(_) => {
                    println!("Test {}: OK", test_case.name());
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
}

