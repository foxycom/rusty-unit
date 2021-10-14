use crate::chromosome::ToSyn;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{fs, io};
use syn::visit_mut::VisitMut;
use syn::{File, Item, ItemMod};

#[derive(Debug, Clone)]
pub struct TestClearer {
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
pub struct TestWriter<S>
where
    S: ToSyn + Clone,
{
    use_all_star: Item,
    test_cases: Vec<S>,
}

impl<S> TestWriter<S>
where
    S: ToSyn + Clone,
{
    const TESTS_MODULE: &'static str = "testify_tests";
    pub fn new() -> Self {
        TestWriter {
            use_all_star: syn::parse_quote! { use super::*; },
            test_cases: vec![],
        }
    }

    pub fn add_tests(&mut self, test_cases: &[S], ast: &File) -> File {
        self.test_cases = test_cases.to_vec();
        let mut ast = ast.clone();
        self.visit_file_mut(&mut ast);
        ast
    }

    fn contains_use_super_star(&self, items: &[Item]) -> bool {
        items.iter().any(|i| *i == self.use_all_star)
    }
}

impl<S> VisitMut for TestWriter<S>
where
    S: ToSyn + Clone,
{
    fn visit_item_mut(&mut self, i: &mut Item) {
        if let Item::Mod(item_mod) = i {
            let mod_name = &item_mod.ident;
            if mod_name.to_string() == TestWriter::<S>::TESTS_MODULE {
                if let Some((_, items)) = &mut item_mod.content {
                    if !self.contains_use_super_star(items) {
                        items.insert(0, self.use_all_star.clone());
                    }

                    if !self.test_cases.is_empty() {
                        let mut code: Vec<Item> =
                            self.test_cases.iter().map(|t| t.to_syn()).collect();
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
pub struct TestRunner {}

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
