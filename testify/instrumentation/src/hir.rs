use rustc_hir::def_id::{LocalDefId, LOCAL_CRATE};
use rustc_hir::{AssocItemKind, BodyId, FnSig, Generics, Impl, ImplItemKind, Item, ItemKind, VariantData, Visibility, VisibilityKind};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use log::{debug, error, info, warn};
use rustc_ast::CrateSugar;

use crate::util::{get_cut_name, get_testify_flags};
use crate::writer::HirWriter;
use generation::analysis::HirAnalysis;
use generation::types::{
    Callable, FieldAccessItem, FunctionItem, MethodItem,
    StaticFnItem, StructInitItem,
};
use generation::util::{
    def_id_to_complex, fn_ret_ty_to_t, generics_to_ts, impl_to_struct_id, item_to_name,
    node_to_name, span_to_path, ty_to_param, ty_to_t,
};
use generation::HIR_LOG_PATH;
lazy_static! {
    pub static ref SOURCE_FILE_MAP: Arc<Mutex<HashMap<PathBuf, usize>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub fn hir_analysis(tcx: TyCtxt<'_>) {
    let testify_flags = get_testify_flags();
    let cut_name = get_cut_name(testify_flags.as_ref());
    let current_crate_name = tcx.crate_name(LOCAL_CRATE);
    if current_crate_name.as_str() != cut_name {
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
                if &item.ident.name.to_string() != "main" && allowed_item(item, &tcx){
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
                    analyze_struct(item.def_id, s, g, &item.vis, file_path.unwrap(), &mut callables, &tcx);
                }
            }
            _ => {}
        }
    }

    let mut analysis = HirAnalysis::new();
    analysis.set_callables(callables);

    let mut writer = HirWriter::new(HIR_LOG_PATH);
    writer.write_analysis(&analysis);
}

fn allowed_item(item: &Item<'_>, tcx: &TyCtxt<'_>) -> bool {
    let item_name = item_to_name(item, &tcx);
    !item_name.contains("serde")
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
        file_path.to_str().unwrap(),
        params,
        return_type,
        is_public,
        hir_id,
        tcx,
    );
    let fn_callable = Callable::Function(function_item);
    callables.push(fn_callable);
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

                    debug!("HIR: Extracted field access item with def id {:?}", def_id);

                    let parent = def_id_to_complex(def_id, tcx).unwrap();
                    let field_item = FieldAccessItem::new(
                        file_path.to_str().unwrap(),
                        ty,
                        parent,
                        is_public,
                        field.hir_id,
                        tcx,
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

    let parent_def_id = impl_to_struct_id(im);

    let impl_generics = generics_to_ts(&im.generics, tcx);

    let parent_hir_id = tcx
        .hir()
        .local_def_id_to_hir_id(parent_def_id.expect_local());
    let def_id = tcx.hir().local_def_id(parent_hir_id).to_def_id();
    let parent = def_id_to_complex(def_id, tcx).unwrap();

    let items = im.items;

    debug!("HIR: Impl generics:\n{:?}", impl_generics);
    for item in items {
        let def_id = item.id.def_id;

        match &item.kind {
            AssocItemKind::Fn { .. } => {
                let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);
                let impl_item = tcx.hir().impl_item(item.id);
                match &impl_item.kind {
                    ImplItemKind::Fn(sig, body_id) => {
                        debug!("HIR: (!) Found method {}, parent: {}", &impl_item.ident, parent_hir_id);

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
                                // {:?}, decl: {:?}", input, &sig.decl
                                return;
                            }
                        }

                        if !sig.decl.implicit_self.has_implicit_self() {
                            // Static method
                            debug!("HIR: Method is static");
                            let static_method_item = StaticFnItem::new(
                                file_path.to_str().unwrap(),
                                params,
                                return_type,
                                parent.clone(),
                                impl_generics.clone(),
                                is_public,
                                hir_id,
                                tcx,
                            );
                            let static_method_callable =
                                Callable::StaticFunction(static_method_item);
                            callables.push(static_method_callable);
                        } else {
                            // Dynamic method
                            debug!("HIR: Method is associative");
                            let method_item = MethodItem::new(
                                file_path.to_str().unwrap(),
                                params,
                                return_type,
                                parent.clone(),
                                impl_generics.clone(),
                                is_public,
                                hir_id,
                                tcx,
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
        VisibilityKind::Crate(sugar) => {
            match sugar {
                CrateSugar::PubCrate => true,
                _ => false
            }
        }
        _ => false
    }
}