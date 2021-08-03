pub mod algo_config;
pub mod general_experiment_config;
pub mod graph_creation_config;
pub mod graph_dynamics_config;

pub use algo_config::{AcoExperiment, AlgoConfig, TwoSwapExperiment};
pub use general_experiment_config::GeneralExperimentConfig;
pub use graph_creation_config::GraphCreationConfig;
pub use graph_dynamics_config::GraphDynamicsConfig;

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub trait Algorithm {
    fn finished(&self) -> bool;
}

#[macro_export]
macro_rules! experiment {
    ($type:ty) => {
        impl Algorithm for $type {
            fn finished(&self) -> bool {
                self.finished
            }
        }
    };
}

pub trait Fix<CorrectType> {
    fn to_fixed(&self) -> CorrectType;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ExperimentConfig {
    pub experiment: GeneralExperimentConfig,
    pub algorithm: AlgoConfig,
    pub graph_creation: GraphCreationConfig,
    // pub graph_dynamics: GraphDynamicsConfig,
}

#[derive(Debug)]
pub enum ExperimentConfigError {
    NotAco,
    NotMMAco,
    NotAcs,
    NotTwoSwap,
    NotRandom,
    InvalidAlgorithmConfig(String),
    NotFileBased,
    NotGrid,
    NotErdosRenyi,
    InvalidGraphConfig(String),
}

impl fmt::Display for ExperimentConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAco => write!(f, "Config is not a valid ACO config."),
            Self::NotMMAco => write!(f, "Config is not a valid MMAco config."),
            Self::NotAcs => write!(f, "Config is not a valid ACS config."),
            Self::NotTwoSwap => write!(f, "Config is not a valid TwoSwap config."),
            Self::NotRandom => write!(f, "Config is not a valid RandomSearch config."),
            Self::InvalidAlgorithmConfig(msg) => write!(f, "{}", msg),
            Self::NotFileBased => write!(f, "Config is not a valid file import config."),
            Self::NotGrid => write!(f, "Config is not a valid generation config."),
            Self::NotErdosRenyi => write!(f, "Config is not a valid ErdosRenyi generation config."),
            Self::InvalidGraphConfig(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ExperimentConfigError {}
