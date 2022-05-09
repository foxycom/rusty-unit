use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use log::{debug, info, warn};
use petgraph::visit::Walker;
use rustc_ast::CrateSugar;
use rustc_data_structures::undo_log::UndoLogs;
use rustc_hir::{AssocItemKind, BodyId, EnumDef, FnSig, Generics, HirId, Impl, ImplItemKind, Item, ItemKind, Variant, VariantData, Visibility, VisibilityKind};
use rustc_middle::ty::{Ty, TyCtxt};
use rustc_span::def_id::{DefId, LOCAL_CRATE, LocalDefId};
use rustc_span::Span;
use crate::{HIR_LOG_PATH, LOG_DIR, RuConfig};
use crate::types::{Callable, def_id_name, EnumInitItem, EnumT, EnumVariant, FieldAccessItem, FunctionItem, MethodItem, Param, StaticFnItem, StructInitItem, StructT, T, Trait};
use crate::util::{def_id_to_t, def_id_to_enum, fn_ret_ty_to_t, generics_to_ts, impl_to_def_id, item_to_name, node_to_name, span_to_path, ty_to_param, ty_to_t, is_local, res_to_name, path_to_name};
use crate::analysis::Analysis;
#[cfg(feature = "analysis")]
use crate::writer::{HirObject, HirWriter, HirObjectBuilder};

#[cfg(feature = "analysis")]
pub fn hir_analysis(tcx: TyCtxt<'_>) {
  let current_crate_name = tcx.crate_name(LOCAL_CRATE);
  if current_crate_name.as_str() != RuConfig::env_crate_name() {
    return;
  }


  let mut callables = vec![];
  for item in tcx.hir().items() {
    let def_id = item.def_id.to_def_id();

    let span: &Span = &item.span;
    let file_path = span_to_path(span, &tcx);
    if file_path.is_none() {
      continue;
    }

    //info!("HIR: Scanning file {:?}", file_path.as_ref());
    if file_path.as_ref().unwrap().ends_with("rusty_monitor.rs") {
      continue;
    }

    match &item.kind {
      ItemKind::Fn(sig, generics, body_id) => {
        if &item.ident.name.to_string() != "main" && allowed_item(item, &tcx) {
          info!("HIR: Analyzing function {}", item_to_name(item, &tcx));
          analyze_fn(
            sig,
            item.def_id,
            body_id,
            &item.vis,
            file_path.unwrap(),
            &mut callables,
            &tcx,
          )
        }
      }
      ItemKind::Impl(im) => {
        if allowed_item(item, &tcx) {
          info!("HIR: Analyzing impl {}", item_to_name(item, &tcx));
          analyze_impl(im, file_path.unwrap(), &mut callables, &tcx)
        }
      }
      ItemKind::Struct(s, g) => {
        if allowed_item(item, &tcx) {
          info!("HIR: Analyzing struct {}", item_to_name(item, &tcx));
          analyze_struct(
            item.def_id,
            s,
            g,
            &item.vis,
            file_path.unwrap(),
            &mut callables,
            &tcx,
          );
        }
      }
      ItemKind::Enum(enum_def, g) => {
        if allowed_item(item, &tcx) {
          info!("HIR: Analyzing enum {}", item_to_name(item, &tcx));
          analyze_enum(
            item.def_id,
            enum_def,
            g,
            &item.vis,
            file_path.unwrap(),
            &mut callables,
            &tcx,
          );
        }
      }
      //ItemKind::Mod(e)
      _ => {}
    }
  }

  // let hir_output_path = Path::new(LOG_DIR).join(HIR_LOG_PATH);
  // let content = serde_json::to_string(&callables).unwrap();
  //
  // #[cfg(file_writer)]
  // FileWriter::new(hir_output_path).write(&content).unwrap();
  //
  // #[cfg(redis_writer)]
  // todo!()

  let hir_object: HirObject = HirObjectBuilder::default()
      .callables(callables)
      .impls(HashMap::new())
      .build()
      .unwrap();
  HirWriter::write(&hir_object);
}

fn allowed_item(item: &Item<'_>, tcx: &TyCtxt<'_>) -> bool {
  let item_name = item_to_name(item, &tcx).to_lowercase();
  !item_name.contains("serde") && !item_name.contains("test") && !item_name.contains("snafu")
}

fn analyze_fn(
  sig: &FnSig,
  local_def_id: LocalDefId,
  body_id: &BodyId,
  vis: &Visibility<'_>,
  file_path: PathBuf,
  callables: &mut Vec<Callable>,
  tcx: &TyCtxt<'_>,
) {
  let hir_id = tcx.hir().local_def_id_to_hir_id(local_def_id);
  let is_public = is_public(vis);
  let fn_decl = &sig.decl;

  let fn_name = node_to_name(&tcx.hir().get(hir_id), tcx).unwrap();
  //let fn_name = tcx.hir().get(hir_id).ident().unwrap().to_string();

  let generics = vec![];

  // self_hir_id must never be used, so just pass a dummy value
  let mut params = Vec::with_capacity(fn_decl.inputs.len());
  for input in fn_decl.inputs.iter() {
    if let Some(param) = ty_to_param(None, input, None, &generics, tcx) {
      params.push(param);
    } else {
      return;
    }
  }

  let return_type = fn_ret_ty_to_t(&fn_decl.output, None, &generics, tcx);

  if let Some(return_type) = &return_type {
    debug!("HIR: Output type is {:?}", return_type);
  }

  let function_item = FunctionItem::new(
    is_public,
    &fn_name,
    vec![],
    params,
    return_type,
    file_path.to_str().unwrap(),
  );
  let fn_callable = Callable::Function(function_item);
  callables.push(fn_callable);
}

