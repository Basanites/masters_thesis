use num_traits::Zero;
use std::cmp::{Eq, Ord, Ordering};
use std::collections::{BinaryHeap, HashMap};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Add;

use crate::graph::{Edge, GenericWeightedGraph, GraphError};
use crate::metaheuristic::Solution;
use crate::util::Max;

#[derive(Debug, Clone)]
pub struct MatrixGraph<IndexType: Clone, Nw, Ew> {
    pub adjacency_matrix: Vec<Vec<Option<Ew>>>,
    node_weights: Vec<Option<Nw>>,
    order: usize,
    size: usize,
    node_map: HashMap<IndexType, usize>,
    inv_node_map: HashMap<usize, IndexType>,
    phantom: PhantomData<IndexType>,
}

/// Implements a weighted, directed graph using an adjacency matrix as datastructure.
#[allow(dead_code)]
impl<Nw: Clone, Ew: Clone> MatrixGraph<usize, Nw, Ew> {
    /// Creates a new Graph instance using the given list of node_weights and weighted edges.
    /// The indices of nodes are inferred from their position in the given array,
    /// meaning the node at nodes[i] will get the index i in the graph instance.
    /// If any of the edges don't fit this scheme an error is returned.
    pub fn new_usize_indexed(
        nodes: Vec<Nw>,
        edges: Vec<(usize, usize, Ew)>,
    ) -> Result<Self, GraphError<usize>> {
        let node_amount = nodes.len();

        let mut graph = MatrixGraph {
            // The initialization of adjacency_matrix makes it necessary, that Ew is of type Clone.
            // If that can be fixed Ew won't need to be Clone.
            adjacency_matrix: (0..node_amount).map(|_| vec![None; node_amount]).collect(),
            node_weights: nodes.into_iter().map(Some).collect(),
            order: node_amount,
            size: edges.len(),
            node_map: HashMap::new(),
            inv_node_map: HashMap::new(),
            phantom: PhantomData,
        };

        for (from, to, weight) in edges.into_iter() {
            if from >= node_amount {
                return Err(GraphError::MissingNode(from));
            } else if to >= node_amount {
                return Err(GraphError::MissingNode(to));
            }

            graph.adjacency_matrix[from][to] = Some(weight);
        }

        Ok(graph)
    }
}

// #[allow(dead_code)]
// impl MatrixGraph<usize, (), ()> {
//     /// Constructs an unweighted MatrixGraph using the given amount of nodes and list of edges.
//     pub fn new_unweighted(nodes: usize, edges: &[Edge<usize>]) -> Result<Self, GraphError<usize>> {
//         // initialization works basically the same way as for generic types.
//         let mut graph = MatrixGraph {
//             adjacency_matrix: (0..nodes).map(|_| vec![None; nodes]).collect(),
//             node_weights: vec![Some(()); nodes],
//             order: nodes,
//             size: edges.len(),
//             node_map: HashMap::new(),
//             inv_node_map: HashMap::new(),
//             phantom: PhantomData,
//         };

//         // A lot of places where to_owned is used. Could possibly be simplified.
//         for (from, to) in edges.iter() {
//             if from >= &nodes {
//                 return Err(GraphError::MissingNode(*from));
//             } else if to >= &nodes {
//                 return Err(GraphError::MissingNode(*to));
//             }

//             graph.adjacency_matrix[*from][*to] = Some(());
//         }

//         Ok(graph)
//     }

//     /// Constructs an empty unweighted MatrixGraph.
//     pub fn default_unweighted() -> Self {
//         MatrixGraph::default()
//     }

//     /// Constructs an unweighted MatrixGraph with capacity for at least the given amount of nodes.
//     pub fn unweighted_with_size(size: usize) -> Self {
//         MatrixGraph::with_size(size)
//     }
// }

