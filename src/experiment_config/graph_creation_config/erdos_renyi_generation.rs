use serde::{Deserialize, Serialize};

use crate::experiment_config::Fix;
use crate::rng::os_random_seed;

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct ErdosRenyiGeneration {
    pub seed: u64,
    pub size: u64,
    pub nw_range: (f64, f64),
    pub ew_range: (f64, f64),
    pub node_weight_probability: f64,
    pub connection_probability: f64,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct UnseededErdosRenyiGeneration {
    pub size: u64,
    pub nw_range: (f64, f64),
    pub ew_range: (f64, f64),
    pub node_weight_probability: f64,
    pub connection_probability: f64,
}

impl Fix<ErdosRenyiGeneration> for UnseededErdosRenyiGeneration {
    fn to_fixed(&self) -> ErdosRenyiGeneration {
        ErdosRenyiGeneration {
            seed: (os_random_seed() >> 64) as u64,
            size: self.size,
            nw_range: self.nw_range,
            ew_range: self.ew_range,
            node_weight_probability: self.node_weight_probability,
            connection_probability: self.connection_probability,
        }
    }
}
