use std::fs::File;
use std::io::Write;
use osmpbfreader::OsmPbfReader;
use osmpbfreader::objects::Node;
use osmpbfreader::{OsmId, OsmObj, NodeId};
use std::collections::{HashMap, BTreeMap, HashSet};
use std::time::SystemTime;

mod graph;
use graph::{MatrixGraph, WeightedGraph, GraphError};
use graph::generate::{Grid, Generate};
use graph::export::{Dot, Export, SVG};
use graph::export::svg::Point;
use tera::ast::ExprVal::Array;

fn find_contractable_nodes(neighbors: &HashMap<OsmId, Vec<OsmId>>,
                           inv_neighbors: &HashMap<OsmId, Vec<OsmId>>,
                           replacement_map: &mut HashMap<OsmId, OsmId>,
                           circle_nodes: &mut HashSet<OsmId>) -> bool {
    let mut changed = false;

    let mut change_count = 0;
    for (&nid, neighbor_nodes) in neighbors.iter() {
        let start_node = nid;
        let maxits = 500;

        if neighbor_nodes.len() == 1 && inv_neighbors[&nid].len() == 1 {
            let mut replacement = neighbor_nodes[0];
            let mut counter = 0;

            // we iterate as deeply, as our replacement map allows
            while replacement_map.contains_key(&replacement) && counter < maxits {
                replacement = replacement_map[&replacement];

                // if we find a new circle we add all its nodes to the circlenodes
                if replacement == start_node && !circle_nodes.contains(&replacement) {
                    println!("circle detected for node {:?}", replacement);

                    replacement = neighbor_nodes[0];
                    while replacement != start_node {
                        print!(" {:?} to", replacement);
                        circle_nodes.insert(replacement);
                        replacement = replacement_map[&replacement];
                    }
                    println!();
                    circle_nodes.insert(replacement);
                    changed = true;
                    change_count += 1;
                    break
                }
                counter += 1;
            }

            // only insert the replacment if it is actually a new discovery
            if !replacement_map.contains_key(&nid) || replacement_map[&nid] != replacement {
                replacement_map.insert(nid, replacement);
                changed = true;
                change_count += 1;
            }

        // if the node is part of a 2-way-path we can still contract it to the one it goes to
        } else if neighbor_nodes.len() == 2 {
            let mut oid = nid;
            for nid in neighbor_nodes {
                for iid in inv_neighbors[nid].iter() {
                    if iid == &start_node {
                        oid = *iid;
                    }
                }
            }
            // oid has changed, so it is part of such a 2-way situation
            if oid != nid {
                replacement_map.insert(nid, oid);
            }
        }
    }
    println!("\t{:?} nodes have changed in replacement map", change_count);
    return changed;
}

fn contract_nodes(nodes: &mut HashMap<OsmId, OsmObj>,
                  neighbors: &mut HashMap<OsmId, Vec<OsmId>>,
                  inv_neighbors: HashMap<OsmId, Vec<OsmId>>) {
    let mut replacement_map: HashMap<OsmId, OsmId> = HashMap::new();
    let mut circle_nodes = HashSet::<OsmId>::new();
    let mut changed = true;
    // find replacements as long, as the replacement map keeps changing
    let mut i = 0;
    while changed {
        println!("contraction iteration {:?} with {:?} nodes to replace", i, replacement_map.len());
        changed = find_contractable_nodes(neighbors, &inv_neighbors, &mut replacement_map, &mut circle_nodes); // assignment with side effects == very bad style
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
        for node in neighbor_nodes.iter_mut() {
            if let Some(x) = replacement_map.get(node) {
                *node = *x;
            }
        }
    }

    // remove all nodes which got replaced
    for (from, _) in replacement_map.iter() {
        neighbors.remove(from);
        nodes.remove(from);
    }

    println!("The replacement map contains {} items after removing {:?} nodes in a circle", replacement_map.len(), circle_nodes.len());
}