impl<IndexType, Nw, Ew> MatrixGraph<IndexType, Nw, Ew>
where
    IndexType: Hash + Copy + Eq + Display + Debug + Ord,
    Nw: Copy,
    Ew: Copy + Max + Zero + Add + Debug + Ord,
{
    pub fn new(
        nodes: Vec<(IndexType, Nw)>,
        edges: Vec<(Edge<IndexType>, Ew)>,
    ) -> Result<Self, GraphError<IndexType>> {
        let mut node_map = HashMap::new();
        let mut inv_node_map = HashMap::new();
        for (i, loc) in nodes.iter().enumerate() {
            node_map.insert(loc.0, i);
            inv_node_map.insert(i, loc.0);
        }

        let mut mapped_edges = Vec::with_capacity(edges.len());
        for (edge, ew) in edges.iter() {
            if !node_map.contains_key(&edge.0) {
                return Err(GraphError::MissingNode(edge.0));
            } else if !node_map.contains_key(&edge.1) {
                return Err(GraphError::MissingNode(edge.1));
            }
            mapped_edges.push((node_map[&edge.0], node_map[&edge.1], *ew))
        }

        let graph =
            MatrixGraph::new_usize_indexed(nodes.iter().map(|x| x.1).collect(), mapped_edges);

        match graph {
            Ok(valid_graph) => Ok(MatrixGraph {
                adjacency_matrix: valid_graph.adjacency_matrix,
                node_weights: valid_graph.node_weights,
                order: valid_graph.order,
                size: valid_graph.size,
                node_map,
                inv_node_map,
                phantom: PhantomData,
            }),
            Err(e) => match e {
                GraphError::MissingNode(node) => Err(GraphError::MissingNode(inv_node_map[&node])),
                GraphError::MissingEdge(edge) => Err(GraphError::MissingEdge((
                    inv_node_map[&edge.0],
                    inv_node_map[&edge.1],
                ))),
                GraphError::DuplicateNode(node) => {
                    Err(GraphError::DuplicateNode(inv_node_map[&node]))
                }
                GraphError::DuplicateEdge(edge) => Err(GraphError::DuplicateEdge((
                    inv_node_map[&edge.0],
                    inv_node_map[&edge.1],
                ))),
            },
        }
    }

    #[allow(dead_code)]
    fn cast_usize_to_generic_graph(
        ugraph: MatrixGraph<usize, Nw, Ew>,
        nmap: HashMap<IndexType, usize>,
        imap: HashMap<usize, IndexType>,
    ) -> MatrixGraph<IndexType, Nw, Ew> {
        MatrixGraph {
            adjacency_matrix: ugraph.adjacency_matrix,
            node_weights: ugraph.node_weights,
            order: ugraph.order,
            size: ugraph.size,
            node_map: nmap,
            inv_node_map: imap,
            phantom: PhantomData,
        }
    }

    /// Default constructor for an empty MatrixGraph.
    /// If the amount of nodes is known beforehand use either MatrixGraph::new()
    /// or MatrixGraph::with_size(), as they don't require resizing later, wich is slow.
    pub fn default() -> Self {
        MatrixGraph {
            adjacency_matrix: Vec::new(),
            node_weights: Vec::new(),
            order: 0,
            size: 0,
            node_map: HashMap::new(),
            inv_node_map: HashMap::new(),
            phantom: PhantomData,
        }
    }

    /// Constructs an empty MatrixGraph with capacity for at least the given amount of nodes.
    pub fn with_size(size: usize) -> Self {
        MatrixGraph {
            adjacency_matrix: (0..size).map(|_| vec![None; size]).collect(),
            node_weights: vec![None; size],
            order: 0,
            size: 0,
            node_map: HashMap::with_capacity(size),
            inv_node_map: HashMap::with_capacity(size),
            phantom: PhantomData,
        }
    }

    fn mapped_result<CorrectType>(
        &self,
        result: Result<CorrectType, GraphError<usize>>,
    ) -> Result<CorrectType, GraphError<IndexType>> {
        match result {
            Err(GraphError::MissingNode(node)) => {
                Err(GraphError::MissingNode(self.inv_node_map[&node]))
            }
            Err(GraphError::MissingEdge(edge)) => Err(GraphError::MissingEdge((
                self.inv_node_map[&edge.0],
                self.inv_node_map[&edge.1],
            ))),
            Err(GraphError::DuplicateNode(node)) => {
                Err(GraphError::DuplicateNode(self.inv_node_map[&node]))
            }
            Err(GraphError::DuplicateEdge(edge)) => Err(GraphError::DuplicateEdge((
                self.inv_node_map[&edge.0],
                self.inv_node_map[&edge.1],
            ))),
            Ok(some) => Ok(some),
        }
    }

    fn _is_empty(&self) -> bool {
        self.node_weights.is_empty()
    }

    fn _order(&self) -> usize {
        self.order
    }

    fn _size(&self) -> usize {
        self.size
    }

    fn _iter_node_ids(&self) -> Box<dyn Iterator<Item = usize> + '_> {
        Box::new(
            self.node_weights
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(move |(i, _)| i),
        )
    }

    fn _iter_nodes(&self) -> Box<dyn Iterator<Item = (usize, &Nw)> + '_> {
        Box::new(
            self.node_weights
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(move |(i, _)| (i, self._node_weight(i).unwrap())),
        )
    }

    fn _node_weight(&self, id: usize) -> Result<&Nw, GraphError<usize>> {
        if !self._has_node(id) {
            return Err(GraphError::MissingNode(id));
        }

        // Unwrapping is ok here, because has_node ensures, that there is a weight in the array at position id.
        Ok(self.node_weights[id].as_ref().unwrap())
    }

    fn _iter_neighbor_ids(
        &self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = usize> + '_>, GraphError<usize>> {
        if !self._has_node(id) {
            return Err(GraphError::MissingNode(id));
        }

        // Get the ids of nodes to which a weighted edge from id exists.
        Ok(Box::new(
            self.adjacency_matrix[id]
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(move |(i, _)| i),
        ))
    }

    fn _neighbor_ids(&self, id: usize) -> Result<Vec<usize>, GraphError<usize>> {
        match self._iter_neighbor_ids(id) {
            Ok(iterator) => Ok(iterator.collect()),
            Err(error) => Err(error),
        }
    }

    #[allow(clippy::type_complexity)]
    fn _iter_neighbors(
        &self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = (usize, &Ew)> + '_>, GraphError<usize>> {
        if !self._has_node(id) {
            return Err(GraphError::MissingNode(id));
        }

        // Get the ids of nodes to which a weighted edge from id exists.
        Ok(Box::new(
            self.adjacency_matrix[id]
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(move |(i, _)| (i, self._edge_weight((id, i)).unwrap())),
        ))
    }

    fn _iter_inv_neighbors(
        &self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = (usize, &Ew)> + '_>, GraphError<usize>> {
        if !self._has_node(id) {
            return Err(GraphError::MissingNode(id));
        }

        // Get the ids of nodes to which a weighted edge from id exists.
        Ok(Box::new(
            self.adjacency_matrix
                .iter()
                .map(move |col| col[id])
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(move |(i, _)| (i, self._edge_weight((i, id)).unwrap())),
        ))
    }

    fn _has_node(&self, id: usize) -> bool {
        self.node_weights.len() > id && self.node_weights[id].is_some()
    }

    fn _add_node(&mut self, id: usize, weight: Nw) -> Result<(), GraphError<usize>> {
        if self.node_weights.len() > id && self._has_node(id) {
            return Err(GraphError::DuplicateNode(id));
        } else if self.node_weights.len() <= id {
            // Resizing here will never shrink the array, because has_node() implies id >= node_weights.len().
            // However calling this every time is slower than checking if the array needs to be resized.
            // Possible empty spots in between will be initialized with None.
            self.node_weights.resize_with(id + 2, || None);
            self.adjacency_matrix.resize_with(id + 2, || vec![None; id]);
            for edge_weights in self.adjacency_matrix.iter_mut() {
                edge_weights.resize_with(id + 2, || None);
            }
        }

        self.node_weights[id] = Some(weight);
        // Adding a node increases order by one.
        self.order += 1;
        Ok(())
    }

    fn _remove_node(&mut self, id: usize) {
        if self._has_node(id) {
            // If a node is removed from the graph there can't be any edges to or from it.
            for i in 0..self.order {
                self._remove_edge((i, id));
                self._remove_edge((id, i));
            }

            self.node_weights[id] = None;
            // Removing the node reduces order by one.
            self.order -= 1;
        }
    }

    fn _change_node(&mut self, id: usize, weight: Nw) {
        if self._has_node(id) {
            self.node_weights[id] = Some(weight);
        } else {
            // Unwrapping is ok here, because we ensured, we don't have this node id yet.
            self._add_node(id, weight).unwrap();
        }
    }

    fn _degree(&self, id: usize) -> Result<usize, GraphError<usize>> {
        // GraphError can be thrown if the node with id is not in the graph.
        Ok(self._neighbor_ids(id)?.len())
    }

    fn _iter_edge_ids(&self) -> Box<dyn Iterator<Item = Edge<usize>> + '_> {
        Box::new(
            self.adjacency_matrix
                .iter()
                .enumerate()
                .flat_map(|(i, edges)| {
                    edges
                        .iter()
                        .enumerate()
                        .filter(|(_, weight)| weight.is_some())
                        .map(move |(j, _)| (i, j))
                }),
        )
    }

    fn _edge_ids(&self) -> Vec<Edge<usize>> {
        self._iter_edge_ids().collect()
    }

    fn _iter_edges(&self) -> Box<dyn Iterator<Item = (Edge<usize>, &Ew)> + '_> {
        Box::new(
            self.adjacency_matrix
                .iter()
                .enumerate()
                .flat_map(move |(i, edges)| {
                    edges
                        .iter()
                        .enumerate()
                        .filter(|(_, weight)| weight.is_some())
                        .map(move |(j, _)| ((i, j), self._edge_weight((i, j)).unwrap()))
                }),
        )
    }

    fn _edge_weight(&self, edge: Edge<usize>) -> Result<&Ew, GraphError<usize>> {
        let (start_node, end_node) = edge;
        if !self._has_edge(edge) {
            return Err(GraphError::MissingEdge(edge));
        }

        Ok(&self.adjacency_matrix[start_node][end_node]
            .as_ref()
            .unwrap())
    }

    fn _has_edge(&self, edge: Edge<usize>) -> bool {
        let (start_node, end_node) = edge;
        if !self._has_node(start_node) || !self._has_node(end_node) {
            return false;
        }

        self.adjacency_matrix[start_node][end_node].is_some()
    }

    fn _add_edge(&mut self, edge: Edge<usize>, weight: Ew) -> Result<(), GraphError<usize>> {
        let (start_node, end_node) = edge;
        if self._has_edge(edge) {
            return Err(GraphError::DuplicateEdge(edge));
        } else if !self._has_node(start_node) {
            return Err(GraphError::MissingNode(start_node));
        } else if !self._has_node(end_node) {
            return Err(GraphError::MissingNode(end_node));
        }

        self.adjacency_matrix[start_node][end_node] = Some(weight);

        // Adding an edge increases size by one.
        self.size += 1;
        Ok(())
    }

    fn _remove_edge(&mut self, edge: Edge<usize>) {
        if self._has_edge(edge) {
            self.adjacency_matrix[edge.0][edge.1] = None;
            // Removing an edge reduces size by one.
            self.size -= 1;
        }
    }

    fn _change_edge(&mut self, edge: Edge<usize>, weight: Ew) -> Result<(), GraphError<usize>> {
        if self._has_edge(edge) {
            self.adjacency_matrix[edge.0][edge.1] = Some(weight);
            Ok(())
        } else {
            self._add_edge(edge, weight)
        }
    }

    fn _inv_shortest_paths(&self, from_node: usize) -> (Vec<Option<usize>>, Vec<Ew>) {
        // dist[node] = current shortest distance from `start` to `node`
        let node_count = self.adjacency_matrix.len();
        let mut dist: Vec<_> = (0..node_count).map(|_| <Ew as Max>::max()).collect();

        let mut prev: Vec<Option<usize>> = vec![None; node_count];
        let mut visited: Vec<bool> = vec![false; node_count];

        let mut heap: BinaryHeap<State<usize, Ew>> = BinaryHeap::with_capacity(node_count);

        // We're at `start`, with a zero cost
        dist[from_node] = Ew::zero();
        prev[from_node] = Some(from_node);
        heap.push(State {
            cost: Ew::zero(),
            position: from_node,
        });

        // Examine the frontier with lower cost nodes first (min-heap)
        while let Some(State { cost, position }) = heap.pop() {
            // Important as we may have already found a better way
            if visited[position] || cost > dist[position] {
                continue;
            }

            visited[position] = true;

            // For each node we can reach, see if we can find a way with
            // a lower cost going through this node
            for (other, &cost_from) in self._iter_inv_neighbors(position).unwrap() {
                let next = State {
                    cost: cost + cost_from,
                    position: other,
                };

                // If so, add it to the frontier and continue
                if next.cost < dist[other] {
                    if !visited[other] {
                        heap.push(next);
                    }
                    // Relaxation, we have now found a better way
                    dist[other] = next.cost;
                    prev[other] = Some(position);
                }
            }
        }

        (prev, dist)
    }

    fn _shortest_paths(&self, from_node: usize) -> (Vec<Option<usize>>, Vec<Ew>) {
        // dist[node] = current shortest distance from `start` to `node`
        let node_count = self.adjacency_matrix.len();
        let mut dist: Vec<_> = (0..node_count).map(|_| <Ew as Max>::max()).collect();

        let mut prev: Vec<Option<usize>> = vec![None; node_count];
        let mut visited: Vec<bool> = vec![false; node_count];

        let mut heap: BinaryHeap<State<usize, Ew>> = BinaryHeap::with_capacity(node_count);

        // We're at `start`, with a zero cost
        dist[from_node] = Ew::zero();
        prev[from_node] = Some(from_node);
        heap.push(State {
            cost: Ew::zero(),
            position: from_node,
        });

        // Examine the frontier with lower cost nodes first (min-heap)
        while let Some(State { cost, position }) = heap.pop() {
            // Important as we may have already found a better way
            if visited[position] || cost > dist[position] {
                continue;
            }

            visited[position] = true;

            // For each node we can reach, see if we can find a way with
            // a lower cost going through this node
            for (other, &cost_to) in self._iter_neighbors(position).unwrap() {
                let next = State {
                    cost: cost + cost_to,
                    position: other,
                };

                // If so, add it to the frontier and continue
                if next.cost < dist[other] {
                    if !visited[other] {
                        heap.push(next);
                    }
                    // Relaxation, we have now found a better way
                    dist[other] = next.cost;
                    prev[other] = Some(position);
                }
            }
        }

        (prev, dist)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State<IndexType, CostType> {
    cost: CostType,
    position: IndexType,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl<IndexType: Ord, CostType: Ord> Ord for State<IndexType, CostType> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl<IndexType: Ord, CostType: Ord> PartialOrd for State<IndexType, CostType> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(dead_code, clippy::map_entry)]
impl<IndexType, Nw, Ew> GenericWeightedGraph for MatrixGraph<IndexType, Nw, Ew>
where
    IndexType: Hash + Copy + Eq + Display + Debug + Ord,
    Nw: Copy,
    Ew: Copy + Max + Zero + Add + Debug + Ord,
{
    type IndexType = IndexType;
    type NodeWeightType = Nw;
    type EdgeWeightType = Ew;

    default fn is_empty(&self) -> bool {
        self._is_empty()
    }

    default fn order(&self) -> usize {
        self._order()
    }

    default fn size(&self) -> usize {
        self._size()
    }

    default fn iter_node_ids(&self) -> Box<dyn Iterator<Item = IndexType> + '_> {
        Box::new(self.node_map.keys().copied())
    }

    default fn node_ids(&self) -> Vec<Self::IndexType> {
        self.iter_node_ids().collect()
    }

    default fn iter_nodes(
        &self,
    ) -> Box<dyn Iterator<Item = (Self::IndexType, &Self::NodeWeightType)> + '_> {
        Box::new(
            self._iter_nodes()
                .map(move |(id, node)| (self.inv_node_map[&id], node)),
        )
    }

    default fn node_weight(
        &self,
        id: Self::IndexType,
    ) -> Result<&Self::NodeWeightType, GraphError<Self::IndexType>> {
        if !self.node_map.contains_key(&id) {
            return Err(GraphError::MissingNode(id));
        }

        let weight = self._node_weight(self.node_map[&id]);
        self.mapped_result(weight)
    }

    default fn iter_neighbor_ids(
        &self,
        id: Self::IndexType,
    ) -> Result<Box<dyn Iterator<Item = Self::IndexType> + '_>, GraphError<Self::IndexType>> {
        if !self.node_map.contains_key(&id) {
            return Err(GraphError::MissingNode(id));
        }

        let inner = self._iter_neighbor_ids(self.node_map[&id]);
        let res = self.mapped_result(inner);
        match res {
            Ok(iterator) => Ok(Box::new(iterator.map(move |id| self.inv_node_map[&id]))),
            Err(e) => Err(e),
        }
    }

    default fn neighbor_ids(
        &self,
        id: Self::IndexType,
    ) -> Result<Vec<Self::IndexType>, GraphError<Self::IndexType>> {
        let res = self.iter_neighbor_ids(id);
        match res {
            Ok(iterator) => Ok(iterator.collect()),
            Err(e) => Err(e),
        }
    }

    #[allow(clippy::type_complexity)]
    default fn iter_neighbors(
        &self,
        id: Self::IndexType,
    ) -> Result<
        Box<dyn Iterator<Item = (Self::IndexType, &Self::EdgeWeightType)> + '_>,
        GraphError<Self::IndexType>,
    > {
        let inner = self._iter_neighbors(self.node_map[&id]);
        let res = self.mapped_result(inner);
        match res {
            Ok(iterator) => Ok(Box::new(
                iterator.map(move |(id, point)| (self.inv_node_map[&id], point)),
            )),
            Err(e) => Err(e),
        }
    }

    default fn neighbors(
        &self,
        id: Self::IndexType,
    ) -> Result<Vec<(Self::IndexType, &Self::EdgeWeightType)>, GraphError<Self::IndexType>> {
        let res = self.iter_neighbors(id);
        match res {
            Ok(iter) => Ok(iter.collect()),
            Err(e) => Err(e),
        }
    }

    default fn has_node(&self, id: Self::IndexType) -> bool {
        self.node_map.contains_key(&id) && self._has_node(self.node_map[&id])
    }

    default fn add_node(
        &mut self,
        id: Self::IndexType,
        weight: Self::NodeWeightType,
    ) -> Result<(), GraphError<Self::IndexType>> {
        if self.node_map.contains_key(&id) {
            return Err(GraphError::DuplicateNode(id));
        }

        // order is always amount of nodes + 1, so we can use it as our new id for internal
        let inner_id = self.order();
        let res = self._add_node(inner_id, weight);
        let mapped_res = self.mapped_result(res);
        match mapped_res {
            Ok(_) => {
                self.node_map.insert(id, inner_id);
                self.inv_node_map.insert(inner_id, id);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    default fn remove_node(&mut self, id: Self::IndexType) {
        if let Some(&inner_id) = self.node_map.get(&id) {
            self.node_map.remove(&id);
            self.inv_node_map.remove(&inner_id);
            self._remove_node(inner_id);
        }
    }

    default fn change_node(&mut self, id: Self::IndexType, weight: Self::NodeWeightType) {
        if self.node_map.contains_key(&id) {
            self._change_node(self.node_map[&id], weight);
        } else {
            let inner_id = self.order();
            self.node_map.insert(id, inner_id);
            self.inv_node_map.insert(inner_id, id);
        }
    }

    default fn degree(&self, id: Self::IndexType) -> Result<usize, GraphError<Self::IndexType>> {
        if !self.node_map.contains_key(&id) {
            return Err(GraphError::MissingNode(id));
        }

        let degree = self._degree(self.node_map[&id]);
        self.mapped_result(degree)
    }

    default fn iter_edge_ids(&self) -> Box<dyn Iterator<Item = Edge<Self::IndexType>> + '_> {
        Box::new(
            self._iter_edge_ids()
                .map(move |(f_id, t_id)| (self.inv_node_map[&f_id], self.inv_node_map[&t_id])),
        )
    }

    default fn edge_ids(&self) -> Vec<Edge<Self::IndexType>> {
        self.iter_edge_ids().collect()
    }

    default fn iter_edges(
        &self,
    ) -> Box<dyn Iterator<Item = (Edge<Self::IndexType>, &Self::EdgeWeightType)> + '_> {
        Box::new(self._iter_edges().map(move |((f_id, t_id), weight)| {
            ((self.inv_node_map[&f_id], self.inv_node_map[&t_id]), weight)
        }))
    }

    default fn edges(&self) -> Vec<(Edge<Self::IndexType>, &Self::EdgeWeightType)> {
        self.iter_edges().collect()
    }

    default fn edge_weight(
        &self,
        edge: Edge<Self::IndexType>,
    ) -> Result<&Self::EdgeWeightType, GraphError<Self::IndexType>> {
        let weight = self._edge_weight((self.node_map[&edge.0], self.node_map[&edge.1]));
        self.mapped_result(weight)
    }

    default fn has_edge(&self, edge: Edge<Self::IndexType>) -> bool {
        self._has_edge((self.node_map[&edge.0], self.node_map[&edge.1]))
    }

    default fn add_edge(
        &mut self,
        edge: Edge<Self::IndexType>,
        weight: Ew,
    ) -> Result<(), GraphError<Self::IndexType>> {
        if !self.node_map.contains_key(&edge.0) {
            return Err(GraphError::MissingNode(edge.0));
        } else if !self.node_map.contains_key(&edge.1) {
            return Err(GraphError::MissingNode(edge.1));
        }
        let edge = self._add_edge((self.node_map[&edge.0], self.node_map[&edge.1]), weight);
        self.mapped_result(edge)
    }

    default fn remove_edge(&mut self, edge: Edge<Self::IndexType>) {
        self._remove_edge((self.node_map[&edge.0], self.node_map[&edge.1]));
    }

    default fn change_edge(
        &mut self,
        edge: Edge<Self::IndexType>,
        weight: Self::EdgeWeightType,
    ) -> Result<(), GraphError<Self::IndexType>> {
        let edge = self._change_edge((self.node_map[&edge.0], self.node_map[&edge.1]), weight);
        self.mapped_result(edge)
    }

    default fn shortest_paths(
        &self,
        from_node: Self::IndexType,
    ) -> HashMap<Self::IndexType, Option<(Solution<Self::IndexType>, Ew)>> {
        let (prevs, dists) = self._shortest_paths(self.node_map[&from_node]);
        let mut res = HashMap::new();

        for i in 0..prevs.len() {
            let mut created = false;
            let mut solution: Solution<Self::IndexType> =
                Solution::from_nodes(vec![self.inv_node_map[&i]]);
            let mut prev = prevs[i];
            while let Some(node) = prev {
                created = true;

                // only push self once (probably only important for to_node)
                if i != node {
                    solution.push_node(self.inv_node_map[&node]);
                }

                let n_prev = prevs[node];
                // only to_node has prev[node] == node so we iterate as long as we haven't reached that
                if n_prev != prev {
                    prev = n_prev
                } else {
                    prev = None
                }
            }

            // the solution needs to be reversed for us to start at the from_node
            solution.reverse();

            if created {
                res.insert(self.inv_node_map[&i], Some((solution, dists[i])));
            } else {
                res.insert(self.inv_node_map[&i], None);
            }
        }

        res
    }

    default fn inv_shortest_paths(
        &self,
        to_node: Self::IndexType,
    ) -> HashMap<Self::IndexType, Option<(Solution<Self::IndexType>, Ew)>> {
        let (prevs, dists) = self._inv_shortest_paths(self.node_map[&to_node]);
        let mut res = HashMap::new();

        for i in 0..prevs.len() {
            let mut created = false;
            let mut solution: Solution<Self::IndexType> =
                Solution::from_nodes(vec![self.inv_node_map[&i]]);
            let mut prev = prevs[i];
            while let Some(node) = prev {
                created = true;

                // only push self once (probably only important for to_node)
                if i != node {
                    solution.push_node(self.inv_node_map[&node]);
                }

                let n_prev = prevs[node];
                // only to_node has prev[node] == node so we iterate as long as we haven't reached that
                if n_prev != prev {
                    prev = n_prev
                } else {
                    prev = None
                }
            }
            if created {
                res.insert(self.inv_node_map[&i], Some((solution, dists[i])));
            } else {
                res.insert(self.inv_node_map[&i], None);
            }
        }

        res
    }
}

