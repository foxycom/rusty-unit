use std::fmt::{Display, Formatter};
use rustc_hir::{BodyId, FnSig, HirId, PrimTy};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use syn::Type;
use crate::chromosome::{Arg, AssignStmt, ConstructorStmt, FieldAccessStmt, FnInvStmt, MethodInvStmt, Statement, StaticFnInvStmt};

#[derive(Debug, Clone)]
pub enum Callable {
    Method(MethodItem),
    StaticFunction(StaticFnItem),
    Function(FunctionItem),
    Constructor(ConstructorItem),
    Primitive(PrimitiveItem),
    FieldAccess(FieldAccessItem),
}

impl Callable {
    pub fn params(&self) -> &Vec<Param> {
        match self {
            Callable::Method(method_item) => &method_item.params,
            Callable::StaticFunction(fn_item) => &fn_item.params,
            Callable::Function(fn_item) => &fn_item.params,
            Callable::Constructor(constructor_item) => &constructor_item.params,
            Callable::Primitive(primitive_item) => primitive_item.params(),
            Callable::FieldAccess(_) => unimplemented!(),
        }
    }

    pub fn params_mut(&mut self) -> &mut Vec<Param> {
        match self {
            Callable::Method(m) => &mut m.params,
            Callable::StaticFunction(f) => &mut f.params,
            Callable::Function(f) => &mut f.params,
            Callable::Constructor(c) => &mut c.params,
            Callable::Primitive(p) => &mut p.params,
            Callable::FieldAccess(_) => unimplemented!(),
        }
    }

