use clap::Clap;
use petgraph::prelude::GraphMap;
use petgraph::Graph;
use std::cell::RefCell;
use std::rc::Rc;
use testify::algorithm::{DynaMOSA, TestSuite, MOSA};
use testify::chromosome::{Chromosome, StatementGenerator, TestCase, TestCaseGenerator};
use testify::generators::TestIdGenerator;
use testify::operators::{SinglePointCrossover, BasicMutation, RankSelection};
use testify::source::{BranchManager, SourceFile};

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String,
}

fn main<C: Chromosome>() {
    let opts: CliOpts = CliOpts::parse();

    let mut source_file = SourceFile::new(&opts.file);
    source_file.instrument();

    let population_size = 20usize;

    let branches = source_file.branches();
    let branch_manager = BranchManager::new(branches);
    let branch_manager_rc = Rc::new(RefCell::new(branch_manager));

    let mutation = Rc::new(BasicMutation::new(branch_manager_rc.clone()));
    let crossover = Rc::new(SinglePointCrossover::new());
    let rank_selection = RankSelection::new(branch_manager_rc.clone());

    let initial_population: Vec<C> = (0..population_size)
        .map(|_| TestCase::random(&source_file, mutation.clone(), crossover.clone()))
        .collect();

    let res = DynaMOSA::new(
        20,
        0.2,
        0.3,
        0.00001,
        10,
        branch_manager_rc.clone(),
    ).run(source_file.clone(), initial_population);
    match res {
        None => {
            println!("Execution failed");
        }
        Some(TestSuite {
            uncovered_branches,
            coverage,
            tests,
        }) => {
            println!(
                "\nUncovered branches: {:?}\nOverall branch coverage: {}",
                uncovered_branches, coverage
            );
            source_file.add_tests(&tests, false);
        }
    }
}

fn generate_random_population<C: Chromosome>(source_file: &SourceFile, population_size: usize) -> Vec<C> {
    let mut population = Vec::new();
    for _ in 0..population_size {
        population.push(TestCase::random(source_file, ));
    }

    population
}