impl<Nw: Copy, Ew> GenericWeightedGraph for MatrixGraph<usize, Nw, Ew>
where
    Nw: Copy,
    Ew: Copy + Max + Zero + Add + Debug + Ord,
{
    fn iter_node_ids(&self) -> Box<dyn Iterator<Item = usize> + '_> {
        self._iter_node_ids()
    }
    fn iter_nodes(&self) -> Box<dyn Iterator<Item = (usize, &Nw)> + '_> {
        self._iter_nodes()
    }

    fn node_weight(&self, id: usize) -> Result<&Nw, GraphError<usize>> {
        self._node_weight(id)
    }

    fn iter_neighbor_ids(
        &self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = usize> + '_>, GraphError<usize>> {
        self._iter_neighbor_ids(id)
    }

    #[allow(clippy::type_complexity)]
    fn iter_neighbors(
        &self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = (usize, &Ew)> + '_>, GraphError<usize>> {
        self._iter_neighbors(id)
    }

    fn has_node(&self, id: usize) -> bool {
        self._has_node(id)
    }

    fn add_node(&mut self, id: usize, weight: Nw) -> Result<(), GraphError<usize>> {
        self._add_node(id, weight)
    }

    fn remove_node(&mut self, id: usize) {
        self._remove_node(id)
    }

    fn change_node(&mut self, id: usize, weight: Nw) {
        self._change_node(id, weight)
    }

    fn degree(&self, id: usize) -> Result<usize, GraphError<usize>> {
        self._degree(id)
    }

    fn iter_edge_ids(&self) -> Box<dyn Iterator<Item = Edge<usize>> + '_> {
        self._iter_edge_ids()
    }

    fn iter_edges(&self) -> Box<dyn Iterator<Item = (Edge<usize>, &Ew)> + '_> {
        self._iter_edges()
    }

    fn edge_weight(&self, edge: Edge<usize>) -> Result<&Ew, GraphError<usize>> {
        self._edge_weight(edge)
    }

    fn has_edge(&self, edge: Edge<usize>) -> bool {
        self._has_edge(edge)
    }

    fn add_edge(&mut self, edge: Edge<usize>, weight: Ew) -> Result<(), GraphError<usize>> {
        self._add_edge(edge, weight)
    }

    fn remove_edge(&mut self, edge: Edge<usize>) {
        self._remove_edge(edge)
    }

    fn change_edge(&mut self, edge: Edge<usize>, weight: Ew) -> Result<(), GraphError<usize>> {
        self._change_edge(edge, weight)
    }

    fn shortest_paths(
        &self,
        from_node: usize,
    ) -> HashMap<Self::IndexType, Option<(Solution<Self::IndexType>, Ew)>> {
        let (prevs, dists) = self._shortest_paths(from_node);
        let mut res = HashMap::new();

        for i in 0..prevs.len() {
            let mut created = false;
            let mut solution: Solution<Self::IndexType> = Solution::from_nodes(vec![i]);
            let mut prev = prevs[i];
            while let Some(node) = prev {
                created = true;

                // only push self once (probably only important for to_node)
                if i != node {
                    solution.push_node(node);
                }

                let n_prev = prevs[node];
                // only to_node has prev[node] == node so we iterate as long as we haven't reached that
                if n_prev != prev {
                    prev = n_prev
                } else {
                    prev = None
                }
            }

            // the solution needs to be reversed for us to start at the from_node
            solution.reverse();

            // if we can reach the node we created a solution
            if created {
                res.insert(i, Some((solution, dists[i])));
            } else {
                res.insert(i, None);
            }
        }

        res
    }

    fn inv_shortest_paths(
        &self,
        to_node: usize,
    ) -> HashMap<Self::IndexType, Option<(Solution<Self::IndexType>, Ew)>> {
        let (prevs, dists) = self._inv_shortest_paths(to_node);
        let mut res = HashMap::new();

        for i in 0..prevs.len() {
            let mut created = false;
            let mut solution: Solution<Self::IndexType> = Solution::from_nodes(vec![i]);
            let mut prev = prevs[i];
            while let Some(node) = prev {
                created = true;

                // only push self once (probably only important for to_node)
                if i != node {
                    solution.push_node(node);
                }

                let n_prev = prevs[node];
                // only to_node has prev[node] == node so we iterate as long as we haven't reached that
                if n_prev != prev {
                    prev = n_prev
                } else {
                    prev = None
                }
            }
            // if we can reach the node we created a solution
            if created {
                res.insert(i, Some((solution, dists[i])));
            } else {
                res.insert(i, None);
            }
        }

        res
    }
}

