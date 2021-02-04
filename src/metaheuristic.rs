pub mod aco;
mod solution;
pub mod two_swap;

use crate::graph::GenericWeightedGraph;
pub use aco::ACO;
pub use solution::{solution_length, Solution, SolutionError};
pub use two_swap::TwoSwap;

pub type Heuristic<IndexType, Nw, Ew> = fn(
    to_node_weight: Nw,
    edge_to_node_weight: Ew,
    to_node_id: IndexType,
    elapsed_time_ratio_until: Ew,
) -> f64;

pub trait Metaheuristic<Params, IndexType, Nw, Ew> {
    fn new(problem: ProblemInstance<IndexType, Nw, Ew>, params: Params) -> Self;
    fn single_iteration(&mut self) -> Option<&Solution<IndexType>>;
}

pub struct ProblemInstance<IndexType, Nw, Ew> {
    graph: Box<dyn GenericWeightedGraph<IndexType, Nw, Ew>>,
    goal_point: IndexType,
    max_time: Ew,
}

impl<IndexType, Nw, Ew> ProblemInstance<IndexType, Nw, Ew> {
    pub fn new(
        graph: Box<dyn GenericWeightedGraph<IndexType, Nw, Ew>>,
        goal_point: IndexType,
        max_time: Ew,
    ) -> Self {
        ProblemInstance {
            graph,
            goal_point,
            max_time,
        }
    }
}
