use crate::graph::{Edge, GenericWeightedGraph, GraphError};

use num_traits::identities::Zero;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Sum;

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
    IndexType: PartialEq + Copy + Debug + Display,
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
) -> Result<Nw, GraphError<IndexType>>
where
    IndexType: PartialEq + Copy + Debug + Display,
    Nw: Sum + Copy,
{
    for id in solution.iter_nodes() {
        if let Err(error) = graph.borrow().node_weight(*id) {
            return Err(error);
        }
    }

    Ok(solution
        .iter_nodes()
        .map(|id| *graph.borrow().node_weight(*id).unwrap())
        .sum())
}

pub fn solution_score_and_length<IndexType, Nw, Ew>(
    solution: &Solution<IndexType>,
    graph: &RefCell<
        dyn GenericWeightedGraph<IndexType = IndexType, NodeWeightType = Nw, EdgeWeightType = Ew>,
    >,
) -> Result<(Nw, Ew), GraphError<IndexType>>
where
    IndexType: PartialEq + Copy + Debug + Display,
    Nw: Copy + Zero,
    Ew: Copy + Zero,
{
    for (from, to) in solution.iter_edges() {
        if let Err(error) = graph.borrow().edge_weight((*from, *to)) {
            return Err(error);
        }
        if let Err(error) = graph.borrow().node_weight(*to) {
            return Err(error);
        }
    }

    Ok(solution
        .iter_edges()
        .map(|(from, to)| {
            (
                *graph.borrow().node_weight(*to).unwrap(),
                *graph.borrow().edge_weight((*from, *to)).unwrap(),
            )
        })
        .fold((Nw::zero(), Ew::zero()), |acc, (nw, ew)| {
            (acc.0 + nw, acc.1 + ew)
        }))
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
    IndexType: PartialEq + Copy,
{
    fn default() -> Self {
        Solution::new()
    }
}

impl<IndexType> Solution<IndexType>
where
    IndexType: PartialEq + Copy,
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

    fn node_list() -> Vec<usize> {
        vec![1, 4, 3, 2, 6]
    }

    fn valid_solution() -> Solution<usize> {
        Solution {
            node_list: node_list(),
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
        solution.push_edge((6, 3));

        assert!(solution.iter_nodes().eq(node_list.iter()));
    }

    #[test]
    fn push_edge_errors_on_invalid_from_node() {
        let mut solution = valid_solution();
        let result = solution.push_edge((1, 3));

        assert_eq!(result, Err(SolutionError::InvalidStartingNode(1)));
    }
}
