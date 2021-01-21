#[allow(dead_code)]
mod erdos_renyi;
#[allow(dead_code)]
mod grid;
#[allow(dead_code)]
mod stochastic_block;

pub use erdos_renyi::ErdosRenyi;
pub use grid::Grid;
pub use stochastic_block::StochasticBlock;

use crate::graph::WeightedGraph;

/// A Generator for weighted graphs.
pub trait Generate<Nw, Ew> {
    /// Generates a boxed weighted graph with node weights Nw and edge weights Ew.
    fn generate(&self) -> Box<dyn WeightedGraph<Nw, Ew>>;
}
