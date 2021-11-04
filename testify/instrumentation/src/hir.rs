use std::path::PathBuf;
use rustc_hir::{BodyId, FnSig, Generics, Impl, ImplItemKind, ItemKind, VariantData};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::ty::TyCtxt;
use rustc_session::config::ErrorOutputType;
use rustc_span::Span;
use generation::analysis::Analysis;
use generation::types::{Callable, ComplexT, FieldAccessItem, FunctionItem, MethodItem, StaticFnItem, T};
use generation::util::{fn_ret_ty_to_t, impl_to_struct_id, node_to_name, span_to_path, ty_to_param, ty_to_t};

pub fn hir_analysis(tcx: TyCtxt<'_>) -> Analysis {
    /*let (entry_def_id, _) = if let Some((entry_def, x)) = tcx.entry_fn(()) {
        (entry_def, x)
    } else {
        let msg = "This tool currently only supports single main";
        rustc_session::early_error(ErrorOutputType::default(), msg);
    };*/

    let mut callables = vec![];
    for item in tcx.hir().items() {
        let span: &Span = &item.span;
        let file_path = span_to_path(span, &tcx);
        match &item.kind {
            ItemKind::Fn(sig, generics, body_id) => {
                if &item.ident.name.to_string() != "main" {
                    println!(
                        "Visited fn {} with def_id {:?}",
                        item.ident.name.to_string(),
                        item.def_id.to_def_id()
                    );
                    analyze_fn(sig, item.def_id, body_id, file_path, &mut callables, &tcx)
                }
            }
            ItemKind::Impl(im) => analyze_impl(im, file_path, &mut callables, &tcx),
            ItemKind::Struct(s, g) => {
                analyze_struct(item.def_id, s, g, file_path, &tcx);
            }
            _ => {}
        }
    }

    let mut analysis = Analysis::new();
    analysis.set_callables(callables);
    analysis
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
    let src_file_id = src_file_map.get(&file_path).unwrap();

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

    let function_item = FunctionItem::new(
        *src_file_id,
        params,
        return_type,
        body_id.clone(),
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
    file_path: PathBuf,
    tcx: &TyCtxt<'_>,
) {
    //let adt_def = tcx.adt_def(struct_local_def_id.to_def_id());
    let src_file_map = SOURCE_FILE_MAP.lock().unwrap();

    let src_file_id = src_file_map.get(&file_path).unwrap();
    let struct_hir_id = tcx.hir().local_def_id_to_hir_id(struct_local_def_id);
    match vd {
        VariantData::Struct(fields, _) => {
            for field in fields.iter() {
                let ty = ty_to_t(field.ty, struct_hir_id, tcx);
                if let Some(ty) = ty {
                    let parent_name = node_to_name(&tcx.hir().get(struct_hir_id), tcx);
                    let def_id = tcx.hir().local_def_id(struct_hir_id).to_def_id();
                    let parent = T::Complex(ComplexT::new(struct_hir_id, def_id, parent_name));
                    let field_item =
                        FieldAccessItem::new(*src_file_id, ty, parent, field.hir_id, tcx);
                }
            }
        }
        _ => {}
    }
}

fn analyze_impl(im: &Impl, file_path: PathBuf, callables: &mut Vec<Callable>, tcx: &TyCtxt<'_>) {
    let src_file_map = SOURCE_FILE_MAP.lock().unwrap();
    let src_file_id = src_file_map.get(&file_path).unwrap();

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

                let return_type = fn_ret_ty_to_t(&sig.decl.output, parent_hir_id, tcx);

                let parent_name = node_to_name(&tcx.hir().get(parent_hir_id), tcx);
                let def_id = tcx.hir().local_def_id(parent_hir_id).to_def_id();
                let parent = T::Complex(ComplexT::new(parent_hir_id, def_id, parent_name));

                if !sig.decl.implicit_self.has_implicit_self() {
                    // Static method
                    let static_method_item = StaticFnItem::new(
                        *src_file_id,
                        params,
                        return_type,
                        parent,
                        body_id.clone(),
                        hir_id,
                        tcx,
                    );
                    let static_method_callable = Callable::StaticFunction(static_method_item);
                    callables.push(static_method_callable);
                } else {
                    // Dynamic method

                    let method_item = MethodItem::new(
                        *src_file_id,
                        params,
                        return_type,
                        parent,
                        body_id.clone(),
                        hir_id,
                        tcx,
                    );
                    let method_callable = Callable::Method(method_item);
                    callables.push(method_callable);
                }
            }
            _ => unimplemented!(),
        }
    }
}