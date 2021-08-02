use std::fmt::{Debug, Display, Formatter, Error};
use syn::{Stmt, Item, ItemFn, FnArg, PatType, Type, Expr};
use std::cmp::Ordering;
use quote::ToTokens;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use crate::generators::{InputGenerator, TestIdGenerator};
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
        /*self.stmts == other.stmts && self.objective == other.objective*/
        self.id == other.id
    }
}

impl Eq for TestCase {}

impl Hash for TestCase {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        /*self.objective.hash(state);
        self.stmts.hash(state);*/
    }
}

impl TestCase {
    pub const TEST_FN_PREFIX: &'static str = "testify";

    pub fn new(id: u64, objective: Branch, stmts: Vec<Stmt>, mutation: BasicMutation) -> Self {
        TestCase {
            id,
            objective,
            stmts,
            results: HashMap::new(),
            mutation,
        }
    }


    pub fn stmts(&mut self) -> &mut Vec<Stmt> {
        &mut self.stmts
    }

    pub fn objective(&self) -> &Branch {
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

    pub fn size(&self) -> usize {
        self.stmts.len()
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn set_results(&mut self, results: HashMap<u64, f64>) {
        self.results = results;
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
        self.mutation.mutate(self)
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
    test_id: Rc<RefCell<TestIdGenerator>>
}

impl TestCaseGenerator {
    pub fn new(branches: Vec<Branch>, mutation: BasicMutation, test_id: Rc<RefCell<TestIdGenerator>>) -> TestCaseGenerator {
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

        let test_id = self.test_id.borrow_mut().next_id();
        let test_case = TestCase::new(
            test_id,
            target,
            vec![stmt],
            self.mutation.clone());
        //test_case.execute();
        test_case
    }
}

