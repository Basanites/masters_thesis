#![feature(test, min_specialization, map_into_keys_values, total_cmp, map_first_last, map_try_insert)]
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
    AlgoConfig,
    ExperimentConfig,
    GeneralExperimentConfig,
    GraphCreationConfig,
    // GraphDynamicsConfig,
};
use metaheuristic::Heuristic;

use decorum::R64;
use glob::glob;
use num_traits::real::Real;
use num_traits::{One, Zero};
use std::env;
use std::fs::{create_dir, write, File};
use std::path::Path;

fn two_swap_h1(nw: R64, _ew: R64, _dist_to_start: R64, _elapsed: R64) -> R64 {
    nw
}

fn two_swap_h2(nw: R64, ew: R64, _dist_to_start: R64, _elapsed: R64) -> R64 {
    nw / ew
}

fn aco_h1(nw: R64, _ew: R64, _dist_to_start: R64, _elapsed: R64) -> R64 {
    if nw != R64::zero() {
        R64::one() - R64::one() / nw
    } else {
        R64::zero()
    }
}

fn aco_h2(nw: R64, ew: R64, _dist_to_start: R64, _elapsed: R64) -> R64 {
    if nw != R64::zero() && ew != R64::zero() {
        // R64::one() - R64::one() / (nw / ew)
        nw / ew
    } else {
        R64::zero()
    }
}

fn aco_h3(nw: R64, _ew: R64, dist_to_start: R64, elapsed: R64) -> R64 {
    if nw != R64::zero() && dist_to_start != R64::zero() {
        R64::powf(R64::one() - R64::one() / nw, R64::one() - elapsed)
            * R64::powf(R64::one() / dist_to_start, elapsed)
    } else if nw != R64::zero() {
        R64::powf(R64::one() - R64::one() / nw, R64::one() - elapsed)
    } else {
        R64::zero()
    }
}

