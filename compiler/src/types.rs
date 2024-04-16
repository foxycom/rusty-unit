use crate::util::is_local;
use log::{error, info};
use rustc_ast::{FloatTy, IntTy, UintTy};
use rustc_hir::def::CtorKind;
use rustc_hir::def_id::DefId;
use rustc_hir::{BodyId, FnSig, HirId, PrimTy};
use rustc_middle::ty::{
    AdtDef, AdtKind, Binder, FloatTy as MirFloat, GenericParamDefKind, IntTy as MirInt,
    PredicateKind, Ty, TyCtxt, TyKind, TypeFoldable,
};
use rustc_target::abi::VariantIdx;
use serde::{Deserialize, Serialize};
use std::collections::hash_set::Union;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuCallable {
    Method(RuMethod),
    StaticFunction(RuStaticMethod),
    Function(RuFunction),
    Primitive(RuPrimitive),
    FieldAccess(FieldAccessItem),
    StructInit(RuStructInit),
    EnumInit(RuEnumInit),
}

impl RuCallable {
    pub fn return_type(&self) -> Option<&RuTy> {
        match self {
            RuCallable::Method(method_item) => method_item.return_type.as_ref(),
            RuCallable::StaticFunction(fn_item) => fn_item.return_type.as_ref(),
            RuCallable::Function(fn_item) => fn_item.return_type.as_ref(),
            RuCallable::Primitive(primitive_item) => Some(&primitive_item.ty),
            RuCallable::FieldAccess(field_access_item) => Some(&field_access_item.ty),
            RuCallable::StructInit(struct_init_item) => Some(struct_init_item.return_type()),
            RuCallable::EnumInit(enum_init_item) => Some(enum_init_item.return_type()),
        }
    }

