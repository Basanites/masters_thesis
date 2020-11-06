use super::super::WeightedGraph;

/// A Generator for weighted graphs.
pub trait Generate<Nw, Ew> {
    /// Generates a boxed weighted graph with node weights Nw and edge weights Ew.
    fn generate(&self) -> Box<dyn WeightedGraph<Nw, Ew>>;
}