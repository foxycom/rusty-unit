use crate::analysis::HirAnalysis;
use crate::branch::Branch;
use crate::fitness::FitnessValue;
use crate::generators::{generate_random_prim, TestIdGenerator};
use crate::operators::{BasicMutation, SinglePointCrossover};
use crate::types::{Callable, ComplexT, FieldAccessItem, FunctionItem, Generic, MethodItem, Param, PrimT, PrimitiveItem, StaticFnItem, StructInitItem, Trait, STD_CALLABLES, T, TYPES, IntT};
use petgraph::dot::Dot;
use petgraph::prelude::StableDiGraph;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::{EdgeIndexable, EdgeRef};
use petgraph::{Direction, EdgeDirection};
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use rustc_hir::def_id::DefId;
use rustc_hir::{BodyId, FnSig, HirId, PrimTy};
use rustc_middle::ty::{TyCtxt, TypeFoldable};
use std::collections::{HashMap, HashSet};
use std::env::var;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::RangeInclusive;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use syn::{Expr, FieldValue, ImplItemMethod, Item, ItemEnum, ItemStruct, Stmt, Type};
use uuid::Uuid;

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
    fn mutate(&self, mutation: &BasicMutation) -> Self;

    /// Returns the fitness of the chromosome with respect to a certain branch
    fn fitness(&self, objective: &Branch) -> FitnessValue;

    /// Applies crossover to this and other chromosome and returns a pair of offsprings
    fn crossover(&self, other: &Self, crossover: &SinglePointCrossover) -> (Self, Self)
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
    Defines,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    TypeBinding(Generic, T),
    Var(Rc<Var>),
    Statement(Uuid),
    GenericTy(Rc<Generic>),
    RealTy(Rc<T>)
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::TypeBinding(generic, ty) => {
                write!(f, "TypeBinding({}, {})", generic.to_string(), ty.name())
            }
            Node::Var(var) => write!(f, "Var({})", &var.name),
            Node::Statement(id) => write!(f, "Stmt({})", id),
            Node::GenericTy(generic) => write!(f, "Generic({})", generic.name()),
            Node::RealTy(real_ty) => write!(f, "Real({})", real_ty.full_name())
        }
    }
}

impl Node {
    pub fn expect_stmt(&self) -> Uuid {
        match self {
            Node::Statement(id) => *id,
            _ => panic!("Is no stmt"),
        }
    }

    pub fn expect_var(&self) -> &Rc<Var> {
        match self {
            Node::Var(var) => var,
            _ => panic!("Is no var"),
        }
    }

    pub fn expect_type_binding(&self) -> (&Generic, &T) {
        match self {
            Node::TypeBinding(generic, ty) => (generic, ty),
            _ => panic!("Is not type binding"),
        }
    }

    pub fn expect_generic_ty(&self) -> &Rc<Generic> {
        match self {
            Node::GenericTy(generic) => generic,
            _ => panic!("Is not generic")
        }
    }

    pub fn expect_real_ty(&self) -> &Rc<T> {
        match self {
            Node::RealTy(real_ty) => real_ty,
            _ => panic!("Is not real ty")
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestCase<'test> {
    pub id: u64,
    pub stmts: Vec<Statement>,
    coverage: HashMap<Branch, FitnessValue>,
    pub ddg: StableDiGraph<Node, Dependency>,
    pub nodes: Vec<&'test Node>,
    /// Stores connection of variables and the appropriate statements
    pub var_table: HashMap<Rc<Var>, Uuid>,
    /// Stores nodes to be able to retrieve dd graph nodes later by their index
    pub node_index_table: HashMap<Node, NodeIndex>,
    var_counters: HashMap<String, usize>,
    analysis: Rc<HirAnalysis>,
}

impl<'test> PartialEq for TestCase<'test> {
    fn eq(&self, other: &Self) -> bool {
        /*self.stmts == other.stmts && self.objective == other.objective*/

        // TODO there is more to it
        self.id == other.id
    }
}

impl<'test> Eq for TestCase<'test> {}

impl<'test> Hash for TestCase<'test> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        /*self.objective.hash(state);
        self.stmts.hash(state);*/
    }
}

