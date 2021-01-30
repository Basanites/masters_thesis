use crate::graph::GenericWeightedGraph;
use crate::metaheuristic::{solution_length, Solution};
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub struct TwoSwap<'a, IndexType, Nw, Ew> {
    graph: Box<dyn GenericWeightedGraph<IndexType, Nw, Ew>>,
    goal_point: IndexType,
    max_time: Ew,
    evaluator: &'a fn(Nw, Ew) -> f64,
    best_solution: Solution<IndexType>,
    best_score: f64,
    best_length: Ew,
}

#[allow(clippy::eq_op)]
impl<'a, IndexType, Nw, Ew> TwoSwap<'a, IndexType, Nw, Ew>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy,
    Ew: Copy + Add<Output = Ew> + Sub<Output = Ew> + AddAssign + SubAssign + PartialOrd + Sum,
{
    pub fn new(
        graph: Box<dyn GenericWeightedGraph<IndexType, Nw, Ew>>,
        goal_point: IndexType,
        max_time: Ew,
        evaluator: &'a fn(Nw, Ew) -> f64,
    ) -> Self {
        let first_edge_weight = *graph.edge_weight(graph.edge_ids()[0]).unwrap();

        let mut swap = TwoSwap {
            graph,
            goal_point,
            max_time,
            evaluator,
            best_solution: Solution::new(),
            best_score: 0.0,
            best_length: first_edge_weight - first_edge_weight,
        };

        swap.initialize();
        swap
    }

    fn score(&self, node_weight: Nw, edge_weight: Ew) -> f64 {
        (self.evaluator)(node_weight, edge_weight)
    }

    fn score_edge(&self, from: IndexType, to: IndexType) -> f64 {
        self.score(
            *self.graph.node_weight(to).unwrap(),
            *self.graph.edge_weight((from, to)).unwrap(),
        )
    }

    fn score_with_known_edge(&self, to: IndexType, edge_weight: Ew) -> f64 {
        self.score(*self.graph.node_weight(to).unwrap(), edge_weight)
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
                    self.score_with_known_edge(id, *weight) + self.score_edge(id, self.goal_point),
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
            self.best_length = solution_length(&self.best_solution, &self.graph).unwrap();
        }
    }

    pub fn single_iteration(&mut self) -> Option<&Solution<IndexType>> {
        let mut new_best = Solution::from_nodes(vec![self.goal_point]);
        let mut head_length = self.best_length; // initialized to the 0 of Ew
        let mut tail_length = head_length - head_length;
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
                self.score(*t_weight, original_distance)
            };
            let mut best_follow = *to;

            for (nid, weight) in self.graph.iter_neighbors(*from).unwrap() {
                // nodes that have been visited before don't have a value to us
                temp_score = if temp_visited.contains_key(&nid) {
                    0.0
                } else {
                    self.score_with_known_edge(nid, *weight)
                };
                if let Ok(return_weight) = self.graph.edge_weight((nid, *to)) {
                    // only score this edge if the to node has not yet been visited
                    temp_score += if temp_visited.contains_key(to) {
                        0.0
                    } else {
                        self.score(*t_weight, *return_weight)
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

    pub fn current_solution(&self) -> (&Solution<IndexType>, f64, Ew) {
        (&self.best_solution, self.best_score, self.best_length)
    }
}

impl<'a, IndexType, Nw, Ew> Iterator for TwoSwap<'a, IndexType, Nw, Ew>
where
    IndexType: Copy + PartialEq + Debug + Hash + Eq + Display,
    Nw: Copy,
    Ew: Copy + Add<Output = Ew> + Sub<Output = Ew> + AddAssign + SubAssign + PartialOrd + Sum,
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
        let eval: fn(f64, f64) -> f64 = |nw, ew| nw;
        let optimizer = TwoSwap::new(Box::new(graph), 0, 100.0, &eval);
        let solution = optimizer.current_solution();
        let correct = Solution::from_edges(vec![(0, 3), (3, 0)]).unwrap();
        assert_eq!(solution.0, &correct);
        assert_eq!(solution.1, 7.0);
    }

    #[test]
    fn single_iteration_works() {
        let graph = weighted_graph();
        let eval: fn(f64, f64) -> f64 = |nw, ew| nw;
        let mut optimizer = TwoSwap::new(Box::new(graph), 0, 100.0, &eval);
        let _ = optimizer.single_iteration();
        let solution = optimizer.current_solution();
        let correct = Solution::<usize>::from_edges(vec![(0, 1), (1, 3), (3, 0)]).unwrap();

        assert_eq!(solution.0, &correct);
        assert_eq!(solution.1, 7.8);
    }
}
