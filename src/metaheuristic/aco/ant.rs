use crate::graph::{GenericWeightedGraph, MatrixGraph};
use crate::metaheuristic::aco::Message;
use crate::metaheuristic::{Heuristic, Solution};
use crate::rng::rng64;
use crate::util::Distance;

use decorum::{Real, R64};
use num_traits::identities::{One, Zero};
use serde::Serialize;
use std::cell::RefCell;
use std::cmp::{Eq, PartialEq};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::AddAssign;
use std::sync::mpsc::Sender;
use std::time::Instant;

#[derive(Clone)]
pub struct Ant<'a, IndexType: Clone, Nw, Ew>
where
    Ew: Serialize,
{
    graph: &'a RefCell<
        dyn GenericWeightedGraph<IndexType = IndexType, NodeWeightType = Nw, EdgeWeightType = Ew>,
    >,
    pheromone_matrix: &'a MatrixGraph<IndexType, (), R64>,
    goal_point: IndexType,
    max_time: Ew,
    alpha: f64,
    beta: f64,
    q_0: f64,
    rng_seed: u128,
    heuristic: &'a Heuristic<Nw, Ew>,
    sender: Sender<Message<Nw, Ew>>,
    id: usize,
    inv_shortest_paths: &'a BTreeMap<IndexType, Option<(Solution<IndexType>, Ew)>>,
}

