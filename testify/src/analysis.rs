use crate::chromosome::{Callable, T};
#[derive(Debug)]
pub struct Analysis {
    callables: Vec<Callable>
}

impl Analysis {
    pub fn new(callables: Vec<Callable>) -> Self {
        Analysis {
            callables
        }
    }

    pub fn callables_of(&self, ty: &T) -> Vec<&Callable> {
        unimplemented!()
    }

    pub fn callables(&self) -> &Vec<Callable> {
        &self.callables
    }

    pub fn generators(&self, ty: &T) -> Vec<&Callable> {
        self.callables
            .iter()
            .filter(|&c| {
                let return_type = c.return_type();
                match return_type {
                    None => false,
                    Some(return_ty) => {
                        let res = ty == return_ty;
                        res
                    }
                }
            })
            .collect()

    }
}