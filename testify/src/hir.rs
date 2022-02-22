use std::path::{Path, PathBuf};
use log::{debug, info, warn};
use rustc_ast::CrateSugar;
use rustc_hir::{AssocItemKind, BodyId, EnumDef, FnSig, Generics, HirId, Impl, ImplItemKind, Item, ItemKind, Variant, VariantData, Visibility, VisibilityKind};
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::{LOCAL_CRATE, LocalDefId};
use rustc_span::Span;
use crate::{HIR_LOG_PATH, LOG_DIR, RuConfig};
use crate::types::{Callable, EnumInitItem, EnumVariant, FieldAccessItem, FunctionItem, MethodItem, Param, StaticFnItem, StructInitItem, T};
use crate::util::{def_id_to_complex, def_id_to_enum, fn_ret_ty_to_t, generics_to_ts, impl_to_def_id, item_to_name, node_to_name, span_to_path, ty_to_param, ty_to_t};
use crate::writer::HirWriter;
use crate::analysis::Analysis;

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

    info!("HIR: Scanning file {:?}", file_path.as_ref());
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
      _ => {}
    }
  }

  let hir_output_path = Path::new(LOG_DIR).join(HIR_LOG_PATH);
  let mut analysis = Analysis::new(callables);
  analysis.to_file(hir_output_path);
}

fn allowed_item(item: &Item<'_>, tcx: &TyCtxt<'_>) -> bool {
  let item_name = item_to_name(item, &tcx);
  !item_name.contains("serde") && !item_name.contains("test")
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

  let fn_name = tcx.hir().get(hir_id).ident().unwrap().to_string();

  // TODO a fn can also have explicit generics defined
  let generics = vec![];

  // self_hir_id must never be used, so just pass a dummy value
  let mut params = Vec::with_capacity(fn_decl.inputs.len());
  for input in fn_decl.inputs.iter() {
    if let Some(param) = ty_to_param(None, input, hir_id, &generics, tcx) {
      params.push(param);
    } else {
      return;
    }
  }

  let return_type = fn_ret_ty_to_t(&fn_decl.output, hir_id, tcx);

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
  generics: &Generics,
  visibility: &Visibility,
  file_path: PathBuf,
  callables: &mut Vec<Callable>,
  tcx: &TyCtxt<'_>,
) {
  let is_public = is_public(visibility);
  let enum_generics = generics_to_ts(generics, tcx);
  let enum_hir_id = tcx.hir().local_def_id_to_hir_id(enum_local_def_id);
  let enum_def_id = tcx.hir().local_def_id(enum_hir_id).to_def_id();

  let parent = def_id_to_enum(enum_def_id, tcx).unwrap();
  let parent_name = node_to_name(&tcx.hir().get(enum_hir_id), tcx).unwrap();
  if parent_name.contains("serde") {
    // Skip too hard stuff
    return;
  }

  for variant in enum_def.variants {
    let variant_name = variant.ident.name.to_ident_string();

    let variant = extract_enum_variant(variant, enum_hir_id, &enum_generics, tcx);
    if let Some(variant) = variant {
      debug!("HIR: Extracted enum variant {}::{}", &parent_name, &variant_name);
      let enum_init = Callable::EnumInit(EnumInitItem::new(
        file_path.to_str().unwrap(),
        variant,
        parent.clone(),
        is_public,
      ));
      callables.push(enum_init);
    } else {
      warn!("HIR: Could not extract enum variant {}::{}", &parent_name, &variant_name);
    }
  }
}

