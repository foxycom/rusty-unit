use std::os::unix::prelude::ExitStatusExt;
use std::process::Command;
use clap::{Parser};

#[derive(Parser, Debug)]
struct CLI {
    #[clap(short, long)]
    pub name: String,

    #[clap(short, long)]
    pub root: String
}

pub const RUSTC_WRAPPER: &'static str = "/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation";

fn main() {
    let args = CLI::parse();
    let output = Command::new("cargo")
        .env("RUSTC_WRAPPER", RUSTC_WRAPPER)
        .env("RU_CRATE_ROOT", &args.root)
        .env("RU_CRATE_NAME", &args.name)
        .arg("+nightly-aarch64-apple-darwin")
        .arg("build")
        .current_dir(&args.root)
        .output();

    let out = String::from_utf8(output.stdout).unwrap();
    println!("{:?}", output.status.into_raw());
}