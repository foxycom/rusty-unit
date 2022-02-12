use proc_macro2::{Ident, Span};
use rustc_ast::{FloatTy, IntTy, UintTy};
use rustc_hir::def_id::DefId;
use rustc_hir::{BodyId, FnSig, HirId, PrimTy};
use rustc_middle::ty::{TyCtxt, TypeFoldable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use syn::{Expr, Type};
use uuid::Uuid;

lazy_static! {
    /*pub static ref TYPES: HashMap<Arc<T>, HashSet<Trait>> = {
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
    };*/
}

static TYPE_PROVIDERS_DIR: &'static str =
    "/Users/tim/Documents/master-thesis/testify/providers/types";
static IMPLEMENTATIONS_DIR: &'static str =
    "/Users/tim/Documents/master-thesis/testify/providers/implementations";
static CALLABLES_DIR: &'static str =
    "/Users/tim/Documents/master-thesis/testify/providers/callables";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Callable {
    Method(MethodItem),
    StaticFunction(StaticFnItem),
    Function(FunctionItem),
    Primitive(PrimitiveItem),
    FieldAccess(FieldAccessItem),
    StructInit(StructInitItem),
    EnumInit(EnumInitItem),
}

impl Callable {
    pub fn return_type(&self) -> Option<&Arc<T>> {
        match self {
            Callable::Method(method_item) => method_item.return_type.as_ref(),
            Callable::StaticFunction(fn_item) => fn_item.return_type.as_ref(),
            Callable::Function(fn_item) => fn_item.return_type.as_ref(),
            Callable::Primitive(primitive_item) => Some(&primitive_item.ty),
            Callable::FieldAccess(field_access_item) => Some(&field_access_item.ty),
            Callable::StructInit(struct_init_item) => Some(struct_init_item.return_type()),
            Callable::EnumInit(enum_init_item) => Some(enum_init_item.return_type()),
        }
    }

    pub fn parent(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => Some(&method_item.parent),
            Callable::StaticFunction(fn_item) => Some(&fn_item.parent),
            Callable::Function(_) => None,
            Callable::Primitive(_) => None,
            Callable::FieldAccess(field_access_item) => Some(&field_access_item.parent),
            Callable::StructInit(struct_init_item) => Some(struct_init_item.return_type()),
            Callable::EnumInit(enum_init_item) => Some(enum_init_item.return_type()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Callable::Method(m) => m.name(),
            Callable::StaticFunction(f) => f.name(),
            Callable::Function(f) => f.name(),
            Callable::Primitive(_) => unimplemented!(),
            Callable::FieldAccess(_) => unimplemented!(),
            Callable::StructInit(_) => unimplemented!(),
            Callable::EnumInit(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveItem {
    pub ty: Arc<T>,
    pub params: Vec<Param>,
}

impl PrimitiveItem {
    pub fn new(ty: Arc<T>) -> PrimitiveItem {
        PrimitiveItem { ty, params: vec![] }
    }

    pub fn params(&self) -> &Vec<Param> {
        // Just for compilation reasons
        &self.params
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumInitItem {
    pub return_type: Arc<T>,
    pub src_file_path: String,
    pub variant: EnumVariant,
    pub is_public: bool,
}

impl EnumInitItem {
    pub fn new(
        src_file_path: &str,
        variant: EnumVariant,
        return_type: Arc<T>,
        is_public: bool,
    ) -> Self {
        Self {
            src_file_path: src_file_path.to_string(),
            variant,
            return_type,
            is_public,
        }
    }

    pub fn variant(&self) -> &EnumVariant {
        &self.variant
    }

    pub fn return_type(&self) -> &Arc<T> {
        &self.return_type
    }

    pub fn params(&self) -> Vec<Param> {
        self.variant.params()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInitItem {
    pub params: Vec<Param>,
    pub return_type: Arc<T>,
    pub src_file_path: String,
}

impl StructInitItem {
    pub fn new(src_file_path: &str, fields: Vec<Param>, return_type: Arc<T>) -> Self {
        StructInitItem {
            params: fields,
            return_type,
            src_file_path: src_file_path.to_string(),
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }

    pub fn return_type(&self) -> &Arc<T> {
        &self.return_type
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodItem {
    pub params: Vec<Param>,
    pub return_type: Option<Arc<T>>,
    pub parent: Arc<T>,
    pub name: String,

    #[serde(skip)]
    pub fn_id: Option<HirId>,
    pub src_file_path: String,
    pub is_public: bool,
}

impl MethodItem {
    pub fn new(
        src_file_path: &str,
        params: Vec<Param>,
        return_type: Option<Arc<T>>,
        parent: Arc<T>,
        generics: Vec<Arc<T>>,
        is_public: bool,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();

        MethodItem {
            src_file_path: src_file_path.to_string(),
            params,
            parent,
            return_type,
            name,
            fn_id: Some(fn_id),
            is_public,
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
    pub fn return_type(&self) -> Option<Arc<T>> {
        self.return_type.clone()
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
    pub return_type: Option<Arc<T>>,
    #[serde(skip)]
    pub fn_id: Option<HirId>,
    pub name: String,
    pub src_file_path: String,
    pub is_public: bool,
}

impl FunctionItem {
    pub fn new(
        src_file_path: &str,
        params: Vec<Param>,
        return_type: Option<Arc<T>>,
        is_public: bool,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();

        FunctionItem {
            src_file_path: src_file_path.to_string(),
            params,
            return_type,
            name,
            is_public,
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
    pub return_type: Option<Arc<T>>,
    pub parent: Arc<T>,
    #[serde(skip)]
    pub fn_id: Option<HirId>,
    pub name: String,
    pub src_file_path: String,
    pub is_public: bool,
}

impl StaticFnItem {
    pub fn new(
        src_file_path: &str,
        params: Vec<Param>,
        return_type: Option<Arc<T>>,
        parent: Arc<T>,
        generics: Vec<Arc<T>>,
        is_public: bool,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();

        StaticFnItem {
            src_file_path: src_file_path.to_string(),
            params,
            parent,
            return_type,
            fn_id: Some(fn_id),
            name,
            is_public,
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
    pub fn return_type(&self) -> Option<&Arc<T>> {
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
    pub ty: Arc<T>,
    #[serde(skip)]
    pub field_id: Option<HirId>,
    pub parent: Arc<T>,
    pub name: String,
    pub src_file_path: String,
    pub is_public: bool,
}

impl FieldAccessItem {
    pub fn new(
        src_file_path: &str,
        ty: Arc<T>,
        parent: Arc<T>,
        is_public: bool,
        field_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(field_id).ident().unwrap();
        let name = ident.name.to_string();

        FieldAccessItem {
            src_file_path: src_file_path.to_string(),
            name,
            ty,
            parent,
            is_public,
            field_id: Some(field_id),
        }
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub enum T {
    Ref(Arc<T>),
    Prim(PrimT),
    Complex(ComplexT),
    Generic(Generic),
    Enum(EnumT),
    Array(Box<ArrayT>),
}

impl Debug for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            T::Ref(ty) => {
                write!(f, "&");
                Debug::fmt(ty, f)
            }
            T::Prim(prim_ty) => Debug::fmt(prim_ty, f),
            T::Complex(complex_ty) => Debug::fmt(complex_ty, f),
            T::Generic(generic_ty) => Debug::fmt(generic_ty, f),
            T::Enum(enum_ty) => Debug::fmt(enum_ty, f),
            T::Array(array_ty) => Debug::fmt(array_ty, f),
        }
    }
}

impl PartialEq for T {
    fn eq(&self, other: &Self) -> bool {
        match self {
            T::Prim(prim) => match other {
                T::Prim(other_prim) => prim == other_prim,
                _ => false,
            },
            T::Complex(comp) => match other {
                T::Complex(other_comp) => comp == other_comp,
                _ => false,
            },
            T::Generic(generic) => match other {
                T::Generic(other_generic) => generic == other_generic,
                _ => false,
            },
            T::Ref(r) => match other {
                T::Ref(other_r) => r == other_r,
                _ => false,
            },
            T::Enum(enum_ty) => match other {
                T::Enum(other_enum) => enum_ty == other_enum,
                _ => false,
            },
            T::Array(array_ty) => match other {
                T::Array(other_array) => array_ty == other_array,
                _ => false,
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
            T::Prim(prim) => prim.name_str().to_string(),
            T::Complex(complex) => complex.name().to_string(),
            T::Generic(generic) => generic.name().to_string(),
            T::Ref(r) => r.name(),
            T::Enum(enum_ty) => enum_ty.name().to_owned(),
            T::Array(_) => String::from("array"),
        }
    }

    pub fn full_name(&self) -> String {
        match self {
            T::Prim(prim) => prim.name_str().to_string(),
            T::Complex(complex) => complex.full_name(),
            T::Generic(generic) => generic.name().to_string(),
            T::Ref(r) => r.full_name(),
            T::Enum(enum_ty) => enum_ty.full_name(),
            T::Array(_) => String::from("array"),
        }
    }

    pub fn var_string(&self) -> String {
        match self {
            T::Prim(prim) => prim.name_str().to_string(),
            T::Complex(complex) => complex.name().split("::").last().unwrap().to_string(),
            T::Generic(generic) => todo!(),
            T::Ref(r) => r.var_string(),
            T::Enum(enum_ty) => enum_ty.name().split("::").last().unwrap().to_string(),
            T::Array(_) => String::from("array"),
        }
    }

    pub fn id(&self) -> Option<DefId> {
        match self {
            T::Prim(_) => unimplemented!(),
            T::Complex(complex) => complex.def_id(),
            T::Generic(generic) => unimplemented!(),
            T::Ref(r) => r.id(),
            T::Enum(enum_ty) => enum_ty.def_id(),
            T::Array(array_ty) => array_ty.def_id(),
        }
    }

    pub fn expect_id(&self) -> DefId {
        match self {
            T::Prim(_) => unimplemented!(),
            T::Complex(complex) => complex.def_id().unwrap(),
            T::Generic(generic) => unimplemented!(),
            T::Ref(r) => r.expect_id(),
            T::Enum(enum_ty) => enum_ty.def_id().unwrap(),
            T::Array(array_ty) => array_ty.def_id.unwrap(),
        }
    }

    pub fn is_prim(&self) -> bool {
        match self {
            T::Prim(_) => true,
            T::Ref(r) => r.is_prim(),
            _ => false,
        }
    }

    pub fn is_complex(&self) -> bool {
        match self {
            T::Complex(_) => true,
            T::Ref(r) => r.is_complex(),
            _ => false,
        }
    }

    pub fn is_enum(&self) -> bool {
        match self {
            T::Enum(_) => true,
            T::Ref(r) => r.is_enum(),
            _ => false,
        }
    }

    pub fn is_generic(&self) -> bool {
        match self {
            T::Generic(_) => true,
            T::Ref(r) => r.is_generic(),
            _ => false,
        }
    }

    pub fn expect_generic(&self) -> &Generic {
        match self {
            T::Generic(generic) => generic,
            T::Ref(r) => r.expect_generic(),
            _ => panic!("Is not generic"),
        }
    }

    pub fn expect_complex(&self) -> &ComplexT {
        match self {
            T::Complex(complex) => complex,
            T::Ref(r) => r.expect_complex(),
            _ => panic!("Is not complex"),
        }
    }

    pub fn expect_primitive(&self) -> &PrimT {
        match self {
            T::Prim(prim) => prim,
            T::Ref(r) => r.expect_primitive(),
            _ => panic!(),
        }
    }

    pub fn expect_enum(&self) -> &EnumT {
        match self {
            T::Ref(r) => r.expect_enum(),
            T::Enum(enum_ty) => enum_ty,
            _ => panic!("Is not enum"),
        }
    }

    pub fn generics(&self) -> Option<&Vec<Arc<T>>> {
        match self {
            T::Prim(_) => None,
            T::Complex(complex) => Some(complex.generics()),
            T::Generic(generic) => None,
            T::Ref(r) => r.generics(),
            T::Enum(enum_ty) => todo!(),
            T::Array(_) => None,
        }
    }

    pub fn to_ident(&self) -> Expr {
        let name = self.full_name();

        let ident = name
            .split("::")
            .map(|segment| Ident::new(segment, Span::call_site()))
            .collect::<Vec<_>>();
        let expr = syn::parse_quote! {
            #(#ident)::*
        };

        return match self {
            T::Ref(_) => syn::parse_quote! {
                &#expr
            },
            _ => expr,
        };
    }
}

impl Display for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            T::Prim(prim) => write!(f, "{}", prim.name_str()),
            T::Complex(complex) => Display::fmt(complex, f),
            T::Generic(generic) => Display::fmt(generic, f),
            T::Ref(r) => {
                write!(f, "&");
                Display::fmt(r.as_ref(), f)
            }
            T::Enum(enum_ty) => Display::fmt(enum_ty, f),
            T::Array(array) => Display::fmt(array, f),
        }
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct ArrayT {
    #[serde(skip)]
    def_id: Option<DefId>,
    length: usize,
    ty: T,
}

impl Debug for ArrayT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for ArrayT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}; {}]", self.ty, self.length)
    }
}

impl ArrayT {
    pub fn new(def_id: Option<DefId>, ty: T, length: usize) -> Self {
        Self { def_id, ty, length }
    }

    pub fn def_id(&self) -> Option<DefId> {
        self.def_id
    }
}

impl PartialEq for ArrayT {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty && self.length == other.length
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct ComplexT {
    #[serde(skip)]
    def_id: Option<DefId>,
    name: String,
    generics: Vec<Arc<T>>,
    is_local: bool,
}

impl Debug for ComplexT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
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
        self.name == other.name && self.generics == other.generics
    }
}

impl ComplexT {
    pub fn new(def_id: Option<DefId>, name: &str, generics: Vec<Arc<T>>) -> Self {
        let is_local = if let Some(def_id) = def_id {
            def_id.is_local()
        } else {
            false
        };

        ComplexT {
            name: name.to_string(),
            is_local,
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

    pub fn generics(&self) -> &Vec<Arc<T>> {
        &self.generics
    }

    pub fn bind_generics(&mut self, types: Vec<Arc<T>>) {
        self.generics = types;
    }

    pub fn full_name(&self) -> String {
        if self.is_local {
            format!("crate::{}", &self.name)
        } else {
            self.name.to_string()
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
pub struct EnumT {
    #[serde(skip)]
    def_id: Option<DefId>,
    name: String,
    generics: Vec<Arc<T>>,
    variants: Vec<EnumVariant>,
    is_local: bool,
}

impl PartialEq for EnumT {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        self.variants == other.variants
    }
}

impl Display for EnumT {
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

impl EnumT {
    pub fn new(
        def_id: Option<DefId>,
        name: &str,
        generics: Vec<Arc<T>>,
        variants: Vec<EnumVariant>,
    ) -> Self {
        let is_local = if let Some(def_id) = def_id {
            def_id.is_local()
        } else {
            false
        };

        Self {
            def_id,
            name: name.to_string(),
            variants,
            generics,
            is_local,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn full_name(&self) -> String {
        if self.is_local {
            format!("crate::{}", &self.name)
        } else {
            self.name.to_string()
        }
    }

    pub fn def_id(&self) -> Option<DefId> {
        self.def_id
    }
}

#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
pub enum EnumVariant {
    Struct(String, Param),
    Tuple(String, Vec<Param>),
    Unit(String),
}

impl EnumVariant {
    pub fn name(&self) -> &str {
        match self {
            EnumVariant::Struct(name, _) => name,
            EnumVariant::Tuple(name, _) => name,
            EnumVariant::Unit(name) => name
        }
    }

    pub fn params(&self) -> Vec<Param> {
        match self {
            EnumVariant::Struct(_, p) => vec![p.clone()],
            EnumVariant::Tuple(_, p) => p.clone(),
            EnumVariant::Unit(_) => vec![]
        }
    }
}

impl PartialEq for EnumVariant {
    fn eq(&self, other: &Self) -> bool {
        match self {
            EnumVariant::Struct(name, s) => match other {
                EnumVariant::Struct(other_name, o) => name == other_name && s == o,
                _ => false,
            },
            EnumVariant::Tuple(name, p) => match other {
                EnumVariant::Tuple(other_name, o) => name == other_name && p == o,
                _ => false,
            },
            EnumVariant::Unit(name) => match other {
                EnumVariant::Unit(other_name) => name == other_name,
                _ => false,
            },
        }
    }
}


#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Generic {
    scope: Uuid,
    name: String,
    bounds: Vec<Trait>,
}

impl Debug for Generic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Generic {
    pub fn new(name: &str, bounds: Vec<Trait>) -> Self {
        Self {
            scope: Uuid::new_v4(),
            name: name.to_string(),
            bounds,
        }
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

#[derive(Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Param {
    ty: Arc<T>,
    mutable: bool,
    name: Option<String>,
}

impl PartialEq for Param {
    fn eq(&self, other: &Self) -> bool {
        self.mutable == other.mutable && self.ty == other.ty && self.name == other.name
    }
}

impl Debug for Param {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}: ", name);
        }

        if self.mutable {
            write!(f, "mut ");
        }
        write!(f, "{:?}", &self.ty)
    }
}

impl Param {
    pub fn new(name: Option<&str>, ty: Arc<T>, mutable: bool) -> Self {
        Param {
            name: name.map(|s| s.to_string()),
            ty,
            mutable,
        }
    }

    pub fn is_self(&self) -> bool {
        todo!()
    }

    pub fn by_reference(&self) -> bool {
        match self.ty.as_ref() {
            T::Ref(_) => true,
            _ => false,
        }
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }

    pub fn real_ty(&self) -> &Arc<T> {
        match self.ty.as_ref() {
            T::Ref(ty) => ty,
            _ => &self.ty,
        }
    }

    pub fn real_ty_mut(&mut self) -> &mut T {
        todo!()
    }

    pub fn is_primitive(&self) -> bool {
        self.ty.is_prim()
    }

    pub fn is_generic(&self) -> bool {
        self.ty.is_generic()
    }

    pub fn original_ty(&self) -> &T {
        &self.ty
    }
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum PrimT {
    Int(IntT),
    Uint(UintT),
    Float(FloatT),
    Str,
    Bool,
    Char,
}

impl Debug for PrimT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name_str())
    }
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum IntT {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
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
