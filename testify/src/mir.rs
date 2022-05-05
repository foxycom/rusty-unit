use log::{debug, error, info, warn};
use rustc_data_structures::graph::WithNumNodes;
use rustc_hir::def_id::DefId;
use rustc_hir::{HirId, ItemKind};
use rustc_index::vec::IndexVec;
use rustc_middle::mir::interpret::{Allocation, ConstValue, Scalar};
use rustc_middle::mir::visit::{MutVisitor, TyContext};
use rustc_middle::mir::StatementKind::{Assign, SetDiscriminant};
use rustc_middle::mir::{
  BasicBlock, BasicBlockData, BinOp, Body, CastKind, Constant, ConstantKind, HasLocalDecls,
  Local, LocalDecl, LocalDecls, Location, Operand, Place, PlaceElem, Rvalue, SourceInfo,
  SourceScope, Statement, StatementKind, SwitchTargets, Terminator, TerminatorKind, UnOp,
};
use rustc_middle::ty;
use rustc_middle::ty::layout::{HasTyCtxt, LayoutOf, MaybeResult};
use rustc_middle::ty::{Const, ConstKind, ConstS, List, Region, RegionKind, ScalarInt, Ty, TyCtxt, TypeAndMut, UintTy};
use rustc_span::Span;
use rustc_target::abi::{Align, VariantIdx};
use std::borrow::Borrow;
use std::fs::File;
use std::ops::Add;
use std::path::Path;
use petgraph::dot::Dot;
use rustc_ast::Mutability;
use crate::mir::ValueDef::Var;
#[cfg(feature = "analysis")]
use crate::data_structures::{cdg, log_graph_to, visualize_graph};
#[cfg(feature = "analysis")]
use crate::writer::{MirObjectBuilder, MirObject, MirWriter};
use crate::{DOT_DIR, INSTRUMENTED_MIR_LOG_NAME, LOG_DIR, RuConfig};
use crate::data_structures::{original_cfg, truncated_cfg};
use crate::monitor::{BinaryOp, UnaryOp};

