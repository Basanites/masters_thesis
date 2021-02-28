use serde::{Deserialize, Serialize};

use crate::experiment_config::Fix;
use crate::rng::os_random_seed;

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct FileLoad {
    pub filename: String,
    pub seed: u128,
    pub nw_range: (f64, f64),
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct UnseededFileLoad {
    pub filename: String,
    pub nw_range: (f64, f64),
}

impl Fix<FileLoad> for UnseededFileLoad {
    fn to_fixed(&self) -> FileLoad {
        FileLoad {
            filename: self.filename.clone(),
            seed: os_random_seed(),
            nw_range: self.nw_range,
        }
    }
}