    pub fn return_type(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => method_item.return_type.as_ref(),
            Callable::StaticFunction(fn_item) => fn_item.return_type.as_ref(),
            Callable::Function(fn_item) => fn_item.return_type.as_ref(),
            Callable::Constructor(constructor_item) => Some(&constructor_item.return_type()),
            Callable::Primitive(primitive_item) => Some(&primitive_item.ty),
            Callable::FieldAccess(field_access_item) => Some(&field_access_item.ty),
        }
    }

    pub fn parent(&self) -> Option<&T> {
        match self {
            Callable::Method(method_item) => Some(&method_item.parent),
            Callable::StaticFunction(fn_item) => Some(&fn_item.parent),
            Callable::Function(_) => None,
            Callable::Constructor(constructor) => Some(&constructor.parent),
            Callable::Primitive(_) => None,
            Callable::FieldAccess(field_access_item) => Some(&field_access_item.parent),
        }
    }

    pub fn to_stmt(&self, args: Vec<Arg>) -> Statement {
        match self {
            Callable::Method(method_item) => {
                Statement::MethodInvocation(MethodInvStmt::new(method_item.clone(), args))
            }
            Callable::StaticFunction(fn_item) => {
                Statement::StaticFnInvocation(StaticFnInvStmt::new(fn_item.clone(), args))
            }
            Callable::Function(fn_item) => {
                Statement::FunctionInvocation(FnInvStmt::new(fn_item.clone(), args))
            }
            Callable::Constructor(constructor_item) => {
                Statement::Constructor(ConstructorStmt::new(constructor_item.clone(), args))
            }
            Callable::Primitive(primitive_item) => {
                Statement::PrimitiveAssignment(AssignStmt::new(primitive_item.clone()))
            }
            Callable::FieldAccess(field_access_item) => {
                Statement::FieldAccess(FieldAccessStmt::new(field_access_item.clone()))
            }
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Callable::Method(m) => m.name(),
            Callable::StaticFunction(f) => f.name(),
            Callable::Function(f) => f.name(),
            Callable::Constructor(c) => "new",
            Callable::Primitive(_) => unimplemented!(),
            Callable::FieldAccess(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrimitiveItem {
    pub ty: T,
    pub params: Vec<Param>,
}

impl PrimitiveItem {
    pub fn new(ty: T) -> PrimitiveItem {
        PrimitiveItem { ty, params: vec![] }
    }

    pub fn params(&self) -> &Vec<Param> {
        // Just for compilation reasons
        &self.params
    }
}

#[derive(Debug, Clone)]
pub struct MethodItem {
    pub params: Vec<Param>,
    pub return_type: Option<T>,
    pub parent: T,
    pub body_id: BodyId,
    pub name: String,
    pub fn_id: HirId,
    pub src_file_id: usize,
}

impl MethodItem {
    pub fn new(
        src_file_id: usize,
        params: Vec<Param>,
        return_type: Option<T>,
        parent: T,
        body_id: BodyId,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();

        MethodItem {
            src_file_id,
            params,
            parent,
            return_type,
            body_id,
            name,
            fn_id,
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
    pub fn return_type(&self) -> Option<&T> {
        self.return_type.as_ref()
    }
    pub fn parent(&self) -> &T {
        &self.parent
    }


    pub fn consumes_parent(&self) -> bool {
        !self.params.first().unwrap().by_reference()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct FunctionItem {
    pub params: Vec<Param>,
    pub return_type: Option<T>,
    pub body_id: BodyId,
    pub fn_id: HirId,
    pub name: String,
    pub src_file_id: usize,
}

impl FunctionItem {
    pub fn new(
        src_file_id: usize,
        params: Vec<Param>,
        return_type: Option<T>,
        body_id: BodyId,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();


        FunctionItem {
            src_file_id,
            params,
            return_type,
            body_id,
            name,
            fn_id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct StaticFnItem {
    pub params: Vec<Param>,
    pub return_type: Option<T>,
    pub parent: T,
    pub body_id: BodyId,
    pub fn_id: HirId,
    pub name: String,
    pub src_file_id: usize,
}

impl StaticFnItem {
    pub fn new(
        src_file_id: usize,
        params: Vec<Param>,
        return_type: Option<T>,
        parent: T,
        body_id: BodyId,
        fn_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(fn_id).ident().unwrap();
        let name = ident.name.to_string();

        StaticFnItem {
            src_file_id,
            params,
            parent,
            return_type,
            body_id,
            fn_id,
            name,
        }
    }

    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
    pub fn return_type(&self) -> Option<&T> {
        self.return_type.as_ref()
    }
    pub fn parent(&self) -> &T {
        &self.parent
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct FieldAccessItem {
    pub ty: T,
    pub field_id: HirId,
    pub parent: T,
    pub name: String,
    pub src_file_id: usize,
}

impl FieldAccessItem {
    pub fn new(
        src_file_id: usize,
        ty: T,
        parent: T,
        field_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        let ident = tcx.hir().get(field_id).ident().unwrap();
        let name = ident.name.to_string();

        FieldAccessItem {
            src_file_id,
            name,
            ty,
            parent,
            field_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstructorItem {
    pub params: Vec<Param>,
    pub parent: T,
    pub body_id: BodyId,
    pub fn_id: HirId,
    pub src_file_id: usize,
}

impl ConstructorItem {
    pub fn new(
        src_file_id: usize,
        fn_sig: &FnSig,
        body_id: BodyId,
        fn_id: HirId,
        parent_hir_id: HirId,
        tcx: &TyCtxt<'_>,
    ) -> Self {
        todo!()
        /*
        ConstructorItem {
            parent,
            params,
            body_id,
            fn_id
        }*/
    }

    pub fn params(&self) -> &Vec<Param> {
        self.params.as_ref()
    }
    pub fn return_type(&self) -> &T {
        &self.parent
    }
    pub fn parent(&self) -> &T {
        &self.parent
    }
}

#[derive(Debug, Clone, Hash, Eq)]
pub enum T {
    Prim(PrimTy),
    Complex(ComplexT),
}

impl PartialEq for T {
    fn eq(&self, other: &Self) -> bool {
        match self {
            T::Prim(prim) => match other {
                T::Prim(other_prim) => prim == other_prim,
                T::Complex(_) => false,
            },
            T::Complex(comp) => match other {
                T::Prim(_) => false,
                T::Complex(other_comp) => comp == other_comp,
            },
        }
    }
}

impl From<PrimTy> for T {
    fn from(ty: PrimTy) -> Self {
        T::Prim(ty)
    }
}

impl T {
    pub fn syn_type(&self) -> &Type {
        unimplemented!()
    }

    pub fn name(&self) -> String {
        match self {
            T::Prim(prim) => {
                // TODO only for debugging
                format!("{:?}", prim)
            }
            T::Complex(complex) => complex.name().to_string(),
        }
    }

    pub fn var_string(&self) -> String {
        match self {
            T::Prim(prim) => prim.name_str().to_string(),
            T::Complex(complex) => complex.name().split("::").last().unwrap().to_string(),
        }
    }

    pub fn id(&self) -> HirId {
        match self {
            T::Prim(_) => unimplemented!(),
            T::Complex(complex) => complex.hir_id(),
        }
    }

    pub fn is_prim(&self) -> bool {
        match self {
            T::Prim(_) => false,
            T::Complex(_) => true,
        }
    }

    pub fn is_complex(&self) -> bool {
        match self {
            T::Prim(_) => false,
            T::Complex(_) => true,
        }
    }
}

impl Display for T {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            T::Prim(prim) => write!(f, "{}", prim.name_str()),
            T::Complex(complex) => write!(f, "{}", complex.name()),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq)]
pub struct ComplexT {
    hir_id: HirId,
    def_id: DefId,
    name: String,
}

impl PartialEq for ComplexT {
    fn eq(&self, other: &Self) -> bool {
        self.hir_id == other.hir_id
    }
}

impl ComplexT {
    pub fn new(hir_id: HirId, def_id: DefId, name: String) -> Self {
        ComplexT {
            hir_id,
            name,
            def_id,
        }
    }
    pub fn hir_id(&self) -> HirId {
        self.hir_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn def_id(&self) -> DefId {
        self.def_id
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    real_ty: T,
    original_ty: T,
    by_reference: bool,
    mutable: bool,
}

impl Param {
    pub fn new(real_ty: T, original_ty: T, by_reference: bool, mutable: bool) -> Self {
        Param {
            real_ty,
            original_ty,
            by_reference,
            mutable,
        }
    }

    pub fn is_self(&self) -> bool {
        todo!()
    }

    pub fn by_reference(&self) -> bool {
        self.by_reference
    }

    pub fn mutable(&self) -> bool {
        self.mutable
    }

    pub fn real_ty(&self) -> &T {
        &self.real_ty
    }

    pub fn real_ty_mut(&mut self) -> &mut T {
        todo!()
    }

    pub fn is_primitive(&self) -> bool {
        match self.real_ty {
            T::Prim(_) => true,
            T::Complex(_) => false,
        }
    }
}