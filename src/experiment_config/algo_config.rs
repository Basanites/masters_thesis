mod aco_experiment;
mod mm_aco_experiment;
mod random_search_experiment;
mod two_swap_experiment;

use serde::{Deserialize, Serialize};

use crate::experiment_config::{ExperimentConfigError, Fix};
pub use aco_experiment::{AcoExperiment, UnseededAcoExperiment};
pub use mm_aco_experiment::{MMAcoExperiment, UnseededMMAcoExperiment};
pub use random_search_experiment::{RandomSearchExperiment, UnseededRandomSearchExperiment};
pub use two_swap_experiment::TwoSwapExperiment;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum AlgoConfig {
    MMAco(MMAcoExperiment),
    UnseededMMAco(UnseededMMAcoExperiment),
    Aco(AcoExperiment),
    UnseededAco(UnseededAcoExperiment),
    Random(RandomSearchExperiment),
    UnseededRandom(UnseededRandomSearchExperiment),
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

    pub fn mm_aco(&self) -> Result<MMAcoExperiment, ExperimentConfigError> {
        match self {
            AlgoConfig::MMAco(mmaco) => Ok(*mmaco),
            AlgoConfig::UnseededMMAco(usmmaco) => Ok(usmmaco.to_fixed()),
            _ => Err(ExperimentConfigError::NotMMAco),
        }
    }

    pub fn two_swap(&self) -> Result<TwoSwapExperiment, ExperimentConfigError> {
        match self {
            AlgoConfig::TwoSwap(two) => Ok(*two),
            _ => Err(ExperimentConfigError::NotTwoSwap),
        }
    }

    pub fn random(&self) -> Result<RandomSearchExperiment, ExperimentConfigError> {
        match self {
            AlgoConfig::Random(random) => Ok(*random),
            AlgoConfig::UnseededRandom(urandom) => Ok(urandom.to_fixed()),
            _ => Err(ExperimentConfigError::NotRandom),
        }
    }
}