impl<'test> TestCase<'test> {
    pub fn new(id: u64, analysis: Rc<HirAnalysis>) -> Self {
        TestCase {
            id,
            stmts: Vec::new(),
            coverage: HashMap::new(),
            ddg: StableDiGraph::new(),
            nodes: Vec::new(),
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

    fn create_var(&mut self, stmt: &mut Statement) -> Option<Rc<Var>> {
        if let Some(return_type) = stmt.return_type() {
            let type_name = return_type.var_string();
            let counter = self
                .var_counters
                .entry(type_name.clone())
                .and_modify(|c| *c = *c + 1)
                .or_insert(0);

            let var_name = format!("{}_{}", type_name.to_lowercase(), counter);
            let var = Rc::new(Var::new(&var_name, return_type.clone()));
            Some(var)
        } else {
            None
        }
    }

    pub fn insert_stmt(&mut self, idx: usize, mut stmt: Statement) -> Option<Rc<Var>> {
        let uuid = stmt.id();

        // Save to DDG
        let stmt_node_idx = self.ddg.add_node(Node::Statement(uuid));
        self.node_index_table
            .insert(Node::Statement(uuid), stmt_node_idx);

        let var = self.create_var(&mut stmt);
        if let Some(var) = &var {
            let var_node_idx = self.ddg.add_node(Node::Var(var.clone()));
            self.ddg
                .add_edge(stmt_node_idx, var_node_idx, Dependency::Defines);
            self.node_index_table
                .insert(Node::Var(var.clone()), var_node_idx);
        }

        if let Some(args) = stmt.args() {
            args.iter().for_each(|arg| {
                if let Arg::Var(var_arg) = arg {
                    let arg_node_idx = self
                        .ddg
                        .node_indices()
                        .find(|i| {
                            let node = self.ddg.node_weight(*i).unwrap();
                            if let Node::Var(var) = node {
                                return var == var_arg.var();
                            }
                            false
                        })
                        .unwrap();

                    if var_arg.is_by_reference() {
                        self.ddg
                            .add_edge(stmt_node_idx, arg_node_idx, Dependency::Borrows);
                    } else {
                        self.ddg
                            .add_edge(stmt_node_idx, arg_node_idx, Dependency::Consumes);
                    }
                }
            });
        }

        //self.nodes.push(node);
        self.stmts.insert(idx, stmt);

        if let Some(var) = &var {
            self.var_table.insert(var.clone(), uuid);
        }

        var
    }

    pub fn add_stmt(&mut self, stmt: Statement) -> Option<Rc<Var>> {
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
        /*let stmt = self.stmts.remove(idx);
        let id = stmt.id();
        if stmt.returns_value() {
            self.var_table.remove(stmt.var().unwrap()).unwrap();
        }

        let stmt_node = Node::Statement(id);
        let node_index = match self.node_index_table.remove(&stmt_node) {
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
                let node = self.ddg.node_weight(n).unwrap();
                self.stmt_position(node.expect_stmt()).unwrap()
            })
            .collect();
        neighbour_ids.iter().for_each(|&i| self.remove_stmt_at(i));
        self.ddg.remove_node(node_index).unwrap();

        assert!(self.stmts.len() >= self.var_table.len());
        assert_eq!(self.stmts.len(), self.ddg.node_count());*/
        todo!()
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

    pub fn instantiated_types(&self) -> Vec<Arc<T>> {
        self.var_table
            .iter()
            .map(|(var, _)| var.ty.clone())
            .collect()
    }

    pub fn variables(&self) -> Vec<&Rc<Var>> {
        self.var_table.keys().collect()
    }

    pub fn get_variable(&self, stmt_id: Uuid) -> Option<&Rc<Var>> {
        let stmt = Node::Statement(stmt_id);
        let stmt_node_idx = *self.node_index_table.get(&stmt).unwrap();
        self.ddg
            .edges_directed(stmt_node_idx, Direction::Outgoing)
            .find_map(|e| {
                if *e.weight() == Dependency::Defines {
                    let var_node = self.ddg.node_weight(e.target()).unwrap();
                    Some(var_node.expect_var())
                } else {
                    None
                }
            })
    }

    pub fn defined_by(&self, var: &Rc<Var>) -> Uuid {
        let var_node = Node::Var(var.clone());
        let var_node_idx = self.node_index_table.get(&var_node).unwrap();

        let mut edges = self.ddg.edges_directed(*var_node_idx, Direction::Incoming);
        edges
            .find_map(|e| {
                if *e.weight() == Dependency::Defines {
                    let stmt_node = self.ddg.node_weight(e.source()).unwrap();
                    Some(stmt_node.expect_stmt())
                } else {
                    None
                }
            })
            .unwrap()
    }

    pub fn is_consumable(&self, var: &Rc<Var>, idx: usize) -> bool {
        let var_node = Node::Var(var.clone());
        let var_node_idx = self.node_index_table.get(&var_node).unwrap();
        let edges = self.ddg.edges_directed(*var_node_idx, Direction::Incoming);
        let stmts = edges
            .filter_map(|e| {
                if *e.weight() == Dependency::Consumes || *e.weight() == Dependency::Borrows {
                    let stmt_node = self.ddg.node_weight(e.source()).unwrap();
                    Some((stmt_node, e.weight()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if stmts.is_empty() {
            return true;
        }

        let mut last_borrow_pos = 0;
        for (stmt, edge) in stmts {
            if *edge == Dependency::Consumes {
                return false;
            }

            let uuid = stmt.expect_stmt();
            let stmt_pos = self.index_of_stmt(uuid);
            last_borrow_pos = last_borrow_pos.max(stmt_pos);
        }

        last_borrow_pos < idx
    }

    pub fn is_borrowable(&self, var: &Rc<Var>, idx: usize) -> bool {
        let var_node = Node::Var(var.clone());
        let var_node_idx = self.node_index_table.get(&var_node).unwrap();
        let mut edges = self.ddg.edges_directed(*var_node_idx, Direction::Incoming);

        let collision = edges
            .find_map(|e| {
                if *e.weight() == Dependency::Consumes {
                    Some(self.ddg.node_weight(e.source()).unwrap())
                } else {
                    None
                }
            })
            .map(|stmt| {
                let uuid = stmt.expect_stmt();
                let consuming_pos = self.stmt_position(uuid).unwrap();
                consuming_pos < idx
            });

        if let Some(collision) = collision {
            collision
        } else {
            true
        }
    }

    pub fn index_of_stmt(&self, stmt_id: Uuid) -> usize {
        self.stmts
            .iter()
            .zip(0..self.stmts.len())
            .find_map(|(stmt, idx)| {
                if stmt.id() == stmt_id {
                    Some(idx)
                } else {
                    None
                }
            })
            .unwrap()
    }

    pub fn unconsumed_variables_typed(&self, ty: &Arc<T>) -> Vec<(Rc<Var>, usize)> {
        let mut vars = self.variables_typed(ty);
        vars.retain(|(v, _)| !self.is_consumed(v));
        vars
    }

    pub fn variables_typed(&self, ty: &Arc<T>) -> Vec<(Rc<Var>, usize)> {
        // Also return their positions in the test case
        let node_indices = self.ddg.node_indices();
        node_indices
            .filter_map(|var_idx| {
                let var_node = self.ddg.node_weight(var_idx).unwrap();
                if let Node::Var(var) = var_node {
                    return Some((var_idx, var));
                }
                None
            })
            .filter(|(var_idx, var)| &var.ty == ty)
            .map(|(var_idx, var)| {
                let edge = self
                    .ddg
                    .edges_directed(var_idx, Direction::Incoming)
                    .find(|e| *e.weight() == Dependency::Defines)
                    .unwrap();
                let stmt_node_idx = edge.source();
                let stmt_node = self.ddg.node_weight(stmt_node_idx).unwrap();
                let stmt_idx = self.index_of_stmt(stmt_node.expect_stmt());
                (var.clone(), stmt_idx)
            })
            .collect::<Vec<_>>()
    }

    pub fn is_consumed(&self, var: &Rc<Var>) -> bool {
        let var_node = Node::Var(var.clone());
        let var_node_index = self.node_index_table.get(&var_node).unwrap();
        let mut edges = self
            .ddg
            .edges_directed(*var_node_index, Direction::Incoming);
        edges.any(|e| *e.weight() == Dependency::Consumes)
    }

    pub fn instantiated_at(&self, var: &Rc<Var>) -> Option<usize> {
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

    pub fn consumed_at(&self, var: &Rc<Var>) -> Option<usize> {
        let var_node = Node::Var(var.clone());
        let var_node_idx = self.node_index_table.get(&var_node).unwrap();
        let consume_pos = self.ddg
            .edges_directed(*var_node_idx, Direction::Incoming)
            .find_map(|e| {
                if *e.weight() == Dependency::Consumes {
                    let stmt = self.ddg.node_weight(e.source()).unwrap();
                    Some(self.stmt_position(stmt.expect_stmt()).unwrap())
                } else {
                    None
                }
            });

        consume_pos
    }

    pub fn borrowed_at(&self, var: &Rc<Var>) -> Vec<usize> {
        let var_node = Node::Var(var.clone());
        let var_node_idx = self.node_index_table.get(&var_node).unwrap();
        self.ddg
            .edges_directed(*var_node_idx, Direction::Incoming)
            .filter_map(|e| {
                if *e.weight() == Dependency::Borrows {
                    let stmt = self.ddg.node_weight(e.source()).unwrap();
                    Some(self.stmt_position(stmt.expect_stmt()).unwrap())
                } else {
                    None
                }
            }).collect::<Vec<_>>()
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
    pub fn available_callables(&self) -> Vec<(&Rc<Var>, &Callable, RangeInclusive<usize>)> {
        self.var_table
            .keys()
            .filter_map(|v| {
                let mut callables = self.analysis.callables_of(v.ty());

                if let Some(idx) = self.consumed_at(v) {
                    // v can only be borrowed
                    let range = self.instantiated_at(v).unwrap()..=idx;
                    // Retain only callables that are borrowing
                    let possible_callables: Vec<(&Rc<Var>, &Callable, RangeInclusive<usize>)> =
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
    pub fn insert_random_stmt(&mut self) -> bool {
        let callables = self.analysis.callables();
        let i = fastrand::usize(0..callables.len());
        let callable = (*(callables.get(i).unwrap())).clone();

        let args: Vec<Arg> = callable
            .params()
            .iter()
            .filter_map(|p| self.generate_arg(p))
            .collect();

        let bounded_generics = self.bind_generics(&callable, &args);

        if args.len() != callable.params().len() {
            println!("Could not generate args for callable: \n{:?}", callable);
            return false;
        }

        let stmt = callable.to_stmt(args, bounded_generics);
        self.add_stmt(stmt);
        true
    }

    pub fn generate_arg(&mut self, param: &Param) -> Option<Arg> {
        self.generate_arg_inner(param, HashSet::new())
    }

    fn get_primitive_type_for_generic(&self, generic_ty: &Generic) -> Option<PrimT> {
        // Generated arg should comply to generic trait bounds
        let bounds = generic_ty.bounds();

        if bounds.is_empty() {
            return Some(PrimT::Int(IntT::Isize));
        }

        let mut possible_primitives: Option<HashSet<PrimT>> = None;
        for bound in bounds {
            let implementors = PrimT::implementors_of(bound);
            if let Some(primitives) = &possible_primitives {
                let intersection = primitives
                    .intersection(&implementors)
                    .cloned()
                    .collect::<HashSet<_>>();
                possible_primitives = Some(intersection);
            } else {
                possible_primitives = Some(implementors);
            }
        }

        // Try to generate simple primitives first
        if let Some(possible_primitives) = possible_primitives {
            if !possible_primitives.is_empty() {
                let primitives = Vec::from_iter(possible_primitives.iter());
                let primitive_i = fastrand::usize(0..primitives.len());
                let primitive = *primitives.get(primitive_i).unwrap();

                return Some(*primitive);
            }
        }
        None
    }

    fn get_complex_type_for_generic(&self, generic_ty: &Generic) -> Option<Arc<T>> {
        // Generated arg should comply to generic trait bounds
        let bounds = generic_ty.bounds();

        let possible_complex_types = TYPES
            .iter()
            .filter_map(|(ty, impl_trait)| {
                if bounds.iter().all(|b| impl_trait.contains(b)) {
                    return Some(ty);
                }
                None
            })
            .collect::<Vec<_>>();

        if possible_complex_types.is_empty() {
            return None;
        }

        let complex_i = fastrand::usize(0..possible_complex_types.len());
        let mut complex_ty = (*possible_complex_types.get(complex_i).unwrap()).clone();

        // Recursively generate generics for generics
        if let T::Complex(complex_ty) = complex_ty.as_ref() {
            println!("Selected complex ty: {:?}", complex_ty);
            let bounded_generics = complex_ty.generics().iter().map(|g| {
                let generic = g.expect_generic();
                if let Some(prim) = self.get_primitive_type_for_generic(generic) {
                    println!("Selected prim for generic: {:?}", prim);
                    //T::Prim(prim)
                    todo!()
                } else {
                    println!("Selected complex for generic");
                    // We must assume that a type is available
                    self.get_complex_type_for_generic(generic).unwrap()
                }
            }).collect::<Vec<_>>();
            //complex_ty.bind_generics(bounded_generics);
        }

        Some(complex_ty.clone())
    }

    fn generate_generic_arg(
        &mut self,
        param: &Param,
        types_to_generate: HashSet<Arc<T>>,
    ) -> Option<Arg> {
        let generic_ty = param.real_ty().expect_generic();
        let primitive_ty = self.get_primitive_type_for_generic(generic_ty);
        if let Some(primitive) = primitive_ty {
            let arg = Arg::Primitive(generate_random_prim(&primitive, param));
            return Some(arg);
        }

        let complex_ty = self.get_complex_type_for_generic(generic_ty)?;

        let generators = STD_CALLABLES
            .iter()
            .filter(|callable| {
                if let Some(return_ty) = callable.return_type() {
                    return_ty == &complex_ty
                } else {
                    false
                }
            })
            .cloned()
            .collect::<Vec<_>>();

        self.generate_arg_from_generators(param, generators, types_to_generate)
    }

    fn generate_arg_inner(&mut self, param: &Param, types_to_generate: HashSet<Arc<T>>) -> Option<Arg> {
        if param.is_primitive() {
            println!("Generating primitive for param: {:?}", param);
            let ty = param.real_ty().expect_primitive();
            return Some(Arg::Primitive(generate_random_prim(ty, param)));
        } else if param.is_generic() {
            println!("Generating generic for param: {:?}", param);
            return self.generate_generic_arg(param, types_to_generate);
        } else {
            println!("Generating complex for param: {:?}", param);
            let generators = self
                .analysis
                .generators(param.real_ty())
                .iter()
                .map(|&c| c.clone())
                .collect();
            self.generate_arg_from_generators(param, generators, types_to_generate)
        }
    }

    fn bind_generics(&self, callable: &Callable, args: &Vec<Arg>) -> Vec<Arc<T>> {
        // Now look which generic parameters are already bounded by arguments and bound the rest
        /*let return_ty = callable.return_type();
        let bounded_generics = if let Some(return_ty) = return_ty {
            if let Some(generics) = return_ty.generics() {
                let mut all_generics = generics
                    .iter()
                    .map(|g| match g.as_ref() {
                        T::Generic(generic) => (generic, None),
                        T::Ref(ty) => (ty.expect_generic(), None),
                        _ => todo!("T is {:?}", g),
                    })
                    .collect::<HashMap<_, _>>();

                args.iter().filter(|a| a.is_generic()).for_each(|a| {
                    // This can on ly be a complex object at the moment
                    let var_arg = a.expect_var();
                    let generic = var_arg.param().real_ty().expect_generic();

                    // Check if the generic type is global or just defined in the func
                    if all_generics.contains_key(generic) {
                        let var_node = Node::Var(var_arg.var.clone());

                        all_generics.insert(generic, Some(var_arg.param().real_ty().clone()));
                    }
                });

                assert_eq!(all_generics.len(), generics.len());

                // Set still unbounded generics
                all_generics
                    .iter_mut()
                    .filter(|(generic, t)| t.is_none())
                    .for_each(|(generic, t)| {
                        // TODO generate some type
                        let primitive = self.get_primitive_type_for_generic(generic);
                        if let Some(primitive) = primitive {
                            *t = Some(T::Prim(primitive));
                        } else {
                            let complex = self.get_complex_type_for_generic(generic);
                            if let Some(complex) = complex {
                                *t = Some(complex);
                            }
                        }
                    });

                generics
                    .iter()
                    .map(|g| match g.as_ref() {
                        T::Generic(generic) => {
                            all_generics.get(generic).unwrap().as_ref().unwrap().clone()
                        }
                        T::Ref(ref_generic) => {
                            let ty = all_generics
                                .get(ref_generic.as_ref().expect_generic())
                                .unwrap()
                                .as_ref()
                                .unwrap();
                            T::Ref(Arc::new(ty.clone()))
                        }
                        _ => todo!(),
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        bounded_generics*/
        todo!()
    }

    fn generate_arg_from_generators(
        &mut self,
        param: &Param,
        mut generators: Vec<Callable>,
        types_to_generate: HashSet<Arc<T>>,
    ) -> Option<Arg> {
        println!(
            "Trying to generate {:?}, generators: {:?}",
            param.real_ty(),
            generators
        );

        let mut generator = None;
        let mut retry = true;
        while retry && !generators.is_empty() {
            retry = false;

            // Pick a random generator
            let i = fastrand::usize(0..generators.len());
            let candidate = generators.get(i).unwrap().clone();
            let params = HashSet::from_iter(candidate.params().iter().map(Param::real_ty).cloned());

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

        let generator = generator?.clone();
        let args: Vec<Arg> = generator
            .params()
            .iter()
            .filter_map(|p| {
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
                    Some(Arg::Var(VarArg::new(var.clone(), p.clone())))
                }
            })
            .collect();

        if args.len() != generator.params().len() {
            println!("Could not generate param: {:?}", param);
            return None;
        }

        let bounded_generics = self.bind_generics(&generator, &args);

        let stmt = generator.to_stmt(args, bounded_generics);

        let return_var = self.add_stmt(stmt).unwrap();

        Some(Arg::Var(VarArg::new(return_var, param.clone())))
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

impl<'test> ToSyn for TestCase<'test> {
    fn to_syn(&self) -> Item {
        let test_name = Ident::new(
            &format!("{}_{}", TEST_FN_PREFIX, &self.id),
            Span::call_site(),
        );
        let stmts: Vec<Stmt> = self.stmts.iter().map(|s| s.to_syn(&self)).collect();

        let test_id = self.id;
        let set_test_id_stmt: Stmt = syn::parse_quote! {
            testify_monitor::set_test_id(#test_id);
        };

        syn::parse_quote! {
            #[test]
            fn #test_name() {
                #set_test_id_stmt
                #(#stmts)*
            }
        }
    }
}

impl<'test> Chromosome for TestCase<'test> {
    fn id(&self) -> u64 {
        self.id
    }

    fn coverage(&self) -> &HashMap<Branch, FitnessValue> {
        &self.coverage
    }

    fn set_coverage(&mut self, coverage: HashMap<Branch, FitnessValue>) {
        self.coverage = coverage;
    }

    fn mutate(&self, mutation: &BasicMutation) -> Self {
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

    fn crossover(&self, other: &Self, crossover: &SinglePointCrossover) -> (Self, Self)
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

        println!(
            "Generated test:\n{}\n{}",
            test_case.to_string(),
            Dot::new(&test_case.ddg)
        );

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

    pub fn param_name(&self) -> Option<&String> {
        match self {
            Arg::Var(var) => var.param.name(),
            Arg::Primitive(prim) => prim.name(),
        }
    }

    pub fn is_generic(&self) -> bool {
        match self {
            Arg::Var(var_arg) => var_arg.param.is_generic(),
            Arg::Primitive(_) => false,
        }
    }

    pub fn is_var(&self) -> bool {
        match self {
            Arg::Var(_) => true,
            _ => false,
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            Arg::Primitive(_) => true,
            _ => false,
        }
    }

    pub fn expect_var(&self) -> &VarArg {
        match self {
            Arg::Var(var_arg) => var_arg,
            _ => panic!("Is no var"),
        }
    }

    pub fn expect_primitive(&self) -> &Primitive {
        match self {
            Arg::Primitive(primitive) => primitive,
            _ => panic!("Is no primitive"),
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
    UInt(Param, UInt),
    Int(Param, Int),
    Bool(Param, bool),
    Str(Param, String),
    Float(Param, Float),
    Char(Param, char),
}

impl Primitive {
    pub fn ty(&self) -> Box<Type> {
        match self {
            Primitive::UInt(_, u) => u.ty(),
            Primitive::Int(_, i) => i.ty(),
            Primitive::Bool(_, _) => Box::new(syn::parse_quote!(bool)),
            Primitive::Str(_, _) => Box::new(syn::parse_quote!(&str)),
            Primitive::Float(_, f) => f.ty(),
            Primitive::Char(_, _) => Box::new(syn::parse_quote!(char)),
        }
    }

    pub fn to_syn(&self) -> Expr {
        match self {
            Primitive::UInt(_, u) => u.to_syn(),
            Primitive::Int(_, i) => i.to_syn(),
            Primitive::Float(_, f) => f.to_syn(),
            Primitive::Bool(_, b) => syn::parse_quote! {#b},
            Primitive::Str(_, s) => syn::parse_quote! {#s},
            Primitive::Char(_, c) => syn::parse_quote! {#c},
        }
    }

    pub fn mutate(&self) -> Primitive {
        /*match self {
            Primitive::U8(param, value) => {
                let new_value;
                if fastrand::f64() < 0.5 {
                    new_value = value.wrapping_add(fastrand::u8(..))
                } else {
                    new_value = value.wrapping_sub(fastrand::u8(..))
                }
                Primitive::U8(param.clone(), new_value)
            }
        }*/
        todo!()
    }

    pub fn name(&self) -> Option<&String> {
        match self {
            Primitive::UInt(param, _) => param.name(),
            Primitive::Int(param, _) => param.name(),
            Primitive::Bool(param, _) => param.name(),
            Primitive::Str(param, _) => param.name(),
            Primitive::Float(param, _) => param.name(),
            Primitive::Char(param, _) => param.name(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UInt {
    Usize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

impl UInt {
    pub fn to_syn(&self) -> Expr {
        match self {
            UInt::Usize(val) => syn::parse_quote!(#val),
            UInt::U8(val) => syn::parse_quote!(#val),
            UInt::U16(val) => syn::parse_quote!(#val),
            UInt::U32(val) => syn::parse_quote!(#val),
            UInt::U64(val) => syn::parse_quote!(#val),
            UInt::U128(val) => syn::parse_quote!(#val),
        }
    }

    pub fn ty(&self) -> Box<Type> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Int {
    Isize(isize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

impl Int {
    pub fn to_syn(&self) -> Expr {
        match self {
            Int::Isize(val) => syn::parse_quote!(#val),
            Int::I8(val) => syn::parse_quote!(#val),
            Int::I16(val) => syn::parse_quote!(#val),
            Int::I32(val) => syn::parse_quote!(#val),
            Int::I64(val) => syn::parse_quote!(#val),
            Int::I128(val) => syn::parse_quote!(#val),
        }
    }

    pub fn ty(&self) -> Box<Type> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Float {
    F32(f32),
    F64(f64),
}

impl Float {
    pub fn to_syn(&self) -> Expr {
        match self {
            Float::F32(val) => syn::parse_quote!(#val),
            Float::F64(val) => syn::parse_quote!(#val),
        }
    }

    pub fn ty(&self) -> Box<Type> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct VarArg {
    param: Param,
    var: Rc<Var>,
}

impl VarArg {
    pub fn new(var: Rc<Var>, param: Param) -> Self {
        VarArg { param, var }
    }

    pub fn param(&self) -> &Param {
        &self.param
    }

    pub fn var(&self) -> &Rc<Var> {
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
    AttributeAccess(AttrStmt),
    MethodInvocation(MethodInvStmt),
    StaticFnInvocation(StaticFnInvStmt),
    FunctionInvocation(FnInvStmt),
    FieldAccess(FieldAccessStmt),
    StructInit(StructInitStmt),
}

impl Statement {
    pub fn to_syn(&self, test_case: &TestCase<'_>) -> Stmt {
        match self {
            Statement::PrimitiveAssignment(primitive_stmt) => primitive_stmt.to_syn(test_case),
            Statement::AttributeAccess(_) => {
                unimplemented!()
            }
            Statement::StaticFnInvocation(fn_inv_stmt) => fn_inv_stmt.to_syn(test_case),
            Statement::MethodInvocation(method_inv_stmt) => method_inv_stmt.to_syn(test_case),
            Statement::FunctionInvocation(fn_inv_stmt) => fn_inv_stmt.to_syn(test_case),
            Statement::FieldAccess(field_access_stmt) => field_access_stmt.to_syn(test_case),
            Statement::StructInit(struct_init_stmt) => struct_init_stmt.to_syn(test_case),
        }
    }

    pub fn returns_value(&self) -> bool {
        match self {
            Statement::StaticFnInvocation(func) => func.returns_value(),
            Statement::PrimitiveAssignment(_) => true,
            Statement::FunctionInvocation(func) => func.returns_value(),
            Statement::MethodInvocation(m) => m.returns_value(),
            Statement::FieldAccess(_) => true,

            Statement::AttributeAccess(_) => unimplemented!(),
            Statement::StructInit(s) => s.returns_value(),
        }
    }

    pub fn return_type(&self) -> Option<&Arc<T>> {
        match self {
            Statement::PrimitiveAssignment(a) => Some(a.return_type()),
            Statement::MethodInvocation(m) => m.return_type(),
            Statement::StaticFnInvocation(f) => f.return_type(),
            Statement::FunctionInvocation(f) => f.return_type(),
            Statement::AttributeAccess(a) => a.return_type(),
            Statement::FieldAccess(f) => Some(f.return_type()),
            Statement::StructInit(s) => Some(s.return_type()),
        }
    }

    pub fn args(&self) -> Option<&Vec<Arg>> {
        match self {
            Statement::PrimitiveAssignment(_) => None,
            Statement::AttributeAccess(_) => None,
            Statement::MethodInvocation(m) => Some(m.args()),
            Statement::StaticFnInvocation(f) => Some(f.args()),
            Statement::FunctionInvocation(f) => Some(f.args()),
            Statement::FieldAccess(f) => unimplemented!(),
            Statement::StructInit(s) => Some(s.args()),
        }
    }

    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        match self {
            Statement::PrimitiveAssignment(_) => panic!("Primitives do not have args"),
            Statement::AttributeAccess(_) => panic!("Attribute access cannot have args"),
            Statement::MethodInvocation(m) => m.set_arg(arg, idx),
            Statement::StaticFnInvocation(f) => f.set_arg(arg, idx),
            Statement::FunctionInvocation(f) => f.set_arg(arg, idx),
            Statement::FieldAccess(_) => unimplemented!(),
            Statement::StructInit(s) => s.set_arg(arg, idx),
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Statement::PrimitiveAssignment(p) => p.id(),
            Statement::AttributeAccess(a) => a.id(),
            Statement::MethodInvocation(m) => m.id(),
            Statement::StaticFnInvocation(f) => f.id(),
            Statement::FunctionInvocation(f) => f.id(),
            Statement::FieldAccess(f) => f.id(),
            Statement::StructInit(s) => s.id(),
        }
    }

    pub fn to_string(&self, test_case: &TestCase<'_>) -> String {
        match self {
            Statement::PrimitiveAssignment(_) => unimplemented!(),
            Statement::AttributeAccess(_) => unimplemented!(),
            Statement::MethodInvocation(m) => {
                let syn_item = m.to_syn(test_case);
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
            Statement::StaticFnInvocation(func) => {
                let syn_item = func.to_syn(test_case);
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
            Statement::FunctionInvocation(func) => {
                let syn_item = func.to_syn(test_case);
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
            Statement::FieldAccess(field) => {
                let syn_item = field.to_syn(test_case);
                let token_stream = syn_item.to_token_stream();
                token_stream.to_string()
            }
            Statement::StructInit(s) => {
                let syn_item = s.to_syn(test_case);
                let token_streanm = syn_item.to_token_stream();
                token_streanm.to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldAccessStmt {
    id: Uuid,
    field: FieldAccessItem,
}

impl FieldAccessStmt {
    pub fn new(field: FieldAccessItem, bounded_generics: Vec<Arc<T>>) -> Self {
        FieldAccessStmt {
            id: Uuid::new_v4(),
            field,
        }
    }

    pub fn to_syn(&self, test_case: &TestCase<'_>) -> Stmt {
        todo!()
    }

    pub fn return_type(&self) -> &Arc<T> {
        &self.field.ty
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct StructInitStmt {
    id: Uuid,
    args: Vec<Arg>,
    struct_init_item: StructInitItem,
    bounded_generics: Vec<Arc<T>>,
}

impl StructInitStmt {
    pub fn new(struct_init_item: StructInitItem, args: Vec<Arg>, bounded_generics: Vec<Arc<T>>) -> Self {
        StructInitStmt {
            id: Uuid::new_v4(),
            struct_init_item,
            args,
            bounded_generics,
        }
    }

    pub fn return_type(&self) -> &Arc<T> {
        self.struct_init_item.return_type()
    }

    pub fn returns_value(&self) -> bool {
        true
    }

    pub fn to_syn(&self, test_case: &TestCase<'_>) -> Stmt {
        let var = test_case.get_variable(self.id);
        if let Some(var) = var {
            let ident = Ident::new(&var.to_string(), Span::call_site());

            let type_name = self.struct_init_item.return_type.to_ident();
            let args: Vec<FieldValue> = self
                .args
                .iter()
                .map(|a| {
                    let field_name = a.param_name().unwrap();
                    let field_ident = Ident::new(field_name, Span::call_site());
                    let val = a.to_syn();

                    syn::parse_quote! {
                        #field_ident : #val
                    }
                })
                .collect();

            syn::parse_quote! {
                let mut #ident = #type_name { #(#args),* };
            }
        } else {
            panic!("Variable has not been set in the test case for a stmt")
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn args(&self) -> &Vec<Arg> {
        &self.args
    }

    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        self.args[idx] = arg;
    }

    pub fn struct_init_item(&self) -> &StructInitItem {
        &self.struct_init_item
    }
}

#[derive(Debug, Clone)]
pub struct StaticFnInvStmt {
    id: Uuid,
    args: Vec<Arg>,
    func: StaticFnItem,
    bounded_generics: Vec<Arc<T>>,
}

impl StaticFnInvStmt {
    pub fn new(func: StaticFnItem, args: Vec<Arg>, bounded_generics: Vec<Arc<T>>) -> Self {
        StaticFnInvStmt {
            id: Uuid::new_v4(),
            args,
            func,
            bounded_generics,
        }
    }

    pub fn return_type(&self) -> Option<&Arc<T>> {
        self.func.return_type.as_ref()
    }

    pub fn returns_value(&self) -> bool {
        self.func.return_type.is_some()
    }

    pub fn to_syn(&self, test_case: &TestCase<'_>) -> Stmt {
        let func_ident = Ident::new(self.func.name(), Span::call_site());
        let args: Vec<Expr> = self.args().iter().map(|a| a.to_syn()).collect();
        let parent_path = self.func.parent().to_ident();

        if self.returns_value() {
            let var = test_case.get_variable(self.id);

            if let Some(var) = var {
                let var_name = Ident::new(&var.to_string(), Span::call_site());
                let return_type_name = self.func.return_type.as_ref().unwrap().to_ident();
                return if self.bounded_generics.is_empty() {
                    syn::parse_quote! {
                        let mut #var_name: #return_type_name = #parent_path::#func_ident(#(#args),*);
                    }
                } else {
                    let bounded_generics_idents = self
                        .bounded_generics
                        .iter()
                        .map(|g| g.to_ident())
                        .collect::<Vec<_>>();
                    syn::parse_quote! {
                        let mut #var_name: #return_type_name<#(#bounded_generics_idents),*> = #parent_path::#func_ident(#(#args),*);
                    }
                };
            } else {
                panic!("Variable has not been set for a stmt")
            }
        } else {
            syn::parse_quote! {
                #parent_path::#func_ident(#(#args),*);
            }
        }
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

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub id: Uuid,
    pub primitive: PrimitiveItem,
}

impl AssignStmt {
    pub fn new(primitive: PrimitiveItem, bounded_generics: Vec<Arc<T>>) -> Self {
        AssignStmt {
            id: Uuid::new_v4(),
            primitive,
        }
    }

    pub fn return_type(&self) -> &Arc<T> {
        &self.primitive.ty
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn to_syn(&self, test_case: &TestCase<'_>) -> Stmt {
        unimplemented!()
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

    pub fn return_type(&self) -> Option<&Arc<T>> {
        unimplemented!()
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct MethodInvStmt {
    id: Uuid,
    method: MethodItem,
    args: Vec<Arg>,
    bounded_generics: Vec<Arc<T>>,
}

impl MethodInvStmt {
    pub fn new(method: MethodItem, args: Vec<Arg>, bounded_generics: Vec<Arc<T>>) -> Self {
        MethodInvStmt {
            method,
            args,
            id: Uuid::new_v4(),
            bounded_generics,
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

    pub fn to_syn(&self, test_case: &TestCase<'_>) -> Stmt {
        /*return if self.bounded_generics.is_empty() {
            syn::parse_quote! {
                        let mut #var_name: #return_type_name = #parent_path::#func_ident(#(#args),*);
                    }
        } else {
            let bounded_generics_idents = self.bounded_generics.iter().map(|g| g.to_ident()).collect::<Vec<_>>();
            println!("Generic idents: {:?}", bounded_generics_idents);
            syn::parse_quote! {
                        let mut #var_name: #return_type_name<#(#bounded_generics_idents),*> = #parent_path::#func_ident(#(#args),*);
                    }
        }*/

        let method_ident = Ident::new(self.method.name(), Span::call_site());
        let args: Vec<Expr> = self.args().iter().map(|a| a.to_syn()).collect();
        let parent_path = self.method.parent().to_ident();

        if self.returns_value() {
            let var = test_case.get_variable(self.id);
            if let Some(var) = var {
                let var_name = Ident::new(&var.to_string(), Span::call_site());
                let return_type_name = self.return_type().unwrap().to_ident();
                return if self.bounded_generics.is_empty() {
                    syn::parse_quote! {
                        let mut #var_name: #return_type_name = #parent_path::#method_ident(#(#args),*);
                    }
                } else {
                    let bounded_generics_idents = self
                        .bounded_generics
                        .iter()
                        .map(|g| g.to_ident())
                        .collect::<Vec<_>>();
                    syn::parse_quote! {
                        let mut #var_name: #return_type_name<#(#bounded_generics_idents),*> = #parent_path::#method_ident(#(#args),*);
                    }
                };
            } else {
                panic!("Variable has not been set in test case for a stmt")
            }
        } else {
            syn::parse_quote! {
                #parent_path::#method_ident(#(#args),*);
            }
        }
    }

    pub fn returns_value(&self) -> bool {
        self.method.return_type.is_some()
    }

    pub fn return_type(&self) -> Option<&Arc<T>> {
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
}

#[derive(Debug, Clone)]
pub struct FnInvStmt {
    id: Uuid,
    args: Vec<Arg>,
    func: FunctionItem,
    bounded_generics: Vec<Arc<T>>,
}

impl FnInvStmt {
    pub fn new(func: FunctionItem, args: Vec<Arg>, bounded_generics: Vec<Arc<T>>) -> Self {
        FnInvStmt {
            args,
            func,
            id: Uuid::new_v4(),
            bounded_generics,
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

    pub fn return_type(&self) -> Option<&Arc<T>> {
        self.func.return_type.as_ref()
    }

    pub fn set_arg(&mut self, arg: Arg, idx: usize) {
        self.args[idx] = arg;
    }

    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn to_syn(&self, test_case: &TestCase<'_>) -> Stmt {
        let fn_ident = Ident::new(self.func.name(), Span::call_site());
        let args: Vec<Expr> = self.args.iter().map(Arg::to_syn).collect();

        if self.returns_value() {
            let var = test_case.get_variable(self.id);

            if let Some(var) = var {
                let var_name = Ident::new(&var.to_string(), Span::call_site());
                let return_type_name = self.return_type().unwrap().to_ident();
                syn::parse_quote! {
                    let mut #var_name: #return_type_name = #fn_ident(#(#args),*);
                }
            } else {
                panic!("Variable has not been set for a returning stmt")
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
    ty: Arc<T>,
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Var {
    pub fn new(name: &str, ty: Arc<T>) -> Self {
        Var {
            name: name.to_owned(),
            ty,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &Arc<T> {
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
