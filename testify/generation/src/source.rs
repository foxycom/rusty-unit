use crate::{util};
use dircpy_stable::copy_dir;
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use rustc_middle::ty::TyCtxt;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io, process};
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::token::Else;
use syn::visit::Visit;
use syn::{File, Item, ItemUse, UseTree, ItemMod};
use toml::value::Table;
use toml::Value;

pub const OUTPUT_ROOT: &'static str = "/Users/tim/Documents/master-thesis/evaluation/current";

pub struct AnalysisError;

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

#[derive(Debug)]
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
    fn executables(toml: &Table, source_dir: &Path) -> Vec<FileType> {
        match toml.get("bin") {
            None => {
                // Default path
                let path = PathBuf::from("src/bin.rs");
                // TODO set default name from the project name
                let name = "".to_owned();

                let absolute_path = source_dir.join("bin.rs");
                if absolute_path.exists() {
                    vec![FileType::Executable(name, path)]
                } else {
                    vec![]
                }
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

    fn library(toml: &Table, source_dir: &Path) -> FileType {
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
        let source_dir = project_root.join("src");

        let toml_path = project_root.join("Cargo.toml");
        let toml_table = ProjectScanner::parse_toml(toml_path.as_path());

        let source_dir = project_root.join("src");
        let mut source_files = vec![];

        let executables = TomlScanner::executables(&toml_table, source_dir.as_path());
        let lib = TomlScanner::library(&toml_table, source_dir.as_path());
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
        )
        .unwrap();

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

#[derive(Debug)]
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

    pub fn output_root(&self) -> &Path {
        self.output_root.as_path()
    }

    pub fn crate_name(&self) -> &str {
        &self.toml.package_name
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    file_path: PathBuf,
    output_path: PathBuf,
    file_type: FileType,
}

impl SourceFile {
    pub fn new(src_path: &Path, output_path: &Path, file_type: FileType) -> SourceFile {
        let mut source_file = SourceFile {
            output_path: output_path.to_path_buf(),
            file_path: src_path.to_path_buf(),
            file_type,
        };

        source_file
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
