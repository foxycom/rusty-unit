use crate::data_structures::{cdg, immediate_post_dominators, post_dominators, truncated_cfg};
use crate::util::get_cut_name;
use crate::writer::MirWriter;
use crate::{get_testify_flags, Stage};
use generation::branch::{Branch, DecisionBranch};
use generation::util::{node_to_name, ty_to_name};
use generation::MIR_LOG_PATH;
use instrumentation::monitor::{BinaryOp, UnaryOp};
use petgraph::algo::dominators::simple_fast;
use petgraph::dot::Dot;
use petgraph::visit::Reversed;
use rustc_data_structures::graph::{WithNumNodes, WithSuccessors};
use rustc_data_structures::tagged_ptr::Pointer;
use rustc_driver::Compilation;
use rustc_hir::def_id::DefId;
use rustc_hir::{HirId, ItemKind, Mutability};
use rustc_index::vec::IndexVec;
use rustc_interface::interface::Compiler;
use rustc_interface::{Config, Queries};
use rustc_middle::hir::map::ParentHirIterator;
use rustc_middle::mir::interpret::{Allocation, ConstValue, Scalar};
use rustc_middle::mir::visit::MutVisitor;
use rustc_middle::mir::StatementKind::{Assign, SetDiscriminant};
use rustc_middle::mir::{BasicBlock, BasicBlockData, BinOp, Body, CastKind, Constant, ConstantKind, Local, LocalDecl, LocalDecls, Operand, Place, Rvalue, SourceInfo, SourceScope, Statement, StatementKind, SwitchTargets, Terminator, TerminatorKind, UnOp, START_BLOCK, PlaceElem};
use rustc_middle::ty;
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::{Const, ConstKind, ConstVid, List, ScalarInt, Ty, TyCtxt, UintTy};
use rustc_span::{Span, Symbol};
use rustc_target::abi::{Align, VariantIdx};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::Arguments;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::iter::FromIterator;
use std::path::PathBuf;
use uuid::Uuid;

type CutPoint<'tcx> = (BasicBlock, usize, BasicBlockData<'tcx>);

pub const CUSTOM_OPT_MIR_ANALYSIS: for<'tcx> fn(_: TyCtxt<'tcx>, _: DefId) -> &'tcx Body<'tcx> =
    |tcx, def| {
        let opt_mir = rustc_interface::DEFAULT_QUERY_PROVIDERS
            .borrow()
            .optimized_mir;
        let body = opt_mir(tcx, def).clone();
        let crate_name = tcx.crate_name(def.krate);
        let hir_id = tcx.hir().local_def_id_to_hir_id(def.expect_local());

        let testify_flags = get_testify_flags();
        let cut_name = get_cut_name(&testify_flags);

        if crate_name.as_str() != cut_name || is_testify_monitor(hir_id, &tcx) || !allowed_item(def) {
            // Don't instrument extern crates
            return tcx.arena.alloc(body);
        }

        //println!("Analyzing {:?}", def);

        let mut writer = MirWriter::new(MIR_LOG_PATH);
        let item_name = tcx.hir().opt_name(hir_id);
        if let None = item_name {
            return tcx.arena.alloc(body);
        };

        let global_id: u32 = def.index.into();
        writer.new_body(&format!("{}", global_id));
        let basic_blocks = body.basic_blocks();
        let blocks = basic_blocks
            .iter_enumerated()
            .map(|(block, data)| format!("{} -> {:?}", block.as_usize(), data))
            .collect::<Vec<_>>();

        /*for block in &blocks {
            println!("{}", block);
        }*/
        writer.write_basic_blocks(&blocks);

        let cdg = cdg(&body);
        writer.write_cdg(serde_json::to_string(&cdg).as_ref().unwrap());

        // INSTRUMENT
        let mut mir_visitor = MirVisitor::new(global_id as u64, body.clone(), tcx);

        let mut instrumented_body = mir_visitor.visit();
        let (basic_blocks, local_decls) = instrumented_body.basic_blocks_and_local_decls_mut();

        let locals = local_decls
            .iter_enumerated()
            .map(|(local, decl)| format!("{:?} -> {:?}", local, decl))
            .collect::<Vec<_>>();
        writer.write_locals(&locals);

        /*let blocks = basic_blocks
        .iter_enumerated()
        .map(|(block, data)| format!("{} -> {:?}", block.as_usize(), data))
        .collect::<Vec<_>>();*/
        //writer.write_basic_blocks(&blocks);

        let branches = serde_json::to_string(&mir_visitor.branches).unwrap();
        writer.write_branches(&branches);

        let op_enum = get_binary_op_def_id(&tcx);
        return tcx.arena.alloc(instrumented_body);
    };

