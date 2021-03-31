use crate::graph::{Edge, GenericWeightedGraph, GraphError};
use crate::metaheuristic::Heuristic;

use decorum::R64;
use num_traits::identities::Zero;
use std::cell::RefCell;
use std::cmp::Eq;
use std::collections::HashSet;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::iter::Sum;
use std::ops::Add;

pub fn solution_length<IndexType, NodeWeightType, EdgeWeightType>(
    solution: &Solution<IndexType>,
    graph: &RefCell<
        dyn GenericWeightedGraph<
            IndexType = IndexType,
            NodeWeightType = NodeWeightType,
            EdgeWeightType = EdgeWeightType,
        >,
    >,
) -> Result<EdgeWeightType, GraphError<IndexType>>
where
    IndexType: PartialEq + Copy + Debug + Display + Hash + Eq,
    EdgeWeightType: Sum + Copy,
{
    for (from, to) in solution.iter_edges() {
        if let Err(error) = graph.borrow().edge_weight((*from, *to)) {
            return Err(error);
        }
    }

    Ok(solution
        .iter_edges()
        .map(|(from, to)| *graph.borrow().edge_weight((*from, *to)).unwrap())
        .sum())
}

pub fn solution_score<IndexType, Nw, Ew>(
    solution: &Solution<IndexType>,
    graph: &RefCell<
        dyn GenericWeightedGraph<IndexType = IndexType, NodeWeightType = Nw, EdgeWeightType = Ew>,
    >,
    heuristic: &Heuristic<IndexType, Nw, Ew>,
) -> Result<R64, GraphError<IndexType>>
where
    IndexType: PartialEq + Copy + Debug + Display + Hash + Eq,
    Nw: Sum + Copy + Debug + Zero + Add<Output = Nw>,
    Ew: Copy + Debug + Zero + Add<Output = Ew>,
{
    let mut visited: HashSet<IndexType> = HashSet::new();
    let mut distance_traveled = Ew::zero();
    let mut sum = R64::zero();
    let g_borrow = graph.borrow();
    for (from, to) in solution.iter_edges() {
        let ew = *g_borrow.edge_weight((*from, *to))?;
        let nw = if !visited.contains(to) {
            *g_borrow.node_weight(*to)?
        } else {
            Nw::zero()
        };

        distance_traveled = ew + distance_traveled;
        sum += heuristic(nw, ew, *to, distance_traveled);
        visited.insert(*to);
    }

    Ok(sum)
}

#[derive(Debug, PartialEq)]
pub enum SolutionError<IndexType: PartialEq> {
    InvalidStartingNode(IndexType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Solution<IndexType> {
    node_list: Vec<IndexType>,
}

impl<IndexType> Default for Solution<IndexType>
where
    IndexType: PartialEq + Copy + Hash + Eq,
{
    fn default() -> Self {
        Solution::new()
    }
}

impl<IndexType> Solution<IndexType>
where
    IndexType: PartialEq + Copy + Hash + Eq,
{
    pub fn new() -> Self {
        Solution {
            node_list: Vec::new(),
        }
    }

    pub fn from_edges(edges: Vec<Edge<IndexType>>) -> Result<Self, SolutionError<IndexType>> {
        let mut solution = Solution::new();
        for edge in edges {
            if let Err(error) = solution.push_edge(edge) {
                return Err(error);
            }
        }

        Ok(solution)
    }

    pub fn from_nodes(nodes: Vec<IndexType>) -> Self {
        Solution { node_list: nodes }
    }

    pub fn push_edge(&mut self, edge: Edge<IndexType>) -> Result<(), SolutionError<IndexType>> {
        // If we are looking at the first node our list will be empty.
        // Thus we need to initialize it with this edge.
        if let Some(last) = self.node_list.last() {
            if last != &edge.0 {
                return Err(SolutionError::InvalidStartingNode(edge.0));
            } else {
                self.node_list.push(edge.1);
            }
        } else {
            self.node_list.push(edge.0);
            self.node_list.push(edge.1);
        }

        Ok(())
    }

    pub fn push_node(&mut self, node: IndexType) {
        self.node_list.push(node);
    }

    pub fn iter_edges(&self) -> Box<dyn Iterator<Item = Edge<&IndexType>> + '_> {
        Box::new(self.node_list.iter().zip(self.node_list.iter().skip(1)))
    }

    pub fn iter_nodes(&self) -> Box<dyn Iterator<Item = &IndexType> + '_> {
        Box::new(self.node_list.iter())
    }

    pub fn edges(&self) -> Vec<Edge<IndexType>> {
        self.iter_edges().map(|x| (*x.0, *x.1)).collect()
    }

    pub fn nodes(&self) -> Vec<IndexType> {
        self.node_list.clone()
    }

    pub fn iter_unique_nodes(&self) -> Box<dyn Iterator<Item = IndexType> + '_> {
        let mut visited = HashSet::new();
        for node in self.node_list.iter() {
            visited.insert(*node);
        }

