use serde::{Deserialize, Serialize};

use crate::experiment_config::Fix;
use crate::rng::os_random_seed;

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct AcoExperiment {
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub seed: u64,
    pub ant_count: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnseededAcoExperiment {
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub ant_count: usize,
}

impl Fix<AcoExperiment> for UnseededAcoExperiment {
    fn to_fixed(&self) -> AcoExperiment {
        AcoExperiment {
            alpha: self.alpha,
            beta: self.beta,
            rho: self.rho,
            ant_count: self.ant_count,
            seed: (os_random_seed() >> 64) as u64,
        }
    }
}
