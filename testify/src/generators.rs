use crate::chromosome::{Arg, Param};
use syn::{Expr, FnArg, Lit, Type};

#[derive(Debug)]
pub struct PrimitivesGenerator {}

impl PrimitivesGenerator {
    pub fn generate_arg(param: &Param) -> Arg {
        if let FnArg::Typed(pattern) = param {
            return match pattern.ty.as_ref() {
                Type::Path(path) => {
                    if path.path.is_ident("u8") {
                        let random_u64 = fastrand::u8(..);
                        let lit: Expr = syn::parse_quote! { #random_u64 };
                        Arg::new(None, lit, param.clone(), true)
                    } else {
                        unimplemented!()
                    }
                }
                _ => {
                    let lit: Expr = syn::parse_quote! { 0 };
                    Arg::new(None, lit, param.clone(), true)
                }
            };
        }
        let lit: Expr = syn::parse_quote! { 0 };
        Arg::new(None, lit, param.clone(), true)
    }

    pub fn is_fn_arg_primitive(arg: &FnArg) -> bool {
        if let FnArg::Typed(pattern) = arg {
            return match pattern.ty.as_ref() {
                Type::Path(path) => {
                    return if path.path.is_ident("u8") {
                        // TODO add all other primitives
                        true
                    } else {
                        false
                    };
                }
                Type::Reference(reference) => {
                    PrimitivesGenerator::is_type_primitive(reference.elem.as_ref())
                }
                _ => {
                    unimplemented!()
                }
            };
        } else {
            unimplemented!()
        }
    }

    pub fn is_type_primitive(typee: &Type) -> bool {
        match typee {
            Type::Path(type_path) => {
                if type_path.path.is_ident("u8") {
                    true
                } else {
                    false
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn mutate_arg(arg: &Arg) -> Arg {
        let val = arg.value();
        match val {
            Expr::Lit(expr_lit) => {
                let lit = &expr_lit.lit;
                return match lit {
                    Lit::Int(int) => {
                        let n: u8 = int.base10_parse().unwrap();
                        let res = if fastrand::f64() < 0.5 {
                            n.overflowing_add(5).0
                        } else {
                            n.overflowing_sub(5).0
                        };
                        let new_expr: Expr = syn::parse_quote! {
                            #res
                        };
                        let mut modified_arg = arg.clone();
                        modified_arg.set_value(new_expr);
                        modified_arg
                    }
                    _ => {
                        unimplemented!()
                    }
                };
            }
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn mutate_arg_dist(arg: &Arg, dist: f64) -> Arg {
        let val = arg.value();
        match val {
            Expr::Lit(expr_lit) => {
                let lit = &expr_lit.lit;
                return match lit {
                    Lit::Int(int) => {
                        let n: u8 = int.base10_parse().unwrap();
                        let res = if fastrand::f64() < 0.5 {
                            n.overflowing_add(dist as u8).0
                        } else {
                            n.overflowing_sub(dist as u8).0
                        };
                        let new_expr: Expr = syn::parse_quote! {
                            #res
                        };
                        let mut modified_arg = arg.clone();
                        modified_arg.set_value(new_expr);
                        modified_arg
                    }
                    _ => {
                        unimplemented!()
                    }
                };
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TestIdGenerator {
    id: u64,
}

impl TestIdGenerator {
    pub fn new() -> TestIdGenerator {
        TestIdGenerator {
            id: Default::default(),
        }
    }

    pub fn next_id(&mut self) -> u64 {
        self.id += 1;
        self.id
    }

    pub fn reset(&mut self) {
        self.id = Default::default()
    }
}
