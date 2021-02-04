mod params;
pub use params::Params;

use crate::graph::GenericWeightedGraph;
use crate::metaheuristic::{solution_length, Heuristic, Metaheuristic, ProblemInstance, Solution};

use num_traits::identities::Zero;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Sub, SubAssign};

pub struct TwoSwap<'a, IndexType, Nw, Ew> {
    graph: &'a dyn GenericWeightedGraph<IndexType, Nw, Ew>,
    goal_point: IndexType,
    heuristic: Heuristic<IndexType, Nw, Ew>,
    max_time: Ew,
    best_solution: Solution<IndexType>,
    best_score: f64,
    best_length: Ew,
}

impl<'a, IndexType, Nw, Ew> TwoSwap<'a, IndexType, Nw, Ew>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy,
    Ew: Copy
        + Zero
        + Add<Output = Ew>
        + Sub<Output = Ew>
        + AddAssign
        + SubAssign
        + PartialOrd
        + Sum
        + Div<Output = Ew>,
{
    fn score(&self, node_weight: Nw, edge_weight: Ew, point: IndexType, distance_up_to: Ew) -> f64 {
        (self.heuristic)(
            node_weight,
            edge_weight,
            point,
            distance_up_to / self.max_time,
        )
    }

    fn score_edge(&self, from: IndexType, to: IndexType, distance_up_to: Ew) -> f64 {
        self.score(
            *self.graph.node_weight(to).unwrap(),
            *self.graph.edge_weight((from, to)).unwrap(),
            to,
            distance_up_to,
        )
    }

    fn score_with_known_edge(&self, to: IndexType, edge_weight: Ew, distance_up_to: Ew) -> f64 {
        self.score(
            *self.graph.node_weight(to).unwrap(),
            edge_weight,
            to,
            distance_up_to,
        )
    }

    pub fn initialize(&mut self) {
        // we take the node with best score we can also get back from
        let max = self
            .graph
            .iter_neighbors(self.goal_point)
            .unwrap()
            .filter(|(id, _)| self.graph.has_edge((*id, self.goal_point)))
            .map(|(id, weight)| -> (IndexType, f64) {
                (
                    id,
                    self.score_with_known_edge(id, *weight, Ew::zero())
                        + self.score_edge(id, self.goal_point, Ew::zero()),
                )
            })
            .inspect(|x| println!("{:?}", x))
            .max_by(|(_, ev_a), (_, ev_b)| ev_a.partial_cmp(ev_b).unwrap());

        // if there is no path back max will have no solution
        if let Some(solution) = max {
            self.best_solution.push_node(self.goal_point);
            self.best_solution.push_node(solution.0);
            self.best_solution.push_node(self.goal_point);
            self.best_score = solution.1;
            self.best_length = solution_length(&self.best_solution, self.graph).unwrap();
        }
    }

    pub fn current_solution(&self) -> (&Solution<IndexType>, f64, Ew) {
        (&self.best_solution, self.best_score, self.best_length)
    }
}

