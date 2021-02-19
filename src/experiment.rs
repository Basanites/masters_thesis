use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::graph::import::import_pbf;
use crate::graph::MatrixGraph;
use crate::rng::os_random_seed;

pub trait Algorithm {
    fn finished(&self) -> bool;
}

macro_rules! experiment {
    ($type:ty) => {
        impl Algorithm for $type {
            fn finished(&self) -> bool {
                self.finished
            }
        }
    };
}

pub trait FixAlgo<CorrectType: Algorithm> {
    fn as_fixed(&self) -> CorrectType;
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct TwoSwapExperiment {
    pub finished: bool,
    pub aggregation_rate: usize,
}

experiment! {TwoSwapExperiment}

#[derive(Deserialize, Serialize, Debug)]
pub struct NoStatTwoSwapExperiment {
    pub aggregation_rate: usize,
}

impl FixAlgo<TwoSwapExperiment> for NoStatTwoSwapExperiment {
    fn as_fixed(&self) -> TwoSwapExperiment {
        TwoSwapExperiment {
            finished: false,
            aggregation_rate: self.aggregation_rate,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NoStatAcoExperiment {
    pub aggregation_rate: usize,
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub ant_count: usize,
}

impl FixAlgo<AcoExperiment> for NoStatAcoExperiment {
    fn as_fixed(&self) -> AcoExperiment {
        AcoExperiment {
            finished: false,
            aggregation_rate: self.aggregation_rate,
            alpha: self.alpha,
            beta: self.beta,
            rho: self.rho,
            ant_count: self.ant_count,
            seed: os_random_seed(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SeededAcoExperiment {
    pub aggregation_rate: usize,
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub seed: u128,
    pub ant_count: usize,
}

impl FixAlgo<AcoExperiment> for SeededAcoExperiment {
    fn as_fixed(&self) -> AcoExperiment {
        AcoExperiment {
            finished: false,
            aggregation_rate: self.aggregation_rate,
            alpha: self.alpha,
            beta: self.beta,
            rho: self.rho,
            ant_count: self.ant_count,
            seed: self.seed,
        }
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct AcoExperiment {
    pub finished: bool,
    pub aggregation_rate: usize,
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub seed: u128,
    pub ant_count: usize,
}

experiment! {AcoExperiment}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct GridGeneration {
    pub seed: usize,
    pub size: usize,
    pub nw_range: (f64, f64),
    pub ew_range: (f64, f64),
    pub node_weight_probability: f64,
    pub change_after_i: usize,
    pub edge_change_probability: f64,
    pub node_change_probability: f64,
    pub edge_change_intensity: f64,
    pub node_change_intensity: f64,
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct ErdosRenyiGeneration {
    pub seed: usize,
    pub size: usize,
    pub nw_range: (f64, f64),
    pub ew_range: (f64, f64),
    pub node_weight_probability: f64,
    pub connection_probability: f64,
    pub change_after_i: usize,
    pub edge_change_probability: f64,
    pub node_change_probability: f64,
    pub edge_change_intensity: f64,
    pub node_change_intensity: f64,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct GraphLoad {
    pub filename: String,
    pub seed: usize,
    pub change_after_i: usize,
    pub edge_change_probability: f64,
    pub node_change_probability: f64,
    pub edge_change_intensity: f64,
    pub node_change_intensity: f64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum GraphConfig {
    File(GraphLoad),
    Grid(GridGeneration),
    ErdosRenyi(ErdosRenyiGeneration),
}

impl GraphConfig {
    pub fn file(&self) -> Result<GraphLoad, ExperimentConfigError> {
        match self {
            GraphConfig::File(file) => Ok(file.clone()),
            _ => Err(ExperimentConfigError::NotFileBased),
        }
    }

    pub fn grid(&self) -> Result<GridGeneration, ExperimentConfigError> {
        match self {
            GraphConfig::Grid(grid) => Ok(*grid),
            _ => Err(ExperimentConfigError::NotFileBased),
        }
    }

    pub fn erdos_renyi(&self) -> Result<ErdosRenyiGeneration, ExperimentConfigError> {
        match self {
            GraphConfig::ErdosRenyi(erdos_renyi) => Ok(*erdos_renyi),
            _ => Err(ExperimentConfigError::NotErdosRenyi),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum AlgoConfig {
    Aco(AcoExperiment),
    NoStatAco(NoStatAcoExperiment),
    SeededAco(SeededAcoExperiment),
    TwoSwap(TwoSwapExperiment),
    NoStatTwoSwap(NoStatTwoSwapExperiment),
}

impl AlgoConfig {
    pub fn aco(&self) -> Result<AcoExperiment, ExperimentConfigError> {
        match self {
            AlgoConfig::Aco(aco) => Ok(*aco),
            AlgoConfig::NoStatAco(nsaco) => Ok(nsaco.as_fixed()),
            AlgoConfig::SeededAco(saco) => Ok(saco.as_fixed()),
            _ => Err(ExperimentConfigError::NotAco),
        }
    }

    pub fn two_swap(&self) -> Result<TwoSwapExperiment, ExperimentConfigError> {
        match self {
            AlgoConfig::TwoSwap(two) => Ok(*two),
            AlgoConfig::NoStatTwoSwap(nstwo) => Ok(nstwo.as_fixed()),
            _ => Err(ExperimentConfigError::NotTwoSwap),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Experiment {
    algo: AlgoConfig,
    graph: GraphConfig,
}

// impl Experiment {
//     pub fn get_graph<IndexType: Clone, Nw, Ew>(
//         &self,
//     ) -> Result<MatrixGraph<IndexType, Nw, Ew>, ExperimentConfigError> {
//         match self.graph {
//             GraphConfig::File(file) => {
//                 if !Path::new(file.filename.as_str()).exists() {
//                     return Err(ExperimentConfigError::InvalidGraphConfig(format!(
//                         "File {} does not exist.",
//                         file.filename
//                     )));
//                 }

//                 Ok(import_pbf(file.filename.as_str()))
//             }
//         }
//     }
// }
#[derive(Debug)]
pub enum ExperimentConfigError {
    NotAco,
    NotTwoSwap,
    InvalidAlgorithmConfig(String),
    NotFileBased,
    NotGrid,
    NotErdosRenyi,
    InvalidGraphConfig(String),
}
