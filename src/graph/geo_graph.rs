use super::{ MatrixGraph };
use crate::util::Point;

use std::collections::HashMap;

#[derive(Debug)]
pub struct GeoGraph<Nw, Ew> {
    point_map: HashMap<Point, usize>,
    graph: MatrixGraph<Nw, Ew>
}

#[allow(dead_code)]
impl<Nw, Ew> GeoGraph<Nw, Ew> {
    pub fn new() {}
}

// PartialEq somehow can't be implemented this way, although MatrixGraph derives PartialEq
// impl<Nw, Ew> PartialEq for GeoGraph<Nw, Ew> {
//     fn eq(&self, other:&Self) -> bool {
//         self.graph == other.graph
//     }
// }
