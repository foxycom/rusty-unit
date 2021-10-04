use crate::algorithm::{PreferenceSorter, SVD};
use crate::chromosome::{
    Arg, Chromosome, ConstructorStmt, FnInvStmt, MethodInvStmt, Statement, TestCase,
};
use crate::generators::PrimitivesGenerator;
use crate::source::BranchManager;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use crate::selection::Selection;

pub trait Crossover: Debug {
    type C: Chromosome;

    fn apply(&self, a: &Self::C, b: &Self::C) -> (Self::C, Self::C);
}

pub trait Mutation: Debug {
    type C: Chromosome;

    fn apply(&self, chromosome: &Self::C) -> Self::C;
}

#[derive(Debug, Clone)]
pub struct SinglePointCrossover {}

impl SinglePointCrossover {
    pub fn new() -> Self {
        SinglePointCrossover {}
    }
}

impl<M: Mutation> Crossover for SinglePointCrossover {
    type C = TestCase<M, Self>;

    fn apply(&self, a: &Self::C, b: &Self::C) -> (Self::C, Self::C) {
        let mut child_a = a.clone();
        let mut child_b = b.clone();

        let a_i = fastrand::usize(0..a.size());
        let b_i = fastrand::usize(0..b.size());
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

impl<C: Crossover> Mutation for BasicMutation {
    type C = TestCase<Self, C>;

    fn apply(&self, chromosome: &Self::C) -> Self::C {
        let mut copy = chromosome.clone();

        if fastrand::f64() < 0.3 && chromosome.size() > 1 {
            // Delete a statement
            let i = fastrand::usize(0..copy.size());
            copy.delete_stmt(i);
            copy
        } else if fastrand::f64() < 0.3 {
            // Insert a method call
            unimplemented!()
        } else {
            // Modify a statement
            unimplemented!()
        }
    }
}

impl BasicMutation {
    pub fn new(branch_manager: Rc<RefCell<BranchManager>>) -> BasicMutation {
        BasicMutation { branch_manager }
    }

    fn mutate_stmt(&self, stmt: &Statement, dist: f64) -> Statement {
        let mut copy = stmt.clone();

        match copy {
            Statement::Constructor(ref mut constructor_stmt) => {
                self.mutate_constructor(constructor_stmt, dist);
            }
            Statement::MethodInvocation(ref mut method_inv_stmt) => {
                self.mutate_method_invocation(method_inv_stmt, dist);
            }
            Statement::FunctionInvocation(ref mut fn_inv_stmt) => {
                self.mutate_fn_invocation(fn_inv_stmt, dist);
            }
            _ => unimplemented!(),
        }

        copy
    }

    fn mutate_method_invocation(&self, method_inv_stmt: &mut MethodInvStmt, dist: f64) {
        let args = method_inv_stmt.args();
        let p = 1.0 / args.len() as f64;
        let mutated_args: Vec<Arg> = args
            .iter()
            .map(|a| BasicMutation::mutate_arg(a, p, dist))
            .collect();

        method_inv_stmt.set_args(mutated_args);
    }

    fn mutate_constructor(&self, costructor_stmt: &mut ConstructorStmt, dist: f64) {
        // Change arguments based on the distance to the selected branch
        let args = costructor_stmt.args();
        let p = 1.0 / args.len() as f64;
        let mutated_args: Vec<Arg> = args
            .iter()
            .map(|a| BasicMutation::mutate_arg(a, p, dist))
            .collect();

        costructor_stmt.set_args(mutated_args);
    }

    fn mutate_fn_invocation(&self, fn_inv_stmt: &mut FnInvStmt, dist: f64) {
        // Change arguments based on the distance to the selected branch
        let args = fn_inv_stmt.args();
        let p = 1.0 / args.len() as f64;
        let mutated_args: Vec<Arg> = args
            .iter()
            .map(|a| BasicMutation::mutate_arg(a, p, dist))
            .collect();

        fn_inv_stmt.set_args(mutated_args);
    }

    fn mutate_arg(arg: &Arg, p: f64, dist: f64) -> Arg {
        if fastrand::f64() < p {
            if dist < f64::MAX {
                PrimitivesGenerator::mutate_arg_dist(arg, dist)
            } else {
                PrimitivesGenerator::mutate_arg(arg)
            }
        } else {
            arg.clone()
        }
    }

    fn insert_statement(&self, test_case: &<Self as Mutation>::C) -> <Self as Mutation>::C {
        let mut copy = test_case.clone();
        self.statement_generator.insert_random_stmt(&mut copy);
        copy
        // TODO maintain correct positions
        /*if let Statement::MethodInvocation(method_inv_stmt) = &stmt {
            let (_, owner_idx) = copy.get_owner(&method_inv_stmt);
            let i = fastrand::usize(owner_idx + 1..=copy.size());
            copy.insert_stmt(i, stmt.clone());
        } else {
            unimplemented!()
        }*/
    }

    fn delete_statement(&self, test_case: &<Self as Mutation>::C) -> <Self as Mutation>::C {
        test_case.clone()

        /*let mut copy = test_case.clone();
        // TODO check dependencies

        let stmts = copy.stmts();
        let i = fastrand::usize(0..stmts.len());
        copy.delete_stmt(i);
        copy*/
    }

    fn reorder_statements(&self, test_case: &<Self as Mutation>::C) -> <Self as Mutation>::C {
        panic!();
        let mut copy = test_case.clone();

        let stmts = copy.stmts();
        // TODO check inequality
        let i = fastrand::usize(0..stmts.len());
        let j = fastrand::usize(0..stmts.len());

        copy.reorder_stmts(i, j);
        copy
    }
}

#[derive(Debug)]
pub struct RankSelection {
    branch_manager: Rc<RefCell<BranchManager>>,
    bias: f64,
}

impl<M: Mutation, C: Crossover> Selection for RankSelection {
    type C = TestCase<M, C>;

    fn apply(&self, population: &Vec<Self::C>) -> Self::C {
        todo!()
    }
}

impl RankSelection {
    pub fn new(branch_manager: Rc<RefCell<BranchManager>>) -> RankSelection {
        RankSelection {
            branch_manager,
            bias: 1.7,
        }
    }

    fn sort(&self, population: &[<Self as Selection>::C]) -> Vec<<Self as Selection>::C> {
        let mut sorted = vec![];
        let mut fronts =
            PreferenceSorter::sort(population, self.branch_manager.borrow().branches());
        fronts.iter_mut().for_each(|(k, v)| {
            *v = SVD::compute(v, self.branch_manager.borrow().branches()).unwrap()
        });
        for v in fronts.values_mut() {
            sorted.append(v);
        }
        sorted
    }

    pub fn select(&self, population: &[<Self as Selection>::C]) -> Option<<Self as Selection>::C> {
        let population = self.sort(population);
        let probabilities: Vec<f64> = (0..population.len())
            .map(|i| {
                self.bias - (2.0 * i as f64 * (self.bias - 1.0)) / (population.len() - 1) as f64
            })
            .collect();

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
