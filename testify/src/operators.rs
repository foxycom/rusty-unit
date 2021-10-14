use crate::algorithm::{PreferenceSorter, SVD};
use crate::chromosome::{
    Arg, Chromosome, ConstructorStmt, FnInvStmt, MethodInvStmt, Param, Primitive, Statement,
    TestCase, Var, VarArg,
};
use crate::selection::Selection;
use crate::source::{BranchManager, SourceFile};
use quote::ToTokens;
use std::cell::RefCell;
use std::env::var;
use std::fmt::Debug;
use std::rc::Rc;
use syn::Stmt;
use uuid::Uuid;

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
    source_file: Rc<SourceFile>,
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
    pub fn new(
        source_file: Rc<SourceFile>,
        branch_manager: Rc<RefCell<BranchManager>>,
    ) -> BasicMutation {
        BasicMutation {
            branch_manager,
            source_file,
        }
    }

    pub fn mutate_invocation(&self, test_case: &mut TestCase, stmt: Statement) {
        let stmt_id = stmt.id();
        let stmt_i = match test_case.stmt_position(stmt_id) {
            None => {
                test_case.to_file();
                panic!(
                    "Looking for stmt {} with id {} in test {}",
                    stmt,
                    stmt_id,
                    test_case.id()
                );
            }
            Some(idx) => idx,
        };

        let mut new_stmt = stmt.clone();
        let args = stmt.args().unwrap();
        if !args.is_empty() {
            // Change the source object for the arg or mutate if it's a primitive
            let arg_i = fastrand::usize(0..args.len());
            let arg = args.get(arg_i).unwrap();
            match arg {
                Arg::Var(var_arg) => {
                    // TODO reduce probability
                    if fastrand::f64() < 1.0 {
                        // Swap object
                        let variables = test_case.variables_typed(var_arg.param().ty());
                        let candidates: Vec<&(Var, usize)>;
                        if var_arg.is_consuming() {
                            // The candidates must not be consumed throughout the whole test
                            // and they also must not be borrowed after stmt_i
                            candidates = variables
                                .iter()
                                .filter(|(v, _)| test_case.is_consumable(v, stmt_i))
                                .collect();
                        } else {
                            // The candidates must not be consumed before stmt_i
                            candidates = variables
                                .iter()
                                .filter(|(v, _)| test_case.is_borrowable(v, stmt_i))
                                .collect();
                        }

                        if !candidates.is_empty() {
                            let candidate_i = fastrand::usize(0..candidates.len());
                            let (new_var, _) = candidates.get(candidate_i).unwrap();
                            let new_arg =
                                Arg::Var(VarArg::new(new_var.clone(), var_arg.param().clone()));
                            new_stmt.set_arg(new_arg, arg_i);
                        } else {
                            // There are no candidates
                            unimplemented!("Was jetzt?")
                        }
                    } else {
                        // Mutate object
                        unimplemented!()
                    }
                }
                Arg::Primitive(primitive) => {
                    let mutated_primitive = Arg::Primitive(primitive.mutate());
                    new_stmt.set_arg(mutated_primitive, arg_i);
                }
            }
        } else {
            // Args is empty, hence it's not an associative method
            // Change to a call with an identical return type

            let return_type = stmt.return_type();
            if let Some(ty) = return_type {
                let generators = test_case.source_file().generators(ty);
                if !generators.is_empty() {
                    let generator_i = fastrand::usize(0..generators.len());
                    let generator = generators.get(generator_i).unwrap();

                    let args: Vec<Arg> = generator
                        .params()
                        .iter()
                        .map(|p| test_case.generate_arg(p))
                        .collect();
                    new_stmt = generator.to_stmt(args);
                } else {
                    unimplemented!()
                }
            } else {
                // TODO is this appropriate?
                unimplemented!()
            }
        }

        test_case.remove_stmt(stmt_id);
        test_case.insert_stmt(stmt_i, new_stmt);
    }

    fn insert_statement(&self, test_case: &<Self as Mutation>::C) -> <Self as Mutation>::C {
        let mut copy = test_case.clone();

        /*if self.source_files.len() > 1 {
            panic!("The implementation is incorrect for multiple files")
        }

        let source_file_i = fastrand::usize(0..self.source_files.len());
        let source_file = self.source_files.get(source_file_i).unwrap();*/

        // TODO types can be 0 length and lead to panic

        let available_callables = test_case.available_callables();

        if !available_callables.is_empty() && fastrand::f64() < 0.5 {
            let callable_i = fastrand::usize(0..available_callables.len());
            let callable_tuple = available_callables.get(callable_i).unwrap();

            let self_param = match callable_tuple.1.params().first() {
                None => {
                    test_case.to_file();
                    panic!(
                        "\nFailing test: {}, callable: {:?}",
                        test_case.id(),
                        callable_tuple.1
                    );
                }
                Some(param) => param,
            };
            let self_arg = Arg::Var(VarArg::new(callable_tuple.0.clone(), self_param.clone()));

            let mut args = Vec::with_capacity(callable_tuple.1.params().len());
            args.push(self_arg);

            callable_tuple.1.params()[1..].iter().for_each(|p| {
                let arg = copy.generate_arg(p);
                args.push(arg);
            });

            let stmt = callable_tuple.1.to_stmt(args);
            let stmt_i = fastrand::usize(callable_tuple.2.clone());
            copy.insert_stmt(stmt_i, stmt);
        } else {
            // Generate a new object
            let types = copy.instantiated_types();
            if types.is_empty() {
                // TODO There is nothing defined, why?
                copy.insert_random_stmt();
            } else {
                let ty = types.get(fastrand::usize(0..types.len())).unwrap();
                let source_file = copy.source_file();

                let callables = source_file.callables_of(ty);
                if callables.is_empty() {
                    println!();
                }
                let i = fastrand::usize(0..callables.len());
                let callable = callables.get(i).unwrap();
                let args = callable
                    .params()
                    .iter()
                    .map(|p| copy.generate_arg(p))
                    .collect();
                let stmt = callable.to_stmt(args);
                copy.add_stmt(stmt);
            }
        }

        copy
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
