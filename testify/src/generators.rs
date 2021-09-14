use syn::{FnArg, Expr, Type, Lit};
use proc_macro2::Ident;
use crate::chromosome::TestCase;
use crate::source::SourceFile;

#[derive(Debug)]
pub struct InputGenerator {}

impl InputGenerator {
    pub fn generate_arg(arg: &FnArg) -> Expr {
        if let FnArg::Typed(pattern) = arg {
            return match pattern.ty.as_ref() {
                Type::Path(path) => {
                    if path.path.is_ident("u8") {
                        let random_u64 = fastrand::u8(..);
                        let lit: Expr = syn::parse_quote! { #random_u64 };
                        lit
                    } else {
                        unimplemented!()
                    }
                }
                _ => {
                    let lit: Expr = syn::parse_quote! { 0 };
                    lit
                }
            };
        }
        let lit: Expr = syn::parse_quote! { 0 };
        lit
    }

    pub fn is_primitive(arg: &FnArg) -> bool {
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
                _ => {
                    unimplemented!()
                }
            };
        } else {
            unimplemented!()
        }
    }

    pub fn mutate_arg(arg: &Expr) -> Expr {
        match arg {
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
                        syn::parse_quote! {
                            #res
                        }
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

    pub fn mutate_arg_dist(arg: &Expr, dist: f64) -> Expr {
        match arg {
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
                        syn::parse_quote! {
                            #res
                        }
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
            id: Default::default()
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