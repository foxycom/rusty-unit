use rustc_driver::Compilation;
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_hir::{
    AssocItemKind, BodyId, FnSig, ForeignItem, Generics, HirId, Impl, ImplItem, ImplItemKind, Item,
    ItemKind, TraitItem, VariantData,
};
use rustc_interface::Config;
use rustc_middle::mir::visit::{MutVisitor, TyContext};
use rustc_middle::mir::{
    BasicBlock, BasicBlockData, BinOp, Body, Local, Location, Place, Rvalue, SourceInfo, Statement,
    StatementKind, TerminatorKind,
};
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::{Ty, TyCtxt};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::analysis::Analysis;
use crate::chromosome::{
    Callable, Chromosome, ComplexT, FieldAccessItem, FunctionItem, MethodItem, Param, StaticFnItem,
    TestCase, T,
};
use crate::source::{Project, ProjectScanner};
use crate::util::{impl_to_struct_id, span_to_path};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::dep_graph::DepContext;
use rustc_session::config::ErrorOutputType;
use rustc_span::def_id::DefId;
use rustc_span::Span;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref PROJECT: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
    static ref SOURCE_FILE_MAP: Arc<Mutex<HashMap<PathBuf, usize>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

const CUSTOM_OPT_MIR: for<'tcx> fn(_: TyCtxt<'tcx>, _: DefId) -> &'tcx Body<'tcx> = |tcx, def| {
    let opt_mir = rustc_interface::DEFAULT_QUERY_PROVIDERS
        .borrow()
        .optimized_mir;
    let mut body = opt_mir(tcx, def).clone();

    let mut mir_visitor = MirVisitor {
        tcx,
        locals_count: body.local_decls.len(),
    };
    //mir_visitor.visit_body(&mut body);
    let local_def_id = LocalDefId {
        local_def_index: def.index.clone(),
    };

    tcx.arena.alloc(body)
};

pub fn start(project: Project) {
    // Clear build directory, otherwise the compiler won't run the main function

    project.clear_build_dirs();
    let args = project.build_args();
    let mut source_file_map = SOURCE_FILE_MAP.lock().unwrap();
    for (pos, file) in project.source_files().iter().enumerate() {
        source_file_map.insert(file.file_path().to_path_buf(), pos);
    }
    drop(source_file_map);

    *PROJECT.lock().unwrap() = Some(project.project_root().to_path_buf());

    // Run analysis
    let mut callbacks = CompilerCallbacks { callables: vec![] };
    rustc_driver::RunCompiler::new(&args, &mut callbacks)
        .run()
        .unwrap();
}

struct CompilerCallbacks {
    callables: Vec<Callable>,
}

impl rustc_driver::Callbacks for CompilerCallbacks {
    fn config(&mut self, _config: &mut Config) {
        // Disable MIR optimization
        //_config.opts.debugging_opts.mir_opt_level = Some(0);

        _config.override_queries = Some(|session, local, external| {
            local.optimized_mir = CUSTOM_OPT_MIR;
        });
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> Compilation {
        enter_with_fn(_queries, run);
        Compilation::Stop
    }
}

fn enter_with_fn<'tcx, TyCtxtFn>(queries: &'tcx rustc_interface::Queries<'tcx>, enter_fn: TyCtxtFn)
where
    TyCtxtFn: Fn(TyCtxt),
{
    queries.global_ctxt().unwrap().peek_mut().enter(enter_fn);
}

/// This is the main entry point to the generation of tests
fn run(tcx: TyCtxt<'_>) {
    let analysis = Rc::new(hir_analysis(&tcx));

    let test_cases: Vec<TestCase> = (0..10)
        .map(|_| TestCase::random(analysis.clone()))
        .collect();

    let project_root = PROJECT.lock().unwrap();
    let project_root = project_root.as_ref().unwrap();
    let mut project = ProjectScanner::open(project_root.to_str().unwrap());

    project.add_tests(&test_cases, &tcx);
    // Create a copy of the project
    /*project.add_tests(&test_cases, &tcx);
    project.run_tests();
    project.clear_tests();*/
}

fn hir_analysis(tcx: &TyCtxt<'_>) -> Analysis {
    let (entry_def_id, _) = if let Some((entry_def, x)) = tcx.entry_fn(()) {
        (entry_def, x)
    } else {
        let msg = "This tool currently only supports single main";
        rustc_session::early_error(ErrorOutputType::default(), msg);
    };

    let mut callables = vec![];
    for item in tcx.hir().items() {
        let span: &Span = &item.span;
        let file_path = span_to_path(span, tcx);
        match &item.kind {
            ItemKind::Fn(sig, generics, body_id) => {
                if &item.ident.name.to_string() != "main" {
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
    let function_item = FunctionItem::new(*src_file_id, sig, body_id.clone(), hir_id, tcx);
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
                let field_item =
                    FieldAccessItem::new(*src_file_id, field, struct_hir_id, field.hir_id, tcx);
                println!("Extracted field: {:?}", field_item);
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
                if !sig.decl.implicit_self.has_implicit_self() {
                    // Static method
                    let static_method_item = StaticFnItem::new(
                        *src_file_id,
                        sig,
                        body_id.clone(),
                        hir_id,
                        parent_hir_id,
                        tcx,
                    );
                    let static_method_callable = Callable::StaticFunction(static_method_item);
                    callables.push(static_method_callable);
                } else {
                    // Dynamic method
                    let method_item = MethodItem::new(
                        *src_file_id,
                        sig,
                        body_id.clone(),
                        hir_id,
                        parent_hir_id,
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

struct MirVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    locals_count: usize,
}

impl<'tcx> MirVisitor<'tcx> {
    fn find_assign_stmt_for(
        &self,
        place: &Place,
        stmts: &Vec<Statement<'tcx>>,
    ) -> Option<Statement<'tcx>> {
        stmts.iter().find_map(|s| {
            if let StatementKind::Assign(assign) = &s.kind {
                let (var, r_value) = assign.as_ref();
                if var == place {
                    return Some(s.clone());
                }
            }
            None
        })
    }

    fn sub_ints(&self, stmt: Statement<'tcx>) -> Option<Statement<'tcx>> {
        let mut compute_dist_stmt = Statement::from(stmt.clone());

        match &stmt.kind {
            StatementKind::Assign(assign) => {
                let (o_place, o_r_value) = assign.as_ref();
                let mut place = Place {
                    local: Local::from(self.locals_count + 1),
                    projection: o_place.projection.clone(),
                };
                if let Rvalue::BinaryOp(op, operands) = o_r_value {
                    let (left, right) = operands.as_ref();
                    let r_value =
                        Rvalue::BinaryOp(BinOp::Sub, Box::new((left.clone(), right.clone())));
                    compute_dist_stmt.kind = StatementKind::Assign(Box::new((place, r_value)));
                    return Some(compute_dist_stmt);
                } else {
                    return None;
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl<'tcx> MutVisitor<'tcx> for MirVisitor<'tcx> {
    fn visit_basic_block_data(&mut self, block: BasicBlock, data: &mut BasicBlockData<'tcx>) {
        if let Some(terminator) = &data.terminator {
            match &terminator.kind {
                TerminatorKind::SwitchInt {
                    discr,
                    switch_ty,
                    targets,
                } => {
                    println!("Contains switchint");
                }
                _ => {}
            }
        }
    }

    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
        self.tcx.tcx()
    }
}
