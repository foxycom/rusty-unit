use crate::chromosome::Chromosome;

pub trait Mutation {
    type C: Chromosome;

    fn apply(&self, chromosome: &Self::C) -> Self::C;
}