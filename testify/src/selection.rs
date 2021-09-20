use crate::chromosome::Chromosome;

pub trait Selection {
    type C: Chromosome;

    fn apply(population: &Vec<Self::C>) -> Self::C;
}