pub const CUSTOM_OPT_MIR: for<'tcx> fn(_: TyCtxt<'tcx>, _: DefId) -> &'tcx Body<'tcx> =
  |tcx, def| {
    let opt_mir = rustc_interface::DEFAULT_QUERY_PROVIDERS
        .borrow()
        .optimized_mir;
    let body = opt_mir(tcx, def).clone();
    let crate_name = tcx.crate_name(def.krate);
    let hir_id = tcx.hir().local_def_id_to_hir_id(def.expect_local());

    if crate_name.as_str() != RuConfig::env_crate_name() || is_rusty_monitor(hir_id, &tcx) || !allowed_item(def) {
      // Don't instrument extern crates
      return tcx.arena.alloc(body);
    }

    let item_name = tcx.hir().opt_name(hir_id);
    if let None = item_name {
      return tcx.arena.alloc(body);
    };

    let global_id = def_id_to_str(def, &tcx).replace("::", "__");

    #[cfg(feature = "analysis")]
    info!("MIR: Analyzing {:?}", def);

    #[cfg(feature = "analysis")]
        let (cfg, _) = truncated_cfg(&body);
    #[cfg(feature = "analysis")]
        let cdg = cdg(&cfg);

    #[cfg(feature = "analysis")]
    {
      let basic_blocks = body.basic_blocks();
      let basic_blocks_str = basic_blocks
          .iter_enumerated()
          .map(|(block, data)| format!("{} -> {:?}", block.as_usize(), data))
          .collect::<Vec<_>>();

      if cfg!(file_writer) {
        let path = Path::new(DOT_DIR).join(format!("{}.dot", &global_id));
        visualize_graph(&cdg, &global_id);
      }

      let locals_decls: &LocalDecls = &body.local_decls;
      let locals_str = locals_decls.iter_enumerated()
          .map(|(local, decl)| format!("{:?} -> {:?}", local, decl))
          .collect::<Vec<_>>();
      let (cfg, _) = original_cfg(&body);
      let (truncated_cfg, _) = truncated_cfg(&body);
      let mut mir_object = MirObjectBuilder::default()
          .global_id(global_id.to_owned())
          .basic_blocks(basic_blocks_str)
          .cdg(serde_json::to_string(&cdg).unwrap())
          .cfg(format!("{}", Dot::new(&cfg)))
          .truncated_cfg(format!("{}", Dot::new(&truncated_cfg)))
          .locals(locals_str)
          .build()
          .unwrap();

      MirWriter::write(&mir_object);
    }

    #[cfg(feature = "instrumentation")]
    {
      info!("MIR: Instrumenting {:?}", def);
    }

    // INSTRUMENT OR ANALYZE
    let mut mir_visitor = MirVisitor::new(&global_id, body.clone(), tcx);
    let mut instrumented_body = mir_visitor.visit();

    #[cfg(feature = "analysis")]
    {
      let (basic_blocks, local_decls) = instrumented_body.basic_blocks_and_local_decls_mut();

      let locals = local_decls
          .iter_enumerated()
          .map(|(local, decl)| format!("{:?} -> {:?}", local, decl))
          .collect::<Vec<_>>();

      let blocks = basic_blocks
          .iter_enumerated()
          .map(|(block, data)| format!("{} -> {:?}", block.as_usize(), data))
          .collect::<Vec<_>>();

      let instrumented_mir_object = MirObjectBuilder::default()
          .global_id(global_id)
          .locals(locals)
          .basic_blocks(blocks)
          .cdg(serde_json::to_string(&cdg).unwrap())
          .cfg("".to_string())
          .truncated_cfg("".to_string())
          .build()
          .unwrap();
      MirWriter::write_instrumented(&instrumented_mir_object);
    }

    return tcx.arena.alloc(instrumented_body);
  };

fn def_id_to_str(def_id: DefId, tcx: &TyCtxt<'_>) -> String {
  tcx.def_path_str(def_id)
}

fn allowed_item(id: DefId) -> bool {
  let name = format!("{:?}", id);
  !(name.contains("serialize") || name.contains("deserialize") || name.contains("tests"))
}

