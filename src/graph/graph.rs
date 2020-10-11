pub use super::{ GraphError };

pub type Edge = (usize, usize);

/// An unweighted, possibly directed Graph.
pub trait Graph {
    /// Returns true if there are no nodes, or false otherwise.
    fn is_empty(&self) -> bool;

    /// Returns the number of nodes in this graph.
    fn order(&self) -> usize;

    /// Returns the number of edges in this graph.
    fn size(&self) -> usize;

    /// Returns the nodes of this graph.
    fn nodes(&self) -> Vec<usize>;

    /// Iterates the neighbors of the node with id.
    /// Returns an error if node is not in graph.
    fn neighbors(&self, id: usize) -> Result<Vec<usize>, GraphError>;

    /// Returns true if node with id is a member, or false otherwise.
    fn has_node(&self, id: usize) -> bool;

    /// Adds a new node to the graph.
    /// Returns an error if a node with the same id already exists.
    fn add_node(&mut self, id: usize) -> Result<(), GraphError>;

    /// Removes a node from the graph.
    /// This in turn means all edges from or to this node will be removed.
    fn remove_node(&mut self, id: usize);

    /// Returns the count of neighbors at node with given id. 
    /// Returns an error if the node is not in the graph.
    fn degree(&self, id: usize) -> Result<usize, GraphError>;

    /// Returns the edges of this graph.
    fn edges(&self) -> Vec<(usize, usize)>;

    /// Returns true if the edge exists, or false otherwise.
    /// Returns MissingNode if either starting or ending nodes of the edge are not in the graph.
    fn has_edge(&self, edge: Edge) -> bool;

    /// Adds a new edge to the graph. 
    /// Returns an error if the edge already exists or one of the nodes is missing.
    fn add_edge(&mut self, edge: Edge) -> Result<(), GraphError>;

    /// Removes an edge from the graph.
    fn remove_edge(&mut self, edge: Edge);
}

/// A weighted and possibly directed Graph.
pub trait WeightedGraph<Nw, Ew> {
    /// Returns true if there are no nodes, or false otherwise.
    fn is_empty(&self) -> bool;

    /// Returns the number of nodes in this graph.
    fn order(&self) -> usize;

    /// Returns the number of edges in this graph.
    fn size(&self) -> usize;

    /// Returns the nodes of this graph.
    fn nodes(&self) -> Vec<usize>;

    /// Returns the weight of node with id.
    fn node_weight(&self, id: usize) -> Result<&Nw, GraphError>;

    /// Iterates the neighbors of the node with id.
    /// Returns an error if node is not in graph.
    fn neighbors(&self, id: usize) -> Result<Vec<usize>, GraphError>;

    /// Returns true if node with id is a member, or false otherwise.
    fn has_node(&self, id: usize) -> bool;

    /// Adds a new node with weight to the graph.
    /// Returns an error if a node with the same id already exists.
    fn add_node(&mut self, id: usize, weight: Nw) -> Result<(), GraphError>;

    /// Removes a weighted node from the graph.
    /// This in turn means all edges from or to this node will be removed.
    fn remove_node(&mut self, id: usize);

    /// Returns the count of neighbors at node with given id. 
    /// Returns an error if the node is not in the graph.
    fn degree(&self, id: usize) -> Result<usize, GraphError>;

    /// Returns the edges of this graph.
    fn edges(&self) -> Vec<(usize, usize)>;

    /// Returns the weight of an edge.
    fn edge_weight(&self, edge: Edge) -> Result<&Ew, GraphError>;

    /// Returns true if the edge exists, or false otherwise.
    /// Returns MissingNode if either starting or ending nodes of the edge are not in the graph.
    fn has_edge(&self, edge: Edge) -> bool;

    /// Adds a new weighted edge to the graph. 
    /// Returns an error if the edge already exists or one of the nodes is missing.
    fn add_edge(&mut self, edge: Edge, weight: Ew) -> Result<(), GraphError>;

    /// Removes a weighted edge from the graph.
    fn remove_edge(&mut self, edge: Edge);
}