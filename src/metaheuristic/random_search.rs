mod message;
mod params;
mod supervisor;

pub use message::Message;
pub use params::Params;
pub use supervisor::Supervisor;

use crate::graph::GenericWeightedGraph;
use crate::metaheuristic::{Heuristic, Metaheuristic, ProblemInstance, Solution};
use crate::rng::rng64;

use decorum::R64;
use num_traits::identities::Zero;
use oorandom::Rand64;
use serde::Serialize;
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::default::Default;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::io::Write;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Sub, SubAssign};
use std::time::{Duration, Instant};

pub struct RandomSearch<
    'a,
    IndexType,
    NodeWeightType: Serialize + Default,
    EdgeWeightType: Serialize + Default,
    W: Write,
> {
    graph: &'a RefCell<
        dyn GenericWeightedGraph<
            IndexType = IndexType,
            NodeWeightType = NodeWeightType,
            EdgeWeightType = EdgeWeightType,
        >,
    >,
    goal_point: IndexType,
    heuristic: &'a Heuristic<IndexType, NodeWeightType, EdgeWeightType>,
    max_time: EdgeWeightType,
    pub best_solution: Solution<IndexType>,
    pub best_score: R64,
    pub best_length: EdgeWeightType,
    pub supervisor: Supervisor<W, NodeWeightType, EdgeWeightType>,
    i: usize,
    inv_shortest_paths: &'a HashMap<IndexType, Option<(Solution<IndexType>, EdgeWeightType)>>,
    rng: Rand64,
}

impl<'a, IndexType, NodeWeightType, EdgeWeightType, W>
    RandomSearch<'a, IndexType, NodeWeightType, EdgeWeightType, W>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    NodeWeightType: Copy
        + Debug
        + Add<Output = NodeWeightType>
        + Sub<Output = NodeWeightType>
        + Serialize
        + Default
        + Zero
        + AddAssign<NodeWeightType>
        + PartialEq,
    EdgeWeightType: Copy
        + Zero
        + Add<Output = EdgeWeightType>
        + Sub<Output = EdgeWeightType>
        + AddAssign
        + SubAssign
        + PartialOrd
        + Sum
        + Div<Output = EdgeWeightType>
        + Default
        + Serialize
        + Debug,
    W: Write,
{
    pub fn current_solution(&self) -> (&Solution<IndexType>, R64, EdgeWeightType) {
        (&self.best_solution, self.best_score, self.best_length)
    }

    pub fn solve(&mut self) {
        while self.next().is_some() {}
        self.supervisor.aggregate_receive();
    }

    fn send_message(
        &self,
        iteration: usize,
        evaluations: usize,
        n_improvements: usize,
        changes: usize,
        phase: usize,
        cpu_time: Duration,
        distance: EdgeWeightType,
        solution: &Solution<IndexType>,
    ) {
        let tx = self.supervisor.sender();

        let g_borrow = self.graph.borrow();
        let mut heuristic_score = R64::zero();
        let mut visited: HashMap<IndexType, bool> = HashMap::new();
        let mut length = EdgeWeightType::zero();
        for (from, to) in solution.iter_edges() {
            let dist = *g_borrow.edge_weight((*from, *to)).unwrap();
            length += dist;
            if !visited.contains_key(to) {
                heuristic_score +=
                    (self.heuristic)(*g_borrow.node_weight(*to).unwrap(), dist, *to, length);
                visited.insert(*to, true);
            } else {
                heuristic_score += (self.heuristic)(NodeWeightType::zero(), dist, *to, length);
            }
        }

        let mut visited_nodes = 0;
        let mut val_sum = NodeWeightType::zero();
        let mut visited_with_val = 0;
        for node in solution.iter_unique_nodes() {
            visited_nodes += 1;
            if let Ok(weight) = g_borrow.node_weight(node) {
                if *weight != NodeWeightType::zero() {
                    visited_with_val += 1;
                    val_sum += *weight - NodeWeightType::zero();
                }
            }
        }

        tx.send(Message::new(
            iteration,
            evaluations,
            n_improvements,
            changes,
            phase,
            cpu_time,
            distance,
            heuristic_score,
            visited_nodes,
            visited_with_val,
            val_sum,
        ))
        .unwrap();
    }

    fn generate(&mut self, start_time: Instant) -> bool {
        let mut visited: HashMap<IndexType, bool> = HashMap::new();
        let mut length = EdgeWeightType::zero();
        let mut solution = Solution::from_nodes(vec![self.goal_point]);
        let mut goal_reached = false;
        let mut next_node = self.goal_point;
        while !goal_reached {
            let viable_candidates: Vec<_> = self
                .graph
                .borrow()
                .iter_neighbor_ids(next_node)
                .unwrap()
                .filter(|node| {
                    if let Some((_, weight)) = &self.inv_shortest_paths[node] {
                        let &weight_to =
                            self.graph.borrow().edge_weight((next_node, *node)).unwrap();
                        if length + *weight + weight_to <= self.max_time {
                            return true;
                        }
                    }

                    false
                })
                .collect();

            // as soon as we have no more candidates to travel to we can just take our calculated shortest path
            if viable_candidates.is_empty() {
                // if we added the path even when we have reached the goal point we get it twice at the end of the solution
                if next_node != self.goal_point {
                    let (mut path, distance) = self.inv_shortest_paths[&next_node].clone().unwrap();
                    solution.append(&mut path);
                    length += distance;
                }
                goal_reached = true;
            }
            let rand = (viable_candidates.len() as f64 * self.rng.rand_float()) as usize;
            let new_next_node = viable_candidates[rand];
            length += *self
                .graph
                .borrow()
                .edge_weight((next_node, new_next_node))
                .unwrap();
            solution.push_node(new_next_node);
            visited.insert(new_next_node, true);
            next_node = new_next_node
        }

        if length < self.best_length {
            self.send_message(
                self.i,
                solution.nodes().len(),
                0,
                0,
                0,
                start_time.elapsed(),
                length,
                &solution,
            );

            self.i += 1;
            true
        } else {
            false
        }
    }
}

