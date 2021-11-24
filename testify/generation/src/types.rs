use std::collections::HashSet;
use std::collections::HashMap;
use crate::chromosome::{Arg, AssignStmt, ConstructorStmt, FieldAccessStmt, FnInvStmt, MethodInvStmt, Statement, StaticFnInvStmt, StructInitStmt};
use rustc_ast::{FloatTy, IntTy, UintTy};
use rustc_hir::def_id::DefId;
use rustc_hir::{BodyId, FnSig, HirId, PrimTy};
use rustc_middle::ty::TyCtxt;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::PathBuf;
use syn::Type;

lazy_static! {
    pub static ref STR_TRAITS: HashSet<Trait> = {
        let mut s = HashSet::new();
        s.insert(Trait::new("std::clone::Clone"));
        s.insert(Trait::new("std::cmp::Eq"));
        s.insert(Trait::new("std::cmp::PartialEq"));
        s.insert(Trait::new("std::hash::Hash"));
        s.insert(Trait::new("std::default::Default"));
        s
    };
    pub static ref UINT_TRAITS: HashSet<Trait> = {
        let mut s = HashSet::new();
        s.insert(Trait::new("std::marker::Copy"));
        s.insert(Trait::new("std::clone::Clone"));
        s.insert(Trait::new("std::hash::Hash"));
        s.insert(Trait::new("std::cmp::Ord"));
        s.insert(Trait::new("std::cmp::PartialOrd"));
        s.insert(Trait::new("std::cmp::Eq"));
        s.insert(Trait::new("std::cmp::PartialEq"));
        s.insert(Trait::new("std::default::Default"));
        s
    };
    pub static ref INT_TRAITS: HashSet<Trait> = {
        let mut s = HashSet::new();
        s.insert(Trait::new("std::marker::Copy"));
        s.insert(Trait::new("std::clone::Clone"));
        s.insert(Trait::new("std::hash::Hash"));
        s.insert(Trait::new("std::cmp::Ord"));
        s.insert(Trait::new("std::cmp::PartialOrd"));
        s.insert(Trait::new("std::cmp::Eq"));
        s.insert(Trait::new("std::cmp::PartialEq"));
        s.insert(Trait::new("std::default::Default"));
        s
    };
    pub static ref FLOAT_TRAITS: HashSet<Trait> = {
        let mut s = HashSet::new();
        s.insert(Trait::new("std::marker::Copy"));
        s.insert(Trait::new("std::clone::Clone"));
        s.insert(Trait::new("std::hash::Hash"));
        s.insert(Trait::new("std::cmp::Ord"));
        s.insert(Trait::new("std::cmp::PartialOrd"));
        s.insert(Trait::new("std::cmp::Eq"));
        s.insert(Trait::new("std::cmp::PartialEq"));
        s.insert(Trait::new("std::default::Default"));
        s
    };
    pub static ref BOOL_TRAITS: HashSet<Trait> = {
        let mut s = HashSet::new();
        s.insert(Trait::new("std::clone::Clone"));
        s.insert(Trait::new("std::marker::Copy"));
        s.insert(Trait::new("std::hash::Hash"));
        s.insert(Trait::new("std::cmp::Ord"));
        s.insert(Trait::new("std::cmp::PartialOrd"));
        s.insert(Trait::new("std::cmp::Eq"));
        s.insert(Trait::new("std::cmp::PartialEq"));
        s.insert(Trait::new("std::default::Default"));
        s
    };

    pub static ref TYPES: HashMap<T, HashSet<Trait>> = {
        let types = load_types().unwrap();

        let mut vec_traits = HashSet::new();
        vec_traits.insert(Trait::new("std::iter::IntoIterator"));
        vec_traits.insert(Trait::new("std::default::Default"));
        vec_traits.insert(Trait::new("std::cmp::Eq"));
        vec_traits.insert(Trait::new("std::cmp::PartialEq"));
        vec_traits.insert(Trait::new("std::cmp::PartialOrd"));
        vec_traits.insert(Trait::new("std::cmp::Ord"));

        let mut m = HashMap::new();
        for (ty, implementations) in types {
            m.insert(ty, implementations);
        }
        m
    };

    pub static ref STD_CALLABLES: Vec<Callable> = load_callables().unwrap();
}

static TYPE_PROVIDERS_DIR: &'static str = "/Users/tim/Documents/master-thesis/testify/providers/types";
static IMPLEMENTATIONS_DIR: &'static str = "/Users/tim/Documents/master-thesis/testify/providers/implementations";
static CALLABLES_DIR: &'static str = "/Users/tim/Documents/master-thesis/testify/providers/callables";

