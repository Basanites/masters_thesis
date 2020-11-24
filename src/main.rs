use std::fs::File;
use osmpbfreader::OsmPbfReader;
use osmpbfreader::objects::Node;
use osmpbfreader::{OsmId, OsmObj, NodeId};
use std::collections::{HashMap, BTreeMap};

mod graph;
use graph::{MatrixGraph, WeightedGraph, GraphError};
use graph::generate::{Grid, Generate};
use graph::export::{Dot, Export, SVG};
use graph::export::svg::Point;

struct Boundingbox {
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
}

fn in_boundingbox(node: &Node, boundingbox: &Boundingbox) -> bool {
    node.lat() >= boundingbox.min_lat &&
    node.lat() <= boundingbox.max_lat &&
    node.lon() >= boundingbox.min_lon &&
    node.lon() <= boundingbox.max_lon

}

fn main() -> std::io::Result<()> {
    // let nw_gen = || 1;
    // let ew_gen = || 2;
    // let gen = Grid::new((2, 2), &nw_gen, &ew_gen);
    // let grid = gen.generate();
    // println!("{}", Dot::from_usize_weighted_graph(grid.as_ref(), "test"));

    let mut pbf = OsmPbfReader::new(File::open("res/sachsen-latest.osm.pbf")?);
    // let filter = ["primary", "secondary", "tertiary", "residential"];
    let filter = ["motorway", "trunk", "primary",
        "secondary", "tertiary", "unclassified",
        "residential", "motorway_link", "trunk_link",
        "primary_link", "secondary_link", "tertiary_link", "living_street"];
    // let boundingbox = Boundingbox {
    //     min_lat: 51.2937,
    //     max_lat: 51.3888,
    //     min_lon: 12.3042,
    //     max_lon: 12.4704
    // };
    let boundingbox = Boundingbox {
        min_lat: 8.5,
        max_lat: 8.7,
        min_lon: 53.35,
        max_lon: 53.5
    };


    // let objs = pbf.iter()
    //     .map(|obj| obj.unwrap())
    //     .filter(|obj| {
    //         obj.is_way() &&
    //         filter.iter().any(|&key| obj.tags().contains("highway", key)) ||
    //         // or it is a node that fits the given boundingbox
    //         obj.node().map_or(false, |node| in_boundingbox(node, &boundingbox))
    //     })
    //     .collect();

    let nodes = pbf.get_objs_and_deps(|obj| obj.node().map_or(false, |node| in_boundingbox(node, &boundingbox))).unwrap();
    // let mut needed_nodes = BTreeMap::<OsmId, OsmObj>::new();
    // let mut pbf = OsmPbfReader::new(File::open("res/sachsen-latest.osm.pbf")?);
    // for obj in pbf.iter() {
    //     let obj = obj.unwrap();
    //     if obj.is_way() && filter.iter().any(|&key| obj.tags().contains("highway", key)) {
    //         for &nid in obj.way().unwrap().nodes.iter() {
    //             let key = OsmId::Node(nid);
    //             if nodes.contains_key(&key) && !needed_nodes.contains_key(&key) {
    //                 needed_nodes.insert(key, nodes[&key].clone());
    //             }
    //         }
    //     }
    // }

    let mut neighbors = HashMap::<OsmId, Vec<OsmId>>::new();
    let mut pbf = OsmPbfReader::new(File::open("res/bremen-latest.osm.pbf")?);
    for obj in pbf.iter() {
        let obj = obj.unwrap();
        if obj.is_way() && filter.iter().any(|&key| obj.tags().contains("highway", key)) {
            let mut pid = NodeId(0);
            for (i, &nid) in obj.way().unwrap().nodes.iter().enumerate() {
                if i > 0 {
                    let key = OsmId::Node(nid);
                    // insert all the neighbors of a node into the hashmap, creating a new vec of neighbors, if there wasnt one before
                    if nodes.contains_key(&key) {
                        if neighbors.contains_key(&key) {
                            neighbors.get_mut(&key).unwrap().push(OsmId::Node(pid))
                        } else {
                            neighbors.insert(key, [OsmId::Node(pid)].to_vec());
                        }
                    }
                }
                pid = nid;
            }
        }
    }

    let mut neighbors_clone = neighbors.clone();
    for (nid, neighbor_nodes) in neighbors_clone.iter() {
        if neighbor_nodes.len() == 1 {
            for (id, oneighbors) in neighbors_clone.iter() {
                if oneighbors.contains(nid) && neighbors.contains_key(id) {
                    // replace the node with its successor
                    neighbors.get_mut(id).unwrap().retain(|oid| oid != nid);
                    neighbors.get_mut(id).unwrap().push(neighbor_nodes[0]);
                }
            }

            // this node is no longer needed
            neighbors.remove(nid);
        }
    }

    print!("We need {} nodes", neighbors.len());

    // Map node ids from osm to consecutive ids starting at 0
    let mut node_map: HashMap<OsmId, usize> = HashMap::new();
    let mut counter = 0;
    for (id, _) in &neighbors {
        let obj = &nodes[id];
        if obj.is_node() {
            node_map.insert(*id, counter);
            counter += 1;
        }
    }

    let mut mapped_graph = MatrixGraph::<(Point, usize), usize>::with_size(counter);

    // Insert nodes into the graph with fixed weight 1
    for (id, &i) in &node_map {
        let obj = &nodes[id];
        let pos = Point {
            x: obj.node().unwrap().lat(),
            y: obj.node().unwrap().lon()
        };
        mapped_graph.add_node(i, (pos, 1)).ok();
    }

    // let mut pbf = OsmPbfReader::new(File::open("res/sachsen-latest.osm.pbf")?);
    // // Insert edges into the graph with fixed weight 1
    // for obj in pbf.iter() {
    //     let obj = obj.unwrap();
    //     if obj.is_way() {
    //         let mut prev_id = 0;
    //         for (i, node) in obj.way().unwrap().nodes.iter().enumerate() {
    //             let new_id = node_map[node];
    //             if i != 0 {
    //                 match mapped_graph.add_edge((prev_id, new_id), 1) {
    //                     _ => {}
    //                 }
    //             }
    //             prev_id = new_id;
    //         }
    //     }
    // }

    for (nid, neighbor_nodes) in neighbors.iter() {
        for neighbor in neighbor_nodes {
            mapped_graph.add_edge((node_map[nid], node_map[neighbor]), 1).ok();
        }
    }

    // println!("{:?}", mapped_graph.nodes());
    // for node in mapped_graph.nodes() {
    //     println!("\t{:?}", mapped_graph.node_weight(node));
    // }

    println!("{:?}", mapped_graph.edges());
    println!("{:?}", mapped_graph.size());
    println!("{:?}\n", mapped_graph.order());

    let svg_exporter = SVG {
        width: 2000,
        height: 1000,
        padding: 50,
    };

    // println!("{}", svg_exporter.from_coordinate_graph(&mapped_graph as &dyn WeightedGraph<(Point, usize), usize>, "Leipzig"));

    Ok(())
}
