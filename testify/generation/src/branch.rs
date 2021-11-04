use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};
use crate::chromosome::Chromosome;
use crate::fitness::FitnessValue;

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

#[derive(Debug, Clone)]
pub struct BranchManager {
    branches: Vec<Branch>,
    uncovered_branches: Vec<Branch>,
}

impl BranchManager {
    pub fn new(branches: &[Branch]) -> Self {
        BranchManager {
            branches: branches.to_vec(),
            uncovered_branches: branches.to_vec(),
        }
    }

    pub fn branches(&self) -> &Vec<Branch> {
        &self.branches
    }
    pub fn uncovered_branches(&self) -> &Vec<Branch> {
        &self.uncovered_branches
    }

    pub fn set_branches(&mut self, branches: &[Branch]) {
        self.branches = branches.to_vec();
    }

    pub fn set_current_population<C: Chromosome>(&mut self, population: &[C]) {
        let uncovered_branches = self.compute_uncovered_branches(population);
        self.uncovered_branches = uncovered_branches;
    }

    fn compute_uncovered_branches<C: Chromosome>(&self, population: &[C]) -> Vec<Branch> {
        let mut uncovered_branches = vec![];
        for branch in &self.branches {
            let mut covered = false;
            for individual in population {
                if individual.fitness(branch).is_zero() {
                    covered = true;
                    break;
                }
            }

            if !covered {
                uncovered_branches.push(branch.clone());
            }
        }

        uncovered_branches
    }
}