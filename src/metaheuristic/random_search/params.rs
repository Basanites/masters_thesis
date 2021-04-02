use crate::metaheuristic::{Heuristic, Solution};

use std::collections::HashMap;

pub struct Params<'a, IndexType, Nw, Ew> {
    pub heuristic: &'a Heuristic<IndexType, Nw, Ew>,
    pub inv_shortest_paths: &'a HashMap<IndexType, Option<(Solution<IndexType>, Ew)>>,
    pub seed: u128,
}

impl<'a, IndexType, Nw, Ew> Params<'a, IndexType, Nw, Ew> {
    pub fn new(
        heuristic: &'a Heuristic<IndexType, Nw, Ew>,
        inv_shortest_paths: &'a HashMap<IndexType, Option<(Solution<IndexType>, Ew)>>,
        seed: u128,
    ) -> Self {
        Params {
            heuristic,
            inv_shortest_paths,
            seed,
        }
    }
}
