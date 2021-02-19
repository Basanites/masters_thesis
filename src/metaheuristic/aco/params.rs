use crate::metaheuristic::Heuristic;
use crate::rng::os_random_seed;

pub struct Params<'a, IndexType, Nw, Ew> {
    pub heuristic: &'a Heuristic<IndexType, Nw, Ew>,
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub seed: u128,
    pub ant_count: usize,
}

impl<'a, IndexType, Nw, Ew> Params<'a, IndexType, Nw, Ew> {
    pub fn new(
        heuristic: &'a Heuristic<IndexType, Nw, Ew>,
        alpha: f64,
        beta: f64,
        rho: f64,
        seed: Option<u128>,
        ant_count: usize,
    ) -> Self {
        Params {
            heuristic,
            alpha,
            beta,
            rho,
            seed: seed.unwrap_or_else(os_random_seed),
            ant_count,
        }
    }
}
