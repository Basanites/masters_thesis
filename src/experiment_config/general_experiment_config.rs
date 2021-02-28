use serde::{Deserialize, Serialize};

use crate::experiment;
use crate::experiment_config::{Algorithm, Fix};
use crate::rng::os_random_seed;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum GeneralExperimentConfig {
    Full(FullConfig),
    NoStat(NoStatConfig),
    Unseeded(UnseededConfig),
    AggregationOnly(AggregationOnly),
}

impl GeneralExperimentConfig {
    pub fn cfg(&self) -> FullConfig {
        match self {
            Self::Full(cfg) => *cfg,
            Self::NoStat(cfg) => cfg.to_fixed(),
            Self::Unseeded(cfg) => cfg.to_fixed(),
            Self::AggregationOnly(cfg) => cfg.to_fixed(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct FullConfig {
    pub finished: bool,
    pub seed: u128,
    pub aggregation_rate: usize,
    pub max_time: f64,
}

experiment! {FullConfig}

#[derive(Deserialize, Serialize, Debug)]
pub struct NoStatConfig {
    pub seed: u128,
    pub aggregation_rate: usize,
    pub max_time: f64,
}

impl Fix<FullConfig> for NoStatConfig {
    fn to_fixed(&self) -> FullConfig {
        FullConfig {
            finished: false,
            seed: self.seed,
            aggregation_rate: self.aggregation_rate,
            max_time: self.max_time,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnseededConfig {
    pub finished: bool,
    pub aggregation_rate: usize,
    pub max_time: f64,
}

impl Fix<FullConfig> for UnseededConfig {
    fn to_fixed(&self) -> FullConfig {
        FullConfig {
            finished: self.finished,
            seed: os_random_seed(),
            aggregation_rate: self.aggregation_rate,
            max_time: self.max_time,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AggregationOnly {
    pub aggregation_rate: usize,
    pub max_time: f64,
}

impl Fix<FullConfig> for AggregationOnly {
    fn to_fixed(&self) -> FullConfig {
        FullConfig {
            finished: false,
            seed: os_random_seed(),
            aggregation_rate: self.aggregation_rate,
            max_time: self.max_time,
        }
    }
}