impl<'a, IndexType, Nw, Ew, W> Metaheuristic<'a, IndexType, Nw, Ew>
    for RandomSearch<'a, IndexType, Nw, Ew, W>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    Nw: Copy
        + Debug
        + Add<Output = Nw>
        + Sub<Output = Nw>
        + Serialize
        + Default
        + Zero
        + AddAssign<Nw>
        + PartialEq,
    Ew: Copy
        + Zero
        + Add<Output = Ew>
        + Sub<Output = Ew>
        + AddAssign
        + SubAssign
        + PartialOrd
        + Sum
        + Div<Output = Ew>
        + Default
        + Serialize
        + Debug,
    W: Write,
{
    type Params = Params<'a, IndexType, Nw, Ew>;
    type SupervisorType = Supervisor<W, Nw, Ew>;

    fn new(
        problem: ProblemInstance<'a, IndexType, Nw, Ew>,
        params: Self::Params,
        supervisor: Self::SupervisorType,
    ) -> Self {
        RandomSearch {
            graph: problem.graph,
            goal_point: problem.goal_point,
            max_time: problem.max_time,
            heuristic: params.heuristic,
            best_solution: Solution::new(),
            best_score: R64::zero(),
            best_length: Ew::zero(),
            supervisor,
            i: 0,
            inv_shortest_paths: params.inv_shortest_paths,
            rng: rng64(params.seed),
        }
    }

    fn single_iteration(&mut self) -> Option<&Solution<IndexType>> {
        let start_time = Instant::now();
        if self.generate(start_time) {
            Some(&self.best_solution)
        } else {
            self.send_message(
                self.i,
                0,
                0,
                0,
                2,
                start_time.elapsed(),
                self.best_length,
                &self.best_solution,
            );
            self.i += 1;

            None
        }
    }
}

impl<'a, IndexType, Nw, Ew, W> Iterator for RandomSearch<'a, IndexType, Nw, Ew, W>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    Nw: Copy
        + Debug
        + Add<Output = Nw>
        + Sub<Output = Nw>
        + Serialize
        + Default
        + Zero
        + AddAssign<Nw>
        + PartialEq,
    Ew: Copy
        + Zero
        + Add<Output = Ew>
        + Sub<Output = Ew>
        + AddAssign
        + SubAssign
        + PartialOrd
        + Sum
        + Div<Output = Ew>
        + Default
        + Serialize
        + Debug,
    W: Write,
{
    type Item = Solution<IndexType>;

    fn next(&mut self) -> Option<Self::Item> {
        self.single_iteration().cloned()
    }
}