// impl<Nw: Copy, Ew: Copy> WeightedGraph for MatrixGraph<usize, Nw, Ew> {}

// impl<Nw: Copy, Ew: Copy> GeoGraph for MatrixGraph<GeoPoint, Nw, Ew> {}

#[cfg(test)]
mod usize_indexed_tests {
    use super::*;
    use crate::graph::GenericWeightedGraph;
    use test::Bencher;
    extern crate test;

    fn valid_weighted() -> MatrixGraph<usize, usize, usize> {
        MatrixGraph::new_usize_indexed(
            vec![1, 2, 3],
            vec![(0, 1, 100), (1, 2, 101), (2, 1, 50), (2, 0, 200)],
        )
        .unwrap()
    }

    fn inv_valid_weighted() -> MatrixGraph<usize, usize, usize> {
        MatrixGraph::new_usize_indexed(
            vec![1, 2, 3],
            vec![(1, 0, 100), (2, 1, 101), (1, 2, 50), (0, 2, 200)],
        )
        .unwrap()
    }

    fn valid_vector_weighted() -> MatrixGraph<usize, Vec<usize>, Vec<usize>> {
        MatrixGraph::new_usize_indexed(
            vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
            vec![
                (0, 1, vec![100, 200, 300]),
                (1, 2, vec![101, 201, 301]),
                (2, 1, vec![50, 150, 250]),
                (2, 0, vec![200, 300, 400]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn new_empty_weighted_works() {
        let graph = MatrixGraph::<usize, usize, usize>::default();

        assert_eq!(
            graph.node_weights,
            Vec::new(),
            "Node weight array is not empty."
        );
        assert_eq!(
            graph.adjacency_matrix,
            Vec::<Vec<Option<usize>>>::new(),
            "Adjacency matrix is not empty."
        );
        assert!(graph.is_empty(), "Graph is not empty.");
    }

    #[test]
    fn new_weighted_with_size_works() {
        let graph = MatrixGraph::<usize, usize, usize>::with_size(5);

        assert!(
            graph.node_weights.capacity() >= 5,
            "Not enough space given for nodes."
        );
        for weight in graph.node_weights.iter() {
            assert!(
                weight.is_none(),
                "Node weight was initialized with a value."
            );
        }
        for edges in graph.adjacency_matrix.iter() {
            for edge in edges.iter() {
                assert!(edge.is_none(), "Edge weight was initialized with a value.");
            }
        }
    }

    #[test]
    fn new_weighted_from_lists_works() {
        let graph = valid_weighted();

        assert!(
            graph.node_weights.capacity() >= 3,
            "Array was created too small."
        );
        assert_eq!(
            graph.node_weights,
            vec![Some(1), Some(2), Some(3)],
            "Node weights are wrong."
        );
        assert_eq!(
            graph.adjacency_matrix,
            vec![
                vec![None, Some(100), None],
                vec![None, None, Some(101)],
                vec![Some(200), Some(50), None]
            ],
            "Edge weights are wrong."
        );
    }

    #[test]
    fn new_vector_weighted_from_lists_works() {
        let graph = valid_vector_weighted();

        assert!(
            graph.node_weights.capacity() >= 3,
            "Array is to small to hold all nodes."
        );
        assert_eq!(
            graph.node_weights,
            vec![
                Some(vec![1, 2, 3]),
                Some(vec![4, 5, 6]),
                Some(vec![7, 8, 9])
            ],
            "Node weights are wrong."
        );
        assert_eq!(
            graph.adjacency_matrix,
            vec![
                vec![None, Some(vec![100, 200, 300]), None],
                vec![None, None, Some(vec![101, 201, 301])],
                vec![Some(vec![200, 300, 400]), Some(vec![50, 150, 250]), None]
            ],
            "Edge weights are wrong."
        );
    }

    #[test]
    fn new_with_missing_from_node_errors() {
        let err = MatrixGraph::new_usize_indexed(vec![1], vec![(1, 0, 1)]).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(1)),
            "Not missing the node it should be missing."
        );
    }

    #[test]
    fn new_with_missing_to_node_errors() {
        let err = MatrixGraph::new_usize_indexed(vec![1], vec![(0, 1, 1)]).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(1)),
            "Not missing the node it should be missing."
        );
    }

