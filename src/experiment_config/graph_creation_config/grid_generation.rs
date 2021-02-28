use serde::{Deserialize, Serialize};

use crate::experiment_config::Fix;
use crate::rng::os_random_seed;

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct GridGeneration {
    pub seed: u128,
    pub size: (usize, usize),
    pub nw_range: (f64, f64),
    pub ew_range: (f64, f64),
    pub node_weight_probability: f64,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct UnseededGridGeneration {
    pub size: (usize, usize),
    pub nw_range: (f64, f64),
    pub ew_range: (f64, f64),
    pub node_weight_probability: f64,
}

impl Fix<GridGeneration> for UnseededGridGeneration {
    fn to_fixed(&self) -> GridGeneration {
        GridGeneration {
            seed: os_random_seed(),
            size: self.size,
            nw_range: self.nw_range,
            ew_range: self.ew_range,
            node_weight_probability: self.node_weight_probability,
        }
    }
}
