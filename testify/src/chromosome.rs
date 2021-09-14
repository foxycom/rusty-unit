use std::fmt::{Debug, Display, Formatter, Error};
use syn::{Stmt, Item, ItemFn, FnArg, PatType, Type, Expr, ImplItemMethod, ReturnType, Pat, PatPath};
use std::cmp::Ordering;
use quote::ToTokens;
use proc_macro2::{Ident, Span};
use std::collections::HashMap;
use crate::generators::{InputGenerator, TestIdGenerator};
use crate::operators::{BasicMutation, BasicCrossover};
use std::rc::Rc;
use std::fs;
use std::hash::{Hash, Hasher};
use crate::parser::TraceParser;
use std::cell::RefCell;
use crate::source::{Branch, BranchManager, SourceFile};
use petgraph::{Graph, Directed};
use petgraph::prelude::{GraphMap, DiGraphMap, StableDiGraph};
use petgraph::visit::Walker;
use petgraph::graph::NodeIndex;
use uuid::Uuid;


pub trait Chromosome: Clone + Debug {
    fn mutate(&self) -> Self;

    fn fitness(&self, objective: &Branch) -> f64;

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized;
}

pub trait ChromosomeGenerator {
    type C: Chromosome;

    fn generate(&mut self) -> Self::C;
}

#[derive(Debug, Clone)]
pub enum Dependency {
    Owns,
    Uses,
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
    index_table: HashMap<String, NodeIndex>,
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
            index_table: self.index_table.clone(),
            var_counters: self.var_counters.clone(),
        }
    }
}

