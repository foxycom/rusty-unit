use std::fmt::{Debug, Display, Formatter, Error};
use syn::{Stmt, Item, ItemFn, FnArg, PatType, Type, Expr};
use std::cmp::Ordering;
use quote::ToTokens;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use crate::generators::{InputGenerator, TestIdGenerator};
use crate::operators::{BasicMutation, BasicCrossover};
use std::rc::Rc;
use std::fs;
use std::hash::{Hash, Hasher};
use crate::parser::TraceParser;
use std::cell::RefCell;
use crate::source::{Branch, BranchManager};


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
    branch: Branch,
    stmts: Vec<Statement>,
    results: HashMap<u64, f64>,
    mutation: BasicMutation,
    crossover: BasicCrossover,
    branch_manager: Rc<RefCell<BranchManager>>,
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

    pub fn new(id: u64, branch: Branch, stmts: Vec<Statement>, mutation: BasicMutation, crossover: BasicCrossover, branch_manager: Rc<RefCell<BranchManager>>) -> Self {
        TestCase {
            id,
            branch,
            stmts,
            results: HashMap::new(),
            mutation,
            crossover,
            branch_manager,
        }
    }


    pub fn stmts(&mut self) -> &mut Vec<Statement> {
        &mut self.stmts
    }

    pub fn objective(&self) -> &Branch {
        &self.branch
    }

    pub fn to_syn(&self) -> Item {
        let ident = Ident::new(&format!("{}_{}", TestCase::TEST_FN_PREFIX, self.id),
                               Span::call_site());
        let id = self.id;
        let set_test_id: Stmt = syn::parse_quote! {
              LOGGER.with(|l| l.borrow_mut().set_test_id(#id));
        };
        let wait: Stmt = syn::parse_quote! {
            LOGGER.with(|l| l.borrow_mut().wait());
        };

        let stmts: Vec<Stmt> = self.stmts.iter().map(Statement::to_syn).collect();
        let test: Item = syn::parse_quote! {
            #[test]
            fn #ident() {
                #set_test_id
                #(#stmts)*
                #wait
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
    pub fn set_stmts(&mut self, stmts: &[Statement]) {
        self.stmts = stmts.to_vec();
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
        self.crossover.crossover(self, other)
    }
}

#[derive(Clone, Debug)]
pub struct Statement {
    params: Vec<FnArg>,
    args: Vec<Expr>,
    ident: Ident,
    item_fn: ItemFn,
}

impl Statement {
    pub fn new(ident: Ident, item_fn: ItemFn, params: Vec<FnArg>, args: Vec<Expr>) -> Self {
        Statement { params, args, ident, item_fn }
    }

    pub fn params(&self) -> &Vec<FnArg> {
        &self.params
    }
    pub fn args(&self) -> &Vec<Expr> {
        &self.args
    }
    pub fn ident(&self) -> &Ident {
        &self.ident
    }
    pub fn item_fn(&self) -> &ItemFn {
        &self.item_fn
    }

    pub fn has_params(&self) -> bool {
        !self.params.is_empty()
    }

    pub fn set_args(&mut self, args: Vec<Expr>) {
        self.args = args;
    }

    pub fn to_syn(&self) -> Stmt {
        let ident = &self.ident;
        let args = &self.args;

        syn::parse_quote! {
            #ident(#(#args),*);
        }
    }
}

#[derive(Debug)]
pub struct TestCaseGenerator {
    branch_manager: Rc<RefCell<BranchManager>>,
    mutation: BasicMutation,
    crossover: BasicCrossover,
    test_id: Rc<RefCell<TestIdGenerator>>,
}

impl TestCaseGenerator {
    pub fn new(branch_manager: Rc<RefCell<BranchManager>>,
               mutation: BasicMutation,
               crossover: BasicCrossover,
               test_id: Rc<RefCell<TestIdGenerator>>) -> TestCaseGenerator {
        TestCaseGenerator {
            branch_manager,
            mutation,
            crossover,
            test_id,
        }
    }
}

impl ChromosomeGenerator for TestCaseGenerator {
    type C = TestCase;

    fn generate(&mut self) -> Self::C {
        let bm = self.branch_manager.borrow();
        let (stmt, target) = bm.get_random_stmt();

        let test_id = self.test_id.borrow_mut().next_id();
        let test_case = TestCase::new(
            test_id,
            target,
            vec![stmt],
            self.mutation.clone(),
            self.crossover.clone(),
            self.branch_manager.clone(),
        );

        test_case
    }
}

