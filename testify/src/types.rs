use rustc_ast::{FloatTy, IntTy, UintTy};
use rustc_hir::def_id::DefId;
use rustc_hir::{BodyId, FnSig, HirId, PrimTy};
use rustc_middle::ty::{AdtDef, AdtKind, Binder, GenericParamDefKind, PredicateKind, Ty, TyCtxt, TyKind, TypeFoldable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use log::{error, info};
use rustc_hir::def::CtorKind;
use rustc_target::abi::VariantIdx;
use uuid::Uuid;

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
    pub fn return_type(&self) -> Option<&T> {
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
pub struct EnumInitItem {
    pub return_type: T,
    pub src_file_path: String,
    pub variant: EnumVariant,
    pub is_public: bool,
}

impl EnumInitItem {
    pub fn new(
        src_file_path: &str,
        variant: EnumVariant,
        return_type: T,
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

    pub fn return_type(&self) -> &T {
        &self.return_type
    }

    pub fn params(&self) -> Vec<Param> {
        self.variant.params()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInitItem {
    pub params: Vec<Param>,
    pub return_type: T,
    pub src_file_path: String,
}

impl StructInitItem {
    pub fn new(src_file_path: &str, fields: Vec<Param>, return_type: T) -> Self {
        StructInitItem {
            params: fields,
            return_type,
            src_file_path: src_file_path.to_string(),
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
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
    pub src_file_path: String,
    pub is_public: bool,
    generics: Vec<T>
}

impl MethodItem {
    pub fn new(
        name: &str,
        src_file_path: &str,
        params: Vec<Param>,
        return_type: Option<T>,
        parent: T,
        generics: Vec<T>,
        is_public: bool
    ) -> Self {
        MethodItem {
            is_public,
            parent,
            name: name.to_string(),
            generics,
            params,
            return_type,
            src_file_path: src_file_path.to_string(),
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
    pub fn return_type(&self) -> Option<T> {
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
    pub return_type: Option<T>,
    pub name: String,
    pub src_file_path: String,
    pub is_public: bool,
    pub generics: Vec<T>
}

impl FunctionItem {
    pub fn new(
        is_public: bool,
        name: &str,
        generics: Vec<T>,
        params: Vec<Param>,
        return_type: Option<T>,
        src_file_path: &str,
    ) -> Self {
        FunctionItem {
            name: name.to_string(),
            src_file_path: src_file_path.to_string(),
            params,
            return_type,
            is_public,
            generics
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticFnItem {
    pub is_public: bool,
    pub parent: T,
    pub name: String,
    pub generics: Vec<T>,
    pub params: Vec<Param>,
    pub return_type: Option<T>,
    pub src_file_path: String,
}

impl StaticFnItem {
    pub fn new(
        name: &str,
        src_file_path: &str,
        params: Vec<Param>,
        return_type: Option<T>,
        parent: T,
        generics: Vec<T>,
        is_public: bool,
    ) -> Self {
        StaticFnItem {
            name: name.to_string(),
            src_file_path: src_file_path.to_string(),
            params,
            parent,
            return_type,
            is_public,
            generics
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
    pub is_public: bool,
    pub parent: T,
    pub name: String,
    pub ty: T,
    pub src_file_path: String,
}

impl FieldAccessItem {
    pub fn new(
        name: &str,
        src_file_path: &str,
        ty: T,
        parent: T,
        is_public: bool,
    ) -> Self {
        FieldAccessItem {
            name: name.to_string(),
            src_file_path: src_file_path.to_string(),
            ty,
            parent,
            is_public,
        }
    }
}


#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub enum T {
    Ref(Box<T>, bool),
    Prim(PrimT),
    Struct(StructT),
    Generic(Generic),
    Enum(EnumT),
    Array(Box<ArrayT>),
    Tuple(TupleT),
}

fn trait_def_to_trait(def_id: DefId, tcx: &TyCtxt<'_>) -> Trait {
    assert!(tcx.is_trait(def_id));

    let bounding_trait_def = tcx.trait_def(def_id);
    let trait_name = def_id_name(bounding_trait_def.def_id, tcx);
    Trait::new(&trait_name)
}

pub fn mir_ty_to_t(ty: rustc_middle::ty::Ty<'_>, tcx: &TyCtxt<'_>) -> T {
    match ty.kind() {
        TyKind::Adt(adt_def, subst_ref) => {
            adt_def_to_t(adt_def, tcx)
        }
        TyKind::Param(param) => {
            T::Generic(Generic::new(param.name.as_str(), vec![]))
        }

        _ => todo!("{:?}", ty.kind())
    }
}

pub fn def_id_name(def_id: DefId, tcx: &TyCtxt<'_>) -> String {
    tcx.def_path_str(def_id)
}

pub fn ty_name<'tcx>(ty: rustc_middle::ty::Ty<'tcx>) -> &'tcx str {
    match ty.kind() {
        TyKind::Param(param) => param.name.as_str(),
        _ => todo!("{:?}", ty.kind())
    }
}

/// Converts an AdtDef to a T
fn adt_def_to_t(adt_def: &AdtDef, tcx: &TyCtxt<'_>) -> T {
    match adt_def.adt_kind() {
        AdtKind::Struct => {
            // Has only one variant
            assert_eq!(adt_def.variants.len(), 1);
            let variant = adt_def.variants.get(VariantIdx::from_usize(0)).unwrap();
            let name = def_id_name(adt_def.did, tcx);
            let generics = generics_of_item(adt_def.did, tcx);
            T::Struct(StructT::new(&name, generics))
        }
        AdtKind::Union => {
            todo!("Adt def is union")
        }
        AdtKind::Enum => {
            let enum_name = def_id_name(adt_def.did, tcx);
            let mut variants = Vec::with_capacity(adt_def.variants.len());
            for variant in &adt_def.variants {
                let variant_name = variant.name.to_string();

                let params = variant.fields.iter().map(|f| {
                    let field_name = f.name.to_string();
                    let field_t = mir_ty_to_t(tcx.type_of(f.did), tcx);
                    Param::new(Some(&field_name), field_t, false)
                }).collect::<Vec<_>>();

                match variant.ctor_kind {
                    CtorKind::Fn => {
                        variants.push(EnumVariant::Tuple(variant_name, params));
                    }
                    _ => todo!("{:?}", variant.ctor_kind)
                }
            }

            let generics = generics_of_item(adt_def.did, tcx);

            let t = T::Enum(EnumT::new(&enum_name, generics, variants));
            info!("Extracted enum: {:?}", t);
            t
        }
    }
}

/// Converts the generics of an item, e.g., struct or method, to Ts
pub fn generics_of_item(def_id: DefId, tcx: &TyCtxt<'_>) -> Vec<T> {
    let adt_predicates = tcx.predicates_defined_on(def_id);

    let mut generics: Vec<Generic> = Vec::new();
    adt_predicates.predicates.iter().filter_map(|(predicate, _)| {

        let binder = predicate.kind();

        match binder.skip_binder() {
            PredicateKind::Trait(trait_predicate) => {
                let self_ty = trait_predicate.self_ty();
                let name = ty_name(self_ty);
                let trait_ = trait_def_to_trait(trait_predicate.def_id(), tcx);
                let g = Generic::new(name, vec![trait_]);

                Some(g)
            }
            PredicateKind::RegionOutlives(_) => None,
            PredicateKind::TypeOutlives(_) => None,
            _ => todo!("{:?}", binder.skip_binder())
        }
    }).for_each(|mut g| {
        // The compiler splits things like T: Copy + Debug into
        // two separate predicates with same name, i.e., T, so we merge them back again
        let existing_generic = generics.iter_mut().find(|e| e.name == g.name);
        if let Some(generics) = existing_generic {
            generics.bounds.append(&mut g.bounds);
        } else {
            generics.push(g);
        }
    });


    generics.iter().map(|g| T::Generic(g.clone())).collect()
}

/// Returns the bounds of a generic item
fn item_bounds(def_id: DefId, tcx: &TyCtxt<'_>) -> Vec<Trait> {
    let item_bounds = tcx.item_bounds(def_id);
    todo!("Item bounds are: {:?}", item_bounds)
}

impl Debug for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            T::Ref(ty, _) => {
                write!(f, "&");
                Debug::fmt(ty, f)
            }
            T::Prim(prim_ty) => Debug::fmt(prim_ty, f),
            T::Struct(complex_ty) => Debug::fmt(complex_ty, f),
            T::Generic(generic_ty) => Debug::fmt(generic_ty, f),
            T::Enum(enum_ty) => Debug::fmt(enum_ty, f),
            T::Array(array_ty) => Debug::fmt(array_ty, f),
            T::Tuple(types) => Debug::fmt(types, f)
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
            T::Struct(comp) => match other {
                T::Struct(other_comp) => comp == other_comp,
                _ => false,
            },
            T::Generic(generic) => match other {
                T::Generic(other_generic) => generic == other_generic,
                _ => false,
            },
            T::Ref(r, _) => match other {
                T::Ref(other_r, _) => r == other_r,
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
            T::Tuple(types) => match other {
                T::Tuple(other_types) => types == other_types,
                _ => false
            }
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
    pub fn name(&self) -> String {
        match self {
            T::Prim(prim) => prim.name_str().to_string(),
            T::Struct(complex) => complex.name().to_string(),
            T::Generic(generic) => generic.name().to_string(),
            T::Ref(r, _) => r.name(),
            T::Enum(enum_ty) => enum_ty.name().to_owned(),
            T::Array(_) => String::from("array"),
            T::Tuple(_) => String::from("tuple")
        }
    }

    pub fn is_prim(&self) -> bool {
        match self {
            T::Prim(_) => true,
            T::Ref(r, _) => r.is_prim(),
            _ => false,
        }
    }

    pub fn is_generic(&self) -> bool {
        match self {
            T::Generic(_) => true,
            T::Ref(r, _) => r.is_generic(),
            _ => false,
        }
    }

    pub fn expect_generic(&self) -> &Generic {
        match self {
            T::Generic(generic) => generic,
            T::Ref(r, _) => r.expect_generic(),
            _ => panic!("Is not generic"),
        }
    }

    pub fn overwrite_generics(&mut self, generics: Vec<T>) {
        match self {
            T::Ref(r, _) => r.as_mut().overwrite_generics(generics),
            T::Struct(s) => s.overwrite_generics(generics),
            T::Enum(e) => e.overwrite_generics(generics),
            _ => todo!()
        }
    }
}

impl Display for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            T::Prim(prim) => write!(f, "{}", prim.name_str()),
            T::Struct(complex) => Display::fmt(complex, f),
            T::Generic(generic) => Display::fmt(generic, f),
            T::Ref(r, _) => {
                write!(f, "&");
                Display::fmt(r.as_ref(), f)
            }
            T::Enum(enum_ty) => Display::fmt(enum_ty, f),
            T::Array(array) => Display::fmt(array, f),
            T::Tuple(types) => {
                let results = types.types.iter().map(|t| Display::fmt(t.as_ref(), f)).collect::<Vec<_>>();
                if let Some(result) = results.last() {
                    *result
                } else {
                    Ok(())
                }
            }
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
pub struct TupleT {
    types: Vec<Box<T>>,
}

impl Debug for TupleT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for TupleT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let types = self.types.iter().map(|ty| format!("{}", ty)).collect::<Vec<_>>();
        write!(f, "({})", types.join(", "))
    }
}

impl PartialEq for TupleT {
    fn eq(&self, other: &Self) -> bool {
        self.types == other.types
    }
}

impl TupleT {
    pub fn new(types: Vec<Box<T>>) -> Self {
        Self {
            types
        }
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct StructT {
    name: String,
    generics: Vec<T>,
    is_local: bool,
}

impl Debug for StructT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for StructT {
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

impl PartialEq for StructT {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.generics == other.generics
    }
}

impl StructT {
    pub fn new(name: &str, generics: Vec<T>) -> Self {
        StructT {
            name: name.to_string(),
            is_local: true,
            generics,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn generics(&self) -> &Vec<T> {
        &self.generics
    }

    pub fn bind_generics(&mut self, types: Vec<T>) {
        self.generics = types;
    }

    pub fn full_name(&self) -> String {
        if self.is_local {
            format!("crate::{}", &self.name)
        } else {
            self.name.to_string()
        }
    }

    pub fn overwrite_generics(&mut self, generics: Vec<T>) {
        self.generics = generics;
    }
}

#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
pub struct EnumT {
    name: String,
    generics: Vec<T>,
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
        name: &str,
        generics: Vec<T>,
        variants: Vec<EnumVariant>,
    ) -> Self {
        Self {
            name: name.to_string(),
            variants,
            generics,
            is_local: true,
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

    pub fn overwrite_generics(&mut self, generics: Vec<T>) {
        self.generics = generics;
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
            let bounds = self.bounds.iter().map(|b| format!("{}", b)).collect::<Vec<_>>().join(" + ");
            write!(f, "{}", bounds)?;
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
    ty: T,
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
    pub fn new(name: Option<&str>, ty: T, mutable: bool) -> Self {
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
        match &self.ty {
            T::Ref(_, _) => true,
            _ => false,
        }
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }

    pub fn real_ty(&self) -> &T {
        match &self.ty {
            T::Ref(ty, _) => ty.as_ref(),
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
