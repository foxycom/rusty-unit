use std::collections::HashMap;
use std::sync::Arc;
use petgraph::Graph;
use crate::types::{Callable, T};
use serde::{Serialize, Deserialize};
use crate::branch::Branch;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirAnalysis {
    callables: Vec<Callable>
}

impl HirAnalysis {
    pub fn new() -> Self {
        HirAnalysis {
            callables: vec![]
        }
    }

    pub fn set_callables(&mut self, callables: Vec<Callable>) {
        self.callables = callables;
    }

    pub fn callables_of(&self, ty: &T) -> Vec<&Callable> {
        unimplemented!()
    }

    pub fn callables(&self) -> &Vec<Callable> {
        &self.callables
    }

    pub fn generators(&self, ty: &Arc<T>) -> Vec<&Callable> {
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

pub struct MirAnalysis {
    pub bodies: HashMap<u32, MirBody>
}

impl MirAnalysis {
    pub fn new() -> Self {
        MirAnalysis { bodies: HashMap::new() }
    }
}


#[derive(Debug, Clone)]
pub struct MirBody {
    pub branches: Vec<Branch>,
    pub cdg: Graph<usize, usize>
}

impl MirBody {
    pub fn new() -> Self {
        MirBody {
            branches: vec![], cdg: Default::default()
        }
    }
}