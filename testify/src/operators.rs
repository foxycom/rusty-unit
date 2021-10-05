use crate::algorithm::{PreferenceSorter, SVD};
use crate::chromosome::{
    Arg, Chromosome, ConstructorStmt, FnInvStmt, MethodInvStmt, Statement, TestCase, Var, VarArg,
};
use crate::selection::Selection;
use crate::source::BranchManager;
use std::cell::RefCell;
use std::env::var;
use std::fmt::Debug;
use std::rc::Rc;

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

impl Crossover for SinglePointCrossover {
    // TODO this is really bad
    type C = TestCase;

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

impl Mutation for BasicMutation {
    type C = TestCase;

    fn apply(&self, chromosome: &Self::C) -> Self::C {
        let mut copy = chromosome.clone();
        if fastrand::f64() < 0.3 {
            // Modify statement
            let stmts = copy.stmts();
            let stmt = stmts.get(fastrand::usize(0..stmts.len())).unwrap().clone();
            match stmt {
                Statement::Constructor(_) => {
                    self.mutate_invocation(&mut copy, stmt.clone());
                }
                Statement::MethodInvocation(_) => {
                    self.mutate_invocation(&mut copy, stmt.clone());
                }
                Statement::FunctionInvocation(_) => {
                    self.mutate_invocation(&mut copy, stmt.clone());
                }
                _ => unimplemented!(),
            }
        } else if fastrand::f64() < 0.3 {
            // Delete statement
            self.delete_statement(&mut copy);
        } else {
            // Insert a statement
            self.insert_statement(&mut copy);
        }

        copy
    }
}

impl BasicMutation {
    pub fn new(branch_manager: Rc<RefCell<BranchManager>>) -> BasicMutation {
        BasicMutation { branch_manager }
    }

    fn mutate_invocation(&self, test_case: &mut TestCase, mut stmt: Statement) {
        let stmt_id = stmt.id();
        let stmt_i = match test_case.stmt_position(stmt_id) {
            None => {
                test_case.to_dot();
                println!("{}", test_case.to_string());
                panic!()
            }
            Some(idx) => idx,
        };

        let args = stmt.args().unwrap();
        if !args.is_empty() {
            // Change the method itself
            unimplemented!()
        } else {
            // Change the source object for the arg or mutate if it's a primitive
            let arg_i = fastrand::usize(0..args.len());
            let arg = args.get(arg_i).unwrap();
            match arg {
                Arg::Var(var_arg) => {
                    if fastrand::f64() < 0.5 {
                        // Swap object
                        if var_arg.is_consuming() {
                            // Only use objects that are not consumed in the whole test and
                            // that will not be borrowed after this call
                            let variables = test_case.variables_typed(var_arg.param().ty());
                            let consumables: Vec<&(Var, usize)> = variables
                                .iter()
                                .filter(|(v, _)| test_case.is_consumable(v, stmt_i))
                                .collect();
                            if !consumables.is_empty() {
                                let consumable_i = fastrand::usize(0..consumables.len());
                                let (new_var, _) = consumables
                                    .get(consumable_i)
                                    .unwrap();
                                let mut new_stmt = stmt.clone();
                                let new_arg = Arg::Var(VarArg::new(new_var.clone(), var_arg.param().clone()));
                                new_stmt.set_arg(new_arg, arg_i);
                                test_case.remove_stmt(stmt_id);
                                test_case.insert_stmt(stmt_i, new_stmt);
                            } else {
                                unimplemented!("Was jetzt?")
                            }
                        } else {
                            unimplemented!()
                        }
                    } else {
                        // Mutate object
                        unimplemented!()
                    }
                }
                Arg::Primitive(primitive) => unimplemented!(),
            }
        }

        // Change which method is called (for now replace borrowing stmts with borrowing ones,
        // and consuming with consuming ones
        unimplemented!();
        // Change params
        /*let args = stmt.args();
        let p = 1.0 / args.len() as f64;
        let mutated_args: Vec<Arg> = args
            .iter()
            .map(|a| BasicMutation::mutate_arg(a, p, 0.0))
            .collect();

        stmt.set_args(mutated_args);*/
    }

    fn insert_statement(&self, test_case: &<Self as Mutation>::C) -> <Self as Mutation>::C {
        let mut copy = test_case.clone();

        copy.add_stmt()
    }

    fn delete_statement(&self, test_case: &<Self as Mutation>::C) -> <Self as Mutation>::C {
        let mut copy = test_case.clone();

        let stmts = copy.stmts();
        let i = fastrand::usize(0..stmts.len());
        copy.remove_stmt_at(i);
        copy
    }
}

#[derive(Debug)]
pub struct RankSelection {
    branch_manager: Rc<RefCell<BranchManager>>,
    bias: f64,
}

impl Selection for RankSelection {
    type C = TestCase;

    fn apply(&self, population: &[Self::C]) -> Self::C {
        self.select(population)
    }
}

impl RankSelection {
    pub fn new(branch_manager: Rc<RefCell<BranchManager>>) -> RankSelection {
        RankSelection {
            branch_manager,
            bias: 1.7,
        }
    }

    fn sort<C: Chromosome>(&self, population: &[C]) -> Vec<C> {
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

    pub fn select<C: Chromosome>(&self, population: &[C]) -> C {
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
            current += probabilities.get(i).unwrap();
            if current > pick {
                return population.get(i).cloned().unwrap();
            }
        }

        panic!("This should never happen")
    }
}
