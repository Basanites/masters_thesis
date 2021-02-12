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
use std::cmp::{Eq, PartialEq};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::time::Instant;

pub struct ACO<'a, IndexType: Clone, Nw, Ew> {
    graph: &'a dyn GenericWeightedGraph<IndexType, Nw, Ew>,
    pheromone_matrix: MatrixGraph<IndexType, (), f64>,
    goal_point: IndexType,
    max_time: Ew,
    heuristic: Heuristic<IndexType, Nw, Ew>,
    seed: u128,
    alpha: f64,
    beta: f64,
    rho: f64,
    q: f64,
    ant_count: usize,
    best_solution: Solution<IndexType>,
    best_score: Nw,
    best_length: Ew,
    supervisor: Supervisor,
    rng: Rand64,
}

impl<'a, IndexType, Nw> ACO<'a, IndexType, Nw, f64>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy + Zero + PartialOrd,
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

impl<'a, IndexType, Nw>
    Metaheuristic<'a, Params<IndexType, Nw, f64>, IndexType, Nw, f64, Supervisor>
    for ACO<'a, IndexType, Nw, f64>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy + Zero + PartialOrd,
{
    fn new(
        problem: ProblemInstance<'a, IndexType, Nw, f64>,
        params: Params<IndexType, Nw, f64>,
        supervisor: Supervisor,
    ) -> Self {
        let graph = problem.graph;
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

        ACO {
            graph,
            pheromone_matrix: pheromones,
            goal_point: problem.goal_point,
            max_time: problem.max_time,
            heuristic: params.heuristic,
            alpha: params.alpha,
            beta: params.beta,
            rho: params.rho,
            q: 1.0,
            seed: params.seed,
            ant_count: params.ant_count,
            best_solution: Solution::new(),
            best_score: Nw::zero(),
            best_length: f64::zero(),
            supervisor,
            rng: rng64(params.seed),
        }
    }

    fn single_iteration(&mut self) -> Option<&Solution<IndexType>> {
        let start_time = Instant::now();
        let ants = vec![
            {
                let (sender, id) = self.supervisor.new_ant();
                Ant::new(
                    self.graph,
                    &self.pheromone_matrix,
                    self.goal_point,
                    self.max_time,
                    &self.heuristic,
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

        let mut best_length = f64::zero();
        let mut best_score = Nw::zero();
        let mut best_solution = Solution::new();
        for solution in solutions.into_iter() {
            if let Ok((score, length)) = solution_score_and_length(&solution, self.graph) {
                if length <= self.max_time && score > best_score {
                    best_score = score;
                    best_length = length;
                    best_solution = solution;
                }
            }
        }

        let duration = start_time.elapsed();
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
