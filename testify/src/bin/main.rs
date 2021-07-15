use testify::algorithm::MOSA;
use testify::chromosome::{ChromosomeGenerator, Chromosome, TestCaseGenerator};
use clap::{Clap};
use instrument::instrument;
use testify::test_writer;

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String
}

fn main() {
    let opts: CliOpts = CliOpts::parse();

    let generator = TestCaseGenerator::new(&opts.file);
    instrument(&opts.file);

    //MOSA::new(generator).population_size(40).run();
    let test_case = generator.generate();
    test_writer::write(&test_case);

}
