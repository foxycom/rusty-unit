#![feature(rustc_private)]
mod data_structures;
mod hir;
mod mir;
mod writer;
mod util;

#[macro_use]
extern crate lazy_static;

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_index;

use crate::hir::hir_analysis;
use crate::mir::{CUSTOM_OPT_MIR_ANALYSIS, CUSTOM_OPT_MIR_INSTRUMENTATION};
use generation::source::{Project, ProjectScanner};
use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;
use rustc_interface::{Config, Queries};
use rustc_middle::ty::TyCtxt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process;
use std::process::exit;
use crate::util::{get_crate_root, get_testify_flags, get_stage, Stage, get_cut_name};

pub struct CompilerCallbacks {
    stage: Stage,
}

impl CompilerCallbacks {
    pub fn new(stage: Stage) -> Self {
        CompilerCallbacks { stage }
    }
}

fn enter_with_fn<'tcx, TyCtxtFn>(queries: &'tcx rustc_interface::Queries<'tcx>, enter_fn: TyCtxtFn)
where
    TyCtxtFn: Fn(TyCtxt),
{
    queries.global_ctxt().unwrap().peek_mut().enter(enter_fn);
}

impl rustc_driver::Callbacks for CompilerCallbacks {
    fn config(&mut self, _config: &mut Config) {
        match &self.stage {
            Stage::Analyze => {
                _config.override_queries = Some(|session, local, external| {
                    local.optimized_mir = CUSTOM_OPT_MIR_ANALYSIS;
                });
            }
            Stage::Instrument => {
                _config.override_queries = Some(|session, local, external| {
                    local.optimized_mir = CUSTOM_OPT_MIR_INSTRUMENTATION;
                });
            }
        }
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &Compiler,
        _queries: &'tcx Queries<'tcx>,
    ) -> Compilation {
        if self.stage == Stage::Analyze {
            enter_with_fn(_queries, hir_analysis);
        }
        Compilation::Continue
    }
}

impl From<&str> for Stage {
    fn from(stage_str: &str) -> Self {
        if stage_str == "analyze" {
            Stage::Analyze
        } else if stage_str == "instrument" {
            Stage::Instrument
        } else {
            panic!("Unknown stage: {}", stage_str);
        }
    }
}

pub fn arg_value<'a>(
    args: impl IntoIterator<Item = &'a String>,
    find_arg: &str,
    pred: impl Fn(&str) -> bool,
) -> Option<&'a str> {
    let mut args = args.into_iter().map(String::as_str);

    while let Some(arg) = args.next() {
        let arg: Vec<_> = arg.splitn(2, '=').collect();
        if arg.get(0) != Some(&find_arg) {
            continue;
        }

        let value = arg.get(1).cloned().or_else(|| args.next());
        if value.as_ref().map_or(false, |p| pred(p)) {
            return value;
        }
    }
    None
}

pub fn sysroot() -> String {
    let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();

    let sysroot = String::from_utf8(out.stdout).unwrap();
    let sysroot = sysroot.trim();
    sysroot.to_string()
}

pub fn set_source_files(project: &Project) {
    let mut source_file_map = hir::SOURCE_FILE_MAP.lock().unwrap();
    for (pos, path) in project.rel_file_names().iter().enumerate() {
        source_file_map.insert(path.to_path_buf(), pos);
    }
    drop(source_file_map);
}

pub fn get_compiler_args(args: &[String]) -> Vec<String> {
    let have_sys_root = arg_value(args, "--sysroot", |_| true).is_some();
    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this/
    let wrapper_mode = Path::new(&args[1]).file_stem() == Some("rustc".as_ref());

    let mut rustc_args: Vec<_>;

    if wrapper_mode {
        // we still want to be able to invoke it normally though
        rustc_args = args
            .iter()
            .skip(1)
            .map(|s| s.to_string())
            .collect();
    } else {
        rustc_args = args
            .iter()
            .skip(1)
            .take_while(|s| *s != "--")
            .map(|s| s.to_string())
            .collect();
        rustc_args.insert(0, "".to_owned());
    }

    // this conditional check for the --sysroot flag is there so users can call
    // `clippy_driver` directly
    // without having to pass --sysroot or anything
    if !have_sys_root {
        rustc_args.push("--sysroot".to_owned());
        rustc_args.push(sysroot());
    }
    /*rustc_args.push("--allow".to_owned());
    rustc_args.push("dead_code".to_owned());*/
    rustc_args.push("--allow".to_owned());
    rustc_args.push("deprecated".to_owned());
    /*rustc_args.push("--allow".to_owned());
    rustc_args.push("unused".to_owned());*/

    rustc_args
}

fn run_rustc() -> Result<(), i32> {
    let std_env_args: Vec<String> = std::env::args().collect();

    let testify_env_flags = get_testify_flags();
    let stage = get_stage(&testify_env_flags);
    let crate_root = get_crate_root(&testify_env_flags);
    let cut_name = get_cut_name(&testify_env_flags);

    let project = ProjectScanner::open(&crate_root);
    set_source_files(&project);
    let rustc_args = get_compiler_args(&std_env_args);
    pass_to_rustc(&rustc_args, stage);
    return Ok(());
}

pub fn pass_to_rustc(rustc_args: &[String], stage: Stage) {
    let mut callbacks = CompilerCallbacks::new(stage);

    let err = rustc_driver::RunCompiler::new(&rustc_args, &mut callbacks).run();
    if err.is_err() {
        eprintln!("Error while compiling dependency");
        std::process::exit(-1);
    }
}

fn main() {
    exit(run_rustc().err().unwrap_or(0))
}
