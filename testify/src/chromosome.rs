use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::env::var;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use petgraph::prelude::{NodeIndex, StableDiGraph};
use petgraph::{Direction, Graph};
use petgraph::dot::{Config, Dot};
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{
    Expr, FnArg, ImplItemMethod, Item, ItemFn, Pat, PatType, Path, ReturnType, Stmt, Type, TypePath,
};
use uuid::Uuid;

use crate::generators::{TestIdGenerator};
use crate::operators::{BasicMutation, Crossover, Mutation, SinglePointCrossover};

use crate::source::{Branch, BranchManager, SourceFile};
use crate::util;
use crate::util::{fn_arg_to_param, is_constructor, is_method, return_type_name, type_name};
lazy_static! {
    static ref CHROMOSOME_ID_GENERATOR: Arc<Mutex<TestIdGenerator>> =
        Arc::new(Mutex::new(TestIdGenerator::new()));
}

const TEST_FN_PREFIX: &'static str = "testify";

pub trait ToSyn {
    fn to_syn(&self) -> Item;
}

pub trait Chromosome: Clone + Debug + ToSyn + PartialEq + Eq + Hash {
    /// Returns the unique id of the test
    fn id(&self) -> u64;

    fn coverage(&self) -> &HashMap<Branch, FitnessValue>;

    fn set_coverage(&mut self, coverage: HashMap<Branch, FitnessValue>);

    /// Applies mutation to the chromosome
    fn mutate<M: Mutation<C = Self>>(&self, mutation: &M) -> Self;

    /// Returns the fitness of the chromosome with respect to a certain branch
    fn fitness(&self, objective: &Branch) -> FitnessValue;

    /// Applies crossover to this and other chromosome and returns a pair of offsprings
    fn crossover<C: Crossover<C = Self>>(&self, other: &Self, crossover: &C) -> (Self, Self)
    where
        Self: Sized;

    /// Generates a random chromosome
    fn random(source_file: &SourceFile<Self>) -> Self;

    fn size(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dependency {
    Owns,
    Consumes,
    Borrows,
}

#[derive(Copy, Clone, Debug)]
pub enum FitnessValue {
    Zero,
    Val(f64),
    Max,
}

impl PartialEq for FitnessValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            FitnessValue::Zero => {
                if let Self::Zero = other {
                    true
                } else {
                    false
                }
            }
            FitnessValue::Val(a) => {
                if let Self::Val(b) = other {
                    a == b
                } else {
                    false
                }
            }
            FitnessValue::Max => {
                if let Self::Max = other {
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Eq for FitnessValue {}

impl PartialOrd for FitnessValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            FitnessValue::Zero => match other {
                FitnessValue::Zero => Some(Ordering::Equal),
                FitnessValue::Val(_) => Some(Ordering::Less),
                FitnessValue::Max => Some(Ordering::Less),
            },
            FitnessValue::Val(a) => match other {
                FitnessValue::Zero => Some(Ordering::Greater),
                FitnessValue::Val(b) => a.partial_cmp(b),
                FitnessValue::Max => Some(Ordering::Less),
            },
            FitnessValue::Max => match other {
                FitnessValue::Zero => Some(Ordering::Greater),
                FitnessValue::Val(_) => Some(Ordering::Greater),
                FitnessValue::Max => Some(Ordering::Equal),
            },
        }
    }
}

impl Display for FitnessValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FitnessValue::Zero => write!(f, "Zero"),
            FitnessValue::Val(val) => write!(f, "Val({})", val),
            FitnessValue::Max => write!(f, "Max"),
        }
    }
}

#[derive(Debug)]
pub struct TestCase {
    id: u64,
    stmts: Vec<Statement>,
    coverage: HashMap<Branch, FitnessValue>,
    ddg: StableDiGraph<Uuid, Dependency>,

    /// Stores connection of variables and the appropriate statements
    var_table: HashMap<Var, Uuid>,

    /// Stores ids of statement to be able to retrieve dd graph nodes later by their index
    node_index_table: HashMap<Uuid, NodeIndex>,
    var_counters: HashMap<String, usize>,
}

impl Clone for TestCase {
    fn clone(&self) -> Self {
        TestCase {
            id: self.id.clone(),
            stmts: self.stmts.clone(),
            coverage: self.coverage.clone(),
            ddg: self.ddg.clone(),
            node_index_table: self.node_index_table.clone(),
            var_table: self.var_table.clone(),
            var_counters: self.var_counters.clone(),
        }
    }
}

