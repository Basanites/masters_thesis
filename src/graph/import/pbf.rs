#![allow(clippy::map_entry)]
use decorum::R64;
use osmpbfreader::objects::Node;
use osmpbfreader::OsmPbfReader;
use osmpbfreader::{NodeId, OsmId, OsmObj};
use std::collections::{BTreeMap, HashSet};
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
fn traveltime_from_distance_map(dist_map: &BTreeMap<String, f64>) -> f64 {
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

fn add_btreemaps(map_a: &BTreeMap<String, f64>, map_b: &BTreeMap<String, f64>) -> BTreeMap<String, f64> {
    let mut new_map = BTreeMap::new();
    for key in map_a.keys().chain(map_b.keys()) {
        if map_a.contains_key(key) && map_b.contains_key(key) {
            new_map.insert(key.into(), map_a[key] + map_b[key]);
        } else if map_a.contains_key(key) {
            new_map.insert(key.into(), map_a[key]);
        } else if map_b.contains_key(key) {
            new_map.insert(key.into(), map_b[key]);
        }
    }
    return new_map
}


/// Contracts all nodes on a single connection path into one endpoint node.
/// The distances for these nodes are updated according to their original distance with many hops in between.
fn contract_nodes(
    nodes: BTreeMap<OsmId, OsmObj>,
    neighbors: BTreeMap<OsmId, BTreeMap<OsmId, BTreeMap<String, f64>>>,
    inv_neighbors: BTreeMap<OsmId, Vec<OsmId>>,
) -> (BTreeMap<OsmId, OsmObj>, BTreeMap<OsmId, BTreeMap<OsmId, BTreeMap<String, f64>>>)
{
    let used_nodes: BTreeMap<OsmId, OsmObj> = nodes.iter().filter(|(id, _)| {
        let ins = neighbors.get(id).map_or(0, |x| x.len());
        let outs =  inv_neighbors.get(id).map_or(0, |x| x.len());
        return !(ins == 1 && outs == 1) && (ins > 0 || outs > 0)
    }).map(|(a, b)| (a.clone(), b.clone())).collect();
    let mut used_neighbors: BTreeMap<OsmId, BTreeMap<OsmId, BTreeMap<String, f64>>> = BTreeMap::new();

    for (node, _) in used_nodes.iter() {
        for (mut neighbor, mut distance_map) in neighbors.get(node).unwrap_or(&BTreeMap::new()).iter() {
            let mut w_temp = distance_map.clone();
            let mut prev = neighbor;                
            while !used_nodes.contains_key(neighbor) {
                prev = neighbor;
                let inner = neighbors.get(neighbor).unwrap().first_key_value().unwrap();
                neighbor = inner.0;
                distance_map = inner.1;
                if prev == neighbor {
                    break
                }
                w_temp = add_btreemaps(&w_temp, distance_map);
                let ind = neighbors.get(neighbor).unwrap().len();
                let outd = inv_neighbors.get(neighbor).unwrap().len();
            }
            if neighbor == node {
                continue
            }
            let mut new_map = BTreeMap::new();
            new_map.insert(*neighbor, w_temp.clone());
            if let Err(_) = used_neighbors.try_insert(*node, new_map) {
                used_neighbors.get_mut(node).unwrap().insert(*neighbor, w_temp);
            }
        }
    }
    
    return (used_nodes, used_neighbors)
}

/// Creates a minimized MatrixGraph from a given pbf file.
/// The nodes are contracted as to not run out of memory for the MatrixGraph.
pub fn import_pbf(
    path: &str,
    nw_gen: &mut dyn FnMut() -> R64,
) -> Result<MatrixGraph<GeoPoint, R64, R64>, ImportError> {
    let file_open = File::open(path);
    let file;
    match file_open {
        Ok(f) => file = f,
        Err(_e) => return Err(ImportError::MissingFile(path.to_string())),
    };

    let mut pbf = OsmPbfReader::new(file);
    let mut neighbors = BTreeMap::<OsmId, BTreeMap<OsmId, BTreeMap<String, f64>>>::new();
    let mut inv_neighbors = BTreeMap::<OsmId, Vec<OsmId>>::new();
    let mut nodes = BTreeMap::<OsmId, OsmObj>::new();
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

                    // insert all the predecessors of a node into the BTreeMap,
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
                            let mut new_dists = BTreeMap::new();
                            new_dists.insert(road_type, distance);
                            neighbor_dists.insert(n_key, new_dists);
                        }
                    } else {
                        let mut outer_map = BTreeMap::new();
                        let mut inner_map = BTreeMap::new();
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
            neighbors.insert(*id, BTreeMap::new());
        }
        if !inv_neighbors.contains_key(id) {
            inv_neighbors.insert(*id, [].to_vec());
        }
    }

    // contract all nodes on single connection paths into one
    let (nodes, neighbors) = contract_nodes(nodes, neighbors, inv_neighbors);

    // Map node ids from osm to consecutive ids starting at 0
    let mut node_map: BTreeMap<OsmId, GeoPoint> = BTreeMap::new();
    for (id, obj) in nodes.iter() {
        if !node_map.contains_key(id) {
            let point = GeoPoint::from_micro_degrees(
                obj.node().unwrap().decimicro_lat,
                obj.node().unwrap().decimicro_lon,
            );
            node_map.insert(*id, point);
        }
    }

    let mut mapped_graph = MatrixGraph::<GeoPoint, R64, R64>::with_size(node_map.len());

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
                    R64::from_inner(traveltime_from_distance_map(dist_map)),
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
                    let _ = mapped_graph.add_edge(
                        (m_fid, m_tid),
                        R64::from_inner(traveltime_from_distance_map(dist_map)),
                    );
                }
            }
        }
    }

    println!(
        "The final graph has {} nodes and {} edges",
        mapped_graph.order(),
        mapped_graph.size()
    );

    Ok(mapped_graph)
}