fn analyze_enum(
  enum_local_def_id: LocalDefId,
  enum_def: &EnumDef,
  g: &Generics,
  visibility: &Visibility,
  file_path: PathBuf,
  callables: &mut Vec<Callable>,
  tcx: &TyCtxt<'_>,
) {
  let is_public = is_public(visibility);
  let generics = generics_to_ts(g, tcx);
  let enum_hir_id = tcx.hir().local_def_id_to_hir_id(enum_local_def_id);
  let enum_def_id = tcx.hir().local_def_id(enum_hir_id).to_def_id();
  let self_name = node_to_name(&tcx.hir().get(enum_hir_id), tcx).unwrap();

  //let self_ty = def_id_to_enum(enum_def_id, tcx).unwrap();
  let self_ty = T::Enum(EnumT::new(&self_name, generics.clone(), vec![], is_local(enum_def_id)));
  if self_name.contains("serde") {
    // Skip too hard stuff
    return;
  }

  for variant in enum_def.variants {
    let variant_name = variant.ident.name.to_ident_string();

    let variant = extract_enum_variant(variant, enum_hir_id, &generics, tcx);
    if let Some(variant) = variant {
      debug!("HIR: Extracted enum variant {}::{}", &self_name, &variant_name);
      let enum_init = Callable::EnumInit(EnumInitItem::new(
        file_path.to_str().unwrap(),
        variant,
        self_ty.clone(),
        is_public,
      ));
      callables.push(enum_init);
    } else {
      warn!("HIR: Could not extract enum variant {}::{}", &self_name, &variant_name);
    }
  }
}

fn extract_enum_variant(variant: &Variant, hir_id: HirId, generics: &Vec<T>, tcx: &TyCtxt<'_>) -> Option<EnumVariant> {
  match &variant.data {
    VariantData::Struct(fields, _) => {
      let def_id = tcx.hir().local_def_id(variant.id).to_def_id();
      let struct_name = node_to_name(&tcx.hir().get(variant.id), tcx).unwrap();
      let params = fields.iter().filter_map(|f|
          ty_to_param(Some(f.ident.as_str()),
                      f.ty,
                      None,
                      &vec![],
                      tcx,
          )).collect::<Vec<_>>();

      if params.len() != fields.len() {
        warn!("Could not extract enum variant: {}", struct_name);
        return None;
      }

      let v = EnumVariant::Struct(variant.ident.name.to_ident_string(), params);
      Some(v)
    }
    VariantData::Tuple(fields, variant_hir_id) => {
      debug!("--> ENUM variant extracting {:?}", variant_hir_id);
      let params = fields.iter()
          .filter_map(|f| ty_to_t(&f.ty, None, generics, tcx))
          .map(|ty| Param::new(None, ty, false))
          .collect::<Vec<_>>();
      if params.len() < fields.len() {
        return None;
      }

      let v = EnumVariant::Tuple(variant.ident.name.to_ident_string(), params);
      Some(v)
    }
    VariantData::Unit(_) => Some(EnumVariant::Unit(variant.ident.name.to_ident_string()))
  }
}

fn analyze_struct(
  struct_local_def_id: LocalDefId,
  vd: &VariantData,
  g: &Generics,
  vis: &Visibility<'_>,
  file_path: PathBuf,
  callables: &mut Vec<Callable>,
  tcx: &TyCtxt<'_>,
) {
  let mut struct_is_public = is_public(vis);

  let struct_generics = generics_to_ts(g, tcx);
  let struct_hir_id = tcx.hir().local_def_id_to_hir_id(struct_local_def_id);
  match vd {
    VariantData::Struct(fields, _) => {
      let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();
      //let self_ty = def_id_to_t(def_id, tcx).unwrap();
      let self_name = node_to_name(&tcx.hir().get(struct_hir_id), tcx).unwrap();
      info!("HIR: {} is public: {}", self_name, struct_is_public);
      let generics = generics_to_ts(g, tcx);
      let self_ty = T::Struct(StructT::new(&self_name, generics, is_local(def_id)));
      if self_name.contains("serde") {
        // Skip too hard stuff
        return;
      }

      for field in fields.iter() {
        let ty = ty_to_t(field.ty, Some(&self_ty), &struct_generics, tcx);
        if let Some(ty) = ty {
          let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();

          let field_name = tcx.hir().get(field.hir_id).ident().unwrap().to_string();
          debug!("HIR: Extracted field {}::{}", &self_name, &field_name);

          /*let parent = def_id_to_t(def_id, tcx).unwrap();
          let field_item = FieldAccessItem::new(
            &field_name,
            file_path.to_str().unwrap(),
            ty,
            parent,
            is_public,
          );*/
        }

        let field_is_public = is_public(&field.vis);
        if !field_is_public {
          struct_is_public = false;
        }
      }

      let mut params = Vec::with_capacity(fields.len());
      for field in fields.iter() {
        let name = field.ident.name.to_ident_string();
        let param =
            ty_to_param(Some(&name), field.ty, Some(&self_ty), &struct_generics, tcx);
        if let Some(param) = param {
          params.push(param);
        } else {
          // An unknown type, ignore function
          return;
        }
      }

      debug!("HIR: Extracted struct init {}: {:?}", self_ty, params);
      callables.push(Callable::StructInit(StructInitItem::new(
        struct_is_public,
        file_path.to_str().unwrap(),
        params,
        self_ty,
      )));
    }
    _ => {}
  }
}

