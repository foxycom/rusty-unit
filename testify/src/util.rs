use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::DefId;
use rustc_hir::definitions::{DefPathData, DisambiguatedDefPathData};
use rustc_hir::{AnonConst, ArrayLen, FnRetTy, GenericArg, GenericBound, GenericParam, GenericParamKind, Generics, HirId, Impl, Item, ItemKind, Mutability, MutTy, Node, ParamName, PathSegment, PrimTy, QPath, Ty, TyKind, WherePredicate};
use rustc_middle::dep_graph::DepContext;
use rustc_middle::ty::subst::{GenericArgKind, SubstsRef};
use rustc_middle::ty::{AdtKind, TyCtxt, TypeckResults};
use rustc_span::def_id::{CrateNum, LocalDefId};
use rustc_span::{FileName, RealFileName, Span};
use std::io;
use std::io::Write;
use std::option::Option::Some;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use log::{debug, error, info, warn};
use crate::extractor::hir_ty_to_t;
use crate::types::{EnumT, Generic, mir_ty_to_t, Param, StructT, T, Trait, TupleT};

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
  self_hir_id: HirId,
  parent_generics: &Vec<T>,
  tcx: &TyCtxt<'_>,
) -> Option<Param> {
  let mutability = match &ty.kind {
    TyKind::Rptr(_, mut_ty) => mut_ty.mutbl == Mutability::Mut,
    _ => false
  };

  let real_ty = ty_to_t(ty, Some(self_hir_id), parent_generics, tcx)?;
  Some(Param::new(name, real_ty, mutability))
}

pub fn ty_to_t(
  ty: &Ty,
  self_: Option<HirId>,
  defined_generics: &Vec<T>,
  tcx: &TyCtxt<'_>,
) -> Option<T> {
  match &ty.kind {
    TyKind::Rptr(_, mut_ty) => ty_to_t(mut_ty.ty, self_, defined_generics, tcx).map(|t| {
      let mutable = match mut_ty.mutbl {
        Mutability::Mut => true,
        Mutability::Not => false
      };
      T::Ref(Box::new(t), mutable)
    }),
    TyKind::Path(q_path) => {
      match q_path {
        QPath::Resolved(_, path) => {
          match &path.res {
            Res::Def(def_kind, def_id) => {
              match def_kind {
                DefKind::Struct | DefKind::Enum => {
                  let generics = path_to_generics(path, self_, defined_generics, tcx);
                  debug!("Generics are: {:?}", generics);
                  path_to_t(def_kind, *def_id, self_, path, &generics, tcx)
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
                _ => None
              }
            }
            Res::PrimTy(prim_ty) => Some(T::from(*prim_ty)),
            Res::SelfTy(trait_def_id, impl_) => {
              // Self type, so replace it with the parent id
              let def_id = tcx.hir().local_def_id(self_.unwrap()).to_def_id();
              def_id_to_t(def_id, tcx)
            }
            _ => {
              unimplemented!("{:?}", &path.res)
            }
          }

          // TODO parse generic args of the type
        }
        QPath::TypeRelative(ty, _) => ty_to_t(ty, self_, defined_generics, tcx),
        _ => unimplemented!("{:?}", q_path),
      }
    }
    TyKind::Slice(ty) => None,
    TyKind::OpaqueDef(item, generic_args) => {
      warn!("HIR: Skipping opaquedef of {:?} with generic args {:?}", item, generic_args);
      None
    }
    TyKind::Tup(tys) => {
      let ts = tys.iter().filter_map(|ty| ty_to_t(ty, self_, defined_generics, tcx))
          .map(Box::new)
          .collect::<Vec<_>>();
      if ts.len() != tys.len() {
        warn!("HIR: Could not extract tuple of ({:?})", tys);
        return None;
      }
      Some(T::Tuple(TupleT::new(ts)))
    }
    TyKind::Array(ty, array_length) => {
      warn!("HIR: Skipping array type");
      None
    }
    _ => todo!("Ty kind is: {:?}", &ty.kind),
  }
}

pub fn path_to_generics(path: &rustc_hir::Path<'_>, self_: Option<HirId>, defined_generics: &Vec<T>, tcx: &TyCtxt<'_>) -> Vec<T> {
  let generics = path.segments.iter().filter_map(|s| if let Some(args) = s.args {
    Some(args.args.iter().filter_map(|a| generic_arg_to_t(a, self_, defined_generics, tcx)).collect::<Vec<_>>())
  } else {
    None
  }).flatten().collect::<Vec<_>>();

  generics
}

pub fn path_to_t(
  def_kind: &DefKind,
  def_id: DefId,
  self_: Option<HirId>,
  path: &rustc_hir::Path<'_>,
  defined_generics: &Vec<T>,
  tcx: &TyCtxt<'_>,
) -> Option<T> {
  let name = tcx.def_path_str(def_id);
  let generics = path_to_generics(path, self_, defined_generics, tcx);

  match def_kind {
    DefKind::Enum => {
      Some(T::Enum(EnumT::new(&name, generics, vec![])))
    }
    DefKind::Struct => {
      Some(T::Struct(StructT::new(&name, generics)))
    }
    _ => unimplemented!()
  }
}

pub fn generic_arg_to_t(generic_arg: &GenericArg, self_: Option<HirId>, defined_generics: &Vec<T>, tcx: &TyCtxt<'_>) -> Option<T> {
  match generic_arg {
    GenericArg::Type(ty) => ty_to_t(ty, self_, defined_generics, tcx),
    _ => None
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
      match adt_def.adt_kind() {
        AdtKind::Struct => {
          let t = T::Struct(StructT::new(&name, generics));

          Some(t)
        }
        AdtKind::Union => {
          warn!("HIR: Skipping tuple");
          None
        }
        AdtKind::Enum => {
          let t = T::Enum(EnumT::new(&name, generics, vec![]));
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
      let t = T::Enum(EnumT::new(&name, generics, variants));

      Some(t)
    }
    _ => todo!()
  }
}

pub fn substs_len(substs: SubstsRef) -> usize {
  substs.non_erasable_generics().filter(|kind| match kind {
    GenericArgKind::Type(_) => true,
    _ => false
  }).count()
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
      /*if ty.is_region_ptr() {
          Some(Arc::new(T::Ref(generic_param)))
      } else {
      }*/
    }
    _ => todo!("Ty is {:?}", ty),
  }
}