pub const CUSTOM_OPT_MIR_INSTRUMENTATION: for<'tcx> fn(
    _: TyCtxt<'tcx>,
    _: DefId,
) -> &'tcx Body<'tcx> = |tcx, def| {
    let opt_mir = rustc_interface::DEFAULT_QUERY_PROVIDERS
        .borrow()
        .optimized_mir;
    let mut body = opt_mir(tcx, def).clone();
    let crate_name = tcx.crate_name(def.krate);
    let hir_id = tcx.hir().local_def_id_to_hir_id(def.expect_local());
    let testify_flags = get_testify_flags();
    let cut_name = get_cut_name(&testify_flags);

    if crate_name.as_str() != cut_name || is_testify_monitor(hir_id, &tcx) || !allowed_item(def) {
        // Don't instrument extern crates
        return tcx.arena.alloc(body);
    }

    println!(">> Instrumenting {:?}", def);

    let global_id: u32 = def.index.into();

    let (basic_blocks, local_decls) = body.basic_blocks_and_local_decls_mut();
    local_decls.iter_enumerated().for_each(|(local, decl)| {
        println!("{:?} -> {:?}", local, decl);
    });
    basic_blocks.iter_enumerated().for_each(|(block, data)| {
        println!("{:?} -> {:?}", block, data);
    });

    // INSTRUMENT
    let mut mir_visitor = MirVisitor::new(global_id as u64, body.clone(), tcx);
    let mut instrumented_body = mir_visitor.visit();

    let (basic_blocks, local_decls) = instrumented_body.basic_blocks_and_local_decls_mut();

    local_decls.iter_enumerated().for_each(|(local, decl)| {
        println!("{:?} -> {:?}", local, decl);
    });

    basic_blocks.iter_enumerated().for_each(|(block, data)| {
        println!("{:?} -> {:?}", block, data);
    });

    return tcx.arena.alloc(instrumented_body);
};

fn allowed_item(id: DefId) -> bool {
    let name = format!("{:?}", id);
    !(name.contains("serialize") || name.contains("deserialize"))
}

