use std::collections::HashMap;

use crate::geo::GeoPoint;
use crate::graph::{regular, Edge, GenericWeightedGraph, GeoGraph, GraphError};

pub struct MatrixGraph<Nw, Ew> {
    internal_graph: regular::MatrixGraph<Nw, Ew>,
    node_map: HashMap<GeoPoint, usize>,
    inv_node_map: HashMap<usize, GeoPoint>,
}

#[allow(dead_code)]
impl<Nw: Copy, Ew: Copy> MatrixGraph<Nw, Ew> {
    pub fn new(
        nodes: Vec<(GeoPoint, Nw)>,
        edges: Vec<(Edge<GeoPoint>, Ew)>,
    ) -> Result<Self, GraphError<GeoPoint>> {
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

        let graph = regular::MatrixGraph::new(nodes.iter().map(|x| x.1).collect(), mapped_edges);

        match graph {
            Ok(valid_graph) => Ok(MatrixGraph {
                internal_graph: valid_graph,
                node_map,
                inv_node_map,
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

    pub fn default() -> Self {
        MatrixGraph {
            internal_graph: regular::MatrixGraph::default(),
            node_map: HashMap::new(),
            inv_node_map: HashMap::new(),
        }
    }

    pub fn with_size(size: usize) -> Self {
        MatrixGraph {
            internal_graph: regular::MatrixGraph::with_size(size),
            node_map: HashMap::with_capacity(size),
            inv_node_map: HashMap::with_capacity(size),
        }
    }

    fn mapped_result<CorrectType>(
        &self,
        result: Result<CorrectType, GraphError<usize>>,
    ) -> Result<CorrectType, GraphError<GeoPoint>> {
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
}

#[allow(dead_code, clippy::map_entry)]
impl<Nw: Copy, Ew: Copy> GenericWeightedGraph<GeoPoint, Nw, Ew> for MatrixGraph<Nw, Ew> {
    fn is_empty(&self) -> bool {
        self.internal_graph.is_empty()
    }

    fn order(&self) -> usize {
        self.internal_graph.order()
    }

    fn size(&self) -> usize {
        self.internal_graph.size()
    }

    fn iter_node_ids(&self) -> Box<dyn Iterator<Item = GeoPoint> + '_> {
        Box::new(self.node_map.keys().copied())
    }

    fn node_ids(&self) -> Vec<GeoPoint> {
        self.iter_node_ids().collect()
    }

    fn iter_nodes(&self) -> Box<dyn Iterator<Item = (GeoPoint, &Nw)> + '_> {
        Box::new(
            self.internal_graph
                .iter_nodes()
                .map(move |(id, node)| (self.inv_node_map[&id], node)),
        )
    }

    fn node_weight(&self, id: GeoPoint) -> Result<&Nw, GraphError<GeoPoint>> {
        if !self.node_map.contains_key(&id) {
            return Err(GraphError::MissingNode(id));
        }

        let weight = self.internal_graph.node_weight(self.node_map[&id]);
        self.mapped_result(weight)
    }

    fn iter_neighbor_ids(
        &self,
        id: GeoPoint,
    ) -> Result<Box<dyn Iterator<Item = GeoPoint> + '_>, GraphError<GeoPoint>> {
        if !self.node_map.contains_key(&id) {
            return Err(GraphError::MissingNode(id));
        }

        let inner = self.internal_graph.iter_neighbor_ids(self.node_map[&id]);
        let res = self.mapped_result(inner);
        match res {
            Ok(iterator) => Ok(Box::new(iterator.map(move |id| self.inv_node_map[&id]))),
            Err(e) => Err(e),
        }
    }

    fn neighbor_ids(&self, id: GeoPoint) -> Result<Vec<GeoPoint>, GraphError<GeoPoint>> {
        let res = self.iter_neighbor_ids(id);
        match res {
            Ok(iterator) => Ok(iterator.collect()),
            Err(e) => Err(e),
        }
    }

    #[allow(clippy::type_complexity)]
    fn iter_neighbors(
        &self,
        id: GeoPoint,
    ) -> Result<Box<dyn Iterator<Item = (GeoPoint, &Ew)> + '_>, GraphError<GeoPoint>> {
        let inner = self.internal_graph.iter_neighbors(self.node_map[&id]);
        let res = self.mapped_result(inner);
        match res {
            Ok(iterator) => Ok(Box::new(
                iterator.map(move |(id, point)| (self.inv_node_map[&id], point)),
            )),
            Err(e) => Err(e),
        }
    }

    fn has_node(&self, id: GeoPoint) -> bool {
        self.node_map.contains_key(&id) && self.internal_graph.has_node(self.node_map[&id])
    }

    fn add_node(&mut self, id: GeoPoint, weight: Nw) -> Result<(), GraphError<GeoPoint>> {
        if self.node_map.contains_key(&id) {
            return Err(GraphError::DuplicateNode(id));
        }

        // order is always amount of nodes + 1, so we can use it as our new id for internal
        let inner_id = self.internal_graph.order();
        let res = self.internal_graph.add_node(inner_id, weight);
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

    fn remove_node(&mut self, id: GeoPoint) {
        if let Some(&inner_id) = self.node_map.get(&id) {
            self.node_map.remove(&id);
            self.inv_node_map.remove(&inner_id);
            self.internal_graph.remove_node(inner_id);
        }
    }

    fn change_node(&mut self, id: GeoPoint, weight: Nw) {
        if self.node_map.contains_key(&id) {
            self.internal_graph.change_node(self.node_map[&id], weight);
        } else {
            let inner_id = self.internal_graph.order();
            self.node_map.insert(id, inner_id);
            self.inv_node_map.insert(inner_id, id);
        }
    }

    fn degree(&self, id: GeoPoint) -> Result<usize, GraphError<GeoPoint>> {
        if !self.node_map.contains_key(&id) {
            return Err(GraphError::MissingNode(id));
        }

        let degree = self.internal_graph.degree(self.node_map[&id]);
        self.mapped_result(degree)
    }

    fn iter_edge_ids(&self) -> Box<dyn Iterator<Item = Edge<GeoPoint>> + '_> {
        Box::new(
            self.internal_graph
                .iter_edge_ids()
                .map(move |(f_id, t_id)| (self.inv_node_map[&f_id], self.inv_node_map[&t_id])),
        )
    }

    fn edge_ids(&self) -> Vec<Edge<GeoPoint>> {
        self.iter_edge_ids().collect()
    }

    fn iter_edges(&self) -> Box<dyn Iterator<Item = (Edge<GeoPoint>, &Ew)> + '_> {
        Box::new(
            self.internal_graph
                .iter_edges()
                .map(move |((f_id, t_id), weight)| {
                    ((self.inv_node_map[&f_id], self.inv_node_map[&t_id]), weight)
                }),
        )
    }

