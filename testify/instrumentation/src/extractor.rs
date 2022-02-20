use log::error;
use rustc_hir::def::Res;
use rustc_hir::TyKind;
use rustc_middle::ty::{TyCtxt, TypeckResults};
use rustc_span::def_id::{DefId, LocalDefId};

/// Convert a hir rustc_hir::Ty to a rustc_middle::ty::Ty
pub fn hir_ty_to_mir_ty<'tcx>(
    ty: &rustc_hir::Ty,
    typeck_results: &TypeckResults,
    tcx: &TyCtxt<'tcx>
) -> rustc_middle::ty::Ty<'tcx> {
    return match &ty.kind {
        TyKind::Path(qpath) => {
            let res = typeck_results.qpath_res(qpath, ty.hir_id);
            error!("---> Res: {:?}", res);
            match res {
                Res::SelfTy(trait_, alias_to) => {
                    if let Some((alias_to, _)) = alias_to {
                        tcx.type_of(alias_to)
                    } else {
                        todo!()
                    }
                }
                _ => todo!("Res is: {:?}", res)
            }
        }
        _ => todo!()
    };
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