impl PartialEq for TestCase {
    fn eq(&self, other: &Self) -> bool {
        /*self.stmts == other.stmts && self.objective == other.objective*/

        // TODO there is more to it
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
    pub fn new(id: u64) -> Self {
        TestCase {
            id,
            stmts: Vec::new(),
            coverage: HashMap::new(),
            ddg: StableDiGraph::new(),
            var_table: HashMap::new(),
            node_index_table: HashMap::new(),
            var_counters: HashMap::new(),
        }
    }

    pub fn to_dot(&self) {
        println!("{:?}", Dot::with_config(&self.ddg, &[Config::NodeIndexLabel]));
    }

    pub fn stmts(&self) -> &Vec<Statement> {
        &self.stmts
    }

    pub fn is_cutable(&self) -> bool {
        self.size() > 1
    }

    fn set_var(&mut self, stmt: &mut Statement) -> Option<Var> {
        if let Some(return_type) = stmt.return_type() {
            let type_name = return_type.to_string();
            let counter = self
                .var_counters
                .entry(type_name.clone())
                .and_modify(|c| *c = *c + 1)
                .or_insert(0);

            let var_name = format!("{}_{}", type_name.to_lowercase(), counter);
            let var = Var::new(&var_name, return_type.clone());
            stmt.set_var(var.clone());
            Some(var)
        } else {
            None
        }
    }

    pub fn add_stmt(&mut self, mut stmt: Statement) -> Option<Var> {
        let var = self.set_var(&mut stmt);
        let uuid = stmt.id();

        let node_index = self.ddg.add_node(uuid);
        self.node_index_table.insert(uuid, node_index.clone());

        let mut insert_position: usize = 0;
        if let Some(args) = stmt.args() {
            insert_position = args.iter()
                .filter_map(|a|
                    // Return the indices of statements that produce variables used
                    // as arguments in a statement. All variables must be defined before
                    // they can be used in a statement
                    if let Arg::Var(var_arg) = a {
                        // Connect used variables in the graph
                        let var = var_arg.var();
                        let generating_statement_uuid = self.var_table.get(var).unwrap();
                        let other_node_index = self.node_index_table.get(
                            &generating_statement_uuid
                        ).unwrap();

                        if var_arg.is_by_reference() {
                            self.ddg.add_edge(node_index, other_node_index.clone(), Dependency::Borrows);
                        } else {
                            self.ddg.add_edge(node_index, other_node_index.clone(), Dependency::Consumes);
                        }

                        let generating_stmt_position = self.stmt_position(generating_statement_uuid.clone()).unwrap();

                        Some(generating_stmt_position)
                    } else {
                        None
                    }
                ).fold(0usize, |a, b| {
                a.max(b)
            });
        }

        let length = self.stmts.len();
        // The statements list might be empty, so we have to check the bounds
        self.stmts.insert(length.min(insert_position + 1), stmt);

        if let Some(var) = &var {
            self.var_table.insert(var.clone(), uuid);
        }

        var
    }

    pub fn get_owner(&self, stmt: &MethodInvStmt) -> (&Statement, usize) {
        for (i, s) in self.stmts.iter().enumerate() {
            if let Statement::Constructor(constructor) = s {
                if constructor.name().unwrap() == stmt.owner() {
                    return (s, i);
                }
            }
        }
        panic!()
    }

    pub fn remove_stmt(&mut self, stmt_id: Uuid) -> usize {
        let position = self.stmts.iter().position(|s| s.id() == stmt_id).unwrap();
        self.remove_stmt_at(position);

        position
    }

    pub fn remove_stmt_at(&mut self, idx: usize) {
        let stmt = self.stmts.remove(idx);
        let id = stmt.id();

        if stmt.returns_value() {
            self.var_table.remove(stmt.var().unwrap());
        }

        let node_index = self.node_index_table.get(&id).unwrap().clone();
        let neighbours = self
            .ddg
            .neighbors_directed(node_index.clone(), Direction::Incoming);

        let neighbour_ids: Vec<usize> = neighbours
            .map(|n| {
                let uuid = self.ddg.node_weight(n.clone()).unwrap();
                self.stmt_position(uuid.clone()).unwrap()
            })
            .collect();
        neighbour_ids.iter().for_each(|&i| self.remove_stmt_at(i));
        self.ddg.remove_node(node_index.clone());
    }

    pub fn stmt_position(&self, id: Uuid) -> Option<usize> {
        self.stmts.iter().position(|s| s.id() == id)
    }

    pub fn var_position(&self, var: &Var) -> Option<usize> {
        let id = self.var_table.get(var);
        if let Some(&id) = id {
            self.stmt_position(id)
        } else {
            None
        }
    }

    pub fn replace_stmt(&mut self, idx: usize, stmt: Statement) {
        std::mem::replace(&mut self.stmts[idx], stmt);
    }

    pub fn swap_stmts(&mut self, idx_a: usize, idx_b: usize) {
        self.stmts.swap(idx_a, idx_b);
    }

    pub fn add_random_stmt(&mut self) {
        unimplemented!()
    }

    pub fn name(&self) -> String {
        format!("{}_{}", TEST_FN_PREFIX, self.id)
    }

    pub fn instantiated_primitives(&mut self) -> Vec<AssignStmt> {
        self.stmts()
            .iter()
            .filter_map(|s| {
                if let Statement::PrimitiveAssignment(stmt) = s {
                    Some(stmt.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn instantiated_types(&self) -> Vec<T> {
        self.var_table
            .iter()
            .map(|(var, _)| var.ty.clone())
            .collect()
    }

    pub fn variables(&self) -> Vec<Var> {
        self.var_table.keys().cloned().collect()
    }

    pub fn is_consumable(&self, var: &Var, idx: usize) -> bool {
        unimplemented!()
    }

    pub fn is_borrowable(&self, var: &Var, idx: usize) -> bool {
        unimplemented!()
    }

    pub fn unconsumed_variables_typed(&self, ty: &T) -> Vec<(Var, usize)> {
        let mut vars = self.variables_typed(ty);
        vars.retain(|(v, _)| !self.is_consumed(v));
        vars
    }

    pub fn variables_typed(&self, ty: &T) -> Vec<(Var, usize)> {
        // Also return their positions in the test case
        self.stmts
            .iter()
            .zip(0..self.stmts.len())
            .filter_map(|(s, i)| {
                if let Some(var) = s.var() {
                    if var.ty() == ty {
                        return Some((var.clone(), i));
                    }
                }
                None
            }).collect()
    }

    pub fn is_consumed(&self, var: &Var) -> bool {
        let uuid = self.var_table.get(var).unwrap();
        let node_index = self.node_index_table.get(uuid).unwrap();
        let mut incoming_edges = self
            .ddg
            .edges_directed(node_index.to_owned(), Direction::Incoming);
        incoming_edges.any(|e| e.weight().to_owned() == Dependency::Consumes)
    }

    pub fn consumed_at(&self, var: &Var) -> Option<usize> {
        let uuid = self.var_table.get(var).unwrap();
        let node_index = self.node_index_table.get(uuid).unwrap();
        let mut neighbours = self
            .ddg
            .neighbors_directed(node_index.clone(), Direction::Incoming);

        let consuming_stmt_index = neighbours.find(|n| {
            let edge = self.ddg.find_edge(node_index.clone(), n.clone());
            if let Some(edge_index) = edge {
                let weight = self.ddg.edge_weight(edge_index).unwrap();
                return *weight == Dependency::Consumes
            }
            false
        });

        if let Some(consuming_stmt_index) = consuming_stmt_index {
            let consuming_stmt_id = self.ddg.node_weight(consuming_stmt_index);
            if let Some(id) = consuming_stmt_id {
                return self.stmts.iter().position(|s| s.id() == *id)
            }
        }
        None
    }

    pub fn set_coverage(&mut self, coverage: HashMap<Branch, FitnessValue>) {
        self.coverage = coverage;
    }
    pub fn set_stmts(&mut self, stmts: &[Statement]) {
        self.stmts = stmts.to_vec();
    }
    pub fn var_counters(&self) -> &HashMap<String, usize> {
        &self.var_counters
    }

    /// Generates a complete random statement and all its dependencies, even if the
    /// test already contains some definitions that can be reused.
    ///
    /// # Arguments
    ///
    /// * `test_case` - The test case where a randomly generated statement should be inserted into.
    pub fn insert_random_stmt(&mut self, source_file: &SourceFile<Self>) {
        // TODO primitive statements are not being generated yet
        let callables = source_file.callables();
        let i = fastrand::usize(0..callables.len());
        let callable = callables.get(i).unwrap();

        let args: Vec<Arg> = callable
            .params()
            .iter()
            .map(|p| self.generate_argument(source_file, p, HashSet::new()))
            .collect();
        let stmt = callable.to_stmt(args);
        self.add_stmt(stmt);
    }

    fn generate_argument(
        &mut self,
        source_file: &SourceFile<Self>,
        param: &Param,
        mut types_to_generate: HashSet<T>,
    ) -> Arg {
        let mut generator = None;
        if param.is_primitive() {
            return Arg::Primitive(Primitive::U8(fastrand::u8(..)));
        } else {
            let mut generators = source_file.generators(param.ty());
            let mut retry = true;
            while retry && !generators.is_empty() {
                retry = false;

                // Pick a random generator
                let i = fastrand::usize(0..generators.len());
                let candidate = generators.get(i).unwrap().clone();
                let params = HashSet::from_iter(candidate.params().iter().map(Param::ty).cloned());

                let intersection = Vec::from_iter(params.intersection(&types_to_generate));
                if !intersection.is_empty() {
                    // We already try to generate a type which is needed as an argument for the call
                    // Hence, this would probably lead to infinite recursive chain. Remove the
                    // generator and retry with another one.
                    generators.remove(i);
                    retry = true;
                } else {
                    generator = Some(candidate);
                }
            }
        }

        if generator.is_none() {
            // No appropriate generator found
            println!("Panic! Param type: {}", param.ty().to_string());
            panic!("No generator")
        }

        let generator = generator.unwrap();
        let args: Vec<Arg> = generator
            .params()
            .iter()
            .map(|p| {
                // TODO instantiate new object with a 10% probability even if there is a free ones
                // already in the test case
                if !self.instantiated_types().contains(p.ty()) {
                    let mut types_to_generate = types_to_generate.clone();
                    types_to_generate.insert(param.ty().clone());
                    self.generate_argument(source_file, p, types_to_generate)
                } else {
                    println!("Unimplemented: Param type: {}", param.ty().to_string());
                    println!(
                        "Params: {:?}",
                        generator
                            .params()
                            .iter()
                            .map(|p| p.ty().to_string())
                            .collect::<Vec<String>>()
                    );
                    unimplemented!()
                }
            })
            .collect();

        let stmt = generator.to_stmt(args);

        let return_var = self.add_stmt(stmt).unwrap();

        Arg::Var(VarArg::new(return_var, param.clone()))
    }
}

impl Display for TestCase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let syn_item = self.to_syn();
        let token_stream = syn_item.to_token_stream();
        write!(f, "{}", token_stream.to_string())
    }
}

impl ToSyn for TestCase {
    fn to_syn(&self) -> Item {
        let ident = Ident::new(
            &format!("{}_{}", TEST_FN_PREFIX, self.id),
            Span::call_site(),
        );
        let id = self.id;

        let stmts: Vec<Stmt> = self.stmts.iter().map(Statement::to_syn).collect();

        let set_test_id: Stmt = syn::parse_quote! {
            LOGGER.with(|l| l.borrow_mut().set_test_id(#id));
        };
        let wait: Stmt = syn::parse_quote! {
            LOGGER.with(|l| l.borrow_mut().wait());
        };

        syn::parse_quote! {
            #[test]
            fn #ident() {
                #set_test_id
                #(#stmts)*
                #wait
            }
        }
    }
}

impl Chromosome for TestCase {
    fn id(&self) -> u64 {
        self.id
    }

    fn coverage(&self) -> &HashMap<Branch, FitnessValue> {
        &self.coverage
    }

    fn set_coverage(&mut self, coverage: HashMap<Branch, FitnessValue>) {
        self.coverage = coverage;
    }

    fn mutate<M: Mutation<C = Self>>(&self, mutation: &M) -> Self {
        let mut mutated_test = mutation.apply(self);
        mutated_test.id = CHROMOSOME_ID_GENERATOR.lock().unwrap().next_id();
        mutated_test
    }

    fn fitness(&self, objective: &Branch) -> FitnessValue {
        objective.fitness(self)
    }

    fn crossover<C: Crossover<C = Self>>(&self, other: &Self, crossover: &C) -> (Self, Self)
    where
        Self: Sized,
    {
        let (mut left, mut right) = crossover.apply(self, other);
        left.id = CHROMOSOME_ID_GENERATOR.lock().unwrap().next_id();
        right.id = CHROMOSOME_ID_GENERATOR.lock().unwrap().next_id();
        (left, right)
    }

    fn random(source_file: &SourceFile<Self>) -> Self {
        let test_id = CHROMOSOME_ID_GENERATOR.lock().as_mut().unwrap().next_id();

        let mut test_case = TestCase::new(test_id);

        // TODO fill test case with statements until a certain length
        test_case.insert_random_stmt(source_file);

        test_case
    }

    fn size(&self) -> usize {
        self.stmts.len()
    }
}

#[derive(Debug, Clone)]
pub enum Arg {
    Var(VarArg),
    Primitive(Primitive),
}

impl Arg {
    pub fn to_syn(&self) -> Expr {
        match self {
            Arg::Var(var_arg) => var_arg.to_syn(),
            Arg::Primitive(primitive_arg) => primitive_arg.to_syn(),
        }
    }

    fn decorate_reference(expr: Expr) -> Expr {
        syn::parse_quote! {
            &#expr
        }
    }

    fn decorate_mut(expr: Expr) -> Expr {
        syn::parse_quote! {
            mut #expr
        }
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    U8(u8),
}

impl Primitive {
    pub fn ty(&self) -> Box<Type> {
        match self {
            Primitive::U8(_) => Box::new(syn::parse_quote!(u8)),
        }
    }

    pub fn to_syn(&self) -> Expr {
        match self {
            Primitive::U8(val) => syn::parse_quote! {#val},
        }
    }
}

#[derive(Debug, Clone)]
pub struct VarArg {
    param: Param,
    var: Var,
}

impl VarArg {
    pub fn new(var: Var, param: Param) -> Self {
        VarArg { param, var }
    }

    pub fn param(&self) -> &Param {
        &self.param
    }

    pub fn var(&self) -> &Var {
        &self.var
    }

    pub fn is_consuming(&self) -> bool {
        !self.param.by_reference()
    }

    pub fn is_by_reference(&self) -> bool {
        self.param.by_reference()
    }

    pub fn is_self(&self) -> bool {
        self.param.is_self()
    }

    pub fn to_syn(&self) -> Expr {
        let expr = self.var.to_syn();
        if self.param().mutable() {
            // mutable
            if self.param().by_reference() {
                // by reference
                syn::parse_quote! {
                    &mut #expr
                }
            } else {
                // mutable and without reference
                panic!("Should not occur");
            }
        } else {
            // non-mutable
            if self.param().by_reference() {
                syn::parse_quote! {
                    &#expr
                }
            } else {
                expr
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Statement {
    PrimitiveAssignment(AssignStmt),
    Constructor(ConstructorStmt),
    AttributeAccess(AttrStmt),
    MethodInvocation(MethodInvStmt),
    StaticFnInvocation(StaticFnInvStmt),
    FunctionInvocation(FnInvStmt),
}

impl Statement {
    pub fn to_syn(&self) -> Stmt {
        match self {
            Statement::PrimitiveAssignment(primitive_stmt) => primitive_stmt.to_syn(),
            Statement::Constructor(constructor_stmt) => constructor_stmt.to_syn(),
            Statement::AttributeAccess(_) => {
                unimplemented!()
            }
            Statement::StaticFnInvocation(fn_inv_stmt) => fn_inv_stmt.to_syn(),
            Statement::MethodInvocation(method_inv_stmt) => method_inv_stmt.to_syn(),
            Statement::FunctionInvocation(fn_inv_stmt) => fn_inv_stmt.to_syn(),
        }
    }

    pub fn returns_value(&self) -> bool {
        match self {
            Statement::Constructor(_) => true,
            Statement::StaticFnInvocation(func) => func.returns_value(),
            Statement::PrimitiveAssignment(_) => true,
            Statement::FunctionInvocation(func) => func.returns_value(),
            Statement::MethodInvocation(m) => m.returns_value(),
            _ => unimplemented!(),
        }
    }

    pub fn var(&self) -> Option<&Var> {
        if !self.returns_value() {
            panic!("Statement does not return anything");
        }

        match self {
            Statement::Constructor(c) => c.var(),
            Statement::MethodInvocation(m) => m.var(),
            Statement::StaticFnInvocation(func) => func.var(),
            Statement::FunctionInvocation(func) => func.var(),
            Statement::PrimitiveAssignment(a) => a.var(),
            _ => unimplemented!(),
        }
    }

    pub fn return_type(&self) -> Option<&T> {
        match self {
            Statement::PrimitiveAssignment(a) => Some(a.return_type()),
            Statement::Constructor(c) => Some(c.return_type()),
            Statement::MethodInvocation(m) => m.return_type(),
            Statement::StaticFnInvocation(f) => f.return_type(),
            Statement::FunctionInvocation(f) => f.return_type(),
            Statement::AttributeAccess(a) => a.return_type(),
        }
    }

    pub fn set_var(&mut self, var: Var) {
        if !self.returns_value() {
            panic!("Statement does not return any value")
        }

        match self {
            Statement::PrimitiveAssignment(ref mut p) => p.set_var(var),
            Statement::Constructor(ref mut c) => c.set_var(var),
            Statement::AttributeAccess(ref mut a) => a.set_var(var),
            Statement::MethodInvocation(ref mut m) => m.set_var(var),
            Statement::StaticFnInvocation(ref mut f) => f.set_var(var),
            Statement::FunctionInvocation(ref mut f) => f.set_var(var),
        }
    }

    pub fn args(&self) -> Option<&Vec<Arg>> {
        match self {
            Statement::PrimitiveAssignment(_) => None,
            Statement::Constructor(c) => Some(c.args()),
            Statement::AttributeAccess(_) => None,
            Statement::MethodInvocation(m) => Some(m.args()),
            Statement::StaticFnInvocation(f) => Some(f.args()),
            Statement::FunctionInvocation(f) => Some(f.args()),
        }
    }

    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        match self {
            Statement::PrimitiveAssignment(_) => panic!("Primitives do not have args"),
            Statement::Constructor(c) => c.set_arg(arg, idx),
            Statement::AttributeAccess(_) => panic!("Attribute access cannot have args"),
            Statement::MethodInvocation(m) => m.set_arg(arg, idx),
            Statement::StaticFnInvocation(f) => f.set_arg(arg, idx),
            Statement::FunctionInvocation(f) => f.set_arg(arg, idx)
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Statement::PrimitiveAssignment(p) => p.id(),
            Statement::Constructor(c) => c.id(),
            Statement::AttributeAccess(a) => a.id(),
            Statement::MethodInvocation(m) => m.id(),
            Statement::StaticFnInvocation(f) => f.id(),
            Statement::FunctionInvocation(f) => f.id(),
        }
    }
}

#[derive(Debug)]
pub struct StaticFnInvStmt {
    id: Uuid,
    args: Vec<Arg>,
    func: StaticFnItem,
    var: Option<Var>,
}

impl Clone for StaticFnInvStmt {
    fn clone(&self) -> Self {
        StaticFnInvStmt {
            id: Uuid::new_v4(),
            args: self.args.clone(),
            func: self.func.clone(),
            var: self.var.clone(),
        }
    }
}

impl StaticFnInvStmt {
    pub fn new(func: StaticFnItem, args: Vec<Arg>) -> Self {
        StaticFnInvStmt {
            id: Uuid::new_v4(),
            args,
            func,
            var: None,
        }
    }

    pub fn return_type(&self) -> Option<&T> {
        self.func.return_type.as_ref()
    }

    pub fn returns_value(&self) -> bool {
        self.func.return_type.is_some()
    }

    pub fn to_syn(&self) -> Stmt {
        let func_ident = &self.func.impl_item_method.sig.ident;
        let args: Vec<Expr> = self.args.iter().cloned().map(|a| a.to_syn()).collect();
        let parent_ident = Ident::new(&self.func.parent.to_string(), Span::call_site());

        if self.returns_value() {
            if let Some(var) = &self.var {
                let ident = Ident::new(&var.to_string(), Span::call_site());
                syn::parse_quote! {
                    let #ident = #parent_ident::#func_ident(#(#args),*);
                }
            } else {
                panic!("Name must have been set before")
            }
        } else {
            syn::parse_quote! {
                #parent_ident::#func_ident(#(#args),*);
            }
        }
    }
    pub fn var(&self) -> Option<&Var> {
        self.var.as_ref()
    }


    pub fn set_var(&mut self, var: Var) {
        self.var = Some(var);
    }
    pub fn args(&self) -> &Vec<Arg> {
        &self.args
    }
    pub fn func(&self) -> &StaticFnItem {
        &self.func
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        self.args[idx] = arg;
    }
}

#[derive(Debug)]
pub struct AssignStmt {
    id: Uuid,
    var: Option<Var>,
    primitive: PrimitiveItem,
}

impl AssignStmt {
    pub fn new(primitive: PrimitiveItem) -> Self {
        AssignStmt {
            id: Uuid::new_v4(),
            var: None,
            primitive,
        }
    }

    pub fn var(&self) -> Option<&Var> {
        self.var.as_ref()
    }

    pub fn return_type(&self) -> &T {
        &self.primitive.ty
    }

    pub fn set_var(&mut self, var: Var) {
        self.var = Some(var);
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn to_syn(&self) -> Stmt {
        unimplemented!()
    }
}

impl Clone for AssignStmt {
    fn clone(&self) -> Self {
        AssignStmt {
            id: Uuid::new_v4(),
            var: self.var.clone(),
            primitive: self.primitive.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ConstructorStmt {
    id: Uuid,
    var: Option<Var>,
    constructor: ConstructorItem,
    args: Vec<Arg>,
}

impl Clone for ConstructorStmt {
    fn clone(&self) -> Self {
        ConstructorStmt {
            id: Uuid::new_v4(),
            var: self.var.clone(),
            constructor: self.constructor.clone(),
            args: self.args.clone(),
        }
    }
}

impl ConstructorStmt {
    pub fn new(constructor: ConstructorItem, args: Vec<Arg>) -> Self {
        ConstructorStmt {
            var: None,
            constructor,
            args,
            id: Uuid::new_v4(),
        }
    }

    pub fn to_syn(&self) -> Stmt {
        if let Some(var) = &self.var {
            let ident = Ident::new(&var.to_string(), Span::call_site());

            let type_name = Ident::new(&self.constructor.parent.to_string(), Span::call_site());
            let args: Vec<Expr> = self.args.iter().map(|a| a.to_syn()).collect();
            /*println!("{:?}", args);
            println!("{}", type_name);
            println!("{:?}", ident);*/
            syn::parse_quote! {
                let mut #ident = #type_name::new(#(#args),*);
            }
        } else {
            panic!("Name must have been set until here")
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn name(&self) -> Option<&Var> {
        self.var.as_ref()
    }

    pub fn params(&self) -> &Vec<Param> {
        self.constructor.params()
    }
    pub fn args(&self) -> &Vec<Arg> {
        &self.args
    }

    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        self.args[idx] = arg;
    }

    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn set_var(&mut self, var: Var) {
        self.var = Some(var.clone());
    }

    pub fn var(&self) -> Option<&Var> {
        self.var.as_ref()
    }
    pub fn constructor(&self) -> &ConstructorItem {
        &self.constructor
    }

    pub fn returns_value(&self) -> bool {
        true
    }

    pub fn return_type(&self) -> &T {
        &self.constructor.return_type
    }
}

#[derive(Debug)]
pub struct AttrStmt {
    id: Uuid,
}

impl Clone for AttrStmt {
    fn clone(&self) -> Self {
        AttrStmt { id: Uuid::new_v4() }
    }
}

impl AttrStmt {
    pub fn new() -> Self {
        AttrStmt { id: Uuid::new_v4() }
    }

    pub fn return_type(&self) -> Option<&T> {
        unimplemented!()
    }

    pub fn set_var(&mut self, var: Var) {
        unimplemented!()
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug)]
pub struct MethodInvStmt {
    id: Uuid,
    var: Option<Var>,
    method: MethodItem,
    args: Vec<Arg>,
}

impl Clone for MethodInvStmt {
    fn clone(&self) -> Self {
        MethodInvStmt {
            id: Uuid::new_v4(),
            var: self.var.clone(),
            method: self.method.clone(),
            args: self.args.clone(),
        }
    }
}

impl MethodInvStmt {
    pub fn new(method: MethodItem, args: Vec<Arg>) -> Self {
        MethodInvStmt {
            method,
            args,
            id: Uuid::new_v4(),
            var: None,
        }
    }

    pub fn callee(&self) -> &Arg {
        // Associative method call must always have a callee
        self.args.first().unwrap()
    }

    pub fn set_callee(&mut self, callee: Arg) {
        if self.args.is_empty() {
            self.args.push(callee);
        } else {
            std::mem::replace(&mut self.args[0], callee);
        }
    }

    pub fn to_syn(&self) -> Stmt {
        let method_ident = &self.method.impl_item_method.sig.ident;
        let args: Vec<Expr> = self.args.iter().map(Arg::to_syn).collect();
        let parent_ident = Ident::new(&self.method.parent.to_string(), Span::call_site());

        if self.returns_value() {
            if let Some(var) = &self.var {
                let ident = Ident::new(&var.to_string(), Span::call_site());
                syn::parse_quote! {
                    let #ident = #parent_ident::#method_ident(#(#args),*);
                }
            } else {
                panic!("Name must have been set before")
            }
        } else {
            syn::parse_quote! {
                #parent_ident::#method_ident(#(#args),*);
            }
        }
    }

    pub fn returns_value(&self) -> bool {
        self.method.return_type.is_some()
    }

    pub fn return_type(&self) -> Option<&T> {
        self.method.return_type.as_ref()
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn owner(&self) -> &Var {
        let first_arg = self.args.first().unwrap();
        if let Arg::Var(var_arg) = first_arg {
            if var_arg.is_self() {
                &var_arg.var
            } else {
                panic!("There should be an owner")
            }
        } else {
            panic!("First arg must be a variable")
        }
    }

    pub fn method(&self) -> &MethodItem {
        &self.method
    }
    pub fn params(&self) -> &Vec<Param> {
        &self.method.params
    }
    pub fn args(&self) -> &Vec<Arg> {
        &self.args
    }
    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        self.args[idx] = arg;
    }
    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn var(&self) -> Option<&Var> {
        self.var.as_ref()
    }
    pub fn set_var(&mut self, var: Var) {
        self.var = Some(var);
    }
}

#[derive(Debug)]
pub struct FnInvStmt {
    id: Uuid,
    args: Vec<Arg>,
    func: FunctionItem,
    var: Option<Var>,
}

impl Clone for FnInvStmt {
    fn clone(&self) -> Self {
        FnInvStmt {
            id: Uuid::new_v4(),
            args: self.args.clone(),
            func: self.func.clone(),
            var: self.var.clone(),
        }
    }
}

impl FnInvStmt {
    pub fn new(func: FunctionItem, args: Vec<Arg>) -> Self {
        FnInvStmt {
            args,
            func,
            id: Uuid::new_v4(),
            var: None,
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.func.params
    }
    pub fn args(&self) -> &Vec<Arg> {
        &self.args
    }

    pub fn func(&self) -> &FunctionItem {
        &self.func
    }

    pub fn has_params(&self) -> bool {
        !self.func.params.is_empty()
    }

    pub fn returns_value(&self) -> bool {
        unimplemented!()
    }

    pub fn return_type(&self) -> Option<&T> {
        self.func.return_type.as_ref()
    }

    pub fn var(&self) -> Option<&Var> {
        self.var.as_ref()
    }

    pub fn set_var(&mut self, var: Var) {
        self.var = Some(var);
    }

    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        self.args[idx] = arg;
    }

    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn to_syn(&self) -> Stmt {
        let ident = &self.func.item_fn.sig.ident;
        let args: Vec<Expr> = self.args.iter().map(Arg::to_syn).collect();

        syn::parse_quote! {
            #ident(#(#args),*);
        }
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Var {
    name: String,
    ty: T,
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Var {
    pub fn new(name: &str, ty: T) -> Self {
        Var {
            name: name.to_owned(),
            ty,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &T {
        &self.ty
    }

    pub fn to_syn(&self) -> Expr {
        let ident = Ident::new(self.name(), Span::call_site());
        syn::parse_quote! {
            #ident
        }
    }
}

#[derive(Debug, Clone)]
pub enum Param {
    Self_(SelfParam),
    Regular(RegularParam),
}

impl Param {
    pub fn is_self(&self) -> bool {
        match self {
            Param::Self_(_) => true,
            Param::Regular(_) => false,
        }
    }

    pub fn by_reference(&self) -> bool {
        match self {
            Param::Self_(self_param) => self_param.by_reference(),
            Param::Regular(regular_param) => regular_param.by_reference(),
        }
    }

    pub fn mutable(&self) -> bool {
        match self {
            Param::Self_(self_param) => self_param.mutable(),
            Param::Regular(regular_param) => regular_param.mutable(),
        }
    }

    pub fn ty(&self) -> &T {
        match self {
            Param::Self_(self_param) => &self_param.ty,
            Param::Regular(regular_param) => &regular_param.ty,
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            Param::Self_(_) => false,
            Param::Regular(regular_param) => {
                let ty = regular_param.ty();
                ty.to_string() == "u8"
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelfParam {
    ty: T,
    fn_arg: FnArg,
    by_reference: bool,
    mutable: bool,
}

impl SelfParam {
    pub fn new(ty: T, fn_arg: FnArg, by_reference: bool, mutable: bool) -> Self {
        SelfParam {
            ty,
            fn_arg,
            by_reference,
            mutable,
        }
    }

    pub fn by_reference(&self) -> bool {
        self.by_reference
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }
}

#[derive(Debug, Clone)]
pub struct RegularParam {
    ty: T,
    fn_arg: FnArg,
    by_reference: bool,
    mutable: bool,
}

impl RegularParam {
    pub fn new(ty: T, fn_arg: FnArg, by_reference: bool, mutable: bool) -> Self {
        RegularParam {
            ty,
            fn_arg,
            by_reference,
            mutable,
        }
    }

    pub fn ty(&self) -> &T {
        &self.ty
    }
    pub fn fn_arg(&self) -> &FnArg {
        &self.fn_arg
    }

    pub fn by_reference(&self) -> bool {
        self.by_reference
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }
}

#[derive(Debug, Clone, Hash, Eq)]
pub struct T {
    name: String,

    // syn defined type
    ty: Box<Type>,
}

impl PartialEq for T {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl T {
    pub fn new(name: &str, ty: Box<Type>) -> Self {
        T {
            name: name.to_owned(),
            ty,
        }
    }
}

impl Display for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub enum Callable {
    Method(MethodItem),
    StaticFunction(StaticFnItem),
    Function(FunctionItem),
    Constructor(ConstructorItem),
    Primitive(PrimitiveItem),
}

impl Callable {
    pub fn params(&self) -> &Vec<Param> {
        match self {
            Callable::Method(method_item) => &method_item.params,
            Callable::StaticFunction(fn_item) => &fn_item.params,
            Callable::Function(fn_item) => &fn_item.params,
            Callable::Constructor(constructor_item) => &constructor_item.params,
            Callable::Primitive(primitive_item) => primitive_item.params(),
        }
    }

    pub fn return_type(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => method_item.return_type.as_ref(),
            Callable::StaticFunction(fn_item) => fn_item.return_type.as_ref(),
            Callable::Function(fn_item) => fn_item.return_type.as_ref(),
            Callable::Constructor(constructor_item) => Some(&constructor_item.return_type),
            Callable::Primitive(primitive_item) => Some(&primitive_item.ty),
        }
    }

    pub fn parent(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => Some(&method_item.parent),
            Callable::StaticFunction(fn_item) => Some(&fn_item.parent),
            Callable::Function(_) => None,
            Callable::Constructor(constructor) => Some(&constructor.parent),
            Callable::Primitive(_) => None,
        }
    }

    pub fn to_stmt(&self, args: Vec<Arg>) -> Statement {
        match self {
            Callable::Method(method_item) => {
                Statement::MethodInvocation(MethodInvStmt::new(method_item.clone(), args))
            }
            Callable::StaticFunction(fn_item) => {
                Statement::StaticFnInvocation(StaticFnInvStmt::new(fn_item.clone(), args))
            }
            Callable::Function(fn_item) => {
                Statement::FunctionInvocation(FnInvStmt::new(fn_item.clone(), args))
            }
            Callable::Constructor(constructor_item) => {
                Statement::Constructor(ConstructorStmt::new(constructor_item.clone(), args))
            }
            Callable::Primitive(primitive_item) => {
                Statement::PrimitiveAssignment(AssignStmt::new(primitive_item.clone()))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrimitiveItem {
    ty: T,
    params: Vec<Param>,
}

impl PrimitiveItem {
    pub fn new(ty: T) -> PrimitiveItem {
        PrimitiveItem { ty, params: vec![] }
    }

    pub fn params(&self) -> &Vec<Param> {
        // Just for compilation reasons
        &self.params
    }
}

#[derive(Debug, Clone)]
pub struct MethodItem {
    params: Vec<Param>,
    return_type: Option<T>,
    parent: T,
    impl_item_method: ImplItemMethod,
}

impl MethodItem {
    /// Creates a new method of a struct or enum
    ///
    /// # Arguments
    ///
    /// * `impl_item_method` - The original syn container for the method
    /// * `ty` - The parent struct or enum of the method
    ///
    pub fn new(impl_item_method: ImplItemMethod, ty: Box<Type>) -> Self {
        let sig = &impl_item_method.sig;
        let params: Vec<Param> = sig
            .inputs
            .iter()
            .map(|input| util::fn_arg_to_param(input, ty.clone()))
            .collect();

        let parent = T::new(&type_name(ty.as_ref()), ty.clone());

        let return_type = match &sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(T::new(&type_name(ty.as_ref()), ty.clone())),
        };

        MethodItem {
            params,
            parent,
            return_type,
            impl_item_method,
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
    pub fn return_type(&self) -> Option<&T> {
        self.return_type.as_ref()
    }
    pub fn parent(&self) -> &T {
        &self.parent
    }

    pub fn impl_item_method(&self) -> &ImplItemMethod {
        &self.impl_item_method
    }
}

#[derive(Debug, Clone)]
pub struct FunctionItem {
    params: Vec<Param>,
    return_type: Option<T>,
    item_fn: ItemFn,
}

impl FunctionItem {
    pub fn new(item_fn: ItemFn) -> Self {
        let sig = &item_fn.sig;
        let params: Vec<Param> = sig
            .inputs
            .iter()
            .map(|input| {
                let ty = match input {
                    FnArg::Receiver(_) => panic!("Should occur"),
                    FnArg::Typed(pat_type) => pat_type.ty.clone(),
                };

                fn_arg_to_param(input, ty)
            })
            .collect();

        let return_type = match &sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(T::new(&type_name(ty.as_ref()), ty.clone())),
        };

        FunctionItem {
            params,
            return_type,
            item_fn,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaticFnItem {
    params: Vec<Param>,
    return_type: Option<T>,
    parent: T,
    impl_item_method: ImplItemMethod,
}

impl StaticFnItem {
    pub fn new(impl_item_method: ImplItemMethod, ty: Box<Type>) -> Self {
        let sig = &impl_item_method.sig;
        let params: Vec<Param> = sig
            .inputs
            .iter()
            .map(|input| fn_arg_to_param(input, ty.clone()))
            .collect();

        let return_type = match &sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(T::new(&type_name(ty.as_ref()), ty.clone())),
        };

        let parent = T::new(&type_name(ty.as_ref()), ty.clone());

        StaticFnItem {
            params,
            parent,
            return_type,
            impl_item_method,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstructorItem {
    params: Vec<Param>,
    return_type: T,
    parent: T,
    impl_item_method: ImplItemMethod,
}

impl ConstructorItem {
    pub fn new(impl_item_method: ImplItemMethod, ty: Box<Type>) -> Self {
        let sig = &impl_item_method.sig;
        let params: Vec<Param> = sig
            .inputs
            .iter()
            .map(|input| fn_arg_to_param(input, ty.clone()))
            .collect();
        let parent = T::new(&type_name(ty.as_ref()), ty.clone());

        ConstructorItem {
            impl_item_method,
            parent,
            params,
            return_type: T::new(&type_name(ty.as_ref()), ty.clone()),
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        self.params.as_ref()
    }
    pub fn return_type(&self) -> &T {
        &self.return_type
    }
    pub fn parent(&self) -> &T {
        &self.parent
    }
    pub fn impl_item_method(&self) -> &ImplItemMethod {
        &self.impl_item_method
    }
}

#[derive(Debug, Clone)]
pub struct Struct {
    ident: Ident,
    constructor: Option<ConstructorItem>,
    methods: Vec<MethodItem>,
    static_methods: Vec<StaticFnItem>,
}

impl Struct {
    pub fn new(ident: Ident) -> Self {
        Struct {
            ident,
            constructor: None,
            methods: Vec::new(),
            static_methods: Vec::new(),
        }
    }

    pub fn constructor(&self) -> &Option<ConstructorItem> {
        &self.constructor
    }

    pub fn methods(&self) -> &Vec<MethodItem> {
        &self.methods
    }
    pub fn static_methods(&self) -> &Vec<StaticFnItem> {
        &self.static_methods
    }
    pub fn set_constructor(&mut self, constructor: ConstructorItem) {
        self.constructor = Some(constructor);
    }

    pub fn add_method(&mut self, method: MethodItem) {
        self.methods.push(method);
    }

    pub fn add_static_method(&mut self, method: StaticFnItem) {
        self.static_methods.push(method);
    }

    pub fn has_constructor(&self) -> bool {
        if let Some(_) = &self.constructor {
            true
        } else {
            false
        }
    }
    pub fn ident(&self) -> &Ident {
        &self.ident
    }
}
