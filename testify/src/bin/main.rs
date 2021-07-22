use testify::algorithm::{MOSA, PreferenceSorter};
use testify::chromosome::{ChromosomeGenerator, Chromosome, TestCaseGenerator};
use clap::{Clap};
use testify::io::writer::{TestWriter, ModuleRegistrar};
use testify::io::runner::TestRunner;
use std::io::Error;
use testify::instr::instr::Instrumenter;
use std::rc::Rc;
use testify::operators::{BasicMutation, RankSelection};

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String
}

fn main() {
    let opts: CliOpts = CliOpts::parse();

    let mut instrumenter = Instrumenter::new();
    let objectives = instrumenter.instrument(&opts.file).to_owned();
    let mutation = BasicMutation::new(objectives.clone());
    let rank_selection = RankSelection::new(objectives.clone(), );
    let mut generator = TestCaseGenerator::new(objectives.clone(), mutation.clone(), 0);
    MOSA::new(generator, rank_selection, objectives.clone()).population_size(5).run();
}
