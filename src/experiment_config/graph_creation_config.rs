mod erdos_renyi_generation;
mod file_load;
mod grid_generation;

pub use erdos_renyi_generation::{ErdosRenyiGeneration, UnseededErdosRenyiGeneration};
pub use file_load::{FileLoad, UnseededFileLoad};
pub use grid_generation::{GridGeneration, UnseededGridGeneration};

use serde::{Deserialize, Serialize};

use super::{ExperimentConfigError, Fix};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum GraphCreationConfig {
    File(FileLoad),
    UnseededFile(UnseededFileLoad),
    Grid(GridGeneration),
    UnseededGrid(UnseededGridGeneration),
    ErdosRenyi(ErdosRenyiGeneration),
    UnseededErdosRenyi(UnseededErdosRenyiGeneration),
}

impl GraphCreationConfig {
    pub fn file(&self) -> Result<FileLoad, ExperimentConfigError> {
        match self {
            Self::File(file) => Ok(file.clone()),
            Self::UnseededFile(file) => Ok(file.to_fixed()),
            _ => Err(ExperimentConfigError::NotFileBased),
        }
    }

    pub fn grid(&self) -> Result<GridGeneration, ExperimentConfigError> {
        match self {
            Self::Grid(grid) => Ok(*grid),
            Self::UnseededGrid(grid) => Ok(grid.to_fixed()),
            _ => Err(ExperimentConfigError::NotFileBased),
        }
    }

    pub fn erdos_renyi(&self) -> Result<ErdosRenyiGeneration, ExperimentConfigError> {
        match self {
            Self::ErdosRenyi(erdos_renyi) => Ok(*erdos_renyi),
            Self::UnseededErdosRenyi(erdos_renyi) => Ok(erdos_renyi.to_fixed()),
            _ => Err(ExperimentConfigError::NotErdosRenyi),
        }
    }
}