fn load_callables() -> std::io::Result<Vec<Callable>> {
    let callables = fs::read_dir(CALLABLES_DIR)?.map(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            panic!("Should not be a dir");
        }

        let callable_content = fs::read_to_string(path.as_path()).unwrap();
        let callables: Vec<Callable> = serde_json::from_str(&callable_content).unwrap();
        callables
    }).flatten().collect::<Vec<_>>();

    Ok(callables)
}

fn load_types() -> std::io::Result<Vec<(T, HashSet<Trait>)>> {
    let mut types = Vec::new();
    for entry in fs::read_dir(TYPE_PROVIDERS_DIR)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            panic!("Should not be a dir");
        }

        // Load type
        let type_content = fs::read_to_string(path.as_path())?;
        let ty = serde_json::from_str(&type_content)?;

        let type_file = path.file_name().unwrap();
        let implementations_path = PathBuf::from(IMPLEMENTATIONS_DIR).join(type_file);

        // Load implemented traits
        let implementations_content = fs::read_to_string(implementations_path)?;
        let implementations: HashSet<Trait> = serde_json::from_str(&implementations_content)?;


        types.push((ty, implementations));
    }

    Ok(types)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Callable {
    Method(MethodItem),
    StaticFunction(StaticFnItem),
    Function(FunctionItem),
    Constructor(ConstructorItem),
    Primitive(PrimitiveItem),
    FieldAccess(FieldAccessItem),
    StructInit(StructInitItem)
}

impl Callable {
    pub fn params(&self) -> &Vec<Param> {
        match self {
            Callable::Method(method_item) => &method_item.params,
            Callable::StaticFunction(fn_item) => &fn_item.params,
            Callable::Function(fn_item) => &fn_item.params,
            Callable::Constructor(constructor_item) => &constructor_item.params,
            Callable::Primitive(primitive_item) => primitive_item.params(),
            Callable::FieldAccess(_) => unimplemented!(),
            Callable::StructInit(struct_init_item) => struct_init_item.params(),
        }
    }

    pub fn params_mut(&mut self) -> &mut Vec<Param> {
        match self {
            Callable::Method(m) => &mut m.params,
            Callable::StaticFunction(f) => &mut f.params,
            Callable::Function(f) => &mut f.params,
            Callable::Constructor(c) => &mut c.params,
            Callable::Primitive(p) => &mut p.params,
            Callable::FieldAccess(_) => unimplemented!(),
            Callable::StructInit(s) => &mut s.fields,
        }
    }

