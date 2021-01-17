#[allow(dead_code)]
mod erdos_renyi;
#[allow(dead_code)]
mod stochastic_block;
#[allow(dead_code)]
mod grid;

pub use erdos_renyi::ErdosRenyi;
pub use stochastic_block::StochasticBlock;
pub use grid::Grid;

use crate::graph::WeightedGraph;

/// A Generator for weighted graphs.
pub trait Generate<Nw, Ew> {
    /// Generates a boxed weighted graph with node weights Nw and edge weights Ew.
    fn generate(&self) -> Box<dyn WeightedGraph<Nw, Ew>>;
}
