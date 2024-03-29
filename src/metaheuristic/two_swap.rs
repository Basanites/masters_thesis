mod message;
mod params;
mod supervisor;

pub use message::Message;
pub use params::Params;
pub use supervisor::Supervisor;

use crate::graph::GenericWeightedGraph;
use crate::metaheuristic::{solution_length, Heuristic, Metaheuristic, ProblemInstance, Solution};
use crate::util::{Distance, SmallVal};

use decorum::R64;
use num_traits::identities::Zero;
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

pub struct TwoSwap<
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
    heuristic: &'a Heuristic<NodeWeightType, EdgeWeightType>,
    max_time: EdgeWeightType,
    pub best_solution: Solution<IndexType>,
    pub best_score: R64,
    pub best_length: EdgeWeightType,
    pub supervisor: Supervisor<W, NodeWeightType, EdgeWeightType>,
    i: usize,
}

impl<'a, IndexType, NodeWeightType, EdgeWeightType, W>
    TwoSwap<'a, IndexType, NodeWeightType, EdgeWeightType, W>
where
    IndexType: Distance<IndexType> + Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    NodeWeightType: Copy
        + Debug
        + Add<Output = NodeWeightType>
        + Sub<Output = NodeWeightType>
        + Serialize
        + Default
        + Zero
        + AddAssign<NodeWeightType>
        + PartialEq
        + SmallVal,
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
    fn score(
        &self,
        node_weight: NodeWeightType,
        edge_weight: EdgeWeightType,
        point: IndexType,
        distance_up_to: EdgeWeightType,
    ) -> R64 {
        (self.heuristic)(
            node_weight,
            edge_weight,
            IndexType::distance(self.goal_point, point),
            distance_up_to / self.max_time,
        )
    }

    fn score_edge(&self, from: IndexType, to: IndexType, distance_up_to: EdgeWeightType) -> R64 {
        self.score(
            *self.graph.borrow().node_weight(to).unwrap(),
            *self.graph.borrow().edge_weight((from, to)).unwrap(),
            to,
            distance_up_to,
        )
    }

    fn score_with_known_edge(
        &self,
        to: IndexType,
        edge_weight: EdgeWeightType,
        distance_up_to: EdgeWeightType,
    ) -> R64 {
        self.score(
            *self.graph.borrow().node_weight(to).unwrap(),
            edge_weight,
            to,
            distance_up_to,
        )
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
        heuristic_score: R64,
        solution: &Solution<IndexType>,
    ) {
        let tx = self.supervisor.sender();

        let g_borrow = self.graph.borrow();
        let mut visited_nodes = 0;
        let mut val_sum = NodeWeightType::zero();
        let mut visited_with_val = 0;
        for node in solution.iter_unique_nodes() {
            visited_nodes += 1;
            if let Ok(weight) = g_borrow.node_weight(node) {
                if *weight != NodeWeightType::small() {
                    visited_with_val += 1;
                    val_sum += *weight - NodeWeightType::small();
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

    pub fn initialize(&mut self) {
        let start_time = Instant::now();
        let mut evals = 0;
        // we take the node with best score we can also get back from
        let max = self
            .graph
            .borrow()
            .iter_neighbors(self.goal_point)
            .unwrap()
            .filter(|(id, _)| self.graph.borrow().has_edge((*id, self.goal_point)))
            .map(|(id, weight)| -> (IndexType, R64) {
                (
                    id,
                    self.score_with_known_edge(id, *weight, EdgeWeightType::zero())
                        + self.score_edge(id, self.goal_point, EdgeWeightType::zero()),
                )
            })
            .inspect(|_| evals += 1)
            .max_by(|(_, ev_a), (_, ev_b)| ev_a.partial_cmp(ev_b).unwrap());

        // if there is no path back max will have no solution
        if let Some(solution) = max {
            self.best_solution.push_node(self.goal_point);
            self.best_solution.push_node(solution.0);
            self.best_solution.push_node(self.goal_point);
            self.best_score = solution.1;
            self.best_length = solution_length(&self.best_solution, self.graph).unwrap();
        }

        self.send_message(
            self.i,
            evals,
            0,
            1,
            0,
            start_time.elapsed(),
            self.best_length,
            self.best_score,
            &self.best_solution,
        );
        self.i += 1;
    }

    pub fn current_solution(&self) -> (&Solution<IndexType>, R64, EdgeWeightType) {
        (&self.best_solution, self.best_score, self.best_length)
    }

    pub fn solve(&mut self) {
        while self.next().is_some() {}
        self.supervisor.aggregate_receive();
    }

    pub fn expand(&mut self, start_time: Instant) -> bool {
        let mut evals = 0;
        let mut new_best = Solution::from_nodes(vec![self.goal_point]);
        let mut head_length = self.best_length; // initialized to the 0 of Ew
        let mut tail_length = EdgeWeightType::zero();
        let mut temp_visited = HashMap::new();
        let mut max: R64;
        let mut score = R64::zero();
        let mut prev_best_score = self.best_score;
        let mut temp_score: R64;
        let mut temp_new_distance = tail_length;
        let mut improvements = 0;
        let mut changes = 0;
        let g_borrowed = self.graph.borrow();
        for (from, to) in self.best_solution.iter_edges() {
            let original_distance = *g_borrowed.edge_weight((*from, *to)).unwrap();
            // temp_visited.insert(*from, true);
            let t_weight = g_borrowed.node_weight(*to).unwrap();
            // if we already visited the node we can ignore it
            max = if temp_visited.contains_key(to) {
                R64::zero()
            } else {
                evals += 1;
                self.score(*t_weight, original_distance, *to, tail_length)
            };
            let mut best_follow = *to;

            for (nid, weight) in g_borrowed.iter_neighbors(*from).unwrap() {
                // nodes that have been visited before don't have a value to us
                temp_score = if temp_visited.contains_key(&nid) {
                    R64::zero()
                } else {
                    evals += 1;
                    self.score_with_known_edge(nid, *weight, tail_length)
                };
                if let Ok(return_weight) = g_borrowed.edge_weight((nid, *to)) {
                    // only score this edge if the to node has not yet been visited
                    temp_score += if temp_visited.contains_key(to) {
                        R64::zero()
                    } else {
                        evals += 1;
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
                changes += 1;
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
            if score > prev_best_score {
                improvements += 1;
                prev_best_score = score;
            }
        }

        if score > self.best_score {
            self.send_message(
                self.i,
                evals,
                improvements,
                changes,
                0,
                start_time.elapsed(),
                tail_length,
                score,
                &new_best,
            );

            self.i += 1;
            self.best_solution = new_best;
            self.best_score = score;
            self.best_length = tail_length;

            true
        } else {
            false
        }
    }

    pub fn contract(&mut self, start_time: Instant) -> bool {
        let mut temp_visited = HashMap::new();
        let mut length = EdgeWeightType::zero();
        let mut improvements = 0;
        let nodes = self.best_solution.nodes();
        let mut new_solution = Solution::from_nodes(vec![self.goal_point]);
        let mut i = 0;
        while i < nodes.len() - 1 {
            temp_visited.insert(nodes[i], true);
            // node has at least two following nodes
            if i < nodes.len() - 3
                // next node has already been visited
                && temp_visited.contains_key(&nodes[i + 1])
                // there is a direct path to the node after the next
                && self
                    .graph
                    .borrow()
                    .iter_neighbors(nodes[i])
                    .unwrap()
                    .any(|(id, _)| id == nodes[i + 2])
            {
                let o_dist = *self
                    .graph
                    .borrow()
                    .edge_weight((nodes[i], nodes[i + 1]))
                    .unwrap();
                let n_dist = *self
                    .graph
                    .borrow()
                    .edge_weight((nodes[i], nodes[i + 2]))
                    .unwrap();
                // and that path is shorter, than the old one would have been
                if n_dist
                    < o_dist
                        + *self
                            .graph
                            .borrow()
                            .edge_weight((nodes[i + 1], nodes[i + 2]))
                            .unwrap()
                {
                    length += n_dist;
                    improvements += 1;
                    new_solution.push_node(nodes[i + 2]);
                    i += 2;
                // otherwise we just take the next node from the old path
                } else {
                    length += o_dist;
                    new_solution.push_node(nodes[i + 1]);
                    i += 1;
                }
            // path is not long enough to fit a node skip
            } else {
                length += *self
                    .graph
                    .borrow()
                    .edge_weight((nodes[i], nodes[i + 1]))
                    .unwrap();
                new_solution.push_node(nodes[i + 1]);
                i += 1;
            }
        }

        if improvements != 0 {
            self.send_message(
                self.i,
                0,
                0,
                improvements,
                1,
                start_time.elapsed(),
                length,
                self.best_score,
                &new_solution,
            );

            self.i += 1;
            self.best_solution = new_solution;
            self.best_length = length;

            true
        } else {
            false
        }
    }
}

impl<'a, IndexType, Nw, Ew, W> Metaheuristic<'a, IndexType, Nw, Ew>
    for TwoSwap<'a, IndexType, Nw, Ew, W>
where
    IndexType: Distance<IndexType> + Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    Nw: Copy
        + Debug
        + Add<Output = Nw>
        + Sub<Output = Nw>
        + Serialize
        + Default
        + Zero
        + AddAssign<Nw>
        + PartialEq
        + SmallVal,
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
    type Params = Params<'a, Nw, Ew>;
    type SupervisorType = Supervisor<W, Nw, Ew>;

    fn new(
        problem: ProblemInstance<'a, IndexType, Nw, Ew>,
        params: Self::Params,
        supervisor: Self::SupervisorType,
    ) -> Self {
        let mut swap = TwoSwap {
            graph: problem.graph,
            goal_point: problem.goal_point,
            max_time: problem.max_time,
            heuristic: params.heuristic,
            best_solution: Solution::new(),
            best_score: R64::zero(),
            best_length: Ew::zero(),
            supervisor,
            i: 0,
        };

        swap.initialize();
        swap
    }

    fn single_iteration(&mut self) -> Option<&Solution<IndexType>> {
        // println!("iteration {}", self.i);
        // println!("best solution {}", self.best_solution);
        // for (edge, weight) in self.graph.borrow().iter_edges() {
        //     println!("{:?} with weight {:?}", edge, weight);
        // }
        let start_time = Instant::now();
        if self.expand(start_time) || self.contract(start_time) {
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
                self.best_score,
                &self.best_solution,
            );
            self.i += 1;

            None
        }
    }
}

impl<'a, IndexType, Nw, Ew, W> Iterator for TwoSwap<'a, IndexType, Nw, Ew, W>
where
    IndexType: Distance<IndexType> + Copy + PartialEq + Debug + Hash + Eq + Display + Ord,
    Nw: Copy
        + Debug
        + Add<Output = Nw>
        + Sub<Output = Nw>
        + Serialize
        + Default
        + Zero
        + AddAssign<Nw>
        + PartialEq
        + SmallVal,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::MatrixGraph;
    use crate::metaheuristic::Metaheuristic;
    use csv::Writer;
    use std::io::{Error, Write};
    use std::result::Result;

    struct Blind {}
    impl Write for Blind {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
            Ok(0)
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }
    }

    fn nw(n: R64, _: R64, _: R64, _: R64) -> R64 {
        n
    }

    fn weighted_graph() -> MatrixGraph<usize, R64, R64> {
        MatrixGraph::new_usize_indexed(
            vec![
                R64::from_inner(0.0),
                R64::from_inner(0.8),
                R64::from_inner(12.0),
                R64::from_inner(7.0),
                R64::from_inner(2.5),
            ],
            vec![
                (0, 1, R64::from_inner(12.0)),
                (0, 3, R64::from_inner(2.0)),
                (1, 0, R64::from_inner(7.0)),
                (1, 2, R64::from_inner(16.0)),
                (1, 3, R64::from_inner(1.5)),
                (2, 1, R64::from_inner(13.5)),
                (2, 4, R64::from_inner(23.0)),
                (3, 0, R64::from_inner(8.1)),
                (3, 1, R64::from_inner(27.0)),
                (3, 4, R64::from_inner(7.5)),
                (4, 1, R64::from_inner(7.0)),
                (4, 2, R64::from_inner(12.0)),
                (4, 3, R64::from_inner(7.5)),
            ],
        )
        .unwrap()
    }

    fn blind_supervisor() -> Supervisor<Blind, R64, R64> {
        Supervisor::new(1, Writer::from_writer(Blind {}))
    }

    #[test]
    fn initialization_works() {
        let graph = RefCell::new(weighted_graph());
        let optimizer = TwoSwap::new(
            ProblemInstance::new(&graph, 0, R64::from_inner(100.0)),
            Params::new(&nw),
            blind_supervisor(),
        );
        let solution = optimizer.current_solution();
        let correct = Solution::from_edges(vec![(0, 3), (3, 0)]).unwrap();
        assert_eq!(solution.0, &correct);
        assert_eq!(solution.1, 7.0);
    }

    #[test]
    fn single_iteration_works() {
        let graph = RefCell::new(weighted_graph());
        let mut optimizer = TwoSwap::new(
            ProblemInstance::new(&graph, 0, R64::from_inner(100.0)),
            Params::new(&nw),
            blind_supervisor(),
        );
        let _ = optimizer.single_iteration();
        let solution = optimizer.current_solution();
        let correct = Solution::<usize>::from_edges(vec![(0, 1), (1, 3), (3, 0)]).unwrap();

        assert_eq!(solution.0, &correct);
        assert_eq!(solution.1, 7.8);
    }

    #[test]
    fn solve_works() {
        let graph = RefCell::new(weighted_graph());
        let mut optimizer = TwoSwap::new(
            ProblemInstance::new(&graph, 0, R64::from_inner(100.0)),
            Params::new(&nw),
            blind_supervisor(),
        );
        optimizer.solve();
        let solution = optimizer.current_solution();
        let correct = Solution::<usize>::from_edges(vec![(0, 1), (1, 3), (3, 0)]).unwrap();

        assert_eq!(solution.0, &correct);
        assert_eq!(solution.1, 7.8);
    }
}
