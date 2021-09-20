use crate::generators::{PrimitivesGenerator, TestIdGenerator};
use crate::operators::{BasicCrossover, BasicMutation};
use crate::source::{Branch, BranchManager, SourceFile};
use petgraph::prelude::{NodeIndex, StableDiGraph};
use petgraph::Direction;
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use syn::{
    Expr, FnArg, ImplItemMethod, Item, ItemFn, Pat, PatType, Path, ReturnType, Stmt, Type, TypePath,
};
use uuid::Uuid;

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
            index_table: HashMap::new(),
            var_counters: HashMap::new(),
        }
    }

    pub fn stmts(&self) -> &Vec<Statement> {
        &self.stmts
    }

    fn set_var_name(&mut self, stmt: &mut Statement) -> Option<String> {
        return match stmt {
            Statement::PrimitiveAssignment(_) => unimplemented!(),
            Statement::Constructor(constructor_stmt) => {
                let type_name = constructor_stmt
                    .struct_item()
                    .ident
                    .to_string()
                    .to_lowercase();
                let counter = self
                    .var_counters
                    .entry(type_name.clone())
                    .and_modify(|c| *c = *c + 1)
                    .or_insert(0);

                let var_name = format!("{}_{}", type_name, counter);
                constructor_stmt.set_name(&var_name);
                Some(var_name)
            }
            Statement::MethodInvocation(method_inv_stmt) => {
                if method_inv_stmt.returns_value() {
                    let type_name = return_type_name(&method_inv_stmt.method.sig.output);
                    if let Some(type_name) = type_name {
                        let type_name = type_name.to_lowercase();
                        let counter = self
                            .var_counters
                            .entry(type_name.clone())
                            .and_modify(|c| *c = *c + 1)
                            .or_insert(0);
                        let var_name = format!("{}_{}", type_name, counter);
                        method_inv_stmt.set_name(&var_name);
                        Some(var_name)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Statement::FunctionInvocation(_) => unimplemented!(),
            _ => panic!(),
        };
    }

    pub fn insert_stmt(&mut self, idx: usize, mut stmt: Statement) -> Option<String> {
        let var_name = self.set_var_name(&mut stmt);

        match &mut stmt {
            Statement::PrimitiveAssignment(_) => {}
            Statement::Constructor(constructor_stmt) => {
                let var_name = constructor_stmt.name().unwrap();
                let node_index = self.ddg.add_node(var_name.to_owned());
                self.index_table.insert(var_name.to_owned(), node_index);
            }
            Statement::AttributeAccess(_) => {}
            Statement::MethodInvocation(method_inv_stmt) => {
                method_inv_stmt.args.iter().for_each(|a| {
                    if a.is_primitive() {
                        // do nothing for now
                    } else {
                        // TODO extract method to insert new nodes
                        let new_node = self.ddg.add_node(method_inv_stmt.id.to_string());
                        self.index_table
                            .insert(method_inv_stmt.id.to_string(), new_node.clone());
                        let arg_name = a.name();
                        if let Some(_) = arg_name {
                            if a.is_by_reference() {
                                self.ddg.add_edge(
                                    new_node,
                                    self.index_table.get(&a.name().unwrap()).unwrap().clone(),
                                    Dependency::Borrows,
                                );
                            } else {
                                // The arg is being consumed
                                self.ddg.add_edge(
                                    new_node,
                                    self.index_table.get(&a.name().unwrap()).unwrap().clone(),
                                    Dependency::Consumes,
                                );
                            }
                        } else {
                            panic!("Variable name has not been set")
                        }
                    }
                });

                // TODO Store into map when method invocation returns something
                let owner_index = self.index_table.get(&method_inv_stmt.owner).unwrap();
                let method_index = self.ddg.add_node(method_inv_stmt.id.to_string());
                self.ddg
                    .add_edge(owner_index.clone(), method_index, Dependency::Owns);
            }
            Statement::FunctionInvocation(_) => {}
        }
        self.stmts.insert(idx, stmt);
        var_name
    }

    pub fn is_consumed(&self, constructor_stmt: &ConstructorStmt) -> bool {
        if let Some(var_name) = constructor_stmt.name() {
            let node_index = self.index_table.get(var_name).unwrap();
            let mut incoming_edges = self
                .ddg
                .edges_directed(node_index.to_owned(), Direction::Incoming);
            incoming_edges.any(|e| e.weight().to_owned() == Dependency::Consumes)
        } else {
            panic!("Name is not set")
        }
    }

    pub fn get_owner(&self, stmt: &MethodInvStmt) -> (&Statement, usize) {
        for (i, s) in self.stmts.iter().enumerate() {
            if let Statement::Constructor(constructor) = s {
                if constructor.name().unwrap().to_string() == stmt.owner {
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

    pub fn unconsumed_complex_definitions(&mut self) -> Vec<ConstructorStmt> {
        self.stmts()
            .iter()
            .filter_map(|s| {
                if let Statement::Constructor(stmt) = s {
                    if !self.is_consumed(stmt) {
                        return Some(stmt.clone());
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
pub struct Arg {
    name: Option<String>,
    param: FnArg,
    value: Expr,
    primitive: bool,
}

impl Arg {
    pub fn new(name: Option<String>, value: Expr, param: FnArg, primitive: bool) -> Self {
        Arg {
            name,
            value,
            param,
            primitive,
        }
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn value(&self) -> &Expr {
        &self.value
    }

    pub fn param(&self) -> &FnArg {
        &self.param
    }

    pub fn is_primitive(&self) -> bool {
        self.primitive
    }

    pub fn is_self(&self) -> bool {
        if let FnArg::Receiver(_) = self.param {
            true
        } else {
            false
        }
    }

    pub fn is_by_reference(&self) -> bool {
        match &self.param {
            FnArg::Receiver(_) => {
                unimplemented!()
            }
            FnArg::Typed(PatType { ty, .. }) => match ty.as_ref() {
                Type::Reference(_) => true,
                _ => false,
            },
        }
    }
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
    pub fn set_value(&mut self, value: Expr) {
        self.value = value;
    }
    pub fn set_param(&mut self, param: FnArg) {
        self.param = param;
    }

    pub fn to_syn(&self) -> Expr {
        // TODO this is only limited to basic references
        if self.is_by_reference() {
            let value = &self.value;
            syn::parse_quote! {
                &#value
            }
        } else {
            self.value.clone()
        }
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
            Statement::Constructor(constructor_stmt) => constructor_stmt.to_syn(),
            Statement::AttributeAccess(_) => {
                unimplemented!()
            }
            Statement::MethodInvocation(method_inv_stmt) => method_inv_stmt.to_syn(),
            Statement::FunctionInvocation(fn_inv_stmt) => fn_inv_stmt.to_syn(),
        }
    }
}

#[derive(Debug)]
pub struct AssignStmt {
    id: Uuid,
}

impl AssignStmt {
    pub fn new() -> Self {
        AssignStmt { id: Uuid::new_v4() }
    }
}

impl Clone for AssignStmt {
    fn clone(&self) -> Self {
        AssignStmt { id: Uuid::new_v4() }
    }
}

#[derive(Debug)]
pub struct ConstructorStmt {
    id: Uuid,
    name: Option<String>,
    struct_item: Struct,
    params: Vec<FnArg>,
    args: Vec<Arg>,
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
    pub fn new(struct_item: Struct, params: Vec<FnArg>, args: Vec<Arg>) -> Self {
        ConstructorStmt {
            name: None,
            struct_item,
            params,
            args,
            id: Uuid::new_v4(),
        }
    }

    pub fn to_syn(&self) -> Stmt {
        if let Some(name) = &self.name {
            let name = Ident::new(name, Span::call_site());
            let struct_item = &self.struct_item;
            let type_name = &struct_item.ident;
            let args: Vec<Expr> = self.args.iter().cloned().map(|a| a.value).collect();
            syn::parse_quote! {
                let mut #name = #type_name::new(#(#args),*);
            }
        } else {
            panic!("Name must have been set until here")
        }
    }
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    pub fn struct_item(&self) -> &Struct {
        &self.struct_item
    }
    pub fn params(&self) -> &Vec<FnArg> {
        &self.params
    }
    pub fn args(&self) -> &Vec<Arg> {
        &self.args
    }

    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_owned());
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
}

#[derive(Debug)]
pub struct MethodInvStmt {
    id: Uuid,
    owner: String,
    name: Option<String>,
    method: ImplItemMethod,
    params: Vec<FnArg>,
    args: Vec<Arg>,
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
    pub fn new(owner: &str, method: ImplItemMethod, params: Vec<FnArg>, args: Vec<Arg>) -> Self {
        MethodInvStmt {
            owner: owner.to_owned(),
            method,
            params,
            args,
            id: Uuid::new_v4(),
            name: None,
        }
    }

    pub fn to_syn(&self) -> Stmt {
        let method_ident = &self.method.sig.ident;
        let args: Vec<Expr> = self.args.iter().cloned().map(|a| a.to_syn()).collect();
        let owner = Ident::new(&self.owner, Span::call_site());
        if self.returns_value() {
            if let Some(name) = &self.name {
                let ident = Ident::new(name, Span::call_site());
                syn::parse_quote! {
                    let #ident = #owner.#method_ident(#(#args),*);
                }
            } else {
                panic!("Name must have been set before")
            }
        } else {
            syn::parse_quote! {
                #owner.#method_ident(#(#args),*);
            }
        }
    }

    pub fn returns_value(&self) -> bool {
        if let ReturnType::Default = &self.method.sig.output {
            false
        } else {
            true
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
    pub fn args(&self) -> &Vec<Arg> {
        &self.args
    }

    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn set_owner(&mut self, owner: String) {
        self.owner = owner;
    }
    pub fn name(&self) -> &Option<String> {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_owned());
    }
}

#[derive(Debug)]
pub struct FnInvStmt {
    id: Uuid,
    params: Vec<FnArg>,
    args: Vec<Arg>,
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
    pub fn new(ident: Ident, item_fn: ItemFn, params: Vec<FnArg>, args: Vec<Arg>) -> Self {
        FnInvStmt {
            params,
            args,
            ident,
            item_fn,
            id: Uuid::new_v4(),
        }
    }

    pub fn params(&self) -> &Vec<FnArg> {
        &self.params
    }
    pub fn args(&self) -> &Vec<Arg> {
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

    pub fn set_args(&mut self, args: Vec<Arg>) {
        self.args = args;
    }

    pub fn to_syn(&self) -> Stmt {
        let ident = &self.ident;
        let args: Vec<Expr> = self.args.iter().cloned().map(|a| a.value).collect();

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
        let unconsumed_defs = test_case.unconsumed_complex_definitions();
        if unconsumed_defs.is_empty() {
            let structs = self.source_file.structs();
            let i = fastrand::usize(0..structs.len());

            let item_struct = structs.get(i).unwrap();
            if let Some(constructor) = item_struct.constructor() {
                let params: Vec<FnArg> = constructor.sig.inputs.iter().cloned().collect();
                let args: Vec<Arg> = params
                    .iter()
                    .map(PrimitivesGenerator::generate_arg)
                    .collect();
                Statement::Constructor(ConstructorStmt::new(item_struct.clone(), params, args))
            } else {
                // No constructor, so initialize directly
                unimplemented!()
            }
        } else {
            let i = fastrand::usize(0..unconsumed_defs.len());
            let constructor = unconsumed_defs.get(i).unwrap();

            let methods = constructor.struct_item.methods();
            // TODO what if there are no methods

            let i = fastrand::usize(0..methods.len());
            let method = methods.get(i).unwrap();

            let params: Vec<FnArg> = method.sig.inputs.iter().cloned().collect();
            let args: Vec<Arg> = params
                .iter()
                .filter_map(|a| {
                    match a {
                        FnArg::Receiver(_) => None,
                        FnArg::Typed(arg_pat_type) => {
                            if PrimitivesGenerator::is_fn_arg_primitive(a) {
                                let arg = PrimitivesGenerator::generate_arg(a);
                                Some(arg)
                            } else {
                                match arg_pat_type.ty.as_ref() {
                                    Type::Path(type_path) => {
                                        // Create consumed argument
                                        Some(self.construct_arg(test_case, a, type_path))
                                    }
                                    Type::Reference(reference) => {
                                        // Create referenced argument
                                        match reference.elem.as_ref() {
                                            Type::Path(type_path) => {
                                                Some(self.construct_arg(test_case, a, type_path))
                                            }
                                            _ => unimplemented!(),
                                        }
                                    }
                                    _ => unimplemented!(),
                                }
                            }
                        }
                    }
                })
                .collect();

            let mut stmt = MethodInvStmt::new(
                constructor.name.as_ref().unwrap(),
                method.clone(),
                params,
                args,
            );

            if stmt.returns_value() {}

            Statement::MethodInvocation(stmt)
        }
    }

    fn construct_arg(&self, test_case: &mut TestCase, fn_arg: &FnArg, type_path: &TypePath) -> Arg {
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
            constructor_params,
            constructor_args,
        ));

        if let Some(var_name) = test_case.insert_stmt(0, stmt) {
            let var_ident = Ident::new(&var_name, Span::call_site());
            let var = syn::parse_quote! {#var_ident};
            Arg::new(Some(var_name), var, fn_arg.clone(), false)
        } else {
            panic!()
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

        let stmt = self.statement_generator.get_random_stmt(&mut test_case);

        let test_id = self.test_id.borrow_mut().next_id();
        test_case.set_id(test_id);
        test_case.insert_stmt(0, stmt);

        test_case
    }
}

fn return_type_name(return_type: &ReturnType) -> Option<String> {
    match return_type {
        ReturnType::Default => None,
        ReturnType::Type(_, data_type) => Some(type_name(data_type.as_ref())),
    }
}

fn type_name(data_type: &Type) -> String {
    match data_type {
        Type::Path(type_path) => {
            let path = &type_path.path;
            merge_path(&path)
        }
        Type::Reference(_) => {
            unimplemented!()
        }
        _ => {
            unimplemented!()
        }
    }
}

fn merge_path(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}
