#![feature(rustc_private)]
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_ast;
extern crate rustc_serialize;
#[macro_use]
extern crate lazy_static;

pub mod util;
pub mod source;
pub mod types;
pub mod analysis;
pub mod branch;
pub const LOG_PATH: &'static str = "/Users/tim/Documents/master-thesis/tmp/testify";
pub const MIR_LOG_PATH: &'static str = "mir.log";
pub const HIR_LOG_PATH: &'static str = "hir.json";
pub const INSTRUMENTED_MIR_LOG_PATH: &'static str = "instrumented-mir.log";