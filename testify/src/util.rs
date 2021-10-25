use crate::chromosome::{ComplexT, Param, T};
use rustc_hir::def::Res;
use rustc_hir::def_id::DefId;
use rustc_hir::{FnRetTy, HirId, Impl, Item, ItemKind, Mutability, Node, PrimTy, QPath, Ty, TyKind};
use rustc_middle::ty::{TyCtxt, TypeckResults};
use std::io;
use std::option::Option::Some;
use std::path::PathBuf;
use rustc_middle::dep_graph::DepContext;
use rustc_span::{FileName, RealFileName, Span};
use syn::{FnArg, ImplItemMethod, Path, ReturnType, Type};

pub(crate) fn is_constructor(method: &ImplItemMethod) -> bool {
    method.sig.ident.to_string() == "new"
}

pub(crate) fn is_method(method: &ImplItemMethod) -> bool {
    method.sig.inputs.iter().any(|a| {
        if let FnArg::Receiver(_) = a {
            true
        } else {
            false
        }
    })
}

pub fn return_type_name(return_type: &ReturnType) -> Option<String> {
    match return_type {
        ReturnType::Default => None,
        ReturnType::Type(_, data_type) => Some(type_name(data_type.as_ref())),
    }
}

pub fn type_name(data_type: &Type) -> String {
    match data_type {
        Type::Path(type_path) => {
            let path = &type_path.path;
            merge_path(&path)
        }
        Type::Reference(type_reference) => type_name(type_reference.elem.as_ref()),
        _ => {
            unimplemented!()
        }
    }
}

pub fn merge_path(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub(crate) fn fn_arg_to_param(fn_arg: &FnArg, ty: &T) -> Param {
    unimplemented!()
}

pub fn by_reference(ty: &Box<Type>) -> bool {
    match ty.as_ref() {
        Type::Reference(_) => true,
        _ => false,
    }
}

pub fn mutable(ty: &Box<Type>) -> bool {
    match ty.as_ref() {
        Type::Reference(type_reference) => type_reference.mutability.is_some(),
        _ => false,
    }
}

pub fn cargo_path() -> io::Result<PathBuf> {
    match which::which("cargo") {
        Ok(p) => Ok(p),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
    }
}

pub fn fmt_path() -> io::Result<PathBuf> {
    match which::which("rustfmt") {
        Ok(p) => Ok(p),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
    }
}

pub fn ty_to_param(ty: &Ty, self_hir_id: HirId, tcx: &TyCtxt<'_>) -> Param {
    let (by_reference, mutable) = if let TyKind::Rptr(_, mut_ty) = &ty.kind {
        let mutable = mut_ty.mutbl == Mutability::Mut;
        (true, mutable)
    } else {
        (false, false)
    };

    let real_ty = ty_to_t(ty, self_hir_id, tcx);
    if real_ty.is_complex() {
        let def_id = tcx.hir().local_def_id(real_ty.id()).to_def_id();
        let original_ty = T::Complex(ComplexT::new(ty.hir_id, def_id, real_ty.name()));
        Param::new(real_ty, original_ty, by_reference, mutable)
    } else {
        Param::new(real_ty.clone(), real_ty, by_reference, mutable)
    }

}

pub fn ty_to_t(ty: &Ty, self_: HirId, tcx: &TyCtxt<'_>) -> T {
    match &ty.kind {
        TyKind::Rptr(_, mut_ty) => ty_to_t(mut_ty.ty, self_, tcx),
        TyKind::Path(q_path) => {
            match q_path {
                QPath::Resolved(_, path) => {
                    match path.res {
                        Res::Def(_, def_id) => {
                            let hir_id = tcx.hir().local_def_id_to_hir_id(def_id.expect_local());
                            let name = join_path_to_str(path);
                            let def_id = tcx.hir().local_def_id(hir_id).to_def_id();
                            let complex_ty = ComplexT::new(hir_id, def_id, name);
                            T::Complex(complex_ty)
                        }
                        Res::PrimTy(prim_ty) => T::from(prim_ty),
                        Res::SelfTy(trait_def_id, impl_) => {
                            // Self type, so replace it with the parent id
                            let name = node_to_name(&tcx.hir().get(self_), tcx);
                            let def_id = tcx.hir().local_def_id(self_).to_def_id();
                            T::Complex(ComplexT::new(self_, def_id, name))
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }
}

pub fn node_to_t(node: &Node<'_>, tcx: &TyCtxt<'_>) -> T {
    match node {
        Node::Item(item) => item_to_t(item, tcx),
        _ => unimplemented!(),
    }
}

pub fn item_to_t(item: &Item<'_>, tcx: &TyCtxt<'_>) -> T {
    match &item.kind {
        ItemKind::Impl(im) => ty_to_t(im.self_ty, item.hir_id(),  tcx),
        _ => unimplemented!(),
    }
}

pub fn fn_ret_ty_to_t(ret_ty: &FnRetTy, self_hir_id: HirId, tcx: &TyCtxt<'_>) -> Option<T> {
    match ret_ty {
        FnRetTy::DefaultReturn(_) => None,
        FnRetTy::Return(ty) => Some(ty_to_t(ty, self_hir_id, tcx)),
    }
}

pub fn join_path_to_str(path: &rustc_hir::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub fn node_to_name(node: &Node<'_>, tcx: &TyCtxt<'_>) -> String {
    match node {
        Node::Item(item) => item_to_name(item, tcx),
        _ => {
            println!("Returning {:?}", node);
            unimplemented!()
        },
    }
}

pub fn item_to_name(item: &Item<'_>, tcx: &TyCtxt<'_>) -> String {

    match &item.kind {
        ItemKind::Impl(im) => ty_to_name(im.self_ty, tcx),
        ItemKind::Struct(_,_) => {
            tcx.def_path_str(item.def_id.to_def_id())
        },
        _ => unimplemented!(),
    }
}

pub fn ty_to_name(ty: &Ty<'_>, tcx: &TyCtxt<'_>) -> String {
    match &ty.kind {
        TyKind::Path(path) => qpath_to_name(path, tcx),
        _ => unimplemented!(),
    }
}

pub fn qpath_to_name(qpath: &QPath<'_>, tcx: &TyCtxt<'_>) -> String {
    match qpath {
        QPath::Resolved(_, path) => {
            res_to_name(&path.res, tcx)
            /*path
                .segments
                .iter()
                .map(|s| s.ident.name.to_string())
                .collect::<Vec<String>>()
                .join("::")*/
        },
        _ => unimplemented!(),
    }
}

pub fn res_to_name(res: &Res, tcx: &TyCtxt<'_>) -> String {
    match res {
        Res::Def(_, def_id) => {
            tcx.def_path_str(*def_id)
        }
        _ => unimplemented!()
    }
}

pub fn impl_to_struct_id(im: &Impl) -> DefId {
    let self_ty = im.self_ty;
    match &self_ty.kind {
        TyKind::Path(qpath) => {
            match qpath {
                QPath::Resolved(_, path) => {
                    path.res.def_id()
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

pub fn span_to_path(span: &Span, tcx: &TyCtxt<'_>) -> PathBuf {
    let file_name = tcx.sess().source_map().span_to_filename(span.clone());
    match file_name {
        FileName::Real(real_file_name) => {
            match real_file_name {
                RealFileName::LocalPath(path) => path,
                RealFileName::Remapped { .. } => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}
