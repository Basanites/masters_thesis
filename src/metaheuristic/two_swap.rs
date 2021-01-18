use crate::graph::{GenericWeightedGraph, Edge};
use std::fmt::Debug;
use std::cmp::{PartialEq, Eq};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Add, Sub};

pub struct TwoSwap<'a, IndexType, Nw, Ew> {
    graph: Box<dyn GenericWeightedGraph<IndexType, Nw, Ew>>,
    goal_point: IndexType,
    max_time: Ew,
    evaluator: &'a fn(Nw, Ew) -> f64,
    best_solution: Vec<Edge<IndexType>>,
    best_score: f64,
    best_length: Ew,
    visited_nodes: HashMap<IndexType, bool>,
}

impl<'a, IndexType: Copy + PartialEq + Debug + Hash + Eq, Nw: Copy, Ew: Copy + Add<Output = Ew> + Sub<Output = Ew>> TwoSwap<'a, IndexType, Nw, Ew> {
    pub fn new(graph: Box<dyn GenericWeightedGraph<IndexType, Nw, Ew>>, goal_point: IndexType, max_time: Ew, evaluator: &'a fn(Nw, Ew) -> f64) -> Self {
        let first_edge_weight = *graph.edge_weight(graph.edge_ids()[0]).unwrap();

        let mut swap = TwoSwap {
            graph,
            goal_point,
            max_time,
            evaluator,
            best_solution: Vec::new(),
            best_score: 0.0,
            best_length: first_edge_weight - first_edge_weight,
            visited_nodes: HashMap::new(),
        };

        swap.initialize();
        swap
    }

    fn score(&self, node_weight: Nw, edge_weight: Ew) -> f64 {
        (self.evaluator)(node_weight, edge_weight)
    }

    fn score_edge(&self, from: IndexType, to: IndexType) -> f64 {
        self.score(*self.graph.node_weight(to).unwrap(), *self.graph.edge_weight((from, to)).unwrap())
    }

    fn score_with_known_edge(&self, to: IndexType, edge_weight: Ew) -> f64 {
        self.score(*self.graph.node_weight(to).unwrap(), edge_weight)
    }

    pub fn initialize(&mut self) {
        let max = self.graph.iter_neighbors(self.goal_point).unwrap()
            .filter(|(id, _)| self.graph.has_edge((*id, self.goal_point)))
            .map(|(id, weight)| -> (IndexType, f64) {
                (id, self.score_with_known_edge(id, *weight) + self.score_edge(id, self.goal_point))
            })
            .inspect(|x| println!("{:?}", x))
            .max_by(|(_, ev_a), (_, ev_b)| ev_a.partial_cmp(ev_b).unwrap());

        // if there is no path back max will have no solution
        if let Some(solution) = max {
            self.best_solution.push((self.goal_point, solution.0));
            self.best_solution.push((solution.0, self.goal_point));
            self.best_score = solution.1;
            self.visited_nodes.insert(self.goal_point, true);
            self.visited_nodes.insert(solution.0, true);
        }
    }

    pub fn single_iteration(&mut self) -> Option<&Vec<Edge<IndexType>>> {
        let mut new_best = Vec::new();
        let mut max: f64;
        let mut score = 0.0;
        let mut temp_score: f64;
        for (from, to) in self.best_solution.iter() {
            let t_weight = self.graph.node_weight(*to).unwrap();
            max = self.score(*t_weight, *self.graph.edge_weight((*from, *to)).unwrap());
            let mut best_follow = *to;

            for (nid, weight) in self.graph.iter_neighbors(*from).unwrap() {
                // only visit nodes, that have not yet been visited
                if !self.visited_nodes.contains_key(&nid) {
                    temp_score = self.score_with_known_edge(nid, *weight);
                    if let Ok(return_weight) = self.graph.edge_weight((nid, *to)) {
                        temp_score += self.score(*t_weight, *return_weight);
                        if temp_score > max {
                            max = temp_score;
                            best_follow = nid;
                        }
                    }
                }
            }

            if best_follow != *to {
                new_best.push((*from, best_follow));
                new_best.push((best_follow, *to));
                self.visited_nodes.insert(best_follow, true);
            } else {
                new_best.push((*from, *to));
            }
            score += max;
        }

        if score > self.best_score {
            println!("old score: {}, new score: {}", self.best_score, score);
            println!("old: {:?}, new {:?}", self.best_solution, new_best);
            self.best_solution = new_best;
            self.best_score = score;
            Some(&self.best_solution)
        } else {
            None
        }
    }

    pub fn current_solution(&self) -> (&Vec<Edge<IndexType>>, f64) {
        (&self.best_solution, self.best_score)
    }
}

impl<'a, IndexType: Copy + PartialEq + Debug + Hash + Eq, Nw: Copy, Ew: Copy + Add<Output = Ew> + Sub<Output = Ew>> Iterator for TwoSwap<'a, IndexType, Nw, Ew> {
    type Item = Vec<Edge<IndexType>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.single_iteration().cloned()
    }
}