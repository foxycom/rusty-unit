use proc_macro2::{Ident, Span};
use quote::ToTokens;
use rustc_middle::ty::TyCtxt;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::token::Else;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{
    Attribute, BinOp, Block, Expr, ExprIf, Fields, File, FnArg, ImplItemMethod, Item, ItemEnum,
    ItemExternCrate, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemUse, Stmt, Type,
    UseTree,
};
use toml::value::Table;
use toml::Value;
use crate::branch::Branch;
use crate::chromosome::{Container, EnumType, StructType, TestCase, ToSyn};
use crate::{fs_util, util};
use crate::types::Callable;


const OUTPUT_ROOT: &'static str = "/Users/tim/Documents/master-thesis/testify/benchmarks";

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    // Name of the executable, path
    Executable(String, PathBuf),
    // Name of the library, path
    Library(String, PathBuf),
    // Path
    SourceCode(PathBuf),
}

impl FileType {
    pub fn to_path_buf(&self) -> PathBuf {
        match self {
            FileType::Executable(_, path) => path.clone(),
            FileType::Library(_, path) => path.clone(),
            FileType::SourceCode(path) => path.clone(),
        }
    }
}

pub struct Toml {
    lib: FileType,
    executables: Vec<FileType>,
    package_name: String,
}

impl Toml {
    pub fn lib(&self) -> &FileType {
        &self.lib
    }
    pub fn executables(&self) -> &Vec<FileType> {
        &self.executables
    }
    pub fn package_name(&self) -> &str {
        &self.package_name
    }
}

struct TomlScanner {}

impl TomlScanner {
    fn executables(toml: &Table) -> Vec<FileType> {
        match toml.get("bin") {
            None => {
                // Default path
                let path = PathBuf::from("src/bin.rs");
                // TODO set default name from the project name
                let name = "".to_owned();
                vec![FileType::Executable(name, path)]
            }
            Some(bin) => {
                if let Value::Array(bin_array) = bin {
                    bin_array
                        .iter()
                        .map(|b| TomlScanner::parse_executable(b))
                        .collect()
                } else {
                    panic!("Should be an array")
                }
            }
        }
    }

    fn parse_executable(bin: &Value) -> FileType {
        let name = if let Some(name) = bin.get("name") {
            if let Value::String(name) = name {
                name.to_string()
            } else {
                panic!("Name is not a string");
            }
        } else {
            "".to_owned()
        };

        let path = if let Some(path) = bin.get("path") {
            if let Value::String(path) = path {
                PathBuf::from(path)
            } else {
                panic!("Path is not a string");
            }
        } else {
            PathBuf::from("src/main.rs")
        };

        FileType::Executable(name, path)
    }

    fn library(toml: &Table) -> FileType {
        match toml.get("lib") {
            None => {
                // Default lib.rs
                let name = "".to_owned();
                let path = PathBuf::from("src/lib.rs");

                FileType::Library(name, path)
            }
            Some(lib) => {
                let name = if let Some(name) = lib.get("name") {
                    if let Value::String(name) = name {
                        name.to_string()
                    } else {
                        panic!("Should be a string");
                    }
                } else {
                    "".to_owned()
                };

                let path = if let Some(path) = lib.get("path") {
                    if let Value::String(path) = path {
                        PathBuf::from(path)
                    } else {
                        panic!("Should be a string");
                    }
                } else {
                    // Default path
                    PathBuf::from("src/lib.rs")
                };

                FileType::Library(name, path)
            }
        }
    }

    fn package_name(toml: &Table) -> String {
        return match toml.get("package") {
            None => {
                unimplemented!()
            }
            Some(package) => {
                let name = if let Some(name) = package.get("name") {
                    if let Value::String(name) = name {
                        name.to_string()
                    } else {
                        panic!("Should be string");
                    }
                } else {
                    panic!("Huh, no package name?")
                };
                name
            }
        };
    }
}

pub struct ProjectScanner {}

