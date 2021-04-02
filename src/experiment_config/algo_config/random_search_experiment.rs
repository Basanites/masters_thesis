use serde::{Deserialize, Serialize};

use crate::experiment_config::Fix;
use crate::rng::os_random_seed;

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct RandomSearchExperiment {
    pub seed: u64,
    pub iterations: usize,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct UnseededRandomSearchExperiment {
    pub iterations: usize,
}

impl Fix<RandomSearchExperiment> for UnseededRandomSearchExperiment {
    fn to_fixed(&self) -> RandomSearchExperiment {
        RandomSearchExperiment {
            seed: (os_random_seed() >> 64) as u64,
            iterations: self.iterations,
        }
    }
}