pub struct MirVisitor<'tcx> {
  tcx: TyCtxt<'tcx>,
  body: Body<'tcx>,
  // We need this to pretend this to be a global id since we cannot access anything outside
  // of the optimized_mir function
  global_id: String,
  locals_num: usize,
  branch_counter: u64,
  cut_points: Vec<(BasicBlock, usize, Vec<(BasicBlock, BasicBlockData<'tcx>)>)>,
  basic_blocks_num: usize,
  instrumentation: Vec<(BasicBlock, Vec<BasicBlockData<'tcx>>)>,
}

impl<'tcx> MirVisitor<'tcx> {
  fn new(global_id: &str, body: Body<'tcx>, tcx: TyCtxt<'tcx>) -> Self {
    MirVisitor {
      tcx,
      global_id: global_id.to_string(),
      locals_num: body.local_decls.len(),
      basic_blocks_num: body.num_nodes(),
      body,
      branch_counter: 0,
      cut_points: vec![],
      instrumentation: vec![],
    }
  }

  fn visit(&mut self) -> Body<'tcx> {
    let mut body = self.body.clone();
    self.visit_body(&mut body);
    body
  }

  fn switch_value_to_const(&self, switch_ty: Ty<'tcx>, value: u128) -> Const<'tcx> {
    let param_env = ty::ParamEnv::empty();
    let switch_ty = self.tcx.lift(switch_ty).unwrap();
    let size = self.tcx.layout_of(param_env.and(switch_ty)).unwrap().size;
    ty::Const::from_scalar(self.tcx, Scalar::from_uint(value, size), switch_ty)
  }

  fn mk_place(&self, index: usize) -> Place<'tcx> {
    Place {
      local: Local::from_usize(index),
      projection: List::empty(),
    }
  }

  fn mk_local_decl(&self, ty: Ty<'tcx>) -> LocalDecl<'tcx> {
    LocalDecl::new(ty, Span::default())
  }

  fn mk_const_int(&self, data: u64) -> Const<'tcx> {
    // let u64_ty = self.tcx.mk_mach_uint(UintTy::U64);
    // let scalar_data = ConstKind::Value(ConstValue::Scalar(Scalar::Int(<ScalarInt as From<u64>>::from(data))));
    //
    // let const_arg = Const {
    //   ty: u64_ty,
    //   val: scalar_data,
    // };

    let const_arg = Const::from_usize(self.tcx, data);
    const_arg
  }

  fn mk_const_str(&self, str: &str) -> Const<'tcx> {
    let str_ty = self.mk_str_ty();

    let allocation = Allocation::from_bytes_byte_aligned_immutable(str.as_bytes());
    // let val = ConstKind::Value(
    //   ConstValue::Slice {
    //     data: self.tcx.intern_const_alloc(allocation),
    //     start: 0,
    //     end: str.len(),
    //   }
    // );

    let val = ConstValue::Slice {
      data: self.tcx.intern_const_alloc(allocation),
      start: 0,
      end: str.len(),
    };

    Const::from_value(self.tcx, val, str_ty)
  }

  fn mk_const_bool(&self, flag: bool) -> Const<'tcx> {
    Const::from_bool(self.tcx, flag)
  }

  fn mk_str_ty(&self) -> Ty<'tcx> {
    let region = self.tcx.mk_region(RegionKind::ReErased);
    self.tcx.mk_ref(region, TypeAndMut { ty: self.tcx.types.str_, mutbl: Mutability::Not })
  }

  fn mk_move_operand(&self, local: Local) -> Operand<'tcx> {
    Operand::Move(<Place as From<Local>>::from(local))
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
      kind: StatementKind::Nop,
    }
  }

  fn mk_const_int_operand(&self, data: u64) -> Operand<'tcx> {
    Operand::Constant(Box::new(Constant {
      span: Default::default(),
      user_ty: None,
      literal: ConstantKind::Ty(self.mk_const_int(data)),
    }))
  }

  fn mk_const_str_operand(&self, str: &str) -> Operand<'tcx> {
    Operand::Constant(Box::new(Constant {
      span: Default::default(),
      user_ty: None,
      literal: ConstantKind::Ty(self.mk_const_str(str)),
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
    let const_s = ConstS {
      ty,
      val
    };

    Operand::Constant(Box::new(Constant {
      span: Default::default(),
      user_ty: None,
      literal: ConstantKind::Ty(self.tcx.mk_const(const_s)),
    }))
  }

  fn mk_call_terminator(
    &mut self,
    args: Vec<Operand<'tcx>>,
    point_to: BasicBlock,
    fn_def_id: DefId,
  ) -> Terminator<'tcx> {
    let terminator_local = self.store_local_decl(self.tcx.mk_unit());
    let terminator_place = self.mk_place(terminator_local.index());

    let fn_ty = self.tcx.type_of(fn_def_id);
    let func_const = Const::zero_sized(self.tcx, fn_ty);

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
      scope: SourceScope::from_usize(0usize),
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

  fn store_unit_local_decl(&mut self) -> Local {
    let unit_ty = self.tcx.mk_unit();
    let local_decl = self.mk_local_decl(unit_ty);
    let local_decls = &mut self.body.local_decls;
    local_decls.push(local_decl);
    let local = Local::from_usize(self.locals_num);
    self.locals_num += 1;
    local
  }

  fn store_local_decl(&mut self, ty: Ty<'tcx>) -> Local {
    let local_decls = &mut self.body.local_decls;
    let local_decl = LocalDecl::new(ty, Span::default());
    local_decls.push(local_decl);
    let local = Local::from_usize(self.locals_num);
    self.locals_num += 1;
    local
  }

  fn get_local_ty(&self, local: Local) -> Ty<'tcx> {
    let local_decls = self.body.local_decls();
    let decl = local_decls.get(local).unwrap();
    decl.ty
  }

  fn mk_trace_branch_hit(&mut self, target_block: usize) -> Vec<Operand<'tcx>> {
    let trace_call_args = vec![
      self.mk_const_str_operand(&self.global_id),
      self.mk_const_int_operand(target_block as u64),
    ];

    trace_call_args
  }

  fn mk_trace_statements_binary_op(
    &mut self,
    target_block: u64,
    op: &BinaryOp,
    left: &Box<ValueDef<'tcx>>,
    right: &Box<ValueDef<'tcx>>,
    is_true_branch: bool,
  ) -> (Vec<Statement<'tcx>>, Vec<Operand<'tcx>>) {
    let op_def_id = get_binary_op_def_id(&self.tcx);
    let op_ty = self.tcx.type_of(op_def_id);
    let op_enum_local = self.store_local_decl(op_ty);
    let op_def_stmt = self.mk_enum_var_stmt(op_enum_local, (*op).into());

    // If operand is a variable, then we have to create a new one and move the value
    // to use it later in the trace call. If it's a const, we can use it directly
    // as argument
    let (left_operand_stmt, left_local) = match left.as_ref() {
      ValueDef::Const(ty, val) => {
        let left_local = self.store_local_decl(*ty);
        let operand = self.mk_const_operand(*ty, *val);

        (
          self.mk_cast_const_as_f64_stmt(operand, left_local),
          left_local,
        )
      }
      ValueDef::Var(place) => {
        let left_ty = self.get_local_ty(place.local);
        let left_local = self.store_local_decl(left_ty);
        (
          self.mk_cast_local_as_f64_stmt(place.local, left_local),
          left_local,
        )
      }
      _ => todo!("Operand is {:?}", left),
    };

    let (right_operand_stmt, right_local) = match right.as_ref() {
      ValueDef::Const(ty, val) => {
        let right_local = self.store_local_decl(*ty);
        let operand = self.mk_const_operand(*ty, *val);

        (
          self.mk_cast_const_as_f64_stmt(operand, right_local),
          right_local,
        )
      }
      ValueDef::Var(place) => {
        let right_ty = self.get_local_ty(place.local);
        let right_local = self.store_local_decl(right_ty);
        (
          self.mk_cast_local_as_f64_stmt(place.local, right_local),
          right_local,
        )
      }
      _ => todo!("Operand is {:?}", right),
    };

    let stmts = vec![op_def_stmt, left_operand_stmt, right_operand_stmt];

    let trace_call_args = vec![
      // Global id
      self.mk_const_str_operand(&self.global_id),
      // Block id
      self.mk_const_int_operand(target_block),
      self.mk_move_operand(left_local),
      self.mk_move_operand(right_local),
      self.mk_move_operand(op_enum_local),
      self.mk_const_bool_operand(is_true_branch),
    ];
    (stmts, trace_call_args)
  }

  fn mk_trace_statements_entry(&mut self) -> Vec<Operand<'tcx>> {
    let args = vec![
      self.mk_const_str_operand(&self.global_id)
    ];

    args
  }

  fn mk_trace_statements_switch_int(
    &mut self,
    target_block: u64,
    value_def: &ValueDef<'tcx>,
    switch_value: Option<Const>,
    is_hit: bool,
  ) -> (Vec<Statement<'tcx>>, Vec<Operand<'tcx>>) {
    //debug!("MIR: mk_trace_statements_switch_int, {:?}", value_def);
    match value_def {
      ValueDef::BinaryOp(op, left, right) => {
        let is_true_branch = switch_value.is_none();
        self.mk_trace_statements_binary_op(target_block, op, left, right, is_true_branch)
      }
      ValueDef::Const(ty, val) => {
        (vec![], vec![])
      }

      ValueDef::Discriminant(_) => {
        let stmts = vec![];

        let trace_call_args = vec![
          // Global id
          self.mk_const_str_operand(&self.global_id),
          // Block id
          self.mk_const_int_operand(target_block as u64),
          self.mk_const_bool_operand(is_hit),
        ];

        (stmts, trace_call_args)
      }
      ValueDef::UnaryOp(op, inner_value_def) => match op {
        UnaryOp::Not => self.mk_trace_statements_switch_int(
          target_block,
          inner_value_def,
          switch_value,
          !is_hit,
        ),
        UnaryOp::Neg => todo!("Neg unary op"),
      },
      ValueDef::Call => {
        let stmts = vec![];
        let trace_call_args = vec![
          // Global id
          self.mk_const_str_operand(&self.global_id),
          // Block id
          self.mk_const_int_operand(target_block as u64),
          self.mk_const_bool_operand(is_hit),
        ];
        //panic!("Target block is {}", target_block as u64);
        (stmts, trace_call_args)
      }
      ValueDef::Field(place, deref) => {
        let stmts = vec![];
        let trace_call_args = vec![
          // Global id
          self.mk_const_str_operand(&self.global_id),
          // Block id
          self.mk_const_int_operand(target_block as u64),
          self.mk_const_bool_operand(is_hit),
        ];

        (stmts, trace_call_args)
      }
      ValueDef::Var(place) => {
        // ValueDef::Var means that we are directly comparing a variable to some
        // constant value, i.e., switch_value, so what we need to construct is
        // a trace to a binary EQ operation

        let stmts = vec![];
        let trace_call_args = vec![
          // Global id
          self.mk_const_str_operand(&self.global_id),
          // Block id
          self.mk_const_int_operand(target_block as u64),
          self.mk_const_bool_operand(is_hit),
        ];
        (stmts, trace_call_args)
      }
      _ => todo!("Value def is {:?}", value_def),
    }
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
              Box::new(ValueDef::from_operand(left)),
              Box::new(ValueDef::from_operand(right)),
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
                Box::new(ValueDef::from_operand(operand)),
              )),
            };
          }
          Rvalue::Use(operand) => match operand {
            Operand::Constant(constant) => match &constant.literal {
              ConstantKind::Ty(c) => {
                //return Some(ValueDef::Const(c.ty, c.val));
                // Don't return the direct const value, e.g., 2u8, but the
                // variable which stores the value. The value might change later
                // during the execution
                return Some(ValueDef::Var(*var));
              }
              ConstantKind::Val(const_value, ty) => {
                return Some(ValueDef::Const(*ty, ConstKind::Value(*const_value)));
              },
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
          Rvalue::Cast(_, operand, to_ty) => {
            return Some(match operand {
              Operand::Copy(place) => self.get_place_definition(place),
              Operand::Move(place) => self.get_place_definition(place),
              Operand::Constant(constant) => match &constant.literal {
                ConstantKind::Ty(c) => todo!("{:?}", c),
                ConstantKind::Val(const_value, ty) => ValueDef::Const(*ty, ConstKind::Value(*const_value))
              }
            });
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
        _ => {}
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
        //debug!("MIR: Place {:?} was defined by statement {:?}", place, value_def);
        return value_def;
      }

      if let Some(terminator) = &data.terminator {
        let value_def = self.get_place_definition_from_terminator(place, terminator);
        if let Some(value_def) = value_def {
          //debug!("MIR: Place {:?} was defined by terminator {:?}", place, value_def);
          return value_def;
        }
      }
    }

    for arg in self.body.args_iter() {
      if place.local == arg {
        return ValueDef::Var(<Place as From<rustc_middle::mir::Local>>::from(arg));
      }
    }

    todo!(
      "No place definition found for {:?}, projection: {:?}",
      place,
      place.projection
    )
  }

  fn shift_block_pointers(&self, body: &mut Body<'tcx>) {
    let basic_blocks = body.basic_blocks_mut();
    for basic_block in basic_blocks {
      if let Some(terminator) = &mut basic_block.terminator {
        match &mut terminator.kind {
          TerminatorKind::Goto { target } => {
            *target = *target + 1;
          }
          TerminatorKind::SwitchInt { targets, .. } => {
            for target in targets.all_targets_mut() {
              *target = *target + 1;
            }
          }
          TerminatorKind::Resume => {}
          TerminatorKind::Abort => {}
          TerminatorKind::Return => {}
          TerminatorKind::Unreachable => {}
          TerminatorKind::Drop { target, unwind, .. } => {
            *target = *target + 1;
            *unwind = unwind.map(|u| u + 1);
          }
          TerminatorKind::DropAndReplace { target, unwind, .. } => {
            *target = *target + 1;
            *unwind = unwind.map(|u| u + 1);
          }
          TerminatorKind::Call { destination, cleanup, .. } => {
            *destination = destination.map(|(place, bb)| (place, bb + 1));
            *cleanup = cleanup.map(|c| c + 1);
          }
          TerminatorKind::Assert { target, cleanup, .. } => {
            *target = *target + 1;
            *cleanup = cleanup.map(|c| c + 1);
          }
          TerminatorKind::Yield { resume, drop, .. } => {
            *resume = *resume + 1;
            *drop = drop.map(|d| d + 1);
          }
          TerminatorKind::GeneratorDrop => {}
          TerminatorKind::FalseEdge { real_target, imaginary_target } => {
            *real_target = *real_target + 1;
            *imaginary_target = *imaginary_target + 1;
          }
          TerminatorKind::FalseUnwind { real_target, unwind } => {
            *real_target = *real_target + 1;
            *unwind = unwind.map(|u| u + 1);
          }
          TerminatorKind::InlineAsm { destination, cleanup, .. } => {
            *destination = destination.map(|d| d + 1);
            *cleanup = cleanup.map(|c| c + 1);
          }
        }
      }
    }
  }
}

