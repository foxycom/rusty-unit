use crate::data_structures::{cdg, cfg, immediate_post_dominators, post_dominators};
use crate::writer::InstrumentationWriter;
use crate::Stage;
use generation::branch::Branch;
use instrumentation::MIR_LOG_PATH;
use petgraph::algo::dominators::simple_fast;
use petgraph::dot::Dot;
use petgraph::visit::Reversed;
use rustc_data_structures::graph::WithSuccessors;
use rustc_hir::def_id::DefId;
use rustc_hir::{HirId, ItemKind};
use rustc_interface::{Config, Queries};
use rustc_middle::mir::interpret::{ConstValue, Scalar};
use rustc_middle::mir::visit::MutVisitor;
use rustc_middle::mir::StatementKind::SetDiscriminant;
use rustc_middle::mir::{
    BasicBlock, BasicBlockData, BinOp, Body, Constant, ConstantKind, Local, LocalDecl, LocalDecls,
    Operand, Place, Rvalue, SourceInfo, SourceScope, Statement, StatementKind, Terminator,
    TerminatorKind, START_BLOCK,
};
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::{Const, ConstKind, List, ScalarInt, Ty, TyCtxt, UintTy};
use rustc_span::{Span, Symbol};
use rustc_target::abi::VariantIdx;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Arguments;
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::path::PathBuf;
use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;

type CutPoint<'tcx> = (BasicBlock, usize, BasicBlockData<'tcx>);

const CUSTOM_OPT_MIR_ANALYSIS: for<'tcx> fn(_: TyCtxt<'tcx>, _: DefId) -> &'tcx Body<'tcx> =
    |tcx, def| {
        let opt_mir = rustc_interface::DEFAULT_QUERY_PROVIDERS
            .borrow()
            .optimized_mir;
        let mut body = opt_mir(tcx, def).clone();
        let crate_name = tcx.crate_name(def.krate);
        let hir_id = tcx.hir().local_def_id_to_hir_id(def.expect_local());

        if crate_name.as_str() != "additions" || is_testify_monitor(hir_id, &tcx) {
            // Don't instrument extern crates
            return tcx.arena.alloc(body);
        }

        let cdg = cdg(&body);
        let mut writer = InstrumentationWriter::new(MIR_LOG_PATH);
        writer.new_body(&format!("{:?}", def));
        writer.write_cdg(serde_json::to_string(&cdg).as_ref().unwrap());

        // INSTRUMENT
        let mut mir_visitor = MirVisitor::new(body.clone(), tcx);

        let mut instrumented_body = mir_visitor.visit();
        let (basic_blocks, local_decls) = instrumented_body.basic_blocks_and_local_decls_mut();

        let locals = local_decls
            .iter_enumerated()
            .map(|(local, decl)| {
                format!("{:?} -> {:?}", local, decl)
            })
            .collect::<Vec<_>>();
        writer.write_locals(&locals);

        let blocks = basic_blocks
            .iter_enumerated()
            .map(|(block, data)| format!("{} -> {:?}", block.as_usize(), data))
            .collect::<Vec<_>>();
        writer.write_basic_blocks(&blocks);

        let op_enum = get_op_enum_def_id(&tcx);
        return tcx.arena.alloc(instrumented_body);
    };

const CUSTOM_OPT_MIR_INSTRUMENTATION: for<'tcx> fn(_: TyCtxt<'tcx>, _: DefId) -> &'tcx Body<'tcx> =
    |tcx, def| {
        let opt_mir = rustc_interface::DEFAULT_QUERY_PROVIDERS
            .borrow()
            .optimized_mir;
        let mut body = opt_mir(tcx, def).clone();
        let crate_name = tcx.crate_name(def.krate);
        let hir_id = tcx.hir().local_def_id_to_hir_id(def.expect_local());

        if crate_name.as_str() != "additions" || is_testify_monitor(hir_id, &tcx) {
            // Don't instrument extern crates
            return tcx.arena.alloc(body);
        }

        // INSTRUMENT
        let mut mir_visitor = MirVisitor::new(body.clone(), tcx);

        let mut instrumented_body = mir_visitor.visit();
        return tcx.arena.alloc(instrumented_body);
    };


pub struct MirVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    body: Body<'tcx>,
    locals_num: usize,
    branch_counter: u64,
    branches: Vec<Branch>,
}

