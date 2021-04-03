use csv::Writer;
use decorum::R64;
use indicatif::ProgressIterator;
use num_traits::Zero;
use oorandom::Rand64;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::hash::Hash;

use crate::experiment_config::{ExperimentConfig, ExperimentConfigError, GraphDynamicsConfig};
use crate::geo::GeoPoint;
use crate::graph::generate::{ErdosRenyi, Generate, Grid};
use crate::graph::import::{import_pbf, ImportError};
use crate::graph::{Edge, GenericWeightedGraph, MatrixGraph};
use crate::metaheuristic::{
    aco, random_search, two_swap, Aco, Heuristic, Metaheuristic, ProblemInstance, RandomSearch,
    TwoSwap,
};
use crate::rng::rng64;
use crate::util::SmallVal;

pub struct DynamicGraphExperiment {}

impl DynamicGraphExperiment {
    pub fn run_geopoint_config(
        config: &ExperimentConfig,
        heuristic: &Heuristic<GeoPoint, R64, R64>,
        filename: &str,
    ) -> Result<(), ExperimentConfigError> {
        if config.experiment.cfg().finished {
            return Ok(());
        }

        if let Ok(f) = config.graph_creation.file() {
            let mut rng = rng64(f.seed as u128);
            let delta = f.nw_range.1 - f.nw_range.0;
            let mut value_gen = || R64::from_inner(rng.rand_float() * delta + f.nw_range.0);
            let pbf = import_pbf(f.filename.as_str(), &mut value_gen);
            match pbf {
                Err(ImportError::MissingFile(msg)) => {
                    Err(ExperimentConfigError::InvalidGraphConfig(msg))
                }
                Ok(graph) => Self::run_experiment::<GeoPoint>(
                    config,
                    heuristic,
                    graph,
                    filename,
                    &mut value_gen,
                    None,
                ),
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
        heuristic: &Heuristic<usize, R64, R64>,
        filename: &str,
    ) -> Result<(), ExperimentConfigError> {
        if config.experiment.cfg().finished {
            return Ok(());
        }

        if let Ok(grid) = config.graph_creation.grid() {
            let rc = RefCell::new(rng64(grid.seed as u128));
            let nw_delta = grid.nw_range.1 - grid.nw_range.0;
            let mut nw_gen = || {
                let mut rng = rc.borrow_mut();
                if rng.rand_float() < grid.node_weight_probability {
                    R64::from_inner(rng.rand_float() * nw_delta + grid.nw_range.0)
                } else {
                    R64::zero()
                }
            };
            let ew_delta = grid.ew_range.1 - grid.ew_range.0;
            let mut ew_gen =
                || R64::from_inner(rc.borrow_mut().rand_float() * ew_delta + grid.ew_range.0);
            let mut grid_gen = Grid::new(
                (grid.size.0 as usize, grid.size.1 as usize),
                &mut nw_gen,
                &mut ew_gen,
            );
            let graph = grid_gen.generate();
            graph.shortest_paths(0);

            //nw_gen is reinitialized here, because we only want it to always create a value now
            let mut nw_gen = || {
                R64::from_inner(
                    rc.borrow_mut().rand_float() * nw_delta + grid.nw_range.0 + f64::small(),
                )
            };
            Self::run_experiment(
                config,
                heuristic,
                graph,
                filename,
                &mut nw_gen,
                Some(&mut ew_gen),
            )
        } else if let Ok(er) = config.graph_creation.erdos_renyi() {
            let rc = RefCell::new(rng64(er.seed as u128));
            let nw_delta = er.nw_range.1 - er.nw_range.0;
            let mut nw_gen =
                || R64::from_inner(rc.borrow_mut().rand_float() * nw_delta + er.nw_range.0);
            let ew_delta = er.ew_range.1 - er.ew_range.0;
            let mut ew_gen =
                || R64::from_inner(rc.borrow_mut().rand_float() * ew_delta + er.ew_range.0);
            let mut er_gen = ErdosRenyi::new(
                er.size as usize,
                er.connection_probability,
                &mut nw_gen,
                &mut ew_gen,
            );
            let graph = er_gen.generate();
            Self::run_experiment(
                config,
                heuristic,
                graph,
                filename,
                &mut nw_gen,
                Some(&mut ew_gen),
            )
        } else {
            Err(ExperimentConfigError::InvalidGraphConfig(
                "usize indexed Graphs are not implemented yet".to_string(),
            ))
        }
    }

    fn run_experiment<IndexType: 'static + Clone + Hash + Copy + Eq + Debug + Display + Ord>(
        config: &ExperimentConfig,
        heuristic: &Heuristic<IndexType, R64, R64>,
        graph: MatrixGraph<IndexType, R64, R64>,
        filename: &str,
        nw_generator: &mut dyn FnMut() -> R64,
        ew_generator: Option<&mut dyn FnMut() -> R64>,
    ) -> Result<(), ExperimentConfigError> {
        let experiment_cfg = config.experiment.cfg();
        let g_nodes = graph.node_ids();
        let mut start_rng = rng64(experiment_cfg.seed as u128);
        let start_node = g_nodes[(start_rng.rand_float() * g_nodes.len() as f64) as usize];
        let mut o_nodes: HashMap<IndexType, R64> = graph
            .iter_nodes()
            .map(|(id, weight)| (id, *weight))
            .collect();
        let mut o_edges: HashMap<Edge<IndexType>, R64> = graph
            .iter_edges()
            .map(|(id, weight)| (id, *weight))
            .collect();
        let graph_rc = RefCell::new(graph);
        let dynamics_cfg = config.graph_dynamics.cfg();
        let mut dyn_rng = rng64(dynamics_cfg.seed as u128);
        let instance = ProblemInstance::new(
            &graph_rc,
            start_node,
            R64::from_inner(experiment_cfg.max_time),
        );
        let fw = File::create(filename).unwrap();

        let mut ew_gen = ew_generator;
        if let Ok(aco_cfg) = config.algorithm.aco() {
            let inv_shortest_paths = graph_rc.borrow().inv_shortest_paths(start_node);
            let params = aco::Params::new(
                heuristic,
                aco_cfg.alpha,
                aco_cfg.beta,
                aco_cfg.rho,
                Some(aco_cfg.seed as u128),
                aco_cfg.ant_count,
                inv_shortest_paths,
            );
            let supervisor =
                aco::Supervisor::new(experiment_cfg.aggregation_rate, Writer::from_writer(fw));
            let mut aco_algo = Aco::new(instance, params, supervisor);

            for _ in (0..aco_cfg.iterations).progress() {
                // if i % dynamics_cfg.change_after_i == 0 {
                //     change_graph(
                //         &graph_rc,
                //         &config.graph_dynamics,
                //         &mut dyn_rng,
                //         nw_generator,
                //         match ew_gen {
                //             Some(ref mut gen) => Some(gen),
                //             _ => None,
                //         },
                //         &mut o_nodes,
                //         &mut o_edges,
                //     );

                //     inv_shortest_paths = graph_rc.borrow().inv_shortest_paths(start_node);
                //     aco_algo.set_inv_shortest_paths(inv_shortest_paths)
                // }
                aco_algo.single_iteration();
            }
            aco_algo.supervisor.aggregate_receive();
        } else if config.algorithm.two_swap().is_ok() {
            let params = two_swap::Params::new(heuristic);
            let supervisor =
                two_swap::Supervisor::new(experiment_cfg.aggregation_rate, Writer::from_writer(fw));
            let mut two_swap_algo = TwoSwap::new(instance, params, supervisor);

            let mut i = 0;
            while two_swap_algo.single_iteration().is_some() {
                // if i % dynamics_cfg.change_after_i == 0 {
                //     change_graph(
                //         &graph_rc,
                //         &config.graph_dynamics,
                //         &mut dyn_rng,
                //         nw_generator,
                //         match ew_gen {
                //             Some(ref mut gen) => Some(gen),
                //             _ => None,
                //         },
                //         &mut o_nodes,
                //         &mut o_edges,
                //     );
                // }
                i += 1;
            }
            println!("Took {} iterations", i);
            two_swap_algo.supervisor.aggregate_receive();
        } else if let Ok(random_cfg) = config.algorithm.random() {
            let inv_shortest_paths = graph_rc.borrow().inv_shortest_paths(start_node);
            let params =
                random_search::Params::new(heuristic, &inv_shortest_paths, random_cfg.seed as u128);
            let supervisor = random_search::Supervisor::new(
                experiment_cfg.aggregation_rate,
                Writer::from_writer(fw),
            );
            let mut random_algo = RandomSearch::new(instance, params, supervisor);
            for _ in (0..random_cfg.iterations).progress() {
                random_algo.single_iteration();
            }
            random_algo.supervisor.aggregate_receive();
        } else {
            return Err(ExperimentConfigError::InvalidGraphConfig(
                "No valid Algorithm config supplied.".to_string(),
            ));
        }

        Ok(())
    }
}

fn change_graph<IndexType: 'static + Clone + Hash + Copy + Eq + Debug + Display + Ord>(
    graph: &RefCell<MatrixGraph<IndexType, R64, R64>>,
    dynamics_cfg: &GraphDynamicsConfig,
    rng: &mut Rand64,
    nw_generator: &mut dyn FnMut() -> R64,
    ew_generator: Option<&mut dyn FnMut() -> R64>,
    original_node_weights: &mut HashMap<IndexType, R64>,
    original_edge_weights: &mut HashMap<Edge<IndexType>, R64>,
) {
    let dynamics_cfg = dynamics_cfg.cfg();

    // determine which nodes will be changed
    let mut change_nodes = Vec::new();
    for nid in graph.borrow().iter_node_ids() {
        if rng.rand_float() < dynamics_cfg.node_change_probability {
            change_nodes.push(nid);
        }
    }

    // determine which edges will be changed
    let mut change_edges = Vec::new();
    for eid in graph.borrow().iter_edge_ids() {
        if rng.rand_float() < dynamics_cfg.edge_change_probability {
            change_edges.push(eid);
        }
    }

    let mut mut_graph = graph.borrow_mut();
    // change nodes
    for nid in change_nodes {
        // this should always contain a value, since all nodes in our graph should be initialized with a min value
        if let (&c_val, Some(&o_val)) = (
            mut_graph.node_weight(nid).unwrap(),
            original_node_weights.get(&nid),
        ) {
            // if we already have a value we reset it to 0 otherwise we take the original value and add onto it.
            // if the original value was the min value we create a new original value for this node and add onto it.
            if c_val > R64::small() {
                mut_graph.change_node(nid, R64::small());
            } else if o_val > R64::small() {
                let n_val = o_val + o_val * rng.rand_float() * dynamics_cfg.node_change_intensity;
                mut_graph.change_node(nid, n_val);
            } else {
                let p_val = (nw_generator)();
                original_node_weights.insert(nid, p_val);
                let n_val = p_val + p_val * rng.rand_float() * dynamics_cfg.node_change_intensity;
                mut_graph.change_node(nid, n_val);
            }
        }
    }

    // change edges
    let mut ew_gen = ew_generator;
    for eid in change_edges {
        let mut previous_val = R64::zero();
        if let Some(&val) = original_edge_weights.get(&eid) {
            if val > f64::small() {
                previous_val = val;
            } else {
                match ew_gen {
                    Some(ref mut gen) => {
                        previous_val = (gen)();
                        original_edge_weights.insert(eid, previous_val);
                    }
                    _ => {
                        previous_val = val;
                    }
                };
            }
        } else if let Some(ref mut gen) = ew_gen {
            previous_val = (gen)();
            original_edge_weights.insert(eid, previous_val);
        }

        let val =
            previous_val + previous_val * rng.rand_float() * dynamics_cfg.edge_change_intensity;
        mut_graph.change_edge(eid, val).unwrap();
    }

    let mut i = 0;
    for node in mut_graph.iter_nodes() {
        if node.1 > &R64::small() {
            i += 1;
        }
    }
    println!("{} nodes with weight", i);
}
