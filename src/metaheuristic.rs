pub mod aco;
mod solution;
mod supervisor;
pub mod two_swap;

use crate::graph::GenericWeightedGraph;
pub use aco::ACO;
pub use solution::{
    solution_length, solution_score, solution_score_and_length, Solution, SolutionError,
};
pub use supervisor::{
    AcoMessage, AcoSupervisor, Message, MessageInfo, Supervisor, TwoSwapMessage, TwoSwapSupervisor,
};
pub use two_swap::TwoSwap;

pub type Heuristic<IndexType, Nw, Ew> = fn(
    to_node_weight: Nw,
    edge_to_node_weight: Ew,
    to_node_id: IndexType,
    elapsed_time_ratio_until: Ew,
) -> f64;

pub trait Metaheuristic<'a, Params, IndexType, Nw, Ew, SupervisorType> {
    fn new(
        problem: ProblemInstance<'a, IndexType, Nw, Ew>,
        params: Params,
        supervisor: SupervisorType,
    ) -> Self;
    fn single_iteration(&mut self) -> Option<&Solution<IndexType>>;
}

pub struct ProblemInstance<'a, IndexType, Nw, Ew> {
    graph: &'a dyn GenericWeightedGraph<IndexType, Nw, Ew>,
    goal_point: IndexType,
    max_time: Ew,
}

impl<'a, IndexType, Nw, Ew> ProblemInstance<'a, IndexType, Nw, Ew> {
    pub fn new(
        graph: &'a dyn GenericWeightedGraph<IndexType, Nw, Ew>,
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
