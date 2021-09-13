use std::fmt::{Debug, Display, Formatter, Error};
use syn::{Stmt, Item, ItemFn, FnArg, PatType, Type, Expr, ImplItemMethod};
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
use petgraph::prelude::{GraphMap, DiGraphMap};
use crate::chromosome::Dependency::Owner;


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
    Owner
}

#[derive(Debug)]
pub struct TestCase {
    id: u64,
    stmts: Vec<Statement>,
    results: HashMap<u64, f64>,
    mutation: BasicMutation,
    crossover: BasicCrossover,
    ddg: GraphMap<Statement, Dependency, Directed>,
    branch_manager: Rc<RefCell<BranchManager>>,
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
            ddg: self.ddg.clone()
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
               stmts: Vec<Statement>,
               mutation: BasicMutation,
               crossover: BasicCrossover,
               branch_manager: Rc<RefCell<BranchManager>>) -> Self {
        TestCase {
            id,
            stmts,
            results: HashMap::new(),
            mutation,
            crossover,
            branch_manager,
            ddg: GraphMap::new()
        }
    }

    pub fn stmts(&self) -> &Vec<Statement> {
        &self.stmts
    }

    pub fn insert_stmt(&mut self, idx: usize, stmt: Statement) {
        match &stmt {
            Statement::PrimitiveAssignment(_) => {}
            Statement::Constructor(_) => {}
            Statement::AttributeAccess(_) => {}
            Statement::MethodInvocation(method_inv_stmt) => {
                self.ddg.add_edge(stmt, stmt, Owner);
            }
            Statement::FunctionInvocation(_) => {}
        }
        self.stmts.insert(idx, stmt);
    }

    pub fn delete_stmt(&mut self, idx: usize) {
        self.stmts.remove(idx);
    }

    pub fn replace_stmt(&mut self, idx: usize, stmt: Statement) {
        std::mem::replace(&mut self.stmts[idx], stmt);
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

#[derive(Clone, Debug)]
pub struct AssignStmt {}

impl AssignStmt {
    pub fn new() -> Self {
        AssignStmt {}
    }
}

#[derive(Clone, Debug)]
pub struct ConstructorStmt {
    name: String,
    struct_item: Struct,
    params: Vec<FnArg>,
    args: Vec<Expr>,
}

impl ConstructorStmt {
    pub fn new(name: String, struct_item: Struct, params: Vec<FnArg>, args: Vec<Expr>) -> Self {
        ConstructorStmt {
            name,
            struct_item,
            params,
            args,
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
}

#[derive(Clone, Debug)]
pub struct AttrStmt {}

impl AttrStmt {
    pub fn new() -> Self {
        AttrStmt {}
    }
}

#[derive(Clone, Debug)]
pub struct MethodInvStmt {
    owner: String,
    method: ImplItemMethod,
    params: Vec<FnArg>,
    args: Vec<Expr>,
}

impl MethodInvStmt {
    pub fn new(owner: String, method: ImplItemMethod, params: Vec<FnArg>, args: Vec<Expr>) -> Self {
        MethodInvStmt { owner, method, params, args }
    }

    pub fn to_syn(&self) -> Stmt {
        let ident = &self.method.sig.ident;
        let args = &self.args;
        let owner = Ident::new(&self.owner, Span::call_site());
        syn::parse_quote! {
            #owner.#ident(#(#args),*);
        }
    }
}

#[derive(Clone, Debug)]
pub struct FnInvStmt {
    params: Vec<FnArg>,
    args: Vec<Expr>,
    ident: Ident,
    item_fn: ItemFn,
}

impl FnInvStmt {
    pub fn new(ident: Ident, item_fn: ItemFn, params: Vec<FnArg>, args: Vec<Expr>) -> Self {
        FnInvStmt { params, args, ident, item_fn }
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
            let i = fastrand::usize((0..methods.len()));
            let method = methods.get(i).unwrap();

            let params: Vec<FnArg> = method.sig.inputs.iter().cloned().collect();
            let args: Vec<Expr> = params.iter().filter_map(|a| {
                if let FnArg::Receiver(_) = a {
                    None
                } else {
                    Some(InputGenerator::generate_arg(a))
                }
            }).collect();

            // TODO variable name
            Statement::MethodInvocation(MethodInvStmt::new(constructor.name.to_owned(),
                                                           method.clone(), params, args))
        }


        /*let sig = &item_fn.sig;
        let params: Vec<FnArg> = sig.inputs.iter().cloned().collect();
        let args: Vec<Expr> = params.iter().map(InputGenerator::generate_arg).collect();

        (Statement::FunctionInvocation(FnInvStmt::new(
            sig.ident.clone(),
            item_fn.clone(),
            params,
            args,
        )), branch.clone())*/
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
            vec![],
            self.mutation.clone(),
            self.crossover.clone(),
            self.branch_manager.clone(),
        );

        let stmt = self.statement_generator.get_random_stmt(&mut test_case);

        let test_id = self.test_id.borrow_mut().next_id();
        test_case.id = test_id;
        test_case.stmts = vec![stmt];

        test_case
    }
}

