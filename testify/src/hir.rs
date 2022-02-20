use log::{debug, error, info, warn};
use rustc_data_structures::snapshot_vec::VecLike;
use rustc_hir::def_id::{LocalDefId, LOCAL_CRATE};
use rustc_hir::{AssocItemKind, BodyId, EnumDef, FnDecl, FnRetTy, FnSig, ForeignItem, Generics, HirId, Impl, ImplItem, ImplItemKind, Item, ItemId, ItemKind, QPath, TraitItem, Ty, TyKind, Variant, VariantData, Visibility, VisibilityKind};
use rustc_middle::ty::{DefIdTree, TyCtxt, TypeckResults};
use rustc_span::Span;
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
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
use crate::analysis::Analysis;
use crate::options::RuConfig;
use crate::types::{Callable, EnumInitItem, EnumVariant, FieldAccessItem, FunctionItem, generics_of_item, MethodItem, mir_ty_to_t, Param, StaticFnItem, StructInitItem, T};

use crate::writer::HirWriter;
use crate::util::{def_id_to_enum, def_id_to_struct, fn_ret_ty_to_t, generics_to_ts, hir_ty_to_t_unprecise, impl_to_struct_id, item_to_name, node_to_name, span_to_path, ty_to_param};

struct HirVisitor<'tcx> {
    types: Vec<String>,
    tcx: TyCtxt<'tcx>,
    callables: Vec<Callable>
}

impl<'tcx> HirVisitor<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            types: vec![],
            tcx,
            callables: vec![]
        }
    }

    pub fn write(&self) -> std::io::Result<()> {
        let path = Path::new(LOG_DIR).join(HIR_LOG_PATH);
        let mut file = OpenOptions::new().write(true)
            .truncate(true)
            .create(true)
            .open(path)?;

        let analysis = Analysis::new(&self.callables);
        let content = serde_json::to_string(&analysis)?;
        file.write(content.as_bytes())?;

        Ok(())
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
                    ty_to_param(None, i, &self.tcx)
                }).collect::<Vec<_>>();

                let file_path = span_to_path(&ii.span, &self.tcx).expect("File path unknown");

                let method = MethodItem::new(&name, file_path.to_str().unwrap(), params, return_t, parent_t, method_generics, is_public);
                self.callables.push(Callable::Method(method));
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
    visitor.write();
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
}

fn allowed_item(item: &Item<'_>, tcx: &TyCtxt<'_>) -> bool {
    let item_name = item_to_name(item, &tcx);
    !item_name.contains("serde") && !item_name.contains("test")
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
