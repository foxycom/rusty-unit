use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::RangeInclusive;
use std::rc::Rc;
use petgraph::Direction;
use petgraph::prelude::StableDiGraph;
use petgraph::stable_graph::NodeIndex;
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use rustc_hir::def_id::DefId;
use rustc_hir::{BodyId, FnSig, HirId, PrimTy};
use rustc_middle::ty::TyCtxt;
use syn::{Expr, ImplItemMethod, Item, ItemEnum, ItemStruct, Stmt, Type};
use uuid::Uuid;
use std::sync::{Mutex, Arc};
use crate::analysis::HirAnalysis;
use crate::branch::Branch;
use crate::fitness::FitnessValue;
use crate::operators::{Crossover, Mutation};
use crate::types::{Callable, ConstructorItem, FieldAccessItem, FunctionItem, MethodItem, Param, PrimitiveItem, StaticFnItem, T};
use crate::generators::TestIdGenerator;

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
    fn random(analysis: Rc<HirAnalysis>) -> Self;

    fn size(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    Owns,
    Consumes,
    Borrows,
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
    analysis: Rc<HirAnalysis>,
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
            analysis: self.analysis.clone(),
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
    pub fn new(id: u64, analysis: Rc<HirAnalysis>) -> Self {
        TestCase {
            id,
            stmts: Vec::new(),
            coverage: HashMap::new(),
            ddg: StableDiGraph::new(),
            var_table: HashMap::new(),
            node_index_table: HashMap::new(),
            var_counters: HashMap::new(),
            analysis,
        }
    }

    pub fn stmts(&self) -> &Vec<Statement> {
        &self.stmts
    }

    pub fn is_cutable(&self) -> bool {
        self.size() > 1
    }

    fn set_var(&mut self, stmt: &mut Statement) -> Option<Var> {
        if let Some(return_type) = stmt.return_type() {
            let type_name = return_type.var_string();
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

    pub fn insert_stmt(&mut self, idx: usize, mut stmt: Statement) -> Option<Var> {
        let var = self.set_var(&mut stmt);
        let uuid = stmt.id();

        // Save to DDG
        let node_index = self.ddg.add_node(uuid);
        self.node_index_table.insert(uuid, node_index.clone());

        if let Some(args) = stmt.args() {
            args.iter().for_each(|arg| {
                if let Arg::Var(var_arg) = arg {
                    let generating_stmt_index = self.var_position(var_arg.var()).unwrap();
                    let generating_stmt_id = self.stmts.get(generating_stmt_index).unwrap().id();
                    let arg_node_index = self.node_index_table.get(&generating_stmt_id).unwrap();

                    if var_arg.is_by_reference() {
                        self.ddg
                            .add_edge(node_index, arg_node_index.clone(), Dependency::Borrows);
                    } else {
                        self.ddg
                            .add_edge(node_index, arg_node_index.clone(), Dependency::Consumes);
                    }
                }
            });
        }

        self.stmts.insert(idx, stmt);

        if let Some(var) = &var {
            self.var_table.insert(var.clone(), uuid);
        }

        assert_eq!(self.stmts.len(), self.ddg.node_count());

        var
    }

    pub fn add_stmt(&mut self, stmt: Statement) -> Option<Var> {
        let mut insert_position: usize = 0;
        if let Some(args) = stmt.args() {
            insert_position = args
                .iter()
                .filter_map(|a| {
                    if let Arg::Var(var_arg) = a {
                        return self.var_position(var_arg.var());
                    }
                    None
                })
                .fold(0usize, |a, b| a.max(b));
        }

        let length = self.stmts.len();
        self.insert_stmt(length.min(insert_position + 1), stmt)
    }

    pub fn remove_stmt(&mut self, stmt_id: Uuid) -> usize {
        let position = self.stmts.iter().position(|s| s.id() == stmt_id).unwrap();
        self.remove_stmt_at(position);

        position
    }

    pub fn remove_stmt_at(&mut self, idx: usize) {
        assert_eq!(self.stmts.len(), self.ddg.node_count());

        let stmt = self.stmts.remove(idx);

        let id = stmt.id();

        if stmt.returns_value() {
            self.var_table.remove(stmt.var().unwrap()).unwrap();
        }

        let node_index = match self.node_index_table.remove(&id) {
            None => {
                println!("\nFailing test: {}", self.id);
                //self.to_file(tcx);
                panic!()
            }
            Some(node_index) => node_index,
        };
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
        self.ddg.remove_node(node_index).unwrap();

        assert!(self.stmts.len() >= self.var_table.len());
        assert_eq!(self.stmts.len(), self.ddg.node_count());
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
        let stmt_id = self.var_table.get(var).unwrap();
        let node_index = self.node_index_table.get(stmt_id).unwrap();

        let mut neighbours = self.ddg.neighbors(node_index.clone());

        // Find the first place where a variable is consumed before or borrowed or consumed after idx
        let collision = neighbours.find(|n| {
            let edge_index = self.ddg.find_edge(n.clone(), node_index.clone()).unwrap();
            let edge = self.ddg.edge_weight(edge_index).unwrap();

            match edge {
                Dependency::Owns => false,
                Dependency::Consumes => true,
                Dependency::Borrows => {
                    let neighbour_id = self.ddg.node_weight(n.clone()).unwrap();
                    let neighbour_stmt_i = self.stmt_position(neighbour_id.clone()).unwrap();
                    if neighbour_stmt_i > idx {
                        return true;
                    }
                    false
                }
            }
        });

        // If there is none, a var is borrowable at index idx
        collision.is_none()
    }

    pub fn is_borrowable(&self, var: &Var, idx: usize) -> bool {
        let stmt_id = self.var_table.get(var).unwrap();
        let node_index = self.node_index_table.get(stmt_id).unwrap();

        let mut neighbours = self.ddg.neighbors(node_index.clone());

        // Find the first place where borrowing and consume collide
        let collision = neighbours.find(|n| {
            let edge_index = self.ddg.find_edge(n.clone(), node_index.clone()).unwrap();
            let edge = self.ddg.edge_weight(edge_index).unwrap();
            if *edge == Dependency::Consumes {
                let neighbour_id = self.ddg.node_weight(n.clone()).unwrap();
                let neighbour_stmt_i = self.stmt_position(neighbour_id.clone()).unwrap();
                if neighbour_stmt_i < idx {
                    return true;
                }
            }
            false
        });

        // If there is none, a var is borrowable at index idx
        collision.is_none()
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
            })
            .collect()
    }

    pub fn is_consumed(&self, var: &Var) -> bool {
        let uuid = self.var_table.get(var).unwrap();
        let node_index = self.node_index_table.get(uuid).unwrap();
        let mut incoming_edges = self
            .ddg
            .edges_directed(node_index.to_owned(), Direction::Incoming);
        incoming_edges.any(|e| e.weight().to_owned() == Dependency::Consumes)
    }

    pub fn instantiated_at(&self, var: &Var) -> Option<usize> {
        let id = self.var_table.get(var);

        id.map(|id| {
            let pos = self.stmts.iter().position(|s| s.id() == *id);
            if pos.is_none() {
                //self.to_file();
                panic!("\nLooking for var {} in test {}", var.name, self.id);
            } else {
                return pos.unwrap();
            }
        })
    }

    pub fn consumed_at(&self, var: &Var) -> Option<usize> {
        let id = self.var_table.get(var).unwrap();
        let node_index = self.node_index_table.get(id).unwrap();
        let mut neighbours = self
            .ddg
            .neighbors_directed(node_index.clone(), Direction::Incoming);

        let consuming_stmt_index = neighbours.find(|n| {
            let edge = self.ddg.find_edge(node_index.clone(), n.clone());
            if let Some(edge_index) = edge {
                let weight = self.ddg.edge_weight(edge_index).unwrap();
                return *weight == Dependency::Consumes;
            }
            false
        });

        if let Some(consuming_stmt_index) = consuming_stmt_index {
            let consuming_stmt_id = self.ddg.node_weight(consuming_stmt_index);
            if let Some(id) = consuming_stmt_id {
                return self.stmt_position(id.clone());
            }
        }
        None
    }

    pub fn borrowed_at(&self, var: &Var) -> Vec<usize> {
        let id = self.var_table.get(var).unwrap();
        let node_index = self.node_index_table.get(id).unwrap();
        let mut neighbours = self
            .ddg
            .neighbors_directed(node_index.clone(), Direction::Incoming);
        neighbours
            .filter_map(|n| {
                let edge = self.ddg.find_edge(node_index.clone(), n.clone());
                if let Some(edge_index) = edge {
                    let weight = self.ddg.edge_weight(edge_index).unwrap();
                    if *weight == Dependency::Borrows {
                        let id = self.ddg.node_weight(n.clone()).unwrap();

                        return Some(self.stmt_position(id.clone()).unwrap());
                    }
                }
                None
            })
            .collect()
    }

    /// Returns the callables and where they can be called in the test case.
    ///
    /// There can be 3 cases for a variable:
    /// 1) It can be defined and not used
    ///
    /// ```
    /// fn test() {
    ///     let a = A::new();
    ///     // ...
    ///     // a is just instantiated and not used any longer. Therefore, it can be both consumed
    ///     // and borrowed at p with 1 < p < test.len().
    /// }
    /// ```
    ///
    /// 2) It can be defined and borrowed one or multiple times
    /// ```
    /// fn test() {
    ///     let a = A::new();
    ///     // ...
    ///     borrow(&a);
    ///     // ...
    ///     // a can be borrowed at any position p with 1 < p < test.len()
    ///     // AND a can be consumed at p with  pos(borrow) < p < test.len()
    /// }
    /// ```
    ///
    /// 3) It can be defined, (probably borrowed) and consumed
    /// ```
    /// fn test() {
    ///     let a = A::new();
    ///     // ...
    ///     borrow(&a);
    ///     // ...
    ///     consume(a);
    ///     // ...
    ///     // a can be only be borrowed at position p with 1 < p < pos(consume)
    /// }
    /// ```
    pub fn available_callables(&self) -> Vec<(&Var, &Callable, RangeInclusive<usize>)> {
        self.var_table
            .keys()
            .filter_map(|v| {
                let mut callables = self.analysis.callables_of(v.ty());

                if let Some(idx) = self.consumed_at(v) {
                    // v can only be borrowed
                    let range = self.instantiated_at(v).unwrap()..=idx;
                    // Retain only callables that are borrowing
                    let possible_callables: Vec<(&Var, &Callable, RangeInclusive<usize>)> =
                        callables
                            .iter()
                            .filter_map(|&c| {
                                if let Callable::Method(method_item) = c {
                                    if !method_item.consumes_parent() {
                                        return Some((v, c, range.clone()));
                                    }
                                }
                                None
                            })
                            .collect();
                    return Some(possible_callables);
                } else if let borrowed_indices = self.borrowed_at(v) {
                    return if !borrowed_indices.is_empty() {
                        // v can be borrowed and consumed at certain positions
                        let range_borrow = self.instantiated_at(v).unwrap()..=self.size();

                        let last_borrow_i = borrowed_indices.iter().max().unwrap().to_owned();
                        let range_consume = last_borrow_i..=self.size();
                        let callables = callables
                            .iter()
                            .filter_map(|&c| {
                                if let Callable::Method(method_item) = c {
                                    return if method_item.consumes_parent() {
                                        Some((v, c, range_consume.clone()))
                                    } else {
                                        Some((v, c, range_borrow.clone()))
                                    };
                                }
                                None
                            })
                            .collect();
                        Some(callables)
                    } else {
                        // v can be borrowed and consumed freely
                        let range = self.instantiated_at(v).unwrap()..=self.size();
                        let callables = callables.iter().map(|&c| (v, c, range.clone())).collect();
                        Some(callables)
                    };
                }

                unimplemented!()
            })
            .flatten()
            .collect()
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

    /// Generates a random statement and all its dependencies, even if the
    /// test already contains some definitions that can be reused.
    pub fn insert_random_stmt(&mut self) {
        // TODO primitive statements are not being generated yet
        let callables = self.analysis.callables();
        let i = fastrand::usize(0..callables.len());
        let callable = (*(callables.get(i).unwrap())).clone();
        println!("Selected callable: {:?}", callable);

        let args: Vec<Arg> = callable
            .params()
            .iter()
            .map(|p| self.generate_arg(p))
            .collect();
        let stmt = callable.to_stmt(args);
        self.add_stmt(stmt);
    }

    pub fn generate_arg(&mut self, param: &Param) -> Arg {
        self.generate_arg_inner(param, HashSet::new())
    }

    fn generate_arg_inner(&mut self, param: &Param, mut types_to_generate: HashSet<T>) -> Arg {
        let mut generator = None;
        if param.is_primitive() {
            return Arg::Primitive(Primitive::U8(fastrand::u8(..)));
        } else {
            let mut generators = self.analysis.generators(param.real_ty());

            // TODO remove
            println!("Generators for {:?}", param.real_ty());
            println!("{:?}", generators);

            let mut retry = true;
            while retry && !generators.is_empty() {
                retry = false;

                // Pick a random generator
                let i = fastrand::usize(0..generators.len());
                let candidate = generators.get(i).unwrap().clone();
                let params =
                    HashSet::from_iter(candidate.params().iter().map(Param::real_ty).cloned());

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
            println!("Panic! Param type: {}", param.real_ty().to_string());
            panic!("No generator")
        }

        let generator = generator.unwrap().clone();
        let args: Vec<Arg> = generator
            .params()
            .iter()
            .map(|p| {
                // TODO instantiate new object with a 10% probability even if there is a free ones
                // already in the test case

                let usable_variables = self.unconsumed_variables_typed(p.real_ty());

                if !self.instantiated_types().contains(p.real_ty()) || usable_variables.is_empty() {
                    let mut types_to_generate = types_to_generate.clone();
                    types_to_generate.insert(param.real_ty().clone());
                    self.generate_arg_inner(p, types_to_generate)
                } else {
                    // TODO check if those are used

                    let var_i = fastrand::usize(0..usable_variables.len());
                    let (var, pos) = usable_variables.get(var_i).unwrap();
                    Arg::Var(VarArg::new(var.clone(), p.clone()))
                }
            })
            .collect();

        let stmt = generator.to_stmt(args);

        let return_var = self.add_stmt(stmt).unwrap();

        Arg::Var(VarArg::new(return_var, param.clone()))
    }
    pub fn analysis(&self) -> Rc<HirAnalysis> {
        self.analysis.clone()
    }

    /*pub fn to_file(&self, tcx: &TyCtxt<'_>) {
        let mut file = File::create(format!("tests_debug/test_{}.rs", self.id)).unwrap();
        file.write_all(format!("{}", self.to_string(tcx)).as_bytes()).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("tests_debug/test_{}.txt", self.id))
            .unwrap();
        let ddg_dot = Dot::with_config(&self.ddg, &[Config::NodeIndexLabel]);
        let stmts: Vec<String> = self
            .stmts
            .iter()
            .filter_map(|s| {
                let node_index = self.node_index_table.get(&s.id());
                if let Some(node_index) = node_index {
                    Some(format!("({}) {}: {}", node_index.index(), s.id(), s))
                } else {
                    println!(
                        "\nFailed test id: {}\nStatement {}: {}\nNode index table:{:?}",
                        self.id,
                        s.id(),
                        s,
                        self.node_index_table
                    );
                    None
                }
            })
            .collect();
        let var_table: Vec<String> = self
            .var_table
            .iter()
            .map(|(var, value)| format!("{}: {}", var.name, value))
            .collect();

        let node_index_table: Vec<String> = self
            .node_index_table
            .iter()
            .map(|(id, node_index)| format!("{}: {}", id, node_index.index()))
            .collect();

        file.write_all(
            format!(
                "DDG:\n{:?}\n\nStmts:\n{}\n\nVar table:\n{}\n\nNode index table:\n{}",
                ddg_dot,
                stmts.join("\n"),
                var_table.join("\n"),
                node_index_table.join("\n")
            )
            .as_bytes(),
        );
    }*/

    pub fn to_string(&self) -> String {
        let syn_item = self.to_syn();
        let token_stream = syn_item.to_token_stream();
        token_stream.to_string()
    }
}

impl ToSyn for TestCase {
    fn to_syn(&self) -> Item {
        let ident = Ident::new(
            &format!("{}_{}", TEST_FN_PREFIX, self.id),
            Span::call_site(),
        );
        let id = self.id;

        let stmts: Vec<Stmt> = self.stmts.iter().map(|s| s.to_syn()).collect();

        let set_test_id: Stmt = syn::parse_quote! {
            testify_monitor::MONITOR.with(|l| l.borrow_mut().set_test_id(#id));
        };
        /*let wait: Stmt = syn::parse_quote! {
            testify_monitor::MONITOR.with(|l| l.borrow_mut().wait());
        };*/

        syn::parse_quote! {
            #[test]
            fn #ident() {
                #set_test_id
                #(#stmts)*
                //#wait
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
        if let Some(&fitness) = self.coverage.get(objective) {
            fitness
        } else {
            FitnessValue::Max
        }
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

    fn random(analysis: Rc<HirAnalysis>) -> Self {
        let test_id = CHROMOSOME_ID_GENERATOR.lock().as_mut().unwrap().next_id();

        let mut test_case = TestCase::new(test_id, analysis.clone());

        // TODO fill test case with statements until a certain length
        while test_case.size() < 5 {
            test_case.insert_random_stmt();
        }

        //test_case.to_file();

        assert_eq!(test_case.size(), test_case.ddg.node_count());
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

    pub fn mutate(&self) -> Primitive {
        match self {
            Primitive::U8(value) => {
                let new_value;
                if fastrand::f64() < 0.5 {
                    new_value = value.wrapping_add(fastrand::u8(..))
                } else {
                    new_value = value.wrapping_sub(fastrand::u8(..))
                }
                Primitive::U8(new_value)
            }
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
    FieldAccess(FieldAccessStmt),
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
            Statement::FieldAccess(field_access_stmt) => field_access_stmt.to_syn(),
        }
    }

    pub fn returns_value(&self) -> bool {
        match self {
            Statement::Constructor(_) => true,
            Statement::StaticFnInvocation(func) => func.returns_value(),
            Statement::PrimitiveAssignment(_) => true,
            Statement::FunctionInvocation(func) => func.returns_value(),
            Statement::MethodInvocation(m) => m.returns_value(),
            Statement::FieldAccess(_) => true,
            _ => unimplemented!(),
        }
    }

    pub fn var(&self) -> Option<&Var> {
        match self {
            Statement::Constructor(c) => c.var(),
            Statement::MethodInvocation(m) => m.var(),
            Statement::StaticFnInvocation(func) => func.var(),
            Statement::FunctionInvocation(func) => func.var(),
            Statement::PrimitiveAssignment(a) => a.var(),
            Statement::FieldAccess(f_stmt) => f_stmt.var(),
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
            Statement::FieldAccess(f) => Some(f.return_type()),
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
            Statement::FieldAccess(ref mut f) => f.set_var(var),
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
            Statement::FieldAccess(f) => unimplemented!(),
        }
    }

    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        match self {
            Statement::PrimitiveAssignment(_) => panic!("Primitives do not have args"),
            Statement::Constructor(c) => c.set_arg(arg, idx),
            Statement::AttributeAccess(_) => panic!("Attribute access cannot have args"),
            Statement::MethodInvocation(m) => m.set_arg(arg, idx),
            Statement::StaticFnInvocation(f) => f.set_arg(arg, idx),
            Statement::FunctionInvocation(f) => f.set_arg(arg, idx),
            Statement::FieldAccess(_) => unimplemented!(),
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
            Statement::FieldAccess(f) => f.id(),
        }
    }

    pub fn to_string(&self, tcx: &TyCtxt<'_>) -> String {
        match self {
            Statement::PrimitiveAssignment(_) => unimplemented!(),
            Statement::Constructor(c) => {
                let syn_item = c.to_syn();
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
            Statement::AttributeAccess(_) => unimplemented!(),
            Statement::MethodInvocation(m) => {
                let syn_item = m.to_syn();
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
            Statement::StaticFnInvocation(func) => {
                let syn_item = func.to_syn();
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
            Statement::FunctionInvocation(func) => {
                let syn_item = func.to_syn();
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
            Statement::FieldAccess(field) => {
                let syn_item = field.to_syn();
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldAccessStmt {
    id: Uuid,
    field: FieldAccessItem,
    var: Option<Var>,
}

impl FieldAccessStmt {
    pub fn new(field: FieldAccessItem) -> Self {
        FieldAccessStmt {
            id: Uuid::new_v4(),
            field,
            var: None,
        }
    }

    pub fn to_syn(&self) -> Stmt {
        todo!()
    }

    pub fn return_type(&self) -> &T {
        &self.field.ty
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn var(&self) -> Option<&Var> {
        self.var.as_ref()
    }

    pub fn set_var(&mut self, var: Var) {
        self.var = Some(var);
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
            id: self.id.clone(),
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
        let func_ident = Ident::new(self.func.name(), Span::call_site());
        let args: Vec<Expr> = self.args().iter().map(|a| a.to_syn()).collect();
        let parent_path: Vec<Ident> = self
            .func
            .parent()
            .name()
            .split("::")
            .map(|segment| Ident::new(segment, Span::call_site()))
            .collect();

        if self.returns_value() {
            if let Some(var) = &self.var {
                let ident = Ident::new(&var.to_string(), Span::call_site());
                return syn::parse_quote! {
                    let mut #ident = #(#parent_path::)*#func_ident(#(#args),*);
                };
            } else {
                panic!("Name must have been set before")
            }
        } else {
            syn::parse_quote! {
                #(#parent_path::)*#func_ident(#(#args),*);
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
    pub id: Uuid,
    pub var: Option<Var>,
    pub primitive: PrimitiveItem,
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
            id: self.id.clone(),
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
            id: self.id.clone(),
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
        self.constructor.parent()
    }
}

#[derive(Debug)]
pub struct AttrStmt {
    id: Uuid,
}

impl Clone for AttrStmt {
    fn clone(&self) -> Self {
        AttrStmt {
            id: self.id.clone(),
        }
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
            id: self.id.clone(),
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
        let method_ident = Ident::new(self.method.name(), Span::call_site());
        let args: Vec<Expr> = self.args().iter().map(|a| a.to_syn()).collect();
        let parent_path: Vec<Ident> = self
            .method
            .parent()
            .name()
            .split("::")
            .map(|segment| Ident::new(segment, Span::call_site()))
            .collect();

        if self.returns_value() {
            if let Some(var) = &self.var {
                let ident = Ident::new(&var.to_string(), Span::call_site());
                syn::parse_quote! {
                    let mut #ident = #(#parent_path::)*#method_ident(#(#args),*);
                }
            } else {
                panic!("Name must have been set before")
            }
        } else {
            syn::parse_quote! {
                #(#parent_path::)*#method_ident(#(#args),*);
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
            id: self.id.clone(),
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
        self.func.return_type.is_some()
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
        let fn_ident = Ident::new(self.func.name(), Span::call_site());
        let args: Vec<Expr> = self.args.iter().map(Arg::to_syn).collect();

        if self.returns_value() {
            if let Some(var) = &self.var {
                let ident = Ident::new(&var.to_string(), Span::call_site());
                syn::parse_quote! {
                    let mut #ident = #fn_ident(#(#args),*);
                }
            } else {
                panic!("Name must have been set before")
            }
        } else {
            syn::parse_quote! {
                #fn_ident(#(#args),*);
            }
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
pub struct EnumType {
    ty: T,
    variants: Vec<String>,
    syn_item_enum: ItemEnum,
}

impl PartialEq for EnumType {
    fn eq(&self, other: &Self) -> bool {
        self.syn_item_enum == other.syn_item_enum
    }
}

impl Eq for EnumType {}

impl Hash for EnumType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.syn_item_enum.hash(state);
    }
}

impl EnumType {
    pub fn new(syn_item_enum: ItemEnum, mut path: Vec<Ident>) -> Self {
        //path.push(syn_item_enum.ident.clone());
        /*let variants = syn_item_enum
        .variants
        .iter()
        .map(|v| v.ident.to_string())
        .collect();*/
        todo!()
        /*EnumType {
            ty: T::new(path),
            variants,
            syn_item_enum,
        }*/
    }

    pub fn name(&self) -> String {
        self.ty.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct StructType {
    ty: T,
    syn_item_struct: ItemStruct,
}

impl PartialEq for StructType {
    fn eq(&self, other: &Self) -> bool {
        self.syn_item_struct == other.syn_item_struct
    }
}

impl Eq for StructType {}

impl Hash for StructType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.syn_item_struct.hash(state);
    }
}

impl StructType {
    pub fn new(syn_item_struct: ItemStruct, mut path: Vec<Ident>) -> Self {
        todo!()
        /*path.push(syn_item_struct.ident.clone());
        StructType {
            ty: T::new(),
            syn_item_struct,
        }*/
    }

    pub fn name(&self) -> String {
        self.ty.to_string()
    }

    pub fn ident(&self) -> &Ident {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Container {
    Struct(StructType),
    Enum(EnumType),
}

impl Container {
    pub fn ty(&self) -> &T {
        match self {
            Container::Struct(s) => &s.ty,
            Container::Enum(e) => &e.ty,
        }
    }
}