impl<'tcx> MirVisitor<'tcx> {
    fn new(body: Body<'tcx>, tcx: TyCtxt<'tcx>) -> Self {
        MirVisitor {
            tcx,
            locals_num: body.local_decls.len(),
            body,
            branch_counter: 0,
            branches: vec![],
        }
    }

    fn visit(&mut self) -> Body<'tcx> {
        let mut body = self.body.clone();
        self.visit_body(&mut body);
        body
    }

    fn find_assign_stmt_for(
        &self,
        place: &Place,
        stmts: &Vec<Statement<'tcx>>,
    ) -> Option<(usize, Statement<'tcx>)> {
        for (pos, stmt) in stmts.iter().enumerate() {
            if let StatementKind::Assign(assign) = &stmt.kind {
                let (var, expr) = assign.as_ref();
                if var == place {
                    return Some((pos, stmt.clone()));
                }
            }
        }
        None
    }

    fn mk_place(&self, index: usize) -> Place<'tcx> {
        Place {
            local: Local::from(index),
            projection: List::empty(),
        }
    }

    fn mk_local_decl(&self, ty: Ty<'tcx>) -> LocalDecl<'tcx> {
        LocalDecl::new(ty, Span::default())
    }

    fn mk_const_int(&self, data: u64) -> &'tcx Const<'tcx> {
        let u64_ty = self.tcx.mk_mach_uint(UintTy::U64);
        let scalar_data = ConstKind::Value(ConstValue::Scalar(Scalar::Int(ScalarInt::from(data))));

        let const_arg = Const {
            ty: u64_ty,
            val: scalar_data,
        };

        let const_arg = self.tcx.mk_const(const_arg);
        const_arg
    }

    fn mk_move_operand(&self, local: Local) -> Operand<'tcx> {
        Operand::Move(Place::from(local))
    }

    fn mk_const_int_operand(&self, data: u64) -> Operand<'tcx> {
        Operand::Constant(Box::new(Constant {
            span: Default::default(),
            user_ty: None,
            literal: ConstantKind::Ty(self.mk_const_int(data)),
        }))
    }

    fn mk_basic_block(
        &self,
        stmts: Vec<Statement<'tcx>>,
        args: Vec<Operand<'tcx>>,
        point_to: BasicBlock,
        place: Place<'tcx>,
    ) -> BasicBlockData<'tcx> {
        let trace_fn_id = get_trace_test_fn_def_id(&self.tcx);
        let fn_ty = self.tcx.type_of(trace_fn_id);
        let func_const = Const {
            ty: fn_ty,
            val: ConstKind::Value(ConstValue::Scalar(Scalar::Int(ScalarInt::ZST))),
        };

        let func_const = self.tcx.mk_const(func_const);

        let func_constant = Constant {
            span: Span::default(),
            user_ty: None,
            literal: ConstantKind::Ty(func_const),
        };

        let func_call = Operand::Constant(Box::new(func_constant));

        let terminator_kind = TerminatorKind::Call {
            func: func_call,
            args,
            destination: Some((place, point_to)),
            cleanup: None,
            from_hir_call: false,
            fn_span: Default::default(),
        };

        let terminator = Terminator {
            source_info: self.mk_dummy_source_info(),
            kind: terminator_kind,
        };

        let mut block = BasicBlockData::new(Some(terminator));

        for stmt in stmts {
            block.statements.push(stmt);
        }
        block
    }

    fn mk_dummy_source_info(&self) -> SourceInfo {
        SourceInfo {
            span: Default::default(),
            scope: SourceScope::from(0usize),
        }
    }

    fn mk_enum_var_stmt(&self, place_idx: usize, variant_idx: u32) -> Statement<'tcx> {
        let stmt_kind = SetDiscriminant {
            place: Box::new(self.mk_place(place_idx)),
            variant_index: VariantIdx::from_u32(variant_idx),
        };
        Statement {
            source_info: self.mk_dummy_source_info(),
            kind: stmt_kind,
        }
    }

    fn next_branch_id(&mut self) -> u64 {
        let id = self.branch_counter;
        self.branch_counter += 1;
        id
    }

    fn update_terminator(
        &self,
        terminator: &mut Terminator<'tcx>,
        idx: usize,
        basic_block: BasicBlock,
    ) {
        match &mut terminator.kind {
            TerminatorKind::SwitchInt { targets, .. } => {
                let targets = targets.all_targets_mut();
                targets[idx] = basic_block;
            }
            _ => {}
        }
    }

    fn store_basic_block(
        &mut self,
        cut_points: &mut Vec<CutPoint<'tcx>>,
        stmts: Vec<Statement<'tcx>>,
        args: Vec<Operand<'tcx>>,
        source_block: BasicBlock,
        target_block: BasicBlock,
        path_idx: usize,
    ) {
        let place = self.mk_place(self.locals_num);
        let trace_block = self.mk_basic_block(stmts, args, target_block, place);

        cut_points.push((source_block, path_idx, trace_block));
        self.branch_counter += 1;
    }

    fn store_unit_local_decl(&mut self, local_decls: &mut LocalDecls<'tcx>) -> Local {
        let unit_ty = self.tcx.mk_unit();
        let local_decl = self.mk_local_decl(unit_ty);
        local_decls.push(local_decl);
        let local = Local::from(self.locals_num);
        self.locals_num += 1;
        local
    }

    fn instr_for_branch(
        &mut self,
        cut_points: &mut Vec<CutPoint<'tcx>>,
        local_decls: &mut LocalDecls<'tcx>,
        path_idx: usize,
        source_block: BasicBlock,
        target_block: BasicBlock,
    ) {
        let op_enum_def_id = get_op_enum_def_id(&self.tcx);
        let op_enum_ty = self.tcx.type_of(op_enum_def_id);
        let local = self.store_local_decl(op_enum_ty, local_decls);
        let stmts = vec![self.mk_enum_var_stmt(local.index(), 0)];
        let branch_id = self.next_branch_id();
        let args = vec![
            self.mk_const_int_operand(branch_id),
            self.mk_move_operand(local),
        ];
        self.store_basic_block(
            cut_points,
            stmts,
            args,
            source_block,
            target_block,
            path_idx,
        );
        self.store_unit_local_decl(local_decls);

        //let branch = Branch::new(branch_id, BranchType::Decision);
    }

    fn store_local_decl(&mut self, ty: Ty<'tcx>, local_decls: &mut LocalDecls<'tcx>) -> Local {
        let local_decl = LocalDecl::new(ty, Span::default());
        local_decls.push(local_decl);
        let local = Local::from(self.locals_num);
        self.locals_num += 1;
        local
    }

    fn get_local(&self, operand: &Operand) -> Local {
        match operand {
            Operand::Copy(place) => place.local.clone(),
            Operand::Move(place) => place.local.clone(),
            _ => unimplemented!(),
        }
    }

    fn switch(
        &self,
        terminator: &mut Terminator<'tcx>,
        stmts: &mut Vec<Statement<'tcx>>,
    ) -> Vec<Local> {
        let operand = match &mut terminator.kind {
            TerminatorKind::SwitchInt { discr, .. } => discr,
            _ => unimplemented!(),
        };
        let mut local_delcs = vec![];
        match operand {
            Operand::Move(place) => {
                let (pos, assign_stmt) = self.find_assign_stmt_for(place, stmts).unwrap();
                if let StatementKind::Assign(b) = &assign_stmt.kind {
                    let (var, expr) = b.as_ref();
                    if let Rvalue::BinaryOp(_, operands) = expr {
                        let (left, right) = operands.as_ref();

                        let mut left_place = var.clone();
                        left_place.local = Local::from(self.locals_num as u32 + 1);
                        let mut left_assignment = assign_stmt.clone();
                        left_assignment.kind = StatementKind::Assign(Box::new((
                            left_place,
                            Rvalue::Use(left.clone()),
                        )));
                        stmts.push(left_assignment);

                        local_delcs.push(self.get_local(left));
                        local_delcs.push(self.get_local(left));

                        let mut right_place = var.clone();
                        right_place.local = Local::from(self.locals_num as u32 + 2);
                        let mut right_assignment = assign_stmt.clone();
                        right_assignment.kind = StatementKind::Assign(Box::new((
                            right_place,
                            Rvalue::Use(right.clone()),
                        )));
                        stmts.push(right_assignment);

                        local_delcs.push(self.get_local(right));
                        local_delcs.push(self.get_local(right));

                        let mut result_place = var.clone();
                        result_place.local = Local::from(self.locals_num as u32 + 3);
                        let mut result_assignment = assign_stmt.clone();
                        let sub_op = Rvalue::BinaryOp(
                            BinOp::Sub,
                            Box::new((Operand::Move(left_place), Operand::Move(right_place))),
                        );
                        result_assignment.kind =
                            StatementKind::Assign(Box::new((result_place, sub_op)));
                        stmts.push(result_assignment);

                        //*operand = Operand::Move(result_place);
                    }
                }
            }
            _ => {}
        }

        local_delcs
    }
}

