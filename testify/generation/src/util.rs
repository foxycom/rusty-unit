use rustc_hir::def::Res;
use rustc_hir::def_id::DefId;
use rustc_hir::{
    FnRetTy, HirId, Impl, Item, ItemKind, Mutability, Node, PrimTy, QPath, Ty, TyKind,
};
use rustc_middle::dep_graph::DepContext;
use rustc_middle::ty::{TyCtxt, TypeckResults};
use rustc_span::{FileName, RealFileName, Span};
use std::io;
use std::io::Write;
use std::option::Option::Some;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use syn::{FnArg, ImplItemMethod, Path, ReturnType, Type};
use crate::types::{ComplexT, Param, T};

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

pub fn ty_to_param(ty: &Ty, self_hir_id: HirId, tcx: &TyCtxt<'_>) -> Option<Param> {
    let (by_reference, mutable) = if let TyKind::Rptr(_, mut_ty) = &ty.kind {
        let mutable = mut_ty.mutbl == Mutability::Mut;
        (true, mutable)
    } else {
        (false, false)
    };

    let real_ty = ty_to_t(ty, self_hir_id, tcx)?;
    if real_ty.is_complex() {
        let def_id = tcx.hir().local_def_id(real_ty.expect_id()).to_def_id();
        let original_ty = T::Complex(ComplexT::new(ty.hir_id, def_id, real_ty.name()));
        Some(Param::new(real_ty, original_ty, by_reference, mutable))
    } else {
        Some(Param::new(real_ty.clone(), real_ty, by_reference, mutable))
    }
}

pub fn ty_to_t(ty: &Ty, self_: HirId, tcx: &TyCtxt<'_>) -> Option<T> {
    match &ty.kind {
        TyKind::Rptr(_, mut_ty) => ty_to_t(mut_ty.ty, self_, tcx),
        TyKind::Path(q_path) => {
            match q_path {
                QPath::Resolved(_, path) => {
                    match &path.res {
                        Res::Def(_, def_id) => {
                            let local_def_id = def_id.as_local()?;

                            let hir_id = tcx.hir().local_def_id_to_hir_id(def_id.expect_local());
                            let name = join_path_to_str(path);
                            let def_id = tcx.hir().local_def_id(hir_id).to_def_id();
                            let complex_ty = ComplexT::new(hir_id, def_id, name);
                            Some(T::Complex(complex_ty))
                        }
                        Res::PrimTy(prim_ty) => Some(T::from(*prim_ty)),
                        Res::SelfTy(trait_def_id, impl_) => {
                            // Self type, so replace it with the parent id
                            let name = node_to_name(&tcx.hir().get(self_), tcx).unwrap();
                            let def_id = tcx.hir().local_def_id(self_).to_def_id();
                            Some(T::Complex(ComplexT::new(self_, def_id, name)))
                        }
                        _ => {
                            unimplemented!("{:?}", &path.res)
                        }
                    }
                }
                QPath::TypeRelative(ty, path_segement) => {
                    println!("TODO parse type relative");
                    None
                }
                _ => unimplemented!("{:?}", q_path),
            }
        }
        _ => unimplemented!(),
    }
}

pub fn node_to_t(node: &Node<'_>, tcx: &TyCtxt<'_>) -> Option<T> {
    match node {
        Node::Item(item) => item_to_t(item, tcx),
        _ => unimplemented!(),
    }
}

pub fn item_to_t(item: &Item<'_>, tcx: &TyCtxt<'_>) -> Option<T> {
    match &item.kind {
        ItemKind::Impl(im) => ty_to_t(im.self_ty, item.hir_id(), tcx),
        _ => unimplemented!(),
    }
}

pub fn fn_ret_ty_to_t(ret_ty: &FnRetTy, self_hir_id: HirId, tcx: &TyCtxt<'_>) -> Option<T> {
    match ret_ty {
        FnRetTy::DefaultReturn(_) => None,
        FnRetTy::Return(ty) => ty_to_t(ty, self_hir_id, tcx),
    }
}

