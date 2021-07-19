use crate::chromosome::{Chromosome, TestCase};
use crate::instr::data::Branch;
use syn::{Stmt, Expr};
use std::rc::Rc;
use std::mem;
use crate::generators::InputGenerator;
use syn::punctuated::Punctuated;

pub trait Crossover {
    type C: Chromosome;

    fn apply(&self, a: &Self::C, b: &Self::C) -> (Self::C, Self::C);
}

pub trait Mutation {
    type C: Chromosome;

    fn apply(&self, chromosome: &Self::C) -> Self::C;
}

#[derive(Debug, Default)]
pub struct BasicMutation {
    branches: Rc<Vec<Branch>>
}

impl BasicMutation {
    pub fn new(branches: Rc<Vec<Branch>>) -> BasicMutation {
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