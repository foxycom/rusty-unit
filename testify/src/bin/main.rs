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
use std::fs::OpenOptions;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process;
use std::rc::Rc;
use std::time::Duration;

use clap::Clap;
use generation::chromosome::{Chromosome, TestCase};
use generation::parser::{HirParser, MirParser};
use generation::source::{Project, ProjectScanner};
use instrumentation::{HIR_LOG_PATH, MIR_LOG_PATH};
use rustc_driver::args::Error::IOError;

static LOG_DIR: &'static str = "/Users/tim/Documents/master-thesis/testify/log";

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    path: String,
}

struct AnalysisError {
    msg: String,
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

    std::fs::create_dir_all(LOG_DIR);
    let hir_analysis = HirParser::parse(HIR_LOG_PATH);
    let mir_analysis = MirParser::parse(MIR_LOG_PATH);

    let callables_log_path = PathBuf::from(LOG_DIR).join("callables.txt");
    let mut callables_log = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(callables_log_path.as_path())
        .unwrap();
    hir_analysis.callables().iter().for_each(|c| {
        callables_log.write_all(format!("{:?}\n", c).as_bytes());
    });

    let mut bodies_log_path = PathBuf::from(LOG_DIR).join("bodies.txt");
    let mut bodies_log = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(bodies_log_path.as_path())
        .unwrap();

    mir_analysis.bodies.iter().for_each(|b| {
        bodies_log.write_all(format!("{:?}\n", b).as_bytes());
    });

    let hir_analysis = Rc::new(hir_analysis);
    /*let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./tests.rs")
        .unwrap();
    (0..10).for_each(|_| {
        let test_case = TestCase::random(hir_analysis.clone());
        let test_str = test_case.to_string();
        file.write_fmt(format_args!("{}\n", test_str)).unwrap();
    })*/

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
    if let Err(err) = std::fs::remove_file(MIR_LOG_PATH) {
        match err.kind() {
            ErrorKind::NotFound => {}
            _ => panic!("{}", err),
        }
    }

    if let Err(err) = std::fs::remove_file(HIR_LOG_PATH) {
        match err.kind() {
            ErrorKind::NotFound => {}
            _ => panic!("{}", err),
        }
    }

    let out = process::Command::new("cargo")
        .env(
            "RUSTC_WRAPPER",
            "/Users/tim/Documents/master-thesis/testify/target/debug/instrumentation",
        )
        .env(
            "TESTIFY_FLAGS",
            &format!(
                "--stage=analyze --crate={} --crate-name={}",
                project.project_root().to_str().unwrap(),
                project.crate_name()
            ),
        )
        .arg("+nightly-aarch64-apple-darwin")
        .arg("build")
        .current_dir(project.output_root())
        .output()
        .unwrap();

    let output = String::from_utf8(out.stdout).unwrap();
    let output = output.trim();

    let log_path = PathBuf::from(LOG_DIR).join("analysis.log");

    let mut log_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(log_path.as_path())
        .unwrap();
    log_file.write_all(output.as_bytes());

    if !out.status.success() {
        let err = String::from_utf8(out.stderr).unwrap();
        let err = AnalysisError {
            msg: format!("Analysis failed!\n{}", err),
        };
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

                    let traces: HashMap<usize, Vec<String>> =
                        serde_json::from_str(response.as_ref()).unwrap();
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
            Ok(stream) => stream,
            Err(e) => {
                println!("Failed to connect: {}", e);
                panic!()
            }
        };
        Client { connection }
    }
}