    fn edge_weight(&self, edge: Edge<GeoPoint>) -> Result<&Ew, GraphError<GeoPoint>> {
        let weight = self
            .internal_graph
            .edge_weight((self.node_map[&edge.0], self.node_map[&edge.1]));
        self.mapped_result(weight)
    }

    fn has_edge(&self, edge: Edge<GeoPoint>) -> bool {
        self.internal_graph
            .has_edge((self.node_map[&edge.0], self.node_map[&edge.1]))
    }

    fn add_edge(&mut self, edge: Edge<GeoPoint>, weight: Ew) -> Result<(), GraphError<GeoPoint>> {
        if !self.node_map.contains_key(&edge.0) {
            return Err(GraphError::MissingNode(edge.0));
        } else if !self.node_map.contains_key(&edge.1) {
            return Err(GraphError::MissingNode(edge.1));
        }
        let edge = self
            .internal_graph
            .add_edge((self.node_map[&edge.0], self.node_map[&edge.1]), weight);
        self.mapped_result(edge)
    }

    fn remove_edge(&mut self, edge: Edge<GeoPoint>) {
        self.internal_graph
            .remove_edge((self.node_map[&edge.0], self.node_map[&edge.1]));
    }

    fn change_edge(
        &mut self,
        edge: Edge<GeoPoint>,
        weight: Ew,
    ) -> Result<(), GraphError<GeoPoint>> {
        let edge = self
            .internal_graph
            .change_edge((self.node_map[&edge.0], self.node_map[&edge.1]), weight);
        self.mapped_result(edge)
    }
}

impl<Nw: Copy, Ew: Copy> GeoGraph<Nw, Ew> for MatrixGraph<Nw, Ew> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::GeoPoint;
    use crate::graph::GenericWeightedGraph;
    extern crate test;

    fn valid_weighted() -> MatrixGraph<usize, usize> {
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

    #[test]
    fn new_empty_weighted_works() {
        let graph = MatrixGraph::<usize, usize>::default();

        assert!(
            graph.internal_graph.is_empty(),
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
        let graph = MatrixGraph::<usize, usize>::with_size(5);

        assert!(
            graph.node_map.capacity() >= 5,
            "Not enough space in node map."
        );
        assert!(
            graph.inv_node_map.capacity() >= 5,
            "Not enough space in inverse node map."
        );
        assert_eq!(
            graph.internal_graph.node_ids().len(),
            0,
            "There is a node in the graph, which should not be there."
        );
        assert_eq!(
            graph.internal_graph.edge_ids().len(),
            0,
            "There is an edge in the graph, which should not be tehere."
        );
    }

    #[test]
    fn new_weighted_from_lists_works() {
        let graph = valid_weighted();

        assert_eq!(
            graph.internal_graph.order(),
            3,
            "Internal graph is too small."
        );
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
        let p1 = GeoPoint::from_degrees(12.7, 21.8);
        let p2 = GeoPoint::from_degrees(9.7, 12.5);
        let p3 = GeoPoint::from_degrees(11.1, 32.5);
        let p4 = GeoPoint::from_degrees(2.4, 53.3);
        let empty = MatrixGraph::<usize, usize>::default();

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
        let empty = MatrixGraph::<usize, usize>::default();
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
}