impl<'tcx> MirVisitor<'tcx> {
  fn instrument_first_block(&mut self, body: &mut Body<'tcx>) {
    self.basic_blocks_num += 1;

    // We have to shift all pointers by 1, like switch_int and so on
    self.shift_block_pointers(body);

    let args = self.mk_trace_statements_entry();
    let trace_fn = find_trace_entry_fn(&self.tcx);
    let terminator = self.mk_call_terminator(args, BasicBlock::from_usize(1usize), trace_fn);
    let trace_block = self.mk_basic_block(vec![], terminator);

    let basic_blocks = body.basic_blocks_mut();
    basic_blocks.raw.insert(0, trace_block);
  }

  fn instrument_switch_int(
    &mut self,
    terminator: &mut Terminator<'tcx>,
    source_block: BasicBlock,
  ) {
    let mut instrumentation = match &mut terminator.kind {
      TerminatorKind::SwitchInt {
        discr,
        switch_ty,
        targets,
      } => {
        debug!("MIR: Instrumenting switch int");
        let switch_operand_place = self
            .get_place(discr)
            .expect("Place has been defined in a previous block");
        let switch_operand_def = self.get_place_definition(switch_operand_place);

        if switch_operand_def.is_const() {
          let (ty, val) = switch_operand_def.expect_const();
          if ty.is_bool() {
            // No need to instrument it since this will always be the same branch
            return;
          }
        }

        let branch_ids = targets
            .all_targets()
            .iter()
            // Shift ids back because they are off by one due to root instrumentation
            .map(|t| t.as_u32() as u64)
            .collect::<Vec<u64>>();

        //debug!("MIR: Branch ids are {:?}", branch_ids);

        let mut instrumentation = Vec::with_capacity(targets.all_targets().len());
        let mut all_targets = targets
            .iter()
            .map(|(switch_value, target_block)| (Some(switch_value), target_block))
            .collect::<Vec<_>>();
        all_targets.push((None, *targets.all_targets().last().unwrap()));

        // Switch value is like false (0), or some numeric value, e.g., when comparing x == 2
        for (idx, (switch_value, target_block)) in all_targets.iter().enumerate() {
          /*debug!(
              "MIR: Creating a tracing chain which points to {}",
              target_block.as_usize() as u64
          );*/
          let switch_value_const =
              switch_value.map(|sv| self.switch_value_to_const(*switch_ty, sv));
          let (first_tracing_block, tracing_chain) = self
              .mk_tracing_chain_from_switch_int(
                &switch_operand_def,
                &branch_ids,
                switch_value_const,
                *target_block,
              );
          /*debug!(
              "MIR: First block (bb{})hain is: {:?}",
              first_tracing_block.as_usize(),
              tracing_chain
          );*/
          instrumentation.push((first_tracing_block, tracing_chain));
        }

        instrumentation
      }
      _ => panic!("Not a switch int"),
    };

    for (idx, (first_block, _)) in instrumentation.iter().enumerate() {
      self.update_terminator(terminator, idx, *first_block);
    }

    self.instrumentation.append(&mut instrumentation);
  }

