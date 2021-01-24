use crate::graph::{Edge, GenericWeightedGraph, GraphError, WeightedGraph};

#[derive(Debug, PartialEq, Clone)]
pub struct MatrixGraph<Nw, Ew> {
    pub adjacency_matrix: Vec<Vec<Option<Ew>>>,
    node_weights: Vec<Option<Nw>>,
    order: usize,
    size: usize,
}

/// Implements a weighted, directed graph using an adjacency matrix as datastructure.
#[allow(dead_code)]
impl<Nw: Clone, Ew: Clone> MatrixGraph<Nw, Ew> {
    /// Creates a new Graph instance using the given list of node_weights and weighted edges.
    /// The indices of nodes are inferred from their position in the given array,
    /// meaning the node at nodes[i] will get the index i in the graph instance.
    /// If any of the edges don't fit this scheme an error is returned.
    pub fn new(nodes: Vec<Nw>, edges: Vec<(usize, usize, Ew)>) -> Result<Self, GraphError<usize>> {
        let node_amount = nodes.len();

        let mut graph = MatrixGraph {
            // The initialization of adjacency_matrix makes it necessary, that Ew is of type Clone.
            // If that can be fixed Ew won't need to be Clone.
            adjacency_matrix: (0..node_amount).map(|_| vec![None; node_amount]).collect(),
            node_weights: nodes.into_iter().map(Some).collect(),
            order: node_amount,
            size: edges.len(),
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

    /// Default constructor for an empty MatrixGraph.
    /// If the amount of nodes is known beforehand use either MatrixGraph::new()
    /// or MatrixGraph::with_size(), as they don't require resizing later, wich is slow.
    pub fn default() -> Self {
        MatrixGraph {
            adjacency_matrix: Vec::new(),
            node_weights: Vec::new(),
            order: 0,
            size: 0,
        }
    }

    /// Constructs an empty MatrixGraph with capacity for at least the given amount of nodes.
    pub fn with_size(size: usize) -> Self {
        MatrixGraph {
            adjacency_matrix: (0..size).map(|_| vec![None; size]).collect(),
            node_weights: vec![None; size],
            order: 0,
            size: 0,
        }
    }
}

#[allow(dead_code)]
impl MatrixGraph<(), ()> {
    /// Constructs an unweighted MatrixGraph using the given amount of nodes and list of edges.
    pub fn new_unweighted(nodes: usize, edges: &[Edge<usize>]) -> Result<Self, GraphError<usize>> {
        // initialization works basically the same way as for generic types.
        let mut graph = MatrixGraph {
            adjacency_matrix: (0..nodes).map(|_| vec![None; nodes]).collect(),
            node_weights: vec![Some(()); nodes],
            order: nodes,
            size: edges.len(),
        };

        // A lot of places where to_owned is used. Could possibly be simplified.
        for (from, to) in edges.iter() {
            if from >= &nodes {
                return Err(GraphError::MissingNode(*from));
            } else if to >= &nodes {
                return Err(GraphError::MissingNode(*to));
            }

            graph.adjacency_matrix[*from][*to] = Some(());
        }

        Ok(graph)
    }

    /// Constructs an empty unweighted MatrixGraph.
    pub fn default_unweighted() -> Self {
        MatrixGraph::default()
    }

    /// Constructs an unweighted MatrixGraph with capacity for at least the given amount of nodes.
    pub fn unweighted_with_size(size: usize) -> Self {
        MatrixGraph::with_size(size)
    }
}

impl<Nw: Clone, Ew: Clone> GenericWeightedGraph<usize, Nw, Ew> for MatrixGraph<Nw, Ew> {
    fn is_empty(&self) -> bool {
        self.node_weights.is_empty()
    }

    fn order(&self) -> usize {
        self.order
    }

    fn size(&self) -> usize {
        self.size
    }

    fn iter_node_ids(&self) -> Box<dyn Iterator<Item = usize> + '_> {
        Box::new(
            self.node_weights
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(move |(i, _)| i),
        )
    }

    fn node_ids(&self) -> Vec<usize> {
        self.iter_node_ids().collect()
    }

    fn iter_nodes(&self) -> Box<dyn Iterator<Item = (usize, &Nw)> + '_> {
        Box::new(
            self.node_weights
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(move |(i, _)| (i, self.node_weight(i).unwrap())),
        )
    }