impl ProjectScanner {
    pub fn open(project_root: &str) -> Project {
        let project_root = PathBuf::from(project_root);
        let toml_path = project_root.join("Cargo.toml");
        let toml_table = ProjectScanner::parse_toml(toml_path.as_path());

        let source_dir = project_root.join("src");
        let mut source_files = vec![];

        let executables = TomlScanner::executables(&toml_table);
        let lib = TomlScanner::library(&toml_table);
        let package_name = TomlScanner::package_name(&toml_table);

        let toml = Toml {
            executables,
            lib,
            package_name,
        };

        ProjectScanner::read_file_tree(
            project_root.as_path(),
            source_dir.as_path(),
            &mut source_files,
            &toml,
        ).unwrap();

        Project::new(project_root, PathBuf::from(OUTPUT_ROOT), toml, source_files)
    }

    fn read_file_tree(
        project_root: &Path,
        src_dir: &Path,
        source_files: &mut Vec<SourceFile>,
        toml: &Toml,
    ) -> io::Result<()> {
        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                ProjectScanner::read_file_tree(project_root, &path, source_files, toml);
            } else if let Some(extension) = path.extension() {
                if extension.eq("rs") {
                    // Check whether it is an executable, a library, or a regular source file

                    let relative_to_root = path.strip_prefix(project_root).unwrap();
                    let file_type = if ProjectScanner::is_library(relative_to_root, &toml.lib) {
                        toml.lib.clone()
                    } else if let Some(executable) =
                        ProjectScanner::is_executable(relative_to_root, &toml.executables)
                    {
                        executable.clone()
                    } else {
                        FileType::SourceCode(path.to_path_buf())
                    };

                    // Put the copy file under the same relative path structure
                    let relative_path = path.strip_prefix(project_root).unwrap();

                    let mut output_path = PathBuf::from(OUTPUT_ROOT);
                    output_path.push(relative_path);

                    let parent_dir = output_path.parent().unwrap();
                    fs::create_dir_all(parent_dir).unwrap();

                    let source_file =
                        SourceFile::new(path.as_path(), output_path.as_path(), file_type);
                    source_files.push(source_file);
                }
            }
        }
        Ok(())
    }

    fn is_library(relative_path: &Path, library: &FileType) -> bool {
        if let FileType::Library(_, lib_path) = library {
            relative_path == lib_path
        } else {
            panic!("Is not a library");
        }
    }

    fn is_executable<'a>(
        relative_path: &Path,
        executables: &'a Vec<FileType>,
    ) -> Option<&'a FileType> {
        for executable in executables {
            if let FileType::Executable(_, exec_path) = executable {
                let exec_path = if !exec_path.has_root() && exec_path.starts_with(Path::new("./")) {
                    exec_path.strip_prefix("./").unwrap()
                } else {
                    exec_path
                };
                if exec_path == relative_path {
                    return Some(executable);
                }
            } else {
                panic!("Not an executable");
            }
        }

        None
    }

    fn parse_toml(toml_path: &Path) -> Table {
        let toml_content = fs::read_to_string(toml_path.clone())
            .expect(&format!("Could not read {}", toml_path.to_str().unwrap()));
        let toml: Table = toml::from_str(&toml_content)
            .expect(&format!("Could not parse {}", toml_path.to_str().unwrap()));
        toml
    }
}

pub struct Project {
    project_root: PathBuf,
    output_root: PathBuf,
    toml: Toml,
    source_files: Vec<SourceFile>,
    cargo_path: PathBuf,
}

impl Project {
    fn new(
        project_root: PathBuf,
        output_root: PathBuf,
        toml: Toml,
        source_files: Vec<SourceFile>,
    ) -> Self {
        let cargo_path = util::cargo_path().unwrap();
        Project {
            project_root,
            output_root,
            toml,
            source_files,
            cargo_path,
        }
    }

    pub fn open(&self) {
        unimplemented!()
    }

    pub fn source_files(&self) -> &Vec<SourceFile> {
        &self.source_files
    }

    pub fn project_root(&self) -> &Path {
        self.project_root.as_path()
    }

    pub fn source_files_mut(&mut self) -> &mut Vec<SourceFile> {
        &mut self.source_files
    }

