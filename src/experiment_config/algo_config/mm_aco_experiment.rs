use serde::{Deserialize, Serialize};

use crate::experiment_config::Fix;
use crate::rng::os_random_seed;

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct MMAcoExperiment {
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub q_0: f64,
    pub seed: u64,
    pub ant_count: usize,
    pub p_best: f64,
    pub iterations: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnseededMMAcoExperiment {
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub q_0: f64,
    pub ant_count: usize,
    pub p_best: f64,
    pub iterations: usize,
}

impl Fix<MMAcoExperiment> for UnseededMMAcoExperiment {
    fn to_fixed(&self) -> MMAcoExperiment {
        MMAcoExperiment {
            alpha: self.alpha,
            beta: self.beta,
            rho: self.rho,
            q_0: self.q_0,
            seed: (os_random_seed() >> 64) as u64,
            ant_count: self.ant_count,
            p_best: self.p_best,
            iterations: self.iterations,
        }
    }
}
