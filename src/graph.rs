mod error;

pub mod export;
pub mod generate;
pub mod geo;
pub mod import;
mod matrix_graph;

use crate::geo::GeoPoint;
pub use error::GraphError;
pub use matrix_graph::MatrixGraph;

pub type Edge<IndexType> = (IndexType, IndexType);

pub trait GenericWeightedGraph<IndexType, Nw, Ew> {
    /// Returns true if there are no nodes, or false otherwise.
    fn is_empty(&self) -> bool;

    /// Returns the number of nodes in this graph.
    fn order(&self) -> usize;

    /// Returns the number of edges in this graph.
    fn size(&self) -> usize;

    /// Returns an iterator over node ids.
    fn iter_node_ids(&self) -> Box<dyn Iterator<Item = IndexType> + '_>;

    /// Returns the node ids of this graph.
    fn node_ids(&self) -> Vec<IndexType>;

    /// Returns an iterator over the node ids and a reference to their weight.
    fn iter_nodes(&self) -> Box<dyn Iterator<Item = (IndexType, &Nw)> + '_>;

    /// Returns the weight of node with id.
    fn node_weight(&self, id: IndexType) -> Result<&Nw, GraphError<IndexType>>;

    /// Returns an iterator over the neighboring ids.
    /// Returns GraphError, if the specified node id is not in the graph.
    fn iter_neighbor_ids(
        &self,
        id: IndexType,
    ) -> Result<Box<dyn Iterator<Item = IndexType> + '_>, GraphError<IndexType>>;

    /// Returns the neighbors of the node with id.
    /// Returns an error if node is not in graph.
    fn neighbor_ids(&self, id: IndexType) -> Result<Vec<IndexType>, GraphError<IndexType>>;

    /// Returns an iterator over the neighbor ids with a reference to that edges weight
    /// Returns an error if the node is not in the graph.
    #[allow(clippy::type_complexity)]
    fn iter_neighbors(
        &self,
        id: IndexType,
    ) -> Result<Box<dyn Iterator<Item = (IndexType, &Ew)> + '_>, GraphError<IndexType>>;

    fn neighbors(&self, id: IndexType) -> Result<Vec<(IndexType, &Ew)>, GraphError<IndexType>>;

    /// Returns true if node with id is a member, or false otherwise.
    fn has_node(&self, id: IndexType) -> bool;

    /// Adds a new node with weight to the graph.
    /// Returns an error if a node with the same id already exists.
    fn add_node(&mut self, id: IndexType, weight: Nw) -> Result<(), GraphError<IndexType>>;

    /// Removes a weighted node from the graph.
    /// This in turn means all edges from or to this node will be removed.
    fn remove_node(&mut self, id: IndexType);

    /// Changes the weight of a node to the new weight.
    /// Adds the node, if it was not in the graph before.
    fn change_node(&mut self, id: IndexType, weight: Nw);

    /// Returns the count of neighbors at node with given id.
    /// Returns an error if the node is not in the graph.
    fn degree(&self, id: IndexType) -> Result<usize, GraphError<IndexType>>;

    /// Returns an iterator over edge ids in the form (from_id, to_id)
    fn iter_edge_ids(&self) -> Box<dyn Iterator<Item = Edge<IndexType>> + '_>;

    /// Returns a vec of all edge ids in the form (from_id, to_id)
    fn edge_ids(&self) -> Vec<Edge<IndexType>>;

    /// Returns an iterator over all edges with their according weights
    fn iter_edges(&self) -> Box<dyn Iterator<Item = (Edge<IndexType>, &Ew)> + '_>;

    /// Returns a vec of all edges and a reference to their weights
    fn edges(&self) -> Vec<(Edge<IndexType>, &Ew)>;

    /// Returns the weight of an edge.
    fn edge_weight(&self, edge: Edge<IndexType>) -> Result<&Ew, GraphError<IndexType>>;

    /// Returns true if the edge exists, or false otherwise.
    /// Returns MissingNode if either starting or ending nodes of the edge are not in the graph.
    fn has_edge(&self, edge: Edge<IndexType>) -> bool;

    /// Adds a new weighted edge to the graph.
    /// Returns an error if the edge already exists or one of the nodes is missing.
    fn add_edge(&mut self, edge: Edge<IndexType>, weight: Ew) -> Result<(), GraphError<IndexType>>;

    /// Removes a weighted edge from the graph.
    fn remove_edge(&mut self, edge: Edge<IndexType>);

    /// Changes the weight of a edge to the new weight.
    /// If the edge did not exist before, it gets created in this process.
    /// If the new edge can't be created, because one of the nodes is not in the graph this errors.
    fn change_edge(
        &mut self,
        edge: Edge<IndexType>,
        weight: Ew,
    ) -> Result<(), GraphError<IndexType>>;
}

pub trait WeightedGraph<Nw, Ew>: GenericWeightedGraph<usize, Nw, Ew> {}

pub trait GeoGraph<Nw, Ew>: GenericWeightedGraph<GeoPoint, Nw, Ew> {}

pub trait GenericGraph<IndexType> {
    /// Returns true if there are no nodes, or false otherwise.
    fn is_empty(&self) -> bool;

    /// Returns the number of nodes in this graph.
    fn order(&self) -> usize;

    /// Returns the number of edges in this graph.
    fn size(&self) -> usize;

    /// Returns an iterator over all nodes of this graph.
    fn iter_nodes(&self) -> Box<dyn Iterator<Item = IndexType>>;

    /// Returns the nodes of this graph.
    fn nodes(&self) -> Vec<IndexType>;

    /// Returns an iterator over the neighbors of node with given id.
    /// Returns an error if that node is not in the graph.
    fn iter_neighbors(
        &self,
        id: IndexType,
    ) -> Result<Box<dyn Iterator<Item = IndexType> + '_>, GraphError<IndexType>>;

    /// Returns the neighbors of the node with id.
    /// Returns an error if node is not in graph.
    fn neighbors(&self, id: IndexType) -> Result<Vec<IndexType>, GraphError<IndexType>>;

    /// Returns true if node with id is a member, or false otherwise.
    fn has_node(&self, id: IndexType) -> bool;

    /// Adds a new node to the graph.
    /// Returns an error if a node with the same id already exists.
    fn add_node(&mut self, id: IndexType) -> Result<(), GraphError<IndexType>>;

    /// Removes a node from the graph.
    /// This in turn means all edges from or to this node will be removed.
    fn remove_node(&mut self, id: IndexType);

    /// Returns the count of neighbors at node with given id.
    /// Returns an error if the node is not in the graph.
    fn degree(&self, id: IndexType) -> Result<IndexType, GraphError<IndexType>>;

    /// Returns an iterator over the edges of this graph.
    fn iter_edges(&self) -> Box<dyn Iterator<Item = Edge<IndexType>> + '_>;

    /// Returns the edges of this graph.
    fn edges(&self) -> Vec<(IndexType, IndexType)>;

    /// Returns true if the edge exists, or false otherwise.
    /// Returns MissingNode if either starting or ending nodes of the edge are not in the graph.
    fn has_edge(&self, edge: Edge<IndexType>) -> bool;

    /// Adds a new edge to the graph.
    /// Returns an error if the edge already exists or one of the nodes is missing.
    fn add_edge(&mut self, edge: Edge<IndexType>) -> Result<(), GraphError<IndexType>>;

    /// Removes an edge from the graph.
    fn remove_edge(&mut self, edge: Edge<IndexType>);
}

pub trait Graph: GenericGraph<usize> {}