impl PartialEq for TestCase {
    fn eq(&self, other: &Self) -> bool {
        /*self.stmts == other.stmts && self.objective == other.objective*/
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

    pub fn new(id: u64,
               mutation: BasicMutation,
               crossover: BasicCrossover,
               branch_manager: Rc<RefCell<BranchManager>>) -> Self {
        TestCase {
            id,
            stmts: Vec::new(),
            results: HashMap::new(),
            mutation,
            crossover,
            branch_manager,
            ddg: StableDiGraph::new(),
            index_table: HashMap::new(),
            var_counters: HashMap::new(),
        }
    }

    pub fn stmts(&self) -> &Vec<Statement> {
        &self.stmts
    }

    fn set_var_name(&mut self, stmt: &mut Statement) -> String {
        match stmt {
            Statement::PrimitiveAssignment(_) => {
                unimplemented!()
            }
            Statement::Constructor(constructor_stmt) => {
                let type_name = constructor_stmt.struct_item().ident.to_string().to_lowercase();
                let counter = self.var_counters
                    .entry(type_name.clone())
                    .and_modify(|c| *c = *c + 1)
                    .or_insert(0);

                let var_name = format!("{}_{}", type_name, self.var_counters.get(&type_name).unwrap());
                constructor_stmt.set_name(var_name.clone());
                return var_name.to_owned();
            }
            Statement::MethodInvocation(method_inv_stmt) => {
                if method_inv_stmt.returns_value() {}
            }
            Statement::FunctionInvocation(_) => {
                unimplemented!()
            }
            _ => {
                panic!()
            }
        }
        "no_name".to_owned()
    }

    pub fn insert_stmt(&mut self, idx: usize, mut stmt: Statement) -> String {
        let var_name = self.set_var_name(&mut stmt);

        match &mut stmt {
            Statement::PrimitiveAssignment(_) => {}
            Statement::Constructor(constructor_stmt) => {
                let node_index = self.ddg.add_node(constructor_stmt.name.to_owned());
                self.index_table.insert(constructor_stmt.name.to_owned(), node_index);
            }
            Statement::AttributeAccess(_) => {}
            Statement::MethodInvocation(method_inv_stmt) => {
                // TODO Store into map when method invocation returns something
                let owner_index = self.index_table.get(&method_inv_stmt.owner).unwrap();
                let method_index = self.ddg.add_node(method_inv_stmt.id.to_string());
                self.ddg.add_edge(owner_index.clone(), method_index, Dependency::Owns);
            }
            Statement::FunctionInvocation(_) => {}
        }
        self.stmts.insert(idx, stmt);
        var_name
    }

    pub fn get_owner(&self, stmt: &MethodInvStmt) -> (&Statement, usize) {
        for (i, s) in self.stmts.iter().enumerate() {
            if let Statement::Constructor(constructor) = s {
                if constructor.name == stmt.owner {
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

    pub fn to_syn(&self) -> Item {
        let ident = Ident::new(&format!("{}_{}", TestCase::TEST_FN_PREFIX, self.id),
                               Span::call_site());
        let id = self.id;
        let set_test_id: Stmt = syn::parse_quote! {
              LOGGER.with(|l| l.borrow_mut().set_test_id(#id));
        };
        let wait: Stmt = syn::parse_quote! {
            LOGGER.with(|l| l.borrow_mut().wait());
        };

        let stmts: Vec<Stmt> = self.stmts.iter().map(Statement::to_syn).collect();
        let test: Item = syn::parse_quote! {
            #[test]
            fn #ident() {
                #set_test_id
                #(#stmts)*
                #wait
            }
        };
        test
    }

    pub fn name(&self) -> String {
        format!("{}_{}", TestCase::TEST_FN_PREFIX, self.id)
    }

    pub fn complex_definitions(&mut self) -> Vec<ConstructorStmt> {
        self.stmts().iter().filter_map(|s| if let Statement::Constructor(constructor_stmt) = s {
            Some(constructor_stmt.clone())
        } else {
            None
        }).collect()
    }

    pub fn primitive_definitions(&mut self) -> Vec<AssignStmt> {
        self.stmts().iter().filter_map(|s| if let Statement::PrimitiveAssignment(stmt) = s {
            Some(stmt.clone())
        } else {
            None
        }).collect()
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
        let syn_item = self.to_syn();
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

    fn crossover(&self, other: &Self) -> (Self, Self) where Self: Sized {
        self.crossover.crossover(self, other)
    }
}

#[derive(Clone, Debug)]
pub enum Statement {
    PrimitiveAssignment(AssignStmt),
    Constructor(ConstructorStmt),
    AttributeAccess(AttrStmt),
    MethodInvocation(MethodInvStmt),
    FunctionInvocation(FnInvStmt),
}

impl Statement {
    pub fn to_syn(&self) -> Stmt {
        match self {
            Statement::PrimitiveAssignment(_) => {
                unimplemented!()
            }
            Statement::Constructor(constructor_stmt) => {
                constructor_stmt.to_syn()
            }
            Statement::AttributeAccess(_) => {
                unimplemented!()
            }
            Statement::MethodInvocation(method_inv_stmt) => {
                method_inv_stmt.to_syn()
            }
            Statement::FunctionInvocation(fn_inv_stmt) => {
                fn_inv_stmt.to_syn()
            }
        }
    }
}

#[derive(Debug)]
pub struct AssignStmt {
    id: Uuid,
}

impl AssignStmt {
    pub fn new() -> Self {
        AssignStmt {
            id: Uuid::new_v4()
        }
    }
}

impl Clone for AssignStmt {
    fn clone(&self) -> Self {
        AssignStmt {
            id: Uuid::new_v4()
        }
    }
}

#[derive(Debug)]
pub struct ConstructorStmt {
    id: Uuid,
    name: String,
    struct_item: Struct,
    params: Vec<FnArg>,
    args: Vec<Expr>,
}

impl Clone for ConstructorStmt {
    fn clone(&self) -> Self {
        ConstructorStmt {
            id: Uuid::new_v4(),
            name: self.name.clone(),
            struct_item: self.struct_item.clone(),
            params: self.params.clone(),
            args: self.args.clone(),
        }
    }
}

impl ConstructorStmt {
    pub fn new(name: String, struct_item: Struct, params: Vec<FnArg>, args: Vec<Expr>) -> Self {
        ConstructorStmt {
            name,
            struct_item,
            params,
            args,
            id: Uuid::new_v4(),
        }
    }

    pub fn to_syn(&self) -> Stmt {
        let name = Ident::new(&self.name, Span::call_site());
        let struct_item = &self.struct_item;
        let type_name = &struct_item.ident;
        let args = &self.args;
        syn::parse_quote! {
            let mut #name = #type_name::new(#(#args),*);
        }
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn struct_item(&self) -> &Struct {
        &self.struct_item
    }
    pub fn params(&self) -> &Vec<FnArg> {
        &self.params
    }
    pub fn args(&self) -> &Vec<Expr> {
        &self.args
    }

    pub fn set_args(&mut self, args: Vec<Expr>) {
        self.args = args;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[derive(Debug)]
pub struct AttrStmt {
    id: Uuid,
}

impl Clone for AttrStmt {
    fn clone(&self) -> Self {
        AttrStmt {
            id: Uuid::new_v4()
        }
    }
}

impl AttrStmt {
    pub fn new() -> Self {
        AttrStmt {
            id: Uuid::new_v4()
        }
    }
}

#[derive(Debug)]
pub struct MethodInvStmt {
    id: Uuid,
    owner: String,
    name: Option<String>,
    method: ImplItemMethod,
    params: Vec<FnArg>,
    args: Vec<Expr>,
}

impl Clone for MethodInvStmt {
    fn clone(&self) -> Self {
        MethodInvStmt {
            id: Uuid::new_v4(),
            owner: self.owner.clone(),
            name: self.name.clone(),
            method: self.method.clone(),
            params: self.params.clone(),
            args: self.args.clone(),
        }
    }
}

impl MethodInvStmt {
    pub fn new(owner: String, method: ImplItemMethod, params: Vec<FnArg>, args: Vec<Expr>) -> Self {
        MethodInvStmt { owner, method, params, args, id: Uuid::new_v4(), name: None }
    }

    pub fn to_syn(&self) -> Stmt {
        let ident = &self.method.sig.ident;
        let args = &self.args;
        let owner = Ident::new(&self.owner, Span::call_site());
        syn::parse_quote! {
            #owner.#ident(#(#args),*);
        }
    }

    pub fn returns_value(&self) -> bool {
        if let ReturnType::Default = &self.method.sig.output {
            true
        } else {
            false
        }
    }

    pub fn return_type(&self) -> Option<Type> {
        match &self.method.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, t) => {
                unimplemented!()
            }
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn owner(&self) -> &str {
        &self.owner
    }
    pub fn method(&self) -> &ImplItemMethod {
        &self.method
    }
    pub fn params(&self) -> &Vec<FnArg> {
        &self.params
    }
    pub fn args(&self) -> &Vec<Expr> {
        &self.args
    }

    pub fn set_args(&mut self, args: Vec<Expr>) {
        self.args = args;
    }

    pub fn set_owner(&mut self, owner: String) {
        self.owner = owner;
    }
    pub fn name(&self) -> &Option<String> {
        &self.name
    }
}

#[derive(Debug)]
pub struct FnInvStmt {
    id: Uuid,
    params: Vec<FnArg>,
    args: Vec<Expr>,
    ident: Ident,
    item_fn: ItemFn,
}

impl Clone for FnInvStmt {
    fn clone(&self) -> Self {
        FnInvStmt {
            id: Uuid::new_v4(),
            params: self.params.clone(),
            args: self.args.clone(),
            ident: self.ident.clone(),
            item_fn: self.item_fn.clone(),
        }
    }
}

impl FnInvStmt {
    pub fn new(ident: Ident, item_fn: ItemFn, params: Vec<FnArg>, args: Vec<Expr>) -> Self {
        FnInvStmt { params, args, ident, item_fn, id: Uuid::new_v4() }
    }

    pub fn params(&self) -> &Vec<FnArg> {
        &self.params
    }
    pub fn args(&self) -> &Vec<Expr> {
        &self.args
    }
    pub fn ident(&self) -> &Ident {
        &self.ident
    }
    pub fn item_fn(&self) -> &ItemFn {
        &self.item_fn
    }

    pub fn has_params(&self) -> bool {
        !self.params.is_empty()
    }

    pub fn set_args(&mut self, args: Vec<Expr>) {
        self.args = args;
    }

    pub fn to_syn(&self) -> Stmt {
        let ident = &self.ident;
        let args = &self.args;

        syn::parse_quote! {
            #ident(#(#args),*);
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

    pub fn get_random_stmt(&self, test_case: &mut TestCase) -> Statement {
        if test_case.complex_definitions().is_empty() {
            let structs = self.source_file.structs();
            let i = fastrand::usize((0..structs.len()));

            let item_struct = structs.get(i).unwrap();
            if let Some(constructor) = item_struct.constructor() {
                let params: Vec<FnArg> = constructor.sig.inputs.iter().cloned().collect();
                let args: Vec<Expr> = params.iter().map(InputGenerator::generate_arg).collect();
                Statement::Constructor(ConstructorStmt::new(
                    "a".to_string(),
                    item_struct.clone(),
                    params,
                    args,
                ))
            } else {
                // No constructor, so initialize directly
                unimplemented!()
            }
        } else {
            let complex_defs = test_case.complex_definitions();

            let i = fastrand::usize((0..complex_defs.len()));
            let constructor = complex_defs.get(i).unwrap();

            let methods = constructor.struct_item.methods();
            // TODO what if there are no methods

            let i = fastrand::usize((0..methods.len()));
            let method = methods.get(i).unwrap();

            let params: Vec<FnArg> = method.sig.inputs.iter().cloned().collect();
            let args: Vec<Expr> = params.iter().filter_map(|a| {
                match a {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(arg_pat_type) => {
                        if InputGenerator::is_primitive(a) {
                            Some(InputGenerator::generate_arg(a))
                        } else {
                            if let Type::Path(type_path) = &arg_pat_type.ty.as_ref() {
                                let struct_type = self.source_file.structs()
                                    .iter()
                                    .filter(|&s| s.ident == *type_path.path.get_ident().unwrap())
                                    .last().unwrap();
                                let constructor = struct_type.constructor().as_ref().unwrap();
                                let constructor_params: Vec<FnArg> = constructor.sig.inputs.iter()
                                    .cloned()
                                    .collect();
                                let constructor_args: Vec<Expr> = constructor_params.iter()
                                    .map(InputGenerator::generate_arg).collect();
                                let stmt = Statement::Constructor(ConstructorStmt::new(
                                    "a".to_string(), struct_type.clone(),
                                    constructor_params, constructor_args
                                ));
                                let var_name = test_case.insert_stmt(0, stmt);
                                let var_ident = Ident::new(&var_name, Span::call_site());
                                let var = syn::parse_quote! {#var_ident};
                                Some(var)
                            } else {
                                unimplemented!()
                            }
                        }
                    }
                }
            }).collect();

            Statement::MethodInvocation(MethodInvStmt::new(constructor.name.to_owned(),
                                                           method.clone(), params, args))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Struct {
    ident: Ident,
    constructor: Option<ImplItemMethod>,
    methods: Vec<ImplItemMethod>,
    static_methods: Vec<ImplItemMethod>,
}

impl Struct {
    pub fn new(ident: Ident) -> Struct {
        Struct {
            ident,
            constructor: None,
            methods: Vec::new(),
            static_methods: Vec::new(),
        }
    }

    pub fn constructor(&self) -> &Option<ImplItemMethod> {
        &self.constructor
    }

    pub fn methods(&self) -> &Vec<ImplItemMethod> {
        &self.methods
    }
    pub fn static_methods(&self) -> &Vec<ImplItemMethod> {
        &self.static_methods
    }
    pub fn set_constructor(&mut self, constructor: ImplItemMethod) {
        self.constructor = Some(constructor);
    }


    pub fn add_method(&mut self, method: ImplItemMethod) {
        self.methods.push(method);
    }

    pub fn add_static_method(&mut self, method: ImplItemMethod) {
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
    pub fn new(statement_generator: Rc<StatementGenerator>,
               branch_manager: Rc<RefCell<BranchManager>>,
               mutation: BasicMutation,
               crossover: BasicCrossover,
               test_id: Rc<RefCell<TestIdGenerator>>) -> TestCaseGenerator {
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

        let stmt = self.statement_generator.get_random_stmt(&mut test_case);

        let test_id = self.test_id.borrow_mut().next_id();
        test_case.set_id(test_id);
        test_case.insert_stmt(0, stmt);

        test_case
    }
}

