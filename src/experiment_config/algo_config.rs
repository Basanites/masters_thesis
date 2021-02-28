mod aco_experiment;
mod two_swap_experiment;

use serde::{Deserialize, Serialize};

use crate::experiment_config::{ExperimentConfigError, Fix};
pub use aco_experiment::{AcoExperiment, UnseededAcoExperiment};
pub use two_swap_experiment::TwoSwapExperiment;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum AlgoConfig {
    Aco(AcoExperiment),
    UnseededAco(UnseededAcoExperiment),
    TwoSwap(TwoSwapExperiment),
}

impl AlgoConfig {
    pub fn aco(&self) -> Result<AcoExperiment, ExperimentConfigError> {
        match self {
            AlgoConfig::Aco(aco) => Ok(*aco),
            AlgoConfig::UnseededAco(usaco) => Ok(usaco.to_fixed()),
            _ => Err(ExperimentConfigError::NotAco),
        }
    }

    pub fn two_swap(&self) -> Result<TwoSwapExperiment, ExperimentConfigError> {
        match self {
            AlgoConfig::TwoSwap(two) => Ok(*two),
            _ => Err(ExperimentConfigError::NotTwoSwap),
        }
    }
}