pub fn join_path_to_str(path: &rustc_hir::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub fn node_to_name(node: &Node<'_>, tcx: &TyCtxt<'_>) -> Option<String> {
    match node {
        Node::Item(item) => Some(item_to_name(item, tcx)),
        Node::Crate(_) => Some("crate".to_string()),
        Node::ForeignItem(fi) => Some(fi.ident.name.to_ident_string()),
        Node::ImplItem(ii) => Some(ii.ident.name.to_ident_string()),
        Node::TraitItem(ti) => Some(ti.ident.name.to_ident_string()),
        Node::Variant(v) => Some(v.ident.name.to_ident_string()),
        Node::Field(f) => Some(f.ident.name.to_ident_string()),
        Node::Lifetime(lt) => Some(lt.name.ident().name.to_ident_string()),
        Node::GenericParam(param) => Some(param.name.ident().name.to_ident_string()),
        _ => {
            None
        }
    }
}

pub fn item_to_name(item: &Item<'_>, tcx: &TyCtxt<'_>) -> String {
    match &item.kind {
        ItemKind::Impl(im) => ty_to_name(im.self_ty, tcx),
        ItemKind::Struct(_, _) => tcx.def_path_str(item.def_id.to_def_id()),
        ItemKind::Enum(_, _) => tcx.def_path_str(item.def_id.to_def_id()),
        _ => item.ident.name.to_ident_string(),
    }
}

pub fn ty_to_name(ty: &Ty<'_>, tcx: &TyCtxt<'_>) -> String {
    match &ty.kind {
        TyKind::Path(path) => qpath_to_name(path, tcx),
        TyKind::Rptr(_, mut_ty) => ty_to_name(mut_ty.ty, tcx),
        _ => unimplemented!("Trying to convert ty to name: {:?}", ty),
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
        }
        _ => unimplemented!(),
    }
}

pub fn res_to_name(res: &Res, tcx: &TyCtxt<'_>) -> String {
    match res {
        Res::Def(_, def_id) => tcx.def_path_str(*def_id),
        _ => unimplemented!(),
    }
}

pub fn impl_to_struct_id(im: &Impl) -> DefId {
    let self_ty = im.self_ty;
    ty_kind_to_struct_id(&self_ty.kind)
}

pub fn ty_kind_to_struct_id(kind: &TyKind<'_>) -> DefId {
    match kind {
        TyKind::Path(qpath) => match qpath {
            QPath::Resolved(_, path) => path.res.def_id(),
            _ => unimplemented!(),
        },
        TyKind::Rptr(lifetime, mut_ty) => {
            let ty = mut_ty.ty;
            ty_kind_to_struct_id(&ty.kind)

        }
        _ => unimplemented!("Trying to convert to struct: {:?}", kind),
    }
}

pub fn span_to_path(span: &Span, tcx: &TyCtxt<'_>) -> Option<PathBuf> {
    let file_name = tcx.sess().source_map().span_to_filename(span.clone());
    match file_name {
        FileName::Real(real_file_name) => match real_file_name {
            RealFileName::LocalPath(path) => Some(path),
            RealFileName::Remapped { .. } => None,
        },
        _ => unimplemented!(),
    }
}

fn fmt_string(source: &str) -> io::Result<String> {
    let rustfmt = fmt_path()?;
    let mut cmd = Command::new(&*rustfmt);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    let source = source.to_owned();
    let stdin_handle = std::thread::spawn(move || {
        let _ = child_stdin.write_all(source.as_bytes());
        source
    });

    let mut output = vec![];
    io::copy(&mut child_stdout, &mut output)?;
    let status = child.wait()?;
    let source = stdin_handle.join().unwrap();

    match String::from_utf8(output) {
        Ok(source) => match status.code() {
            Some(0) => Ok(source),
            Some(2) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Rustfmt parsing errors".to_string(),
            )),
            Some(3) => Ok(source),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal rustfmt error".to_string(),
            )),
        },
        Err(_) => Ok(source),
    }
}
