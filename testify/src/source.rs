use proc_macro2::{Ident, Span};
use quote::ToTokens;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::token::Else;
use syn::visit::Visit;
use syn::{
    Attribute, BinOp, Block, Expr, ExprIf, Fields, File, FnArg, ImplItemMethod, Item, ItemEnum,
    ItemExternCrate, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStruct, ItemUse, Stmt, Type,
    UseTree,
};
use toml::value::Table;

use crate::chromosome::{
    Callable, Chromosome, ConstructorItem, Container, EnumType, FitnessValue, FnInvStmt,
    FunctionItem, MethodItem, Param, PrimitiveItem, StaticFnItem, StructType, TestCase, T,
};
use crate::parser::TraceParser;
use crate::util;
use crate::util::type_name;

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

pub struct ProjectScanner {}

impl ProjectScanner {
    pub fn open(project_root: &str) -> Project {
        let project_root = PathBuf::from(project_root);
        let toml_path = project_root.join("Cargo.toml");
        let toml = ProjectScanner::parse_toml(toml_path.as_path());

        let source_dir = project_root.join("src");
        let mut source_files = vec![];

        ProjectScanner::read_file_tree(source_dir.as_path(), &mut source_files);
        Project {
            toml,
            source_files,
            project_root,
        }
    }

    fn read_file_tree(src_dir: &Path, source_files: &mut Vec<SourceFile>) -> io::Result<()> {
        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                ProjectScanner::read_file_tree(&path, source_files);
            } else if let Some(extension) = path.extension() {
                if extension.eq("rs") {
                    let source_file = SourceFile::new(path.as_path());
                    source_files.push(source_file);
                }
            }
        }
        Ok(())
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
    toml: Table,
    source_files: Vec<SourceFile>,
}

impl Project {
    pub fn open(&self) {}
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
    ast: File,
    callables: Vec<Callable>,
    branches: Vec<Branch>,
    containers: Vec<Container>,
    imports: Vec<Import>,
    visit_state: VisitState,
}

impl SourceFile {
    pub fn new(src_path: &Path) -> SourceFile {
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
            file_path: src_path.to_path_buf(),
            ast: ast.clone(),
            callables: vec![],
            branches: vec![],
            containers: vec![],
            imports: vec![],
            visit_state,
        };
        source_file.visit_file(&ast);

        source_file.make_paths_explicit();

