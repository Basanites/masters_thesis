use crate::metaheuristic::{Heuristic, Solution};
use crate::rng::os_random_seed;

use std::collections::HashMap;

pub struct Params<'a, IndexType, Nw, Ew> {
    pub heuristic: &'a Heuristic<Nw, Ew>,
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub seed: u128,
    pub ant_count: usize,
    pub p_best: f64,
    pub inv_shortest_paths: HashMap<IndexType, Option<(Solution<IndexType>, Ew)>>,
}

impl<'a, IndexType, Nw, Ew> Params<'a, IndexType, Nw, Ew> {
    pub fn new(
        heuristic: &'a Heuristic<Nw, Ew>,
        alpha: f64,
        beta: f64,
        rho: f64,
        seed: Option<u128>,
        ant_count: usize,
        p_best: f64,
        inv_shortest_paths: HashMap<IndexType, Option<(Solution<IndexType>, Ew)>>,
    ) -> Self {
        Params {
            heuristic,
            alpha,
            beta,
            rho,
            seed: seed.unwrap_or_else(os_random_seed),
            ant_count,
            p_best,
            inv_shortest_paths,
        }
    }
}
