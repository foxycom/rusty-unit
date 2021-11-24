use rustc_hir::def_id::{LocalDefId, LOCAL_CRATE};
use rustc_hir::intravisit::{Map, NestedVisitorMap, Visitor};
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_hir::{AssocItemKind, Body, BodyId, FnSig, ForeignItem, ForeignItemId, Generics, HirId, Impl, ImplItem, ImplItemId, ImplItemKind, Item, ItemId, ItemKind, Node, TraitItem, TraitItemId, VariantData};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::ErrorOutputType;
use rustc_span::Span;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::util::{get_cut_name, get_testify_flags};
use crate::writer::HirWriter;
use generation::analysis::HirAnalysis;
use generation::types::{
    Callable, ComplexT, FieldAccessItem, FunctionItem, MethodItem, StaticFnItem, T,
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

        if file_path.as_ref().unwrap().ends_with("testify_monitor.rs") {
            continue;
        }

        match &item.kind {
            ItemKind::Fn(sig, generics, body_id) => {
                if &item.ident.name.to_string() != "main" {
                    /*println!(
                        "Visited fn {} with def_id {:?}",
                        item.ident.name.to_string(),
                        item.def_id.to_def_id()
                    );*/
                    analyze_fn(
                        sig,
                        item.def_id,
                        body_id,
                        file_path.unwrap(),
                        &mut callables,
                        &tcx,
                    )
                }
            }
            ItemKind::Impl(im) => {
                if allowed_item(item, &tcx) {
                    println!("Analyzing item {}", item_to_name(item, &tcx));
                    analyze_impl(im, file_path.unwrap(), &mut callables, &tcx)
                }
            }
            ItemKind::Struct(s, g) => {
                if allowed_item(item, &tcx) {
                    analyze_struct(item.def_id, s, g, file_path.unwrap(), &tcx);
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
    file_path: PathBuf,
    callables: &mut Vec<Callable>,
    tcx: &TyCtxt<'_>,
) {
    let hir_id = tcx.hir().local_def_id_to_hir_id(local_def_id);
    let src_file_map = SOURCE_FILE_MAP.lock().unwrap();
    let src_file_id = *src_file_map.get(&file_path).unwrap();
    drop(src_file_map);

    let fn_decl = &sig.decl;

    // TODO a fn can also have explicit generics defined
    let generics = vec![];

    // self_hir_id must never be used, so just pass a dummy value
    let mut params = Vec::with_capacity(fn_decl.inputs.len());
    for input in fn_decl.inputs.iter() {
        if let Some(param) = ty_to_param(input, hir_id, &generics, tcx) {
            params.push(param);
        } else {
            return;
        }
    }

    let return_type = fn_ret_ty_to_t(&fn_decl.output, hir_id, tcx);

    let function_item = FunctionItem::new(src_file_id, params, return_type, hir_id, tcx);
    let fn_callable = Callable::Function(function_item);
    callables.push(fn_callable);
}

fn analyze_struct(
    struct_local_def_id: LocalDefId,
    vd: &VariantData,
    g: &Generics,
    file_path: PathBuf,
    tcx: &TyCtxt<'_>,
) {


    println!(">> U8 type: {:?}", tcx.types.u8.kind());
    //let adt_def = tcx.adt_def(struct_local_def_id.to_def_id());
    let src_file_map = SOURCE_FILE_MAP.lock().unwrap();

    let src_file_id = *src_file_map.get(&file_path).expect(&format!(
        "Source file map does not contain {:?}",
        &file_path
    ));
    drop(src_file_map);


    let struct_generics = generics_to_ts(g, tcx);
    let struct_hir_id = tcx.hir().local_def_id_to_hir_id(struct_local_def_id);
    match vd {
        VariantData::Struct(fields, _) => {
            let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();
            let parent = def_id_to_complex(def_id, tcx).unwrap();
            let parent_name = node_to_name(&tcx.hir().get(parent_hir_id), tcx).unwrap();
            if parent_name.contains("serde") {
                // Skip too hard stuff
                return;
            }

            for field in fields.iter() {
                let ty = ty_to_t(field.ty, Some(struct_hir_id), &struct_generics, tcx);
                if let Some(ty) = ty {
                    let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();
                    println!("Def id is {:?}", def_id);
                    let parent = def_id_to_complex(def_id, tcx).unwrap();
                    let field_item =
                        FieldAccessItem::new(src_file_id, ty, parent, field.hir_id, tcx);
                }
            }

            let mut params = Vec::with_capacity(sig.decl.inputs.len());
            for field in fields.iter() {
                let param = ty_to_param(field.ty, struct_hir_id, &struct_generics, tcx);
                if let Some(param) = param {
                    params.push(param);
                } else {
                    // An unknown type, ignore function
                    println!("Unknown field: {:?} ", field);
                    return;
                }
            }


            todo!()
        }
        _ => {}
    }
}

fn analyze_impl(im: &Impl, file_path: PathBuf, callables: &mut Vec<Callable>, tcx: &TyCtxt<'_>) {
    let src_file_map = SOURCE_FILE_MAP.lock().unwrap();
    let src_file_id = *src_file_map.get(&file_path).expect(&format!(
        "Source file map does not contain {:?}",
        &file_path
    ));
    drop(src_file_map);

    if let Some(_) = im.of_trait {
        // Skip trait implementation for now
        return;
    }

    let parent_def_id = impl_to_struct_id(im);

    let impl_generics = generics_to_ts(&im.generics, tcx);

    let parent_hir_id = tcx
        .hir()
        .local_def_id_to_hir_id(parent_def_id.expect_local());
    let items = im.items;

    println!("Analyzing impl: {:?}\nIt's generics are: {:?}", im, impl_generics);
    for item in items {
        let def_id = item.id.def_id;

        match &item.kind {
            AssocItemKind::Fn { .. } => {
                let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);
                let impl_item = tcx.hir().impl_item(item.id);
                match &impl_item.kind {
                    ImplItemKind::Fn(sig, body_id) => {
                        println!("Analyzing method {}", &impl_item.ident);

                        let parent_name = node_to_name(&tcx.hir().get(parent_hir_id), tcx).unwrap();
                        if parent_name.contains("serde") {
                            // Skip too hard stuff
                            continue;
                        }

                        let mut fn_generics = generics_to_ts(&impl_item.generics, tcx);
                        let mut overall_generics = impl_generics.clone();
                        overall_generics.append(&mut fn_generics);

                        let mut params = Vec::with_capacity(sig.decl.inputs.len());
                        for input in sig.decl.inputs.iter() {
                            let param = ty_to_param(input, parent_hir_id, &overall_generics, tcx);
                            if let Some(param) = param {
                                params.push(param);
                            } else {
                                // An unknown type, ignore function
                                println!("Unknown param: {:?} ", input);
                                println!("Decl is: {:?}", &sig.decl);
                                return;
                            }
                        }

                        let def_id = tcx.hir().local_def_id(parent_hir_id).to_def_id();
                        let parent = def_id_to_complex(def_id, tcx).unwrap();
                        let return_type = fn_ret_ty_to_t(&sig.decl.output, parent_hir_id, tcx);

                        if let Some(return_type) = return_type.as_ref() {
                            println!(">> Return type is {:?}", &sig.decl.output);
                        }
                        if !sig.decl.implicit_self.has_implicit_self() {
                            // Static method
                            let static_method_item = StaticFnItem::new(
                                src_file_id,
                                params,
                                return_type,
                                parent,
                                impl_generics.clone(),
                                hir_id,
                                tcx,
                            );
                            let static_method_callable = Callable::StaticFunction(static_method_item);
                            callables.push(static_method_callable);
                        } else {
                            // Dynamic method

                            let method_item = MethodItem::new(
                                src_file_id,
                                params,
                                return_type,
                                parent,
                                impl_generics.clone(),
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