        source_file
    }

    fn make_paths_explicit(&mut self) {
        for callable in &mut self.callables {
            for param in callable.params_mut() {
                let ty = param.ty_mut();

                let path_prefix = self.imports.iter().find(|&i| i.contains(ty));

                if let Some(prefix) = path_prefix {
                    // In some cases, the absolute path might already be specified
                    let mut prefix = prefix.path();
                    prefix.remove(prefix.len() - 1);
                    prefix.append(ty.path_mut());
                    ty.set_path(prefix);
                }
            }
        }
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

    pub fn generators(&self, ty: &T) -> Vec<Callable> {
        self.callables
            .iter()
            .filter(|&c| {
                let return_type = c.return_type();
                match return_type {
                    None => false,
                    Some(return_ty) => return_ty == ty,
                }
            })
            .cloned()
            .collect()
    }

    pub fn callables_of(&self, ty: &T) -> Vec<&Callable> {
        self.callables
            .iter()
            .filter(|&c| {
                if let Some(parent) = c.parent() {
                    return parent == ty;
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn callables(&self) -> &Vec<Callable> {
        &self.callables
    }

    pub fn callables_mut(&mut self) -> &mut Vec<Callable> {
        &mut self.callables
    }

    pub fn branches(&self) -> &Vec<Branch> {
        &self.branches
    }

    pub fn file_path(&self) -> &Path {
        self.file_path.as_path()
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

impl Visit<'_> for SourceFile {
    fn visit_impl_item_method(&mut self, i: &ImplItemMethod) {
        let container = self.containers.last_mut().unwrap();

        // FIXME here two copies of the same thing are created
        if util::is_constructor(i) {
            let constructor = ConstructorItem::new(i.clone(), container.ty().clone());
            //container.add_callable(Callable::Constructor(constructor.clone()));
            self.callables.push(Callable::Constructor(constructor));
        } else if util::is_method(i) {
            let method = MethodItem::new(i.clone(), container.ty().clone());
            //container.add_callable(Callable::Method(method.clone()));
            self.callables.push(Callable::Method(method));
        } else {
            let func = StaticFnItem::new(i.clone(), container.ty().clone());
            //container.add_callable(Callable::StaticFunction(func.clone()));
            self.callables.push(Callable::StaticFunction(func));
        }

        for it in &i.attrs {
            Visit::visit_attribute(self, it);
        }

        Visit::visit_visibility(self, &i.vis);
        Visit::visit_signature(self, &i.sig);
        Visit::visit_block(self, &i.block);
    }

    fn visit_item_enum(&mut self, i: &ItemEnum) {
        let container = Container::Enum(EnumType::new(
            i.clone(),
            self.visit_state.current_path.to_vec(),
        ));
        self.containers.push(container);

        for it in &i.attrs {
            Visit::visit_attribute(self, it);
        }
        Visit::visit_visibility(self, &i.vis);
        Visit::visit_ident(self, &i.ident);
        Visit::visit_generics(self, &i.generics);
        for el in Punctuated::pairs(&i.variants) {
            let (it, p) = el.into_tuple();
            Visit::visit_variant(self, it);
        }
    }

    fn visit_item_fn(&mut self, i: &ItemFn) {
        for at in &i.attrs {
            Visit::visit_attribute(self, at);
        }

        let func = FunctionItem::new(i.clone());
        self.callables.push(Callable::Function(func));

        Visit::visit_visibility(self, &i.vis);
        Visit::visit_signature(self, &i.sig);
        Visit::visit_block(self, &i.block);
    }

    fn visit_item_mod(&mut self, i: &'_ ItemMod) {
        if i.content.is_none() {
            // The mod is imported
            let import = Import::Mod(Mod::new(i.clone()));
            self.imports.push(import);
        } else {
            // The mod is defined here
            self.visit_state.push_path(&i.ident);
        }

        for it in &i.attrs {
            Visit::visit_attribute(self, it);
        }
        Visit::visit_visibility(self, &i.vis);
        Visit::visit_ident(self, &i.ident);
        if let Some(it) = &i.content {
            for it in &(it).1 {
                Visit::visit_item(self, it);
            }
        };

        if i.content.is_some() {
            self.visit_state.pop_path();
        }
    }

    fn visit_item_struct(&mut self, i: &ItemStruct) {
        let container = Container::Struct(StructType::new(
            i.clone(),
            self.visit_state.current_path.to_vec(),
        ));
        self.containers.push(container);

        for it in &i.attrs {
            Visit::visit_attribute(self, it);
        }
        Visit::visit_visibility(self, &i.vis);
        Visit::visit_ident(self, &i.ident);
        Visit::visit_generics(self, &i.generics);
        Visit::visit_fields(self, &i.fields);
    }

    fn visit_item_use(&mut self, i: &'_ ItemUse) {
        let import = Import::Use(Use::new(i.clone()));
        self.imports.push(import);

        for it in &i.attrs {
            Visit::visit_attribute(self, it);
        }
        Visit::visit_visibility(self, &i.vis);
        Visit::visit_use_tree(self, &i.tree);
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
    pub fn fitness<C: Chromosome>(&self, test_case: &C) -> FitnessValue {
        test_case
            .coverage()
            .get(self)
            .unwrap_or(&FitnessValue::Max)
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

    pub fn set_current_population<C: Chromosome>(&mut self, population: &[C]) {
        let uncovered_branches = self.compute_uncovered_branches(population);
        self.uncovered_branches = uncovered_branches;
    }

    fn compute_uncovered_branches<C: Chromosome>(&self, population: &[C]) -> Vec<Branch> {
        let mut uncovered_branches = vec![];
        for branch in &self.branches {
            let mut covered = false;
            for individual in population {
                if individual.fitness(branch) == FitnessValue::Zero {
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

#[derive(Debug, Clone)]
pub enum Import {
    Use(Use),
    Mod(Mod),
}

impl Import {
    pub fn contains(&self, ty: &T) -> bool {
        let first_ident = ty.path().first().unwrap();
        match self {
            Import::Use(u) => u.ends_with(first_ident),
            Import::Mod(m) => m.ident() == first_ident,
        }
    }

    pub fn path(&self) -> Vec<Ident> {
        match self {
            Import::Use(u) => u.path.clone(),
            Import::Mod(m) => vec![m.ident().clone()],
        }
    }
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

        let us = crate::source::Use::new(import);
    }

    #[test]
    fn test_toml_parsing() {
        let table: Table = toml::from_str(
            r#"
            [package]
            name = "additions"
            version = "0.1.0"
            edition = "2018"
            
            
            [[bin]]
            name = "additions"
            path = "./src/main.rs"
            
            [dependencies]
            lazy_static = "1.4.0"
        "#,
        )
        .unwrap();

        println!("{:?}", table);
        let first_binary = table.get("bin").unwrap();
        assert!(std::matches!(first_binary, Value::Array(..)));
    }

    #[test]
    fn test_project_scanner_directory_tree() {
        let mut source_files = vec![];
        let path =
            PathBuf::from("/Users/tim/Documents/master-thesis/testify/tests/examples");

        ProjectScanner::read_file_tree(path.as_path(), &mut source_files);

        assert_eq!(source_files.len(), 2);
    }

    #[test]
    fn test_main_contains_3_structs() {
        let main_path = Path::new(
            "/Users/tim/Documents/master-thesis/testify/tests/examples/src/main.rs",
        );
        let main = SourceFile::new(&main_path);

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
        let main_path = Path::new(
            "/Users/tim/Documents/master-thesis/testify/tests/examples/src/main.rs",
        );
        let main = SourceFile::new(&main_path);

        assert_eq!(main.containers.len(), 3);

        let path = vec![ident("main"), ident("AreaCalculator")];
        let area_calculator_ty = T::new(path);

        assert_eq!(main.callables_of(&area_calculator_ty).len(), 3);
    }

    #[test]
    fn test_dependency_has_full_path() {
        let main_path = Path::new(
            "/Users/tim/Documents/master-thesis/testify/tests/examples/src/main.rs",
        );
        let main = SourceFile::new(&main_path);

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
            .find(|&p| p.ty() == &dependency_ty);

        assert!(dep_param.is_some());
    }

    #[test]
    fn test_nested_dependency() {
        let main_path = Path::new(
            "/Users/tim/Documents/master-thesis/testify/tests/examples/src/main.rs",
        );
        let main = SourceFile::new(&main_path);

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
            ident("NestedStruct")
        ]);

        let dep_param = dep_method
            .unwrap()
            .params()
            .iter()
            .find(|&p| p.ty() == &nested_dependency_ty);

        assert!(dep_param.is_some())
    }

}
