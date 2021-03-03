mod ant;
mod message;
mod params;
mod supervisor;

pub use ant::Ant;
pub use message::Message;
pub use params::Params;
pub use supervisor::Supervisor;

use crate::graph::{Edge, GenericWeightedGraph, MatrixGraph};
use crate::metaheuristic::{
    solution_score_and_length, Heuristic, Metaheuristic, ProblemInstance, Solution,
};
use crate::rng::rng64;

use num_traits::identities::Zero;
use oorandom::Rand64;
use serde::Serialize;
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::Write;
use std::ops::Add;
use std::time::Instant;

pub struct Aco<'a, IndexType, Nw, Ew, W>
where
    IndexType: Clone,
    W: Write,
    Ew: Serialize + Add<Output = Ew>,
{
    graph: &'a RefCell<
        dyn GenericWeightedGraph<IndexType = IndexType, NodeWeightType = Nw, EdgeWeightType = Ew>,
    >,
    pheromone_matrix: MatrixGraph<IndexType, (), f64>,
    goal_point: IndexType,
    max_time: Ew,
    heuristic: &'a Heuristic<IndexType, Nw, Ew>,
    alpha: f64,
    beta: f64,
    rho: f64,
    q: f64,
    ant_count: usize,
    best_solution: Solution<IndexType>,
    best_score: Nw,
    best_length: Ew,
    pub supervisor: Supervisor<W, Ew>,
    rng: Rand64,
}

impl<'a, IndexType, Nw, W> Aco<'a, IndexType, Nw, f64, W>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy + Zero + PartialOrd,
    W: Write,
{
    fn pheromone_update(&mut self, solution: &Solution<IndexType>, solution_length: f64) {
        let to_add = self.q / solution_length;

        for edge in self.pheromone_matrix.edge_ids() {
            let weight = *self.pheromone_matrix.edge_weight(edge).unwrap();
            let _res = self
                .pheromone_matrix
                .change_edge(edge, (1.0 - self.rho) * weight);
        }

        for (from, to) in solution.iter_edges() {
            let weight = *self.pheromone_matrix.edge_weight((*from, *to)).unwrap();
            let _res = self
                .pheromone_matrix
                .change_edge((*from, *to), weight + to_add);
        }
    }
}

impl<'a, IndexType, W> Metaheuristic<'a, IndexType, f64, f64> for Aco<'a, IndexType, f64, f64, W>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    W: Write,
{
    type Params = Params<'a, IndexType, f64, f64>;
    type SupervisorType = Supervisor<W, f64>;

    fn new(
        problem: ProblemInstance<'a, IndexType, f64, f64>,
        params: Self::Params,
        supervisor: Self::SupervisorType,
    ) -> Self {
        let graph = problem.graph.borrow();
        let pheromones = MatrixGraph::new(
            graph.iter_node_ids().map(|id| (id, ())).collect(),
            graph
                .iter_node_ids()
                .map(|x| -> Vec<(Edge<IndexType>, f64)> {
                    graph.iter_node_ids().map(|y| ((x, y), 1.0)).collect()
                })
                .flatten()
                .collect(),
        )
        .unwrap();

        Aco {
            graph: problem.graph,
            pheromone_matrix: pheromones,
            goal_point: problem.goal_point,
            max_time: problem.max_time,
            heuristic: params.heuristic,
            alpha: params.alpha,
            beta: params.beta,
            rho: params.rho,
            q: 1.0,
            ant_count: params.ant_count,
            best_solution: Solution::new(),
            best_score: f64::zero(),
            best_length: f64::zero(),
            supervisor,
            rng: rng64(params.seed),
        }
    }

    fn single_iteration(&mut self) -> Option<&Solution<IndexType>> {
        let ants = vec![
            {
                let (sender, id) = self.supervisor.new_ant();
                Ant::new(
                    self.graph,
                    &self.pheromone_matrix,
                    self.goal_point,
                    self.max_time,
                    self.heuristic,
                    self.rng.rand_u64() as u128 + ((self.rng.rand_u64() as u128) << 64),
                    self.alpha,
                    self.beta,
                    sender,
                    id,
                )
            };
            self.ant_count
        ];

        let mut solutions = Vec::new();
        for ant in ants {
            solutions.push(ant.get_solution())
        }

        let start_time = Instant::now();
        let mut best_length = f64::zero();
        let mut best_score = f64::zero();
        let mut best_solution = Solution::new();
        let mut improvements = 0;
        for solution in solutions.into_iter() {
            if let Ok((score, length)) = solution_score_and_length(&solution, self.graph) {
                if length <= self.max_time && score > best_score {
                    improvements += 1;
                    best_score = score;
                    best_length = length;
                    best_solution = solution;
                }
            }
        }

        let duration = start_time.elapsed();
        let _ = self.supervisor.sender.send(Message::new(
            0,
            0,
            0,
            improvements,
            improvements,
            0,
            duration,
            best_length,
            best_score,
        )); // Ant 0 is always supervisor
        self.supervisor.aggregate_receive();
        self.supervisor.reset();
        if best_score > self.best_score {
            self.pheromone_update(&best_solution, best_length);
            self.best_solution = best_solution;
            self.best_score = best_score;
            self.best_length = best_length;

            return Some(&self.best_solution);
        }

        None
    }
}
