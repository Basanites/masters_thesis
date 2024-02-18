use serde::{Deserialize, Serialize};

use crate::experiment_config::Fix;
use crate::rng::os_random_seed;

#[derive(Copy, Clone, Deserialize, Serialize, Debug)]
pub struct AcsExperiment {
	pub alpha: f64,
	pub beta: f64,
	pub rho: f64,
	pub q_0: f64,
	pub t_0: f64,
	pub seed: u64,
	pub ant_count: usize,
	pub iterations: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UnseededAcsExperiment {
	pub alpha: f64,
	pub beta: f64,
	pub rho: f64,
	pub q_0: f64,
	pub t_0: f64,
	pub ant_count: usize,
	pub iterations: usize,
}

impl Fix<AcsExperiment> for UnseededAcsExperiment {
	fn to_fixed(&self) -> AcsExperiment {
		AcsExperiment {
			alpha: self.alpha,
			beta: self.beta,
			rho: self.rho,
			q_0: self.q_0,
			t_0: self.t_0,
			ant_count: self.ant_count,
			seed: (os_random_seed() >> 64) as u64,
			iterations: self.iterations,
		}
	}
}
