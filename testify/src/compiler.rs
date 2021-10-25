use rustc_driver::Compilation;
use rustc_hir::itemlikevisit::ItemLikeVisitor;
use rustc_hir::{AssocItemKind, BodyId, FnSig, ForeignItem, Generics, HirId, Impl, ImplItem, ImplItemKind, Item, ItemKind, TraitItem, VariantData};
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
use std::io::ErrorKind;

use crate::analysis::Analysis;
use crate::chromosome::{Callable, Chromosome, ComplexT, FieldAccessItem, FunctionItem, MethodItem, Param, StaticFnItem, T, TestCase};
use rustc_hir::def_id::LocalDefId;
use rustc_session::config::ErrorOutputType;
use rustc_span::def_id::DefId;
use std::process;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use crate::util::impl_to_struct_id;

const MAIN: for<'tcx> fn(_: TyCtxt<'tcx>, _: DefId) -> &'tcx Body<'tcx> = |tcx, def| {
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

const TRACE: for<'tcx> fn(_: TyCtxt<'tcx>, _: DefId) -> &'tcx Body<'tcx> = |tcx, def| {
    let opt_mir = rustc_interface::DEFAULT_EXTERN_QUERY_PROVIDERS
        .borrow()
        .optimized_mir;
    let mut body = opt_mir(tcx, def).clone();

    let mut mir_visitor = MirVisitor {
        tcx,
        locals_count: body.local_decls.len(),
    };
    mir_visitor.visit_body(&mut body);
    let local_def_id = LocalDefId {
        local_def_index: def.index.clone(),
    };

    tcx.arena.alloc(body)
};

pub fn run_compiler() {
    // Clear build directory, otherwise the compiler won't run the main function

    remove_quite("/Users/tim/Documents/master-thesis/testify/benchmarks/target");
    remove_quite("/Users/tim/Documents/master-thesis/testify/benchmarks/debug");
    let deps_dir = std::path::Path::new(
        "/Users/tim/Documents/master-thesis/testify/benchmarks/target/debug/deps",
    );
    std::fs::create_dir_all(deps_dir).unwrap();
    let args = [
        "rustc",
        "--crate-name",
        "additions",
        "--edition=2018",
        "/Users/tim/Documents/master-thesis/testify/benchmarks/src/main.rs",
        "--error-format=json",
        "--json=diagnostic-rendered-ansi",
        "--crate-type",
        "bin",
        "--emit=dep-info,link",
        "-C",
        "embed-bitcode=no",
        "-C",
        "split-debuginfo=unpacked",
        "-C",
        "debuginfo=2",
        "-C",
        "metadata=5978598c4741d9d6",
        "--out-dir",
        "/Users/tim/Documents/master-thesis/testify/benchmarks/target/debug/deps",
        "-C",
        "incremental=/Users/tim/Documents/master-thesis/testify/benchmarks/debug/incremental",
        "-L",
        "dependency=/Users/tim/Documents/master-thesis/testify/benchmarks/target/debug/deps",
        "--sysroot",
        &sysroot(),
    ];

    let mut callbacks = CompilerCallbacks { callables: vec![] };
    let args: Vec<String> = args.iter().map(|&a| a.to_string()).collect();

    rustc_driver::RunCompiler::new(&args, &mut callbacks)
        .run()
        .unwrap();
}

fn sysroot() -> String {
    let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();

    let sysroot = String::from_utf8(out.stdout).unwrap();
    let sysroot = sysroot.trim();
    sysroot.to_string()
}

struct CompilerCallbacks {
    callables: Vec<Callable>,
}

impl rustc_driver::Callbacks for CompilerCallbacks {
    fn config(&mut self, _config: &mut Config) {
        // Disable MIR optimization
        //_config.opts.debugging_opts.mir_opt_level = Some(0);

        _config.override_queries = Some(|session, local, external| {
            //local.optimized_mir = MAIN;
            external.optimized_mir = TRACE;
        });
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _queries: &'tcx rustc_interface::Queries<'tcx>,
    ) -> Compilation {
        enter_with_fn(_queries, run);
        Compilation::Continue
    }
}

