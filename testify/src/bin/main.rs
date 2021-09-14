use testify::algorithm::{MOSA};
use testify::chromosome::{TestCaseGenerator, StatementGenerator};
use clap::{Clap};
use std::rc::Rc;
use testify::operators::{BasicMutation, RankSelection, BasicCrossover};
use testify::generators::TestIdGenerator;
use std::cell::RefCell;
use testify::source::{SourceFile, BranchManager};
use petgraph::Graph;
use petgraph::prelude::GraphMap;

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String
}

fn main() {
    let opts: CliOpts = CliOpts::parse();

    let test_id_generator = Rc::new(RefCell::new(TestIdGenerator::new()));
    let mut source_file = SourceFile::new(&opts.file);
    source_file.instrument();

    let branches = source_file.branches();
    let branch_manager = BranchManager::new(branches);
    let branch_manager_rc = Rc::new(RefCell::new(branch_manager));
    let statement_generator = Rc::new(StatementGenerator::new(source_file.clone()));

    let mutation = BasicMutation::new(statement_generator.clone(), branch_manager_rc.clone());
    let crossover = BasicCrossover::new();
    let rank_selection = RankSelection::new(branch_manager_rc.clone());
    let generator = TestCaseGenerator::new(statement_generator.clone(), branch_manager_rc.clone(), mutation.clone(), crossover.clone(), test_id_generator.clone());
    let res = MOSA::new(generator, rank_selection, branch_manager_rc, test_id_generator.clone())
        .population_size(20)
        .generations(10)
        .run(source_file.clone());
    match res {
        None => {
            println!("Execution failed");
        }
        Some((uncovered_branches, coverage)) => {
            println!("\nUncovered branches: {:?}\nOverall branch coverage: {}", uncovered_branches, coverage);
        }
    }

    // Restore the original file
    //source_file.restore();
}