        Box::new(visited.into_iter())
    }

    pub fn unique_nodes(&self) -> Vec<IndexType> {
        self.iter_unique_nodes().collect()
    }

    pub fn iter_unique_edges(&self) -> Box<dyn Iterator<Item = (&IndexType, &IndexType)> + '_> {
        let mut visited = HashSet::new();
        for edge in self.iter_edges() {
            visited.insert(edge);
        }

        Box::new(visited.into_iter())
    }

    pub fn unique_edges(&self) -> Vec<(&IndexType, &IndexType)> {
        self.iter_unique_edges().collect()
    }

    pub fn reversed(&self) -> Self {
        Self {
            node_list: self.node_list.iter().rev().copied().collect(),
        }
    }

    pub fn reverse(&mut self) {
        self.node_list.reverse();
    }

    pub fn append(&mut self, other: &mut Self) {
        self.node_list.append(&mut other.node_list);
    }
}

impl<IndexType: Display> Display for Solution<IndexType> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.node_list
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(" -> ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::MatrixGraph;
    use decorum::R64;

    fn node_list() -> Vec<usize> {
        vec![1, 4, 3, 2, 6]
    }

    fn node_list_with_duplicates() -> Vec<usize> {
        vec![1, 2, 3, 2, 3, 4, 1]
    }

    fn nw_heuristic<IndexType>(nw: R64, _ew: R64, _id: IndexType, _elapsed: R64) -> R64 {
        nw
    }

    fn weighted_graph() -> MatrixGraph<usize, R64, R64> {
        MatrixGraph::new(
            vec![
                (0, R64::zero()),
                (1, R64::from_inner(2.0)),
                (2, R64::from_inner(5.0)),
                (3, R64::from_inner(1.0)),
                (4, R64::zero()),
                (5, R64::zero()),
                (6, R64::from_inner(10.0)),
            ],
            vec![
                ((1, 2), R64::from_inner(1.0)),
                ((1, 4), R64::from_inner(1.0)),
                ((2, 3), R64::from_inner(1.0)),
                ((3, 2), R64::from_inner(1.0)),
                ((3, 2), R64::from_inner(1.0)),
                ((3, 4), R64::from_inner(1.0)),
                ((4, 1), R64::from_inner(1.0)),
                ((4, 3), R64::from_inner(1.0)),
            ],
        )
        .unwrap()
    }

    fn valid_solution() -> Solution<usize> {
        Solution {
            node_list: node_list(),
        }
    }

    fn valid_solution_with_duplicates() -> Solution<usize> {
        Solution {
            node_list: node_list_with_duplicates(),
        }
    }

    #[test]
    fn iter_nodes_works() {
        let node_list = node_list();
        let solution = valid_solution();

        assert!(solution.iter_nodes().eq(node_list.iter()));
    }

    #[test]
    fn iter_edges_works() {
        let node_list = node_list();
        let edge_it = node_list.iter().zip(node_list.iter().skip(1));
        let solution = valid_solution();

        assert!(solution.iter_edges().eq(edge_it));
    }

    #[test]
    fn nodes_works() {
        let node_list = node_list();
        let solution = valid_solution();

        assert_eq!(solution.nodes(), node_list);
    }

    #[test]
    fn edges_works() {
        let node_list = node_list();
        let edges: Vec<Edge<usize>> = node_list
            .iter()
            .zip(node_list.iter().skip(1))
            .map(|(a, b)| (*a, *b))
            .collect();
        let solution = valid_solution();

        assert_eq!(solution.edges(), edges);
    }

    #[test]
    fn from_edges_works() {
        let node_list = node_list();
        let edges: Vec<Edge<usize>> = node_list
            .iter()
            .zip(node_list.iter().skip(1))
            .map(|(a, b)| (*a, *b))
            .collect();
        let solution = Solution::from_edges(edges.clone()).unwrap();

        assert_eq!(solution.edges(), edges);
    }

    #[test]
    fn from_nodes_works() {
        let list = node_list();
        let solution = Solution::from_nodes(list.clone());

        assert!(solution.iter_nodes().eq(list.iter()));
    }

    #[test]
    fn push_node_works() {
        let mut solution = valid_solution();
        let mut node_list = node_list();
        node_list.push(3);
        solution.push_node(3);

        assert!(solution.iter_nodes().eq(node_list.iter()));
    }

    #[test]
    fn push_edge_works() {
        let mut solution = valid_solution();
        let mut node_list = node_list();
        node_list.push(3);
        let _ = solution.push_edge((6, 3));

        assert!(solution.iter_nodes().eq(node_list.iter()));
    }

    #[test]
    fn push_edge_errors_on_invalid_from_node() {
        let mut solution = valid_solution();
        let result = solution.push_edge((1, 3));

        assert_eq!(result, Err(SolutionError::InvalidStartingNode(1)));
    }

    #[test]
    fn append_works() {
        let mut s1 = valid_solution();
        let mut s2 = Solution::from_nodes(vec![7, 8, 9]);
        s1.append(&mut s2);

        assert_eq!(s1, Solution::from_nodes(vec![1, 4, 3, 2, 6, 7, 8, 9]));
    }

    #[test]
    fn unique_nodes_works() {
        let s1 = valid_solution_with_duplicates();

        assert_eq!(
            s1.unique_nodes().sort_unstable(),
            vec![1, 2, 3, 4].sort_unstable()
        );
    }

    #[test]
    fn solution_score_works() {
        let s1 = valid_solution_with_duplicates();
        let g = weighted_graph();
        let rc = RefCell::new(g);

        assert_eq!(
            solution_score(&s1, &rc, &nw_heuristic).unwrap(),
            R64::from_inner(8.0)
        );
    }
}
