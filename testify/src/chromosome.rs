use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use petgraph::prelude::{NodeIndex, StableDiGraph};
use petgraph::Direction;
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{
    Expr, FnArg, ImplItemMethod, Item, ItemFn, Pat, PatType, Path, ReturnType, Stmt, Type, TypePath,
};
use uuid::Uuid;

use crate::generators::{PrimitivesGenerator, TestIdGenerator};
use crate::operators::{BasicCrossover, BasicMutation};
use crate::source::{Branch, BranchManager, SourceFile};
use crate::util;
use crate::util::{fn_arg_to_param, is_constructor, is_method, return_type_name, type_name};

pub trait Chromosome: Clone + Debug {
    fn mutate(&self) -> Self;

    fn fitness(&self, objective: &Branch) -> f64;

    fn crossover(&self, other: &Self) -> (Self, Self)
    where
        Self: Sized;
}

pub trait ChromosomeGenerator {
    type C: Chromosome;

    fn generate(&mut self) -> Self::C;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dependency {
    Owns,
    Consumes,
    Borrows,
}

#[derive(Debug)]
pub struct TestCase {
    id: u64,
    stmts: Vec<Statement>,
    results: HashMap<u64, f64>,
    mutation: BasicMutation,
    crossover: BasicCrossover,
    ddg: StableDiGraph<String, Dependency>,
    branch_manager: Rc<RefCell<BranchManager>>,

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
            results: self.results.clone(),
            mutation: self.mutation.clone(),
            crossover: self.crossover.clone(),
            branch_manager: self.branch_manager.clone(),
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
    pub const TEST_FN_PREFIX: &'static str = "testify";

    pub fn new(
        id: u64,
        mutation: BasicMutation,
        crossover: BasicCrossover,
        branch_manager: Rc<RefCell<BranchManager>>,
    ) -> Self {
        TestCase {
            id,
            stmts: Vec::new(),
            results: HashMap::new(),
            mutation,
            crossover,
            branch_manager,
            ddg: StableDiGraph::new(),
            var_table: HashMap::new(),
            node_index_table: HashMap::new(),
            var_counters: HashMap::new(),
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
            let type_name = return_type.to_string();
            let counter = self
                .var_counters
                .entry(type_name.clone())
                .and_modify(|c| *c = *c + 1)
                .or_insert(0);

            let var_name = format!("{}_{}", type_name, counter);
            let var = Var::new(&var_name, return_type.clone());
            stmt.set_var(var.clone());
            Some(var)
        } else {
            None
        }
    }

    pub fn add_stmt(&mut self, mut stmt: Statement) -> Option<Var> {
        let var = self.set_var(&mut stmt);

        let node_index = self.ddg.add_node(stmt.id().to_string());
        self.node_index_table.insert(stmt.id(), node_index.clone());

        let mut insert_position: usize = 0;
        if let Some(args) = stmt.args() {
            insert_position = args.iter()
                .filter_map(|a|
                    // Return the indeces of statements that produce variables used
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

                        let generating_stmt_position = self.stmts.iter().position(|s| {
                            s.id() == generating_statement_uuid
                        }).unwrap();

                        Some(generating_statement_position)
                    } else {
                        None
                    }
                ).fold(0usize, |a, b| {
                a.max(b)
            });
        }

        self.stmts.insert(insert_position + 1, stmt);

        if let Some(var) = &var {
            self.var_table.insert(var.clone(), stmt.id());
        }

        var
    }

    pub fn is_consumed(&self, stmt: &Statement) -> bool {
        if !stmt.returns_value() {
            panic!("The statement does not return anything");
        }

        if let Some(var) = stmt.var() {
            let uuid = self.var_table.get(var).unwrap();
            let node_index = self.node_index_table.get(uuid).unwrap();
            let mut incoming_edges = self
                .ddg
                .edges_directed(node_index.to_owned(), Direction::Incoming);
            incoming_edges.any(|e| e.weight().to_owned() == Dependency::Consumes)
        } else {
            panic!("Variable is not defined even though the statement returns a value");
        }
    }

    pub fn get_owner(&self, stmt: &MethodInvStmt) -> (&Statement, usize) {
        for (i, s) in self.stmts.iter().enumerate() {
            if let Statement::Constructor(constructor) = s {
                if constructor.name().unwrap() == &stmt.owner() {
                    return (s, i);
                }
            }
        }
        panic!()
    }

