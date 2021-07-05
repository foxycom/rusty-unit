use crate::chromosome::Chromosome;

pub trait Crossover {
    type C: Chromosome;

    fn apply(&self, a: &Self::C, b: &Self::C) -> (Self::C, Self::C);
}