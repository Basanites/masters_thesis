use super::super::WeightedGraph;
use super::Export;
use std::fmt::Display;

/// Empty struct. Only implements GraphExport trait functionalities.
#[allow(dead_code)]
pub struct Dot {}

impl Export for Dot {
    /// Returns a string representing the graph in Graphviz dot format.
    /// The weights are just added as a label for the corresponding node / edge.
    fn from_weighted_graph<Nw: Display, Ew: Display>(graph: &impl WeightedGraph<Nw, Ew>, name: &str) -> String {
        let mut out = String::from(format!("digraph {} {{\n", name));

        for node in graph.nodes() {
            // This can never panic, because nodes only gets the nodes with a weight attached.
            let weight = graph.node_weight(node).unwrap();

            out.push_str(format!("\t{} [label=\"{}\"]\n", node, weight).as_str());
        }

        for edge in graph.edges() {
            // This can never panic, because edges only gets the edges with a weight attached.
            let weight = graph.edge_weight(edge).unwrap();

            out.push_str(format!("\t{} -> {} [label=\"{}\"]\n", edge.0, edge.1, weight).as_str());
        }

        out.push_str("}");
        out
    }

    /// Returns a string representing the graph in Graphviz dot format.
    /// The edge weights are used to set the weight parameter of the edge in Graphviz.
    /// This makes edges with a low weight more likely to be short than ones with a high weight.
    /// All weights are also added as label for the node / edge.
    fn from_usize_weighted_graph(graph: &impl WeightedGraph<usize, usize>, name: &str) -> String {
        let mut out = String::from(format!("digraph {} {{\n", name));

        for node in graph.nodes() {
            // This can never panic, because nodes only gets the nodes with a weight attached.
            let weight = graph.node_weight(node).unwrap();

            out.push_str(format!("\t{} [label=\"{}\"]\n", node, weight).as_str());
        }

        let mut max_len = 1;
        for edge in graph.edges() {
            // This can never panic, because edges only gets the edges with a weight attached.
            let weight = graph.edge_weight(edge).unwrap();
            if weight > &max_len {
                max_len = weight.clone();
            }
        }

        for edge in graph.edges() {
            // This can never panic, because edges only gets the edges with a weight attached.
            let weight = graph.edge_weight(edge).unwrap().clone() as f64;

            out.push_str(format!("\t{} -> {} [weight={} label={}]\n", edge.0, edge.1, max_len as f64 / weight, weight).as_str());
        }

        out.push_str("}");
        out
    }
}