fn main() -> std::io::Result<()> {
    let mut pbf = OsmPbfReader::new(File::open("res/Leipzig_rough_center.osm.pbf")?);
    let mut neighbors = HashMap::<OsmId, Vec<OsmId>>::new();
    let mut inv_neighbors = HashMap::<OsmId, Vec<OsmId>>::new();
    let mut nodes = HashMap::<OsmId, OsmObj>::new();
    // read all nodes from the pbf to their respective lists.
    // neighbors contain all successors of a node while inv_neighbors contains its predecessors.
    for obj in pbf.iter() {
        let obj = obj.unwrap();
        if obj.is_node() {
            nodes.insert(obj.id(), obj);
        }
        else if obj.is_way() {
            let mut pid = NodeId(0);
            for (i, &nid) in obj.way().unwrap().nodes.iter().enumerate() {
                if i > 0 {
                    let n_key = OsmId::Node(nid);
                    let p_key = OsmId::Node(pid);
                    // insert all the predecessors of a node into the hashmap,
                    // creating a new vec of neighbors, if there wasnt one before
                    if inv_neighbors.contains_key(&n_key) {
                        inv_neighbors.get_mut(&n_key).unwrap().push(p_key);
                    } else {
                        inv_neighbors.insert(n_key, [p_key].to_vec());
                    }

                    // insert all the successors of a node into the hashmap,
                    // creating a new vec of neighbors, if there wasnt one before
                    if neighbors.contains_key(&p_key) {
                        neighbors.get_mut(&p_key).unwrap().push(n_key);
                    } else {
                        neighbors.insert(p_key, [n_key].to_vec());
                    }
                }
                pid = nid;
            }
        }
    }

    for (id, _) in &nodes {
        if !neighbors.contains_key(id) {
            neighbors.insert(*id, [].to_vec());
        }
        if !inv_neighbors.contains_key(id) {
            inv_neighbors.insert(*id, [].to_vec());
        }
    }

    // for (_, ids) in &neighbors {
    //     for id in ids {
    //         if !nodes.contains_key(id) {
    //             println!("nodes does not have id {:?}", id);
    //         }
    //     }
    // }

    println!("Number of neighbors is {:?} with {:?} nodes.", neighbors.len(), nodes.len());

    contract_nodes(&mut nodes, &mut neighbors, inv_neighbors);

    // Map node ids from osm to consecutive ids starting at 0
    let mut node_map: HashMap<OsmId, usize> = HashMap::new();
    let mut counter = 0;
    for (id, _) in &nodes {
        if !node_map.contains_key(id) {
            node_map.insert(*id, counter);
            counter += 1;
        }
    }

    println!("We have {} mapped nodes and {} nodes", node_map.len(), nodes.len());

    let mut mapped_graph = MatrixGraph::<(Point, usize), usize>::with_size(counter);

    // Insert nodes into the graph with fixed weight 1
    for (id, &i) in &node_map {
        let obj = &nodes[id];
        let pos = Point {
            x: obj.node().unwrap().lat(),
            y: obj.node().unwrap().lon()
        };
        mapped_graph.add_node(i, (pos, 1));
    }

    for (from_id, neighbor_nodes) in neighbors.iter() {
        for to_id in neighbor_nodes {
            if node_map.contains_key(from_id) && node_map.contains_key(to_id) && from_id != to_id {
                mapped_graph.add_edge((node_map[from_id], node_map[to_id]), 1);
            }
        }
    }

    println!("{:?}", mapped_graph.edges());
    println!("{:?}", mapped_graph.size());
    println!("{:?}", mapped_graph.nodes());
    println!("{:?}\n", mapped_graph.order());

    let svg_exporter = SVG {
        width: 2000,
        height: 1000,
        padding: 50,
    };

    // println!("{}", svg_exporter.from_coordinate_graph(&mapped_graph as &dyn WeightedGraph<(Point, usize), usize>, "Leipzig"));
    let mut out_file = File::create("graph_out.svg").expect("Error creating file");
    out_file.write_all(svg_exporter.from_coordinate_graph(&mapped_graph as &dyn WeightedGraph<(Point, usize), usize>, "Leipzig").as_bytes()).expect("Error writing to file");

    Ok(())
}
