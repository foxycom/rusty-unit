use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Branch {
    Decision(DecisionBranch),
    Root(RootBranch)
}

impl Branch {
    pub fn id(&self) -> u64 {
        match self {
            Branch::Decision(branch) => branch.id,
            Branch::Root(branch) => branch.id
        }
    }

    pub fn is_decision(&self) -> bool {
        if let Branch::Decision(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_root(&self) -> bool {
        if let Branch::Root(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionBranch {
    id: u64,
    source: usize,
    target: usize
}

impl DecisionBranch {
    pub fn new(id: u64, source: usize, target: usize) -> Self {
        DecisionBranch { id, source, target }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RootBranch {
    id: u64
}

impl RootBranch {
}

impl Hash for RootBranch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Hash for DecisionBranch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.source.hash(state);
        self.target.hash(state);
    }
}

