pub mod aco;
mod solution;
pub mod supervisor;
pub mod two_swap;

pub use aco::Aco;
pub use solution::{
    solution_length, solution_score, solution_score_and_length, Solution, SolutionError,
};
pub use two_swap::TwoSwap;

use decorum::R64;
use std::cell::RefCell;

use crate::graph::GenericWeightedGraph;

pub type Heuristic<IndexType, Nw, Ew> = dyn Fn(Nw, Ew, IndexType, Ew) -> R64;

pub trait Metaheuristic<'a, IndexType, NodeWeightType, EdgeWeightType> {
    type Params;
    type SupervisorType;

    fn new(
        problem: ProblemInstance<'a, IndexType, NodeWeightType, EdgeWeightType>,
        params: Self::Params,
        supervisor: Self::SupervisorType,
    ) -> Self;
    fn single_iteration(&mut self) -> Option<&Solution<IndexType>>;
}

pub struct ProblemInstance<'a, IndexType, NodeWeightType, EdgeWeightType> {
    graph: &'a RefCell<
        dyn GenericWeightedGraph<
            IndexType = IndexType,
            NodeWeightType = NodeWeightType,
            EdgeWeightType = EdgeWeightType,
        >,
    >,
    goal_point: IndexType,
    max_time: EdgeWeightType,
}

impl<'a, IndexType, NodeWeightType, EdgeWeightType>
    ProblemInstance<'a, IndexType, NodeWeightType, EdgeWeightType>
{
    pub fn new(
        graph: &'a RefCell<
            dyn GenericWeightedGraph<
                IndexType = IndexType,
                NodeWeightType = NodeWeightType,
                EdgeWeightType = EdgeWeightType,
            >,
        >,
        goal_point: IndexType,
        max_time: EdgeWeightType,
    ) -> Self {
        ProblemInstance {
            graph,
            goal_point,
            max_time,
        }
    }
}