    fn node_weight(&self, id: usize) -> Result<&Nw, GraphError<usize>> {
        if !self.has_node(id) {
            return Err(GraphError::MissingNode(id));
        }

        // Unwrapping is ok here, because has_node ensures, that there is a weight in the array at position id.
        Ok(self.node_weights[id].as_ref().unwrap())
    }

    fn iter_neighbor_ids(
        &self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = usize> + '_>, GraphError<usize>> {
        if !self.has_node(id) {
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

    fn neighbor_ids(&self, id: usize) -> Result<Vec<usize>, GraphError<usize>> {
        match self.iter_neighbor_ids(id) {
            Ok(iterator) => Ok(iterator.collect()),
            Err(error) => Err(error),
        }
    }

    fn iter_neighbors(
        &self,
        id: usize,
    ) -> Result<Box<dyn Iterator<Item = (usize, &Ew)> + '_>, GraphError<usize>> {
        if !self.has_node(id) {
            return Err(GraphError::MissingNode(id));
        }

        // Get the ids of nodes to which a weighted edge from id exists.
        Ok(Box::new(
            self.adjacency_matrix[id]
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(move |(i, _)| (i, self.edge_weight((id, i)).unwrap())),
        ))
    }

    fn has_node(&self, id: usize) -> bool {
        self.node_weights.len() > id && self.node_weights[id].is_some()
    }

    fn add_node(&mut self, id: usize, weight: Nw) -> Result<(), GraphError<usize>> {
        if self.node_weights.len() > id && self.has_node(id) {
            return Err(GraphError::DuplicateNode(id));
        } else if self.node_weights.len() < id {
            // Resizing here will never shrink the array, because has_node() implies id >= node_weights.len().
            // However calling this every time is slower than checking if the array needs to be resized.
            // Possible empty spots in between will be initialized with None.
            self.node_weights.resize_with(id + 1, || None);
            self.adjacency_matrix.resize_with(id + 1, || vec![None; id]);
            for edge_weights in self.adjacency_matrix.iter_mut() {
                edge_weights.resize_with(id + 1, || None);
            }
        }

        self.node_weights[id] = Some(weight);
        // Adding a node increases order by one.
        self.order += 1;
        Ok(())
    }

    fn remove_node(&mut self, id: usize) {
        if self.has_node(id) {
            // If a node is removed from the graph there can't be any edges to or from it.
            for i in 0..self.order {
                self.remove_edge((i, id));
                self.remove_edge((id, i));
            }

            self.node_weights[id] = None;
            // Removing the node reduces order by one.
            self.order -= 1;
        }
    }

    fn change_node(&mut self, id: usize, weight: Nw) {
        if self.has_node(id) {
            self.node_weights[id] = Some(weight);
        } else {
            // Unwrapping is ok here, because we ensured, we don't have this node id yet.
            self.add_node(id, weight).unwrap();
        }
    }

    fn degree(&self, id: usize) -> Result<usize, GraphError<usize>> {
        // GraphError can be thrown if the node with id is not in the graph.
        Ok(self.neighbor_ids(id)?.len())
    }

    fn iter_edge_ids(&self) -> Box<dyn Iterator<Item = Edge<usize>> + '_> {
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

    fn edge_ids(&self) -> Vec<Edge<usize>> {
        self.iter_edge_ids().collect()
    }

    fn iter_edges(&self) -> Box<dyn Iterator<Item = (Edge<usize>, &Ew)> + '_> {
        Box::new(
            self.adjacency_matrix
                .iter()
                .enumerate()
                .flat_map(move |(i, edges)| {
                    edges
                        .iter()
                        .enumerate()
                        .filter(|(_, weight)| weight.is_some())
                        .map(move |(j, _)| ((i, j), self.edge_weight((i, j)).unwrap()))
                }),
        )
    }

