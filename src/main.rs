#![feature(test, min_specialization, map_into_keys_values)]
#![allow(dead_code)]
mod dynamic_graph_experiment;
mod experiment_config;
mod geo;
mod graph;
mod metaheuristic;
mod rng;
mod util;

use dynamic_graph_experiment::DynamicGraphExperiment;
use experiment_config::{
    AlgoConfig, ExperimentConfig, GeneralExperimentConfig, GraphCreationConfig, GraphDynamicsConfig,
};
use geo::GeoPoint;

use glob::glob;
use std::fs::{create_dir, write, File};
use std::io::ErrorKind;

type UsizeHeuristic = dyn Fn(f64, f64, usize, f64) -> f64;
type GeoPointHeuristic = dyn Fn(f64, f64, GeoPoint, f64) -> f64;

fn two_swap_h1<IndexType>(nw: f64, _ew: f64, _id: IndexType, _elapsed: f64) -> f64 {
    nw
}

fn two_swap_h2<IndexType>(nw: f64, ew: f64, _id: IndexType, _elapsed: f64) -> f64 {
    nw / ew
}

fn aco_h1<IndexType>(nw: f64, _ew: f64, _id: IndexType, _elapsed: f64) -> f64 {
    nw
}

fn aco_h2<IndexType>(nw: f64, ew: f64, _id: IndexType, _elapsed: f64) -> f64 {
    nw / ew
}

fn main() {
    let experiment_location = "./experiments";
    let two_swap_functions_usize: Vec<(&UsizeHeuristic, &str)> =
        vec![(&two_swap_h1, "h1"), (&two_swap_h2, "h2")];
    let two_swap_functions_geo: Vec<(&GeoPointHeuristic, &str)> =
        vec![(&two_swap_h1, "h1"), (&two_swap_h2, "h2")];

    let aco_functions_usize: Vec<(&UsizeHeuristic, &str)> = vec![(&aco_h1, "h1"), (&aco_h2, "h2")];
    let aco_functions_geo: Vec<(&GeoPointHeuristic, &str)> = vec![(&aco_h1, "h1"), (&aco_h2, "h2")];

    for entry in glob(format!("{}/*.yaml", experiment_location).as_str())
        .expect("Failed to read glob pattern")
    {
        let path = entry.unwrap();
        let entry = path.as_path();
        let stem = entry.file_stem().unwrap().to_str().unwrap();
        println!("Running Config {}: ", stem);
        // create directory for logging.
        // errors if exists, but we don't care about that.
        let res = create_dir(format!("{}/{}", experiment_location, stem).as_str());
        match res {
            Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {}
            Err(e) => eprintln!("{}", e),
            _ => {}
        };

        let reader = File::open(entry).unwrap();
        let experiment = serde_yaml::from_reader::<File, ExperimentConfig>(reader);
        let mut experiment = match experiment {
            Ok(val) => val,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        // update all cfg entries to their full versions
        let general_cfg = GeneralExperimentConfig::Full(experiment.experiment.cfg());
        let algo_cfg = if let Ok(two) = experiment.algorithm.two_swap() {
            AlgoConfig::TwoSwap(two)
        } else if let Ok(aco) = experiment.algorithm.aco() {
            AlgoConfig::Aco(aco)
        } else {
            eprintln!("Invalid Algorithm config for {}", entry.to_str().unwrap());
            continue;
        };
        let graph_creation_cfg = if let Ok(f) = experiment.graph_creation.file() {
            GraphCreationConfig::File(f)
        } else if let Ok(g) = experiment.graph_creation.grid() {
            GraphCreationConfig::Grid(g)
        } else if let Ok(e) = experiment.graph_creation.erdos_renyi() {
            GraphCreationConfig::ErdosRenyi(e)
        } else {
            eprintln!(
                "Invalid Graph Creation config for {}",
                entry.to_str().unwrap()
            );
            continue;
        };
        let graph_dynamics_cfg = GraphDynamicsConfig::Full(experiment.graph_dynamics.cfg());

        // write full version to cfg for later usage
        experiment.experiment = general_cfg;
        experiment.algorithm = algo_cfg;
        experiment.graph_creation = graph_creation_cfg;
        experiment.graph_dynamics = graph_dynamics_cfg;
        let par_string = serde_yaml::to_string(&experiment).unwrap();
        println!("{}", par_string);
        let res = write(entry, par_string.as_bytes());
        if let Err(e) = res {
            eprintln!("{}", e);
        }

        // create directory for log storage
        let log_folder = path.parent().unwrap().join(stem);
        let _res = create_dir(&log_folder);

        if experiment.algorithm.two_swap().is_ok() {
            if experiment.graph_creation.file().is_ok() {
                for (heuristic, name) in two_swap_functions_geo.iter() {
                    let filename = format!("{}/{}.csv", log_folder.to_str().unwrap(), name);
                    let res = DynamicGraphExperiment::run_geopoint_config(
                        &experiment,
                        heuristic,
                        filename.as_str(),
                    );
                    if let Err(e) = res {
                        eprintln!("{}", e);
                    }
                }
            } else {
                for (heuristic, name) in two_swap_functions_usize.iter() {
                    let filename = format!("{}/{}.csv", log_folder.to_str().unwrap(), name);
                    let res = DynamicGraphExperiment::run_usize_config(
                        &experiment,
                        heuristic,
                        filename.as_str(),
                    );
                    if let Err(e) = res {
                        eprintln!("{}", e);
                    }
                }
            }
        } else if experiment.algorithm.aco().is_ok() {
            if experiment.graph_creation.file().is_ok() {
                for (heuristic, name) in aco_functions_geo.iter() {
                    let filename = format!("{}/{}.csv", log_folder.to_str().unwrap(), name);
                    let res = DynamicGraphExperiment::run_geopoint_config(
                        &experiment,
                        heuristic,
                        filename.as_str(),
                    );
                    if let Err(e) = res {
                        eprintln!("{}", e);
                    }
                }
            } else {
                for (heuristic, name) in aco_functions_usize.iter() {
                    let filename = format!("{}/{}.csv", log_folder.to_str().unwrap(), name);
                    let res = DynamicGraphExperiment::run_usize_config(
                        &experiment,
                        heuristic,
                        filename.as_str(),
                    );
                    if let Err(e) = res {
                        eprintln!("{}", e);
                    }
                }
            }
        }
    }
}
