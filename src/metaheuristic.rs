pub mod aco;
mod solution;
pub mod supervisor;
pub mod two_swap;

pub use aco::Aco;
pub use solution::{
    solution_length, solution_score, solution_score_and_length, Solution, SolutionError,
};
pub use two_swap::TwoSwap;

use crate::graph::GenericWeightedGraph;

pub type Heuristic<IndexType, Nw, Ew> = dyn Fn(Nw, Ew, IndexType, Ew) -> f64;

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