pub struct MirVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
    body: Body<'tcx>,
    // We need this to pretend this to be a global id since we cannot access anything outside
    // of the optimized_mir function
    global_id: u64,
    locals_num: usize,
    branch_counter: u64,
    branches: Vec<Branch>,
    cut_points: Vec<(BasicBlock, usize, Vec<(BasicBlock, BasicBlockData<'tcx>)>)>,
    basic_blocks_num: usize,
}

impl<'tcx> MirVisitor<'tcx> {
    fn new(global_id: u64, body: Body<'tcx>, tcx: TyCtxt<'tcx>) -> Self {
        MirVisitor {
            tcx,
            global_id,
            locals_num: body.local_decls.len(),
            basic_blocks_num: body.num_nodes(),
            body,
            branch_counter: 0,
            branches: vec![],
            cut_points: vec![],
        }
    }

    fn visit(&mut self) -> Body<'tcx> {
        let mut body = self.body.clone();
        self.visit_body(&mut body);
        body
    }

    fn get_switch_value(&self, switch_ty: Ty<'tcx>, value: u128) -> &'tcx Const<'tcx> {
        let param_env = ty::ParamEnv::empty();
        let switch_ty = self.tcx.lift(switch_ty).unwrap();
        let size = self.tcx.layout_of(param_env.and(switch_ty)).unwrap().size;
        ty::Const::from_scalar(self.tcx, Scalar::from_uint(value, size), switch_ty)
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

    fn mk_const(&self, ty: Ty<'tcx>, val: ConstKind<'tcx>) -> &'tcx Const<'tcx> {
        let constant = Const { ty, val };

        let interned_constant = self.tcx.mk_const(constant);
        interned_constant
    }

    fn mk_const_bool(&self, flag: bool) -> &'tcx Const<'tcx> {
        let bool_ty = self.tcx.types.bool;
        let data = ConstKind::Value(ConstValue::Scalar(Scalar::Int(ScalarInt::from(flag))));

        let const_arg = Const {
            ty: bool_ty,
            val: data,
        };

        let const_arg = self.tcx.mk_const(const_arg);
        const_arg
    }

    fn mk_move_operand(&self, local: Local) -> Operand<'tcx> {
        Operand::Move(Place::from(local))
    }

    fn mk_cast_local_as_f64_stmt(&self, from: Local, to: Local) -> Statement<'tcx> {
        let to_place = self.mk_place(to.index());
        let f64_ty = self.tcx.types.f64;

        let stmt_kind = StatementKind::Assign(Box::new((
            to_place,
            Rvalue::Cast(CastKind::Misc, self.mk_move_operand(from), f64_ty),
        )));

        Statement {
            source_info: self.mk_dummy_source_info(),
            kind: stmt_kind,
        }
    }

    fn mk_cast_const_as_f64_stmt(&self, operand: Operand<'tcx>, to: Local) -> Statement<'tcx> {
        let to_place = self.mk_place(to.index());
        let f64_ty = self.tcx.types.f64;

        let stmt_kind = StatementKind::Assign(Box::new((
            to_place,
            Rvalue::Cast(CastKind::Misc, operand, f64_ty),
        )));

        Statement {
            source_info: self.mk_dummy_source_info(),
            kind: stmt_kind,
        }
    }

    fn mk_move_stmt(&self) -> Statement<'tcx> {
        Statement {
            source_info: self.mk_dummy_source_info(),
            kind: StatementKind::Nop
        }
    }

    fn mk_const_int_operand(&self, data: u64) -> Operand<'tcx> {
        Operand::Constant(Box::new(Constant {
            span: Default::default(),
            user_ty: None,
            literal: ConstantKind::Ty(self.mk_const_int(data)),
        }))
    }

    fn mk_const_bool_operand(&self, flag: bool) -> Operand<'tcx> {
        Operand::Constant(Box::new(Constant {
            span: Default::default(),
            user_ty: None,
            literal: ConstantKind::Ty(self.mk_const_bool(flag)),
        }))
    }

    fn mk_const_operand(&self, ty: Ty<'tcx>, val: ConstKind<'tcx>) -> Operand<'tcx> {
        Operand::Constant(Box::new(Constant {
            span: Default::default(),
            user_ty: None,
            literal: ConstantKind::Ty(self.mk_const(ty, val)),
        }))
    }

    fn mk_call_terminator(
        &mut self,
        local_decls: &mut LocalDecls<'tcx>,
        args: Vec<Operand<'tcx>>,
        point_to: BasicBlock,
        fn_def_id: DefId,
    ) -> Terminator<'tcx> {
        let terminator_local = self.store_local_decl(local_decls, self.tcx.mk_unit());
        let terminator_place = self.mk_place(terminator_local.index());

        let fn_ty = self.tcx.type_of(fn_def_id);
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
            destination: Some((terminator_place, point_to)),
            cleanup: None,
            from_hir_call: false,
            fn_span: Default::default(),
        };

        let terminator = Terminator {
            source_info: self.mk_dummy_source_info(),
            kind: terminator_kind,
        };

        terminator
    }

    fn mk_basic_block(
        &self,
        stmts: Vec<Statement<'tcx>>,
        terminator: Terminator<'tcx>,
    ) -> BasicBlockData<'tcx> {
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

    fn mk_enum_var_stmt(&mut self, local: Local, variant_idx: u32) -> Statement<'tcx> {
        let place_idx = local.index();
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

    fn store_unit_local_decl(&mut self, local_decls: &mut LocalDecls<'tcx>) -> Local {
        let unit_ty = self.tcx.mk_unit();
        let local_decl = self.mk_local_decl(unit_ty);
        local_decls.push(local_decl);
        let local = Local::from(self.locals_num);
        self.locals_num += 1;
        local
    }

    fn store_local_decl(&mut self, local_decls: &mut LocalDecls<'tcx>, ty: Ty<'tcx>) -> Local {
        let local_decl = LocalDecl::new(ty, Span::default());
        local_decls.push(local_decl);
        let local = Local::from(self.locals_num);
        self.locals_num += 1;
        local
    }

    fn get_local_ty(&self, local_decls: &LocalDecls<'tcx>, local: Local) -> Ty<'tcx> {
        let decl = local_decls.get(local).unwrap();
        decl.ty
    }

    fn mk_trace_statements(
        &mut self,
        local_decls: &mut LocalDecls<'tcx>,
        block_id: usize,
        branch_id: u64,
        value_def: &ValueDef<'tcx>,
        switch_value: Option<&'tcx Const>,
        is_hit: bool,
    ) -> (Vec<Statement<'tcx>>, Vec<Operand<'tcx>>) {
        match value_def {
            ValueDef::BinaryOp(op, left, right) => {
                let op_def_id = get_binary_op_def_id(&self.tcx);
                let op_ty = self.tcx.type_of(op_def_id);
                let op_enum_local = self.store_local_decl(local_decls, op_ty);
                let op_def_stmt = self.mk_enum_var_stmt(op_enum_local, (*op).into());

                // If operand is a variable, then we have to create a new one and move the value
                // to use it later in the trace call. If it's a const, we can use it directly
                // as argument
                let (left_operand_stmt, left_local) = match left.as_ref() {
                    ValueDef::Const(ty, val) => {
                        let left_local = self.store_local_decl(local_decls, ty);
                        let operand = self.mk_const_operand(*ty, *val);

                        (
                            self.mk_cast_const_as_f64_stmt(operand, left_local),
                            left_local,
                        )
                    }
                    ValueDef::Var(place) => {
                        let left_ty = self.get_local_ty(local_decls, place.local);
                        let left_local = self.store_local_decl(local_decls, left_ty);
                        (
                            self.mk_cast_local_as_f64_stmt(place.local, left_local),
                            left_local,
                        )
                    }
                    _ => todo!("Operand is {:?}", left),
                };

                let (right_operand_stmt, right_local) = match right.as_ref() {
                    ValueDef::Const(ty, val) => {
                        let right_local = self.store_local_decl(local_decls, ty);
                        let operand = self.mk_const_operand(*ty, *val);

                        (
                            self.mk_cast_const_as_f64_stmt(operand, right_local),
                            right_local,
                        )
                    }
                    ValueDef::Var(place) => {
                        let right_ty = self.get_local_ty(local_decls, place.local);
                        let right_local = self.store_local_decl(local_decls, right_ty);
                        (
                            self.mk_cast_local_as_f64_stmt(place.local, right_local),
                            right_local,
                        )
                    }
                    _ => todo!("Operand is {:?}", right),
                };

                // We need to know whether we are executing a true or a false branch
                let branch_value_arg = if let Some(switch_value) = switch_value {
                    let flag = switch_value.val.try_to_bool().unwrap();
                    self.mk_const_bool_operand(flag)
                } else {
                    self.mk_const_bool_operand(true)
                };

                let stmts = vec![op_def_stmt, left_operand_stmt, right_operand_stmt];

                let trace_call_args = vec![
                    // Global id
                    self.mk_const_int_operand(self.global_id),
                    // Local id
                    self.mk_const_int_operand(branch_id),
                    // Block id
                    self.mk_const_int_operand(block_id as u64),
                    self.mk_move_operand(left_local),
                    self.mk_move_operand(right_local),
                    self.mk_move_operand(op_enum_local),
                    branch_value_arg,
                    self.mk_const_bool_operand(is_hit),
                ];
                (stmts, trace_call_args)
            }
            ValueDef::Const(ty, val) => {
                todo!("Const (ty: {:?}, val: {:?})", ty, val)
            }

            ValueDef::Discriminant(_) => {
                let stmts = vec![];

                let trace_call_args = vec![
                    // Global id
                    self.mk_const_int_operand(self.global_id),
                    // Local id
                    self.mk_const_int_operand(branch_id),
                    // Block id
                    self.mk_const_int_operand(block_id as u64),
                    self.mk_const_bool_operand(is_hit),
                ];

                (stmts, trace_call_args)
            }
            ValueDef::UnaryOp(op, inner_value_def) => match op {
                UnaryOp::Not => {
                    self.mk_trace_statements(local_decls, block_id, branch_id, inner_value_def, switch_value, !is_hit)
                },
                UnaryOp::Neg => todo!("Neg unary op"),
            },
            ValueDef::Call => {
                let stmts = vec![];
                let trace_call_args = vec![
                    // Global id
                    self.mk_const_int_operand(self.global_id),
                    // Local id
                    self.mk_const_int_operand(branch_id),
                    // Block id
                    self.mk_const_int_operand(block_id as u64),
                    self.mk_const_bool_operand(is_hit),
                ];
                (stmts, trace_call_args)
            }
            ValueDef::Field(place, deref) => {
                let stmts = vec![];
                let trace_call_args = vec![
                    // Global id
                    self.mk_const_int_operand(self.global_id),
                    // Local id
                    self.mk_const_int_operand(branch_id),
                    // Block id
                    self.mk_const_int_operand(block_id as u64),
                    self.mk_const_bool_operand(is_hit)
                ];

                (stmts, trace_call_args)
            }
            _ => todo!("Value def is {:?}", value_def),
        }
    }

    fn binary_branch(
        &mut self,
        local_decls: &mut LocalDecls<'tcx>,
        operand_def: &ValueDef<'tcx>,
        targets: &SwitchTargets,
        branch_ids: &Vec<u64>,
        my_branch_id: u64,
        switch_value: Option<&'tcx Const>,
        branch_ty: Ty<'tcx>,
        source_block: BasicBlock,
        target_block: BasicBlock,
    ) -> Vec<(BasicBlock, BasicBlockData<'tcx>)> {
        let mut blocks_sequence = Vec::with_capacity(targets.all_targets().len());
        let trace_fn = find_trace_fn_for(&self.tcx, &operand_def);

        for (idx, (value, target)) in targets.iter().enumerate() {
            let branch_id = *branch_ids.get(idx).unwrap();
            let is_hit = my_branch_id == branch_id;
            let (stmts, args) = self.mk_trace_statements(
                local_decls,
                source_block.as_usize(),
                branch_id,
                operand_def,
                switch_value,
                is_hit,
            );

            let current_block = BasicBlock::from(self.basic_blocks_num);
            self.basic_blocks_num += 1;
            let next_block = BasicBlock::from(self.basic_blocks_num);

            let terminator = self.mk_call_terminator(local_decls, args, next_block, trace_fn);
            let trace_block = self.mk_basic_block(stmts, terminator);
            blocks_sequence.push((current_block, trace_block));
        }

        let current_block = BasicBlock::from(self.basic_blocks_num);
        self.basic_blocks_num += 1;
        let branch_id = *branch_ids.last().unwrap();
        let is_hit = branch_id == my_branch_id;
        let (stmts, args) = self.mk_trace_statements(
            local_decls,
            source_block.as_usize(),
            branch_id,
            operand_def,
            switch_value,
            is_hit,
        );

        let terminator = self.mk_call_terminator(local_decls, args, target_block, trace_fn);

        blocks_sequence.push((current_block, self.mk_basic_block(stmts, terminator)));

        /*let branch = Branch::Decision(DecisionBranch::new(
            my_branch_id,
            source_block.as_usize(),
            target_block.as_usize(),
        ));
        self.branches.push(branch);*/

        blocks_sequence
    }

    fn get_place<'a>(&self, operand: &'a Operand<'tcx>) -> Option<&'a Place<'tcx>> {
        match operand {
            Operand::Copy(place) => Some(place),
            Operand::Move(place) => Some(place),
            _ => None,
        }
    }

    fn get_place_definition_from_stmt(
        &self,
        var: &Place<'tcx>,
        stmt: &Statement<'tcx>,
    ) -> Option<ValueDef<'tcx>> {
        match &stmt.kind {
            Assign(assign) => {
                let (place, value) = assign.as_ref();
                if place != var {
                    return None;
                }

                match value {
                    Rvalue::BinaryOp(op, operands) => {
                        let (left, right) = operands.as_ref();
                        return Some(ValueDef::BinaryOp(
                            to_binary_op(op),
                            Box::new(ValueDef::from(left)),
                            Box::new(ValueDef::from(right)),
                        ));
                    }
                    Rvalue::UnaryOp(op, operand) => {
                        return match operand {
                            Operand::Copy(place) | Operand::Move(place) => {
                                let inner_value_def = self.get_place_definition(place);
                                if let ValueDef::BinaryOp(op, left, right) = inner_value_def {
                                    return Some(ValueDef::BinaryOp(
                                        invert_binary_op(&op),
                                        left,
                                        right,
                                    ));
                                }
                                Some(ValueDef::UnaryOp(
                                    to_unary_op(op),
                                    Box::new(inner_value_def),
                                ))
                            }
                            Operand::Constant(_) => Some(ValueDef::UnaryOp(
                                to_unary_op(op),
                                Box::new(ValueDef::from(operand)),
                            )),
                        }
                    }
                    Rvalue::Use(operand) => match operand {
                        Operand::Constant(constant) => match &constant.literal {
                            ConstantKind::Ty(c) => {
                                return Some(ValueDef::Const(c.ty, c.val));
                            }
                            ConstantKind::Val(_, _) => todo!(),
                        },
                        Operand::Move(place) | Operand::Copy(place) => {
                            //let place = self.get_place(operand).unwrap();
                            //return Some(ValueDef::Var(place.clone()));
                            return Some(self.get_place_definition(place));
                        }
                    },
                    Rvalue::Discriminant(place) => {
                        return Some(ValueDef::Discriminant(*place));
                    }
                    _ => todo!("Value is {:?}", value),
                }
            }
            SetDiscriminant { place, .. } => {
                let place = place.as_ref();
                if var == place {
                    return Some(ValueDef::Discriminant(*place));
                }
            }
            _ => todo!(),
        }
        None
    }

    fn get_place_definition_from_terminator(
        &self,
        var: &Place<'tcx>,
        terminator: &Terminator<'tcx>,
    ) -> Option<ValueDef<'tcx>> {
        if let TerminatorKind::Call { destination, .. } = &terminator.kind {
            if let Some((place, _)) = destination {
                if place == var {
                    return Some(ValueDef::Call);
                }
            }
        }

        None
    }

    fn get_from_place_projection(&self, place: &Place<'tcx>) -> Option<ValueDef<'tcx>> {
        let projection = place.projection;
        let mut deref = projection.iter().any(|p| p == PlaceElem::Deref);
        let mut value_def = None;
        for p in projection {
            match p {
                PlaceElem::Field(_, _) => {
                    value_def = Some(ValueDef::Field(*place, deref));
                }
                _ => { }
            }
        }

        value_def
    }

    fn get_place_definition(&self, place: &Place<'tcx>) -> ValueDef<'tcx> {
        // TODO projection
        if !place.projection.is_empty() {
            if let Some(value_def) = self.get_from_place_projection(place) {
                return value_def;
            }
        }

        for data in self.body.basic_blocks() {
            let value_def = data
                .statements
                .iter()
                .find_map(|stmt| self.get_place_definition_from_stmt(place, stmt));

            if let Some(value_def) = value_def {
                return value_def;
            }

            if let Some(terminator) = &data.terminator {
                let value_def = self.get_place_definition_from_terminator(place, terminator);
                if let Some(value_def) = value_def {
                    return value_def;
                }
            }
        }



        todo!("No place definition found for {:?}, projection: {:?}", place, place.projection)
    }
}

impl<'tcx> MutVisitor<'tcx> for MirVisitor<'tcx> {
    fn visit_body(&mut self, body: &mut Body<'tcx>) {
        let (basic_blocks, local_decls) = body.basic_blocks_and_local_decls_mut();

        for (source_block, data) in basic_blocks.iter_enumerated_mut() {
            if let Some(terminator) = &mut data.terminator {
                match &mut terminator.kind {
                    TerminatorKind::SwitchInt {
                        discr,
                        switch_ty,
                        targets,
                    } => {
                        let operand_place = self
                            .get_place(discr)
                            .expect("Local has been defined in a previous block");
                        let operand_def = self.get_place_definition(operand_place);

                        if operand_def.is_const() {
                            let (ty, val) = operand_def.expect_const();
                            if ty.is_bool() {
                                // No need to instrument it since this will always be the same branch
                                continue;
                            }
                        }

                        let branch_ids = targets
                            .all_targets()
                            .iter()
                            .map(|_| self.next_branch_id())
                            .collect::<Vec<_>>();
                        let mut index = 0;
                        for (target_index, (value, target)) in targets.iter().enumerate() {
                            let branch_value = self.get_switch_value(switch_ty, value);
                            let branch_id = *branch_ids.get(target_index).unwrap();
                            let tracing_blocks = self.binary_branch(
                                local_decls,
                                &operand_def,
                                targets,
                                &branch_ids,
                                branch_id,
                                Some(branch_value),
                                switch_ty,
                                source_block.clone(),
                                target.clone(),
                            );
                            index = target_index;

                            self.cut_points
                                .push((source_block, target_index, tracing_blocks));
                        }

                        let branch_id = *branch_ids.last().unwrap();
                        let tracing_blocks = self.binary_branch(
                            local_decls,
                            &operand_def,
                            targets,
                            &branch_ids,
                            branch_id,
                            None,
                            switch_ty,
                            source_block.clone(),
                            targets.otherwise(),
                        );
                        self.cut_points
                            .push((source_block, index + 1, tracing_blocks));
                    }
                    _ => {}
                }
            }
        }

        for (source_basic_block, idx, tracing_blocks) in &self.cut_points {
            let (first_tracing_block, _) = tracing_blocks.first().unwrap();
            tracing_blocks.iter().for_each(|(_, data)| {
                let _ = basic_blocks.push(data.clone());
            });

            let block_data = basic_blocks.get_mut(*source_basic_block).unwrap();
            let terminator = block_data.terminator.as_mut().unwrap();
            self.update_terminator(terminator, *idx, *first_tracing_block);
        }
    }

    fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
        self.tcx.tcx()
    }
}