    pub fn delete_stmt(&mut self, idx: usize) {
        self.stmts.remove(idx);
    }

    pub fn replace_stmt(&mut self, idx: usize, stmt: Statement) {
        std::mem::replace(&mut self.stmts[idx], stmt);
    }

    pub fn reorder_stmts(&mut self, idx_a: usize, idx_b: usize) {
        self.stmts.swap(idx_a, idx_b);
    }

    pub fn add_random_stmt(&mut self) {}

    pub fn to_syn(&self, decorate: bool) -> Item {
        let ident = Ident::new(
            &format!("{}_{}", TestCase::TEST_FN_PREFIX, self.id),
            Span::call_site(),
        );
        let id = self.id;

        let stmts: Vec<Stmt> = self.stmts.iter().map(Statement::to_syn).collect();
        let test: Item;

        if decorate {
            let set_test_id: Stmt = syn::parse_quote! {
                LOGGER.with(|l| l.borrow_mut().set_test_id(#id));
            };
            let wait: Stmt = syn::parse_quote! {
                LOGGER.with(|l| l.borrow_mut().wait());
            };
            test = syn::parse_quote! {
                #[test]
                fn #ident() {
                    #set_test_id
                    #(#stmts)*
                    #wait
                }
            };
        } else {
            test = syn::parse_quote! {
                #[test]
                fn #ident() {
                    #(#stmts)*
                }
            }
        }

        test
    }

    pub fn name(&self) -> String {
        format!("{}_{}", TestCase::TEST_FN_PREFIX, self.id)
    }

