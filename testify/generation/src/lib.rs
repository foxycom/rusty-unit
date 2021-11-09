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


pub mod parser;
pub mod test;
pub mod analysis;
pub mod types;
pub mod algorithm;
pub mod fitness;
pub mod branch;
pub mod chromosome;
pub mod util;
pub mod source;
pub mod generators;
pub mod fs_util;
pub mod operators;