impl<'a, IndexType, Nw> Ant<'a, IndexType, Nw, R64>
where
    IndexType: Distance<IndexType> + Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    Nw: Copy + Zero + One + AddAssign<Nw> + PartialEq,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        graph: &'a RefCell<
            dyn GenericWeightedGraph<
                IndexType = IndexType,
                NodeWeightType = Nw,
                EdgeWeightType = R64,
            >,
        >,
        pheromone_matrix: &'a MatrixGraph<IndexType, (), R64>,
        goal_point: IndexType,
        max_time: R64,
        heuristic: &'a Heuristic<Nw, R64>,
        rng_seed: u128,
        alpha: f64,
        beta: f64,
        q_0: f64,
        sender: Sender<Message<Nw, R64>>,
        id: usize,
        inv_shortest_paths: &'a BTreeMap<IndexType, Option<(Solution<IndexType>, R64)>>,
    ) -> Self {
        Ant {
            graph,
            pheromone_matrix,
            goal_point,
            max_time,
            heuristic,
            rng_seed,
            alpha,
            beta,
            q_0,
            sender,
            id,
            inv_shortest_paths,
        }
    }

    fn weighted_heuristic(&self, to: IndexType, edge_weight: R64, tail_length: R64) -> R64 {
        self.weighted_heuristic_with_known_val(
            *self.graph.borrow().node_weight(to).unwrap(),
            to,
            edge_weight,
            tail_length,
        )
    }

    fn weighted_heuristic_with_known_val(
        &self,
        value: Nw,
        to: IndexType,
        edge_weight: R64,
        tail_length: R64,
    ) -> R64 {
        R64::powf(
            (self.heuristic)(
                value,
                edge_weight,
                IndexType::distance(self.goal_point, to),
                tail_length / self.max_time,
            ),
            R64::from_inner(self.beta),
        )
    }

    /// If condition is true, node weight is assumed to be 0, else the weight from the graph is used.
    fn conditional_weighted_heuristic(
        &self,
        cond: bool,
        to: IndexType,
        edge_weight: R64,
        tail_length: R64,
    ) -> R64 {
        if cond {
            self.weighted_heuristic(to, edge_weight, tail_length)
        } else {
            // self.weighted_heuristic_with_known_val(Nw::one(), to, edge_weight, tail_length)
            self.weighted_heuristic_with_known_val(Nw::zero(), to, edge_weight, tail_length)
        }
    }

    pub fn get_solution(&self) -> AntSolution<IndexType, Nw> {
        let start_time = Instant::now();
        let mut evals = 0;
        let mut changes = 0;
        let mut score = R64::zero();
        let mut rng = rng64(self.rng_seed);
        let mut solution = Solution::new();
        solution.push_node(self.goal_point);

        let mut tail_length = R64::zero();
        let mut next_node = self.goal_point;
        let mut goal_reached = false;
        let mut visited: BTreeMap<IndexType, bool> = BTreeMap::new();
        let mut val_sum = Nw::zero();
        let mut nodes_with_val = 0;
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
                        if tail_length + *weight + weight_to <= self.max_time {
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
                    tail_length += distance;
                }
                goal_reached = true;
            }

            // weighted pheromone sum will be used in case we visited all neighbors of this node
            let weighted_pheromone_sum = viable_candidates
                .iter()
                .map(|&id| self.pheromone_matrix.edge_weight((next_node, id)).unwrap())
                .fold(R64::zero(), |acc, weight| {
                    acc + R64::powf(*weight, R64::from_inner(self.alpha))
                });
            // the default is using the weighted sum as given by the paper
            let mut weighted_sum = viable_candidates
                .iter()
                .map(|&id| {
                    (
                        id,
                        *self.graph.borrow().edge_weight((next_node, id)).unwrap(),
                        self.pheromone_matrix.edge_weight((next_node, id)).unwrap(),
                    )
                })
                .inspect(|_| evals += 1) // increment evals for each call to heuristic
                .fold(R64::zero(), |acc, (to, h_weight, p_weight)| {
                    acc + self.conditional_weighted_heuristic(
                        !visited.contains_key(&to),
                        to,
                        h_weight,
                        tail_length,
                    ) * R64::powf(*p_weight, R64::from_inner(self.alpha))
                });

            let mut visited_all_viable = false;
            if weighted_sum == R64::zero() {
                weighted_sum = weighted_pheromone_sum;
                visited_all_viable = true;
            }

            // as soon, as we reach a point where the sum of the weighted pheromones and heuristic
            // is equal to the random number, we have hit the value with the correct probability
            // according to the formula at https://en.wikipedia.org/wiki/Ant_colony_optimization_algorithms#Edge_selection
            let rand = R64::from_inner(rng.rand_float()) * weighted_sum;

            let mut sum = R64::zero();
            for &id in viable_candidates.iter() {
                let pheromone_level = self.pheromone_matrix.edge_weight((next_node, id)).unwrap();
                let distance = *self.graph.borrow().edge_weight((next_node, id)).unwrap();
                let weighted_heuristic = if !visited_all_viable {
                    self.conditional_weighted_heuristic(
                        !visited.contains_key(&id),
                        id,
                        distance,
                        tail_length,
                    )
                } else {
                    R64::one()
                };
                sum +=
                    weighted_heuristic * R64::powf(*pheromone_level, R64::from_inner(self.alpha));
                evals += 1;

                // sum is bigger than the random value we generated, so we hit our node
                // with the correct probability
                if sum >= rand {
                    // add to value sum and nodes with val
                    let borrow = self.graph.borrow();
                    let nw = borrow.node_weight(id);
                    if !visited.contains_key(&id) && nw.is_ok() {
                        let nw_val = *nw.unwrap();
                        if nw_val != Nw::zero() {
                            nodes_with_val += 1;
                            val_sum += nw_val;
                        }
                    }

                    solution.push_node(id);
                    tail_length += distance;
                    score += weighted_heuristic;
                    visited.insert(id, true);
                    changes += 1;
                    // if id == self.goal_point {
                    //     goal_reached = true;
                    // }
                    next_node = id;
                    break;
                }
            }
        }

        let visited_nodes = visited.len();

        // TODO: log errors from sending here
        let _res = self.sender.send(Message::new(
            self.id,
            0,
            evals,
            0,
            changes,
            0,
            start_time.elapsed(),
            tail_length,
            score,
            visited_nodes,
            nodes_with_val,
            val_sum,
        ));

        AntSolution {
            solution,
            length: tail_length,
            score,
            visited_nodes,
            visited_with_val: nodes_with_val,
            val_sum,
        }
    }
}

pub struct AntSolution<IndexType, NwType> {
    pub solution: Solution<IndexType>,
    pub length: R64,
    pub score: R64,
    pub visited_nodes: usize,
    pub visited_with_val: usize,
    pub val_sum: NwType,
}
