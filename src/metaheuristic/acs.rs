mod ant;
mod params;

pub use ant::Ant;
pub use params::Params;

use crate::graph::{GenericWeightedGraph, MatrixGraph};
use crate::metaheuristic::aco::{Message, Supervisor};
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

pub struct Acs<'a, IndexType, Nw, Ew, W>
where
	IndexType: Clone,
	W: Write,
	Nw: Serialize + Add<Output = Nw>,
	Ew: Serialize + Add<Output = Ew>,
{
	graph: &'a RefCell<
		dyn GenericWeightedGraph<
			IndexType = IndexType,
			NodeWeightType = Nw,
			EdgeWeightType = Ew,
		>,
	>,
	pheromone_matrix: RefCell<MatrixGraph<IndexType, (), R64>>,
	goal_point: IndexType,
	max_time: Ew,
	heuristic: &'a Heuristic<Nw, Ew>,
	alpha: f64,
	beta: f64,
	rho: f64,
	ant_count: usize,
	q_0: f64,
	t_0: f64,
	avg_options: usize,
	best_solution: Solution<IndexType>,
	best_score: R64,
	best_length: Ew,
	pub supervisor: Supervisor<W, Nw, Ew>,
	rng: Rand64,
	inv_shortest_paths: BTreeMap<IndexType, Option<(Solution<IndexType>, Ew)>>,
}

impl<'a, IndexType, Nw, W> Acs<'a, IndexType, Nw, R64, W>
where
	IndexType: Distance<IndexType> + Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
	Nw: Copy + Zero + PartialOrd + Serialize + SmallVal,
	W: Write,
{
	fn pheromone_update(&mut self, solution: &Solution<IndexType>, solution_score: R64) {
		let to_add = R64::one() - R64::one() / solution_score;
		let mut pheromone_borrow = self.pheromone_matrix.borrow_mut();

		// pheromone decay
		for edge in pheromone_borrow.edge_ids() {
			let weight = *pheromone_borrow.edge_weight(edge).unwrap();
			let after_decay = R64::from_inner(1.0 - self.rho) * weight;
			let _res = pheromone_borrow.change_edge(edge, after_decay);
		}

		// adding best solution
		for (from, to) in solution.iter_unique_edges() {
			let weight = *pheromone_borrow.edge_weight((*from, *to)).unwrap();
			let _res = pheromone_borrow.change_edge((*from, *to), weight + to_add);
		}
	}

	pub fn set_inv_shortest_paths(
		&mut self,
		inv_shortest_paths: BTreeMap<IndexType, Option<(Solution<IndexType>, R64)>>,
	) {
		self.inv_shortest_paths = inv_shortest_paths
	}
}

impl<'a, IndexType, W> Metaheuristic<'a, IndexType, R64, R64> for Acs<'a, IndexType, R64, R64, W>
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
		let pheromones = RefCell::new(
			MatrixGraph::new(
				graph.iter_node_ids().map(|id| (id, ())).collect(),
				graph.iter_edge_ids()
					.map(|edge| (edge, R64::from_inner(1.0)))
					.collect(),
			)
			.unwrap(),
		);

		Acs {
			graph: problem.graph,
			pheromone_matrix: pheromones,
			goal_point: problem.goal_point,
			max_time: problem.max_time,
			heuristic: params.heuristic,
			alpha: params.alpha,
			beta: params.beta,
			rho: params.rho,
			ant_count: params.ant_count,
			q_0: params.q_0,
			t_0: params.t_0,
			avg_options: graph.order() / 2,
			best_solution: Solution::new(),
			best_score: R64::one(),
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
			ants.push(Ant::new(
				self.graph,
				&self.pheromone_matrix,
				self.goal_point,
				self.max_time,
				self.heuristic,
				self.rng.rand_u64() as u128 + ((self.rng.rand_u64() as u128) << 64),
				self.alpha,
				self.beta,
				self.rho,
				self.q_0,
				self.t_0,
				sender,
				id,
				&self.inv_shortest_paths,
			));
		}

		let mut solutions = Vec::new();
		for ant in ants {
			solutions.push(ant.get_solution())
		}

		let start_time = Instant::now();
		let mut best_length = R64::zero();
		let mut best_score = R64::zero();
		let mut best_solution = Solution::new();
		let mut visited_nodes = 0;
		let mut visited_with_val = 0;
		let mut val_sum = R64::zero();
		let mut improvements = 0;
		for ant_solution in solutions.into_iter() {
			if ant_solution.length <= self.max_time && ant_solution.score > best_score {
				improvements += 1;
				best_score = ant_solution.score;
				best_length = ant_solution.length;
				best_solution = ant_solution.solution;
				visited_nodes = ant_solution.visited_nodes;
				val_sum = ant_solution.val_sum;
				visited_with_val = ant_solution.visited_with_val;
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
		if best_score > self.best_score
			|| best_length < self.best_length && best_score == self.best_score
		{
			// println!("solution improved");
			self.best_solution = best_solution;
			self.best_score = best_score;
			self.best_length = best_length;

			return Some(&self.best_solution);
		}
		None
	}
}