impl<'a, IndexType, Nw, Ew> Metaheuristic<'a, Params<IndexType, Nw, Ew>, IndexType, Nw, Ew>
    for TwoSwap<'a, IndexType, Nw, Ew>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy,
    Ew: Copy
        + Zero
        + Add<Output = Ew>
        + Sub<Output = Ew>
        + AddAssign
        + SubAssign
        + PartialOrd
        + Sum
        + Div<Output = Ew>,
{
    fn new(
        problem: ProblemInstance<'a, IndexType, Nw, Ew>,
        params: Params<IndexType, Nw, Ew>,
    ) -> Self {
        let mut swap = TwoSwap {
            graph: problem.graph,
            goal_point: problem.goal_point,
            max_time: problem.max_time,
            heuristic: params.heuristic,
            best_solution: Solution::new(),
            best_score: 0.0,
            best_length: Ew::zero(),
        };

        swap.initialize();
        swap
    }

    fn single_iteration(&mut self) -> Option<&Solution<IndexType>> {
        let mut new_best = Solution::from_nodes(vec![self.goal_point]);
        let mut head_length = self.best_length; // initialized to the 0 of Ew
        let mut tail_length = Ew::zero();
        let mut temp_visited = HashMap::new();
        let mut max: f64;
        let mut score = 0.0;
        let mut temp_score: f64;
        let mut temp_new_distance = tail_length;
        for (from, to) in self.best_solution.iter_edges() {
            let original_distance = *self.graph.edge_weight((*from, *to)).unwrap();
            temp_visited.insert(*from, true);
            let t_weight = self.graph.node_weight(*to).unwrap();
            // if we already visited the node we can ignore it
            max = if temp_visited.contains_key(to) {
                0.0
            } else {
                self.score(*t_weight, original_distance, *to, tail_length)
            };
            let mut best_follow = *to;

            for (nid, weight) in self.graph.iter_neighbors(*from).unwrap() {
                // nodes that have been visited before don't have a value to us
                temp_score = if temp_visited.contains_key(&nid) {
                    0.0
                } else {
                    self.score_with_known_edge(nid, *weight, tail_length)
                };
                if let Ok(return_weight) = self.graph.edge_weight((nid, *to)) {
                    // only score this edge if the to node has not yet been visited
                    temp_score += if temp_visited.contains_key(to) {
                        0.0
                    } else {
                        self.score(*t_weight, *return_weight, *to, tail_length + *weight)
                    };
                    let new_distance =
                        tail_length + head_length - original_distance + *weight + *return_weight;
                    if temp_score > max && new_distance <= self.max_time {
                        max = temp_score;
                        best_follow = nid;
                        temp_new_distance = *weight + *return_weight;
                    }
                }
            }

            head_length -= original_distance;
            if best_follow != *to {
                temp_visited.insert(best_follow, true);
                temp_visited.insert(*to, true);
                new_best.push_node(best_follow);
                new_best.push_node(*to);
                tail_length += temp_new_distance;
            } else {
                temp_visited.insert(*to, true);
                new_best.push_node(*to);
                tail_length += original_distance;
            }
            score += max;
        }

        if score > self.best_score {
            println!("old score: {}, new score: {}", self.best_score, score);
            println!("old: {}, new {}", self.best_solution, new_best);
            self.best_solution = new_best;
            self.best_score = score;
            self.best_length = tail_length + head_length;
            Some(&self.best_solution)
        } else {
            None
        }
    }
}

impl<'a, IndexType, Nw, Ew> Iterator for TwoSwap<'a, IndexType, Nw, Ew>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy,
    Ew: Copy
        + Zero
        + Add<Output = Ew>
        + Sub<Output = Ew>
        + AddAssign
        + SubAssign
        + PartialOrd
        + Sum
        + Div<Output = Ew>,
{
    type Item = Solution<IndexType>;

    fn next(&mut self) -> Option<Self::Item> {
        println!("called next");
        self.single_iteration().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::MatrixGraph;
    use crate::metaheuristic::Metaheuristic;

    fn weighted_graph() -> MatrixGraph<usize, f64, f64> {
        MatrixGraph::new_usize_indexed(
            vec![0.0, 0.8, 12.0, 7.0, 2.5],
            vec![
                (0, 1, 12.0),
                (0, 3, 2.0),
                (1, 0, 7.0),
                (1, 2, 16.0),
                (1, 3, 1.5),
                (2, 1, 13.5),
                (2, 4, 23.0),
                (3, 0, 8.1),
                (3, 1, 27.0),
                (3, 4, 7.5),
                (4, 1, 7.0),
                (4, 2, 12.0),
                (4, 3, 7.5),
            ],
        )
        .unwrap()
    }

    #[test]
    fn initialization_works() {
        let graph = weighted_graph();
        let optimizer = TwoSwap::new(
            ProblemInstance::new(&graph, 0, 100.0),
            Params::new(|nw, _, _, _| nw),
        );
        let solution = optimizer.current_solution();
        let correct = Solution::from_edges(vec![(0, 3), (3, 0)]).unwrap();
        assert_eq!(solution.0, &correct);
        assert_eq!(solution.1, 7.0);
    }

    #[test]
    fn single_iteration_works() {
        let graph = weighted_graph();
        let mut optimizer = TwoSwap::new(
            ProblemInstance::new(&graph, 0, 100.0),
            Params::new(|nw, _, _, _| nw),
        );
        let _ = optimizer.single_iteration();
        let solution = optimizer.current_solution();
        let correct = Solution::<usize>::from_edges(vec![(0, 1), (1, 3), (3, 0)]).unwrap();

        assert_eq!(solution.0, &correct);
        assert_eq!(solution.1, 7.8);
    }
}
