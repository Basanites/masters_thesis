pub mod aco;
pub mod random_search;
mod solution;
pub mod supervisor;
pub mod two_swap;

pub use aco::Aco;
pub use random_search::RandomSearch;
pub use solution::{solution_length, solution_score, Solution, SolutionError};
pub use two_swap::TwoSwap;

use decorum::R64;
use std::cell::RefCell;

use crate::graph::GenericWeightedGraph;

pub type Heuristic<Nw, Ew> = dyn Fn(Nw, Ew, R64, Ew) -> R64;

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
