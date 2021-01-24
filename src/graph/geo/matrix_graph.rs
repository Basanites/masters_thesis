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

        let graph = regular::MatrixGraph::new(
            nodes.iter().map(|x| x.1).collect(),
            edges
                .iter()
                .map(|(edge, ew)| (node_map[&edge.0], node_map[&edge.1], *ew))
                .collect(),
        );

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
        let weight = self.internal_graph.node_weight(self.node_map[&id]);
        self.mapped_result(weight)
    }

    fn iter_neighbor_ids(
        &self,
        id: GeoPoint,
    ) -> Result<Box<dyn Iterator<Item = GeoPoint> + '_>, GraphError<GeoPoint>> {
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
