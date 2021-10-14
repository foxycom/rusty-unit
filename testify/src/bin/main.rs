use std::cell::RefCell;
use std::rc::Rc;

use clap::Clap;

use testify::algorithm::{DynaMOSA, OffspringGenerator, TestSuite};
use testify::chromosome::{Chromosome, TestCase};
use testify::instrument::Instrumenter;
use testify::operators::{BasicMutation, RankSelection, SinglePointCrossover};
use testify::source::{BranchManager, SourceFile};

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String,
}

fn main() {
    let opts: CliOpts = CliOpts::parse();

    let mut source_file = SourceFile::new(&opts.file);
    let mut instrumenter = Instrumenter::new();
    instrumenter.instrument(&source_file);

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