  fn mk_tracing_chain_from_switch_int(
    &mut self,
    switch_operand_def: &ValueDef<'tcx>,
    branch_to_trace_ids: &Vec<u64>,
    switch_value: Option<Const>,
    target_block: BasicBlock,
  ) -> (BasicBlock, Vec<BasicBlockData<'tcx>>) {
    let mut tracing_chain = Vec::with_capacity(branch_to_trace_ids.len());
    let trace_fn = find_trace_fn_for(&self.tcx, &switch_operand_def);
    let first_block = BasicBlock::from_usize(self.basic_blocks_num);

    let mut branches = branch_to_trace_ids.iter().peekable();
    while let Some(&branch_to_trace_id) = branches.next() {
      let is_branch_hit = branch_to_trace_id == target_block.as_u32() as u64;
      self.basic_blocks_num += 1;

      let next_block = if branches.peek().is_some() {
        BasicBlock::from_usize(self.basic_blocks_num)
      } else {
        // If this is the last element in the tracing chain, then point to the
        // original basic block
        target_block
      };

      //debug!("MIR: Next block in chain is {}", next_block.as_usize());

      if is_branch_hit {
        let args = self.mk_trace_branch_hit(target_block.as_usize());
        let trace_fn = find_trace_branch_hit_fn(&self.tcx);
        let terminator = self.mk_call_terminator(args, next_block, trace_fn);
        let trace_block = self.mk_basic_block(Vec::new(), terminator);
        tracing_chain.push(trace_block);
      } else {
        let (stmts, args) = self.mk_trace_statements_switch_int(
          branch_to_trace_id,
          switch_operand_def,
          switch_value,
          is_branch_hit,
        );
        let terminator = self.mk_call_terminator(args, next_block, trace_fn);
        let trace_block = self.mk_basic_block(stmts, terminator);
        tracing_chain.push(trace_block);
      }
    }

    (first_block, tracing_chain)
  }
}

