use crate::chromosome::{Param, RegularParam, SelfParam, T};
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

pub(crate) fn fn_arg_to_param(fn_arg: &FnArg, syn_type: Box<Type>) -> Param {

    match fn_arg {
        FnArg::Receiver(recv) => {
            let t = T::new(&type_name(syn_type.as_ref()), syn_type);
            let self_param = SelfParam::new(
                t,
                fn_arg.clone(),
                recv.reference.is_some(),
                recv.mutability.is_some()
            );

            Param::Self_(self_param)
        }
        FnArg::Typed(typed) => {
            let syn_type = typed.ty.clone();
            let t = T::new(&type_name(syn_type.as_ref()), syn_type);

            // TODO this is not always true, self can also be typed, see doc about FnArg
            let regular_param = RegularParam::new(
                t,
                fn_arg.clone(),
                by_reference(&typed.ty),
                mutable(&typed.ty)
            );
            Param::Regular(regular_param)
        }
    }
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
        _ => false
    }
}

