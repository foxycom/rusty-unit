use std::fmt::{Debug, Display, Formatter, Error};
use syn::{Stmt, Item, ItemFn};
use std::cmp::Ordering;
use quote::ToTokens;
use crate::analyze::analyze_src;
use rand::{random, thread_rng, Rng};
use crate::data::Target;
use proc_macro2::{Ident, Span};

const TEST_FN_NAME: &'static str = "testify_test_target_fn";

pub trait Chromosome: Clone + Debug {
    fn mutate(&self) -> Self;

    fn calculate_fitness(&self) -> f64;

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized;
}

pub trait ChromosomeGenerator {
    type C: Chromosome;

    fn generate(&self) -> Self::C;
}


#[derive(Clone, Debug)]
pub struct TestCase {
    target: Target,
    stmts: Vec<Stmt>,
    crowding_distance: f64,
}

impl TestCase {
    pub fn new(target: Target, stmts: Vec<Stmt>) -> Self {
        TestCase { target, stmts, crowding_distance: 0.0 }
    }

    pub fn set_crowding_distance(&mut self, crowding_distance: f64) {
        self.crowding_distance = crowding_distance;
    }
}

impl TestCase {
    pub fn stmts(&self) -> &Vec<Stmt> {
        &self.stmts
    }
    pub fn crowding_distance(&self) -> f64 {
        self.crowding_distance
    }

    pub fn target(&self) -> &Target {
        &self.target
    }

    pub fn to_syn(&self) -> Item {
        let ident = Ident::new(TEST_FN_NAME, Span::call_site());
        let stmts = &self.stmts;
        let test: Item = syn::parse_quote! {
            fn #ident() {
                #(#stmts)*
            }
        };
        test
    }
}

impl Display for TestCase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let syn_item = self.to_syn();
        let token_stream = syn_item.to_token_stream();
        write!(f, "{}", token_stream.to_string())
    }
}

impl Chromosome for TestCase {
    fn mutate(&self) -> Self {
        println!("Cloning chromosome");
        self.clone()
    }

    fn calculate_fitness(&self) -> f64 {
        println!("Calculating fitness");
        10.0
    }

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized {
        println!("Doing crossover");
        (self.clone(), self.clone())
    }
}

#[derive(Debug)]
pub struct TestCaseGenerator {
    targets: Vec<Target>,
}

impl TestCaseGenerator {
    pub fn new(path: &str) -> TestCaseGenerator {
        let targets = analyze_src(path);
        TestCaseGenerator {
            targets
        }
    }
}

impl ChromosomeGenerator for TestCaseGenerator {
    type C = TestCase;

    fn generate(&self) -> Self::C {
        let mut rng = thread_rng();
        let rand = rng.gen_range((0..self.targets.len()));
        let target = self.targets.get(rand).unwrap();
        let ident = &target.target_fn().sig.ident;
        let stmt = syn::parse_quote! {
            #ident();
        };
        TestCase::new(target.clone(), vec![stmt])
    }
}

#[cfg(test)]
mod tests {
    use crate::chromosome::TestCase;
}