mod dot;
pub mod svg;

pub use dot::Dot;
pub use svg::SVG;

use crate::graph::WeightedGraph;
use std::fmt::Display;

/// An exporter for weighted graphs.
pub trait Export {
    /// Returns a String representation of the graph according to the export format used.
    fn from_weighted_graph<Nw: Display, Ew: Display>(graph: &dyn WeightedGraph<Nw, Ew>, name: &str) -> String;

    /// Returns a String representation of the graph according to the export format used.
    fn from_usize_weighted_graph(graph: &dyn WeightedGraph<usize, usize>, name: &str) -> String;
}