    #[test]
    fn is_empty_works() {
        let not_empty = valid_weighted();
        let empty = MatrixGraph::<usize, usize, usize>::default();

        assert!(!not_empty.is_empty(), "Graph should not be empty.");
        assert!(empty.is_empty(), "Graph should be empty.");
    }

    #[test]
    fn order_works() {
        let not_empty = valid_weighted();
        let empty = MatrixGraph::<usize, usize, usize>::default();

        assert_eq!(
            not_empty.order(),
            3,
            "Graph should have three nodes, thus order 3."
        );
        assert_eq!(
            empty.order(),
            0,
            "Graph should be empty, thus have order 0."
        );
    }

    #[test]
    fn size_works() {
        let not_empty = valid_weighted();
        let empty = MatrixGraph::<usize, usize, usize>::default();

        assert_eq!(
            not_empty.size(),
            4,
            "Graph should have four edges, thus size 4."
        );
        assert_eq!(empty.size(), 0, "Graph should be empty, thus have order 0.");
    }

    #[test]
    fn nodes_works() {
        let mut graph = valid_weighted();
        let empty = MatrixGraph::<usize, usize, usize>::default();

        assert_eq!(
            graph.node_ids(),
            vec![0, 1, 2],
            "Nodes are not the ones used to construct."
        );
        assert_eq!(
            empty.node_ids(),
            Vec::<usize>::new(),
            "Nodes should be empty, since graph is empty."
        );

        graph.add_node(4, 5).unwrap();

        assert_eq!(
            graph.node_ids(),
            vec![0, 1, 2, 4],
            "The node 4 should be in, since it was just added."
        );
    }

    #[test]
    fn node_weight_works() {
        let graph = valid_weighted();

        assert_eq!(graph.node_weight(1).unwrap(), &2, "Node has wrong weight.");
    }

