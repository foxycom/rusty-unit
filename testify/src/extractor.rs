use log::{error, info};
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{QPath, TyKind};
use rustc_middle::ty::{TyCtxt, TypeckResults};
use rustc_span::def_id::{DefId, LocalDefId};
use crate::types::{def_id_name, Generic, generics_of_item, mir_ty_to_t, StructT, T};

/// Convert a hir rustc_hir::Ty to a T
pub fn hir_ty_to_t(
  ty: &rustc_hir::Ty,
  typeck_results: &TypeckResults,
  tcx: &TyCtxt<'_>,
) -> T {
  return match &ty.kind {
    TyKind::Path(qpath) => {
      let res = typeck_results.qpath_res(qpath, ty.hir_id);
      match res {
        Res::SelfTy(trait_, alias_to) => {
          info!("Self ty");
          if let Some((alias_to, _)) = alias_to {
            let mir_ty = tcx.type_of(alias_to);
            mir_ty_to_t(mir_ty, tcx)
          } else {
            todo!()
          }
        }
        Res::Def(def_kind, def_id) => {
          info!("Def ty: {:?}", def_id);
          res_def_to_t(&def_kind, def_id, tcx)
        }
        Res::PrimTy(prim_ty) => {
          T::from(prim_ty)
        }
        _ => todo!("Res is: {:?}", res)
      }
    }
    TyKind::Rptr(_, mut_ty) => {
      hir_ty_to_t(mut_ty.ty, typeck_results, tcx)
    }
    _ => todo!("{:?}", &ty.kind)
  };
}

fn res_def_to_t(def_kind: &DefKind, def_id: DefId, tcx: &TyCtxt<'_>) -> T {
  match def_kind {
    DefKind::TyParam => {
      let bounds = vec![];
      let name = def_id_name(def_id, tcx);
      T::Generic(Generic::new(&name, bounds))
    }
    DefKind::Struct => {
      let mir_ty = tcx.type_of(def_id);
      info!("Is struct: {:?}", mir_ty);

      let generics = generics_of_item(def_id, tcx);
      let mut t = mir_ty_to_t(mir_ty, tcx);

      t.overwrite_generics(generics);
      t
    }
    _ => todo!("{:?}", def_kind)
  }
}

/// Returns the type an method is associated with, e.g., struct or enum
///
/// # Arguments
///
/// * `def_id`: Def id of the method
///
/// returns: ()
///
/// # Examples
///
/// ```
///
/// ```
pub fn parent_of_method<'tcx>(def_id: DefId, tcx: &TyCtxt<'tcx>) -> rustc_middle::ty::Ty<'tcx> {
  let impl_def_id = tcx.impl_of_method(def_id).expect("Not a method");
  tcx.type_of(impl_def_id)
}