fn analyze_impl(im: &Impl, file_path: PathBuf, callables: &mut Vec<Callable>, tcx: &TyCtxt<'_>) {
  if let Some(_) = &im.of_trait {
    return;
  }
  let parent_def_id_opt = impl_to_def_id(im);
  
  if let Some(parent_def_id) = parent_def_id_opt {
    if parent_def_id.as_local().is_none() {
      return;
    }
  } else {
    return;
  }

  let trait_name = im.of_trait.as_ref().map(|trait_ref| path_to_name(&trait_ref.path, tcx));

  let parent_def_id = parent_def_id_opt.unwrap();

  let parent_hir_id = tcx
      .hir()
      .local_def_id_to_hir_id(parent_def_id.expect_local());
  //let parent = def_id_to_t(parent_def_id, tcx).unwrap();

  let impl_generics = generics_to_ts(&im.generics, tcx);
  let self_ty = ty_to_t(im.self_ty, None, &impl_generics, tcx).unwrap();

  let items = im.items;
  for item in items {
    let item_def_id = item.id.def_id;

    match &item.kind {
      AssocItemKind::Fn { .. } => {
        let hir_id = tcx.hir().local_def_id_to_hir_id(item_def_id);
        let impl_item = tcx.hir().impl_item(item.id);
        match &impl_item.kind {
          ImplItemKind::Fn(sig, body_id) => {
            debug!("HIR: Found method {}, parent: {}", &impl_item.ident, parent_hir_id);
            let fn_name = tcx.hir().get(item.id.hir_id()).ident().unwrap().to_string();
            let fn_generics = generics_to_ts(&impl_item.generics, tcx);

            let mut overall_generics = impl_generics.clone();
            overall_generics.append(&mut fn_generics.clone());

            let return_type = fn_ret_ty_to_t(&sig.decl.output, Some(&self_ty), &overall_generics, tcx);
            if let Some(return_type) = return_type.as_ref() {
              debug!("HIR: Return type is {:?}", &return_type);
            }

            let is_public = is_public(&impl_item.vis);
            let parent_name = node_to_name(&tcx.hir().get(parent_hir_id), tcx).unwrap();
            if parent_name.contains("serde") {
              // Skip too hard stuff
              warn!("HIR: Skipping serde method");
              continue;
            }

            let mut params = Vec::with_capacity(sig.decl.inputs.len());
            for input in sig.decl.inputs.iter() {
              let param =
                  ty_to_param(None, input, Some(&self_ty), &overall_generics, tcx);
              if let Some(param) = param {
                debug!("HIR: Extracting parameter {:?}", param);
                params.push(param);
              } else {
                // An unknown type, ignore function
                warn!("HIR: Unknown parameter, skipping method.");
                return;
              }
            }

            if !sig.decl.implicit_self.has_implicit_self() {
              // Static method
              debug!("HIR: Method is static");
              let static_method_item = StaticFnItem::new(
                &fn_name,
                file_path.to_str().unwrap(),
                params,
                return_type,
                self_ty.clone(),
                fn_generics,
                is_public,
                trait_name.clone()
              );
              let static_method_callable =
                  Callable::StaticFunction(static_method_item);
              callables.push(static_method_callable);
            } else {
              // Dynamic method
              debug!("HIR: Method is associative");
              let method_item = MethodItem::new(
                &fn_name,
                file_path.to_str().unwrap(),
                params,
                return_type,
                self_ty.clone(),
                fn_generics,
                is_public,
                trait_name.clone()
              );
              let method_callable = Callable::Method(method_item);
              callables.push(method_callable);
            }
          }
          _ => {}
        }
      }
      _ => {}
    }
  }
}

fn is_public(vis: &Visibility<'_>) -> bool {
  match &vis.node {
    VisibilityKind::Public => true,
    VisibilityKind::Crate(sugar) => match sugar {
      CrateSugar::PubCrate => true,
      _ => false,
    },
    _ => false,
  }
}




