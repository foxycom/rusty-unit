use crate::chromosome::{Chromosome, TestCase};
use crate::instr::data::Branch;
use syn::{Stmt, Expr};
use std::rc::Rc;
use std::mem;
use crate::generators::InputGenerator;
use syn::punctuated::Punctuated;
use crate::algorithm::{PreferenceSorter, SVD};

pub trait Crossover {
    type C: Chromosome;

    fn apply(&self, a: &Self::C, b: &Self::C) -> (Self::C, Self::C);
}

pub trait Mutation {
    type C: Chromosome;

    fn apply(&self, chromosome: &Self::C) -> Self::C;
}

#[derive(Debug, Clone)]
pub struct BasicMutation {
    branches: Vec<Branch>
}

impl BasicMutation {
    pub fn new(branches: Vec<Branch>) -> BasicMutation {
        BasicMutation {
            branches
        }
    }

    fn mutate_stmt(&self, stmt: &Stmt) -> Stmt {
        let mut mut_stmt = stmt.clone();

        match mut_stmt {
            Stmt::Semi(ref mut expr, _) => {
                match expr {
                    Expr::Call(call) => {
                        let args = &call.args;
                        let p = 1.0 / args.len() as f64;
                        let mutated_args: Vec<Expr> = args.iter()
                            .map(InputGenerator::mutate_arg)
                            .collect();
                        call.args = syn::parse_quote! {
                            #(#mutated_args),*
                        };
                    }
                    _ => {}
                }
            }
            _ => {}
        }


        mut_stmt
    }

    pub fn mutate(&self, test_case: &TestCase) -> TestCase {
        let mut mut_test_case = test_case.clone();
        let len = mut_test_case.stmts().len();
        let p = 1.0 / len as f64;
        for (i, stmt) in mut_test_case.stmts().iter_mut().enumerate() {
            if fastrand::f64() < p {
                let mutated_stmt = self.mutate_stmt(&stmt);
                mem::replace(stmt, mutated_stmt);
            }
        }


        mut_test_case
    }
}

#[derive(Debug)]
pub struct RankSelection {
    objectives: Vec<Branch>,
    bias: f64
}

impl RankSelection {
    pub fn new(objectives: Vec<Branch>) -> RankSelection {
        RankSelection {
            objectives,
            bias: 1.7
        }
    }

    fn sort(&self, population: &[TestCase]) -> Vec<TestCase> {
        let mut sorted = vec![];
        let mut fronts = PreferenceSorter::sort(population, &self.objectives);
        fronts.iter_mut().for_each(|(k, v)| *v = SVD::compute(v, &self.objectives).unwrap());
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