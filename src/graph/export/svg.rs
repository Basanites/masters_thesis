use crate::graph::WeightedGraph;
use crate::util::{scale::PointScaler, Point};

use std::fs::File;
use std::io::prelude::*;
use tera::Context;
use tera::Tera;

pub struct SVG {
    pub width: usize,
    pub height: usize,
    pub padding: usize,
}

impl SVG {
    fn scaled_point(&self, point: &Point, scaler: &PointScaler) -> Point {
        let scaled_point = scaler.scale_point(point);

        // The scaled point needs to be adjusted to our SVG canvas size and padding.
        Point {
            x: (scaled_point.x * self.width as f64) + self.padding as f64,
            y: (scaled_point.y * (self.height as f64 * -1.0)) + (self.padding + self.height) as f64,
        }
    }

    pub fn export_coordinate_graph<Nw, Ew>(
        &self,
        graph: &dyn WeightedGraph<(Point, Nw), Ew>,
        name: &str,
    ) -> String {
        let mut context = Context::new();

        context.insert("name", &name);
        context.insert("width", &self.width);
        context.insert("height", &self.height);
        context.insert("padding", &self.padding);

        let point_iter = graph.iter_nodes().map(|(_, weight)| weight.0);
        let scaler = PointScaler::from_point_iterator(point_iter);

        let nodes: Vec<(Point, &str)> = graph
            .iter_nodes()
            .map(|(_, weight)| (self.scaled_point(&weight.0, &scaler), "black"))
            .collect();
        // let nodes = Vec::<(Point, &str)>::new();

        let paths: Vec<(String, &str)> = graph
            .iter_edge_ids()
            .map(|(f_id, t_id)| {
                let p1 = self.scaled_point(&graph.node_weight(f_id).unwrap().0, &scaler);
                let p2 = self.scaled_point(&graph.node_weight(t_id).unwrap().0, &scaler);
                (format!("M {} {} L {} {}", p1.x, p1.y, p2.x, p2.y), "black")
            })
            .collect();

        context.insert("points", &nodes);
        context.insert("paths", &paths);

        let mut reader = File::open("src/templates/graph.svg").unwrap();
        let mut template = String::new();
        reader.read_to_string(&mut template).unwrap();
        Tera::one_off(&template, &context, true).expect("Could not draw graph")
    }
}