    fn edge_weight(&self, edge: Edge<usize>) -> Result<&Ew, GraphError<usize>> {
        let (start_node, end_node) = edge;
        if !self.has_edge(edge) {
            return Err(GraphError::MissingEdge(edge));
        }

        Ok(&self.adjacency_matrix[start_node][end_node]
            .as_ref()
            .unwrap())
    }

    fn has_edge(&self, edge: Edge<usize>) -> bool {
        let (start_node, end_node) = edge;
        if !self.has_node(start_node) || !self.has_node(end_node) {
            return false;
        }

        self.adjacency_matrix[start_node][end_node].is_some()
    }

    fn add_edge(&mut self, edge: Edge<usize>, weight: Ew) -> Result<(), GraphError<usize>> {
        let (start_node, end_node) = edge;
        if self.has_edge(edge) {
            return Err(GraphError::DuplicateEdge(edge));
        } else if !self.has_node(start_node) {
            return Err(GraphError::MissingNode(start_node));
        } else if !self.has_node(end_node) {
            return Err(GraphError::MissingNode(end_node));
        }

        self.adjacency_matrix[start_node][end_node] = Some(weight);

        // Adding an edge increases size by one.
        self.size += 1;
        Ok(())
    }

    fn remove_edge(&mut self, edge: Edge<usize>) {
        if self.has_edge(edge) {
            self.adjacency_matrix[edge.0][edge.1] = None;
            // Removing an edge reduces size by one.
            self.size -= 1;
        }
    }

    fn change_edge(&mut self, edge: Edge<usize>, weight: Ew) -> Result<(), GraphError<usize>> {
        if self.has_edge(edge) {
            self.adjacency_matrix[edge.0][edge.1] = Some(weight);
            Ok(())
        } else {
            self.add_edge(edge, weight)
        }
    }
}

impl<Nw: Clone, Ew: Clone> WeightedGraph<Nw, Ew> for MatrixGraph<Nw, Ew> {}

