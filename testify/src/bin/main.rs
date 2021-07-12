use testify::algorithm::MOSA;
use testify::chromosome::{ChromosomeGenerator, Chromosome};
use clap::{Clap};
use instrument::instrument;

struct Generator {


}

impl ChromosomeGenerator for Generator {
    type C = BitString;

    fn generate(&self) -> Box<Self::C> {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct BitString {

}

impl Chromosome for BitString {
    fn mutate(&self) -> Self {
        todo!()
    }

    fn calculate_fitness(&self) -> f64 {
        todo!()
    }

    fn crossover(&self, other: &Self) -> (Self, Self) {
        todo!()
    }
}

#[derive(Clap)]
struct CliOpts {
    #[clap(short, long)]
    file: String
}

fn main() {
    //let generator = Generator {};
    //MOSA::new().chromosome_generator(generator).population_size(40).run();
    let opts: CliOpts = CliOpts::parse();
    println!("Instrumenting {}", opts.file);

    instrument(opts.file);


}