impl<'tcx> MutVisitor<'tcx> for MirVisitor<'tcx> {
    fn visit_body(&mut self, body: &mut Body<'tcx>) {
        let (basic_blocks, local_decls) = body.basic_blocks_and_local_decls_mut();

        let mut cut_points = vec![];

        for (basic_block, data) in basic_blocks.iter_enumerated_mut() {
            if let Some(terminator) = &mut data.terminator {
                match &mut terminator.kind {
                    TerminatorKind::SwitchInt {
                        discr,
                        switch_ty,
                        targets,
                    } => match targets.all_targets_mut() {
                        [true_branch, false_branch] => {
                            self.instr_for_branch(
                                &mut cut_points,
                                local_decls,
                                0,
                                basic_block.clone(),
                                true_branch.clone(),
                            );
                            self.instr_for_branch(
                                &mut cut_points,
                                local_decls,
                                1,
                                basic_block.clone(),
                                false_branch.clone(),
                            );
                        }
                        _ => {}
                    },
                    TerminatorKind::Call {
                        func,
                        args,
                        destination,
                        cleanup,
                        from_hir_call,
                        fn_span,
                    } => match func {
                        Operand::Copy(_) => {}
                        Operand::Move(_) => {}
                        Operand::Constant(constant) => {
                            println!("Func {:?}", func);
                            println!("Args:");
                        }
                    },
                    _ => {}
                }
            }
        }

        for (target_basic_block, idx, trace_block) in cut_points {
            let basic_blocks_num = basic_blocks.len();
            basic_blocks.push(trace_block);
            let tracing_basic_block = BasicBlock::from(basic_blocks_num);

            let data = basic_blocks.get_mut(target_basic_block).unwrap();
            let terminator = data.terminator.as_mut().unwrap();
            self.update_terminator(terminator, idx, tracing_basic_block);
        }
    }

    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
        self.tcx.tcx()
    }
}

