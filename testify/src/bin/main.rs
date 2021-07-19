use testify::algorithm::MOSA;
use testify::chromosome::{ChromosomeGenerator, Chromosome, TestCaseGenerator};
use clap::{Clap};
use testify::tests::writer::{TestWriter, ModuleRegistrar};
use testify::tests::runner::TestRunner;
use std::io::Error;
use testify::instr::instr::Instrumenter;
use std::rc::Rc;
use testify::operators::BasicMutation;

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String
}

fn main() {
    let opts: CliOpts = CliOpts::parse();

    let mut instrumenter = Instrumenter::new();
    let branches = Rc::new(instrumenter.instrument(&opts.file).to_owned());
    let mutation = Rc::new(BasicMutation::new(branches.clone()));
    let mut generator = TestCaseGenerator::new(&opts.file, branches, mutation.clone());
    MOSA::new(generator, branches.clone()).population_size(40).run();

}
