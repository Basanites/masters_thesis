#![feature(test, min_specialization)]
#![allow(dead_code, unused_imports)]
mod experiment;
mod geo;
mod graph;
mod metaheuristic;
mod rng;
mod util;

use experiment::{AlgoConfig, NoStatTwoSwapExperiment, TwoSwapExperiment};
use geo::GeoPoint;
use graph::export::Svg;
use graph::import::import_pbf;
use graph::{Edge, GenericWeightedGraph, WeightedGraph};
use metaheuristic::{two_swap, Metaheuristic, ProblemInstance, TwoSwap};

use csv::Writer;
use glob::glob;
use std::fs::{create_dir, write, File};
use std::path::Path;

fn two_swap_h1<IndexType>(nw: f64, _ew: f64, _id: IndexType, _elapsed: f64) -> f64 {
    nw
}

fn two_swap_h2<IndexType>(nw: f64, ew: f64, _id: IndexType, _elapsed: f64) -> f64 {
    nw / ew
}

fn main() {
    let experiment_location = "./experiments";
    let mut two_swap_functions_usize: Vec<(&dyn Fn(f64, f64, usize, f64) -> f64, &str)> =
        Vec::new();
    two_swap_functions_usize.push((&two_swap_h1, "h1"));
    two_swap_functions_usize.push((&two_swap_h2, "h2"));
    let mut two_swap_functions_geo: Vec<(&dyn Fn(f64, f64, GeoPoint, f64) -> f64, &str)> =
        Vec::new();
    two_swap_functions_geo.push((&two_swap_h1, "h1"));
    two_swap_functions_geo.push((&two_swap_h2, "h2"));
    let graph = graph::MatrixGraph::new_usize_indexed(
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

    for entry in glob(format!("{}/*.yaml", experiment_location).as_str())
        .expect("Failed to read glob pattern")
    {
        let path = entry.unwrap();
        let entry = path.as_path();
        let stem = entry.file_stem().unwrap().to_str().unwrap();
        println!("{}", stem);
        // create directory for logging.
        // errors if exists, but we don't care about that.
        let res = create_dir(format!("{}/{}", experiment_location, stem).as_str());
        if let Err(e) = res {
            eprintln!("{}", e);
        }

        let mut reader = File::open(entry).unwrap();
        let experiment = serde_yaml::from_reader::<File, AlgoConfig>(reader);
        let experiment = match experiment {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        if let Ok(params) = experiment.two_swap() {
            let par_string = serde_yaml::to_string(&params).unwrap();

            // update yaml file to include all configuration options
            let res = write(entry, par_string.as_bytes());
            if let Err(e) = res {
                eprintln!("{}", e);
            }

            // run TwoSwap on the specified parameters and all heuristics and graphs
            for (heuristic, name) in two_swap_functions_usize.iter() {
                let log_location = path
                    .parent()
                    .unwrap()
                    .join(format!("{}_{}.csv", stem, name));
                let file_writer = File::create(&log_location).unwrap();
                let mut optimizer = TwoSwap::new(
                    ProblemInstance::new(&graph, 0, 100.0),
                    two_swap::Params::new(heuristic),
                    two_swap::Supervisor::new(
                        params.aggregation_rate,
                        Writer::from_writer(file_writer),
                    ),
                );
            }
        } else if let Ok(params) = experiment.aco() {
            let par_string = serde_yaml::to_string(&params).unwrap();

            // update yaml file to include all configuration options
            let res = write(entry, par_string.as_bytes());
            if let Err(e) = res {
                eprintln!("{}", e);
            }
        }
    }
    // let graph = graph::MatrixGraph::new_usize_indexed(
    //     vec![0.0, 0.8, 12.0, 7.0, 2.5],
    //     vec![
    //         (0, 1, 12.0),
    //         (0, 3, 2.0),
    //         (1, 0, 7.0),
    //         (1, 2, 16.0),
    //         (1, 3, 1.5),
    //         (2, 1, 13.5),
    //         (2, 4, 23.0),
    //         (3, 0, 8.1),
    //         (3, 1, 27.0),
    //         (3, 4, 7.5),
    //         (4, 1, 7.0),
    //         (4, 2, 12.0),
    //         (4, 3, 7.5),
    //     ],
    // )
    // .unwrap();

    // let eval: fn(f64, f64, usize, f64) -> f64 = |nw, _, _, _| nw;
    // let mut optimizer = TwoSwap::new(
    //     ProblemInstance::new(&graph, 0, 100.0),
    //     two_swap::Params::new(eval),
    //     two_swap::Supervisor::default(),
    // );
    // println!("{:?}", optimizer.current_solution());
    // for _ in 1..5 {
    //     let val = optimizer.next();
    //     if val.is_some() {
    //         println!("{:?}", optimizer.current_solution());
    //     } else {
    //         break;
    //     }
    // }

    // optimizer.supervisor.aggregate_receive();

    /////////////////////////////////////////////////////////////////////////////////////

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
}
