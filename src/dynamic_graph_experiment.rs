use csv::Writer;
use oorandom::Rand64;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::hash::Hash;

use crate::experiment_config::{ExperimentConfig, ExperimentConfigError, GraphDynamicsConfig};
use crate::geo::GeoPoint;
use crate::graph::generate::{ErdosRenyi, Generate, Grid};
use crate::graph::import::{import_pbf, ImportError};
use crate::graph::{GenericWeightedGraph, MatrixGraph};
use crate::metaheuristic::{
    aco, two_swap, Aco, Heuristic, Metaheuristic, ProblemInstance, TwoSwap,
};
use crate::rng::rng64;

pub struct DynamicGraphExperiment {}

impl DynamicGraphExperiment {
    pub fn run_geopoint_config(
        config: &ExperimentConfig,
        heuristic: &Heuristic<GeoPoint, f64, f64>,
        filename: &str,
    ) -> Result<(), ExperimentConfigError> {
        if config.experiment.cfg().finished {
            return Ok(());
        }

        if let Ok(f) = config.graph_creation.file() {
            let mut rng = rng64(f.seed);
            let delta = f.nw_range.1 - f.nw_range.0;
            let mut value_gen = || rng.rand_float() * delta + f.nw_range.0;
            let pbf = import_pbf(f.filename.as_str(), &mut value_gen);
            match pbf {
                Err(ImportError::MissingFile(msg)) => {
                    Err(ExperimentConfigError::InvalidGraphConfig(msg))
                }
                Ok(graph) => Self::run_experiment::<GeoPoint>(config, heuristic, graph, filename),
                _ => panic!("pbf import threw an undefined error"),
            }
        } else {
            Err(ExperimentConfigError::InvalidGraphConfig(
                "GeoPoint indexed experiments can only be ran on pbf imports yet.".to_string(),
            ))
        }
    }

    pub fn run_usize_config(
        config: &ExperimentConfig,
        heuristic: &Heuristic<usize, f64, f64>,
        filename: &str,
    ) -> Result<(), ExperimentConfigError> {
        if config.experiment.cfg().finished {
            return Ok(());
        }

        if let Ok(grid) = config.graph_creation.grid() {
            let mut actual_rng = rng64(grid.seed);
            let nw_delta = grid.nw_range.1 - grid.nw_range.0;
            let nw_gen = |mut rng: Rand64| rng.rand_float() * nw_delta + grid.nw_range.0;
            let ew_delta = grid.ew_range.1 - grid.ew_range.0;
            let ew_gen = |mut rng: Rand64| rng.rand_float() * ew_delta + grid.ew_range.0;
            let mut grid_gen = Grid::new(grid.size, &nw_gen, &ew_gen, &mut actual_rng);
            let graph = grid_gen.generate();
            Self::run_experiment(config, heuristic, graph, filename)
        } else if let Ok(er) = config.graph_creation.erdos_renyi() {
            let mut actual_rng = rng64(er.seed);
            let nw_delta = er.nw_range.1 - er.nw_range.0;
            let nw_gen = |mut rng: Rand64| rng.rand_float() * nw_delta + er.nw_range.0;
            let ew_delta = er.ew_range.1 - er.ew_range.0;
            let ew_gen = |mut rng: Rand64| rng.rand_float() * ew_delta + er.ew_range.0;
            let mut er_gen = ErdosRenyi::new(
                er.size,
                er.connection_probability,
                &nw_gen,
                &ew_gen,
                &mut actual_rng,
            );
            let graph = er_gen.generate();
            Self::run_experiment(config, heuristic, graph, filename)
        } else {
            Err(ExperimentConfigError::InvalidGraphConfig(
                "usize indexed Graphs are not implemented yet".to_string(),
            ))
        }
    }

    fn run_experiment<IndexType: 'static + Clone + Hash + Copy + Eq + Debug + Display>(
        config: &ExperimentConfig,
        heuristic: &Heuristic<IndexType, f64, f64>,
        graph: MatrixGraph<IndexType, f64, f64>,
        filename: &str,
    ) -> Result<(), ExperimentConfigError> {
        let experiment_cfg = config.experiment.cfg();
        let g_nodes = graph.node_ids();
        let mut start_rng = rng64(experiment_cfg.seed);
        let start_node = g_nodes[(start_rng.rand_float() * g_nodes.len() as f64) as usize];
        let graph_rc = RefCell::new(graph);
        let dynamcis_cfg = config.graph_dynamics.cfg();
        let mut dyn_rng = rng64(dynamcis_cfg.seed);
        let instance = ProblemInstance::new(&graph_rc, start_node, experiment_cfg.max_time);
        let fw = File::create(filename).unwrap();

        if let Ok(aco_cfg) = config.algorithm.aco() {
            let params = aco::Params::new(
                heuristic,
                aco_cfg.alpha,
                aco_cfg.beta,
                aco_cfg.rho,
                Some(aco_cfg.seed),
                aco_cfg.ant_count,
            );
            let supervisor =
                aco::Supervisor::new(experiment_cfg.aggregation_rate, Writer::from_writer(fw));
            let mut aco_algo = Aco::new(instance, params, supervisor);

            let mut i = 0;
            while aco_algo.single_iteration().is_some() {
                if i % dynamcis_cfg.change_after_i == 0 {
                    change_graph(&graph_rc, &config.graph_dynamics, &mut dyn_rng);
                }
                i += 1;
            }
        } else if config.algorithm.two_swap().is_ok() {
            let params = two_swap::Params::new(heuristic);
            let supervisor =
                two_swap::Supervisor::new(experiment_cfg.aggregation_rate, Writer::from_writer(fw));
            let mut two_swap_algo = TwoSwap::new(instance, params, supervisor);

            let mut i = 0;
            while two_swap_algo.single_iteration().is_some() {
                if i % dynamcis_cfg.change_after_i == 0 {
                    change_graph(&graph_rc, &config.graph_dynamics, &mut dyn_rng);
                }
                i += 1;
            }
        } else {
            return Err(ExperimentConfigError::InvalidGraphConfig(
                "No valid Algorithm config supplied.".to_string(),
            ));
        }

        Ok(())
    }
}

fn change_graph<IndexType: 'static + Clone + Hash + Copy + Eq + Debug + Display>(
    graph: &RefCell<MatrixGraph<IndexType, f64, f64>>,
    dynamics_cfg: &GraphDynamicsConfig,
    rng: &mut Rand64,
) {
    let cfg = dynamics_cfg.cfg();

    // determine which nodes will be changed
    let mut change_nodes = Vec::new();
    for nid in graph.borrow().iter_node_ids() {
        if rng.rand_float() > cfg.node_change_probability {
            change_nodes.push(nid);
        }
    }

    // determine which edges will be changed
    let mut change_edges = Vec::new();
    for eid in graph.borrow().iter_edge_ids() {
        if rng.rand_float() > cfg.edge_change_probability {
            change_edges.push(eid);
        }
    }

    let mut mut_graph = graph.borrow_mut();
    // change nodes
    for nid in change_nodes {
        let mut val = *mut_graph.node_weight(nid).unwrap();
        val = val * (rng.rand_float() * 2.0 - 1.0) * cfg.node_change_intensity;
        if val < 0.0 {
            val = 0.0;
        }

        mut_graph.change_node(nid, val);
    }

    // change edges
    for eid in change_edges {
        let mut val = *mut_graph.edge_weight(eid).unwrap();
        val = val * (rng.rand_float() * 2.0 - 1.0) * cfg.edge_change_intensity;
        if val < 0.0 {
            val = 0.0;
        }

        mut_graph.change_edge(eid, val).unwrap();
    }
}