    pub fn complex_definitions(&mut self) -> Vec<ConstructorStmt> {
        self.stmts()
            .iter()
            .filter_map(|s| {
                if let Statement::Constructor(stmt) = s {
                    Some(stmt.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn unconsumed_complex_definitions(&mut self) -> Vec<Statement> {
        self.stmts()
            .iter()
            .filter_map(|s| {
                if s.returns_value() {
                    if !self.is_consumed(s) {
                        return Some(s.clone());
                    }
                }
                None
            })
            .collect()
    }

    pub fn primitive_definitions(&mut self) -> Vec<AssignStmt> {
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

    pub fn instantiated_types(&self) -> &Vec<T> {
        unimplemented!()
    }

    pub fn free_instantiated_types(&self) -> &Vec<T> {
        unimplemented!()
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
    pub fn var_counters(&self) -> &HashMap<String, usize> {
        &self.var_counters
    }
}

impl Display for TestCase {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let syn_item = self.to_syn(false);
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

    fn crossover(&self, other: &Self) -> (Self, Self)
    where
        Self: Sized,
    {
        self.crossover.crossover(self, other)
    }
}

#[derive(Debug, Clone)]
pub enum Arg {
    Var(VarArg),
}

impl Arg {
    pub fn to_syn(&self) -> Expr {
        match self {
            Arg::Var(var_arg) => var_arg.to_syn(),
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
        !self.param.is_by_reference()
    }

    pub fn is_by_reference(&self) -> bool {
        self.param.is_by_reference()
    }

    pub fn is_self(&self) -> bool {
        self.param.is_self()
    }

    pub fn to_syn(&self) -> Expr {
        if self.param.is_by_reference() {
            let expr = self.var.to_syn();
            syn::parse_quote! {
                &#expr
            }
        } else {
            self.var.to_syn()
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
            Statement::PrimitiveAssignment(_) => {
                unimplemented!()
            }
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
            Statement::PrimitiveAssignment(a) => a.return_type(),
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
}

#[derive(Debug)]
pub struct AssignStmt {
    id: Uuid,
    var: Option<Var>,
}

impl AssignStmt {
    pub fn new() -> Self {
        AssignStmt {
            id: Uuid::new_v4(),
            var: None,
        }
    }

    pub fn var(&self) -> Option<&Var> {
        self.var.as_ref()
    }

    pub fn return_type(&self) -> Option<&T> {
        unimplemented!()
    }

    pub fn set_var(&mut self, var: Var) {
        self.var = Some(var);
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

impl Clone for AssignStmt {
    fn clone(&self) -> Self {
        AssignStmt {
            id: Uuid::new_v4(),
            var: self.var.clone(),
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

            let type_name = self.constructor.parent.to_string();
            let args: Vec<Expr> = self.args.iter().cloned().map(|a| a.value).collect();
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
    pub fn owner(&self) -> Var {
        let first_param = self.params().first().unwrap();
        if first_param.is_self() {
            let owner_type = first_param.ty();
            Var::new(&first_arg.name().unwrap(), owner_type.clone())
        } else {
            panic!("There should be an owner")
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

    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn to_syn(&self) -> Stmt {
        let ident = &self.func.item_fn.sig.ident;
        let args: Vec<Expr> = self.args.iter().cloned().map(|a| a.value).collect();

        syn::parse_quote! {
            #ident(#(#args),*);
        }
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug)]
pub struct StatementGenerator {
    source_file: SourceFile,
}

impl StatementGenerator {
    pub fn new(source_file: SourceFile) -> Self {
        StatementGenerator { source_file }
    }

    /// Generates a complete random statement and all its dependencies, even if the
    /// test already contains some definitions that can be reused.
    ///
    /// # Arguments
    ///
    /// * `test_case` - The test case where a randomly generated statement should be inserted into.
    pub fn insert_random_stmt(&self, test_case: &mut TestCase) {
        // TODO primitive statements are not being generated yet
        let callables = self.source_file.callables();
        let i = fastrand::usize(0..callables.len());
        let callable = callables.get(i).unwrap();

        let args: Vec<Arg> = callable
            .params()
            .iter()
            .map(|p| self.generate_arg(test_case, param))
            .collect();
        let stmt = callable.to_stmt(args);
        test_case.add_stmt(stmt);
    }

    fn generate_arg(&self, test_case: &mut TestCase, param: &Param) -> Arg {
        let generators = self.source_file.generators(param.ty());

        // Pick a random generator
        let i = fastrand::usize(0..generators.len());
        let generator = generators.get(i).unwrap();
        let args: Vec<Arg> = generator
            .params()
            .iter()
            .map(|p| {
                // TODO instantiate new object with a 10% probability even if there is a free ones
                // already in the test case
                if !test_case.instantiated_types().contains(p.ty()) {
                    let var = self.generate_arg(test_case, p).unwrap();
                    (var, param.clone())
                } else {
                    unimplemented!()
                }
            })
            .map(|(var, param)| Arg::Var(VarArg::new(var, param)))
            .collect();

        let stmt = generator.to_stmt(args);

        let return_var = test_case.add_stmt(stmt).unwrap();

        Arg::Var(VarArg::new(return_var, param.clone()))
    }

    /*pub fn get_random_stmt_old(&self, test_case: &mut TestCase) -> Statement {
        let unconsumed_defs = test_case.unconsumed_complex_definitions();
        if unconsumed_defs.is_empty() {
            let structs = self.source_file.structs();
            let i = fastrand::usize(0..structs.len());
            let item_struct = structs.get(i).unwrap();
            if let Some(constructor) = item_struct.constructor() {
                let params = constructor.params();
                let args: Vec<Arg> = params
                    .iter()
                    .map(PrimitivesGenerator::generate_arg)
                    .collect();
                Statement::Constructor(ConstructorStmt::new(constructor.clone(), args))
            } else {
                // No constructor, so initialize directly
                unimplemented!()
            }
        } else {
            let i = fastrand::usize(0..unconsumed_defs.len());
            let constructor_stmt = unconsumed_defs.get(i).unwrap();

            let methods = constructor_stmt.struct_item.methods();
            // TODO what if there are no methods

            let i = fastrand::usize(0..methods.len());
            let method = methods.get(i).unwrap();

            let params = method.params();
            let args: Vec<Arg> = params
                .iter()
                .filter_map(|p| {

                })
                .collect();

            let owner = constructor_stmt.name();
            if let Some(owner_var) = owner {
                let stmt = MethodInvStmt::new(method.clone(), args);

                if stmt.returns_value() {
                    // TODO
                }

                Statement::MethodInvocation(stmt)
            } else {
                panic!("Owner var is not set")
            }
        }
    }*/

    /*fn construct_arg(&self, test_case: &mut TestCase, param: &Param, type_path: &TypePath) -> Arg {
        // Find the required struct by the param from the registered structs in the source code
        let struct_type = self
            .source_file
            .structs()
            .iter()
            .filter(|&s| s.ident == *type_path.path.get_ident().unwrap())
            .last()
            .unwrap();

        // Get the constructor to initialize the type
        let constructor = struct_type.constructor().as_ref().unwrap();

        // Get the params for the constructor
        let constructor_params: Vec<FnArg> = constructor.sig.inputs.iter().cloned().collect();

        // Generate arguments for the constructor invocation
        let constructor_args: Vec<Arg> = constructor_params
            .iter()
            .map(PrimitivesGenerator::generate_arg)
            .collect();

        // Create constructor invocation
        let stmt = Statement::Constructor(ConstructorStmt::new(
            struct_type.clone(),
            constructor.clone(),
            constructor_args,
        ));

        if let Some(var_name) = test_case.insert_stmt(0, stmt) {
            let var_ident = Ident::new(&var_name, Span::call_site());
            let var = syn::parse_quote! {#var_ident};
            Arg::new(Some(var_name), var, param.clone(), false)
        } else {
            panic!()
        }
    }*/
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

    pub fn is_by_reference(&self) -> bool {
        match self {
            Param::Self_(_) => true,
            Param::Regular(_) => false,
        }
    }

    pub fn ty(&self) -> &T {
        match self {
            Param::Self_(self_param) => &self_param.ty,
            Param::Regular(regular_param) => &regular_param.ty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelfParam {
    ty: T,
    fn_arg: FnArg,
    by_reference: bool,
}

impl SelfParam {
    pub fn new(ty: T, fn_arg: FnArg, by_reference: bool) -> Self {
        SelfParam {
            ty,
            fn_arg,
            by_reference,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegularParam {
    ty: T,
    fn_arg: FnArg,
}

impl RegularParam {
    pub fn new(ty: T, fn_arg: FnArg) -> Self {
        RegularParam { ty, fn_arg }
    }

    pub fn ty(&self) -> &T {
        &self.ty
    }
    pub fn fn_arg(&self) -> &FnArg {
        &self.fn_arg
    }
}

#[derive(Debug, Clone)]
pub struct T {
    name: String,

    // syn defined type
    ty: Box<Type>,
}

impl PartialEq for T {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
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
}

impl Callable {
    pub fn params(&self) -> &Vec<Param> {
        match self {
            Callable::Method(method_item) => &method_item.params,
            Callable::StaticFunction(fn_item) => &fn_item.params,
            Callable::Function(fn_item) => &fn_item.params,
            Callable::Constructor(constructor_item) => &constructor_item.params,
        }
    }

    pub fn return_type(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => method_item.return_type.as_ref(),
            Callable::StaticFunction(fn_item) => fn_item.return_type.as_ref(),
            Callable::Function(fn_item) => fn_item.return_type.as_ref(),
            Callable::Constructor(constructor_item) => Some(&constructor_item.return_type),
        }
    }

    pub fn parent(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => Some(&method_item.parent),
            Callable::StaticFunction(fn_item) => Some(&fn_item.parent),
            Callable::Function(_) => None,
            Callable::Constructor(constructor) => Some(&constructor.parent),
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
        }
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

#[derive(Debug)]
pub struct TestCaseGenerator {
    branch_manager: Rc<RefCell<BranchManager>>,
    statement_generator: Rc<StatementGenerator>,
    mutation: BasicMutation,
    crossover: BasicCrossover,
    test_id: Rc<RefCell<TestIdGenerator>>,
}

impl TestCaseGenerator {
    pub fn new(
        statement_generator: Rc<StatementGenerator>,
        branch_manager: Rc<RefCell<BranchManager>>,
        mutation: BasicMutation,
        crossover: BasicCrossover,
        test_id: Rc<RefCell<TestIdGenerator>>,
    ) -> TestCaseGenerator {
        TestCaseGenerator {
            statement_generator,
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
        let mut test_case = TestCase::new(
            0,
            self.mutation.clone(),
            self.crossover.clone(),
            self.branch_manager.clone(),
        );

        self.statement_generator.insert_random_stmt(&mut test_case);

        let test_id = self.test_id.borrow_mut().next_id();
        test_case.set_id(test_id);

        test_case
    }
}
