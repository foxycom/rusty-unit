#![feature(rustc_private)]
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process;
use std::rc::Rc;
use std::time::Duration;

use clap::Clap;
use rustc_driver::args::Error::IOError;
use generation::compiler;
use generation::source::{Project, ProjectScanner};
use instrumentation::INSTRUMENTATION_LOG_PATH;


#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    path: String,
}

struct AnalysisError {
    msg: String
}

fn main() {

    let opts: CliOpts = CliOpts::parse();

    let mut project = ProjectScanner::open(&opts.path);
    project.clear_build_dirs();
    project.make_copy();
    if let Err(AnalysisError { msg }) = analyze_project(&project) {
        eprintln!("{}", msg);
        panic!();
    }


    //compiler::start(project);


    /*let mut instrumenter = Instrumenter::new();
    instrumenter.instrument(&mut project);
    project.write();
    project.run_tests();*/

    /*let mut source_file = Rc::new(source_file);

    let population_size = 20usize;

    let branches = source_file.branches();
    let branch_manager = BranchManager::new(branches);
    let branch_manager_rc = Rc::new(RefCell::new(branch_manager));

    let mutation = Rc::new(BasicMutation::new(source_file.clone(), branch_manager_rc.clone()));
    let crossover = Rc::new(SinglePointCrossover::new());
    let rank_selection = Rc::new(RankSelection::new(branch_manager_rc.clone()));
    let offspring_generator = Rc::new(OffspringGenerator::new(
        rank_selection.clone(),
        mutation.clone(),
        crossover.clone(),
        0.0,
        0.2,
    ));
    let initial_population: Vec<TestCase> = (0..population_size)
        .map(|_| TestCase::random(source_file.clone()))
        .collect();

    let res = DynaMOSA::new(
        20,
        0.2,
        0.3,
        0.00001,
        10,
        branch_manager_rc.clone(),
        offspring_generator.clone(),
    )
    .run(source_file.as_ref().clone(), initial_population);
    match res {
        Ok(TestSuite {
                 uncovered_branches,
                 coverage,
                 tests,
             }) => {
            println!(
                "\nUncovered branches: {:?}\nOverall branch coverage: {}",
                uncovered_branches, coverage
            );
            //source_file.add_tests(&tests, false);
        }
        Err(err) => {
            println!("{}", err);
        }
    }*/
}

fn analyze_project(project: &Project) -> Result<(), AnalysisError> {
    if let Err(err) = std::fs::remove_file(INSTRUMENTATION_LOG_PATH) {
        match err.kind() {
            ErrorKind::NotFound => {}
            _ => panic!("{}", err)
        }
    }

    let out = process::Command::new("cargo")
        .env("RUSTC_WRAPPER", "/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation")
        .arg("+nightly-aarch64-apple-darwin")
        .arg("rustc")
        .arg("--")
        .arg("--testify-stage=analyze")
        .current_dir(project.output_root())
        .output()
        .unwrap();

    let output = String::from_utf8(out.stdout).unwrap();
    let output = output.trim();

    if !out.status.success() {
        let err = String::from_utf8(out.stderr).unwrap();
        let err = AnalysisError {msg: format!("Analysis failed!\n{}", err)};
        return Err(err);
    }

    Ok(())
}

pub struct Client {
    connection: TcpStream,
}

impl Client {
    pub fn get(&mut self) -> HashMap<usize, Vec<String>> {
        self.connection.write(b"get").unwrap();
        println!("Started reading");

        let mut data = [0 as u8; 1024];
        match self.connection.read(&mut data) {
            Ok(size) => {
                if size == 0 {
                    // connection closed
                    panic!("Connection closed");
                } else {
                    let response = String::from_utf8_lossy(&data[0..size]);
                    println!("{}", response);

                    let traces: HashMap<usize, Vec<String>> = serde_json::from_str(response.as_ref()).unwrap();
                    return traces;
                }
            }
            Err(_) => {
                panic!("Could not read traces from server");
            }
        }
    }

    pub fn new() -> Self {
        let connection = match TcpStream::connect("localhost:3333") {
            Ok(stream) => {
                stream
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                panic!()
            }
        };
        Client { connection }
    }
}