    #[test]
    fn node_weight_errors_for_missing() {
        let graph = valid_weighted();
        let err = graph.node_weight(5).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(5)),
            "The node 5 should not be in the graph"
        );
    }

    #[test]
    fn neighbors_works() {
        let mut graph = valid_weighted();
        graph.add_node(5, 0).unwrap();

        assert_eq!(
            graph.neighbor_ids(1).unwrap(),
            vec![2],
            "Node 1 should only have 2 as neighbor."
        );
        assert_eq!(
            graph.neighbor_ids(2).unwrap().sort_unstable(),
            vec![1, 0].sort_unstable(),
            "Node 2 should have 1 and 0 as neighbor."
        );
        assert_eq!(
            graph.neighbor_ids(5).unwrap(),
            Vec::<usize>::new(),
            "Node 5 was just inserted and has no neighbors."
        );
    }

    #[test]
    fn neighbors_errors_for_missing() {
        let graph = valid_weighted();
        let err = graph.neighbor_ids(5).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(5)),
            "The node 5 should not be in the graph, thus has no neighbors."
        );
    }

    #[test]
    fn has_node_works() {
        let graph = valid_weighted();

        assert!(graph.has_node(1));
        assert!(!graph.has_node(5));
    }

    #[test]
    fn adding_nodes_works() {
        let mut graph = valid_weighted();
        graph.add_node(4, 5).unwrap();

        assert!(graph.has_node(4), "Node was not added.");
        assert!(!graph.has_node(3), "Incorrect node was added.");
        assert_eq!(
            graph.order(),
            4,
            "Order was not updated correctly after insertion."
        );
        assert_eq!(
            graph.size(),
            4,
            "Size should not change during node insertion."
        );
        assert_eq!(
            graph.node_ids().sort_unstable(),
            vec![0, 1, 2, 4].sort_unstable(),
            "Node list was not updated correctly after insertion."
        );
        assert_eq!(
            graph.node_weight(4).unwrap(),
            &5,
            "Incorrect weight was applied to new node."
        );
        assert!(
            graph.node_weights.len() >= 4,
            "Not enough space for all possible node weights."
        );
        assert!(
            graph.adjacency_matrix.len() >= 4,
            "Not enough space for all possible edge weights."
        );
        for edges in graph.adjacency_matrix.iter() {
            assert!(
                edges.len() >= 4,
                "Not enough space for all possible edge weights."
            );
        }
    }

    #[test]
    fn adding_duplicate_node_errors() {
        let mut graph = valid_weighted();
        let err = graph.add_node(1, 1).err();

        assert_eq!(
            err,
            Some(GraphError::DuplicateNode(1)),
            "The node should be a duplicate."
        );
    }

    #[test]
    fn removing_nodes_works() {
        let mut graph = valid_weighted();
        graph.remove_node(1);

        assert!(!graph.has_node(1), "Has removed node.");
        assert_eq!(
            graph.order(),
            2,
            "Order was updated incorrectly after removal."
        );
        assert_eq!(
            graph.size(),
            1,
            "Size was updated incorrectly after removal."
        );
        assert_eq!(
            graph.node_ids().sort_unstable(),
            vec![0, 2].sort_unstable(),
            "Nodelist was updated incorrectly after removal."
        );
        assert_eq!(
            graph.edge_ids().sort_unstable(),
            vec![(2, 0)].sort_unstable(),
            "Edgelist was updated incorrectly after removal."
        );
    }

    #[test]
    fn degree_works() {
        let graph = valid_weighted();

        assert_eq!(
            graph.degree(0).unwrap(),
            1,
            "Node 0 only has an edge to node 1."
        );
        assert_eq!(
            graph.degree(2).unwrap(),
            2,
            "Node 2 has an edge to node 0 and node 1."
        );
    }

    #[test]
    fn degree_errors_for_missing() {
        let graph = valid_weighted();
        let err = graph.degree(5).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(5)),
            "Graph does not have node 5."
        );
    }

    #[test]
    fn edges_works() {
        let graph = valid_weighted();
        let empty = MatrixGraph::<usize, usize, usize>::default();

        assert_eq!(
            graph.edge_ids(),
            vec![(0, 1), (1, 2), (2, 0), (2, 1)],
            "Edge list is wrong."
        );
        assert_eq!(
            empty.edge_ids(),
            vec![],
            "Edge list should be empty, since graph is empty."
        );
    }

    #[test]
    fn edge_weight_works() {
        let graph = valid_weighted();

        assert_eq!(
            graph.edge_weight((0, 1)).unwrap(),
            &100,
            "Edge has wrong weight."
        );
    }

    #[test]
    fn edge_weight_errors_for_missing() {
        let graph = valid_weighted();

        assert_eq!(
            graph.edge_weight((1, 1)).err(),
            Some(GraphError::MissingEdge((1, 1))),
            "Edge (1, 1) should not be in the graph."
        );
    }

    #[test]
    fn has_edge_works() {
        let graph = valid_weighted();

        assert!(
            graph.has_edge((0, 1)),
            "Graph should have an edge from node 0 to node 1."
        );
        assert!(
            graph.has_edge((2, 0)),
            "Graph should have an edge from node 2 to node 0."
        );
        assert!(
            !graph.has_edge((1, 0)),
            "Graph should not have an edge from node 1 to node 0."
        );
    }

    #[test]
    fn adding_edge_works() {
        let mut graph = valid_weighted();
        graph.add_edge((1, 0), 17).unwrap();

        assert!(graph.has_node(0), "A node was removed.");
        assert!(graph.has_node(1), "A node was removed.");
        assert!(graph.has_edge((1, 0)), "Edge was not inserted.");
        assert_eq!(
            graph.edge_weight((1, 0)).unwrap(),
            &17,
            "Edge has wrong weight."
        )
    }

    #[test]
    fn adding_duplicate_edge_errors() {
        let mut graph = valid_weighted();
        let err = graph.add_edge((0, 1), 1000).err();

        assert_eq!(
            err,
            Some(GraphError::DuplicateEdge((0, 1))),
            "The edge should be a duplicate."
        )
    }

    #[test]
    fn adding_edge_with_missing_node_errors() {
        let mut graph = valid_weighted();
        let err1 = graph.add_edge((0, 5), 1000).err();
        let err2 = graph.add_edge((5, 0), 1000).err();

        assert_eq!(
            err1,
            Some(GraphError::MissingNode(5)),
            "The node 5 should not be in the graph."
        );
        assert_eq!(
            err2,
            Some(GraphError::MissingNode(5)),
            "The node 5 should not be in the graph."
        );
    }

    #[test]
    fn removing_edge_works() {
        let mut graph = valid_weighted();
        graph.remove_edge((0, 1));

        assert!(graph.has_node(0), "A node was removed.");
        assert!(graph.has_node(1), "A node was removed.");
        assert!(!graph.has_edge((0, 1)), "The edge is still there.");

        graph.remove_edge((0, 1));
        assert!(
            !graph.has_edge((0, 1)),
            "Multiple deletions insert the edge back."
        );
    }

    #[test]
    fn internal_shortest_path_works() {
        let graph = valid_weighted();
        let (prevs, dists) = graph._shortest_paths(0);

        assert_eq!(prevs[0], Some(0), "0 should always go via 0");
        assert_eq!(prevs[1], Some(0), "1 should have a direct path to 0");
        assert_eq!(prevs[2], Some(1), "2 should go via 1");

        assert_eq!(dists[0], 0, "Distance to start node should be 0");
        assert_eq!(
            dists[1], 100,
            "Distance to 1 should be 100 via straight path"
        );
        assert_eq!(dists[2], 201, "Distance to 2 should be 201 via path over 1");
    }

    #[test]
    fn shortest_paths_works() {
        let graph = valid_weighted();
        let map = graph.shortest_paths(0);

        assert_eq!(
            map[&0],
            Some((Solution::from_nodes(vec![0]), 0)),
            "0 is the start node"
        );
        assert_eq!(
            map[&1],
            Some((Solution::from_nodes(vec![0, 1]), 100)),
            "1 should have a direct path to 0 with length 100"
        );
        assert_eq!(
            map[&2],
            Some((Solution::from_nodes(vec![0, 1, 2]), 201)),
            "2 should go via 1 and have length 201"
        );
    }

    #[test]
    fn internal_iter_neighbors_inv_works() {
        let graph = valid_weighted();
        let neighbors: Vec<_> = graph._iter_inv_neighbors(0).unwrap().collect();

        assert_eq!(
            neighbors,
            vec![(2, &200)],
            "Node 0 only has one inverse neighbor, which is 2"
        )
    }

    #[test]
    fn internal_inv_shortest_path_works() {
        let graph = inv_valid_weighted();
        let (prevs, dists) = graph._inv_shortest_paths(0);

        assert_eq!(prevs[0], Some(0), "0 should always go via 0");
        assert_eq!(prevs[1], Some(0), "1 should have a direct path to 0");
        assert_eq!(prevs[2], Some(1), "2 should go via 1");

        assert_eq!(dists[0], 0, "Distance to start node should be 0");
        assert_eq!(
            dists[1], 100,
            "Distance to 1 should be 100 via straight path"
        );
        assert_eq!(dists[2], 201, "Distance to 2 should be 201 via path over 1");
    }

    #[test]
    fn inv_shortest_paths_works() {
        let graph = inv_valid_weighted();
        let map = graph.inv_shortest_paths(0);

        assert_eq!(
            map[&0],
            Some((Solution::from_nodes(vec![0]), 0)),
            "0 is the start node"
        );
        assert_eq!(
            map[&1],
            Some((Solution::from_nodes(vec![1, 0]), 100)),
            "1 should have a direct path to 0 with length 100"
        );
        assert_eq!(
            map[&2],
            Some((Solution::from_nodes(vec![2, 1, 0]), 201)),
            "2 should go via 1 and have length 201"
        );
    }

    #[bench]
    fn bench_iter_edge_ids(b: &mut Bencher) {
        let graph = valid_weighted();

        b.iter(|| {
            for _ in graph.iter_edge_ids() {
                let _n = test::black_box(1);
            }
        })
    }

    #[bench]
    fn bench_iter_edges(b: &mut Bencher) {
        let graph = valid_weighted();

        b.iter(|| {
            for _ in graph.iter_edges() {
                let _n = test::black_box(1);
            }
        })
    }

    #[bench]
    fn bench_edge_ids(b: &mut Bencher) {
        let graph = valid_weighted();

        b.iter(|| {
            for _ in graph.edge_ids() {
                let _n = test::black_box(1);
            }
        })
    }

    #[bench]
    fn bench_sequential_weights(b: &mut Bencher) {
        let graph = valid_weighted();

        b.iter(|| {
            let edges = graph.edge_ids();
            for edge in edges {
                graph.edge_weight(edge).unwrap();
                let _n = test::black_box(1);
            }
        })
    }
}

