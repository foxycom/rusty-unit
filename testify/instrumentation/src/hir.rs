use rustc_hir::def_id::{LocalDefId, LOCAL_CRATE};
use rustc_hir::{BodyId, FnSig, Generics, Impl, ImplItemKind, Item, ItemKind, VariantData};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::ErrorOutputType;
use rustc_span::Span;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::writer::HirWriter;
use generation::analysis::HirAnalysis;
use generation::types::{
    Callable, ComplexT, FieldAccessItem, FunctionItem, MethodItem, StaticFnItem, T,
};
use generation::util::{fn_ret_ty_to_t, impl_to_struct_id, item_to_name, node_to_name, span_to_path, ty_to_param, ty_to_t};
use instrumentation::HIR_LOG_PATH;
use crate::util::{get_cut_name, get_testify_flags};
lazy_static! {
    pub static ref SOURCE_FILE_MAP: Arc<Mutex<HashMap<PathBuf, usize>>> =
        Arc::new(Mutex::new(HashMap::new()));
}
pub fn hir_analysis(tcx: TyCtxt<'_>) {
    /*let source_map = SOURCE_FILE_MAP.lock().unwrap();
    println!("Source map contains");
    source_map.iter().map(|(path, _)| path.as_path()).for_each(|p| println!("{:?}", p));
    drop(source_map);*/

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
                    //println!("Analyzing item {}", item_to_name(item, &tcx));
                    analyze_impl(im, file_path.unwrap(), &mut callables, &tcx)
                }
            },
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

    // self_hir_id must never be used, so just pass a dummy value
    let mut params = Vec::with_capacity(fn_decl.inputs.len());
    for input in fn_decl.inputs.iter() {
        if let Some(param) = ty_to_param(input, hir_id, tcx) {
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
    //let adt_def = tcx.adt_def(struct_local_def_id.to_def_id());
    let src_file_map = SOURCE_FILE_MAP.lock().unwrap();

    let src_file_id = *src_file_map.get(&file_path).expect(&format!(
        "Source file map does not contain {:?}",
        &file_path
    ));
    drop(src_file_map);

    let struct_hir_id = tcx.hir().local_def_id_to_hir_id(struct_local_def_id);
    match vd {
        VariantData::Struct(fields, _) => {
            for field in fields.iter() {
                let ty = ty_to_t(field.ty, struct_hir_id, tcx);
                if let Some(ty) = ty {
                    let parent_name = node_to_name(&tcx.hir().get(struct_hir_id), tcx).unwrap();
                    let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();
                    let parent = T::Complex(ComplexT::new(struct_hir_id, def_id, parent_name));
                    let field_item =
                        FieldAccessItem::new(src_file_id, ty, parent, field.hir_id, tcx);
                }
            }
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

    let parent_def_id = impl_to_struct_id(im);

    let parent_hir_id = tcx
        .hir()
        .local_def_id_to_hir_id(parent_def_id.expect_local());
    let items = im.items;
    for item in items {
        let def_id = item.id.def_id;
        let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);
        let impl_item = tcx.hir().impl_item(item.id);
        match &impl_item.kind {
            ImplItemKind::Fn(sig, body_id) => {
                let parent_name = node_to_name(&tcx.hir().get(parent_hir_id), tcx).unwrap();
                if parent_name.contains("serde") {
                    continue;
                }

                let mut params = Vec::with_capacity(sig.decl.inputs.len());
                for input in sig.decl.inputs.iter() {
                    let param = ty_to_param(input, parent_hir_id, tcx);
                    if let Some(param) = param {
                        params.push(param);
                    } else {
                        // An unknown type, ignore function
                        return;
                    }
                }

                let def_id = tcx.hir().local_def_id(parent_hir_id).to_def_id();
                let parent = T::Complex(ComplexT::new(parent_hir_id, def_id, parent_name));

                let return_type = fn_ret_ty_to_t(&sig.decl.output, parent_hir_id, tcx);

                if !sig.decl.implicit_self.has_implicit_self() {
                    // Static method
                    let static_method_item =
                        StaticFnItem::new(src_file_id, params, return_type, parent, hir_id, tcx);
                    let static_method_callable = Callable::StaticFunction(static_method_item);
                    callables.push(static_method_callable);
                } else {
                    // Dynamic method

                    let method_item =
                        MethodItem::new(src_file_id, params, return_type, parent, hir_id, tcx);
                    let method_callable = Callable::Method(method_item);
                    callables.push(method_callable);
                }
            }
            _ => {  },
        }
    }
}
