use testify::algorithm::MOSA;
use testify::chromosome::{ChromosomeGenerator, Chromosome};

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

    fn crossover(&self, other: impl Chromosome) -> Self {
        todo!()
    }
}

fn main() {
    let generator = Generator {};
    MOSA::new().chromosome_generator(generator).population_size(40).run();

}
