use std::fmt::Debug;

pub trait Chromosome: Clone + Debug {
    fn mutate(&self) -> Self;

    fn calculate_fitness(&self) -> f64;

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized;
}

pub trait ChromosomeGenerator {
    type C: Chromosome;

    fn generate(&self) -> Box<Self::C>;
}