use crate::extractor::hir_ty_to_t;
use crate::traits::analyze_trait;
use crate::types::{mir_ty_to_t, ArrayT, EnumT, Generic, Param, StructT, Trait, TupleT, T, TraitObjT, UnionT};
use log::{debug, error, info, warn};
use rustc_data_structures::fx::FxIndexMap;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::DefId;
use rustc_hir::definitions::{DefPathData, DisambiguatedDefPathData};
use rustc_hir::lang_items::LangItemGroup;
use rustc_hir::{
    AnonConst, ArrayLen, FnRetTy, GenericArg, GenericBound, GenericParam, GenericParamKind,
    Generics, HirId, Impl, Item, ItemKind, MutTy, Mutability, Node, ParamName, Path, PathSegment,
    PrimTy, QPath, Ty, TyKind, WherePredicate,
};
use rustc_middle::dep_graph::DepContext;
use rustc_middle::mir::interpret::{ConstValue, Scalar};
use rustc_middle::ty::fast_reject::SimplifiedType;
use rustc_middle::ty::subst::{GenericArgKind, SubstsRef};
use rustc_middle::ty::{AdtKind, TyCtxt, TypeckResults};
use rustc_span::def_id::{CrateNum, LocalDefId};
use rustc_span::{FileName, RealFileName, Span, DUMMY_SP};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::option::Option::Some;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn rustc_get_crate_name(rustc_args: &[String]) -> String {
    let pos = rustc_args.iter().position(|a| a == "--crate-name").unwrap();
    rustc_args.get(pos + 1).map(|s| s.to_string()).unwrap()
}

pub fn cargo_path() -> io::Result<PathBuf> {
    match which::which("cargo") {
        Ok(p) => Ok(p),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
    }
}

pub fn fmt_path() -> io::Result<PathBuf> {
    match which::which("rustfmt") {
        Ok(p) => Ok(p),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
    }
}

pub fn ty_to_param(
    name: Option<&str>,
    ty: &Ty,
    self_ty: Option<&T>,
    parent_generics: &Vec<T>,
    tcx: &TyCtxt<'_>,
) -> Option<Param> {
    let mutability = match &ty.kind {
        TyKind::Rptr(_, mut_ty) => mut_ty.mutbl == Mutability::Mut,
        _ => false,
    };

    let real_ty = ty_to_t(ty, self_ty, parent_generics, tcx)?;
    Some(Param::new(name, real_ty, mutability))
}

pub fn ty_to_t(
    ty: &Ty,
    self_ty: Option<&T>,
    defined_generics: &Vec<T>,
    tcx: &TyCtxt<'_>,
) -> Option<T> {
    info!("Returns {:?}", ty);
    match &ty.kind {
        TyKind::Rptr(_, mut_ty) => ty_to_t(mut_ty.ty, self_ty, defined_generics, tcx).map(|t| {
            let mutable = match mut_ty.mutbl {
                Mutability::Mut => true,
                Mutability::Not => false,
            };
            T::Ref(Box::new(t), mutable)
        }),
        TyKind::Path(q_path) => {
            match q_path {
                QPath::Resolved(_, path) => {
                    path_to_t(path, self_ty, defined_generics, tcx)
                    // TODO parse generic args of the type
                }
                QPath::TypeRelative(ty, segment) => {
                    let as_type = ty_to_t(ty, self_ty, defined_generics, tcx);
                    as_type.map(|as_type| T::Relative(Box::new(as_type), segment.ident.to_string()))
                },
                _ => unimplemented!("{:?}", q_path),
            }
        }
        TyKind::Slice(ty) => ty_to_t(ty, self_ty, defined_generics, tcx).map(|t| {
            T::Slice(Box::new(t))
        }),
        TyKind::OpaqueDef(item, generic_args) => {
            warn!(
                "HIR: Skipping opaquedef of {:?} with generic args {:?}",
                item, generic_args
            );
            None
        }
        TyKind::Tup(tys) => {
            let ts = tys
                .iter()
                .filter_map(|ty| ty_to_t(ty, self_ty, defined_generics, tcx))
                .map(Box::new)
                .collect::<Vec<_>>();
            if ts.len() != tys.len() {
                todo!("HIR: Could not extract tuple of ({:?})", tys)
            }
            Some(T::Tuple(TupleT::new(ts)))
        }
        TyKind::Array(ty, array_len) => {
            let array_len = eval_array_len(array_len, tcx);
            if let Some(array_len) = array_len {
                return ty_to_t(ty, self_ty, defined_generics, tcx)
                    .map(|ty| T::Array(Box::new(ArrayT::new(ty, array_len))));
            }

            todo!("Unknown array length")
        }
        TyKind::TraitObject(trait_refs, _, _) => {
            assert!(trait_refs.len() == 1);
            let poly_trait_ref = trait_refs.get(0).unwrap();
            let name = res_to_name(&poly_trait_ref.trait_ref.path.res, tcx);
            let trait_obj = TraitObjT::new(&name, false);
            Some(T::TraitObj(trait_obj))
        }
        TyKind::Ptr(ptr) => {
            let t = ty_to_t(ptr.ty, self_ty, defined_generics, tcx);
            t.map(|t| T::RawPointer(Box::new(t), ptr.mutbl == Mutability::Mut)).or_else(|| todo!("{:?}", ptr))
        }
        TyKind::BareFn(bare_fn) => {
            None
        }
        _ => todo!("Ty kind is: {:?}", &ty.kind),
    }
}

