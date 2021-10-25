#![feature(rustc_private)]
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate lazy_static;

pub mod algorithm;
pub mod chromosome;
pub mod generators;
pub mod operators;
pub mod parser;
pub mod selection;
pub mod source;
pub mod util;
pub mod instrument;
pub mod test;
pub mod monitor;
pub mod compiler;
pub mod analysis;
pub mod fs_util;
pub mod serialization;
