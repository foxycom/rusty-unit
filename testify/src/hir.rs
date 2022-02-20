use log::{debug, error, info, warn};
use rustc_data_structures::snapshot_vec::VecLike;
use rustc_hir::def_id::{LocalDefId, LOCAL_CRATE};
use rustc_hir::{AssocItemKind, BodyId, EnumDef, FnDecl, FnRetTy, FnSig, ForeignItem, Generics, HirId, Impl, ImplItem, ImplItemKind, Item, ItemId, ItemKind, QPath, TraitItem, Ty, TyKind, Variant, VariantData, Visibility, VisibilityKind};
use rustc_middle::ty::{DefIdTree, TyCtxt, TypeckResults};
use rustc_span::Span;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use rustc_ast::CrateSugar;
use rustc_ast::visit::walk_crate;
use rustc_data_structures::map_in_place::MapInPlace;
use rustc_hir::def::Res;
use rustc_hir::intravisit::nested_filter::NestedFilter;
use rustc_hir::intravisit::Visitor;
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_middle::hir::nested_filter::{All, OnlyBodies};
use rustc_middle::mir::{Body, HasLocalDecls};
use crate::extractor::{hir_ty_to_t, parent_of_method};
use crate::{HIR_LOG_PATH, LOG_DIR};
use crate::options::RuConfig;
use crate::types::{Callable, EnumInitItem, EnumVariant, FieldAccessItem, FunctionItem, generics_of_item, MethodItem, mir_ty_to_t, Param, StaticFnItem, StructInitItem, T};

use crate::writer::HirWriter;
use crate::util::{def_id_to_enum, def_id_to_struct, fn_ret_ty_to_t, generics_to_ts, hir_ty_to_t_unprecise, impl_to_struct_id, item_to_name, node_to_name, span_to_path, ty_to_param};

struct HirVisitor<'tcx> {
    types: Vec<String>,
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> HirVisitor<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            types: vec![],
            tcx,
        }
    }
}

impl<'tcx> Visitor<'tcx> for HirVisitor<'tcx> {
    type NestedFilter = All;

    fn nested_visit_map(&mut self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_impl_item(&mut self, ii: &'tcx ImplItem<'tcx>) {
        let typeck_results = TypeckResults::new(ii.def_id);

        let is_public = ii.vis.node.is_pub();

        match &ii.kind {
            ImplItemKind::Fn(fn_sig, _) => {
                error!("--> Fn {}", ii.ident.name.to_ident_string());

                let name = ii.ident.name.to_string();
                let params: Vec<Param> = vec![];

                info!("Parsing parent");
                let parent_ty = parent_of_method(ii.def_id.to_def_id(), &self.tcx);
                let parent_t = mir_ty_to_t(parent_ty, &self.tcx);

                info!("Parsing method generics");
                let method_generics = generics_of_item(ii.def_id.to_def_id(), &self.tcx);

                let return_t = match &fn_sig.decl.output {
                    FnRetTy::Return(ret_ty) => {
                        info!("Parsing return type");
                        //let ret_t = hir_ty_to_t_(ret_ty, &typeck_results, &self.tcx);
                        let ret_t = hir_ty_to_t_unprecise(ret_ty, &self.tcx);
                        Some(ret_t)
                    }
                    _ => None
                };

                let params = fn_sig.decl.inputs.iter().map(|i| {
                    info!("Parsing param");
                    let param_t = hir_ty_to_t(i, &typeck_results, &self.tcx);
                    Param::new(None, param_t, false)
                }).collect::<Vec<_>>();

                let file_path = span_to_path(&ii.span, &self.tcx).expect("File path unknown");

                let method = MethodItem::new(&name, file_path.to_str().unwrap(), params, return_t, parent_t, method_generics, is_public);
                info!("Extracted method: {:?}", method);
            }
            _ => {}
        }
    }
}

impl<'hir> ItemLikeVisitor<'hir> for HirVisitor<'_> {
    fn visit_item(&mut self, item: &'hir Item<'hir>) {
        Visitor::visit_nested_item(self, item.item_id());
    }

    fn visit_trait_item(&mut self, trait_item: &'hir TraitItem<'hir>) {}

    fn visit_impl_item(&mut self, impl_item: &'hir ImplItem<'hir>) {}

    fn visit_foreign_item(&mut self, foreign_item: &'hir ForeignItem<'hir>) {}
}

fn analyze_tcx(tcx: &TyCtxt<'_>) {
    let mut visitor = HirVisitor::new(tcx.clone());
    tcx.hir().visit_all_item_likes(&mut visitor);

    /*for key in tcx.mir_keys(()) {
        let def_id = key.to_def_id();
        if tcx.def_path_str(def_id).contains("rusty_monitor") {
            continue;
        }

         info!("ANALYSIS: Mir key is {:?}",  def_id);
         let impl_of_method = tcx.impl_of_method(def_id);
         if let Some(impl_of_method) = impl_of_method {
             info!("Is method of impl {:?}", impl_of_method);
             info!("Item name: {}", tcx.item_name(def_id));
             let mir: &Body<'_> = tcx.optimized_mir(def_id);

             let generics = tcx.generics_of(def_id);
             if !generics.params.is_empty() {
                 for param in generics.params.iter() {
                     info!("Generic is: {:?}", param);
                     info!("Bounds: {:?}", tcx.item_bounds(def_id));
                 }
                 info!("Bounds")
             }
             let mut i = 1;
             info!("Return type is: {:?}", mir.return_ty().kind());
         }
    }*/
}

