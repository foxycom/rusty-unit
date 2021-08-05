use testify::algorithm::{MOSA, PreferenceSorter};
use testify::chromosome::{ChromosomeGenerator, Chromosome, TestCaseGenerator};
use clap::{Clap};
use std::io::Error;
use testify::instr::instr::Instrumenter;
use std::rc::Rc;
use testify::operators::{BasicMutation, RankSelection, BasicCrossover};
use testify::generators::TestIdGenerator;
use std::cell::RefCell;
use testify::instr::data::BranchManager;

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String
}

fn main() {
    let opts: CliOpts = CliOpts::parse();

    let test_id_generator = Rc::new(RefCell::new(TestIdGenerator::new()));
    let mut instrumenter = Instrumenter::new();
    let branches = instrumenter.instrument(&opts.file).to_owned();
    let branch_manager = BranchManager::new(&branches);
    let branch_manager_rc = Rc::new(RefCell::new(branch_manager));
    let mutation = BasicMutation::new(branch_manager_rc.clone());
    let crossover = BasicCrossover::new();
    let rank_selection = RankSelection::new(branch_manager_rc.clone());
    let mut generator = TestCaseGenerator::new(branch_manager_rc.clone(), mutation.clone(), crossover.clone(), test_id_generator.clone());
    let res = MOSA::new(generator, rank_selection, branch_manager_rc, test_id_generator.clone())
        .population_size(20)
        .generations(10)
        .run();
    match res {
        None => {
            println!("Execution failed");
        }
        Some((uncovered_branches, coverage)) => {
            println!("\nUncovered branches: {:?}\nOverall branch coverage: {}", uncovered_branches, coverage);
        }
    }
}
