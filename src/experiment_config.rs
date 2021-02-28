mod algo_config;
mod general_experiment_config;
mod graph_creation_config;
mod graph_dynamics_config;

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
    pub graph_dynamics: GraphDynamicsConfig,
}

// pub struct DynamicGraphExperiment<'a, IndexType, M>
// where
//     M: Metaheuristic<'a, IndexType, f64, f64>,
// {
//     graph_rng: Rand64,
//     nw_range: (f64, f64),
//     change_after_i: usize,
//     edge_change_probability: f64,
//     node_change_probability: f64,
//     edge_change_intensity: f64,
//     node_change_intensity: f64,
//     algo: M,
//     graph: Box<dyn GenericWeightedGraph<IndexType = IndexType, NodeWeightType = f64, EdgeWeightType = f64>>,
//     phantom: PhantomData<&'a IndexType>,
// }

// impl<'a, IndexType, M> DynamicGraphExperiment<'a, IndexType, M>
// where
//     M: Metaheuristic<'a, IndexType, f64, f64>,
// {
//     pub fn from_config(config: ExperimentConfig, heuristic: &'a Heuristic<IndexType, f64, f64>) -> Result<Self, ExperimentConfigError> {
//         let rng;
//         let graph: Box<dyn GenericWeightedGraph<IndexType = _, NodeWeightType = _, EdgeWeightType = f64>> = if let GraphConfig::File(f) = config.graph {
//             rng = rng64(f.seed);
//             let delta = f.nw_range.1 - f.nw_range.0;
//             let value_gen = || rng.rand_float() * delta + f.nw_range.0;
//             let pbf = import_pbf(f.filename.as_str(), &value_gen);
//             Box::new(pbf)
//         } else if let GraphConfig::Grid(g) = config.graph {
//             rng = rng64(g.seed);
//             let nw_delta = g.nw_range.1 - g.nw_range.0;
//             let ew_delta = g.ew_range.1 - g.ew_range.0;
//             let nw_gen = || rng.rand_float() * nw_delta;
//             let ew_gen = || rng.rand_float() * ew_delta;
//             let grid_gen = Grid::new(g.size, &nw_gen, &ew_gen);
//             grid_gen.generate() as Box<dyn GenericWeightedGraph<IndexType = usize, NodeWeightType = f64, EdgeWeightType = f64>>
//         } else {
//             return Err(ExperimentConfigError::InvalidGraphConfig("Graph config is not valid for any of the allowed types."));
//         }

// u
//         // let algo = if let Ok(aco) = config.algo.aco() {
//         //     Aco::new()
//         // }

//     }

//     fn next_nw(&self, weight: f64) -> f64 {
//         self.graph_rng.rand_float() * self.node_change_intensity * weight
//     }

//     fn next_ew(&self, weight: f64) -> f64 {
//         self.graph_rng.rand_float() * self.edge_change_intensity * weight
//     }
// }

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

impl fmt::Display for ExperimentConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAco => write!(f, "Config is not a valid ACO config."),
            Self::NotTwoSwap => write!(f, "Config is not a valid TwoSwap config."),
            Self::InvalidAlgorithmConfig(msg) => write!(f, "{}", msg),
            Self::NotFileBased => write!(f, "Config is not a valid file import config."),
            Self::NotGrid => write!(f, "Config is not a valid generation config."),
            Self::NotErdosRenyi => write!(f, "Config is not a valid ErdosRenyi generation config."),
            Self::InvalidGraphConfig(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ExperimentConfigError {}