fn get_params(body: &Body<'_>) -> Vec<Param> {
    //let params = Vec::with_capacity(body.arg_count);
    for local in body.args_iter() {
        let arg = body.local_decls().get(local).unwrap();
    }

    todo!()
}

pub fn hir_analysis(tcx: TyCtxt<'_>) {
    analyze_tcx(&tcx);

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

    //let mut analysis = HirAnalysis::new();
    //analysis.set_callables(callables);

    let hir_output_path = Path::new(LOG_DIR).join(HIR_LOG_PATH);
    let mut writer = HirWriter::new(hir_output_path);
    //writer.write_analysis(&analysis);
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

    // TODO a fn can also have explicit generics defined
    let generics = vec![];

    // self_hir_id must never be used, so just pass a dummy value
    let mut params = Vec::with_capacity(fn_decl.inputs.len());
    for input in fn_decl.inputs.iter() {
        params.push(ty_to_param(None, input, tcx));
    }

    let return_type = fn_ret_ty_to_t(&fn_decl.output, tcx);

    if let Some(return_type) = &return_type {
        debug!("HIR: Output type is {:?}", return_type);
    }

    let name = tcx.hir().name(hir_id).to_ident_string();

    let function_item = FunctionItem::new(
        is_public,
        &name,
        generics,
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

        let variant = extract_enum_variant(variant, &enum_generics, tcx);
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

fn extract_enum_variant(variant: &Variant, generics: &Vec<T>, tcx: &TyCtxt<'_>) -> Option<EnumVariant> {
    match &variant.data {
        VariantData::Struct(fields, _) => {
            let ctor_hir_id = variant.data.ctor_hir_id().unwrap();
            let def_id = tcx.hir().local_def_id(ctor_hir_id).to_def_id();
            let struct_type = def_id_to_struct(def_id, tcx).unwrap();
            let struct_name = node_to_name(&tcx.hir().get(ctor_hir_id), tcx).unwrap();
            let v = EnumVariant::Struct(variant.ident.name.to_ident_string(), Param::new(Some(&struct_name), struct_type, false));
            Some(v)
        }
        VariantData::Tuple(fields, variant_hir_id) => {
            debug!("--> ENUM variant extracting {:?}", variant_hir_id);
            let params = fields.iter()
                .map(|f| hir_ty_to_t_unprecise(&f.ty, tcx))
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
    let is_public = is_public(vis);

    let struct_generics = generics_to_ts(g, tcx);
    let struct_hir_id = tcx.hir().local_def_id_to_hir_id(struct_local_def_id);
    match vd {
        VariantData::Struct(fields, _) => {
            let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();
            let parent = def_id_to_struct(def_id, tcx).unwrap();
            let parent_name = node_to_name(&tcx.hir().get(struct_hir_id), tcx).unwrap();
            if parent_name.contains("serde") {
                // Skip too hard stuff
                return;
            }

            for field in fields.iter() {
                let ty = hir_ty_to_t_unprecise(field.ty, tcx);
                let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();

                debug!("HIR: Extracted field access item with def id {:?}", def_id);

                let name = tcx.hir().name(field.hir_id).to_ident_string();
                let parent = def_id_to_struct(def_id, tcx).unwrap();
                let field_item = FieldAccessItem::new(
                    &name,
                    file_path.to_str().unwrap(),
                    ty,
                    parent,
                    is_public,
                );
            }

            let mut params = Vec::with_capacity(fields.len());
            for field in fields.iter() {
                let name = field.ident.name.to_ident_string();
                params.push(ty_to_param(Some(&name), field.ty, tcx));
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

    let parent_def_id = impl_to_struct_id(im);

    let impl_generics = generics_to_ts(&im.generics, tcx);

    let parent_hir_id = tcx
        .hir()
        .local_def_id_to_hir_id(parent_def_id.expect_local());
    let def_id = tcx.hir().local_def_id(parent_hir_id).to_def_id();
    let parent = def_id_to_struct(def_id, tcx).unwrap();

    let items = im.items;

    debug!("HIR: Impl generics:\n{:?}", impl_generics);
    for item in items {
        let def_id = item.id.def_id;
        let typeck_results = TypeckResults::new(def_id);

        match &item.kind {
            AssocItemKind::Fn { has_self } => {
                let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);
                let impl_item = tcx.hir().impl_item(item.id);
                match &impl_item.kind {
                    ImplItemKind::Fn(sig, body_id) => {
                        debug!(
                            "HIR: (!) Found method {}, parent: {}",
                            &impl_item.ident, parent_hir_id
                        );

                        let return_type = fn_ret_ty_to_t(&sig.decl.output, tcx);

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
                            params.push(ty_to_param(None, input, tcx));
                        }

                        let name = tcx.hir().name(hir_id).to_ident_string();
                        if *has_self {
                            // Method
                            debug!("HIR: Method is associative");
                            let method_item = MethodItem::new(
                                &name,
                                file_path.to_str().unwrap(),
                                params,
                                return_type,
                                parent.clone(),
                                overall_generics.clone(),
                                is_public,
                            );
                            let method_callable = Callable::Method(method_item);
                            callables.push(method_callable);
                        } else {
                            // Associative function
                            debug!("HIR: Method is static");
                            let static_method_item = StaticFnItem::new(
                                &name,
                                file_path.to_str().unwrap(),
                                params,
                                return_type,
                                parent.clone(),
                                overall_generics.clone(),
                                is_public,
                            );
                            let static_method_callable =
                                Callable::StaticFunction(static_method_item);
                            callables.push(static_method_callable);
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
