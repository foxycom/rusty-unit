use syn::{Path, Type, ReturnType, ImplItemMethod, FnArg};
use crate::chromosome::{Param, SelfParam, T, RegularParam};

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
        Type::Reference(_) => {
            unimplemented!()
        }
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
            let self_param = SelfParam::new(t, fn_arg.clone(), recv.reference.is_some());

            Param::Self_(self_param)
        }
        FnArg::Typed(typed) => {
            let syn_type = typed.ty.clone();
            let t = T::new(&type_name(syn_type.as_ref()), syn_type);

            // TODO this is not always true, self can also be typed, see doc about FnArg
            let regular_param = RegularParam::new(t, fn_arg.clone());
            Param::Regular(regular_param)
        }
    }
}