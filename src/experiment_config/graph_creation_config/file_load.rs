use serde::{Deserialize, Serialize};

use crate::experiment_config::Fix;
use crate::rng::os_random_seed;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct FileLoad {
    pub filename: String,
    pub seed: u64,
    pub nw_range: (f64, f64),
    pub node_weight_probability: f64,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct UnseededFileLoad {
    pub filename: String,
    pub nw_range: (f64, f64),
    pub node_weight_probability: f64,
}

impl Fix<FileLoad> for UnseededFileLoad {
    fn to_fixed(&self) -> FileLoad {
        FileLoad {
            filename: self.filename.clone(),
            seed: (os_random_seed() >> 64) as u64,
            nw_range: self.nw_range,
            node_weight_probability: self.node_weight_probability,
        }
    }
}
