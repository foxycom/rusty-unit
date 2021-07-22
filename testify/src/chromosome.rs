use std::fmt::{Debug, Display, Formatter, Error};
use syn::{Stmt, Item, ItemFn, FnArg, PatType, Type, Expr};
use std::cmp::Ordering;
use quote::ToTokens;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use crate::generators::InputGenerator;
use crate::io::writer::{TestWriter, ModuleRegistrar};
use crate::io::runner::TestRunner;
use crate::instr::data::{Branch};
use crate::operators::BasicMutation;
use std::rc::Rc;
use std::fs;
use std::hash::{Hash, Hasher};
use crate::parser::TraceParser;
use std::cell::RefCell;


pub trait Chromosome: Clone + Debug {
    fn mutate(&self) -> Self;

    fn fitness(&self, objective: &Branch) -> f64;

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized;
}

pub trait ChromosomeGenerator {
    type C: Chromosome;

    fn generate(&mut self) -> Self::C;
}

#[derive(Clone, Debug)]
pub struct TestCase {
    id: u64,
    objective: Branch,
    stmts: Vec<Stmt>,
    results: HashMap<u64, f64>,
    mutation: BasicMutation,
}

impl PartialEq for TestCase {
    fn eq(&self, other: &Self) -> bool {
        self.stmts == other.stmts && self.objective == other.objective
    }
}

impl Eq for TestCase {}

impl Hash for TestCase {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.objective.hash(state);
        self.stmts.hash(state);
    }
}

impl TestCase {
    pub const TEST_FN_PREFIX: &'static str = "testify";

    pub fn new(id: u64, target: Branch, stmts: Vec<Stmt>, mutation: BasicMutation) -> Self {
        TestCase {
            id,
            objective: target,
            stmts,
            results: HashMap::new(),
            mutation,
        }
    }


    pub fn stmts(&mut self) -> &mut Vec<Stmt> {
        &mut self.stmts
    }

    pub fn target(&self) -> &Branch {
        &self.objective
    }

    pub fn to_syn(&self) -> Item {
        let ident = Ident::new(&format!("{}_{}", TestCase::TEST_FN_PREFIX, self.id),
                               Span::call_site());
        let stmts = &self.stmts;
        let test: Item = syn::parse_quote! {
            #[test]
            fn #ident() {
                #(#stmts)*
            }
        };
        test
    }

    pub fn name(&self) -> String {
        format!("{}_{}", TestCase::TEST_FN_PREFIX, self.id)
    }

    pub fn results(&self) -> &HashMap<u64, f64> {
        &self.results
    }

    pub fn execute(&mut self) {
        // TODO reuse the objects
        let mut test_registrar = ModuleRegistrar::new(&self.objective);
        test_registrar.register();

        let test_runner = TestRunner::new();
        match test_runner.run(&self) {
            Ok(_) => {
                println!("Test {} went ok", self.id);
            }
            Err(_) => {
                println!("Test {} failed", self.id);
            }
        }
        test_registrar.unregister();

        // TODO dynamic path
        self.results = TraceParser::parse("/Users/tim/Documents/master-thesis/testify/src/examples/additions/trace.txt").unwrap();
        fs::remove_file("/Users/tim/Documents/master-thesis/testify/src/examples/additions/trace.txt").unwrap();
    }

    pub fn size(&self) -> usize {
        self.stmts.len()
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn id(&self) -> u64 {
        self.id
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

    fn fitness(&self, objective: &Branch) -> f64 {
        objective.fitness(self)
    }

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized {
        (self.clone(), self.clone())
    }
}

#[derive(Debug)]
pub struct TestCaseGenerator {
    branches: Vec<Branch>,
    mutation: BasicMutation,
    test_id: u64
}

impl TestCaseGenerator {
    pub fn new(branches: Vec<Branch>, mutation: BasicMutation, test_id: u64) -> TestCaseGenerator {
        TestCaseGenerator {
            branches,
            mutation,
            test_id
        }
    }
}

impl ChromosomeGenerator for TestCaseGenerator {
    type C = TestCase;

    fn generate(&mut self) -> Self::C {
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

        let test_case = TestCase::new(
            self.test_id,
            target,
            vec![stmt],
            self.mutation.clone());
        self.test_id += 1;
        //test_case.execute();
        test_case
    }
}

