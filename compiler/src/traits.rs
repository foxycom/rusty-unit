use std::collections::HashSet;
use std::iter;
use log::error;
use rustc_middle::ty;
use rustc_middle::ty::{Ty, TyCtxt, TyKind};
use rustc_span::def_id::DefId;

pub fn analyze_trait(trait_id: DefId, tcx: &TyCtxt<'_>) {
  let crates = tcx.crates(());
  let impls = tcx.all_impls(trait_id);
  let lang_items = tcx.lang_items();


  for i in impls {
    let t = tcx.type_of(i);
    if is_generic_type(&t) {
      let mut super_traits = super_traits_of(trait_id, tcx);
      super_traits.retain(|e| *e != trait_id);
      for super_trait in super_traits {
        analyze_trait(super_trait, tcx);
      }
    }

  }
  // for crate_num in crates {
  //   let impls = tcx.all_impls((*crate_num, trait_id));
  //   if impls.is_empty() {
  //     continue;
  //   }
  //   error!("Trait {:?} -> {:?}", trait_id, impls);
  // }
}

fn is_generic_type(ty: &Ty<'_>) -> bool {
  match ty.kind() {
    TyKind::Param(_) => true,
    _ => false
  }
}

fn super_traits_of(trait_def_id: DefId, tcx: &TyCtxt) -> Vec<DefId> {
  let mut set = std::collections::HashSet::new();
  let mut stack = vec![trait_def_id];

  set.insert(trait_def_id);

  iter::from_fn(move || -> Option<DefId> {
    let trait_did = stack.pop()?;
    let generic_predicates = tcx.super_predicates_of(trait_did);

    for (predicate, _) in generic_predicates.predicates {
      if let ty::PredicateKind::Trait(data) = predicate.kind().skip_binder() {
        if set.insert(data.def_id()) {
          stack.push(data.def_id());
        }
      }
    }

    Some(trait_did)
  }).collect::<Vec<_>>()
}