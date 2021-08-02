use testify::algorithm::{MOSA, PreferenceSorter};
use testify::chromosome::{ChromosomeGenerator, Chromosome, TestCaseGenerator};
use clap::{Clap};
use std::io::Error;
use testify::instr::instr::Instrumenter;
use std::rc::Rc;
use testify::operators::{BasicMutation, RankSelection};
use testify::generators::TestIdGenerator;
use std::cell::RefCell;

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String
}

fn main() {
    let opts: CliOpts = CliOpts::parse();

    let test_id_generator = Rc::new(RefCell::new(TestIdGenerator::new()));
    let mut instrumenter = Instrumenter::new();
    let objectives = instrumenter.instrument(&opts.file).to_owned();
    let mutation = BasicMutation::new(objectives.clone());
    let rank_selection = RankSelection::new(objectives.clone(), );
    let mut generator = TestCaseGenerator::new(objectives.clone(), mutation.clone(), test_id_generator.clone());
    let res = MOSA::new(generator, rank_selection, objectives.clone(), test_id_generator.clone())
        .population_size(10)
        .generations(10)
        .run();
    match res {
        None => {
            println!("Execution failed");
        }
        Some((uncovered_branches, coverage)) => {
            println!("Uncovered branches: {:?}", uncovered_branches);
            println!("Overall branch coverage: {}", coverage);
        }
    }
}