impl<'tcx> MutVisitor<'tcx> for MirVisitor<'tcx> {
  fn visit_body(&mut self, body: &mut Body<'tcx>) {
    self.super_body(body);

    #[cfg(feature = "instrumentation")]
    {
      // Now push the tracing chains after they have created
      for (_, tracing_chain) in &self.instrumentation {
        let basic_blocks = body.basic_blocks_mut();
        tracing_chain.iter().for_each(|tb| {
          let _ = basic_blocks.push(tb.clone());
        });
      }

      self.instrument_first_block(body);


      // Also apply local definitions
      body.local_decls = self.body.local_decls.clone();
    }
  }

  #[cfg(feature = "instrumentation")]
  fn visit_basic_block_data(&mut self, block: BasicBlock, data: &mut BasicBlockData<'tcx>) {
    if let Some(terminator) = &mut data.terminator {
      match &mut terminator.kind {
        // Instrument branching
        TerminatorKind::SwitchInt { .. } => {
          debug!("MIR: (bb{}) switch int", block.as_usize());
          self.instrument_switch_int(terminator, block);
        }
        _ => {}
      }
    }
  }

  fn tcx<'a>(&'a self) -> TyCtxt<'tcx> {
    self.tcx.tcx()
  }

  /*fn visit_const(&mut self, constant: &mut &'tcx Const<'tcx>, _: Location) {
      info!("MIR: Visiting const");
  }*/

  /*fn visit_constant(&mut self, constant: &mut Constant<'tcx>, location: Location) {
      let Constant {
          span,
          user_ty,
          literal,
      } = constant;

      info!("MIR: Found constant {:?}", literal);
      self.visit_span(span);
      match literal {
          ConstantKind::Ty(ct) => self.visit_const(ct, location),
          ConstantKind::Val(_, t) => self.visit_ty(t, TyContext::Location(location)),
      }
  }*/
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
      .expect(&format!(
        "Could not find rusty_monitor::{} in the crate",
        name
      ))
}

