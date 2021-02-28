#![allow(clippy::map_entry)]
use osmpbfreader::objects::Node;
use osmpbfreader::OsmPbfReader;
use osmpbfreader::{NodeId, OsmId, OsmObj};
use std::collections::{HashMap, HashSet};
use std::fs::File;

use crate::geo::{geodistance_haversine, GeoPoint};
use crate::graph::import::ImportError;
use crate::graph::{GenericWeightedGraph, MatrixGraph};

/// Calculates the distance between two nodes in km.
fn get_node_distance(node_1: &Node, node_2: &Node) -> f64 {
    let p1 = GeoPoint::from_degrees(node_1.lat(), node_1.lon());
    let p2 = GeoPoint::from_degrees(node_2.lat(), node_2.lon());
    geodistance_haversine(p1, p2)
}

/// Calculates the traveltime in minutes for a given distance_map in km.
fn traveltime_from_distance_map(dist_map: &HashMap<String, f64>) -> f64 {
    dist_map
        .iter()
        .map(|(key, val)| -> f64 {
            // speeds are given in km/h so dividing by them returns time in hrs
            let factor = match &key[..] {
                "motorway" => 1.0 / 130.0,
                "primary" => 1.0 / 100.0,
                "secondary" => 1.0 / 90.0,
                "tertiary" => 1.0 / 70.0,
                "residential" => 1.0 / 50.0,
                "living_street" => 1.0 / 30.0,
                _ => 0.5, // if we don't know the street type we just assume 50km/h
            };
            // we need to multiply by 60 to get to minutes
            val * factor * 60.0
        })
        .sum()
}

/// Finds the replacement for a node and updates the distance_map and
/// returns the replacement as well as the according distance map
fn get_node_replacement(
    replacement_map: &HashMap<OsmId, (OsmId, HashMap<String, f64>)>,
    replacement: OsmId,
    dist_map: HashMap<String, f64>,
) -> (OsmId, HashMap<String, f64>) {
    // the updated distance map contains the sum of the replacement distances with the
    // distances it takes to get there.
    let mut updated_dist_map = HashMap::new();
    let replacement_pair = &replacement_map[&replacement];
    let other_dist_map = &replacement_pair.1;
    for key in dist_map.keys().chain(other_dist_map.keys()) {
        if dist_map.contains_key(key) && other_dist_map.contains_key(key) {
            updated_dist_map.insert(key.into(), dist_map[key] + other_dist_map[key]);
        } else if dist_map.contains_key(key) {
            updated_dist_map.insert(key.into(), dist_map[key]);
        } else if other_dist_map.contains_key(key) {
            updated_dist_map.insert(key.into(), other_dist_map[key]);
        }
    }
    (replacement_pair.0, updated_dist_map)
}

/// Finds all nodes which are on a path with just a single connection and
/// updates the replacement map accordingly.
/// Returns true if there have been any new replacements.
fn find_contractable_nodes(
    neighbors: &HashMap<OsmId, HashMap<OsmId, HashMap<String, f64>>>,
    inv_neighbors: &HashMap<OsmId, Vec<OsmId>>,
    replacement_map: &mut HashMap<OsmId, (OsmId, HashMap<String, f64>)>,
    circle_nodes: &mut HashSet<OsmId>,
) -> bool {
    let mut changed = false;

    let mut change_count = 0;
    for (&nid, neighbor_nodes) in neighbors.iter() {
        let start_node = nid;
        let maxits = 500;

        if neighbor_nodes.len() == 1 && inv_neighbors[&nid].len() == 1 {
            let replacement_pair = neighbor_nodes.iter().last().unwrap();
            let mut replacement = *replacement_pair.0;
            let mut dist_map = replacement_pair.1.clone();
            let mut counter = 0;

            // we iterate as deeply, as our replacement map allows
            while replacement_map.contains_key(&replacement) && counter < maxits {
                let new_replacement_pair =
                    get_node_replacement(&replacement_map, replacement, dist_map.clone());
                replacement = new_replacement_pair.0;
                dist_map = new_replacement_pair.1;

                // if we find a new circle we add all its nodes to the circle_nodes
                if replacement == start_node && !circle_nodes.contains(&replacement) {
                    println!("circle detected for node {:?}", replacement);

                    let new_replacement_pair = neighbor_nodes.iter().last().unwrap();
                    let mut replacement = new_replacement_pair.0;
                    while *replacement != start_node {
                        print!(" {:?} to", replacement);
                        circle_nodes.insert(*replacement);
                        replacement = &replacement_map[replacement].0;
                    }
                    println!();
                    circle_nodes.insert(*replacement);
                    changed = true;
                    change_count += 1;
                    break;
                }
                counter += 1;
            }

            // only insert the replacement if it is actually a new discovery
            if !replacement_map.contains_key(&nid) || replacement_map[&nid].0 != replacement {
                replacement_map.insert(nid, (replacement, dist_map));
                changed = true;
                change_count += 1;
            }
        }
        // if the node is part of a 2-way-path we can still contract it to the one it goes to
        // else if neighbor_nodes.len() == 2 {
        //     let mut oid = nid;
        //     for (nid, _) in neighbor_nodes {
        //         for iid in inv_neighbors[nid].iter() {
        //             if iid == &start_node {
        //                 oid = *iid;
        //             }
        //         }
        //     }
        //     // oid has changed, so it is part of such a 2-way situation
        //     if oid != nid {
        //         replacement_map.insert(nid, oid);
        //     }
        // }
    }
    println!("\t{:?} nodes have changed in replacement map", change_count);
    changed
}

