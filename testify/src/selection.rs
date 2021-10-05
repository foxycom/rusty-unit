use crate::chromosome::Chromosome;

pub trait Selection {
    type C;

    fn apply(&self, population: &[Self::C]) -> Self::C;
}