fn find_trace_bool_fn(tcx: &TyCtxt<'_>) -> DefId {
  find_monitor_fn_by_name(tcx, "trace_branch_bool")
}

fn find_trace_entry_fn(tcx: &TyCtxt<'_>) -> DefId {
  find_monitor_fn_by_name(tcx, "trace_entry")
}

fn find_trace_enum_fn(tcx: &TyCtxt<'_>) -> DefId {
  find_monitor_fn_by_name(tcx, "trace_branch_enum")
}

fn find_trace_const(tcx: &TyCtxt<'_>) -> DefId {
  find_monitor_fn_by_name(tcx, "trace_const")
}

fn find_trace_fn_for(tcx: &TyCtxt<'_>, value_def: &ValueDef<'_>) -> DefId {
  match value_def {
    ValueDef::BinaryOp(_, _, _) => find_trace_bool_fn(tcx),
    ValueDef::Discriminant(_) => find_trace_enum_fn(tcx),
    ValueDef::UnaryOp(_, inner_value_def) => find_trace_fn_for(tcx, inner_value_def.as_ref()),
    ValueDef::Field(_, _) => find_trace_enum_fn(tcx),
    ValueDef::Call => find_trace_enum_fn(tcx),
    ValueDef::Var(_) => find_trace_enum_fn(tcx),
    ValueDef::Const(_, _) => find_trace_const(tcx),
    _ => {
      todo!("Value def is {:?}", value_def)
    }
  }
}

fn find_trace_branch_hit_fn(tcx: &TyCtxt<'_>) -> DefId {
  find_monitor_fn_by_name(tcx, "trace_branch_hit")
}

fn is_rusty_monitor(hir_id: HirId, tcx: &TyCtxt<'_>) -> bool {
  let name = format!("{:?}", hir_id);
  name.contains("rusty_monitor")
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
        Option::None
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
  Field(Place<'a>, bool),
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

  fn from_operand(operand: &Operand<'a>) -> ValueDef<'a> {
    <Self as From<&Operand>>::from(operand)
  }
}

impl<'a> From<&Operand<'a>> for ValueDef<'a> {
  fn from(operand: &Operand<'a>) -> Self {
    match operand {
      Operand::Copy(place) | Operand::Move(place) => ValueDef::Var(*place),
      Operand::Constant(constant) => match &constant.literal {
        ConstantKind::Ty(constant_ty) => {
          let ty = constant_ty.ty();
          let val = constant_ty.val();
          ValueDef::Const(ty, val)
        }
        ConstantKind::Val(const_value, ty) => ValueDef::Const(*ty, ConstKind::Value(*const_value)),
      },
    }
  }
}