/// Contracts all nodes on a single connection path into one endpoint node.
/// The distances for these nodes are updated according to their original distance with many hops in between.
fn contract_nodes(
    nodes: &mut HashMap<OsmId, OsmObj>,
    neighbors: &mut HashMap<OsmId, HashMap<OsmId, HashMap<String, f64>>>,
    inv_neighbors: HashMap<OsmId, Vec<OsmId>>,
) {
    let mut replacement_map: HashMap<OsmId, (OsmId, HashMap<String, f64>)> = HashMap::new();
    let mut circle_nodes = HashSet::<OsmId>::new();
    let mut changed = true;
    // find replacements as long, as the replacement map keeps changing
    let mut i = 0;

    while changed {
        println!(
            "contraction iteration {:?} with {:?} nodes to replace",
            i,
            replacement_map.len()
        );
        changed = find_contractable_nodes(
            neighbors,
            &inv_neighbors,
            &mut replacement_map,
            &mut circle_nodes,
        ); // assignment with side effects == very bad style
        i += 1;
    }

    // remove the nodes which only form a circle
    // which is not connected to the rest of the graph in any way.
    for node in &circle_nodes {
        nodes.remove(node);
        neighbors.remove(node);
        replacement_map.remove(node);
    }

    // replace all occurences of a node with the specified replacement node from the map
    for (_, neighbor_nodes) in neighbors.iter_mut() {
        // find all nodes that need to be replaced
        let mut to_change = Vec::new();
        for &node in neighbor_nodes.keys() {
            if replacement_map.contains_key(&node) {
                to_change.push(node)
            }
        }
        // replace them by removing the old values and inserting the new one
        for id in to_change.iter() {
            neighbor_nodes.remove(id);
            let replacement = replacement_map.get(id).unwrap();
            neighbor_nodes.insert(replacement.0, replacement.1.clone());
        }
    }

    // remove all nodes which got replaced
    for (from, _) in replacement_map.iter() {
        neighbors.remove(from);
        nodes.remove(from);
    }

    println!(
        "The replacement map contains {} items after removing {:?} nodes in a circle",
        replacement_map.len(),
        circle_nodes.len()
    );
}