fn find_monitor_fn_by_name(tcx: &TyCtxt<'_>, name: &str) -> DefId {
    tcx.hir()
        .items()
        .find_map(|i| {
            if let ItemKind::Fn(_, _, _) = &i.kind {
                if i.ident.name.to_string() == name {
                    return Some(i.def_id.to_def_id());
                }
            }
            None
        })
        .unwrap()
}

fn find_trace_bool_fn(tcx: &TyCtxt<'_>) -> DefId {
    find_monitor_fn_by_name(tcx, "trace_branch_bool")
}

fn find_trace_enum_fn(tcx: &TyCtxt<'_>) -> DefId {
    find_monitor_fn_by_name(tcx, "trace_branch_enum")
}

fn find_trace_fn_for(tcx: &TyCtxt<'_>, value_def: &ValueDef<'_>) -> DefId {
    match value_def {
        ValueDef::BinaryOp(_, _, _) => find_trace_bool_fn(tcx),
        ValueDef::Discriminant(_) => find_trace_enum_fn(tcx),
        ValueDef::UnaryOp(_, inner_value_def) => find_trace_fn_for(tcx, inner_value_def.as_ref()),
        ValueDef::Field(_, _) => find_trace_enum_fn(tcx),
        ValueDef::Call => find_trace_enum_fn(tcx),
        _ => {
            todo!("Value def is {:?}", value_def)
        }
    }
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

fn get_binary_op_def_id(tcx: &TyCtxt<'_>) -> DefId {
    tcx.hir()
        .items()
        .find_map(|i| {
            if let ItemKind::Enum(_, _) = &i.kind {
                if i.ident.name.to_string() == "BinaryOp" {
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

fn to_binary_op(op: &BinOp) -> BinaryOp {
    match op {
        BinOp::Add => BinaryOp::Add,
        BinOp::Sub => BinaryOp::Sub,
        BinOp::Mul => BinaryOp::Mul,
        BinOp::Div => BinaryOp::Div,
        BinOp::Rem => BinaryOp::Rem,
        BinOp::BitXor => BinaryOp::BitXor,
        BinOp::BitAnd => BinaryOp::BitAnd,
        BinOp::BitOr => BinaryOp::BitOr,
        BinOp::Shl => BinaryOp::Shl,
        BinOp::Shr => BinaryOp::Shr,
        BinOp::Eq => BinaryOp::Eq,
        BinOp::Lt => BinaryOp::Lt,
        BinOp::Le => BinaryOp::Le,
        BinOp::Ne => BinaryOp::Ne,
        BinOp::Ge => BinaryOp::Ge,
        BinOp::Gt => BinaryOp::Gt,
        BinOp::Offset => BinaryOp::Offset,
    }
}

fn to_unary_op(op: &UnOp) -> UnaryOp {
    match op {
        UnOp::Not => UnaryOp::Not,
        UnOp::Neg => UnaryOp::Neg,
    }
}

fn invert_binary_op(op: &BinaryOp) -> BinaryOp {
    match op {
        BinaryOp::Eq => BinaryOp::Ne,
        BinaryOp::Lt => BinaryOp::Ge,
        BinaryOp::Le => BinaryOp::Gt,
        BinaryOp::Ne => BinaryOp::Eq,
        BinaryOp::Ge => BinaryOp::Lt,
        BinaryOp::Gt => BinaryOp::Le,
        _ => todo!("Should never happpen"),
    }
}

enum SwitchPath {
    Value(u128),
    Otherwise,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ValueDef<'a> {
    BinaryOp(BinaryOp, Box<ValueDef<'a>>, Box<ValueDef<'a>>),
    UnaryOp(UnaryOp, Box<ValueDef<'a>>),
    Const(Ty<'a>, ConstKind<'a>),
    Var(Place<'a>),
    Discriminant(Place<'a>),
    Call,
    // Deref?
    Field(Place<'a>, bool)
}

impl<'a> ValueDef<'a> {
    fn expect_const(&self) -> (&Ty<'a>, &ConstKind<'a>) {
        if let ValueDef::Const(ty, val) = self {
            return (ty, val);
        }
        panic!("Is not const");
    }

    fn is_const(&self) -> bool {
        if let ValueDef::Const(_, _) = self {
            return true;
        }

        false
    }

    fn is_var(&self) -> bool {
        if let ValueDef::Var(_) = self {
            return true;
        }

        false
    }

    fn expect_var(&self) -> Place<'a> {
        if let ValueDef::Var(place) = self {
            return *place;
        }

        panic!("Is not var");
    }
}

impl<'a> From<&Operand<'a>> for ValueDef<'a> {
    fn from(operand: &Operand<'a>) -> Self {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => ValueDef::Var(*place),
            Operand::Constant(constant) => match &constant.literal {
                ConstantKind::Ty(constant_ty) => {
                    let ty = constant_ty.ty;
                    let val = constant_ty.val;
                    ValueDef::Const(ty, val)
                }
                ConstantKind::Val(_, _) => todo!(),
            },
        }
    }
}