#[cfg(test)]
mod geopoint_indexed_tests {
    use super::*;
    use crate::geo::GeoPoint;
    use crate::graph::GenericWeightedGraph;
    extern crate test;

    fn valid_weighted() -> MatrixGraph<GeoPoint, usize, usize> {
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);

        MatrixGraph::new(
            vec![(p1, 12), (p2, 21), (p3, 7)],
            vec![
                ((p1, p2), 100),
                ((p2, p3), 101),
                ((p3, p2), 50),
                ((p3, p1), 200),
            ],
        )
        .unwrap()
    }

    fn inv_valid_weighted() -> MatrixGraph<GeoPoint, usize, usize> {
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);

        MatrixGraph::new(
            vec![(p1, 12), (p2, 21), (p3, 7)],
            vec![
                ((p2, p1), 100),
                ((p3, p2), 101),
                ((p2, p3), 50),
                ((p1, p3), 200),
            ],
        )
        .unwrap()
    }

    #[test]
    fn new_empty_weighted_works() {
        let graph = MatrixGraph::<GeoPoint, usize, usize>::default();

        assert!(
            graph.adjacency_matrix.is_empty(),
            "Internal graph is not empty."
        );
        assert!(graph.is_empty(), "Graph is not empty.");
        assert!(graph.node_map.is_empty(), "Node map is not empty.");
        assert!(
            graph.inv_node_map.is_empty(),
            "Inverse node map is not empty."
        );
    }

    #[test]
    fn new_weighted_with_size_works() {
        let graph = MatrixGraph::<GeoPoint, usize, usize>::with_size(5);

        assert!(
            graph.node_map.capacity() >= 5,
            "Not enough space in node map."
        );
        assert!(
            graph.inv_node_map.capacity() >= 5,
            "Not enough space in inverse node map."
        );
        assert_eq!(
            graph.node_ids().len(),
            0,
            "There is a node in the graph, which should not be there."
        );
        assert_eq!(
            graph.edge_ids().len(),
            0,
            "There is an edge in the graph, which should not be tehere."
        );
    }

    #[test]
    fn new_weighted_from_lists_works() {
        let graph = valid_weighted();

        assert_eq!(graph.order(), 3, "Internal graph is too small.");
        assert!(
            graph
                .node_map
                .contains_key(&GeoPoint::from_degrees(12.7, 21.8)),
            "Node map is missing a key."
        );
        assert!(
            graph.inv_node_map.contains_key(&2),
            "Inverse node map is missing a key."
        );
    }

    #[test]
    fn new_with_missing_from_node_errors() {
        let p1 = GeoPoint::from_degrees(12.3, 1.2);
        let p2 = GeoPoint::from_degrees(13.3, 1.1);
        let err = MatrixGraph::new(vec![(p1, 1)], vec![((p2, p1), 2)]).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(p2)),
            "Not missing the node it should be missing."
        );
    }

    #[test]
    fn new_with_missing_to_node_errors() {
        let p1 = GeoPoint::from_degrees(12.3, 1.2);
        let p2 = GeoPoint::from_degrees(13.3, 1.1);
        let err = MatrixGraph::new(vec![(p1, 1)], vec![((p1, p2), 2)]).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(p2)),
            "Not missing the node it should be missing."
        );
    }

    #[test]
    fn is_empty_works() {
        let not_empty = valid_weighted();
        let empty = MatrixGraph::<GeoPoint, usize, usize>::default();

        assert!(!not_empty.is_empty(), "Graph should not be empty.");
        assert!(empty.is_empty(), "Graph should be empty.");
    }

    #[test]
    fn order_works() {
        let not_empty = valid_weighted();
        let empty = MatrixGraph::<GeoPoint, usize, usize>::default();

        assert_eq!(
            not_empty.order(),
            3,
            "Graph should have three nodes, thus order 3."
        );
        assert_eq!(
            empty.order(),
            0,
            "Graph should be empty, thus have order 0."
        );
    }

    #[test]
    fn size_works() {
        let not_empty = valid_weighted();
        let empty = MatrixGraph::<GeoPoint, usize, usize>::default();

        assert_eq!(
            not_empty.size(),
            4,
            "Graph should have four edges, thus size 4."
        );
        assert_eq!(empty.size(), 0, "Graph should be empty, thus have order 0.");
    }

    #[test]
    fn nodes_works() {
        let mut graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        let p4 = GeoPoint::from_degrees(2.4, 53.3);
        let empty = MatrixGraph::<GeoPoint, usize, usize>::default();

        assert_eq!(
            graph.node_ids().sort(),
            vec![p1, p2, p3].sort(),
            "Nodes are not the ones used to construct."
        );
        assert_eq!(
            empty.node_ids(),
            Vec::<GeoPoint>::new(),
            "Nodes should be empty, since graph is empty."
        );

        graph.add_node(p4, 5).unwrap();

        assert_eq!(
            graph.node_ids().sort(),
            vec![p1, p2, p3, p4].sort(),
            "The node p4 should be in, since it was just added."
        );
    }

    #[test]
    fn node_weight_works() {
        let graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);

        assert_eq!(
            graph.node_weight(p1).unwrap(),
            &12,
            "Node has wrong weight."
        );
    }

    #[test]
    fn node_weight_errors_for_missing() {
        let graph = valid_weighted();
        let p4 = GeoPoint::from_degrees(2.4, 53.3);
        let err = graph.node_weight(p4).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(p4)),
            "The node 5 should not be in the graph"
        );
    }

    #[test]
    fn neighbors_works() {
        let mut graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        let p4 = GeoPoint::from_degrees(2.4, 53.3);
        graph.add_node(p4, 0).unwrap();

        assert_eq!(
            graph.neighbor_ids(p1).unwrap(),
            vec![p2],
            "Node p2 should only have p3 as neighbor."
        );
        assert_eq!(
            graph.neighbor_ids(p3).unwrap().len(),
            vec![p1, p2].len(),
            "Node p3 should have p2 and p1 as neighbor."
        );
        assert_eq!(
            graph.neighbor_ids(p4).unwrap(),
            Vec::<GeoPoint>::new(),
            "Node p4 was just inserted and has no neighbors."
        );
    }

    #[test]
    fn neighbors_errors_for_missing() {
        let graph = valid_weighted();
        let p4 = GeoPoint::from_degrees(2.4, 53.3);
        let err = graph.neighbor_ids(p4).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(p4)),
            "The node p4 should not be in the graph, thus has no neighbors."
        );
    }

    #[test]
    fn has_node_works() {
        let graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p4 = GeoPoint::from_degrees(2.4, 53.3);

        assert!(graph.has_node(p1));
        assert!(!graph.has_node(p4));
    }

    #[test]
    fn adding_nodes_works() {
        let mut graph = valid_weighted();
        let p4 = GeoPoint::from_degrees(2.4, 53.3);
        graph.add_node(p4, 5).unwrap();

        assert!(graph.has_node(p4), "Node was not added.");
        assert_eq!(
            graph.order(),
            4,
            "Order was not updated correctly after insertion."
        );
        assert_eq!(
            graph.size(),
            4,
            "Size should not change during node insertion."
        );
        assert_eq!(
            graph.node_weight(p4).unwrap(),
            &5,
            "Incorrect weight was applied to new node."
        );
    }

    #[test]
    fn adding_duplicate_node_errors() {
        let mut graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let err = graph.add_node(p1, 1).err();

        assert_eq!(
            err,
            Some(GraphError::DuplicateNode(p1)),
            "The node should be a duplicate."
        );
    }

    #[test]
    fn removing_nodes_works() {
        let mut graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        graph.remove_node(p1);

        assert!(!graph.has_node(p1), "Has removed node.");
        assert_eq!(
            graph.order(),
            2,
            "Order was updated incorrectly after removal."
        );
        assert_eq!(
            graph.size(),
            2,
            "Size was updated incorrectly after removal."
        );
        assert_eq!(
            graph.node_ids().sort(),
            vec![p2, p3].sort(),
            "Nodelist was updated incorrectly after removal."
        );
        assert_eq!(
            graph.edge_ids().sort(),
            vec![(p2, p3), (p3, p2)].sort(),
            "Edgelist was updated incorrectly after removal."
        );
    }

    #[test]
    fn degree_works() {
        let graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);

        assert_eq!(
            graph.degree(p1).unwrap(),
            1,
            "Node 0 only has an edge to node 1."
        );
        assert_eq!(
            graph.degree(p3).unwrap(),
            2,
            "Node 2 has an edge to node 0 and node 1."
        );
    }

    #[test]
    fn degree_errors_for_missing() {
        let graph = valid_weighted();
        let p4 = GeoPoint::from_degrees(2.4, 53.3);
        let err = graph.degree(p4).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(p4)),
            "Graph does not have node 5."
        );
    }

    #[test]
    fn edges_works() {
        let graph = valid_weighted();
        let empty = MatrixGraph::<GeoPoint, usize, usize>::default();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        let mut edges = vec![(p1, p2), (p2, p3), (p3, p2), (p3, p1)];

        assert_eq!(graph.edge_ids().sort(), edges.sort(), "Edge list is wrong.");
        assert_eq!(
            empty.edge_ids(),
            vec![],
            "Edge list should be empty, since graph is empty."
        );
    }

    #[test]
    fn edge_weight_works() {
        let graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);

        assert_eq!(
            graph.edge_weight((p1, p2)).unwrap(),
            &100,
            "Edge has wrong weight."
        );
    }

    #[test]
    fn edge_weight_errors_for_missing() {
        let graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);

        assert_eq!(
            graph.edge_weight((p1, p1)).err(),
            Some(GraphError::MissingEdge((p1, p1))),
            "Edge (1, 1) should not be in the graph."
        );
    }

    #[test]
    fn has_edge_works() {
        let graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);

        assert!(
            graph.has_edge((p1, p2)),
            "Graph should have an edge from node 0 to node 1."
        );
        assert!(
            graph.has_edge((p3, p1)),
            "Graph should have an edge from node 2 to node 0."
        );
        assert!(
            !graph.has_edge((p3, p3)),
            "Graph should not have an edge from node 2 to node 2."
        );
    }

    #[test]
    fn adding_edge_works() {
        let mut graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        graph.add_edge((p1, p3), 17).unwrap();

        assert!(graph.has_node(p1), "A node was removed.");
        assert!(graph.has_node(p3), "A node was removed.");
        assert!(graph.has_edge((p1, p3)), "Edge was not inserted.");
        assert_eq!(
            graph.edge_weight((p1, p3)).unwrap(),
            &17,
            "Edge has wrong weight."
        )
    }

    #[test]
    fn adding_duplicate_edge_errors() {
        let mut graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        let err = graph.add_edge((p3, p1), 1000).err();

        assert_eq!(
            err,
            Some(GraphError::DuplicateEdge((p3, p1))),
            "The edge should be a duplicate."
        )
    }

    #[test]
    fn adding_edge_with_missing_node_errors() {
        let mut graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p4 = GeoPoint::from_degrees(2.4, 53.3);
        let err1 = graph.add_edge((p1, p4), 1000).err();
        let err2 = graph.add_edge((p4, p1), 1000).err();

        assert_eq!(
            err1,
            Some(GraphError::MissingNode(p4)),
            "The node 5 should not be in the graph."
        );
        assert_eq!(
            err2,
            Some(GraphError::MissingNode(p4)),
            "The node 5 should not be in the graph."
        );
    }

    #[test]
    fn removing_edge_works() {
        let mut graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        graph.remove_edge((p1, p2));

        assert!(graph.has_node(p1), "A node was removed.");
        assert!(graph.has_node(p2), "A node was removed.");
        assert!(!graph.has_edge((p1, p2)), "The edge is still there.");

        graph.remove_edge((p1, p2));
        assert!(
            !graph.has_edge((p1, p2)),
            "Multiple deletions should not insert the edge back."
        );
    }

    #[test]
    fn shortest_paths_works() {
        let graph = valid_weighted();
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        let map = graph.shortest_paths(p1);

        assert_eq!(
            map[&p1],
            Some((Solution::from_nodes(vec![p1]), 0)),
            "0 is the start node"
        );
        assert_eq!(
            map[&p2],
            Some((Solution::from_nodes(vec![p1, p2]), 100)),
            "1 should have a direct path to 0 with length 100"
        );
        assert_eq!(
            map[&p3],
            Some((Solution::from_nodes(vec![p1, p2, p3]), 201)),
            "2 should go via 1 and have length 201"
        );
    }

    #[test]
    fn internal_inv_shortest_path_works() {
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        let graph = inv_valid_weighted();
        let map = graph.inv_shortest_paths(p1);

        assert_eq!(
            map[&p1],
            Some((Solution::from_nodes(vec![p1]), 0)),
            "0 is the start node"
        );
        assert_eq!(
            map[&p2],
            Some((Solution::from_nodes(vec![p2, p1]), 100)),
            "1 should have a direct path to 0 with length 100"
        );
        assert_eq!(
            map[&p3],
            Some((Solution::from_nodes(vec![p3, p2, p1]), 201)),
            "2 should go via 1 and have length 201"
        );
    }
}