    pub fn make_copy(&mut self) {
        // Clear the target dir
        fs::remove_dir_all(self.output_root.as_path()).unwrap();

        // Write source files
        for file in &mut self.source_files {
            file.write();
        }

        // Copy monitor module
        let monitor_source_path =
            PathBuf::from("/Users/tim/Documents/master-thesis/testify/src/monitor.rs");
        let monitor_target_path = self.output_root.join("src/testify_monitor.rs");
        fs::copy(monitor_source_path, monitor_target_path).unwrap();

        // Copy toml
        let original_toml_path = self.project_root.join("Cargo.toml");
        let output_toml_path = self.output_root.join("Cargo.toml");
        fs::copy(original_toml_path, output_toml_path).unwrap();

        let deps_path = self.output_root.join("target/debug/deps");
        fs::create_dir_all(deps_path).unwrap();
    }

    pub fn toml(&self) -> &Toml {
        &self.toml
    }

    pub fn add_tests(&mut self, test_cases: &Vec<TestCase>, tcx: &TyCtxt<'_>) {
        let first_source_file = self.source_files.first_mut().unwrap();
        for test_case in test_cases {
            first_source_file.add_test(test_case, tcx);
        }

        first_source_file.write();
    }

    pub fn run_tests(&self) {
        let mut cmd = Command::new(self.cargo_path.as_path());
        let log_file = fs::File::create("out.log").unwrap();
        let err_file = fs::File::create("err.log").unwrap();
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(err_file));

        // Run the tests
        cmd.args(&[
            "test",
           /* "--package",
            &self.toml.package_name,*/
            "testify_tests",
        ])
        .current_dir(self.output_root.as_path());
        match cmd.status() {
            Ok(_) => {
                //println!("Test {}: OK", test_case.name());
            }
            Err(e) => panic!("{}", e),
        }
    }

    pub fn clear_tests(&mut self) {
        for file in &mut self.source_files {
            file.clear_tests();
            file.write();
        }
    }

    pub fn clear_build_dirs(&self) {
        let target_path = self.project_root.join("target");
        let debug_path = self.project_root.join("debug");

        fs_util::remove_dir_all(target_path.as_path()).unwrap();
        fs_util::remove_dir_all(debug_path.as_path()).unwrap();

        let deps_path = self.project_root.join("target/debug/deps");
        fs::create_dir_all(deps_path.as_path()).unwrap();
    }

    pub fn build_args(&self) -> Vec<String> {
        let args = [
            "rustc",
            "--crate-name",
            "additions",
            "--edition=2018",
            "/Users/tim/Documents/master-thesis/testify/benchmarks/src/main.rs",
            "--error-format=json",
            "--json=diagnostic-rendered-ansi",
            "--crate-type",
            "bin",
            "--emit=dep-info,link",
            "-C",
            "embed-bitcode=no",
            "-C",
            "split-debuginfo=unpacked",
            "-C",
            "debuginfo=2",
            "-C",
            "metadata=5978598c4741d9d6",
            "--out-dir",
            "/Users/tim/Documents/master-thesis/testify/benchmarks/target/debug/deps",
            "-C",
            "incremental=/Users/tim/Documents/master-thesis/testify/benchmarks/debug/incremental",
            "-L",
            "dependency=/Users/tim/Documents/master-thesis/testify/benchmarks/target/debug/deps",
            "--sysroot",
            &fs_util::sysroot(),
        ];
        args.iter().map(|a| a.to_string()).collect()
    }

    pub fn test_build_args(&self) -> Vec<String> {
        let args = [
            "rustc",
            "--crate-name",
            "additions",
            "--edition=2018",
            "/Users/tim/Documents/master-thesis/testify/benchmarks/src/main.rs",
            "--error-format=json",
            "--json=diagnostic-rendered-ansi",
            "--crate-type",
            "bin",
            "--emit=dep-info,link",
            "-C",
            "embed-bitcode=no",
            "-C",
            "split-debuginfo=unpacked",
            "-C",
            "debuginfo=2",
            "-C",
            "metadata=5978598c4741d9d6",
            "--out-dir",
            "/Users/tim/Documents/master-thesis/testify/benchmarks/target/debug/deps",
            "-C",
            "incremental=/Users/tim/Documents/master-thesis/testify/benchmarks/debug/incremental",
            "-L",
            "dependency=/Users/tim/Documents/master-thesis/testify/benchmarks/target/debug/deps",
            "--sysroot",
            &fs_util::sysroot(),
        ];
        args.iter().map(|a| a.to_string()).collect::<Vec<String>>();
        todo!()
    }
    pub fn output_root(&self) -> &Path {
        self.output_root.as_path()
    }
}

