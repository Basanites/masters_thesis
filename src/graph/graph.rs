pub use super::{ GraphError };

pub type Edge<IndexType> = (IndexType, IndexType);

pub trait GenericWeightedGraph<IndexType, Nw, Ew> {
    /// Returns true if there are no nodes, or false otherwise.
    fn is_empty(&self) -> bool;

    /// Returns the number of nodes in this graph.
    fn order(&self) -> usize;

    /// Returns the number of edges in this graph.
    fn size(&self) -> usize;

    /// Returns the nodes of this graph.
    fn nodes(&self) -> Vec<usize>;

    /// Returns the weight of node with id.
    fn node_weight(&self, id: IndexType) -> Result<&Nw, GraphError>;

    /// Iterates the neighbors of the node with id.
    /// Returns an error if node is not in graph.
    fn neighbors(&self, id: IndexType) -> Result<Vec<IndexType>, GraphError>;

    /// Returns true if node with id is a member, or false otherwise.
    fn has_node(&self, id: IndexType) -> bool;

    /// Adds a new node with weight to the graph.
    /// Returns an error if a node with the same id already exists.
    fn add_node(&mut self, id: IndexType, weight: Nw) -> Result<(), GraphError>;

    /// Removes a weighted node from the graph.
    /// This in turn means all edges from or to this node will be removed.
    fn remove_node(&mut self, id: IndexType);

    /// Changes the weight of a node to the new weight.
    /// Adds the node, if it was not in the graph before.
    fn change_node(&mut self, id: IndexType, weight: Nw);

    /// Returns the count of neighbors at node with given id.
    /// Returns an error if the node is not in the graph.
    fn degree(&self, id: IndexType) -> Result<IndexType, GraphError>;

    /// Returns the edges of this graph.
    fn edges(&self) -> Vec<Edge<IndexType>>;

    /// Returns the weight of an edge.
    fn edge_weight(&self, edge: Edge<IndexType>) -> Result<&Ew, GraphError>;

    /// Returns true if the edge exists, or false otherwise.
    /// Returns MissingNode if either starting or ending nodes of the edge are not in the graph.
    fn has_edge(&self, edge: Edge<IndexType>) -> bool;

    /// Adds a new weighted edge to the graph.
    /// Returns an error if the edge already exists or one of the nodes is missing.
    fn add_edge(&mut self, edge: Edge<IndexType>, weight: Ew) -> Result<(), GraphError>;

    /// Removes a weighted edge from the graph.
    fn remove_edge(&mut self, edge: Edge<IndexType>);

    /// Changes the weight of a edge to the new weight.
    /// If the edge did not exist before, it gets created in this process.
    /// If the new edge can't be created, because one of the nodes is not in the graph this errors.
    fn change_edge(&mut self, edge: Edge<IndexType>, weight: Ew) -> Result<(), GraphError>;

}

pub trait GenericGraph<IndexType> {
    /// Returns true if there are no nodes, or false otherwise.
    fn is_empty(&self) -> bool;

    /// Returns the number of nodes in this graph.
    fn order(&self) -> usize;

    /// Returns the number of edges in this graph.
    fn size(&self) -> usize;

    /// Returns the nodes of this graph.
    fn nodes(&self) -> Vec<IndexType>;

    /// Iterates the neighbors of the node with id.
    /// Returns an error if node is not in graph.
    fn neighbors(&self, id: IndexType) -> Result<Vec<IndexType>, GraphError>;

    /// Returns true if node with id is a member, or false otherwise.
    fn has_node(&self, id: IndexType) -> bool;

    /// Adds a new node to the graph.
    /// Returns an error if a node with the same id already exists.
    fn add_node(&mut self, id: IndexType) -> Result<(), GraphError>;

    /// Removes a node from the graph.
    /// This in turn means all edges from or to this node will be removed.
    fn remove_node(&mut self, id: IndexType);

    /// Returns the count of neighbors at node with given id.
    /// Returns an error if the node is not in the graph.
    fn degree(&self, id: IndexType) -> Result<IndexType, GraphError>;

    /// Returns the edges of this graph.
    fn edges(&self) -> Vec<(IndexType, IndexType)>;

    /// Returns true if the edge exists, or false otherwise.
    /// Returns MissingNode if either starting or ending nodes of the edge are not in the graph.
    fn has_edge(&self, edge: Edge<IndexType>) -> bool;

    /// Adds a new edge to the graph.
    /// Returns an error if the edge already exists or one of the nodes is missing.
    fn add_edge(&mut self, edge: Edge<IndexType>) -> Result<(), GraphError>;

    /// Removes an edge from the graph.
    fn remove_edge(&mut self, edge: Edge<IndexType>);
}

pub trait Graph : GenericGraph<usize> {}

pub trait WeightedGraph<Nw, Ew> : GenericWeightedGraph<usize, Nw, Ew> {}
