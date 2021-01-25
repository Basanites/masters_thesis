use float_cmp::approx_eq;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        approx_eq!(f64, self.x, other.x) && approx_eq!(f64, self.y, other.y)
    }
}

impl Eq for Point {}
