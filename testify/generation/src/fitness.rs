use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum FitnessValue {
    Zero,
    Val(f64),
    Max,
}
impl FitnessValue {
    pub fn is_zero(&self) -> bool {
        self == &FitnessValue::Zero
    }

    pub fn is_max(&self) -> bool {
        self == &FitnessValue::Max
    }

    pub fn value(&self) -> Option<f64> {
        if let FitnessValue::Val(val) = self {
            Some(*val)
        } else {
            None
        }
    }
}

impl PartialEq for FitnessValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            FitnessValue::Zero => {
                if let Self::Zero = other {
                    true
                } else {
                    false
                }
            }
            FitnessValue::Val(a) => {
                if let Self::Val(b) = other {
                    a == b
                } else {
                    false
                }
            }
            FitnessValue::Max => {
                if let Self::Max = other {
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Eq for FitnessValue {}

impl PartialOrd for FitnessValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            FitnessValue::Zero => match other {
                FitnessValue::Zero => Some(Ordering::Equal),
                FitnessValue::Val(_) => Some(Ordering::Less),
                FitnessValue::Max => Some(Ordering::Less),
            },
            FitnessValue::Val(a) => match other {
                FitnessValue::Zero => Some(Ordering::Greater),
                FitnessValue::Val(b) => a.partial_cmp(b),
                FitnessValue::Max => Some(Ordering::Less),
            },
            FitnessValue::Max => match other {
                FitnessValue::Zero => Some(Ordering::Greater),
                FitnessValue::Val(_) => Some(Ordering::Greater),
                FitnessValue::Max => Some(Ordering::Equal),
            },
        }
    }
}

impl Display for FitnessValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FitnessValue::Zero => write!(f, "Zero"),
            FitnessValue::Val(val) => write!(f, "Val({})", val),
            FitnessValue::Max => write!(f, "Max"),
        }
    }
}