fn enter_with_fn<'tcx, TyCtxtFn>(queries: &'tcx rustc_interface::Queries<'tcx>, enter_fn: TyCtxtFn)
where
    TyCtxtFn: Fn(TyCtxt),
{
    queries.global_ctxt().unwrap().peek_mut().enter(enter_fn);
}

fn run(tcx: TyCtxt<'_>) {
    let analysis = hir_analysis(tcx);

    let analysis = Rc::new(analysis);
    let test_case = TestCase::random(analysis.clone());

    println!("{}", test_case.to_string(&tcx));
}

fn hir_analysis(tcx: TyCtxt<'_>) -> Analysis {
    let (entry_def_id, _) = if let Some((entry_def, x)) = tcx.entry_fn(()) {
        (entry_def, x)
    } else {
        let msg = "This tool currently only supports single main";
        rustc_session::early_error(ErrorOutputType::default(), msg);
    };

    let main_id = entry_def_id;

    let krate = tcx.hir().krate();
    let mut callables = vec![];
    for item in tcx.hir().items() {
        match &item.kind {
            ItemKind::Fn(sig, generics, body_id) => {
                analyze_fn(sig, item.def_id, body_id, &mut callables, &tcx)
            }
            ItemKind::Impl(im) => {
                analyze_impl(im, &mut callables, &tcx)
            },
            ItemKind::Struct(s, g) => {
                analyze_struct(item.def_id, s, g, &tcx);
            }
            _ => {}
        }
    }

    Analysis::new(callables)
}

fn analyze_fn(
    sig: &FnSig,
    local_def_id: LocalDefId,
    body_id: &BodyId,
    callables: &mut Vec<Callable>,
    tcx: &TyCtxt<'_>,
) {
    let hir_id = tcx.hir().local_def_id_to_hir_id(local_def_id);
    let function_item = FunctionItem::new(sig, body_id.clone(), hir_id, tcx);
    let fn_callable = Callable::Function(function_item);
    callables.push(fn_callable);
}

fn analyze_struct(struct_local_def_id: LocalDefId, vd: &VariantData, g: &Generics, tcx: &TyCtxt<'_>) {
    println!("Analyzing struct: {:?}", struct_local_def_id);
    let struct_hir_id = tcx.hir().local_def_id_to_hir_id(struct_local_def_id);
    match vd {
        VariantData::Struct(fields, _) => {
            for field in fields.iter() {
                let field_item = FieldAccessItem::new(field, struct_hir_id, field.hir_id, tcx);
                println!("Extracted field: {:?}", field_item);
            }
        }
        _ => {}
    }
}

fn analyze_impl(im: &Impl, callables: &mut Vec<Callable>, tcx: &TyCtxt<'_>) {
    let parent_hir_id = impl_to_struct_id(im);
    let parent_hir_id = tcx.hir().local_def_id_to_hir_id(parent_hir_id.expect_local());
    let items = im.items;
    for item in items {
        let def_id = item.id.def_id;
        let hir_id = tcx.hir().local_def_id_to_hir_id(def_id);
        let impl_item = tcx.hir().impl_item(item.id);
        match &impl_item.kind {
            ImplItemKind::Fn(sig, body_id) => {
                if !sig.decl.implicit_self.has_implicit_self() {
                    // Static method
                    let static_method_item =
                        StaticFnItem::new(sig, body_id.clone(), hir_id, parent_hir_id, tcx);
                    let static_method_callable = Callable::StaticFunction(static_method_item);
                    callables.push(static_method_callable);
                } else {
                    // Dynamic method
                    let method_item = MethodItem::new(sig, body_id.clone(), hir_id, parent_hir_id, tcx);
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

fn remove_quite(path: &str) -> std::io::Result<()> {
    let path = std::path::Path::new(path);
    if let Err(err) = std::fs::remove_dir_all(path) {
        if err.kind() != ErrorKind::NotFound {
            return Err(err);
        }
    }

    Ok(())
}
