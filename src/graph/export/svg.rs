use super::super::WeightedGraph;
use super::Export;
use crate::util::Point;

use tera::Context;
use tera::Tera;
use std::fs::File;
use std::io::prelude::*;

pub struct SVG {
    pub width: usize,
    pub height: usize,
    pub padding: usize,
}

impl SVG {
    fn scaled_point(&self, point: &Point, max_x: f64, max_y: f64, min_x: f64, min_y: f64) -> Point {
        Point {
            x: ((point.x - min_x) / (max_x - min_x) * self.width as f64) + self.padding as f64,
            y: ((point.y - min_y) / (max_y - min_y) * (self.height as f64 * -1.0)) + (self.padding + self.height) as f64,
        }
    }

    pub fn export_coordinate_graph<Nw, Ew>(&self, graph: &dyn WeightedGraph<(Point, Nw), Ew>, name: &str) -> String {
        let mut context = Context::new();

        context.insert("name", &name);
        context.insert("width", &self.width);
        context.insert("height", &self.height);
        context.insert("padding", &self.padding);

        let extremes = graph.nodes()
            .iter()
            .map(|&id| graph.node_weight(id).unwrap().0)
            .fold((0., 0., f64::MAX, f64::MAX), |acc, point| (f64::max(acc.0, point.x), f64::max(acc.1, point.y), f64::min(acc.2, point.x), f64::min(acc.3, point.y)));

        // let nodes: Vec<(Point, &str)> = graph.nodes()
        //     .iter()
        //     .map(|&id| (self.scaled_point(&graph.node_weight(id).unwrap().0, extremes.0, extremes.1, extremes.2, extremes.3), "black"))
        //     .collect();
        let nodes = Vec::<(Point, &str)>::new();

        let paths: Vec<(String, &str)> = graph.edges()
            .iter()
            .map(|&(f_id, t_id)| {
                let p1 = self.scaled_point(&graph.node_weight(f_id).unwrap().0, extremes.0, extremes.1, extremes.2, extremes.3);
                let p2 = self.scaled_point(&graph.node_weight(t_id).unwrap().0, extremes.0, extremes.1, extremes.2, extremes.3);
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
