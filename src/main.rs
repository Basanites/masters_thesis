#![feature(test)]
mod geo;
mod graph;
mod metaheuristic;
mod util;

use graph::export::SVG;
use graph::import::import_pbf;
use graph::{Edge, GenericWeightedGraph, WeightedGraph};
use metaheuristic::TwoSwap;
use util::Point;

use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let graph = graph::regular::MatrixGraph::new(
        vec![0.0, 0.8, 12.0, 7.0, 2.5],
        vec![
            (0, 1, 12.0),
            (0, 3, 2.0),
            (1, 0, 7.0),
            (1, 2, 16.0),
            (1, 3, 1.5),
            (2, 1, 13.5),
            (2, 4, 23.0),
            (3, 0, 8.1),
            (3, 1, 27.0),
            (3, 4, 7.5),
            (4, 1, 7.0),
            (4, 2, 12.0),
            (4, 3, 7.5),
        ],
    )
    .unwrap();

    let eval: fn(f64, f64) -> f64 = |nw, ew| nw;
    let mut optimizer = TwoSwap::new(Box::new(graph), 0, 100.0, &eval);
    println!("{:?}", optimizer.current_solution());
    for _ in 1..5 {
        let val = optimizer.next();
        if val.is_some() {
            println!("{:?}", optimizer.current_solution());
        } else {
            break;
        }
    }

    // let mapped_graph = import_pbf("res/Leipzig_rough_center.osm.pbf");
    //
    // let mut start_points = Vec::new();
    // for (from, to) in mapped_graph.iter_edge_ids() {
    //     if mapped_graph.has_edge((to, from)) {
    //         start_points.push(from);
    //     }
    // }
    //
    // let eval: fn((Point, usize), f64) -> f64 = |nw, ew| nw.1 as f64 / ew;
    // for start_point in start_points {
    //     let mut optimizer = TwoSwap::new(Box::new(mapped_graph.clone()), start_point, 100.0, &eval);
    //
    //     let mut val;
    //     for i in 1..10 {
    //         val = optimizer.next();
    //         if val.is_some() {
    //             println!("{:?}", val);
    //         } else {
    //             break
    //         }
    //     }
    //     println!();
    // }
    //
    // let svg_exporter = SVG {
    //     width: 2000,
    //     height: 1000,
    //     padding: 50,
    // };
    //
    // println!("{}", svg_exporter.from_coordinate_graph(&mapped_graph as &dyn WeightedGraph<(Point, usize), usize>, "Leipzig"));
    // let mut out_file = File::create("graph_out.svg").expect("Error creating file");
    // out_file.write_all(svg_exporter.export_coordinate_graph(&mapped_graph as &dyn WeightedGraph<(Point, usize), f64>, "Leipzig").as_bytes()).expect("Error writing to file");

    Ok(())
}
