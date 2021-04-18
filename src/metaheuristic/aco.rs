mod ant;
mod message;
mod params;
mod supervisor;

pub use ant::Ant;
pub use message::Message;
pub use params::Params;
pub use supervisor::Supervisor;

use crate::graph::{GenericWeightedGraph, MatrixGraph};
use crate::metaheuristic::{
    solution_length, solution_score, Heuristic, Metaheuristic, ProblemInstance, Solution,
};
use crate::rng::rng64;
use crate::util::{Distance, SmallVal};

use decorum::R64;
use num_traits::identities::{One, Zero};
use oorandom::Rand64;
use serde::Serialize;
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::Write;
use std::ops::Add;
use std::time::Instant;

pub struct Aco<'a, IndexType, Nw, Ew, W>
where
    IndexType: Clone,
    W: Write,
    Nw: Serialize + Add<Output = Nw>,
    Ew: Serialize + Add<Output = Ew>,
{
    graph: &'a RefCell<
        dyn GenericWeightedGraph<IndexType = IndexType, NodeWeightType = Nw, EdgeWeightType = Ew>,
    >,
    pheromone_matrix: MatrixGraph<IndexType, (), R64>,
    goal_point: IndexType,
    max_time: Ew,
    heuristic: &'a Heuristic<Nw, Ew>,
    alpha: f64,
    beta: f64,
    rho: f64,
    q: f64,
    ant_count: usize,
    best_solution: Solution<IndexType>,
    best_score: Nw,
    best_length: Ew,
    pub supervisor: Supervisor<W, Nw, Ew>,
    rng: Rand64,
    inv_shortest_paths: BTreeMap<IndexType, Option<(Solution<IndexType>, Ew)>>,
}

impl<'a, IndexType, Nw, W> Aco<'a, IndexType, Nw, R64, W>
where
    IndexType: Distance<IndexType> + Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    Nw: Copy + Zero + PartialOrd + Serialize + SmallVal,
    W: Write,
{
    fn pheromone_update(&mut self, solution: &Solution<IndexType>, solution_score: R64) {
        // let to_add = R64::one() - R64::from_inner(self.q) / solution_score;
        // println!("adding {}", to_add);

        let mut evaporated_pheromones = R64::zero();
        // pheromone decay
        for edge in self.pheromone_matrix.edge_ids() {
            let weight = *self.pheromone_matrix.edge_weight(edge).unwrap();
            let after_decay = R64::from_inner(1.0 - self.rho) * weight;
            evaporated_pheromones += weight - after_decay;
            let _res = self.pheromone_matrix.change_edge(edge, after_decay);
        }

        // println!(
        //     "evaporated {} and have {} nodes in new solution. This would lead to {} being added.",
        //     evaporated_pheromones,
        //     solution.unique_edges().len(),
        //     evaporated_pheromones / R64::from_inner(solution.unique_edges().len() as f64)
        // );
        let to_add = evaporated_pheromones / R64::from_inner(solution.unique_edges().len() as f64);
        // adding best solution
        for (from, to) in solution.iter_unique_edges() {
            let weight = *self.pheromone_matrix.edge_weight((*from, *to)).unwrap();
            let _res = self
                .pheromone_matrix
                .change_edge((*from, *to), weight + to_add);
        }
    }

    pub fn set_inv_shortest_paths(
        &mut self,
        inv_shortest_paths: BTreeMap<IndexType, Option<(Solution<IndexType>, R64)>>,
    ) {
        self.inv_shortest_paths = inv_shortest_paths
    }
}

impl<'a, IndexType, W> Metaheuristic<'a, IndexType, R64, R64> for Aco<'a, IndexType, R64, R64, W>
where
    IndexType: Distance<IndexType> + Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    W: Write,
{
    type Params = Params<'a, IndexType, R64, R64>;
    type SupervisorType = Supervisor<W, R64, R64>;

    fn new(
        problem: ProblemInstance<'a, IndexType, R64, R64>,
        params: Self::Params,
        supervisor: Self::SupervisorType,
    ) -> Self {
        let graph = problem.graph.borrow();
        let pheromones = MatrixGraph::new(
            graph.iter_node_ids().map(|id| (id, ())).collect(),
            graph
                .iter_edge_ids()
                .map(|edge| (edge, R64::from_inner(1.0)))
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
            best_score: R64::zero(),
            best_length: R64::zero(),
            supervisor,
            rng: rng64(params.seed),
            inv_shortest_paths: params.inv_shortest_paths,
        }
    }

    fn single_iteration(&mut self) -> Option<&Solution<IndexType>> {
        let mut ants = Vec::with_capacity(self.ant_count);
        for _ in 0..self.ant_count {
            let (sender, id) = self.supervisor.new_ant();
            let seed = self.rng.rand_u64() as u128 + ((self.rng.rand_u64() as u128) << 64);
            ants.push(Ant::new(
                self.graph,
                &self.pheromone_matrix,
                self.goal_point,
                self.max_time,
                self.heuristic,
                seed,
                self.alpha,
                self.beta,
                sender,
                id,
                &self.inv_shortest_paths,
            ));
        }

        let mut solutions = Vec::new();
        for ant in ants {
            let solution = ant.get_solution();
            solutions.push(solution)
        }

        let start_time = Instant::now();
        let mut best_length = R64::zero();
        let mut best_score = R64::zero();
        let mut best_solution = Solution::new();
        let mut improvements = 0;
        for solution in solutions.into_iter() {
            let score_cont = solution_score(&solution, self.graph, self.heuristic);
            let length_cont = solution_length(&solution, self.graph);
            if let (Ok(score), Ok(length)) = (score_cont, length_cont) {
                if length <= self.max_time && score > best_score {
                    improvements += 1;
                    best_score = score;
                    best_length = length;
                    best_solution = solution;
                }
            }
        }

        let g_borrow = self.graph.borrow();
        let mut visited_nodes = 0;
        let mut visited_with_val = 0;
        let mut val_sum = R64::zero();
        for node in best_solution.iter_unique_nodes() {
            visited_nodes += 1;
            if let Ok(weight) = g_borrow.node_weight(node) {
                visited_with_val += 1;
                val_sum += *weight;
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
            visited_nodes,
            visited_with_val,
            val_sum,
        )); // Ant 0 is always supervisor
        self.supervisor.prepare_next();

        self.pheromone_update(&best_solution, best_score);
        if best_score > self.best_score {
            // println!("solution improved");
            self.best_solution = best_solution;
            self.best_score = best_score;
            self.best_length = best_length;

            return Some(&self.best_solution);
        } else if best_length < self.best_length && best_score == self.best_score {
            // println!("solution length improved");
            self.best_solution = best_solution;
            self.best_score = best_score;
            self.best_length = best_length;
        }
        None
    }
}
