use serde::{Deserialize, Serialize};

use super::Fix;
use crate::rng::os_random_seed;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum GraphDynamicsConfig {
    Full(FullConfig),
    Unseeded(UnseededConfig),
}

impl GraphDynamicsConfig {
    pub fn cfg(&self) -> FullConfig {
        match self {
            Self::Full(cfg) => *cfg,
            Self::Unseeded(cfg) => cfg.to_fixed(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct FullConfig {
    pub seed: u64,
    pub change_after_i: u64,
    pub edge_change_probability: f64,
    pub node_change_probability: f64,
    pub edge_change_intensity: f64,
    pub node_change_intensity: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnseededConfig {
    pub change_after_i: u64,
    pub edge_change_probability: f64,
    pub node_change_probability: f64,
    pub edge_change_intensity: f64,
    pub node_change_intensity: f64,
}

impl Fix<FullConfig> for UnseededConfig {
    fn to_fixed(&self) -> FullConfig {
        FullConfig {
            seed: (os_random_seed() >> 64) as u64,
            change_after_i: self.change_after_i,
            edge_change_probability: self.edge_change_probability,
            node_change_probability: self.node_change_probability,
            edge_change_intensity: self.edge_change_intensity,
            node_change_intensity: self.node_change_intensity,
        }
    }
}