pub fn generics_to_ts(generics: &Generics, tcx: &TyCtxt<'_>) -> Vec<T> {
  if !generics.where_clause.predicates.is_empty() {
    let predicates = generics.where_clause.predicates;
    let bounds_generic_param = predicates.iter().filter_map(|p| match p {
      WherePredicate::BoundPredicate(p) => Some(p.bound_generic_params.is_empty()),
      _ => None,
    }).any(|bounds| bounds);
    // We do not handle bound generic params in where clause at the moment
    if bounds_generic_param {
      warn!("TODO bound generic params not handled in where clause");
      return vec![];
    }

    let where_generics = predicates.iter().filter_map(|p| match p {
      WherePredicate::BoundPredicate(p) => {
        let defined_generics = vec![];
        debug!("Parsing where bounds: {:?}", &p.bounds);
        let ty = ty_to_t(p.bounded_ty, None, &defined_generics, tcx);
        let bounds = p.bounds.iter().filter_map(|b| generic_bound_to_trait(b, tcx)).collect::<Vec<_>>();
        if let Some(ty) = ty {
          Some((ty, bounds))
        } else {
          None
        }
      }
      _ => None
    }).collect::<Vec<_>>();
    todo!("Where generics: {:?}", &where_generics);
  }

  generics
      .params
      .iter()
      .filter_map(|g| generic_param_to_t(g, tcx))
      .collect::<Vec<_>>()
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

      let name = tcx.def_path_str(def_id);
      Some(Trait::new(&name))
    }
    GenericBound::LangItemTrait(_, _, _, _) => todo!(),
    GenericBound::Outlives(_) => None,
  }
}

pub fn node_to_t(node: &Node<'_>, tcx: &TyCtxt<'_>) -> Option<T> {
  match node {
    Node::Item(item) => item_to_t(item, tcx),
    _ => todo!(),
  }
}

pub fn item_to_t(item: &Item<'_>, tcx: &TyCtxt<'_>) -> Option<T> {
  let generics = vec![];
  match &item.kind {
    ItemKind::Impl(im) => ty_to_t(im.self_ty, Some(item.hir_id()), &generics, tcx),
    _ => todo!(),
  }
}

pub fn fn_ret_ty_to_t(ret_ty: &FnRetTy, self_hir_id: HirId, tcx: &TyCtxt<'_>) -> Option<T> {
  let generics = vec![];
  match ret_ty {
    FnRetTy::DefaultReturn(_) => None,
    FnRetTy::Return(ty) => {
      ty_to_t(ty, Some(self_hir_id), &generics, tcx)
    }
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
    _ => item.ident.name.to_ident_string(),
  }
}

pub fn ty_to_name(ty: &Ty<'_>, tcx: &TyCtxt<'_>) -> String {
  match &ty.kind {
    TyKind::Path(path) => qpath_to_name(path, tcx),
    TyKind::Rptr(_, mut_ty) => ty_to_name(mut_ty.ty, tcx),
    _ => todo!("Trying to convert ty to name: {:?}", ty),
  }
}

pub fn qpath_to_name(qpath: &QPath<'_>, tcx: &TyCtxt<'_>) -> String {
  match qpath {
    QPath::Resolved(_, path) => {
      res_to_name(&path.res, tcx)
    }
    _ => todo!(),
  }
}

pub fn res_to_name(res: &Res, tcx: &TyCtxt<'_>) -> String {
  match res {
    Res::Def(_, def_id) => tcx.def_path_str(*def_id),
    _ => todo!(),
  }
}

pub fn impl_to_def_id(im: &Impl) -> DefId {
  let self_ty = im.self_ty;
  ty_kind_to_def_id(&self_ty.kind)
}

pub fn ty_kind_to_def_id(kind: &TyKind<'_>) -> DefId {
  match kind {
    TyKind::Path(qpath) => match qpath {
      QPath::Resolved(_, path) => path.res.def_id(),
      _ => todo!(),
    },
    TyKind::Rptr(lifetime, mut_ty) => {
      let ty = mut_ty.ty;
      ty_kind_to_def_id(&ty.kind)
    }
    _ => todo!("Trying to convert to struct: {:?}", kind),
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
    DefPathData::ForeignMod => None
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