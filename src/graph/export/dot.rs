use crate::graph::WeightedGraph;
use super::Export;
use std::fmt::Display;

/// Empty struct. Only implements GraphExport trait functionalities.
#[allow(dead_code)]
pub struct Dot {}

impl Export for Dot {
    /// Returns a string representing the graph in Graphviz dot format.
    /// The weights are just added as a label for the corresponding node / edge.
    fn from_weighted_graph<Nw: Display, Ew: Display>(graph: &dyn WeightedGraph<Nw, Ew>, name: &str) -> String {
        let mut out = format!("digraph {} {{\n", name);

        for (node, weight) in graph.iter_nodes() {
            out.push_str(format!("\t{} [label=\"{}\"]\n", node, weight).as_str());
        }

        for (edge, weight) in graph.iter_edges() {
            out.push_str(format!("\t{} -> {} [label=\"{}\"]\n", edge.0, edge.1, weight).as_str());
        }

        out.push('}');
        out
    }

    /// Returns a string representing the graph in Graphviz dot format.
    /// The edge weights are used to set the weight parameter of the edge in Graphviz.
    /// This makes edges with a low weight more likely to be short than ones with a high weight.
    /// All weights are also added as label for the node / edge.
    fn from_usize_weighted_graph(graph: &dyn WeightedGraph<usize, usize>, name: &str) -> String {
        let mut out = format!("digraph {} {{\n", name);

        for (node, weight) in graph.iter_nodes() {
            out.push_str(format!("\t{} [label=\"{}\"]\n", node, weight).as_str());
        }

        let mut max_len = 1;
        for (_, weight) in graph.iter_edges() {
            if weight > &max_len {
                max_len = *weight;
            }
        }

        for (edge, weight) in graph.iter_edges() {
            let weight = *weight as f64;

            out.push_str(format!("\t{} -> {} [weight={} label={}]\n", edge.0, edge.1, max_len as f64 / weight, weight).as_str());
        }

        out.push('}');
        out
    }
}