#[derive(Debug, Clone)]
pub struct VisitState {
    current_path: Vec<Ident>,
    file_name: String,
}

impl VisitState {
    pub fn new(file_name: &str) -> Self {
        VisitState {
            file_name: file_name.to_owned(),
            current_path: vec![Ident::new(file_name, Span::call_site())],
        }
    }

    pub fn push_path(&mut self, ident: &Ident) {
        self.current_path.push(ident.clone());
    }

    pub fn pop_path(&mut self) -> Ident {
        self.current_path.pop().expect("Path is already empty")
    }
}



#[derive(Debug, Clone)]
pub struct SourceFile {
    file_path: PathBuf,
    output_path: PathBuf,
    ast: File,
    callables: Vec<Callable>,
    branches: Vec<Branch>,
    containers: Vec<Container>,
    imports: Vec<Import>,
    visit_state: VisitState,
    file_type: FileType,
}

impl SourceFile {
    pub fn new(src_path: &Path, output_path: &Path, file_type: FileType) -> SourceFile {
        let content = fs::read_to_string(src_path.clone())
            .expect(&format!("Could not read {}", src_path.to_str().unwrap()));
        let ast = syn::parse_file(&content)
            .expect("Could not parse the contents of the Rust source file with syn");

        let file_name = src_path
            .file_stem()
            .expect("No file name given")
            .to_str()
            .expect("File name contains unsupported encoding");
        let visit_state = VisitState::new(file_name);
        let mut source_file = SourceFile {
            output_path: output_path.to_path_buf(),
            file_path: src_path.to_path_buf(),
            ast: ast.clone(),
            callables: vec![],
            branches: vec![],
            containers: vec![],
            imports: vec![],
            visit_state,
            file_type,
        };

        source_file
    }

    pub fn structs(&self) -> Vec<&StructType> {
        unimplemented!()
    }

    pub fn enums(&self) -> Vec<&EnumType> {
        unimplemented!()
    }

    pub fn types(&self) -> &Vec<Container> {
        &self.containers
    }

    pub fn branches(&self) -> &Vec<Branch> {
        &self.branches
    }

    pub fn file_path(&self) -> &Path {
        self.file_path.as_path()
    }

    pub fn ast(&self) -> &File {
        &self.ast
    }

    pub fn set_ast(&mut self, ast: File) {
        self.ast = ast;
    }
    pub fn file_type(&self) -> &FileType {
        &self.file_type
    }

    pub fn add_test(&mut self, test_case: &TestCase, tcx: &TyCtxt<'_>) {
        let tests_mod = self
            .ast
            .items
            .iter_mut()
            .find_map(|i| {
                if let Item::Mod(item_mod) = i {
                    if item_mod.ident.to_string() == "testify_tests" {
                        return Some(item_mod);
                    }
                }
                None
            });

        let code = test_case.to_syn(tcx);
        if let Some(tests_mod) = tests_mod {
            let (_, ref mut content) = tests_mod.content.as_mut().unwrap();
            content.push(code);
        } else {
            let tests_mod: Item = syn::parse_quote! {
                #[cfg(test)]
                mod testify_tests {
                    use super::*;
                    #code
                }
            };

            self.ast.items.push(tests_mod);
        }
    }

    pub fn imports_monitor(&self) -> bool {
        let monitor_mod = self.ast.items.iter().find(|i| {
            if let Item::Mod(item_mod) = i {
                return item_mod.ident.to_string() == "testify_monitor";
            }
            false
        });

        monitor_mod.is_some()
    }