// impl GenericGraph<usize> for MatrixGraph<(), ()> {
//     fn is_empty(&self) -> bool {
//         <Self as GenericWeightedGraph<usize, (), ()>>::is_empty(self)
//     }
//
//     fn order(&self) -> usize {
//         <Self as GenericWeightedGraph<usize, (), ()>>::order(self)
//     }
//
//     fn size(&self) -> usize {
//         <Self as GenericWeightedGraph<usize, (), ()>>::size(self)
//     }
//
//     fn nodes(&self) -> Vec<usize> {
//         <Self as GenericWeightedGraph<usize, (), ()>>::nodes(self)
//     }
//
//     fn neighbors(&self, id: usize) -> Result<Vec<usize>, GraphError> {
//         <Self as GenericWeightedGraph<usize, (), ()>>::neighbors(self, id)
//     }
//
//     fn has_node(&self, id: usize) -> bool {
//         <Self as GenericWeightedGraph<usize, (), ()>>::has_node(self, id)
//     }
//
//     fn add_node(&mut self, id: usize) -> Result<(), GraphError> {
//         <Self as GenericWeightedGraph<usize,(), ()>>::add_node(self, id, ())
//     }
//
//     fn remove_node(&mut self, id: usize) {
//         <Self as GenericWeightedGraph<usize, (), ()>>::remove_node(self, id)
//     }
//
//     fn degree(&self, id: usize) -> Result<usize, GraphError> {
//         <Self as GenericWeightedGraph<usize, (), ()>>::degree(self, id)
//     }
//
//     fn edges(&self) -> Vec<Edge<usize>> {
//         <Self as GenericWeightedGraph<usize, (), ()>>::edges(self)
//     }
//
//     fn has_edge(&self, edge: Edge<usize>) -> bool {
//         <Self as GenericWeightedGraph<usize, (), ()>>::has_edge(self, edge)
//     }
//
//     fn add_edge(&mut self, edge: Edge<usize>) -> Result<(), GraphError> {
//         <Self as GenericWeightedGraph<usize, (), ()>>::add_edge(self, edge, ())
//     }
//
//     fn remove_edge(&mut self, edge: Edge<usize>) {
//         <Self as GenericWeightedGraph<usize, (), ()>>::remove_edge(self, edge)
//     }
// }
//
// impl Graph for MatrixGraph<(), ()> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GenericWeightedGraph;
    use test::Bencher;
    extern crate test;

    fn valid_weighted() -> MatrixGraph<usize, usize> {
        MatrixGraph::new(
            vec![1, 2, 3],
            vec![(0, 1, 100), (1, 2, 101), (2, 1, 50), (2, 0, 200)],
        )
        .unwrap()
    }

    fn valid_vector_weighted() -> MatrixGraph<Vec<usize>, Vec<usize>> {
        MatrixGraph::new(
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
        let graph = MatrixGraph::<usize, usize>::default();

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
        let graph = MatrixGraph::<usize, usize>::with_size(5);

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
        let err = MatrixGraph::new(vec![1], vec![(1, 0, 1)]).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(1)),
            "Not missing the node it should be missing."
        );
    }

    #[test]
    fn new_with_missing_to_node_errors() {
        let err = MatrixGraph::new(vec![1], vec![(0, 1, 1)]).err();

        assert_eq!(
            err,
            Some(GraphError::MissingNode(1)),
            "Not missing the node it should be missing."
        );
    }

    #[test]
    fn is_empty_works() {
        let not_empty = valid_weighted();
        let empty = MatrixGraph::<usize, usize>::default();

        assert!(!not_empty.is_empty(), "Graph should not be empty.");
        assert!(empty.is_empty(), "Graph should be empty.");
    }

    #[test]
    fn order_works() {
        let not_empty = valid_weighted();
        let empty = MatrixGraph::<usize, usize>::default();

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
        let empty = MatrixGraph::<usize, usize>::default();

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
        let empty = MatrixGraph::<usize, usize>::default();

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
            graph.neighbor_ids(2).unwrap().sort(),
            vec![1, 0].sort(),
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
            graph.node_ids().sort(),
            vec![0, 1, 2, 4].sort(),
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
            graph.node_ids().sort(),
            vec![0, 2].sort(),
            "Nodelist was updated incorrectly after removal."
        );
        assert_eq!(
            graph.edge_ids().sort(),
            vec![(2, 0)].sort(),
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
        let empty = MatrixGraph::<usize, usize>::default();

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

    #[bench]
    fn bench_iter_edge_ids(b: &mut Bencher) {
        let graph = valid_weighted();

        b.iter(|| {
            let n = test::black_box(1);
            valid_weighted().iter_edge_ids();
        })
    }

    #[bench]
    fn bench_iter_edges(b: &mut Bencher) {
        let graph = valid_weighted();

        b.iter(|| {
            let n = test::black_box(1);
            valid_weighted().iter_edges();
        })
    }

    #[bench]
    fn bench_edge_ids(b: &mut Bencher) {
        let graph = valid_weighted();

        b.iter(|| {
            let n = test::black_box(1);
            valid_weighted().edge_ids();
        })
    }

    #[bench]
    fn bench_sequential_weights(b: &mut Bencher) {
        let graph = valid_weighted();

        b.iter(|| {
            let n = test::black_box(1);
            let edges = valid_weighted().edge_ids();
            for edge in edges {
                valid_weighted().edge_weight(edge).unwrap();
            }
        })
    }
}