    pub fn return_type(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => method_item.return_type.as_ref(),
            Callable::StaticFunction(fn_item) => fn_item.return_type.as_ref(),
            Callable::Function(fn_item) => fn_item.return_type.as_ref(),
            Callable::Constructor(constructor_item) => Some(&constructor_item.return_type()),
            Callable::Primitive(primitive_item) => Some(&primitive_item.ty),
            Callable::FieldAccess(field_access_item) => Some(&field_access_item.ty),
            Callable::StructInit(struct_init_item) => Some(struct_init_item.return_type())
        }
    }

    pub fn parent(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => Some(&method_item.parent),
            Callable::StaticFunction(fn_item) => Some(&fn_item.parent),
            Callable::Function(_) => None,
            Callable::Constructor(constructor) => Some(&constructor.parent),
            Callable::Primitive(_) => None,
            Callable::FieldAccess(field_access_item) => Some(&field_access_item.parent),
            Callable::StructInit(struct_init_item) => Some(struct_init_item.return_type())
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
            Callable::FieldAccess(field_access_item) => {
                Statement::FieldAccess(FieldAccessStmt::new(field_access_item.clone()))
            }
            Callable::StructInit(struct_init_item) => {
                Statement::StructInit(StructInitStmt::new(struct_init_item.clone(), args))
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Callable::Method(m) => m.name(),
            Callable::StaticFunction(f) => f.name(),
            Callable::Function(f) => f.name(),
            Callable::Constructor(c) => "new",
            Callable::Primitive(_) => unimplemented!(),
            Callable::FieldAccess(_) => unimplemented!(),
            Callable::StructInit(_) => unimplemented!()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveItem {
    pub ty: T,
    pub params: Vec<Param>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInitItem {
    pub fields: Vec<Param>,
    pub return_type: T,


    pub src_file_id: usize,
}

impl StructInitItem {
    pub fn new(src_file_id: usize, fields: Vec<Param>, return_type: T) -> Self {
        StructInitItem { fields, return_type, src_file_id }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.fields
    }

    pub fn return_type(&self) -> &T {
        &self.return_type
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodItem {
    pub params: Vec<Param>,
    pub return_type: Option<T>,
    pub parent: T,
    pub name: String,

    #[serde(skip)]
    pub fn_id: Option<HirId>,
    pub src_file_id: usize,
}

impl MethodItem {
    pub fn new(
        src_file_id: usize,
        params: Vec<Param>,
        return_type: Option<T>,
        parent: T,
        generics: Vec<T>,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();

        MethodItem {
            src_file_id,
            params,
            parent,
            return_type,
            name,
            fn_id: Some(fn_id),
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

    pub fn consumes_parent(&self) -> bool {
        !self.params.first().unwrap().by_reference()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionItem {
    pub params: Vec<Param>,
    pub return_type: Option<T>,
    #[serde(skip)]
    pub fn_id: Option<HirId>,
    pub name: String,
    pub src_file_id: usize,
}

impl FunctionItem {
    pub fn new(
        src_file_id: usize,
        params: Vec<Param>,
        return_type: Option<T>,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();

        FunctionItem {
            src_file_id,
            params,
            return_type,
            name,
            fn_id: Some(fn_id),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFnItem {
    pub params: Vec<Param>,
    pub return_type: Option<T>,
    pub parent: T,
    #[serde(skip)]
    pub fn_id: Option<HirId>,
    pub name: String,
    pub src_file_id: usize,
}

impl StaticFnItem {
    pub fn new(
        src_file_id: usize,
        params: Vec<Param>,
        return_type: Option<T>,
        parent: T,
        generics: Vec<T>,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();

        StaticFnItem {
            src_file_id,
            params,
            parent,
            return_type,
            fn_id: Some(fn_id),
            name,
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

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAccessItem {
    pub ty: T,
    #[serde(skip)]
    pub field_id: Option<HirId>,
    pub parent: T,
    pub name: String,
    pub src_file_id: usize,
}

impl FieldAccessItem {
    pub fn new(src_file_id: usize, ty: T, parent: T, field_id: HirId, tcx: &TyCtxt<'_>) -> Self {
        let ident = tcx.hir().get(field_id).ident().unwrap();
        let name = ident.name.to_string();

        FieldAccessItem {
            src_file_id,
            name,
            ty,
            parent,
            field_id: Some(field_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructorItem {
    pub params: Vec<Param>,
    pub parent: T,
    #[serde(skip)]
    pub fn_id: Option<HirId>,
    pub src_file_id: usize,
}

impl ConstructorItem {
    pub fn new(
        src_file_id: usize,
        fn_sig: &FnSig,
        fn_id: HirId,
        parent_hir_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        todo!()
        /*
        ConstructorItem {
            parent,
            params,
            body_id,
            fn_id
        }*/
    }

    pub fn params(&self) -> &Vec<Param> {
        self.params.as_ref()
    }
    pub fn return_type(&self) -> &T {
        &self.parent
    }
    pub fn parent(&self) -> &T {
        &self.parent
    }
}

#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
pub enum T {
    Prim(PrimT),
    Complex(ComplexT),
    Generic(Generic),
}

impl PartialEq for T {
    fn eq(&self, other: &Self) -> bool {
        match self {
            T::Prim(prim) => match other {
                T::Prim(other_prim) => prim == other_prim,
                T::Complex(_) => false,
                T::Generic(_) => false,
            },
            T::Complex(comp) => match other {
                T::Prim(_) => false,
                T::Complex(other_comp) => comp == other_comp,
                T::Generic(_) => false,
            },
            T::Generic(generic) => match other {
                T::Prim(_) => false,
                T::Complex(_) => false,
                T::Generic(other_generic) => generic == other_generic,
            },
        }
    }
}

impl From<PrimTy> for T {
    fn from(ty: PrimTy) -> Self {
        let ty = match ty {
            PrimTy::Int(int_ty) => {
                let int_ty = match int_ty {
                    IntTy::Isize => IntT::Isize,
                    IntTy::I8 => IntT::I8,
                    IntTy::I16 => IntT::I16,
                    IntTy::I32 => IntT::I32,
                    IntTy::I64 => IntT::I64,
                    IntTy::I128 => IntT::I128,
                };
                PrimT::Int(int_ty)
            }
            PrimTy::Uint(uint_ty) => {
                let uint_ty = match uint_ty {
                    UintTy::Usize => UintT::Usize,
                    UintTy::U8 => UintT::U8,
                    UintTy::U16 => UintT::U16,
                    UintTy::U32 => UintT::U32,
                    UintTy::U64 => UintT::U64,
                    UintTy::U128 => UintT::U128,
                };
                PrimT::Uint(uint_ty)
            }
            PrimTy::Float(float_ty) => {
                let float_ty = match float_ty {
                    FloatTy::F32 => FloatT::F32,
                    FloatTy::F64 => FloatT::F64,
                };
                PrimT::Float(float_ty)
            }
            PrimTy::Str => PrimT::Str,
            PrimTy::Bool => PrimT::Bool,
            PrimTy::Char => PrimT::Char,
        };
        T::Prim(ty)
    }
}

impl T {
    pub fn syn_type(&self) -> &Type {
        unimplemented!()
    }

    pub fn name(&self) -> String {
        match self {
            T::Prim(prim) => {
                // TODO only for debugging
                format!("{:?}", prim)
            }
            T::Complex(complex) => complex.name().to_string(),
            T::Generic(generic) => generic.name().to_string(),
        }
    }

    pub fn var_string(&self) -> String {
        match self {
            T::Prim(prim) => prim.name_str().to_string(),
            T::Complex(complex) => complex.name().split("::").last().unwrap().to_string(),
            T::Generic(generic) => todo!(),
        }
    }

    pub fn id(&self) -> Option<DefId> {
        match self {
            T::Prim(_) => unimplemented!(),
            T::Complex(complex) => complex.def_id(),
            T::Generic(generic) => unimplemented!(),
        }
    }

    pub fn expect_id(&self) -> DefId {
        match self {
            T::Prim(_) => unimplemented!(),
            T::Complex(complex) => complex.def_id().unwrap(),
            T::Generic(generic) => unimplemented!(),
        }
    }

    pub fn is_prim(&self) -> bool {
        match self {
            T::Prim(_) => true,
            T::Complex(_) => false,
            T::Generic(_) => false,
        }
    }

    pub fn is_complex(&self) -> bool {
        match self {
            T::Prim(_) => false,
            T::Complex(_) => true,
            T::Generic(_) => false,
        }
    }

    pub fn is_generic(&self) -> bool {
        match self {
            T::Prim(_) => false,
            T::Complex(_) => false,
            T::Generic(_) => true,
        }
    }

    pub fn expect_generic(&self) -> &Generic {
        match self {
            T::Generic(generic) => generic,
            _ => panic!("Is not generic"),
        }
    }

    pub fn expect_complex(&self) -> &ComplexT {
        match self {
            T::Complex(complex) => complex,
            _ => panic!("Is not complex"),
        }
    }

    pub fn expect_primitive(&self) -> &PrimT {
        match self {
            T::Prim(prim) => prim,
            _ => panic!()
        }
    }
}

impl Display for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            T::Prim(prim) => write!(f, "{}", prim.name_str()),
            T::Complex(complex) => Display::fmt(complex, f),
            T::Generic(generic) => Display::fmt(generic, f),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
pub struct ComplexT {
    #[serde(skip)]
    def_id: Option<DefId>,
    name: String,
    generics: Vec<T>,
}

impl Display for ComplexT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.generics.is_empty() {
            write!(f, "<")?;
            for (idx, gen) in self.generics.iter().enumerate() {
                if idx == self.generics.len() - 1 {
                    write!(f, "{}", gen)?;
                } else {
                    write!(f, "{}, ", gen)?;
                }
            }
            write!(f, ">")?;
        }

        Ok(())
    }
}

impl PartialEq for ComplexT {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl ComplexT {
    pub fn new(def_id: Option<DefId>, name: &str, generics: Vec<T>) -> Self {
        ComplexT {
            name: name.to_string(),
            def_id,
            generics,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn def_id(&self) -> Option<DefId> {
        self.def_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Generic {
    name: String,
    bounds: Vec<Trait>,
}

impl Generic {
    pub fn new(name: &str, bounds: Vec<Trait>) -> Self {
        Self { name: name.to_string(), bounds }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bounds(&self) -> &Vec<Trait> {
        &self.bounds
    }
}

impl Display for Generic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;
        if !self.bounds.is_empty() {
            write!(f, ": ")?;
            for (idx, bound) in self.bounds.iter().enumerate() {
                if idx == self.bounds.len() - 1 {
                    write!(f, "{} + ", bound)?;
                } else {
                    write!(f, "{}", bound)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Trait {
    name: String,
}

impl Trait {
    pub fn new(name: &str) -> Self {
        Trait {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for Trait {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    real_ty: T,
    original_ty: T,
    by_reference: bool,
    mutable: bool,
    name: String
}

impl Param {
    pub fn new(name: &str, real_ty: T, original_ty: T, by_reference: bool, mutable: bool) -> Self {
        Param {
            name: name.to_string(),
            real_ty,
            original_ty,
            by_reference,
            mutable,
        }
    }

    pub fn is_self(&self) -> bool {
        todo!()
    }

    pub fn by_reference(&self) -> bool {
        self.by_reference
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }

    pub fn real_ty(&self) -> &T {
        &self.real_ty
    }

    pub fn real_ty_mut(&mut self) -> &mut T {
        todo!()
    }

    pub fn is_primitive(&self) -> bool {
        self.real_ty.is_prim()
    }

    pub fn is_generic(&self) -> bool {
        self.real_ty.is_generic()
    }


    pub fn original_ty(&self) -> &T {
        &self.original_ty
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum PrimT {
    Int(IntT),
    Uint(UintT),
    Float(FloatT),
    Str,
    Bool,
    Char,
}

impl PrimT {
    pub fn name_str(self) -> &'static str {
        match self {
            PrimT::Int(i) => i.name_str(),
            PrimT::Uint(u) => u.name_str(),
            PrimT::Float(f) => f.name_str(),
            PrimT::Str => "str",
            PrimT::Bool => "bool",
            PrimT::Char => "char",
        }
    }

    pub fn implementors_of(trayt: &Trait) -> HashSet<PrimT> {
        let mut implementators = HashSet::new();

        if INT_TRAITS.contains(trayt) {
            vec![
                IntT::Isize,
                IntT::I8,
                IntT::I16,
                IntT::I32,
                IntT::I64,
                IntT::I128,
            ].iter().for_each(|i| {
                let _ = implementators.insert(PrimT::Int(*i));
            });
        }

        if UINT_TRAITS.contains(trayt) {
            vec![
                UintT::Usize,
                UintT::U8,
                UintT::U16,
                UintT::U32,
                UintT::U64,
                UintT::U128
            ].iter().for_each(|t| {
                let _ = implementators.insert(PrimT::Uint(*t));
            });
        }

        if FLOAT_TRAITS.contains(trayt) {
            implementators.insert(PrimT::Float(FloatT::F32));
            implementators.insert(PrimT::Float(FloatT::F64));
        }

        if BOOL_TRAITS.contains(trayt) {
            implementators.insert(PrimT::Bool);
        }

        if STR_TRAITS.contains(trayt) {
            implementators.insert(PrimT::Str);
        }

        implementators
    }

    pub fn implements(&self, trayt: &Trait) -> bool {
        match self {
            PrimT::Int(_) => {
                INT_TRAITS.contains(trayt)
            }
            PrimT::Uint(_) => {
                UINT_TRAITS.contains(trayt)
            }
            PrimT::Float(_) => {
                FLOAT_TRAITS.contains(trayt)
            }
            PrimT::Str => {
                STR_TRAITS.contains(trayt)
            }
            PrimT::Bool => {
                BOOL_TRAITS.contains(trayt)
            }
            PrimT::Char => {
                todo!()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum IntT {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128
}

impl IntT {
    pub fn name_str(&self) -> &'static str {
        match *self {
            IntT::Isize => "isize",
            IntT::I8 => "i8",
            IntT::I16 => "i16",
            IntT::I32 => "i32",
            IntT::I64 => "i64",
            IntT::I128 => "i128",
        }
    }

}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum UintT {
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl UintT {
    pub fn name_str(&self) -> &'static str {
        match *self {
            UintT::Usize => "usize",
            UintT::U8 => "u8",
            UintT::U16 => "u16",
            UintT::U32 => "u32",
            UintT::U64 => "u64",
            UintT::U128 => "u128",
        }
    }

}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum FloatT {
    F32,
    F64,
}

impl FloatT {
    pub fn name_str(self) -> &'static str {
        match self {
            FloatT::F32 => "f32",
            FloatT::F64 => "f64",
        }
    }
}