    pub fn write(&mut self) {
        match &self.file_type {
            FileType::Executable(_, _) | FileType::Library(_, _) => {
                if !self.imports_monitor() {
                    let import: Item = syn::parse_quote! {
                        pub mod testify_monitor;
                    };
                    self.ast.items.insert(0, import);
                }
            }
            _ => {}
        }

        let token_stream = self.ast.to_token_stream();
        let code = token_stream.to_string();

        let parent = self.output_path.parent().unwrap();
        fs::create_dir_all(parent);

        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.output_path)
            .unwrap();
        file.write_all(&code.as_bytes());
    }

    pub fn clear_tests(&mut self) {
        self.ast.items.retain(|i| {
            if let Item::Mod(item_mod) = i {
                return item_mod.ident.to_string() != "testify_tests";
            }
            true
        });

        let tests_mod: Item = syn::parse_quote! {
                #[cfg(test)]
                mod testify_tests {
                    use super::*;
                }
            };

        self.ast.items.push(tests_mod);
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


#[derive(Debug, Clone)]
pub enum Import {
    Use(Use),
    Mod(Mod),
}

#[derive(Debug, Clone)]
pub struct Use {
    syn_item_use: ItemUse,
    path: Vec<Ident>,
}

impl Use {
    pub fn new(syn_item_use: ItemUse) -> Self {
        let mut path = vec![];
        Use::path(&syn_item_use.tree, &mut path);
        Use { syn_item_use, path }
    }

    fn path(use_tree: &UseTree, path: &mut Vec<Ident>) {
        match use_tree {
            UseTree::Path(use_path) => {
                path.push(use_path.ident.clone());
                Use::path(use_path.tree.as_ref(), path);
            }
            UseTree::Name(name) => path.push(name.ident.clone()),
            UseTree::Rename(rename) => path.push(rename.rename.clone()),
            UseTree::Glob(_) => path.push(Ident::new("*", Span::call_site())),
            UseTree::Group(group) => {
                for item in group.items.iter() {
                    Use::path(item, path);
                }
            }
        }
    }

    pub fn ends_with(&self, ident: &Ident) -> bool {
        let last_ident = self.path.last().unwrap();
        last_ident == ident
    }
}

#[derive(Debug, Clone)]
pub struct Mod {
    syn_item_mod: ItemMod,
}

impl Mod {
    pub fn new(syn_item_mod: ItemMod) -> Self {
        // Mod can only have one-level identifier
        Mod { syn_item_mod }
    }

    pub fn ident(&self) -> &Ident {
        &self.syn_item_mod.ident
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Ident;
    use toml::value::Table;
    use toml::Value;

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    #[test]
    fn test_use_contains_basic_ident() {
        let import: ItemUse = syn::parse_quote! {use something::from::Hello;};
        let use_item = Use::new(import);

        let ident = Ident::new("Hello", Span::call_site());
        assert!(use_item.ends_with(&ident));
    }

    #[test]
    fn test_use_contains_path() {
        let import: ItemUse = syn::parse_quote! {use something::from::Hello;};

        let us = common::source::Use::new(import);
    }

    #[test]
    fn test_toml_parse_library() {
        let table: Table = toml::from_str(
            r#"
            [package]
            name = "additions"

            [lib]
            name = "additions-lib"
            path = "./src/lib.rs"
        "#,
        )
        .unwrap();

        let lib = TomlScanner::library(&table);
        let expected_lib =
            FileType::Library("additions-lib".to_string(), PathBuf::from("./src/lib.rs"));
        assert_eq!(expected_lib, lib);
    }

    #[test]
    fn test_toml_parse_executables() {
        let table: Table = toml::from_str(
            r#"
            [package]
            name = "additions"

            [[bin]]
            name = "additions"
            path = "./src/bin/main.rs"

            [[bin]]
            name = "some-other-bin"
        "#,
        )
        .unwrap();

        let executables = TomlScanner::executables(&table);
        let expected_additions =
            FileType::Executable("additions".to_string(), PathBuf::from("./src/bin/main.rs"));
        let expected_other_bin =
            FileType::Executable("some-other-bin".to_string(), PathBuf::from("src/main.rs"));
        assert_eq!(executables.len(), 2);
        assert!(executables.contains(&expected_additions));
        assert!(executables.contains(&expected_other_bin));
    }

    #[test]
    fn test_toml_parse_package() {
        let table: Table = toml::from_str(
            r#"
            [package]
            name = "additions"

            [[bin]]
            name = "bin"
            path = "./src/bin/main.rs"
        "#,
        )
        .unwrap();

        let package_name = TomlScanner::package_name(&table);
        assert_eq!(String::from("additions"), package_name);
    }

    #[test]
    fn test_project_scanner_directory_tree() {
        let path = "/Users/tim/Documents/master-thesis/testify/tests/examples";

        let project = ProjectScanner::open(path);

        assert_eq!(project.source_files().len(), 2);

        let main_file = project
            .source_files()
            .iter()
            .find(|&sf| sf.file_path().ends_with(Path::new("main.rs")))
            .unwrap();

        assert!(if let FileType::Executable(_, _) = main_file.file_type() {
            true
        } else {
            false
        });

        let dependency_file = project
            .source_files()
            .iter()
            .find(|&sf| sf.file_path().ends_with(Path::new("dependency.rs")))
            .unwrap();
        assert!(
            if let FileType::SourceCode(_) = dependency_file.file_type() {
                true
            } else {
                false
            }
        );
    }

    #[test]
    fn test_main_contains_3_structs() {
        let main_path =
            Path::new("/Users/tim/Documents/master-thesis/testify/tests/examples/src/main.rs");
        let file_type = FileType::Executable("".to_owned(), main_path.to_path_buf());
        let output_dir = std::env::temp_dir();
        let main = SourceFile::new(&main_path, output_dir.as_path(), file_type);

        assert_eq!(main.containers.len(), 3);

        let area_calculator = T::new(vec![ident("main"), ident("AreaCalculator")]);
        let rectangle = T::new(vec![ident("main"), ident("Rectangle")]);
        let some_struct = T::new(vec![ident("main"), ident("SomeStruct")]);

        let structs = vec![area_calculator, rectangle, some_struct];
        let contains_all_structs = structs
            .iter()
            .map(|s| main.containers.iter().find(|&c| c.ty() == s).is_some())
            .all(|r| r);
        assert!(contains_all_structs);
    }

    #[test]
    fn test_area_calculator_has_2_methods() {
        let main_path =
            Path::new("/Users/tim/Documents/master-thesis/testify/tests/examples/src/main.rs");
        let file_type = FileType::Executable("".to_owned(), main_path.to_path_buf());
        let output_dir = std::env::temp_dir();
        let main = SourceFile::new(&main_path, output_dir.as_path(), file_type);

        assert_eq!(main.containers.len(), 3);

        let path = vec![ident("main"), ident("AreaCalculator")];
        let area_calculator_ty = T::new(path);

        assert_eq!(main.callables_of(&area_calculator_ty).len(), 3);
    }

    #[test]
    fn test_dependency_has_full_path() {
        let main_path =
            Path::new("/Users/tim/Documents/master-thesis/testify/tests/examples/src/main.rs");
        let file_type = FileType::Executable("".to_owned(), main_path.to_path_buf());
        let output_dir = std::env::temp_dir();

        let main = SourceFile::new(&main_path, output_dir.as_path(), file_type);

        assert_eq!(main.containers.len(), 3);

        let path = vec![ident("main"), ident("SomeStruct")];
        let some_struct_ty = T::new(path);

        let callables = main.callables_of(&some_struct_ty);

        let mut dep_method = None;
        for c in callables {
            if c.name() == "something_with_dependency" {
                dep_method = Some(c);
                break;
            }
        }

        let dependency_ty = T::new(vec![
            ident("crate"),
            ident("dependency"),
            ident("DependencyStruct"),
        ]);

        let dep_param = dep_method
            .unwrap()
            .params()
            .iter()
            .find(|&p| p.real_ty() == &dependency_ty);

        assert!(dep_param.is_some());
    }

    #[test]
    fn test_nested_dependency() {
        let main_path =
            Path::new("/Users/tim/Documents/master-thesis/testify/tests/examples/src/main.rs");
        let file_type = FileType::Executable("".to_owned(), main_path.to_path_buf());
        let output_dir = std::env::temp_dir();
        let main = SourceFile::new(&main_path, output_dir.as_path(), file_type);

        assert_eq!(main.containers.len(), 3);

        let path = vec![ident("main"), ident("SomeStruct")];
        let some_struct_ty = T::new(path);

        let callables = main.callables_of(&some_struct_ty);

        let mut dep_method = None;
        for c in callables {
            if c.name() == "invoke_nested_dependency" {
                dep_method = Some(c);
                break;
            }
        }

        let nested_dependency_ty = T::new(vec![
            ident("crate"),
            ident("dependency"),
            ident("nested_mod"),
            ident("sub_mod"),
            ident("NestedStruct"),
        ]);

        let dep_param = dep_method
            .unwrap()
            .params()
            .iter()
            .find(|&p| p.real_ty() == &nested_dependency_ty);

        assert!(dep_param.is_some())
    }
}
