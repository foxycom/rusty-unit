use crate::chromosome::Chromosome;

pub trait Selection {
    type C: Chromosome;

    fn apply(&self, population: &Vec<Self::C>) -> Self::C;
}


