use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Branch {
    Root(u64),
    Decision(u64)
}