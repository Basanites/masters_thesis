use std::fs::File;
use osmpbfreader::OsmPbfReader;
use std::collections::HashMap;

mod graph;
use graph::{MatrixGraph, WeightedGraph};
use graph::generate::{Grid, Generate};
use graph::export::{Dot, Export, SVG};
use graph::export::svg::Point;


fn main() -> std::io::Result<()> {
    // let nw_gen = || 1;
    // let ew_gen = || 2;
    // let gen = Grid::new((2, 2), &nw_gen, &ew_gen);
    // let grid = gen.generate();
    // println!("{}", Dot::from_usize_weighted_graph(grid.as_ref(), "test"));

    let reader = File::open("res/sachsen-latest.osm.pbf")?;
    let mut pbf = OsmPbfReader::new(reader);
    let filter = ["motorway", "trunk", "primary",
        "secondary", "tertiary", "unclassified",
        "residential", "motorway_link", "trunk_link",
        "primary_link", "secondary_link", "tertiary_link", "living_street"];
    let boundingbox = ((11.9579, 51.2318), (12.8217, 51.4544));
    

    let objs = pbf.get_objs_and_deps(|obj| {
        obj.is_way() &&
        filter.iter().any(|&key| obj.tags().contains_key(key)) &&
        obj.node().map_or(true, |node| {
            node.lat() >= (boundingbox.0).0 &&
            node.lat() <= (boundingbox.1).0 &&
            node.lon() >= (boundingbox.0).1 &&
            node.lon() <= (boundingbox.1).1
        })
    }).unwrap();

    // Map node ids from osm to consecutive ids starting at 0
    let mut node_map: HashMap<osmpbfreader::NodeId, usize> = HashMap::new();
    let mut counter = 0;
    for (id, obj) in &objs {
        if obj.is_node() {
            node_map.insert(id.node().unwrap(), counter);
            counter += 1;
        }
    }

    let mut mapped_graph = MatrixGraph::<(Point, usize), usize>::with_size(counter);

    // Insert nodes into the graph with fixed weight 1
    for (id, obj) in &objs {
        if obj.is_node() {
            let pos = Point {
                x: obj.node().unwrap().lat(),
                y: obj.node().unwrap().lon()
            };
            mapped_graph.add_node(node_map[&id.node().unwrap()], (pos, 1)).ok();
        }
    }

    // Insert edges into the graph with fixed weight 1
    for (id, obj) in &objs {
        if obj.is_way() {
            let mut prev_id = 0;
            for (i, node) in obj.way().unwrap().nodes.iter().enumerate() {
                let new_id = node_map[node];
                if i != 0 {
                    mapped_graph.add_edge((prev_id, new_id), 1).ok();
                }
                prev_id = new_id;
            }
        }
    }

    // println!("{:?}", mapped_graph.nodes());
    // for node in mapped_graph.nodes() {
    //     println!("\t{:?}", mapped_graph.node_weight(node));
    // }

    // println!("{:?}", mapped_graph.edges());
    // println!("{:?}", mapped_graph.size());
    // println!("{:?}\n", mapped_graph.order());

    let svg_exporter = SVG {
        width: 2000,
        height: 1000,
        padding: 50,
    };

    println!("{}", svg_exporter.from_coordinate_graph(&mapped_graph as &dyn WeightedGraph<(Point, usize), usize>, "Leipzig"));

    Ok(())
}