fn run_cfg(path: &Path, experiment_location: &str) {
    let two_swap_functions_usize: Vec<(&Heuristic<R64, R64>, &str)> =
        vec![(&two_swap_h1, "h1"), (&two_swap_h2, "h2")];
    let two_swap_functions_geo: Vec<(&Heuristic<R64, R64>, &str)> =
        vec![(&two_swap_h1, "h1"), (&two_swap_h2, "h2")];

    let aco_functions_usize: Vec<(&Heuristic<R64, R64>, &str)> =
        vec![(&aco_h1, "h1"), (&aco_h2, "h2")];
    let aco_functions_geo: Vec<(&Heuristic<R64, R64>, &str)> =
        vec![(&aco_h1, "h1"), (&aco_h2, "h2"), (&aco_h3, "h3")];

    let random_functions_usize: Vec<(&Heuristic<R64, R64>, &str)> = vec![(&aco_h2, "h2")];
    let random_functions_geo: Vec<(&Heuristic<R64, R64>, &str)> = vec![(&aco_h2, "h2")];
    let entry = path;
    let stem = entry.file_stem().unwrap().to_str().unwrap();

    println!("\n---------------------------------------------------");
    println!("Running config {}: ", stem);
    let reader = File::open(entry).unwrap();
    let experiment = serde_yaml::from_reader::<File, ExperimentConfig>(reader);
    let mut experiment = match experiment {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    // update all cfg entries to their full versions
    let general_cfg = GeneralExperimentConfig::Full(experiment.experiment.cfg());
    let algo_cfg = if let Ok(two) = experiment.algorithm.two_swap() {
        AlgoConfig::TwoSwap(two)
    } else if let Ok(mmaco) = experiment.algorithm.mm_aco() {
        AlgoConfig::MMAco(mmaco)
    } else if let Ok(aco) = experiment.algorithm.aco() {
        AlgoConfig::Aco(aco)
    } else if let Ok(random) = experiment.algorithm.random() {
        AlgoConfig::Random(random)
    } else {
        eprintln!("Invalid Algorithm config for {}", entry.to_str().unwrap());
        return;
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
        return;
    };
    // let graph_dynamics_cfg = GraphDynamicsConfig::Full(experiment.graph_dynamics.cfg());

    // write full version to cfg for later usage
    experiment.experiment = general_cfg;
    experiment.algorithm = algo_cfg;
    experiment.graph_creation = graph_creation_cfg;
    // experiment.graph_dynamics = graph_dynamics_cfg;
    let par_string = serde_yaml::to_string(&experiment).unwrap();
    println!("{}", par_string);
    let res = write(entry, par_string.as_bytes());
    if let Err(e) = res {
        eprintln!("{}", e);
    }

    // create directory for log storage
    let log_folder = Path::new(experiment_location).join(stem);
    let _res = create_dir(&log_folder);

    if experiment.algorithm.two_swap().is_ok() {
        if experiment.graph_creation.file().is_ok() {
            for (heuristic, name) in two_swap_functions_geo.iter() {
                println!("Running heuristic {}", name);
                let file = log_folder.join(name);
                let res = DynamicGraphExperiment::run_geopoint_config(
                    &experiment,
                    heuristic,
                    file.to_str().unwrap(),
                );
                if let Err(e) = res {
                    eprintln!("{}", e);
                }
            }
        } else {
            for (heuristic, name) in two_swap_functions_usize.iter() {
                println!("Running heuristic {}", name);
                let file = log_folder.join(name);
                let res = DynamicGraphExperiment::run_usize_config(
                    &experiment,
                    heuristic,
                    file.to_str().unwrap(),
                );
                if let Err(e) = res {
                    eprintln!("{}", e);
                }
            }
        }
    } else if experiment.algorithm.aco().is_ok() || experiment.algorithm.mm_aco().is_ok() {
        if experiment.graph_creation.file().is_ok() {
            for (heuristic, name) in aco_functions_geo.iter() {
                println!("Running heuristic {}", name);
                let file = log_folder.join(name);
                let res = DynamicGraphExperiment::run_geopoint_config(
                    &experiment,
                    heuristic,
                    file.to_str().unwrap(),
                );
                if let Err(e) = res {
                    eprintln!("{}", e);
                }
            }
        } else {
            for (heuristic, name) in aco_functions_usize.iter() {
                println!("Running heuristic {}", name);
                let file = log_folder.join(name);
                let res = DynamicGraphExperiment::run_usize_config(
                    &experiment,
                    heuristic,
                    file.to_str().unwrap(),
                );
                if let Err(e) = res {
                    eprintln!("{}", e);
                }
            }
        }
    } else if experiment.algorithm.random().is_ok() {
        if experiment.graph_creation.file().is_ok() {
            for (heuristic, name) in random_functions_geo.iter() {
                println!("Running heuristic {}", name);
                let file = log_folder.join(name);
                let res = DynamicGraphExperiment::run_geopoint_config(
                    &experiment,
                    heuristic,
                    file.to_str().unwrap(),
                );
                if let Err(e) = res {
                    eprintln!("{}", e);
                }
            }
        } else {
            for (heuristic, name) in random_functions_usize.iter() {
                println!("Running heuristic {}", name);
                let file = log_folder.join(name);
                let res = DynamicGraphExperiment::run_usize_config(
                    &experiment,
                    heuristic,
                    file.to_str().unwrap(),
                );
                if let Err(e) = res {
                    eprintln!("{}", e);
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut experiment_location = "./experiments";

    if args.len() > 1 {
        let path = Path::new(&args[1]);
        if args.len() > 2 {
            experiment_location = &args[2];
        }
        run_cfg(&path, &experiment_location);
    } else {
        for entry in glob(format!("{}/*.yaml", experiment_location).as_str())
            .expect("Failed to read glob pattern")
        {
            run_cfg(&entry.unwrap(), &experiment_location);
        }
    }
}
