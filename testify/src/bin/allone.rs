use testify::chromosome::{Chromosome, ChromosomeGenerator};
use testify::mutation::Mutation;
use std::cmp::max;
use rand::{random, Rng};
use testify::crossover::Crossover;
use testify::algorithm::{SimulatedAnnealing, CoolingStrategy, TemperatureStrategy};
use std::ops::Deref;

#[derive(Clone, Debug)]
struct OnesMutation;

impl Mutation for OnesMutation {
    type C = Ones;

    fn apply(&self, chromosome: &Self::C) -> Self::C {
        let mut mutated = chromosome.clone();
        let mut ones = &mut mutated.ones;
        let threshold = 1.0 / ones.len() as f64;

        for i in 0..ones.len() {
            let p = rand::random::<f64>();
            if p < threshold {
                ones[i] = (ones[i] + 1) % 2;
            }
        }

        mutated
    }
}

#[derive(Clone, Debug)]
struct OnesCrossover;

impl Crossover for OnesCrossover {
    type C = Ones;

    fn apply(&self, a: &Self::C, b: &Self::C) -> (Self::C, Self::C) {
        let mut rng = rand::thread_rng();

        let cut_point = rng.gen_range(0..a.ones.len());
        let mut child_a = a.clone();
        let mut child_b = b.clone();

        for i in cut_point..a.ones.len() {
            child_a.ones[i] = b.ones[i];
            child_b.ones[i] = a.ones[i];
        }
        (child_a, child_b)
    }
}

#[derive(Debug, Clone)]
struct Ones {
    ones: Vec<u8>,
    mutation: Box<OnesMutation>,
    crossover: Box<OnesCrossover>,
}

impl Chromosome for Ones {
    fn mutate(&self) -> Self {
        self.mutation.apply(self)
    }

    fn fitness(&self) -> f64 {
        self.ones.len() as f64 - self.ones.iter().fold(0, |acc, &x| acc + x) as f64
    }

    fn crossover(&self, other: &Self) -> (Self, Self) {
        self.crossover.apply(self, other)
    }
}

struct Generator {
    size: usize,
    mutation: OnesMutation,
    crossover: OnesCrossover,
}

impl Generator {
    fn new(size: usize, mutation: OnesMutation, crossover: OnesCrossover) -> Generator {
        Generator { size, mutation, crossover }
    }
}

impl ChromosomeGenerator for Generator {
    type C = Ones;

    fn generate(&self) -> Box<Self::C> {
        let ones: Vec<u8> = (0..self.size).map(|i| if random::<f64>() < 0.5 {
            0
        } else {
            1
        }).collect();
        Box::new(Ones {
            ones,
            mutation: Box::new(self.mutation.clone()),
            crossover: Box::new(self.crossover.clone()),
        })
    }
}

fn main() {


    let mutation = OnesMutation;
    let crossover = OnesCrossover;
    let generator = Generator::new(100, mutation, crossover);
    let init = generator.generate();

    let (start, t0) = TemperatureStrategy::new(0.6, 0.1, 0.0, 1.0).init_temperature(*init);
    let mut cooling_strategy = CoolingStrategy::new();

    let result = SimulatedAnnealing::new(cooling_strategy)
        .initial_chromosome(start)
        .max_transformations(1000)
        .run();
    println!("{:?}", result);
}