fn extract_enum_variant(variant: &Variant, hir_id: HirId, generics: &Vec<T>, tcx: &TyCtxt<'_>) -> Option<EnumVariant> {
  match &variant.data {
    VariantData::Struct(fields, _) => {
      let ctor_hir_id = variant.data.ctor_hir_id().unwrap();
      let def_id = tcx.hir().local_def_id(ctor_hir_id).to_def_id();
      let struct_type = def_id_to_complex(def_id, tcx).unwrap();
      let struct_name = node_to_name(&tcx.hir().get(ctor_hir_id), tcx).unwrap();
      let v = EnumVariant::Struct(variant.ident.name.to_ident_string(), crate::types::Param::new(Some(&struct_name), struct_type, false));
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
  //let adt_def = tcx.adt_def(struct_local_def_id.to_def_id());
  let is_public = is_public(vis);

  let struct_generics = generics_to_ts(g, tcx);
  let struct_hir_id = tcx.hir().local_def_id_to_hir_id(struct_local_def_id);
  match vd {
    VariantData::Struct(fields, _) => {
      let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();
      let parent = def_id_to_complex(def_id, tcx).unwrap();
      let parent_name = node_to_name(&tcx.hir().get(struct_hir_id), tcx).unwrap();
      if parent_name.contains("serde") {
        // Skip too hard stuff
        return;
      }

      for field in fields.iter() {
        let ty = ty_to_t(field.ty, Some(struct_hir_id), &struct_generics, tcx);
        if let Some(ty) = ty {
          let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();

          let field_name = tcx.hir().get(field.hir_id).ident().unwrap().to_string();
          debug!("HIR: Extracted field {}::{}", &parent_name, &field_name);

          let parent = def_id_to_complex(def_id, tcx).unwrap();
          let field_item = FieldAccessItem::new(
            &field_name,
            file_path.to_str().unwrap(),
            ty,
            parent,
            is_public,
          );
        }
      }

      let mut params = Vec::with_capacity(fields.len());
      for field in fields.iter() {
        let name = field.ident.name.to_ident_string();
        let param =
            ty_to_param(Some(&name), field.ty, struct_hir_id, &struct_generics, tcx);
        if let Some(param) = param {
          params.push(param);
        } else {
          // An unknown type, ignore function
          return;
        }
      }

      debug!("HIR: Extracted struct init {}: {:?}", parent, params);
      callables.push(Callable::StructInit(StructInitItem::new(
        file_path.to_str().unwrap(),
        params,
        parent,
      )));
    }
    _ => {}
  }
}

fn analyze_impl(im: &Impl, file_path: PathBuf, callables: &mut Vec<Callable>, tcx: &TyCtxt<'_>) {
  if let Some(_) = im.of_trait {
    // Skip trait implementation for now
    return;
  }

  let parent_def_id = impl_to_def_id(im);

  let impl_generics = generics_to_ts(&im.generics, tcx);

  let parent_hir_id = tcx
      .hir()
      .local_def_id_to_hir_id(parent_def_id.expect_local());
  let def_id = tcx.hir().local_def_id(parent_hir_id).to_def_id();
  let parent = def_id_to_complex(def_id, tcx).unwrap();

  let items = im.items;

  for item in items {
    let def_id = item.id.def_id;

    match &item.kind {
      AssocItemKind::Fn { .. } => {
        let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);
        let impl_item = tcx.hir().impl_item(item.id);
        match &impl_item.kind {
          ImplItemKind::Fn(sig, body_id) => {
            debug!(
                            "HIR: Found method {}, parent: {}",
                            &impl_item.ident, parent_hir_id
                        );

            let fn_name = tcx.hir().get(item.id.hir_id()).ident().unwrap().to_string();

            let return_type = fn_ret_ty_to_t(&sig.decl.output, parent_hir_id, tcx);
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

            let mut fn_generics = generics_to_ts(&impl_item.generics, tcx);
            let mut overall_generics = impl_generics.clone();
            overall_generics.append(&mut fn_generics);

            let mut params = Vec::with_capacity(sig.decl.inputs.len());
            for input in sig.decl.inputs.iter() {
              let param =
                  ty_to_param(None, input, parent_hir_id, &overall_generics, tcx);
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
                parent.clone(),
                impl_generics.clone(),
                is_public,
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
                parent.clone(),
                impl_generics.clone(),
                is_public,
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