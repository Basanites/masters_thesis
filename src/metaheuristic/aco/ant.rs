use crate::graph::{GenericWeightedGraph, MatrixGraph};
use crate::metaheuristic::aco::Message;
use crate::metaheuristic::{Heuristic, Solution};
use crate::rng::rng64;

use num_traits::identities::Zero;
use num_traits::Pow;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::mpsc::Sender;
use std::time::Instant;

#[derive(Clone)]
pub struct Ant<'a, IndexType: Clone, Nw, Ew> {
    graph: &'a dyn GenericWeightedGraph<IndexType, Nw, Ew>,
    pheromone_matrix: &'a MatrixGraph<IndexType, (), f64>,
    goal_point: IndexType,
    max_time: Ew,
    alpha: f64,
    beta: f64,
    rng_seed: u128,
    heuristic: &'a Heuristic<IndexType, Nw, Ew>,
    sender: Sender<Message>,
    id: usize,
}

impl<'a, IndexType, Nw> Ant<'a, IndexType, Nw, f64>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy + Zero,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        graph: &'a dyn GenericWeightedGraph<IndexType, Nw, f64>,
        pheromone_matrix: &'a MatrixGraph<IndexType, (), f64>,
        goal_point: IndexType,
        max_time: f64,
        heuristic: &'a Heuristic<IndexType, Nw, f64>,
        rng_seed: u128,
        alpha: f64,
        beta: f64,
        sender: Sender<Message>,
        id: usize,
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
            sender,
            id,
        }
    }

    fn weighted_heuristic(&self, to: IndexType, edge_weight: f64, tail_length: f64) -> f64 {
        self.weighted_heuristic_with_known_val(
            *self.graph.node_weight(to).unwrap(),
            to,
            edge_weight,
            tail_length,
        )
    }

    fn weighted_heuristic_with_known_val(
        &self,
        value: Nw,
        to: IndexType,
        edge_weight: f64,
        tail_length: f64,
    ) -> f64 {
        f64::pow(
            (self.heuristic)(value, edge_weight, to, tail_length / self.max_time),
            self.beta,
        )
    }

    /// If condition is true, node weight is assumed to be 0, else the weight from the graph is used.
    fn conditional_weighted_heuristic(
        &self,
        cond: bool,
        to: IndexType,
        edge_weight: f64,
        tail_length: f64,
    ) -> f64 {
        if cond {
            self.weighted_heuristic(to, edge_weight, tail_length)
        } else {
            self.weighted_heuristic_with_known_val(Nw::zero(), to, edge_weight, tail_length)
        }
    }

    pub fn get_solution(&self) -> Solution<IndexType> {
        let start_time = Instant::now();
        let mut evals = 0;
        let mut rng = rng64(self.rng_seed);
        let mut solution = Solution::new();
        solution.push_node(self.goal_point);

        let mut tail_length = 0.0;
        let mut next_node = self.goal_point;
        let mut goal_reached = false;
        let mut visited: HashMap<IndexType, bool> = HashMap::new();
        while !goal_reached {
            // calculate the sums of the weighted heuristic and pheromones for all neighbors of next_node
            let weighted_pheromone_sum = self
                .pheromone_matrix
                .iter_neighbors(next_node)
                .unwrap()
                .fold(0.0, |acc, (_, weight)| acc + f64::pow(*weight, self.alpha));
            let weighted_heuristic_sum = self
                .graph
                .iter_neighbors(next_node)
                .unwrap()
                .inspect(|_| evals += 1) // increment evals for each call to heuristic
                .fold(0.0, |acc, (to, weight)| {
                    acc + self.weighted_heuristic(to, *weight, tail_length)
                });

            // as soon, as we reach a point where the sum of the weighted pheromones and heuristic
            // is equal to the random number, we have hit the value with the correct probability
            // according to the formula at https://en.wikipedia.org/wiki/Ant_colony_optimization_algorithms#Edge_selection
            let rand = rng.rand_float() * weighted_heuristic_sum * weighted_pheromone_sum;
            let mut sum = 0.0;
            for (id, pheromone_level) in self.pheromone_matrix.iter_neighbors(next_node).unwrap() {
                // the edge weight we want to use for the heuristic needs to be got from the distance graph,
                // not the pheromone graph, so we have to get it from there specifically
                let distance = *self.graph.edge_weight((next_node, id)).unwrap();
                let weighted_heuristic = self.conditional_weighted_heuristic(
                    visited.contains_key(&id),
                    id,
                    distance,
                    tail_length,
                );
                sum += weighted_heuristic * f64::pow(*pheromone_level, self.alpha);
                evals += 1;

                // sum is bigger than the random value we generated, so we hit our node
                // with the correct probability
                if sum >= rand {
                    solution.push_node(id);
                    tail_length += distance;
                    visited.insert(id, true);
                    if id == self.goal_point {
                        goal_reached = true;
                    }
                    next_node = id;
                    break;
                }
            }
        }

        // TODO: log errors from sending here
        let _res = self
            .sender
            .send(Message::new(self.id, evals, start_time.elapsed()));
        solution
    }
}
