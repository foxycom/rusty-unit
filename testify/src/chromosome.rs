use std::fmt::{Debug, Display, Formatter, Error};
use syn::{Stmt, Item, ItemFn, FnArg, PatType, Type, Expr};
use std::cmp::Ordering;
use quote::ToTokens;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use crate::generators::InputGenerator;
use crate::tests::writer::{TestWriter, ModuleRegistrar};
use crate::tests::runner::TestRunner;
use crate::instr::data::{Branch};
use crate::analyze::analyze_src;
use crate::operators::BasicMutation;
use std::rc::Rc;


pub trait Chromosome: Clone + Debug {
    fn mutate(&self) -> Self;

    fn fitness(&self) -> f64;

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized;
}

pub trait ChromosomeGenerator {
    type C: Chromosome;

    fn generate(&self) -> Self::C;
}


#[derive(Clone, Debug)]
pub struct TestCase {
    objective: Branch,
    stmts: Vec<Stmt>,
    results: HashMap<u64, f64>,
    mutation: Rc<BasicMutation>
}

impl TestCase {
    pub const TEST_FN_NAME: &'static str = "testify_test_target_fn";

    pub fn new(target: Branch, stmts: Vec<Stmt>, mutation: Rc<BasicMutation>) -> Self {
        TestCase { objective: target, stmts, results: HashMap::new(), mutation }
    }

    pub fn set_crowding_distance(&mut self, crowding_distance: f64) {
        self.crowding_distance = crowding_distance;
    }

    pub fn stmts(&mut self) -> &mut Vec<Stmt> {
        &mut self.stmts
    }
    pub fn crowding_distance(&self) -> f64 {
        self.crowding_distance
    }

    pub fn target(&self) -> &Branch {
        &self.objective
    }

    pub fn to_syn(&self) -> Item {
        let ident = Ident::new(TestCase::TEST_FN_NAME, Span::call_site());
        let stmts = &self.stmts;
        let test: Item = syn::parse_quote! {
            #[test]
            fn #ident() {
                #(#stmts)*
            }
        };
        test
    }

    pub fn results(&self) -> &HashMap<u64, f64> {
        &self.results
    }

    pub fn execute(&self) {
        // TODO reuse the objects
        let mut test_writer = TestWriter::new(&self);
        test_writer.write().unwrap();

        let mut test_registrar = ModuleRegistrar::new(&self.objective);
        test_registrar.register();

        let test_runner = TestRunner::new();
        match test_runner.run(&self) {
            Ok(_) => {
                println!("Test went ok");
            }
            Err(_) => {
                println!("Test didn't work");
            }
        }
        test_registrar.unregister();
        test_writer.unwrite().unwrap();
    }

    pub fn size(&self) -> usize {
        self.stmts.len()
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
        self.mutation.mutate(&self)
    }

    fn fitness(&self) -> f64 {
        println!("Calculating fitness");
        10.0
    }

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized {
        println!("Doing crossover");
        (self.clone(), self.clone())
    }
}

#[derive(Debug, Default)]
pub struct TestCaseGenerator {
    branches: Rc<Vec<Branch>>,
    mutation: Rc<BasicMutation>
}

impl TestCaseGenerator {
    pub fn new(path: &str, branches: Rc<Vec<Branch>>, mutation: Rc<BasicMutation>) -> TestCaseGenerator {
        TestCaseGenerator {
            branches, mutation
        }
    }
}

impl ChromosomeGenerator for TestCaseGenerator {
    type C = TestCase;

    fn generate(&self) -> Self::C {
        let rand = fastrand::usize(..self.branches.len());
        let target = self.branches
            .get(rand)
            .cloned()
            .unwrap();
        let sig = &target.target_fn().sig;

        let args: Vec<Expr> = sig.inputs.iter().map(InputGenerator::generate_arg).collect();


        let ident = &sig.ident;
        let stmt = syn::parse_quote! {
            #ident(#(#args),*);
        };
        TestCase::new(target, vec![stmt], self.mutation.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::chromosome::TestCase;
}