fn get_trace_test_fn_def_id(tcx: &TyCtxt<'_>) -> DefId {
    tcx.hir()
        .items()
        .find_map(|i| {
            if let ItemKind::Fn(_, _, _) = &i.kind {
                if i.ident.name.to_string() == "trace_test" {
                    return Some(i.def_id.to_def_id());
                }
            }
            None
        })
        .unwrap()
}

fn is_testify_monitor(hir_id: HirId, tcx: &TyCtxt<'_>) -> bool {
    /*let name = tcx.hir().name(hir_id).as_str();
    if name == "testify_monitor" {
        true
    } else if name == "additions" {
        false
    } else {
        let parent = tcx.parent_module(hir_id);
        is_testify_monitor(tcx.hir().local_def_id_to_hir_id(parent), tcx)
    }*/
    let name = format!("{:?}", hir_id);
    name.contains("testify_monitor")
}

fn get_op_enum_def_id(tcx: &TyCtxt<'_>) -> DefId {
    tcx.hir()
        .items()
        .find_map(|i| {
            if let ItemKind::Enum(_, _) = &i.kind {
                if i.ident.name.to_string() == "Op" {
                    return Some(i.def_id.to_def_id());
                }
            }
            None
        })
        .unwrap()
}

fn get_output_file() -> File {
    std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/Users/tim/Documents/master-thesis/testify/results/instrumentation.log")
        .unwrap()
}

fn get_post_dominators_file() -> File {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open("/Users/tim/Documents/master-thesis/testify/instrumentation/post-dominators.dot")
        .unwrap()
}

fn get_cfg_file() -> File {
    std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open("/Users/tim/Documents/master-thesis/testify/instrumentation/cfg.dot")
        .unwrap()
}

fn get_cdg_file() -> File {
    std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open("/Users/tim/Documents/master-thesis/testify/instrumentation/cdg.dot")
        .unwrap()
}