pub fn path_to_generics(
    path: &rustc_hir::Path<'_>,
    self_: Option<&T>,
    defined_generics: &Vec<T>,
    tcx: &TyCtxt<'_>,
) -> Vec<T> {
    let generics = path
        .segments
        .iter()
        .filter_map(|s| {
            if let Some(args) = s.args {
                Some(
                    args.args
                        .iter()
                        .filter_map(|a| generic_arg_to_t(a, self_, defined_generics, tcx))
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<_>>();

    generics
}

pub fn path_to_t(
    path: &Path,
    self_ty: Option<&T>,
    defined_generics: &Vec<T>,
    tcx: &TyCtxt<'_>,
) -> Option<T> {
    match &path.res {
        Res::Def(def_kind, def_id) => match def_kind {
            DefKind::Struct | DefKind::Enum | DefKind::Union => {
                let generics = path_to_generics(path, self_ty, defined_generics, tcx);
                debug!("Generics are: {:?}", generics);
                def_kind_to_t(def_kind, *def_id, self_ty, path, &generics, tcx)
            }
            DefKind::TyParam => {
                let name = path
                    .segments
                    .iter()
                    .map(|s| s.ident.name.to_ident_string())
                    .collect::<Vec<_>>()
                    .join("::");

                let bounds = defined_generics
                    .iter()
                    .find(|g| g.name() == name)
                    .map(|g| g.expect_generic().bounds())
                    .map_or(vec![], |bounds| bounds.clone());

                return Some(T::Generic(Generic::new(&name, bounds)));
            }
            DefKind::Impl => {
                warn!("HIR: impl is being returned");
                None
            }
            _ => None,
        },
        Res::PrimTy(prim_ty) => Some(T::from(*prim_ty)),
        Res::SelfTy { trait_, .. } => {
            if let Some(trait_) = trait_ {
                let trait_name = tcx.def_path_str(*trait_);
                let trait_instance = Trait::new(&trait_name, vec![], vec![]);
                self_ty.map(|self_ty| T::AsTrait(Box::new(self_ty.clone()), trait_instance))
            } else {
                self_ty.map(|self_ty| self_ty.clone())
            }
        },
        _ => {
            unimplemented!("{:?}", &path.res)
        }
    }
}

pub fn def_kind_to_t(
    def_kind: &DefKind,
    def_id: DefId,
    self_ty: Option<&T>,
    path: &rustc_hir::Path<'_>,
    defined_generics: &Vec<T>,
    tcx: &TyCtxt<'_>,
) -> Option<T> {
    let name = tcx.def_path_str(def_id);
    let generics = path_to_generics(path, self_ty, defined_generics, tcx);

    match def_kind {
        DefKind::Enum => Some(T::Enum(EnumT::new(
            &name,
            generics,
            vec![],
            is_local(def_id),
        ))),
        DefKind::Struct => Some(T::Struct(StructT::new(&name, generics, is_local(def_id)))),
        DefKind::Union => Some(T::Union(UnionT::new(&name, is_local(def_id)))),
        _ => unimplemented!(),
    }
}

pub fn generic_arg_to_t(
    generic_arg: &GenericArg,
    self_: Option<&T>,
    defined_generics: &Vec<T>,
    tcx: &TyCtxt<'_>,
) -> Option<T> {
    match generic_arg {
        GenericArg::Type(ty) => ty_to_t(ty, self_, defined_generics, tcx),
        _ => None,
    }
}

pub fn def_id_to_t(def_id: DefId, tcx: &TyCtxt<'_>) -> Option<T> {
    let ty = tcx.type_of(def_id);
    match ty.kind() {
        rustc_middle::ty::TyKind::Adt(adt_def, substs) => {
            let generics = substs
                .non_erasable_generics()
                .filter_map(|kind| generic_to_t(kind, tcx))
                .collect::<Vec<_>>();

            if generics.len() != substs_len(substs) {
                warn!("HIR: not all generics could be parsed: {:?}", substs);
                return None;
            }
            let name = tcx.def_path_str(def_id);

            info!("Type of is {:?}", adt_def.adt_kind());

            match adt_def.adt_kind() {
                AdtKind::Struct => {
                    let t = T::Struct(StructT::new(&name, generics, is_local(def_id)));

                    Some(t)
                }
                AdtKind::Union => {
                    warn!("HIR: Skipping tuple");
                    None
                }
                AdtKind::Enum => {
                    let t = T::Enum(EnumT::new(&name, generics, vec![], is_local(def_id)));
                    Some(t)
                }
            }
        }
        _ => todo!(),
    }
}

pub fn def_id_to_enum(def_id: DefId, tcx: &TyCtxt<'_>) -> Option<T> {
    let ty = tcx.type_of(def_id);
    match ty.kind() {
        rustc_middle::ty::TyKind::Adt(_, substs) => {
            let generics = substs
                .non_erasable_generics()
                .filter_map(|kind| generic_to_t(kind, tcx))
                .collect::<Vec<_>>();
            if generics.len() != substs.len() {
                warn!("HIR: not all generics could be parsed: {:?}", substs);
                return None;
            }

            let name = tcx.def_path_str(def_id);
            let variants = vec![];
            let t = T::Enum(EnumT::new(&name, generics, variants, is_local(def_id)));

            Some(t)
        }
        _ => todo!(),
    }
}

pub fn substs_len(substs: SubstsRef) -> usize {
    substs
        .non_erasable_generics()
        .filter(|kind| match kind {
            GenericArgKind::Type(_) => true,
            _ => false,
        })
        .count()
}

pub fn generic_to_t(generic_kind: GenericArgKind, tcx: &TyCtxt<'_>) -> Option<T> {
    match generic_kind {
        GenericArgKind::Lifetime(_) => None,
        GenericArgKind::Type(ty) => tys_to_t(ty, tcx),
        GenericArgKind::Const(c) => {
            todo!("Const is {:?}", c)
        }
    }
}

pub fn tys_to_t(ty: rustc_middle::ty::Ty, tcx: &TyCtxt<'_>) -> Option<T> {
    match ty.kind() {
        rustc_middle::ty::TyKind::Param(param) => {
            let name = param.name.to_string();
            let generic_param = T::Generic(Generic::new(&name, vec![]));
            Some(generic_param)
        }
        _ => todo!("Ty is {:?}", ty),
    }
}

pub fn generics_to_ts(generics: &Generics, tcx: &TyCtxt<'_>) -> Vec<T> {
    let mut where_generics = if !generics.where_clause.predicates.is_empty() {
        let predicates = generics.where_clause.predicates;
        let where_generics = predicates
            .iter()
            .filter_map(|p| match p {
                WherePredicate::BoundPredicate(p) => {
                    info!("{:?}", p);
                    let ty = ty_to_t(p.bounded_ty, None, &vec![], tcx);
                    if let Some(ty) = &ty {
                        if !ty.is_generic() {
                            return None;
                        }
                    }
                    let bounds = p
                        .bounds
                        .iter()
                        .filter_map(|b| generic_bound_to_trait(b, tcx))
                        .collect::<Vec<_>>();
                    ty.map(|mut ty| {
                        ty.expect_generic_mut().set_bounds(bounds);
                        (ty.name(), ty)
                    })
                }
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        where_generics
    } else {
        HashMap::new()
    };

    generics
        .params
        .iter()
        .filter_map(|g| generic_param_to_t(g, tcx))
        .for_each(|g| {
            let _ = where_generics
                .entry(g.name())
                .and_modify(|e| *e = merge_bounds(&g, e))
                .or_insert(g);
        });

    let overall_generics = where_generics.values().cloned().collect::<Vec<_>>();
    info!("Overall generics: {:?}", overall_generics);
    overall_generics
}

fn merge_bounds(a: &T, b: &T) -> T {
    assert_eq!(a.name(), b.name());
    assert!(a.is_generic() && b.is_generic());

    let mut a_bounds = a.expect_generic().bounds().clone();
    let mut b_bounds = b.expect_generic().bounds().clone();

    a_bounds.append(&mut b_bounds);
    let generic = Generic::new(&a.name(), a_bounds);
    T::Generic(generic)
}

pub fn generic_param_to_t(param: &GenericParam<'_>, tcx: &TyCtxt<'_>) -> Option<T> {
    if let GenericParamKind::Type { .. } = &param.kind {
        if let ParamName::Plain(ident) = &param.name {
            let name = ident.name.to_ident_string();

            let bounds = param
                .bounds
                .iter()
                .filter_map(|b| generic_bound_to_trait(b, tcx))
                .collect::<Vec<_>>();
            return Some(T::Generic(Generic::new(&name, bounds)));
        }
    }

    None
}

pub fn generic_bound_to_trait(bound: &GenericBound<'_>, tcx: &TyCtxt<'_>) -> Option<Trait> {
    match bound {
        GenericBound::Trait(trait_ref, _) => {
            let def_id = trait_ref.trait_ref.trait_def_id()?;

            analyze_trait(def_id, tcx);

            let name = tcx.def_path_str(def_id);

            //let implemented_by = trait_implementations(def_id, tcx);
            //let std_impls = implemented_by.iter().filter(|im| im).collect::<Vec<_>>();
            //info!("Trait {}: implemented by {:?}", name, implemented_by);
            //info!(
            //    "Trait {} implemented by: {:?}",
            //    name, &implemented_by.non_blanket_impls
            //);
            Some(Trait::new(&name, vec![], vec![]))
        }
        GenericBound::LangItemTrait(_, _, _, _) => todo!(),
        GenericBound::Outlives(_) => None,
    }
}

pub fn fn_ret_ty_to_t(
    ret_ty: &FnRetTy,
    self_ty: Option<&T>,
    defined_generics: &Vec<T>,
    tcx: &TyCtxt<'_>,
) -> Option<T> {
    match ret_ty {
        FnRetTy::DefaultReturn(_) => None,
        FnRetTy::Return(ty) => ty_to_t(ty, self_ty, defined_generics, tcx),
    }
}

pub fn join_path_to_str(path: &rustc_hir::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub fn node_to_name(node: &Node<'_>, tcx: &TyCtxt<'_>) -> Option<String> {
    match node {
        Node::Item(item) => Some(item_to_name(item, tcx)),
        Node::Crate(_) => Some("crate".to_string()),
        Node::ForeignItem(fi) => Some(fi.ident.name.to_ident_string()),
        Node::ImplItem(ii) => Some(ii.ident.name.to_ident_string()),
        Node::TraitItem(ti) => Some(ti.ident.name.to_ident_string()),
        Node::Variant(v) => Some(v.ident.name.to_ident_string()),
        Node::Field(f) => Some(f.ident.name.to_ident_string()),
        Node::Lifetime(lt) => Some(lt.name.ident().name.to_ident_string()),
        Node::GenericParam(param) => Some(param.name.ident().name.to_ident_string()),
        _ => None,
    }
}

pub fn item_to_name(item: &Item<'_>, tcx: &TyCtxt<'_>) -> String {
    match &item.kind {
        ItemKind::Impl(im) => ty_to_name(im.self_ty, tcx),
        ItemKind::Struct(_, _) => tcx.def_path_str(item.def_id.to_def_id()),
        ItemKind::Enum(_, _) => tcx.def_path_str(item.def_id.to_def_id()),
        ItemKind::Fn(_, _, _) => tcx.def_path_str(item.def_id.to_def_id()),
        _ => item.ident.name.to_ident_string(),
    }
}

pub fn ty_to_name(ty: &Ty<'_>, tcx: &TyCtxt<'_>) -> String {
    match &ty.kind {
        TyKind::Path(path) => qpath_to_name(path, tcx),
        TyKind::Rptr(_, mut_ty) => ty_to_name(mut_ty.ty, tcx),
        TyKind::Array(ty, len) => format!("[{}; {}]", ty_to_name(ty, tcx), eval_array_len(len, tcx).unwrap()),
        _ => todo!("Trying to convert ty to name: {:?}", ty),
    }
}

pub fn qpath_to_name(qpath: &QPath<'_>, tcx: &TyCtxt<'_>) -> String {
    match qpath {
        QPath::Resolved(_, path) => res_to_name(&path.res, tcx),
        _ => todo!(),
    }
}

pub fn path_to_name(path: &Path, tcx: &TyCtxt<'_>) -> String {
    res_to_name(&path.res, tcx)
}

pub fn res_to_name(res: &Res, tcx: &TyCtxt<'_>) -> String {
    match res {
        Res::Def(_, def_id) => tcx.def_path_str(*def_id),
        Res::PrimTy(prim) => prim.name_str().to_string(),
        _ => todo!("{:?}", res),
    }
}

pub fn impl_to_def_id(im: &Impl, tcx: &TyCtxt<'_>) -> Option<DefId> {
    let self_ty = im.self_ty;
    ty_to_def_id(self_ty, tcx)
}

pub fn ty_to_def_id(ty: &Ty<'_>, tcx: &TyCtxt<'_>) -> Option<DefId> {
    match &ty.kind {
        TyKind::Path(qpath) => match qpath {
            QPath::Resolved(_, path) => path.res.opt_def_id(),
            _ => todo!(),
        },
        TyKind::Rptr(lifetime, mut_ty) => {
            let ty = mut_ty.ty;
            ty_to_def_id(ty, tcx)
        }
        TyKind::Array(ty, _) => None,
        _ => todo!("Trying to convert to struct: {:?}", &ty.kind),
    }
}

fn eval_array_len(array_len: &ArrayLen, tcx: &TyCtxt<'_>) -> Option<usize> {
    let array_len_def_id = tcx.hir().local_def_id(array_len.hir_id()).to_def_id();
    let len = tcx.const_eval_poly(array_len_def_id);

    if let Some(len) = len.ok() {
        match len {
            ConstValue::Scalar(scalar) => match scalar {
                Scalar::Int(i) => i.try_to_machine_usize(tcx.clone()).ok().map(|v| v as usize),
                Scalar::Ptr(_, _) => None,
            },
            ConstValue::Slice { .. } => None,
            ConstValue::ByRef { .. } => None,
        }
    } else {
        None
    }
}

fn def_path_data(data: &DefPathData) -> Option<String> {
    match data {
        DefPathData::CrateRoot => Some("crate".to_string()),
        DefPathData::Misc => None,
        DefPathData::Impl => None,
        DefPathData::TypeNs(ty) => Some(ty.to_ident_string()),
        DefPathData::ValueNs(value) => Some(value.to_ident_string()),
        DefPathData::MacroNs(mac) => Some(mac.to_ident_string()),
        DefPathData::LifetimeNs(_) => None,
        DefPathData::ClosureExpr => None,
        DefPathData::Ctor => None,
        DefPathData::AnonConst => None,
        DefPathData::ImplTrait => None,
        DefPathData::ForeignMod => None,
    }
}

pub fn span_to_path(span: &Span, tcx: &TyCtxt<'_>) -> Option<PathBuf> {
    let file_name = tcx.sess().source_map().span_to_filename(span.clone());
    match file_name {
        FileName::Real(real_file_name) => match real_file_name {
            RealFileName::LocalPath(path) => Some(path),
            RealFileName::Remapped { .. } => None,
        },
        _ => todo!(),
    }
}

fn fmt_string(source: &str) -> io::Result<String> {
    let rustfmt = fmt_path()?;
    let mut cmd = Command::new(&*rustfmt);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    let source = source.to_owned();
    let stdin_handle = std::thread::spawn(move || {
        let _ = child_stdin.write_all(source.as_bytes());
        source
    });

    let mut output = vec![];
    io::copy(&mut child_stdout, &mut output)?;
    let status = child.wait()?;
    let source = stdin_handle.join().unwrap();

    match String::from_utf8(output) {
        Ok(source) => match status.code() {
            Some(0) => Ok(source),
            Some(2) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Rustfmt parsing errors".to_string(),
            )),
            Some(3) => Ok(source),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal rustfmt error".to_string(),
            )),
        },
        Err(_) => Ok(source),
    }
}

pub fn is_local(def_id: DefId) -> bool {
    def_id.krate == CrateNum::from_u32(0)
}

/// Fetch all implementations of a trait with given def_id
fn trait_implementations<'tcx>(def_id: DefId, tcx: &TyCtxt<'tcx>) -> &'tcx PublicTraitImpls {
    let trait_impls = tcx.trait_impls_of(def_id);
    // Make the private fields of trait impls instance public
    let public_trait_impls: &PublicTraitImpls = unsafe { std::mem::transmute(trait_impls) };

    public_trait_impls
}

pub struct PublicTraitImpls {
    pub blanket_impls: Vec<DefId>,
    pub non_blanket_impls: FxIndexMap<SimplifiedType, Vec<DefId>>,
}