/// Creates a minimized MatrixGraph from a given pbf file.
/// The nodes are contracted as to not run out of memory for the MatrixGraph.
pub fn import_pbf(
    path: &str,
    nw_gen: &mut dyn FnMut() -> f64,
) -> Result<MatrixGraph<GeoPoint, f64, f64>, ImportError> {
    let mut file_open = File::open(path);
    let mut file;
    match file_open {
        Ok(f) => file = f,
        Err(e) => return Err(ImportError::MissingFile(path.to_string())),
    };

    let mut pbf = OsmPbfReader::new(file);
    let mut neighbors = HashMap::<OsmId, HashMap<OsmId, HashMap<String, f64>>>::new();
    let mut inv_neighbors = HashMap::<OsmId, Vec<OsmId>>::new();
    let mut nodes = HashMap::<OsmId, OsmObj>::new();
    // read all nodes from the pbf to their respective lists.
    // neighbors contain all successors of a node while inv_neighbors contains its predecessors.
    for obj in pbf.iter() {
        let obj = obj.unwrap();
        if obj.is_node() {
            nodes.insert(obj.id(), obj);
        } else if obj.is_way() {
            let mut pid = NodeId(0);
            for (i, &nid) in obj.way().unwrap().nodes.iter().enumerate() {
                if i > 0 {
                    // Loading the nodes from the node array will fail if
                    // they are not listed first in the pbf file.
                    // If the pbf is generated correctly this won't happen.
                    let n_key = OsmId::Node(nid);
                    let n_node = nodes[&n_key].node().unwrap(); // !!!
                    let p_key = OsmId::Node(pid);
                    let p_node = nodes[&p_key].node().unwrap(); // !!!

                    // insert all the predecessors of a node into the hashmap,
                    // creating a new vec of neighbors, if there wasnt one before
                    // This is just a list of neighbors going backwards.
                    // No further information is encoded.
                    // Accessing inv_neighbors[to_key] returns a vec of all node ids pointing at to_id
                    if inv_neighbors.contains_key(&n_key) {
                        inv_neighbors.get_mut(&n_key).unwrap().push(p_key);
                    } else {
                        inv_neighbors.insert(n_key, [p_key].to_vec());
                    }

                    // create a mapping for all neighbors of a node and their respective distances
                    // when using a specific road type.
                    // The map accessed as neighbors[from_key][to_key][road_type] returns
                    // the distance one would travel on that specific road type.
                    // Thus the complete distance would be neighbors[from_key][to_key].values().sum()
                    let road_type = obj.way().unwrap().tags.get("highway").unwrap().to_string();
                    let distance = get_node_distance(p_node, n_node);
                    // println!("distance between {:?} and {:?} is {:?}km", p_node, n_node, distance);

                    if neighbors.contains_key(&p_key) {
                        let neighbor_dists = neighbors.get_mut(&p_key).unwrap();
                        if neighbor_dists.contains_key(&n_key) {
                            neighbor_dists
                                .get_mut(&n_key)
                                .unwrap()
                                .insert(road_type, distance);
                        } else {
                            let mut new_dists = HashMap::new();
                            new_dists.insert(road_type, distance);
                            neighbor_dists.insert(n_key, new_dists);
                        }
                    } else {
                        let mut outer_map = HashMap::new();
                        let mut inner_map = HashMap::new();
                        inner_map.insert(road_type, distance);
                        outer_map.insert(n_key, inner_map);
                        neighbors.insert(p_key, outer_map);
                    }
                }
                pid = nid;
            }
        }
    }

    // initialize all nodes, which were in the nodes array but never appeared in a way
    for id in nodes.keys() {
        if !neighbors.contains_key(id) {
            neighbors.insert(*id, HashMap::new());
        }
        if !inv_neighbors.contains_key(id) {
            inv_neighbors.insert(*id, [].to_vec());
        }
    }

    // contract all nodes on single connection paths into one
    contract_nodes(&mut nodes, &mut neighbors, inv_neighbors);

    // Map node ids from osm to consecutive ids starting at 0
    let mut node_map: HashMap<OsmId, GeoPoint> = HashMap::new();
    for (id, obj) in nodes.iter() {
        if !node_map.contains_key(id) {
            let point = GeoPoint::from_micro_degrees(
                obj.node().unwrap().decimicro_lat,
                obj.node().unwrap().decimicro_lon,
            );
            node_map.insert(*id, point);
        }
    }

    let mut mapped_graph = MatrixGraph::<GeoPoint, f64, f64>::with_size(node_map.len());

    // Insert nodes into the graph with fixed weight 1
    for (_, point) in &node_map {
        // TODO: when logger is here, log this to errorlog
        let _ = mapped_graph.add_node(*point, nw_gen());
    }

    // Insert edges with their weight being the traveltime between each other.
    for (from_id, neighbor_nodes) in neighbors.iter() {
        for (to_id, dist_map) in neighbor_nodes {
            if node_map.contains_key(from_id) && node_map.contains_key(to_id) && from_id != to_id {
                // TODO: when logger is here this needs to go to errorlog
                let _ = mapped_graph.add_edge(
                    (node_map[from_id], node_map[to_id]),
                    traveltime_from_distance_map(dist_map),
                );
            }
        }
    }

    // Insert inverse edges with their weight being the traveltime between each other.
    for (to_id, neighbor_nodes) in neighbors.iter() {
        for (from_id, dist_map) in neighbor_nodes {
            if node_map.contains_key(from_id) && node_map.contains_key(to_id) && from_id != to_id {
                let m_fid = node_map[from_id];
                let m_tid = node_map[to_id];
                if !mapped_graph.has_edge((m_fid, m_tid)) {
                    // TODO: when logger is here this needs to go to errorlog
                    let _ = mapped_graph
                        .add_edge((m_fid, m_tid), traveltime_from_distance_map(dist_map));
                }
            }
        }
    }

    Ok(mapped_graph)
}
