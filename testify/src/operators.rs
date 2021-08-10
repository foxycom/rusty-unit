use crate::chromosome::{Chromosome, TestCase, Statement};
use syn::{Stmt, Expr};
use std::rc::Rc;
use std::mem;
use crate::generators::InputGenerator;
use syn::punctuated::Punctuated;
use crate::algorithm::{PreferenceSorter, SVD};
use std::cell::RefCell;
use crate::source::BranchManager;

pub trait Crossover {
    type C: Chromosome;

    fn apply(&self, a: &Self::C, b: &Self::C) -> (Self::C, Self::C);
}

pub trait Mutation {
    type C: Chromosome;

    fn apply(&self, chromosome: &Self::C) -> Self::C;
}

#[derive(Debug, Clone)]
pub struct BasicCrossover {
}

impl BasicCrossover {
    pub fn new() -> Self {
        BasicCrossover {}
    }

    pub fn crossover(&self, a: &TestCase, b: &TestCase) -> (TestCase, TestCase) {
        let mut child_a = a.clone();
        let mut child_b = b.clone();

        let a_i = fastrand::usize((0..a.size()));
        let b_i = fastrand::usize((0..b.size()));
        let (stmts_a1, stmts_a2) = child_a.stmts().split_at(a_i);
        let (stmts_b1, stmts_b2) = child_b.stmts().split_at(b_i);

        let mut stmts_a = Vec::with_capacity(stmts_a1.len() + stmts_b2.len());
        stmts_a.append(&mut stmts_a1.to_vec());
        stmts_a.append(&mut stmts_b2.to_vec());

        let mut stmts_b = Vec::with_capacity(stmts_b1.len() + stmts_a2.len());
        stmts_b.append(&mut stmts_b1.to_vec());
        stmts_b.append(&mut stmts_a2.to_vec());

        child_a.set_stmts(&stmts_a);
        child_b.set_stmts(&stmts_b);
        // TODO consider upper limit of statements

        (child_a, child_b)
    }
}

#[derive(Debug, Clone)]
pub struct BasicMutation {
    branch_manager: Rc<RefCell<BranchManager>>,
}

impl BasicMutation {
    pub fn new(branch_manager: Rc<RefCell<BranchManager>>) -> BasicMutation {
        BasicMutation {
            branch_manager
        }
    }

    fn mutate_stmt(&self, stmt: &Statement, dist: f64) -> Statement {
        let mut copy = stmt.clone();

        // Change arguments based on the distance to the selected branch
        let args = copy.args();
        let p = 1.0 / args.len() as f64;
        let mutated_args: Vec<Expr> = args.iter()
            .map(|a| {
                if fastrand::f64() < p {
                    if dist < f64::MAX {
                        InputGenerator::mutate_arg_dist(a, dist)
                    } else {
                        InputGenerator::mutate_arg(a)
                    }
                } else {
                    a.clone()
                }
            }).collect();

        copy.set_args(mutated_args);
        copy
    }

    pub fn mutate(&self, test_case: &TestCase) -> TestCase {
        let bm = self.branch_manager.borrow();

        let uncovered_branches = bm.uncovered_branches();

        // TODO magic numbers
        return if uncovered_branches.is_empty() {
            test_case.clone()
        } else if fastrand::f64() < 0.1 && test_case.size() > 1 {
            // Reorder statementes
            self.reorder_statements(test_case)
        } else if fastrand::f64() < 0.1 && test_case.size() > 1 {
            // Delete statement
            self.delete_statement(test_case)
        } else {
            // Select a branch that has not been covered yet
            let branch_idx = fastrand::usize((0..uncovered_branches.len()));
            let branch = uncovered_branches.get(branch_idx).unwrap();

            // The value which the previous execution of the test was off to the branch
            let dist = test_case.fitness(branch);
            if dist == f64::MAX {
                // Insert call to target
                self.insert_statement(test_case)
            } else {
                let mut copy = test_case.clone();

                let len = copy.size();
                let p = 1.0 / len as f64;
                for (i, stmt) in copy.stmts().iter_mut().enumerate() {
                    if fastrand::f64() < p {
                        let mutated_stmt = self.mutate_stmt(&stmt, dist);
                        mem::replace(stmt, mutated_stmt);
                    }
                }
                copy
            }
        };
    }

    fn insert_statement(&self, test_case: &TestCase) -> TestCase {
        let bm = self.branch_manager.borrow();
        let mut copy = test_case.clone();
        // TODO check size or whether there are available target functions
        let (stmt, _) = bm.get_random_stmt();
        let stmts = copy.stmts();
        let i = fastrand::usize((0..=stmts.len()));

        stmts.insert(i, stmt);
        copy
    }

    fn delete_statement(&self, test_case: &TestCase) -> TestCase {
        let mut copy = test_case.clone();
        // TODO check size
        // TODO check dependencies

        let stmts = copy.stmts();
        let i = fastrand::usize((0..stmts.len()));
        stmts.remove(i);
        copy
    }

    fn reorder_statements(&self, test_case: &TestCase) -> TestCase {
        let mut copy = test_case.clone();

        let stmts = copy.stmts();
        // TODO check inequality
        let i = fastrand::usize((0..stmts.len()));
        let j = fastrand::usize((0..stmts.len()));

        stmts.swap(i, j);
        copy
    }
}

#[derive(Debug)]
pub struct RankSelection {
    branch_manager: Rc<RefCell<BranchManager>>,
    bias: f64,
}

impl RankSelection {
    pub fn new(branch_manager: Rc<RefCell<BranchManager>>) -> RankSelection {
        RankSelection {
            branch_manager,
            bias: 1.7,
        }
    }

    fn sort(&self, population: &[TestCase]) -> Vec<TestCase> {
        let mut sorted = vec![];
        let mut fronts = PreferenceSorter::sort(population, self.branch_manager.borrow().branches());
        fronts.iter_mut()
            .for_each(|(k, v)| {
                *v = SVD::compute(v, self.branch_manager.borrow().branches()).unwrap()
            });
        for v in fronts.values_mut() {
            sorted.append(v);
        }
        sorted
    }

    pub fn select(&self, population: &[TestCase]) -> Option<TestCase> {
        let population = self.sort(population);
        let probabilities: Vec<f64> = (0..population.len()).map(|i| {
            self.bias - (2.0 * i as f64 * (self.bias - 1.0)) / (population.len() - 1) as f64
        }).collect();

        let fitness_sum: f64 = probabilities.iter().sum();
        let pick = fastrand::f64() * fitness_sum;
        let mut current = 0.0;
        for i in 0..probabilities.len() {
            current += probabilities.get(i)?;
            if current > pick {
                return population.get(i).cloned();
            }
        }

        None
    }
}