    pub fn parent(&self) -> Option<&RuTy> {
        match self {
            RuCallable::Method(method_item) => Some(&method_item.parent),
            RuCallable::StaticFunction(fn_item) => Some(&fn_item.parent),
            RuCallable::Function(_) => None,
            RuCallable::Primitive(_) => None,
            RuCallable::FieldAccess(field_access_item) => Some(&field_access_item.parent),
            RuCallable::StructInit(struct_init_item) => Some(struct_init_item.return_type()),
            RuCallable::EnumInit(enum_init_item) => Some(enum_init_item.return_type()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            RuCallable::Method(m) => m.name(),
            RuCallable::StaticFunction(f) => f.name(),
            RuCallable::Function(f) => f.name(),
            RuCallable::Primitive(_) => unimplemented!(),
            RuCallable::FieldAccess(_) => unimplemented!(),
            RuCallable::StructInit(_) => unimplemented!(),
            RuCallable::EnumInit(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuPrimitive {
    pub ty: RuTy,
    pub params: Vec<RuParam>,
}

impl RuPrimitive {
    pub fn new(ty: RuTy) -> RuPrimitive {
        RuPrimitive { ty, params: vec![] }
    }

    pub fn params(&self) -> &Vec<RuParam> {
        // Just for compilation reasons
        &self.params
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuEnumInit {
    pub return_type: RuTy,
    pub src_file_path: String,
    pub variant: RuEnumVariant,
    pub is_public: bool,
}

impl RuEnumInit {
    pub fn new(
        src_file_path: &str,
        variant: RuEnumVariant,
        return_type: RuTy,
        is_public: bool,
    ) -> Self {
        Self {
            src_file_path: src_file_path.to_string(),
            variant,
            return_type,
            is_public,
        }
    }

    pub fn variant(&self) -> &RuEnumVariant {
        &self.variant
    }

    pub fn return_type(&self) -> &RuTy {
        &self.return_type
    }

    pub fn params(&self) -> Vec<RuParam> {
        self.variant.params()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuStructInit {
    pub params: Vec<RuParam>,
    pub return_type: RuTy,
    pub src_file_path: String,
    pub is_public: bool,
}

impl RuStructInit {
    pub fn new(
        is_public: bool,
        src_file_path: &str,
        fields: Vec<RuParam>,
        return_type: RuTy,
    ) -> Self {
        RuStructInit {
            is_public,
            params: fields,
            return_type,
            src_file_path: src_file_path.to_string(),
        }
    }

    pub fn params(&self) -> &Vec<RuParam> {
        &self.params
    }

    pub fn return_type(&self) -> &RuTy {
        &self.return_type
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuMethod {
    pub params: Vec<RuParam>,
    pub return_type: Option<RuTy>,
    pub parent: RuTy,
    pub name: String,
    pub src_file_path: String,
    pub is_public: bool,
    pub of_trait: Option<String>,
    generics: Vec<RuTy>,
    pub global_id: String,
}

impl RuMethod {
    pub fn new(
        name: &str,
        src_file_path: &str,
        params: Vec<RuParam>,
        return_type: Option<RuTy>,
        parent: RuTy,
        generics: Vec<RuTy>,
        is_public: bool,
        of_trait: Option<String>,
        global_id: String,
    ) -> Self {
        RuMethod {
            is_public,
            parent,
            name: name.to_string(),
            generics,
            params,
            return_type,
            src_file_path: src_file_path.to_string(),
            of_trait,
            global_id,
        }
    }

    pub fn params(&self) -> &Vec<RuParam> {
        &self.params
    }
    pub fn return_type(&self) -> Option<RuTy> {
        self.return_type.clone()
    }
    pub fn parent(&self) -> &RuTy {
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
pub struct RuFunction {
    pub params: Vec<RuParam>,
    pub return_type: Option<RuTy>,
    pub name: String,
    pub src_file_path: String,
    pub is_public: bool,
    pub generics: Vec<RuTy>,
    pub global_id: String,
}

impl RuFunction {
    pub fn new(
        is_public: bool,
        name: &str,
        generics: Vec<RuTy>,
        params: Vec<RuParam>,
        return_type: Option<RuTy>,
        src_file_path: &str,
        global_id: String,
    ) -> Self {
        RuFunction {
            name: name.to_string(),
            src_file_path: src_file_path.to_string(),
            params,
            return_type,
            is_public,
            generics,
            global_id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuStaticMethod {
    pub is_public: bool,
    pub parent: RuTy,
    pub name: String,
    pub generics: Vec<RuTy>,
    pub params: Vec<RuParam>,
    pub return_type: Option<RuTy>,
    pub src_file_path: String,
    pub of_trait: Option<String>,
    pub global_id: String,
}

impl RuStaticMethod {
    pub fn new(
        name: &str,
        src_file_path: &str,
        params: Vec<RuParam>,
        return_type: Option<RuTy>,
        parent: RuTy,
        generics: Vec<RuTy>,
        is_public: bool,
        of_trait: Option<String>,
        global_id: String,
    ) -> Self {
        RuStaticMethod {
            name: name.to_string(),
            src_file_path: src_file_path.to_string(),
            params,
            parent,
            return_type,
            is_public,
            generics,
            of_trait,
            global_id,
        }
    }

    pub fn params(&self) -> &Vec<RuParam> {
        &self.params
    }
    pub fn return_type(&self) -> Option<&RuTy> {
        self.return_type.as_ref()
    }
    pub fn parent(&self) -> &RuTy {
        &self.parent
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAccessItem {
    pub is_public: bool,
    pub parent: RuTy,
    pub name: String,
    pub ty: RuTy,
    pub src_file_path: String,
}

impl FieldAccessItem {
    pub fn new(name: &str, src_file_path: &str, ty: RuTy, parent: RuTy, is_public: bool) -> Self {
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
pub enum RuTy {
    Ref(Box<RuTy>, bool),
    Prim(RuPrim),
    Struct(RuStruct),
    Generic(RuGeneric),
    Enum(RuEnum),
    Array(Box<RuArray>),
    Tuple(RuTuple),
    TraitObj(RuTraitObj),
    RawPointer(Box<RuTy>, bool),
    Union(RuUnion),
    Slice(Box<RuTy>),
    AsTrait(Box<RuTy>, RuTrait),
    Relative(Box<RuTy>, String),
    Fn,
}

impl PartialEq for RuTy {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RuTy::Prim(prim) => match other {
                RuTy::Prim(other_prim) => prim == other_prim,
                _ => false,
            },
            RuTy::Struct(comp) => match other {
                RuTy::Struct(other_comp) => comp == other_comp,
                _ => false,
            },
            RuTy::Generic(generic) => match other {
                RuTy::Generic(other_generic) => generic == other_generic,
                _ => false,
            },
            RuTy::Ref(r, _) => match other {
                RuTy::Ref(other_r, _) => r == other_r,
                _ => false,
            },
            RuTy::Enum(enum_ty) => match other {
                RuTy::Enum(other_enum) => enum_ty == other_enum,
                _ => false,
            },
            RuTy::Array(array_ty) => match other {
                RuTy::Array(other_array) => array_ty == other_array,
                _ => false,
            },
            RuTy::Tuple(types) => match other {
                RuTy::Tuple(other_types) => types == other_types,
                _ => false,
            },
            RuTy::TraitObj(trait_obj) => match other {
                RuTy::TraitObj(other_trait_obj) => trait_obj == other_trait_obj,
                _ => false,
            },
            RuTy::RawPointer(pointer, mutability) => match other {
                RuTy::RawPointer(other_pointer, other_mutability) => {
                    pointer == other_pointer && mutability == other_mutability
                }
                _ => false,
            },
            RuTy::Union(union_t) => match other {
                RuTy::Union(other_union) => union_t == other_union,
                _ => false,
            },
            RuTy::Slice(slice) => match other {
                RuTy::Slice(other_slice) => slice == other_slice,
                _ => false,
            },
            RuTy::AsTrait(orig_ty, as_ty) => match other {
                RuTy::AsTrait(o_orig_ty, o_as_ty) => orig_ty == o_orig_ty && as_ty == o_as_ty,
                _ => false,
            },
            RuTy::Relative(rel, segment) => match other {
                RuTy::Relative(other_rel, other_segment) => {
                    rel == other_rel && segment == other_segment
                }
                _ => false,
            },
            RuTy::Fn => match other {
                RuTy::Fn => true,
                _ => false,
            },
        }
    }
}

fn trait_def_to_trait(def_id: DefId, tcx: &TyCtxt<'_>) -> RuTrait {
    assert!(tcx.is_trait(def_id));

    let bounding_trait_def = tcx.trait_def(def_id);
    let generics = tcx.generics_of(bounding_trait_def.def_id);
    info!("Trait generics are: {:?}", generics);
    let trait_name = def_id_name(bounding_trait_def.def_id, tcx);
    RuTrait::new(&trait_name, vec![], vec![])
}

pub fn mir_ty_to_t(ty: rustc_middle::ty::Ty<'_>, tcx: &TyCtxt<'_>) -> RuTy {
    match ty.kind() {
        TyKind::Adt(adt_def, subst_ref) => adt_def_to_t(adt_def, tcx),
        TyKind::Param(param) => RuTy::Generic(RuGeneric::new(param.name.as_str(), vec![])),

        _ => todo!("{:?}", ty.kind()),
    }
}

pub fn def_id_name(def_id: DefId, tcx: &TyCtxt<'_>) -> String {
    tcx.def_path_str(def_id)
}

pub fn ty_name(ty: Ty) -> &str {
    match ty.kind() {
        TyKind::Param(param) => param.name.as_str(),
        TyKind::Int(int) => int.name_str(),
        TyKind::Uint(uint) => uint.name_str(),
        TyKind::Float(float) => float.name_str(),
        _ => todo!("{:?}", ty.kind()),
    }
}

/// Converts an AdtDef to a T
fn adt_def_to_t(adt_def: &AdtDef, tcx: &TyCtxt<'_>) -> RuTy {
    match adt_def.adt_kind() {
        AdtKind::Struct => {
            // Has only one variant
            assert_eq!(adt_def.variants().len(), 1);
            let variant = adt_def.variants().get(VariantIdx::from_usize(0)).unwrap();
            let name = def_id_name(adt_def.did(), tcx);
            let generics = generics_of_item(adt_def.did(), tcx);
            RuTy::Struct(RuStruct::new(&name, generics, is_local(adt_def.did())))
        }
        AdtKind::Union => {
            todo!("Adt def is union")
        }
        AdtKind::Enum => {
            let enum_name = def_id_name(adt_def.did(), tcx);
            let mut variants = Vec::with_capacity(adt_def.variants().len());
            for variant in adt_def.variants() {
                let variant_name = variant.name.to_string();

                let params = variant
                    .fields
                    .iter()
                    .map(|f| {
                        let field_name = f.name.to_string();
                        let field_t = mir_ty_to_t(tcx.type_of(f.did), tcx);
                        RuParam::new(Some(&field_name), field_t, false)
                    })
                    .collect::<Vec<_>>();

                match variant.ctor_kind {
                    CtorKind::Fn => {
                        variants.push(RuEnumVariant::Tuple(variant_name, params));
                    }
                    _ => todo!("{:?}", variant.ctor_kind),
                }
            }

            let generics = generics_of_item(adt_def.did(), tcx);

            let t = RuTy::Enum(RuEnum::new(
                &enum_name,
                generics,
                variants,
                is_local(adt_def.did()),
            ));
            info!("Extracted enum: {:?}", t);
            t
        }
    }
}

/// Converts the generics of an item, e.g., struct or method, to Ts
pub fn generics_of_item(def_id: DefId, tcx: &TyCtxt<'_>) -> Vec<RuTy> {
    let adt_predicates = tcx.predicates_defined_on(def_id);

    let mut generics: Vec<RuGeneric> = Vec::new();
    adt_predicates
        .predicates
        .iter()
        .filter_map(|(predicate, _)| {
            let binder = predicate.kind();

            match binder.skip_binder() {
                PredicateKind::Trait(trait_predicate) => {
                    let self_ty = trait_predicate.self_ty();
                    let name = ty_name(self_ty);
                    let trait_ = trait_def_to_trait(trait_predicate.def_id(), tcx);
                    let g = RuGeneric::new(name, vec![trait_]);

                    Some(g)
                }
                PredicateKind::RegionOutlives(_) => None,
                PredicateKind::TypeOutlives(_) => None,
                _ => todo!("{:?}", binder.skip_binder()),
            }
        })
        .for_each(|mut g| {
            // The compiler splits things like T: Copy + Debug into
            // two separate predicates with same name, i.e., T, so we merge them back again
            let existing_generic = generics.iter_mut().find(|e| e.name == g.name);
            if let Some(generics) = existing_generic {
                generics.bounds.append(&mut g.bounds);
            } else {
                generics.push(g);
            }
        });

    generics.iter().map(|g| RuTy::Generic(g.clone())).collect()
}

/// Returns the bounds of a generic item
fn item_bounds(def_id: DefId, tcx: &TyCtxt<'_>) -> Vec<RuTrait> {
    let item_bounds = tcx.item_bounds(def_id);
    todo!("Item bounds are: {:?}", item_bounds)
}

impl Debug for RuTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuTy::Ref(ty, _) => {
                write!(f, "&")?;
                Debug::fmt(ty, f)
            }
            RuTy::Prim(prim_ty) => Debug::fmt(prim_ty, f),
            RuTy::Struct(struct_t) => Debug::fmt(struct_t, f),
            RuTy::Generic(generic_ty) => Debug::fmt(generic_ty, f),
            RuTy::Enum(enum_ty) => Debug::fmt(enum_ty, f),
            RuTy::Array(array_ty) => Debug::fmt(array_ty, f),
            RuTy::Tuple(types) => Debug::fmt(types, f),
            RuTy::TraitObj(trait_obj) => Debug::fmt(trait_obj, f),
            RuTy::RawPointer(pointer, mutable) => {
                let mutability = if *mutable { "mut" } else { "const" };
                write!(f, "*")?;
                write!(f, "{}", mutability)?;
                write!(f, " ")?;
                Debug::fmt(pointer, f)
            }
            RuTy::Union(union_t) => Debug::fmt(union_t, f),
            RuTy::Slice(slice) => {
                write!(f, "[")?;
                Debug::fmt(slice, f)?;
                write!(f, "]")
            }
            RuTy::AsTrait(orig_ty, as_ty) => {
                write!(f, "<")?;
                Debug::fmt(orig_ty, f)?;
                write!(f, " as ")?;
                Debug::fmt(as_ty, f)?;
                write!(f, ">")
            }
            RuTy::Relative(rel, segment) => {
                Debug::fmt(rel, f)?;
                write!(f, "::{}", segment)
            }
            RuTy::Fn => {
                write!(f, "fn")
            }
        }
    }
}

impl From<TyKind<'_>> for RuTy {
    fn from(kind: TyKind) -> Self {
        match kind {
            TyKind::Bool => RuTy::Prim(RuPrim::Bool),
            TyKind::Char => RuTy::Prim(RuPrim::Char),
            TyKind::Int(int) => <RuTy as From<MirInt>>::from(int),
            _ => todo!("{:?}", kind),
        }
    }
}

impl From<MirInt> for RuTy {
    fn from(_: MirInt) -> Self {
        todo!()
    }
}

impl From<PrimTy> for RuTy {
    fn from(ty: PrimTy) -> Self {
        let ty = match ty {
            PrimTy::Int(int_ty) => {
                let int_ty = match int_ty {
                    IntTy::Isize => RuInt::Isize,
                    IntTy::I8 => RuInt::I8,
                    IntTy::I16 => RuInt::I16,
                    IntTy::I32 => RuInt::I32,
                    IntTy::I64 => RuInt::I64,
                    IntTy::I128 => RuInt::I128,
                };
                RuPrim::Int(int_ty)
            }
            PrimTy::Uint(uint_ty) => {
                let uint_ty = match uint_ty {
                    UintTy::Usize => RuUInt::Usize,
                    UintTy::U8 => RuUInt::U8,
                    UintTy::U16 => RuUInt::U16,
                    UintTy::U32 => RuUInt::U32,
                    UintTy::U64 => RuUInt::U64,
                    UintTy::U128 => RuUInt::U128,
                };
                RuPrim::Uint(uint_ty)
            }
            PrimTy::Float(float_ty) => {
                let float_ty = match float_ty {
                    FloatTy::F32 => RuFloat::F32,
                    FloatTy::F64 => RuFloat::F64,
                };
                RuPrim::Float(float_ty)
            }
            PrimTy::Str => RuPrim::Str,
            PrimTy::Bool => RuPrim::Bool,
            PrimTy::Char => RuPrim::Char,
        };
        RuTy::Prim(ty)
    }
}

impl From<String> for RuTy {
    fn from(s: String) -> Self {
        let ty = match s.as_str() {
            "i8" => RuPrim::Int(RuInt::I8),
            "i16" => RuPrim::Int(RuInt::I16),
            "i32" => RuPrim::Int(RuInt::I32),
            "i64" => RuPrim::Int(RuInt::I64),
            "i128" => RuPrim::Int(RuInt::I128),
            "isize" => RuPrim::Int(RuInt::Isize),
            "u8" => RuPrim::Uint(RuUInt::U8),
            "u16" => RuPrim::Uint(RuUInt::U16),
            "u32" => RuPrim::Uint(RuUInt::U32),
            "u64" => RuPrim::Uint(RuUInt::U64),
            "u128" => RuPrim::Uint(RuUInt::U128),
            "usize" => RuPrim::Uint(RuUInt::Usize),
            "f32" => RuPrim::Float(RuFloat::F32),
            "f64" => RuPrim::Float(RuFloat::F64),
            "bool" => RuPrim::Bool,
            _ => todo!("{}", s),
        };

        RuTy::Prim(ty)
    }
}

impl RuTy {
    pub fn name(&self) -> String {
        match self {
            RuTy::Prim(prim) => prim.name_str().to_string(),
            RuTy::Struct(struct_t) => struct_t.name().to_string(),
            RuTy::Generic(generic) => generic.name().to_string(),
            RuTy::Ref(r, _) => r.name(),
            RuTy::Enum(enum_ty) => enum_ty.name().to_owned(),
            RuTy::Array(_) => String::from("array"),
            RuTy::Tuple(_) => String::from("tuple"),
            RuTy::TraitObj(_) => String::from("trait object"),
            RuTy::RawPointer(_, _) => String::from("raw pointer"),
            RuTy::Union(union_t) => union_t.name().to_string(),
            RuTy::Slice(slice) => String::from("slice"),
            RuTy::AsTrait(orig_ty, as_ty) => "as".to_string(),
            RuTy::Relative(_, _) => "relative".to_string(),
            RuTy::Fn => "fn".to_string(),
        }
    }

    pub fn is_prim(&self) -> bool {
        match self {
            RuTy::Prim(_) => true,
            RuTy::Ref(r, _) => r.is_prim(),
            _ => false,
        }
    }

    pub fn is_generic(&self) -> bool {
        match self {
            RuTy::Generic(_) => true,
            RuTy::Ref(r, _) => r.is_generic(),
            _ => false,
        }
    }

    pub fn is_enum(&self) -> bool {
        match self {
            RuTy::Enum(_) => true,
            _ => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        match self {
            RuTy::Struct(_) => true,
            _ => false,
        }
    }

    pub fn expect_generic(&self) -> &RuGeneric {
        match self {
            RuTy::Generic(generic) => generic,
            RuTy::Ref(r, _) => r.expect_generic(),
            _ => panic!("Is not generic: {:?}", self),
        }
    }

    pub fn expect_generic_mut(&mut self) -> &mut RuGeneric {
        match self {
            RuTy::Generic(generic) => generic,
            RuTy::Ref(r, _) => r.expect_generic_mut(),
            _ => panic!("Is not a generic: {:?}", self),
        }
    }

    pub fn expect_enum(&self) -> &RuEnum {
        match self {
            RuTy::Enum(enum_t) => enum_t,
            _ => panic!("Is not an enum"),
        }
    }

    pub fn expect_enum_mut(&mut self) -> &mut RuEnum {
        match self {
            RuTy::Enum(enum_t) => enum_t,
            _ => panic!("Is not an enum"),
        }
    }

    pub fn expect_struct(&self) -> &RuStruct {
        match self {
            RuTy::Struct(struct_t) => struct_t,
            _ => panic!("Is not a struct"),
        }
    }

    pub fn expect_struct_mut(&mut self) -> &mut RuStruct {
        match self {
            RuTy::Struct(struct_t) => struct_t,
            _ => panic!("Is not a struct"),
        }
    }

    pub fn overwrite_generics(&mut self, generics: Vec<RuTy>) {
        match self {
            RuTy::Ref(r, _) => r.as_mut().overwrite_generics(generics),
            RuTy::Struct(s) => s.overwrite_generics(generics),
            RuTy::Enum(e) => e.overwrite_generics(generics),
            _ => todo!(),
        }
    }
}

impl Display for RuTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuTy::Prim(prim) => write!(f, "{}", prim.name_str()),
            RuTy::Struct(struct_t) => Display::fmt(struct_t, f),
            RuTy::Generic(generic) => Display::fmt(generic, f),
            RuTy::Ref(r, _) => {
                write!(f, "&")?;
                Display::fmt(r.as_ref(), f)
            }
            RuTy::Enum(enum_ty) => Display::fmt(enum_ty, f),
            RuTy::Array(array) => Display::fmt(array, f),
            RuTy::Tuple(types) => {
                let results = types
                    .types
                    .iter()
                    .map(|t| Display::fmt(t.as_ref(), f))
                    .collect::<Vec<_>>();
                if let Some(result) = results.last() {
                    *result
                } else {
                    Ok(())
                }
            }
            RuTy::TraitObj(trait_obj) => Display::fmt(trait_obj, f),
            RuTy::RawPointer(pointer, mutable) => {
                let mutability = if *mutable { "mut" } else { "const" };
                write!(f, "*")?;
                write!(f, "{}", mutability)?;
                write!(f, " ")?;
                Display::fmt(pointer, f)
            }
            RuTy::Union(union_t) => Display::fmt(union_t, f),
            RuTy::Slice(slice) => {
                write!(f, "[")?;
                Display::fmt(slice, f)?;
                write!(f, "]")
            }
            RuTy::AsTrait(orig_ty, trait_) => {
                write!(f, "<")?;
                Display::fmt(orig_ty, f)?;
                write!(f, " as ")?;
                Display::fmt(trait_, f)?;
                write!(f, ">")
            }
            RuTy::Relative(rel, segment) => {
                Display::fmt(rel, f)?;
                write!(f, "::{}", segment)
            }
            RuTy::Fn => {
                write!(f, "fn")
            }
        }
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct RuArray {
    length: usize,
    ty: RuTy,
}

impl Debug for RuArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for RuArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}; {}]", self.ty, self.length)
    }
}

impl RuArray {
    pub fn new(ty: RuTy, length: usize) -> Self {
        Self { ty, length }
    }
}

impl PartialEq for RuArray {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty && self.length == other.length
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct RuTuple {
    types: Vec<Box<RuTy>>,
}

impl Debug for RuTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for RuTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let types = self
            .types
            .iter()
            .map(|ty| format!("{}", ty))
            .collect::<Vec<_>>();
        write!(f, "({})", types.join(", "))
    }
}

impl PartialEq for RuTuple {
    fn eq(&self, other: &Self) -> bool {
        self.types == other.types
    }
}

impl RuTuple {
    pub fn new(types: Vec<Box<RuTy>>) -> Self {
        Self { types }
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct RuTraitObj {
    name: String,
    is_local: bool,
}

impl Debug for RuTraitObj {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for RuTraitObj {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "dyn {}", self.name)
    }
}

impl PartialEq for RuTraitObj {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.is_local == other.is_local
    }
}

impl RuTraitObj {
    pub fn new(name: &str, is_local: bool) -> Self {
        RuTraitObj {
            name: name.to_string(),
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
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct RuUnion {
    name: String,
    is_local: bool,
}

impl Debug for RuUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for RuUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for RuUnion {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.is_local == other.is_local
    }
}

impl RuUnion {
    pub fn new(name: &str, is_local: bool) -> Self {
        Self {
            name: name.to_string(),
            is_local,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Hash, Eq, Serialize, Deserialize)]
pub struct RuStruct {
    name: String,
    generics: Vec<RuTy>,
    is_local: bool,
}

impl Debug for RuStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for RuStruct {
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

impl PartialEq for RuStruct {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.generics == other.generics
    }
}

impl RuStruct {
    pub fn new(name: &str, generics: Vec<RuTy>, is_local: bool) -> Self {
        RuStruct {
            name: name.to_string(),
            is_local,
            generics,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn generics(&self) -> &Vec<RuTy> {
        &self.generics
    }

    pub fn bind_generics(&mut self, types: Vec<RuTy>) {
        self.generics = types;
    }

    pub fn overwrite_generics(&mut self, generics: Vec<RuTy>) {
        self.generics = generics;
    }
}

#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
pub struct RuEnum {
    name: String,
    generics: Vec<RuTy>,
    variants: Vec<RuEnumVariant>,
    is_local: bool,
}

impl PartialEq for RuEnum {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        self.variants == other.variants
    }
}

impl Display for RuEnum {
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

impl RuEnum {
    pub fn new(
        name: &str,
        generics: Vec<RuTy>,
        variants: Vec<RuEnumVariant>,
        is_local: bool,
    ) -> Self {
        Self {
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

    pub fn overwrite_generics(&mut self, generics: Vec<RuTy>) {
        self.generics = generics;
    }
}

#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize)]
pub enum RuEnumVariant {
    Struct(String, Vec<RuParam>),
    Tuple(String, Vec<RuParam>),
    Unit(String),
}

impl RuEnumVariant {
    pub fn name(&self) -> &str {
        match self {
            RuEnumVariant::Struct(name, _) => name,
            RuEnumVariant::Tuple(name, _) => name,
            RuEnumVariant::Unit(name) => name,
        }
    }

    pub fn params(&self) -> Vec<RuParam> {
        match self {
            RuEnumVariant::Struct(_, p) => p.clone(),
            RuEnumVariant::Tuple(_, p) => p.clone(),
            RuEnumVariant::Unit(_) => vec![],
        }
    }
}

impl PartialEq for RuEnumVariant {
    fn eq(&self, other: &Self) -> bool {
        match self {
            RuEnumVariant::Struct(name, s) => match other {
                RuEnumVariant::Struct(other_name, o) => name == other_name && s == o,
                _ => false,
            },
            RuEnumVariant::Tuple(name, p) => match other {
                RuEnumVariant::Tuple(other_name, o) => name == other_name && p == o,
                _ => false,
            },
            RuEnumVariant::Unit(name) => match other {
                RuEnumVariant::Unit(other_name) => name == other_name,
                _ => false,
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuGeneric {
    scope: Uuid,
    name: String,
    bounds: Vec<RuTrait>,
}

impl Debug for RuGeneric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl RuGeneric {
    pub fn new(name: &str, bounds: Vec<RuTrait>) -> Self {
        Self {
            scope: Uuid::new_v4(),
            name: name.to_string(),
            bounds,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bounds(&self) -> &Vec<RuTrait> {
        &self.bounds
    }

    pub fn set_bounds(&mut self, bounds: Vec<RuTrait>) {
        self.bounds = bounds;
    }
}

impl Display for RuGeneric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;
        if !self.bounds.is_empty() {
            write!(f, ": ")?;
            let bounds = self
                .bounds
                .iter()
                .map(|b| format!("{}", b))
                .collect::<Vec<_>>()
                .join(" + ");
            write!(f, "{}", bounds)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuTrait {
    name: String,
    generics: Vec<RuTy>,
    associated_types: Vec<RuAssociatedType>,
}

impl RuTrait {
    pub fn new(name: &str, generics: Vec<RuTy>, associated_types: Vec<RuAssociatedType>) -> Self {
        RuTrait {
            name: name.to_string(),
            generics,
            associated_types,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn generics(&self) -> &Vec<RuTy> {
        &self.generics
    }

    pub fn associated_types(&self) -> &Vec<RuAssociatedType> {
        &self.associated_types
    }
}

impl Display for RuTrait {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuAssociatedType {
    name: String,
    ty: RuTy,
}

#[derive(Clone, Eq, Hash, Serialize, Deserialize)]
pub struct RuParam {
    ty: RuTy,
    mutable: bool,
    name: Option<String>,
}

impl PartialEq for RuParam {
    fn eq(&self, other: &Self) -> bool {
        self.mutable == other.mutable && self.ty == other.ty && self.name == other.name
    }
}

impl Debug for RuParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}: ", name)?;
        }

        if self.mutable {
            write!(f, "mut ")?;
        }
        write!(f, "{:?}", &self.ty)
    }
}

impl RuParam {
    pub fn new(name: Option<&str>, ty: RuTy, mutable: bool) -> Self {
        RuParam {
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
            RuTy::Ref(_, _) => true,
            _ => false,
        }
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }

    pub fn real_ty(&self) -> &RuTy {
        match &self.ty {
            RuTy::Ref(ty, _) => ty.as_ref(),
            _ => &self.ty,
        }
    }

    pub fn real_ty_mut(&mut self) -> &mut RuTy {
        todo!()
    }

    pub fn is_primitive(&self) -> bool {
        self.ty.is_prim()
    }

    pub fn is_generic(&self) -> bool {
        self.ty.is_generic()
    }

    pub fn original_ty(&self) -> &RuTy {
        &self.ty
    }
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum RuPrim {
    Int(RuInt),
    Uint(RuUInt),
    Float(RuFloat),
    Str,
    Bool,
    Char,
}

impl Debug for RuPrim {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name_str())
    }
}

impl RuPrim {
    pub fn name_str(self) -> &'static str {
        match self {
            RuPrim::Int(i) => i.name_str(),
            RuPrim::Uint(u) => u.name_str(),
            RuPrim::Float(f) => f.name_str(),
            RuPrim::Str => "str",
            RuPrim::Bool => "bool",
            RuPrim::Char => "char",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum RuInt {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
}

impl RuInt {
    pub fn name_str(&self) -> &'static str {
        match *self {
            RuInt::Isize => "isize",
            RuInt::I8 => "i8",
            RuInt::I16 => "i16",
            RuInt::I32 => "i32",
            RuInt::I64 => "i64",
            RuInt::I128 => "i128",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum RuUInt {
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl RuUInt {
    pub fn name_str(&self) -> &'static str {
        match *self {
            RuUInt::Usize => "usize",
            RuUInt::U8 => "u8",
            RuUInt::U16 => "u16",
            RuUInt::U32 => "u32",
            RuUInt::U64 => "u64",
            RuUInt::U128 => "u128",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum RuFloat {
    F32,
    F64,
}

impl RuFloat {
    pub fn name_str(self) -> &'static str {
        match self {
            RuFloat::F32 => "f32",
            RuFloat::F64 => "f64",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuConstVal {
    val: String,
    ty: RuTy,
}

impl RuConstVal {
    pub fn new(val: String, ty: RuTy) -> Self {
        Self { val, ty }
    }
}
