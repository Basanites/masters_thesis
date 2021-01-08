use std::fs::File;
use std::io::Write;
use osmpbfreader::OsmPbfReader;

mod graph;
use graph::{GenericWeightedGraph, WeightedGraph};
use graph::export::{SVG};
use graph::import::import_pbf;
mod geo;
use geo::GeoPoint;
mod util;
use util::Point;



fn main() -> std::io::Result<()> {
    let mapped_graph = import_pbf("res/Leipzig_rough_center.osm.pbf");

    // println!("{:?}", mapped_graph.edges());
    // println!("{:?}", mapped_graph.size());
    // println!("{:?}", mapped_graph.nodes());
    // println!("{:?}\n", mapped_graph.order());
    for edge in mapped_graph.edges().iter() {
       println!("Edge {:?} takes {:?} minutes", edge, mapped_graph.edge_weight(*edge).unwrap());
    }

    let svg_exporter = SVG {
        width: 2000,
        height: 1000,
        padding: 50,
    };

    // println!("{}", svg_exporter.from_coordinate_graph(&mapped_graph as &dyn WeightedGraph<(Point, usize), usize>, "Leipzig"));
    let mut out_file = File::create("graph_out.svg").expect("Error creating file");
    out_file.write_all(svg_exporter.export_coordinate_graph(&mapped_graph as &dyn WeightedGraph<(Point, usize), f64>, "Leipzig").as_bytes()).expect("Error writing to file");

    Ok(())
}
